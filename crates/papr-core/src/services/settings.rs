use std::sync::Arc;

use crate::ai::{AiProfile, AiPurpose};
use crate::db::{Db, REFRESH_OFF_MINUTES};
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
        snapshot.refresh_interval_min =
            stored_refresh_interval(self.db.get_setting("refresh_interval_min")?.as_deref());
        snapshot.notifications_enabled =
            self.bool_setting("notify_enabled", snapshot.notifications_enabled)?;
        snapshot.notification_quiet_hours =
            self.bool_setting("notify_dnd_night", snapshot.notification_quiet_hours)?;
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

    /// Persist the settings that drive Android background refresh and alerts.
    pub async fn set_background_settings(
        &self,
        refresh_interval_min: i64,
        notifications_enabled: bool,
        notification_quiet_hours: bool,
    ) -> Result<(), CoreError> {
        if refresh_interval_min != REFRESH_OFF_MINUTES && !(5..=120).contains(&refresh_interval_min)
        {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "invalidRefreshInterval",
                Some(refresh_interval_min.to_string()),
            ));
        }
        let values = vec![
            ("refresh_interval_min", refresh_interval_min.to_string()),
            (
                "notify_enabled",
                if notifications_enabled { "1" } else { "0" }.to_string(),
            ),
            (
                "notify_dnd_night",
                if notification_quiet_hours { "1" } else { "0" }.to_string(),
            ),
        ];
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_settings(&values))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {e}")))?
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

    /// Read non-sensitive AI profile metadata. Credentials live in platform storage.
    pub async fn list_ai_profiles(&self) -> Result<Vec<AiProfile>, CoreError> {
        let value = self.db.get_setting("ai_profiles")?;
        let mut profiles = match value {
            Some(value) => serde_json::from_str::<Vec<AiProfile>>(&value).map_err(|_| {
                CoreError::coded(ErrorCategory::InvalidInput, "invalidAiProfile", None)
            })?,
            None => Vec::new(),
        };
        for profile in &profiles {
            profile.validate()?;
        }
        if profiles.iter().filter(|profile| profile.enabled).count() > 1 {
            let active_index = profiles
                .iter()
                .position(|profile| {
                    profile.enabled && profile.default_for.contains(&AiPurpose::Summary)
                })
                .or_else(|| profiles.iter().position(|profile| profile.enabled));
            for (index, profile) in profiles.iter_mut().enumerate() {
                profile.enabled = Some(index) == active_index;
            }
            let value = serde_json::to_string(&profiles).map_err(|_| {
                CoreError::coded(ErrorCategory::InvalidInput, "invalidAiProfile", None)
            })?;
            let db = Arc::clone(&self.db);
            tokio::task::spawn_blocking(move || db.set_setting("ai_profiles", &value))
                .await
                .map_err(|e| CoreError::Platform(format!("blocking task failed: {e}")))??;
        }
        Ok(profiles)
    }

    /// Insert or replace one AI profile without ever accepting a credential value.
    pub async fn save_ai_profile(&self, mut profile: AiProfile) -> Result<(), CoreError> {
        profile.id = profile.id.trim().to_string();
        profile.name = profile.name.trim().to_string();
        profile.model = profile.model.trim().to_string();
        profile.base_url = profile.base_url.trim().trim_end_matches('/').to_string();
        profile.validate()?;

        let profile_id = profile.id.clone();
        let should_enable = profile.enabled;
        let mut profiles = self.list_ai_profiles().await?;
        if let Some(index) = profiles.iter().position(|item| item.id == profile.id) {
            profiles[index] = profile;
        } else {
            if profiles.len() >= 20 {
                return Err(CoreError::coded(
                    ErrorCategory::InvalidInput,
                    "tooManyAiProfiles",
                    None,
                ));
            }
            profiles.push(profile);
        }
        if should_enable {
            for item in &mut profiles {
                item.enabled = item.id == profile_id;
            }
        }
        let value = serde_json::to_string(&profiles)
            .map_err(|_| CoreError::coded(ErrorCategory::InvalidInput, "invalidAiProfile", None))?;
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_setting("ai_profiles", &value))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {e}")))?
    }

    /// Toggle the one profile used by mobile AI features.
    ///
    /// Enabling a profile disables every other profile in the same settings
    /// write. Disabling the active profile deliberately leaves no fallback.
    pub async fn set_ai_profile_enabled(
        &self,
        profile_id: String,
        enabled: bool,
    ) -> Result<(), CoreError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "invalidAiProfile",
                None,
            ));
        }

        let mut profiles = self.list_ai_profiles().await?;
        if !profiles.iter().any(|profile| profile.id == profile_id) {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "invalidAiProfile",
                None,
            ));
        }
        if enabled {
            for profile in &mut profiles {
                profile.enabled = profile.id == profile_id;
            }
        } else {
            for profile in &mut profiles {
                profile.enabled = false;
            }
        }

        let value = serde_json::to_string(&profiles)
            .map_err(|_| CoreError::coded(ErrorCategory::InvalidInput, "invalidAiProfile", None))?;
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_setting("ai_profiles", &value))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {e}")))?
    }

    /// Delete profile metadata first and return its credential alias for best-effort cleanup.
    pub async fn delete_ai_profile(&self, profile_id: String) -> Result<Option<String>, CoreError> {
        let profile_id = profile_id.trim();
        if profile_id.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "invalidAiProfile",
                None,
            ));
        }
        let mut profiles = self.list_ai_profiles().await?;
        let credential_ref = profiles
            .iter()
            .find(|profile| profile.id == profile_id)
            .and_then(|profile| profile.credential_ref.clone());
        profiles.retain(|profile| profile.id != profile_id);
        let value = serde_json::to_string(&profiles)
            .map_err(|_| CoreError::coded(ErrorCategory::InvalidInput, "invalidAiProfile", None))?;
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.set_setting("ai_profiles", &value))
            .await
            .map_err(|e| CoreError::Platform(format!("blocking task failed: {e}")))??;
        Ok(credential_ref)
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

