//! Personal GitHub snapshot synchronization, independent of GReader.

pub mod merge;
pub mod model;
pub mod service;
pub mod storage;
pub mod transport;

use crate::error::{CoreError, ErrorCategory};

pub(crate) fn error(code: &'static str) -> CoreError {
    CoreError::coded(ErrorCategory::Sync, code, None)
}
