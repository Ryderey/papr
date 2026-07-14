//! Core error type. Platform adapters map this to their own error models.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("database error: {0}")]
    Db(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("AI error: {0}")]
    Ai(String),

    #[error("platform error: {0}")]
    Platform(String),

    #[error("unknown error: {0}")]
    Unknown(String),
}

impl CoreError {
    pub fn unknown<E: std::fmt::Display>(err: E) -> Self {
        Self::Unknown(err.to_string())
    }
}