pub(crate) fn stored_refresh_interval(value: Option<&str>) -> i64 {
    match value.and_then(|value| value.parse::<i64>().ok()) {
        Some(value) if value >= REFRESH_OFF_MINUTES => REFRESH_OFF_MINUTES,
        Some(value) if (5..=120).contains(&value) => value,
        _ => 30,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::{AiAuthMode, AiProtocol, AiPurpose};
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn appearance_settings_validate_and_persist() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&tmp.path().join("test.db")).unwrap());
        let service = SettingsService::new(Arc::clone(&db));

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

    #[tokio::test]
    async fn background_settings_validate_and_persist_atomically() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&tmp.path().join("test.db")).unwrap());
        let service = SettingsService::new(Arc::clone(&db));

        let defaults = service.get_settings().await.unwrap();
        assert_eq!(defaults.refresh_interval_min, 30);
        assert!(!defaults.notifications_enabled);
        assert!(!defaults.notification_quiet_hours);

        service
            .set_background_settings(60, true, true)
            .await
            .unwrap();
        let saved = service.get_settings().await.unwrap();
        assert_eq!(saved.refresh_interval_min, 60);
        assert!(saved.notifications_enabled);
        assert!(saved.notification_quiet_hours);

        assert_eq!(
            service
                .set_background_settings(4, false, false)
                .await
                .unwrap_err()
                .code(),
            "invalidRefreshInterval"
        );
        assert_eq!(
            service.get_settings().await.unwrap().refresh_interval_min,
            60
        );

        service
            .set_background_settings(REFRESH_OFF_MINUTES, false, false)
            .await
            .unwrap();
        assert_eq!(
            service.get_settings().await.unwrap().refresh_interval_min,
            REFRESH_OFF_MINUTES
        );
        assert_eq!(stored_refresh_interval(Some("broken")), 30);
        assert_eq!(stored_refresh_interval(Some("4")), 30);
        assert_eq!(stored_refresh_interval(Some("999999")), REFRESH_OFF_MINUTES);
    }

    #[tokio::test]
    async fn ai_profiles_persist_only_metadata_and_delete_returns_the_alias() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&tmp.path().join("test.db")).unwrap());
        let service = SettingsService::new(Arc::clone(&db));
        let profile = AiProfile {
            id: " profile-1 ".into(),
            name: " Primary ".into(),
            protocol: AiProtocol::OpenaiChatCompletions,
            model: " model ".into(),
            base_url: "https://example.com/v1/".into(),
            auth: AiAuthMode::Bearer,
            headers: BTreeMap::new(),
            credential_ref: Some("papr.ai.profile-1".into()),
            enabled: true,
            default_for: vec![AiPurpose::Summary],
        };

        service.save_ai_profile(profile).await.unwrap();
        let profiles = service.list_ai_profiles().await.unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, "profile-1");
        assert_eq!(profiles[0].base_url, "https://example.com/v1");
        let stored = db.get_setting("ai_profiles").unwrap().unwrap();
        assert!(!stored.contains("api_key"));
        assert!(!stored.contains("secret"));

        assert_eq!(
            service.delete_ai_profile("profile-1".into()).await.unwrap(),
            Some("papr.ai.profile-1".into())
        );
        assert!(service.list_ai_profiles().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn ai_profile_enablement_is_exclusive_and_can_be_fully_disabled() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&tmp.path().join("test.db")).unwrap());
        let service = SettingsService::new(Arc::clone(&db));
        let primary = AiProfile {
            id: "primary".into(),
            name: "Primary".into(),
            protocol: AiProtocol::OpenaiChatCompletions,
            model: "model-a".into(),
            base_url: "https://example.com/v1".into(),
            auth: AiAuthMode::None,
            headers: BTreeMap::new(),
            credential_ref: None,
            enabled: true,
            default_for: vec![AiPurpose::Summary],
        };
        let secondary = AiProfile {
            id: "secondary".into(),
            name: "Secondary".into(),
            model: "model-b".into(),
            ..primary.clone()
        };

        db.set_setting(
            "ai_profiles",
            &serde_json::to_string(&vec![primary.clone(), secondary.clone()]).unwrap(),
        )
        .unwrap();
        let profiles = service.list_ai_profiles().await.unwrap();
        assert!(profiles[0].enabled);
        assert!(!profiles[1].enabled);
        let persisted: Vec<AiProfile> =
            serde_json::from_str(&db.get_setting("ai_profiles").unwrap().unwrap()).unwrap();
        assert!(persisted[0].enabled);
        assert!(!persisted[1].enabled);

        service
            .set_ai_profile_enabled("secondary".into(), true)
            .await
            .unwrap();
        let profiles = service.list_ai_profiles().await.unwrap();
        assert!(!profiles[0].enabled);
        assert!(profiles[1].enabled);

        service
            .set_ai_profile_enabled("secondary".into(), false)
            .await
            .unwrap();
        assert!(service
            .list_ai_profiles()
            .await
            .unwrap()
            .iter()
            .all(|profile| !profile.enabled));
    }
}
