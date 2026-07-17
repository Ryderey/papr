use std::sync::Arc;

use crate::db::{Db, FeedRefreshInfo};
use crate::dto::{RefreshError, RefreshOptions, RefreshReport};
use crate::error::CoreError;
use crate::ingestion;

pub struct IngestionService {
    db: Arc<Db>,
}

impl IngestionService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub async fn refresh_feeds(
        &self,
        options: RefreshOptions,
    ) -> Result<RefreshReport, CoreError> {
        let feeds = {
            let db = Arc::clone(&self.db);
            tokio::task::block_in_place(move || db.feeds_to_refresh())?
        };

        let feeds: Vec<_> = match options.feed_ids {
            Some(ids) => {
                let id_set: std::collections::HashSet<i64> = ids.into_iter().collect();
                feeds.into_iter().filter(|f| id_set.contains(&f.id)).collect()
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

        let client = ingestion::fetch::build_client(30, "system")?;

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
                &bytes,
                None, // content_type could be passed here if we stored it
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
                    db.set_feed_fetch_state(
                        feed_id,
                        etag2,
                        last_modified2,
                        None,
                    )?;
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
        let svc = IngestionService::new(db.clone());

        let feed_url = "https://example.com/feed";
        db.add_feed(feed_url).unwrap();

        let report = svc.refresh_feeds(RefreshOptions::default()).await.unwrap();
        assert_eq!(report.total_feeds, 1);
        assert!(!report.errors.is_empty());

        let feed = db.get_feed(1).unwrap();
        assert!(feed.fetch_error.is_some());
    }
}
