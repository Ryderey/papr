use std::sync::Arc;

use crate::db::{Db, FeedRefreshInfo};
use crate::dto::{Feed, RefreshError, RefreshOptions, RefreshReport, SourceType};
use crate::error::{CoreError, ErrorCategory};
use crate::ingestion;
use crate::ingestion::parse::ParsedFeed;

pub struct IngestionService {
    db: Arc<Db>,
    http: Arc<reqwest::Client>,
}

impl IngestionService {
    pub fn new(db: Arc<Db>, http: Arc<reqwest::Client>) -> Self {
        Self { db, http }
    }

    /// Add a subscription from a URL/query: source normalization, (page) feed
    /// discovery, first fetch, parse, classification, then persist feed + articles.
    pub async fn add_feed(&self, input: String) -> Result<Feed, CoreError> {
        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "emptyFeedUrl",
                None,
            ));
        }

        let client = self.http.as_ref();

        // Resolve to a concrete feed URL + initial source type.
        let (feed_url, source_type) = resolve_input(&trimmed, client).await?;

        // Dedup by feed_url.
        let db = Arc::clone(&self.db);
        let dedup_url = feed_url.clone();
        let already_exists = tokio::task::spawn_blocking(move || db.find_feed_by_url(&dedup_url))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))??;
        if already_exists.is_some() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "feedAlreadyExists",
                Some(feed_url),
            ));
        }

        // Fetch + parse, discovering a feed URL from an HTML page if needed.
        let (feed_url, parsed, source_type) =
            fetch_and_parse(client, &feed_url, source_type).await?;

        // Persist the feed, articles, enclosures, FTS rows, and feed change log
        // in one transaction on the blocking pool.
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || {
            let title = parsed.title.unwrap_or_else(|| feed_url.clone());
            let feed_id = db.insert_feed_with_articles(
                &feed_url,
                parsed.site_url.as_deref(),
                &title,
                parsed.description.as_deref(),
                source_type,
                None,
                &parsed.articles,
            )?;
            db.get_feed(feed_id)
        })
        .await
        .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn refresh_feeds(&self, options: RefreshOptions) -> Result<RefreshReport, CoreError> {
        let feeds = {
            let db = Arc::clone(&self.db);
            tokio::task::block_in_place(move || db.feeds_to_refresh())?
        };

        let feeds: Vec<_> = match options.feed_ids {
            Some(ids) => {
                let id_set: std::collections::HashSet<i64> = ids.into_iter().collect();
                feeds
                    .into_iter()
                    .filter(|f| id_set.contains(&f.id))
                    .collect()
            }
            None => feeds,
        };

        let total_feeds = feeds.len() as i64;
        if total_feeds == 0 {
            return Ok(RefreshReport {
                total_feeds: 0,
                new_articles: 0,
                errors: Vec::new(),
            });
        }

        let client = self.http.as_ref();

        let mut new_articles: i64 = 0;
        let mut errors: Vec<RefreshError> = Vec::new();

        for FeedRefreshInfo {
            id,
            feed_url,
            etag,
            last_modified,
        } in feeds
        {
            match refresh_one(
                &self.db,
                &client,
                id,
                &feed_url,
                etag.as_deref(),
                last_modified.as_deref(),
            )
            .await
            {
                Ok(count) => {
                    new_articles += count;
                }
                Err(e) => {
                    let message = format!("{e:#}");
                    let _ = tokio::task::block_in_place({
                        let db = Arc::clone(&self.db);
                        let message = message.clone();
                        move || db.set_feed_fetch_state(id, None, None, Some(&message))
                    });
                    errors.push(RefreshError {
                        feed_id: id,
                        message,
                    });
                }
            }
        }

        Ok(RefreshReport {
            total_feeds,
            new_articles,
            errors,
        })
    }
}

