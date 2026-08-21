use std::sync::Arc;

use crate::db::Db;
use crate::dto::{ReadingSettings, SettingsSnapshot};
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
        if let Some(value) = self.db.get_setting("reading_font")? {
            if matches!(value.as_str(), "system" | "serif" | "sans") {
                snapshot.reading.font = value;
            }
        }
        snapshot.reading.font_size =
            self.number_setting("reading_font_size", snapshot.reading.font_size, 14.0, 24.0)?;
        snapshot.reading.line_height = self.number_setting(
            "reading_line_height",
            snapshot.reading.line_height,
            1.3,
            2.0,
        )?;
        snapshot.reading.content_width = self.number_setting(
            "reading_content_width",
            snapshot.reading.content_width,
            320.0,
            840.0,
        )?;
        snapshot.reading.show_reading_time =
            self.bool_setting("reading_show_time", snapshot.reading.show_reading_time)?;
        snapshot.reading.auto_extract =
            self.bool_setting("reading_auto_extract", snapshot.reading.auto_extract)?;

        Ok(snapshot)
    }

    fn number_setting(
        &self,
        key: &str,
        fallback: f64,
        min: f64,
        max: f64,
    ) -> Result<f64, CoreError> {
        Ok(self
            .db
            .get_setting(key)?
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .map(|value| value.clamp(min, max))
            .unwrap_or(fallback))
    }

    fn bool_setting(&self, key: &str, fallback: bool) -> Result<bool, CoreError> {
        Ok(match self.db.get_setting(key)?.as_deref() {
            Some("1") | Some("true") => true,
            Some("0") | Some("false") => false,
            _ => fallback,
        })
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

    pub async fn set_reading_settings(&self, settings: ReadingSettings) -> Result<(), CoreError> {
        if !matches!(settings.font.as_str(), "system" | "serif" | "sans") {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "invalidReadingFont",
                Some(settings.font),
            ));
        }
        validate_range(settings.font_size, 14.0, 24.0, "invalidReadingFontSize")?;
        validate_range(settings.line_height, 1.3, 2.0, "invalidReadingLineHeight")?;
        validate_range(settings.content_width, 320.0, 840.0, "invalidReadingWidth")?;

        let values = vec![
            ("reading_font", settings.font),
            ("reading_font_size", settings.font_size.to_string()),
            ("reading_line_height", settings.line_height.to_string()),
            ("reading_content_width", settings.content_width.to_string()),
            (
                "reading_show_time",
                if settings.show_reading_time { "1" } else { "0" }.to_string(),
            ),
            (
                "reading_auto_extract",
                if settings.auto_extract { "1" } else { "0" }.to_string(),
            ),
        ];
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_settings(&values))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {e}")))?
    }
}

fn validate_range(value: f64, min: f64, max: f64, code: &'static str) -> Result<(), CoreError> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            code,
            Some(value.to_string()),
        ))
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

        let reading = ReadingSettings {
            font: "serif".to_string(),
            font_size: 19.0,
            line_height: 1.8,
            content_width: 720.0,
            show_reading_time: false,
            auto_extract: true,
        };
        service.set_reading_settings(reading.clone()).await.unwrap();
        assert_eq!(service.get_settings().await.unwrap().reading, reading);

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
        assert_eq!(
            service
                .set_reading_settings(ReadingSettings {
                    font_size: f64::NAN,
                    ..ReadingSettings::default()
                })
                .await
                .unwrap_err()
                .code(),
            "invalidReadingFontSize"
        );
    }
}
