//! Core error type. Platform adapters map this to their own error models.
//!
//! Every error carries three things for the boundary:
//! - `category()`: a coarse bucket the adapter maps onto its own error enum.
//! - `code()`: a stable, machine-readable identifier the frontend localises.
//! - `detail()`: an optional, *safe* human-readable context (no secrets).

use thiserror::Error;

/// Coarse error category, mapped 1:1 onto adapter error enums.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    #[error("sync error: {0}")]
    Sync(String),

    #[error("unknown error: {0}")]
    Unknown(String),

    /// A known failure identified by a stable, localisable code. `detail` is an
    /// optional safe context — it must never contain secrets.
    #[error("{code}")]
    Coded {
        category: ErrorCategory,
        code: &'static str,
        detail: Option<String>,
    },
}

impl CoreError {
    pub fn unknown<E: std::fmt::Display>(err: E) -> Self {
        Self::Unknown(err.to_string())
    }

    /// Construct a localisable error carrying a stable code.
    pub fn coded(category: ErrorCategory, code: &'static str, detail: Option<String>) -> Self {
        Self::Coded {
            category,
            code,
            detail,
        }
    }

    /// Coarse category, for adapter/FRB mapping.
    pub fn category(&self) -> ErrorCategory {
        match self {
            CoreError::Db(_) => ErrorCategory::Db,
            CoreError::Network(_) => ErrorCategory::Network,
            CoreError::Parse(_) => ErrorCategory::Parse,
            CoreError::InvalidInput(_) => ErrorCategory::InvalidInput,
            CoreError::NotFound(_) => ErrorCategory::NotFound,
            CoreError::Ai(_) => ErrorCategory::Ai,
            CoreError::Platform(_) => ErrorCategory::Platform,
            CoreError::Sync(_) => ErrorCategory::Sync,
            CoreError::Unknown(_) => ErrorCategory::Unknown,
            CoreError::Coded { category, .. } => *category,
        }
    }

    /// Stable machine-readable code, used by the frontend for localisation.
    pub fn code(&self) -> &'static str {
        match self {
            CoreError::Db(_) => "db",
            CoreError::Network(_) => "network",
            CoreError::Parse(_) => "parse",
            CoreError::InvalidInput(_) => "invalidInput",
            CoreError::NotFound(_) => "notFound",
            CoreError::Ai(_) => "ai",
            CoreError::Platform(_) => "platform",
            CoreError::Sync(_) => "sync",
            CoreError::Unknown(_) => "unknown",
            CoreError::Coded { code, .. } => code,
        }
    }

    /// Optional safe context, redacted of credential-looking material.
    pub fn detail(&self) -> Option<String> {
        match self {
            CoreError::Db(s)
            | CoreError::Network(s)
            | CoreError::Parse(s)
            | CoreError::InvalidInput(s)
            | CoreError::NotFound(s)
            | CoreError::Ai(s)
            | CoreError::Platform(s)
            | CoreError::Sync(s)
            | CoreError::Unknown(s) => Some(redact(s)),
            CoreError::Coded { detail, .. } => detail.as_deref().map(redact),
        }
    }
}

/// Best-effort scrubbing of credential-looking values before a message crosses
/// an adapter boundary. This is defensive: the primary guarantee is that
/// secrets never enter error text. It masks the obvious `key=value`,
/// `key: value` and `Bearer <token>` shapes that dependency errors can leak.
fn redact(input: &str) -> String {
    const SENSITIVE: &[&str] = &[
        "password",
        "passwd",
        "secret",
        "token",
        "apikey",
        "api_key",
        "authorization",
        "bearer",
    ];

    let lower = input.to_lowercase();
    let bytes = input.as_bytes();
    let len = input.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Longest sensitive-key match at i, respecting word boundaries.
        let mut hit: Option<usize> = None; // end index
        for key in SENSITIVE {
            if lower[i..].starts_with(key) {
                let end = i + key.len();
                let prev_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
                let next_ok = end >= len || !bytes[end].is_ascii_alphanumeric();
                if prev_ok && next_ok && hit.map_or(true, |e| end > e) {
                    hit = Some(end);
                }
            }
        }

        match hit {
            Some(end) => {
                out.push_str(&input[i..end]); // the key, verbatim
                let mut j = end;
                // Consume separators (kept verbatim).
                while j < len {
                    let b = bytes[j];
                    if b == b'=' || b == b':' || b == b' ' || b == b'\t' {
                        out.push(b as char);
                        j += 1;
                    } else {
                        break;
                    }
                }
                // Mask the value token that follows a separator.
                if j > end {
                    let start = j;
                    while j < len {
                        let b = bytes[j];
                        if b == b' ' || b == b',' || b == b'}' || b == b'"' || b == b'\'' {
                            break;
                        }
                        j += 1;
                    }
                    if j > start {
                        out.push_str("[REDACTED]");
                        i = j;
                        continue;
                    }
                }
                i = end;
            }
            None => {
                let ch = input[i..].chars().next().unwrap();
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coded_error_exposes_stable_code() {
        let e = CoreError::coded(
            ErrorCategory::InvalidInput,
            "feedAlreadyExists",
            Some("https://example.com/feed".to_string()),
        );
        assert_eq!(e.category(), ErrorCategory::InvalidInput);
        assert_eq!(e.code(), "feedAlreadyExists");
        assert_eq!(e.detail().as_deref(), Some("https://example.com/feed"));
    }

    #[test]
    fn default_variants_have_default_codes() {
        assert_eq!(CoreError::Network("x".into()).code(), "network");
        assert_eq!(CoreError::NotFound("x".into()).code(), "notFound");
        assert_eq!(CoreError::Db("x".into()).category(), ErrorCategory::Db);
    }

    #[test]
    fn redact_masks_credential_values() {
        assert_eq!(redact("token=abc123"), "token=[REDACTED]");
        assert_eq!(redact("password: hunter2"), "password: [REDACTED]");
        assert_eq!(redact("Bearer abc123"), "Bearer [REDACTED]");
        // Unrelated text passes through untouched.
        assert_eq!(
            redact("network timeout after 30s"),
            "network timeout after 30s"
        );
    }
}