/// Resolve a user-pasted string to a concrete feed URL + initial source type.
async fn resolve_input(
    input: &str,
    client: &reqwest::Client,
) -> Result<(String, SourceType), CoreError> {
    match ingestion::sources::normalize_source(input) {
        ingestion::sources::Normalized::Feed { url, source_type } => Ok((url, source_type)),
        ingestion::sources::Normalized::NeedsYoutubeResolution { page_url } => {
            let (bytes, _, _) = ingestion::fetch::get(client, &page_url).await?;
            let html = ingestion::fetch::decode_html(&bytes, None);
            let channel_id = ingestion::sources::extract_channel_id(&html).ok_or_else(|| {
                CoreError::coded(ErrorCategory::InvalidInput, "feedNotFound", None)
            })?;
            Ok((
                ingestion::sources::youtube_feed_url(&channel_id),
                SourceType::Youtube,
            ))
        }
        ingestion::sources::Normalized::Untouched => {
            if let Some(expanded) = ingestion::sources::expand_rsshub(
                input,
                ingestion::sources::DEFAULT_RSSHUB_INSTANCE,
            ) {
                Ok((expanded, SourceType::Rss))
            } else {
                Ok((
                    ingestion::discovery::normalize_query_url(input),
                    SourceType::Rss,
                ))
            }
        }
    }
}

/// Fetch a feed document, falling back to page feed-discovery when the URL
/// resolves to an HTML page rather than a feed.
async fn fetch_and_parse(
    client: &reqwest::Client,
    feed_url: &str,
    source_type: SourceType,
) -> Result<(String, ParsedFeed, SourceType), CoreError> {
    let mut url = feed_url.to_string();
    loop {
        let fetched = ingestion::fetch::conditional_get(client, &url, None, None).await?;
        let bytes = match fetched {
            ingestion::fetch::Fetched::Body { bytes, .. } => bytes,
            ingestion::fetch::Fetched::NotModified => {
                return Err(CoreError::coded(ErrorCategory::Parse, "feedNotFound", None));
            }
        };
        if ingestion::parse::looks_like_feed(&bytes) {
            let parsed = ingestion::parse::parse_feed(&bytes, &url)?;
            let refined = ingestion::parse::refine_source_type(source_type, &parsed, &url);
            return Ok((url, parsed, refined));
        }
        // HTML page — discover a feed URL and retry.
        let html = ingestion::fetch::decode_html(&bytes, None);
        let discovered = ingestion::parse::discover_feeds(&html, &url);
        match discovered.into_iter().next() {
            Some(u) => url = u,
            None => {
                return Err(CoreError::coded(ErrorCategory::Parse, "feedNotFound", None));
            }
        }
    }
}

async fn refresh_one(
    db: &Arc<Db>,
    client: &reqwest::Client,
    feed_id: i64,
    feed_url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<i64, CoreError> {
    let fetched = ingestion::fetch::conditional_get(client, feed_url, etag, last_modified).await?;

    match fetched {
        ingestion::fetch::Fetched::NotModified => {
            tokio::task::block_in_place({
                let db = Arc::clone(db);
                move || db.touch_feed(feed_id)
            })?;
            Ok(0)
        }
        ingestion::fetch::Fetched::Body {
            bytes,
            etag,
            last_modified,
        } => {
            let text = ingestion::fetch::decode_html(
                &bytes, None, // content_type could be passed here if we stored it
            );
            let parsed = ingestion::parse::parse_feed(text.as_bytes(), feed_url)?;

            let mut inserted: i64 = 0;
            for article in &parsed.articles {
                let was_inserted = tokio::task::block_in_place({
                    let db = Arc::clone(db);
                    let article = article.clone();
                    move || db.upsert_article(feed_id, &article)
                })?;
                if was_inserted {
                    inserted += 1;
                }
            }

            let title = parsed.title.as_deref();
            let site_url = parsed.site_url.as_deref();
            let description = parsed.description.as_deref();
            let icon = parsed.icon.as_deref();
            let etag2 = etag.as_deref();
            let last_modified2 = last_modified.as_deref();
            tokio::task::block_in_place({
                let db = Arc::clone(db);
                move || {
                    db.update_feed_meta(feed_id, title, site_url, description, icon)?;
                    db.set_feed_fetch_state(feed_id, etag2, last_modified2, None)?;
                    db.touch_feed(feed_id)?;
                    Ok::<(), CoreError>(())
                }
            })?;

            Ok(inserted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn refresh_report_counts_feeds_and_keeps_db_consistent() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&tmp.path().join("test.db")).unwrap());
        let http = Arc::new(crate::ingestion::fetch::build_client(30, "system", None).unwrap());
        let svc = IngestionService::new(db.clone(), http);

        let feed_url = "https://example.com/feed";
        db.add_feed(feed_url).unwrap();

        let report = svc.refresh_feeds(RefreshOptions::default()).await.unwrap();
        assert_eq!(report.total_feeds, 1);
        assert!(!report.errors.is_empty());

        let feed = db.get_feed(1).unwrap();
        assert!(feed.fetch_error.is_some());
    }
}
