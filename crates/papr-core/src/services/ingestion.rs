use std::sync::Arc;

use crate::db::Db;
use crate::dto::{RefreshOptions, RefreshReport};
use crate::error::CoreError;

pub struct IngestionService {
    db: Arc<Db>,
}

impl IngestionService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub async fn refresh_feeds(
        &self,
        _options: RefreshOptions,
    ) -> Result<RefreshReport, CoreError> {
        // Phase 1: stub. Real implementation will fetch feeds, parse entries,
        // and upsert articles.
        Ok(RefreshReport {
            total_feeds: 0,
            new_articles: 0,
            errors: Vec::new(),
        })
    }
}
