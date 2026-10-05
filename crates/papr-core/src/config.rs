//! Runtime configuration injected by platform adapters.

use std::path::PathBuf;

use crate::error::CoreError;

/// Platform-independent runtime configuration.
///
/// Adapters are responsible for resolving platform paths and passing them in;
/// `papr-core` never calls platform-specific path APIs.
#[derive(Debug, Clone)]
pub struct PaprCoreConfig {
    /// Absolute path to the directory that holds app data.
    pub data_dir: PathBuf,
    /// Absolute path to the SQLite database file.
    pub database_path: PathBuf,
    /// Optional absolute path to a cache directory.
    pub cache_dir: Option<PathBuf>,
    /// Optional absolute path to a log directory.
    pub log_dir: Option<PathBuf>,
    /// Optional log level, e.g. `"info"` or `"debug"`.
    pub log_level: Option<String>,
    /// HTTP timeout in seconds (clamped to 5..=300). `None` uses the default.
    pub http_timeout_secs: Option<u64>,
    /// Proxy mode: `"system"` (default), `"none"`, or a custom proxy URL.
    pub http_proxy: Option<String>,
    /// Override the default User-Agent header.
    pub http_user_agent: Option<String>,
    /// Target platform; used to gate platform-specific behaviour.
    pub platform: Platform,
}

impl PaprCoreConfig {
    /// Validate that required paths are absolute and writable.
    pub fn validate(&self) -> Result<(), CoreError> {
        if !self.data_dir.is_absolute() {
            return Err(CoreError::InvalidInput(format!(
                "data_dir must be an absolute path: {}",
                self.data_dir.display()
            )));
        }
        if !self.database_path.is_absolute() {
            return Err(CoreError::InvalidInput(format!(
                "database_path must be an absolute path: {}",
                self.database_path.display()
            )));
        }
        Ok(())
    }
}

/// Supported target platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Desktop,
    Android,
    Ios,
}
