use flutter_rust_bridge::frb;

/// Coarse error category, mirrored from `papr_core::error::ErrorCategory`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[frb]
pub enum ErrorCategory {
    Db,
    Network,
    Parse,
    InvalidInput,
    NotFound,
    Ai,
    Platform,
    Sync,
    Unknown,
}

impl From<papr_core::error::ErrorCategory> for ErrorCategory {
    fn from(c: papr_core::error::ErrorCategory) -> Self {
        match c {
            papr_core::error::ErrorCategory::Db => Self::Db,
            papr_core::error::ErrorCategory::Network => Self::Network,
            papr_core::error::ErrorCategory::Parse => Self::Parse,
            papr_core::error::ErrorCategory::InvalidInput => Self::InvalidInput,
            papr_core::error::ErrorCategory::NotFound => Self::NotFound,
            papr_core::error::ErrorCategory::Ai => Self::Ai,
            papr_core::error::ErrorCategory::Platform => Self::Platform,
            papr_core::error::ErrorCategory::Sync => Self::Sync,
            papr_core::error::ErrorCategory::Unknown => Self::Unknown,
        }
    }
}

/// Errors returned across the Flutter bridge.
///
/// Carries the three parts the Flutter side needs to localise a failure: a
/// coarse category, a stable machine-readable code, and an optional safe detail
/// (never containing secrets).
#[derive(Debug)]
#[frb]
pub struct PaprBridgeError {
    pub category: ErrorCategory,
    pub code: String,
    pub detail: Option<String>,
}

impl std::fmt::Display for PaprBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {}", self.code, detail),
            None => write!(f, "{}", self.code),
        }
    }
}

impl std::error::Error for PaprBridgeError {}

impl From<papr_core::error::CoreError> for PaprBridgeError {
    fn from(e: papr_core::error::CoreError) -> Self {
        Self {
            category: e.category().into(),
            code: e.code().to_string(),
            detail: e.detail(),
        }
    }
}
