use std::sync::Arc;

use crate::db::Db;
use crate::dto::SettingsSnapshot;
use crate::error::{CoreError, ErrorCategory};

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

    pub async fn set_theme(&self, theme: String) -> Result<(), CoreError> {
        if !matches!(theme.as_str(), "system" | "light" | "dark") {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "invalidTheme",
                Some(theme),
            ));
        }
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_setting("theme", &theme))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }

    pub async fn set_language(&self, language: String) -> Result<(), CoreError> {
        if !matches!(language.as_str(), "en" | "zh" | "ja") {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "invalidLanguage",
                Some(language),
            ));
        }
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_setting("language", &language))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn appearance_settings_validate_and_persist() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&tmp.path().join("test.db")).unwrap());
        let service = SettingsService::new(db);

        service.set_theme("dark".to_string()).await.unwrap();
        service.set_language("ja".to_string()).await.unwrap();
        let snapshot = service.get_settings().await.unwrap();
        assert_eq!(snapshot.theme, "dark");
        assert_eq!(snapshot.language, "ja");

        assert_eq!(
            service
                .set_theme("sepia".to_string())
                .await
                .unwrap_err()
                .code(),
            "invalidTheme"
        );
        assert_eq!(
            service
                .set_language("fr".to_string())
                .await
                .unwrap_err()
                .code(),
            "invalidLanguage"
        );
    }
}
