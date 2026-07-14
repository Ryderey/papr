use flutter_rust_bridge::frb;

/// Errors returned across the Flutter bridge.
#[derive(Debug)]
#[frb]
pub enum PaprBridgeError {
    Database { message: String },
    Network { message: String },
    Parse { message: String },
    InvalidInput { message: String },
    NotFound { message: String },
    Ai { message: String },
    Platform { message: String },
    Unknown { message: String },
}

impl std::fmt::Display for PaprBridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            PaprBridgeError::Database { message } => message,
            PaprBridgeError::Network { message } => message,
            PaprBridgeError::Parse { message } => message,
            PaprBridgeError::InvalidInput { message } => message,
            PaprBridgeError::NotFound { message } => message,
            PaprBridgeError::Ai { message } => message,
            PaprBridgeError::Platform { message } => message,
            PaprBridgeError::Unknown { message } => message,
        };
        f.write_str(message)
    }
}

impl std::error::Error for PaprBridgeError {}

impl From<papr_core::error::CoreError> for PaprBridgeError {
    fn from(e: papr_core::error::CoreError) -> Self {
        use papr_core::error::CoreError;
        let message = e.to_string();
        match e {
            CoreError::Db(_) => Self::Database { message },
            CoreError::Network(_) => Self::Network { message },
            CoreError::Parse(_) => Self::Parse { message },
            CoreError::InvalidInput(_) => Self::InvalidInput { message },
            CoreError::NotFound(_) => Self::NotFound { message },
            CoreError::Ai(_) => Self::Ai { message },
            CoreError::Platform(_) => Self::Platform { message },
            CoreError::Unknown(_) => Self::Unknown { message },
        }
    }
}
