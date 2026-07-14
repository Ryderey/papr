use std::sync::Arc;

use crate::db::Db;
use crate::dto::SettingsSnapshot;
use crate::error::CoreError;

pub struct SettingsService {
    db: Arc<Db>,
}

impl SettingsService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    pub async fn get_settings(&self) -> Result<SettingsSnapshot, CoreError> {
        let mut snapshot = SettingsSnapshot::default();

        if let Some(value) = self.db.get_setting("theme")? {
            snapshot.theme = value;
        }
        if let Some(value) = self.db.get_setting("language")? {
            snapshot.language = value;
        }
        if let Some(value) = self.db.get_setting("refresh_interval_min")? {
            if let Ok(n) = value.parse::<i64>() {
                snapshot.refresh_interval_min = n;
            }
        }

        Ok(snapshot)
    }
}
