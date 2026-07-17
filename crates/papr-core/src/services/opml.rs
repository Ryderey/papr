use std::sync::Arc;

use crate::db::Db;
use crate::dto::OpmlImportReport;
use crate::error::CoreError;

pub struct OpmlService {
    _db: Arc<Db>,
}

impl OpmlService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { _db: db }
    }

    pub async fn import_text(
        &self,
        _opml_text: String,
    ) -> Result<OpmlImportReport, CoreError> {
        // Phase 1: stub. Real implementation will parse OPML and insert feeds.
        Ok(OpmlImportReport {
            imported_feeds: 0,
            failed_feeds: 0,
            errors: Vec::new(),
        })
    }
}
