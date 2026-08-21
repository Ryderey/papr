use std::sync::Arc;

use crate::db::Db;
use crate::dto::{OpmlImportReport, SourceType};
use crate::error::CoreError;
use crate::opml;

pub struct OpmlService {
    db: Arc<Db>,
}

impl OpmlService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    /// Import feeds from OPML text, preserving folder structure and never
    /// duplicating an existing subscription.
    pub async fn import_text(&self, opml_text: String) -> Result<OpmlImportReport, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || import_text_blocking(&db, &opml_text))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    /// Export current subscriptions as OPML text (newsletter sources excluded).
    pub async fn export_text(&self) -> Result<String, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || {
            let feeds = db.feeds_for_export()?;
            opml::build(&feeds)
        })
        .await
        .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }
}

fn import_text_blocking(db: &Db, opml_text: &str) -> Result<OpmlImportReport, CoreError> {
    let imported = opml::parse(opml_text)?;
    let mut imported_feeds: i64 = 0;
    let mut failed_feeds: i64 = 0;
    let mut errors: Vec<String> = Vec::new();

    for feed in imported {
        let folder_id = match &feed.folder {
            Some(name) => Some(db.create_folder(name)?),
            None => None,
        };
        // Never re-subscribe an existing feed (dedup by feed_url).
        if db.find_feed_by_url(&feed.feed_url)?.is_some() {
            continue;
        }
        match db.insert_feed(
            &feed.feed_url,
            None,
            &feed.title,
            None,
            SourceType::Rss,
            folder_id,
        ) {
            Ok(_) => imported_feeds += 1,
            Err(e) => {
                failed_feeds += 1;
                errors.push(format!("{}: {}", feed.feed_url, e));
            }
        }
    }

    Ok(OpmlImportReport {
        imported_feeds,
        failed_feeds,
        errors,
    })
}
