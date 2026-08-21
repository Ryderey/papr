use std::sync::Arc;

use crate::db::Db;
use crate::dto::Folder;
use crate::error::CoreError;

pub struct FolderService {
    db: Arc<Db>,
}

impl FolderService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub async fn list_folders(&self) -> Result<Vec<Folder>, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.list_folders())
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    /// Create a folder, returning the id of the existing folder when a same-name
    /// folder (case-insensitive) is already present.
    pub async fn create_folder(&self, name: String) -> Result<i64, CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.create_folder(&name))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn rename_folder(&self, id: i64, name: String) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.rename_folder(id, &name))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    /// Delete a folder. Its feeds move to uncategorised, never deleted.
    pub async fn delete_folder(&self, id: i64) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.delete_folder(id))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn reorder_folders(&self, folder_ids: Vec<i64>) -> Result<(), CoreError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.reorder_folders(&folder_ids))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }
}
