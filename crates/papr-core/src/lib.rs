//! Papr core: platform-agnostic RSS business logic.

pub mod config;
pub use config::{PaprCoreConfig, Platform};
pub mod db;
pub mod dto;
pub use dto::*;
pub mod error;
pub mod extraction;
pub mod ingestion;
pub mod opml;
pub mod services;
pub mod translate;

use std::sync::Arc;

use db::Db;
use error::CoreError;
use services::{
    ArticleService, FeedService, FolderService, IngestionService, OpmlService, SettingsService,
};

/// The root handle for all Papr business operations.
///
/// `PaprCore` is intentionally free of Tauri, Flutter, or other platform
/// dependencies. Adapters construct it with platform-specific paths and then
/// call its services.
pub struct PaprCore {
    _db: Arc<Db>,
    http: Arc<reqwest::Client>,
    config: PaprCoreConfig,
    feed_service: FeedService,
    folder_service: FolderService,
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

        let http = Arc::new(crate::ingestion::fetch::build_client(
            config.http_timeout_secs.unwrap_or(30),
            config.http_proxy.as_deref().unwrap_or("system"),
            config.http_user_agent.as_deref(),
        )?);

        Ok(Self {
            feed_service: FeedService::new(Arc::clone(&db)),
            folder_service: FolderService::new(Arc::clone(&db)),
            article_service: ArticleService::new(Arc::clone(&db), Arc::clone(&http)),
            ingestion_service: IngestionService::new(Arc::clone(&db), Arc::clone(&http)),
            opml_service: OpmlService::new(Arc::clone(&db)),
            settings_service: SettingsService::new(Arc::clone(&db)),
            _db: db,
            http,
            config,
        })
    }

    pub fn config(&self) -> &PaprCoreConfig {
        &self.config
    }

    /// Shared HTTP client, configured once at initialisation.
    pub fn http(&self) -> &Arc<reqwest::Client> {
        &self.http
    }

    pub fn platform(&self) -> Platform {
        self.config.platform
    }

    pub fn feed_service(&self) -> &FeedService {
        &self.feed_service
    }

    pub fn folder_service(&self) -> &FolderService {
        &self.folder_service
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
