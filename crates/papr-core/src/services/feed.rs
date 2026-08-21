use std::sync::Arc;

use crate::db::Db;
use crate::dto::Feed;
use crate::error::{CoreError, ErrorCategory};

pub struct FeedService {
    db: Arc<Db>,
}

impl FeedService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub async fn list_feeds(&self) -> Result<Vec<Feed>, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.list_feeds())
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn add_feed(&self, feed_url: String) -> Result<Feed, CoreError> {
        let url = feed_url.trim().to_string();
        if url.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "emptyFeedUrl",
                None,
            ));
        }
        let db = Arc::clone(&self.db);
        let id = tokio::task::spawn_blocking(move || db.add_feed(&url))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))??;
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.get_feed(id))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn delete_feed(&self, id: i64) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.delete_feed(id))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn rename_feed(&self, id: i64, title: String) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.rename_feed(id, &title))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn move_feed(&self, id: i64, folder_id: Option<i64>) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.move_feed(id, folder_id))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn set_feed_refresh_interval(
        &self,
        id: i64,
        minutes: Option<i64>,
    ) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_feed_refresh_interval(id, minutes))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }
}
