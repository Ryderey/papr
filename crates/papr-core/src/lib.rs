//! Papr core: platform-agnostic RSS business logic.

pub mod config;
pub use config::{PaprCoreConfig, Platform};
pub mod db;
pub mod dto;
pub use dto::*;
pub mod error;
pub mod services;

use std::sync::Arc;

use db::Db;
use error::CoreError;
use services::{
    ArticleService, FeedService, IngestionService, OpmlService, SettingsService,
};

/// The root handle for all Papr business operations.
///
/// `PaprCore` is intentionally free of Tauri, Flutter, or other platform
/// dependencies. Adapters construct it with platform-specific paths and then
/// call its services.
pub struct PaprCore {
    db: Arc<Db>,
    config: PaprCoreConfig,
    feed_service: FeedService,
    article_service: ArticleService,
    ingestion_service: IngestionService,
    opml_service: OpmlService,
    settings_service: SettingsService,
}

impl PaprCore {
    /// Initialise a new `PaprCore` instance.
    ///
    /// Validates paths, creates missing directories, opens the SQLite database,
    /// runs migrations, and constructs all services.
    pub async fn new(config: PaprCoreConfig) -> Result<Self, CoreError> {
        config.validate()?;

        let db_path = &config.database_path;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CoreError::Platform(format!(
                    "failed to create data directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        let db = Arc::new(Db::new(db_path)?);

        Ok(Self {
            feed_service: FeedService::new(Arc::clone(&db)),
            article_service: ArticleService::new(Arc::clone(&db)),
            ingestion_service: IngestionService::new(Arc::clone(&db)),
            opml_service: OpmlService::new(Arc::clone(&db)),
            settings_service: SettingsService::new(Arc::clone(&db)),
            db,
            config,
        })
    }

    pub fn config(&self) -> &PaprCoreConfig {
        &self.config
    }

    pub fn platform(&self) -> Platform {
        self.config.platform
    }

    pub fn feed_service(&self) -> &FeedService {
        &self.feed_service
    }

    pub fn article_service(&self) -> &ArticleService {
        &self.article_service
    }

    pub fn ingestion_service(&self) -> &IngestionService {
        &self.ingestion_service
    }

    pub fn opml_service(&self) -> &OpmlService {
        &self.opml_service
    }

    pub fn settings_service(&self) -> &SettingsService {
        &self.settings_service
    }
}
