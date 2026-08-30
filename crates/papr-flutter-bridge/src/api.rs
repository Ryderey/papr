//! Flutter Rust Bridge v2 public API.
//!
//! This file declares the exact Rust surface exposed to Dart. All business
//! logic lives in `papr-core`; this crate is only a thin wrapper plus DTO
//! conversion.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use flutter_rust_bridge::frb;
use papr_core::PaprCore;

use crate::dto::{
    AddFeedInput, AiAuthMode, AiFollowUpTurn, AiHeader, AiProfile, AiProtocol, AiPurpose,
    AiStreamEvent, AiSummaryCache, ArticleCounts, ArticleDetail, ArticleFilter, ArticleFilterKind,
    ArticleSummary, DiscoveryResult, Enclosure, Feed, Folder, Highlight, HighlightInput,
    OpmlImportReport, PaprCoreConfig, Platform, ReadingSettings, RefreshError, RefreshOptions,
    RefreshReport, ResolvedHighlight, Rule, RuleInput, RulePreview, SettingsSnapshot, SourceType,
    SummaryTemplate, Tag, TagSummary,
};
use crate::error::PaprBridgeError;
use crate::frb_generated::StreamSink;

pub(crate) struct AiRequestRegistry {
    active: Mutex<HashMap<String, papr_core::ai::AiCancellation>>,
}

impl AiRequestRegistry {
    fn new() -> Self {
        Self {
            active: Mutex::new(HashMap::new()),
        }
    }

    fn register(self: &Arc<Self>, request_id: &str) -> Result<AiRequestLease, PaprBridgeError> {
        if request_id.trim().is_empty() {
            return Err(invalid_ai_request());
        }

        let cancellation = papr_core::ai::AiCancellation::default();
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if active.contains_key(request_id) {
            return Err(invalid_ai_request());
        }
        active.insert(request_id.to_string(), cancellation.clone());
        drop(active);

        Ok(AiRequestLease {
            registry: Arc::clone(self),
            request_id: request_id.to_string(),
            cancellation,
        })
    }

    fn cancel(&self, request_id: &str) -> bool {
        let cancellation = self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(request_id)
            .cloned();
        if let Some(cancellation) = cancellation {
            cancellation.cancel();
            true
        } else {
            false
        }
    }

    fn finish(&self, request_id: &str) {
        self.active
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(request_id);
    }
}

struct AiRequestLease {
    registry: Arc<AiRequestRegistry>,
    request_id: String,
    cancellation: papr_core::ai::AiCancellation,
}

impl Drop for AiRequestLease {
    fn drop(&mut self) {
        self.registry.finish(&self.request_id);
    }
}

fn invalid_ai_request() -> PaprBridgeError {
    PaprBridgeError {
        category: crate::error::ErrorCategory::InvalidInput,
        code: "invalidInput".to_string(),
        detail: None,
    }
}

/// Streaming APIs report expected operational failures through their typed event
/// channel. Returning such a failure from the FRB task would create an
/// unobserved Future error because Flutter Rust Bridge starts that task eagerly.
fn emit_ai_stream_error(
    sink: &StreamSink<AiStreamEvent>,
    request_id: &str,
    code: impl Into<String>,
) {
    let _ = sink.add(AiStreamEvent::Error {
        request_id: request_id.to_string(),
        code: code.into(),
    });
}

/// Opaque handle to a `PaprCore` instance.
///
/// Dart owns the lifetime of this handle via Riverpod. The inner `PaprCore`
/// is reference-counted so it can be used across multiple bridge calls.
#[frb(opaque)]
pub struct PaprCoreBridge {
    inner: Arc<PaprCore>,
    ai_requests: Arc<AiRequestRegistry>,
}

/// Optional initialisation hook called by FRB before the first API use.
#[frb(init)]
pub fn init_app() {
    // Placeholder for panic hooks or logging initialisation.
}

/// Initialise `PaprCore` with platform-provided configuration.
pub async fn init_papr_core(config: PaprCoreConfig) -> Result<PaprCoreBridge, PaprBridgeError> {
    let core = PaprCore::new(config.into()).await?;
    Ok(PaprCoreBridge {
        inner: Arc::new(core),
        ai_requests: Arc::new(AiRequestRegistry::new()),
    })
}

/// List all feeds.
pub async fn get_feeds(core: &PaprCoreBridge) -> Result<Vec<Feed>, PaprBridgeError> {
    let feeds = core.inner.feed_service().list_feeds().await?;
    Ok(feeds.into_iter().map(Into::into).collect())
}

/// Add a new feed by URL.
pub async fn add_feed(core: &PaprCoreBridge, input: AddFeedInput) -> Result<Feed, PaprBridgeError> {
    let feed = core.inner.ingestion_service().add_feed(input.input).await?;
    Ok(feed.into())
}

/// List all folders.
pub async fn list_folders(core: &PaprCoreBridge) -> Result<Vec<Folder>, PaprBridgeError> {
    let folders = core.inner.folder_service().list_folders().await?;
    Ok(folders.into_iter().map(Into::into).collect())
}

/// Create a folder and return its ID.
pub async fn create_folder(core: &PaprCoreBridge, name: String) -> Result<i64, PaprBridgeError> {
    Ok(core.inner.folder_service().create_folder(name).await?)
}

/// Rename a folder.
pub async fn rename_folder(
    core: &PaprCoreBridge,
    id: i64,
    name: String,
) -> Result<(), PaprBridgeError> {
    Ok(core.inner.folder_service().rename_folder(id, name).await?)
}

/// Delete a folder without deleting its feeds.
pub async fn delete_folder(core: &PaprCoreBridge, id: i64) -> Result<(), PaprBridgeError> {
    Ok(core.inner.folder_service().delete_folder(id).await?)
}

/// Persist the complete folder order.
pub async fn reorder_folders(
    core: &PaprCoreBridge,
    folder_ids: Vec<i64>,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .folder_service()
        .reorder_folders(folder_ids)
        .await?)
}

/// Delete a feed and its dependent rows.
pub async fn delete_feed(core: &PaprCoreBridge, id: i64) -> Result<(), PaprBridgeError> {
    Ok(core.inner.feed_service().delete_feed(id).await?)
}

/// Rename a feed.
pub async fn rename_feed(
    core: &PaprCoreBridge,
    id: i64,
    title: String,
) -> Result<(), PaprBridgeError> {
    Ok(core.inner.feed_service().rename_feed(id, title).await?)
}

/// Move a feed to a folder, or to uncategorised when `folder_id` is absent.
pub async fn move_feed(
    core: &PaprCoreBridge,
    id: i64,
    folder_id: Option<i64>,
) -> Result<(), PaprBridgeError> {
    Ok(core.inner.feed_service().move_feed(id, folder_id).await?)
}

/// Override a feed's refresh interval, or clear the override.
pub async fn set_feed_refresh_interval(
    core: &PaprCoreBridge,
    id: i64,
    minutes: Option<i64>,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .feed_service()
        .set_feed_refresh_interval(id, minutes)
        .await?)
}

/// Force-refresh one feed through the shared refresh pipeline.
pub async fn refresh_feed(
    core: &PaprCoreBridge,
    id: i64,
) -> Result<RefreshReport, PaprBridgeError> {
    let report = core
        .inner
        .ingestion_service()
        .refresh_feeds(papr_core::RefreshOptions {
            feed_ids: Some(vec![id]),
            force: true,
        })
        .await?;
    Ok(report.into())
}

/// Search the bundled subscription directory.
pub fn search_directory(query: String, lang: String) -> Vec<DiscoveryResult> {
    papr_core::ingestion::discovery::search_directory(&query, &lang)
        .into_iter()
        .map(Into::into)
        .collect()
}

/// Extract a subscription target from a Papr deep link.
pub fn parse_deep_link(url: String) -> Option<String> {
    match papr_core::ingestion::discovery::parse_deep_link(&url) {
        Some(papr_core::ingestion::discovery::DeepLink::Subscribe { url }) => Some(url),
        None => None,
    }
}

/// List articles matching a filter.
pub async fn get_articles(
    core: &PaprCoreBridge,
    filter: ArticleFilter,
) -> Result<Vec<ArticleSummary>, PaprBridgeError> {
    let articles = core
        .inner
        .article_service()
        .list_articles(filter.into())
        .await?;
    Ok(articles.into_iter().map(Into::into).collect())
}

/// Count articles matching an unbounded filter.
pub async fn count_articles(
    core: &PaprCoreBridge,
    filter: ArticleFilter,
) -> Result<i64, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .count_articles(filter.into())
        .await?)
}

/// Return counts for the built-in smart views.
pub async fn get_article_counts(core: &PaprCoreBridge) -> Result<ArticleCounts, PaprBridgeError> {
    Ok(core.inner.article_service().article_counts().await?.into())
}

/// List existing tags for read-only article filtering.
pub async fn list_article_tags(core: &PaprCoreBridge) -> Result<Vec<TagSummary>, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .list_tags()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn create_tag(core: &PaprCoreBridge, name: String) -> Result<i64, PaprBridgeError> {
    Ok(core.inner.article_service().create_tag(name).await?)
}

pub async fn rename_tag(
    core: &PaprCoreBridge,
    id: i64,
    name: String,
) -> Result<(), PaprBridgeError> {
    Ok(core.inner.article_service().rename_tag(id, name).await?)
}

pub async fn set_tag_color(
    core: &PaprCoreBridge,
    id: i64,
    color: String,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .set_tag_color(id, color)
        .await?)
}

pub async fn reorder_tags(core: &PaprCoreBridge, tag_ids: Vec<i64>) -> Result<(), PaprBridgeError> {
    Ok(core.inner.article_service().reorder_tags(tag_ids).await?)
}

pub async fn delete_tag(core: &PaprCoreBridge, id: i64) -> Result<(), PaprBridgeError> {
    Ok(core.inner.article_service().delete_tag(id).await?)
}

pub async fn set_article_tag(
    core: &PaprCoreBridge,
    article_id: i64,
    tag_id: i64,
    attached: bool,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .set_article_tag(article_id, tag_id, attached)
        .await?)
}

pub async fn list_rules(core: &PaprCoreBridge) -> Result<Vec<Rule>, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .list_rules()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn create_rule(core: &PaprCoreBridge, input: RuleInput) -> Result<i64, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .create_rule(input.into())
        .await?)
}

pub async fn update_rule(
    core: &PaprCoreBridge,
    id: i64,
    input: RuleInput,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .update_rule(id, input.into())
        .await?)
}

pub async fn delete_rule(core: &PaprCoreBridge, id: i64) -> Result<(), PaprBridgeError> {
    Ok(core.inner.article_service().delete_rule(id).await?)
}

pub async fn preview_rule(
    core: &PaprCoreBridge,
    input: RuleInput,
) -> Result<RulePreview, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .preview_rule(input.into())
        .await?
        .into())
}

pub async fn apply_rule_to_existing(
    core: &PaprCoreBridge,
    input: RuleInput,
) -> Result<i64, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .apply_rule_to_existing(input.into())
        .await?)
}

pub async fn list_highlights(
    core: &PaprCoreBridge,
    article_id: i64,
) -> Result<Vec<Highlight>, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .list_highlights(article_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn list_all_highlights(core: &PaprCoreBridge) -> Result<Vec<Highlight>, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .list_all_highlights()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn create_highlight(
    core: &PaprCoreBridge,
    input: HighlightInput,
) -> Result<i64, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .create_highlight(input.into())
        .await?)
}

pub async fn update_highlight_note(
    core: &PaprCoreBridge,
    id: i64,
    note: String,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .update_highlight_note(id, note)
        .await?)
}

pub async fn set_highlight_color(
    core: &PaprCoreBridge,
    id: i64,
    color: String,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .set_highlight_color(id, color)
        .await?)
}

pub async fn delete_highlight(core: &PaprCoreBridge, id: i64) -> Result<(), PaprBridgeError> {
    Ok(core.inner.article_service().delete_highlight(id).await?)
}

pub async fn resolve_highlights(
    core: &PaprCoreBridge,
    article_id: i64,
    text: String,
) -> Result<Vec<ResolvedHighlight>, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .resolve_highlights(article_id, text)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

/// Fetch the full detail for one article.
pub async fn get_article_detail(
    core: &PaprCoreBridge,
    article_id: i64,
) -> Result<ArticleDetail, PaprBridgeError> {
    let detail = core
        .inner
        .article_service()
        .get_article_detail(article_id)
        .await?;
    Ok(detail.into())
}

pub async fn set_article_read(
    core: &PaprCoreBridge,
    article_id: i64,
    value: bool,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .set_read(article_id, value)
        .await?)
}

pub async fn set_article_starred(
    core: &PaprCoreBridge,
    article_id: i64,
    value: bool,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .set_starred(article_id, value)
        .await?)
}

pub async fn set_article_read_later(
    core: &PaprCoreBridge,
    article_id: i64,
    value: bool,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .set_read_later(article_id, value)
        .await?)
}

pub async fn mark_all_articles_read(
    core: &PaprCoreBridge,
    filter: ArticleFilter,
) -> Result<i64, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .mark_all_read(filter.into())
        .await?)
}

pub async fn extract_article_fulltext(
    core: &PaprCoreBridge,
    article_id: i64,
) -> Result<String, PaprBridgeError> {
    Ok(core
        .inner
        .article_service()
        .extract_fulltext(article_id)
        .await?)
}

/// Refresh feeds.
pub async fn refresh_feeds(
    core: &PaprCoreBridge,
    options: RefreshOptions,
) -> Result<RefreshReport, PaprBridgeError> {
    let report = core
        .inner
        .ingestion_service()
        .refresh_feeds(options.into())
        .await?;
    Ok(report.into())
}

/// Import subscriptions from OPML text.
pub async fn import_opml(
    core: &PaprCoreBridge,
    opml_text: String,
) -> Result<OpmlImportReport, PaprBridgeError> {
    let report = core.inner.opml_service().import_text(opml_text).await?;
    Ok(report.into())
}

/// Export subscriptions as OPML text.
pub async fn export_opml(core: &PaprCoreBridge) -> Result<String, PaprBridgeError> {
    Ok(core.inner.opml_service().export_text().await?)
}

/// Read a snapshot of user settings.
pub async fn get_settings(core: &PaprCoreBridge) -> Result<SettingsSnapshot, PaprBridgeError> {
    let snapshot = core.inner.settings_service().get_settings().await?;
    Ok(snapshot.into())
}

/// Persist the application theme (`system`, `light`, or `dark`).
pub async fn set_theme(core: &PaprCoreBridge, theme: String) -> Result<(), PaprBridgeError> {
    Ok(core.inner.settings_service().set_theme(theme).await?)
}

/// Persist the UI language (`en`, `zh`, or `ja`).
pub async fn set_language(core: &PaprCoreBridge, language: String) -> Result<(), PaprBridgeError> {
    Ok(core.inner.settings_service().set_language(language).await?)
}

/// Persist validated automatic-refresh and notification settings atomically.
pub async fn set_background_settings(
    core: &PaprCoreBridge,
    refresh_interval_min: i64,
    notifications_enabled: bool,
    notification_quiet_hours: bool,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .settings_service()
        .set_background_settings(
            refresh_interval_min,
            notifications_enabled,
            notification_quiet_hours,
        )
        .await?)
}

/// Persist validated reader appearance and behaviour settings.
pub async fn set_reading_settings(
    core: &PaprCoreBridge,
    settings: ReadingSettings,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .settings_service()
        .set_reading_settings(settings.into())
        .await?)
}

/// List persistable AI profile metadata. This API never returns credentials.
pub async fn list_ai_profiles(core: &PaprCoreBridge) -> Result<Vec<AiProfile>, PaprBridgeError> {
    Ok(core
        .inner
        .settings_service()
        .list_ai_profiles()
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

/// Insert or replace non-sensitive AI profile metadata.
pub async fn save_ai_profile(
    core: &PaprCoreBridge,
    profile: AiProfile,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .settings_service()
        .save_ai_profile(profile.into())
        .await?)
}

/// Set the sole active AI profile, or disable the selected profile.
pub async fn set_ai_profile_enabled(
    core: &PaprCoreBridge,
    profile_id: String,
    enabled: bool,
) -> Result<(), PaprBridgeError> {
    Ok(core
        .inner
        .settings_service()
        .set_ai_profile_enabled(profile_id, enabled)
        .await?)
}

/// Delete profile metadata and return its credential alias for platform cleanup.
pub async fn delete_ai_profile(
    core: &PaprCoreBridge,
    profile_id: String,
) -> Result<Option<String>, PaprBridgeError> {
    Ok(core
        .inner
        .settings_service()
        .delete_ai_profile(profile_id)
        .await?)
}

/// Verify a saved profile through the same streaming provider path as summaries.
/// Credentials are transient and this does not create or replace any cache.
pub async fn test_ai_connection(
    core: &PaprCoreBridge,
    profile: AiProfile,
    credential: Option<String>,
) -> Result<(), PaprBridgeError> {
    let profile = papr_core::ai::AiProfile::from(profile);
    let credential = credential
        .map(papr_core::ai::ResolvedAiCredential::new)
        .transpose()?;
    Ok(core
        .inner
        .ai_service()
        .test_connection(&profile, credential.as_ref())
        .await?)
}

/// Read the most recent complete summary without starting a network request.
pub async fn get_ai_summary_cache(
    core: &PaprCoreBridge,
    article_id: i64,
) -> Result<Option<AiSummaryCache>, PaprBridgeError> {
    Ok(core
        .inner
        .ai_service()
        .summary_cache(article_id)
        .await?
        .map(Into::into))
}

/// Stream a summary and atomically cache it only after complete success.
#[allow(clippy::too_many_arguments)]
pub async fn stream_ai_summary(
    core: &PaprCoreBridge,
    article_id: i64,
    profile: AiProfile,
    credential: Option<String>,
    template: SummaryTemplate,
    language: String,
    request_id: String,
    sink: StreamSink<AiStreamEvent>,
) -> Result<(), PaprBridgeError> {
    let lease = match core.ai_requests.register(&request_id) {
        Ok(lease) => lease,
        Err(error) => {
            emit_ai_stream_error(&sink, &request_id, error.code.clone());
            return Ok(());
        }
    };
    let profile = papr_core::ai::AiProfile::from(profile);
    let credential = match credential
        .map(papr_core::ai::ResolvedAiCredential::new)
        .transpose()
    {
        Ok(credential) => credential,
        Err(error) => {
            emit_ai_stream_error(&sink, &request_id, error.code().to_string());
            return Ok(());
        }
    };

    // `AiService::summarize` emits one terminal Error event for every service
    // failure. Keep the FRB task successful so the generated eager task does
    // not surface that same expected error as an unhandled Dart Future.
    let _ = core
        .inner
        .ai_service()
        .summarize(
            article_id,
            &profile,
            credential.as_ref(),
            template.into(),
            &language,
            &request_id,
            &lease.cancellation,
            |event| sink.add(event.into()).is_ok(),
        )
        .await;
    Ok(())
}

/// Translate one article through the configured LLM and cache only a complete
/// sanitized result. The event stream reports batch progress, never raw text.
#[allow(clippy::too_many_arguments)]
pub async fn stream_ai_translation(
    core: &PaprCoreBridge,
    article_id: i64,
    profile: AiProfile,
    credential: Option<String>,
    language: String,
    request_id: String,
    sink: StreamSink<AiStreamEvent>,
) -> Result<(), PaprBridgeError> {
    let lease = match core.ai_requests.register(&request_id) {
        Ok(lease) => lease,
        Err(error) => {
            emit_ai_stream_error(&sink, &request_id, error.code.clone());
            return Ok(());
        }
    };
    let profile = papr_core::ai::AiProfile::from(profile);
    let credential = match credential
        .map(papr_core::ai::ResolvedAiCredential::new)
        .transpose()
    {
        Ok(credential) => credential,
        Err(error) => {
            emit_ai_stream_error(&sink, &request_id, error.code().to_string());
            return Ok(());
        }
    };

    let _ = core
        .inner
        .ai_service()
        .translate_with_profile(
            article_id,
            &profile,
            credential.as_ref(),
            &language,
            &request_id,
            &lease.cancellation,
            |event| sink.add(event.into()).is_ok(),
        )
        .await;
    Ok(())
}

/// Stream a follow-up answer using only the supplied summary and Q&A history.
#[allow(clippy::too_many_arguments)]
pub async fn stream_ai_follow_up(
    core: &PaprCoreBridge,
    profile: AiProfile,
    credential: Option<String>,
    summary: String,
    history: Vec<AiFollowUpTurn>,
    question: String,
    language: String,
    request_id: String,
    sink: StreamSink<AiStreamEvent>,
) -> Result<(), PaprBridgeError> {
    let lease = match core.ai_requests.register(&request_id) {
        Ok(lease) => lease,
        Err(error) => {
            emit_ai_stream_error(&sink, &request_id, error.code.clone());
            return Ok(());
        }
    };
    let profile = papr_core::ai::AiProfile::from(profile);
    let credential = match credential
        .map(papr_core::ai::ResolvedAiCredential::new)
        .transpose()
    {
        Ok(credential) => credential,
        Err(error) => {
            emit_ai_stream_error(&sink, &request_id, error.code().to_string());
            return Ok(());
        }
    };
    let history = history
        .into_iter()
        .map(|turn| (turn.question, turn.answer))
        .collect::<Vec<_>>();

    // `AiService::follow_up` emits one terminal Error event for every service
    // failure; do not return it through FRB's eagerly-started task as well.
    let _ = core
        .inner
        .ai_service()
        .follow_up(
            &profile,
            credential.as_ref(),
            &summary,
            &history,
            &question,
            &language,
            &request_id,
            &lease.cancellation,
            |event| sink.add(event.into()).is_ok(),
        )
        .await;
    Ok(())
}

/// Cooperatively cancel an active AI request. Missing IDs are already cancelled.
pub fn cancel_ai_request(core: &PaprCoreBridge, request_id: String) -> bool {
    core.ai_requests.cancel(&request_id)
}

// ---------------------------------------------------------------------------
// DTO conversions: bridge DTOs ↔ papr-core DTOs
// ---------------------------------------------------------------------------

impl From<AiProtocol> for papr_core::ai::AiProtocol {
    fn from(protocol: AiProtocol) -> Self {
        match protocol {
            AiProtocol::AnthropicMessages => Self::AnthropicMessages,
            AiProtocol::OpenaiChatCompletions => Self::OpenaiChatCompletions,
        }
    }
}

impl From<papr_core::ai::AiProtocol> for AiProtocol {
    fn from(protocol: papr_core::ai::AiProtocol) -> Self {
        match protocol {
            papr_core::ai::AiProtocol::AnthropicMessages => Self::AnthropicMessages,
            papr_core::ai::AiProtocol::OpenaiChatCompletions => Self::OpenaiChatCompletions,
        }
    }
}

impl From<AiAuthMode> for papr_core::ai::AiAuthMode {
    fn from(auth: AiAuthMode) -> Self {
        match auth {
            AiAuthMode::Bearer => Self::Bearer,
            AiAuthMode::XApiKey => Self::XApiKey,
            AiAuthMode::None => Self::None,
        }
    }
}

impl From<papr_core::ai::AiAuthMode> for AiAuthMode {
    fn from(auth: papr_core::ai::AiAuthMode) -> Self {
        match auth {
            papr_core::ai::AiAuthMode::Bearer => Self::Bearer,
            papr_core::ai::AiAuthMode::XApiKey => Self::XApiKey,
            papr_core::ai::AiAuthMode::None => Self::None,
        }
    }
}

impl From<AiPurpose> for papr_core::ai::AiPurpose {
    fn from(purpose: AiPurpose) -> Self {
        match purpose {
            AiPurpose::Summary => Self::Summary,
            AiPurpose::Translate => Self::Translate,
        }
    }
}

impl From<papr_core::ai::AiPurpose> for AiPurpose {
    fn from(purpose: papr_core::ai::AiPurpose) -> Self {
        match purpose {
            papr_core::ai::AiPurpose::Summary => Self::Summary,
            papr_core::ai::AiPurpose::Translate => Self::Translate,
        }
    }
}

impl From<AiProfile> for papr_core::ai::AiProfile {
    fn from(profile: AiProfile) -> Self {
        let headers = profile
            .headers
            .into_iter()
            .map(|header| (header.name, header.value))
            .collect::<BTreeMap<_, _>>();
        Self {
            id: profile.id,
            name: profile.name,
            protocol: profile.protocol.into(),
            model: profile.model,
            base_url: profile.base_url,
            auth: profile.auth.into(),
            headers,
            credential_ref: profile.credential_ref,
            enabled: profile.enabled,
            default_for: profile.default_for.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<papr_core::ai::AiProfile> for AiProfile {
    fn from(profile: papr_core::ai::AiProfile) -> Self {
        Self {
            id: profile.id,
            name: profile.name,
            protocol: profile.protocol.into(),
            model: profile.model,
            base_url: profile.base_url,
            auth: profile.auth.into(),
            headers: profile
                .headers
                .into_iter()
                .map(|(name, value)| AiHeader { name, value })
                .collect(),
            credential_ref: profile.credential_ref,
            enabled: profile.enabled,
            default_for: profile.default_for.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SummaryTemplate> for papr_core::SummaryTemplate {
    fn from(template: SummaryTemplate) -> Self {
        match template {
            SummaryTemplate::Classic => Self::Classic,
            SummaryTemplate::News5w1h => Self::News5w1h,
            SummaryTemplate::Decision => Self::Decision,
            SummaryTemplate::Funnel => Self::Funnel,
            SummaryTemplate::Argument => Self::Argument,
            SummaryTemplate::Minimal => Self::Minimal,
        }
    }
}

impl From<papr_core::AiSummaryCache> for AiSummaryCache {
    fn from(cache: papr_core::AiSummaryCache) -> Self {
        Self {
            summary: cache.summary,
            template: cache.template,
            language: cache.language,
        }
    }
}

impl From<papr_core::ai::AiStreamEvent> for AiStreamEvent {
    fn from(event: papr_core::ai::AiStreamEvent) -> Self {
        match event {
            papr_core::ai::AiStreamEvent::Delta { request_id, text } => {
                Self::Delta { request_id, text }
            }
            papr_core::ai::AiStreamEvent::Progress {
                request_id,
                completed,
                total,
            } => Self::Progress {
                request_id,
                completed,
                total,
            },
            papr_core::ai::AiStreamEvent::Completed { request_id } => {
                Self::Completed { request_id }
            }
            papr_core::ai::AiStreamEvent::Error { request_id, code } => {
                Self::Error { request_id, code }
            }
        }
    }
}

impl From<PaprCoreConfig> for papr_core::PaprCoreConfig {
    fn from(c: PaprCoreConfig) -> Self {
        Self {
            data_dir: PathBuf::from(c.data_dir),
            database_path: PathBuf::from(c.database_path),
            cache_dir: c.cache_dir.map(PathBuf::from),
            log_dir: c.log_dir.map(PathBuf::from),
            log_level: c.log_level,
            http_timeout_secs: None,
            http_proxy: None,
            http_user_agent: None,
            platform: c.platform.into(),
        }
    }
}

impl From<Platform> for papr_core::Platform {
    fn from(p: Platform) -> Self {
        match p {
            Platform::Desktop => Self::Desktop,
            Platform::Android => Self::Android,
            Platform::Ios => Self::Ios,
        }
    }
}

impl From<papr_core::SourceType> for SourceType {
    fn from(s: papr_core::SourceType) -> Self {
        match s {
            papr_core::SourceType::Rss => Self::Rss,
            papr_core::SourceType::Youtube => Self::Youtube,
            papr_core::SourceType::Podcast => Self::Podcast,
            papr_core::SourceType::Mastodon => Self::Mastodon,
            papr_core::SourceType::Bluesky => Self::Bluesky,
            papr_core::SourceType::Reddit => Self::Reddit,
            papr_core::SourceType::Newsletter => Self::Newsletter,
        }
    }
}

impl From<papr_core::Feed> for Feed {
    fn from(f: papr_core::Feed) -> Self {
        Self {
            id: f.id,
            feed_url: f.feed_url,
            site_url: f.site_url,
            title: f.title,
            description: f.description,
            favicon_url: f.favicon_url,
            folder_id: f.folder_id,
            source_type: f.source_type.into(),
            last_fetched_at: f.last_fetched_at,
            fetch_error: f.fetch_error,
            unread_count: f.unread_count,
            custom_title: f.custom_title,
            refresh_interval_min: f.refresh_interval_min,
        }
    }
}

impl From<papr_core::Folder> for Folder {
    fn from(f: papr_core::Folder) -> Self {
        Self {
            id: f.id,
            name: f.name,
            position: f.position,
        }
    }
}

impl From<papr_core::DiscoveryResult> for DiscoveryResult {
    fn from(d: papr_core::DiscoveryResult) -> Self {
        Self {
            title: d.title,
            feed_url: d.feed_url,
            site_url: d.site_url,
            category: d.category,
            description: d.description,
            from_directory: d.from_directory,
        }
    }
}

impl From<papr_core::ArticleSummary> for ArticleSummary {
    fn from(a: papr_core::ArticleSummary) -> Self {
        Self {
            id: a.id,
            feed_id: a.feed_id,
            feed_title: a.feed_title,
            source_type: a.source_type.into(),
            title: a.title,
            author: a.author,
            snippet: a.snippet,
            image_url: a.image_url,
            url: a.url,
            published_at: a.published_at,
            is_read: a.is_read,
            is_starred: a.is_starred,
            read_later: a.read_later,
        }
    }
}

impl From<papr_core::Enclosure> for Enclosure {
    fn from(e: papr_core::Enclosure) -> Self {
        Self {
            url: e.url,
            mime_type: e.mime_type,
            length: e.length,
        }
    }
}

impl From<papr_core::Tag> for Tag {
    fn from(t: papr_core::Tag) -> Self {
        Self {
            id: t.id,
            name: t.name,
            color: t.color,
        }
    }
}

impl From<ArticleFilterKind> for papr_core::ArticleFilterKind {
    fn from(k: ArticleFilterKind) -> Self {
        match k {
            ArticleFilterKind::All => Self::All,
            ArticleFilterKind::Unread => Self::Unread,
            ArticleFilterKind::Starred => Self::Starred,
            ArticleFilterKind::ReadLater => Self::ReadLater,
            ArticleFilterKind::Feed { feed_id } => Self::Feed(feed_id),
            ArticleFilterKind::Folder { folder_id } => Self::Folder(folder_id),
            ArticleFilterKind::Tag { tag_id } => Self::Tag(tag_id),
        }
    }
}

impl From<ArticleFilter> for papr_core::ArticleFilter {
    fn from(f: ArticleFilter) -> Self {
        Self {
            kind: f.kind.into(),
            search: f.search,
            unread_only: f.unread_only,
            oldest_first: f.oldest_first,
            limit: f.limit,
            offset: f.offset,
        }
    }
}

impl From<papr_core::ArticleCounts> for ArticleCounts {
    fn from(c: papr_core::ArticleCounts) -> Self {
        Self {
            all: c.all,
            unread: c.unread,
            starred: c.starred,
            read_later: c.read_later,
        }
    }
}

impl From<papr_core::TagSummary> for TagSummary {
    fn from(t: papr_core::TagSummary) -> Self {
        Self {
            id: t.id,
            name: t.name,
            color: t.color,
            article_count: t.article_count,
            position: t.position,
        }
    }
}

impl From<papr_core::Rule> for Rule {
    fn from(rule: papr_core::Rule) -> Self {
        Self {
            id: rule.id,
            name: rule.name,
            enabled: rule.enabled,
            feed_id: rule.feed_id,
            field: rule.field,
            query: rule.query,
            action: rule.action,
            position: rule.position,
        }
    }
}

impl From<RuleInput> for papr_core::RuleInput {
    fn from(input: RuleInput) -> Self {
        Self {
            name: input.name,
            enabled: input.enabled,
            feed_id: input.feed_id,
            field: input.field,
            query: input.query,
            action: input.action,
        }
    }
}

impl From<papr_core::RulePreview> for RulePreview {
    fn from(preview: papr_core::RulePreview) -> Self {
        Self {
            count: preview.count,
            samples: preview.samples,
        }
    }
}

impl From<papr_core::Highlight> for Highlight {
    fn from(highlight: papr_core::Highlight) -> Self {
        Self {
            id: highlight.id,
            article_id: highlight.article_id,
            quote: highlight.quote,
            prefix: highlight.prefix,
            suffix: highlight.suffix,
            text_offset: highlight.text_offset,
            color: highlight.color,
            note: highlight.note,
            created_at: highlight.created_at,
        }
    }
}

impl From<HighlightInput> for papr_core::HighlightInput {
    fn from(input: HighlightInput) -> Self {
        Self {
            article_id: input.article_id,
            quote: input.quote,
            prefix: input.prefix,
            suffix: input.suffix,
            text_offset: input.text_offset,
            color: input.color,
            note: input.note,
        }
    }
}

impl From<papr_core::ResolvedHighlight> for ResolvedHighlight {
    fn from(resolved: papr_core::ResolvedHighlight) -> Self {
        Self {
            highlight: resolved.highlight.into(),
            start: resolved.start,
            end: resolved.end,
        }
    }
}

impl From<papr_core::ArticleDetail> for ArticleDetail {
    fn from(a: papr_core::ArticleDetail) -> Self {
        Self {
            id: a.id,
            feed_id: a.feed_id,
            feed_title: a.feed_title,
            source_type: a.source_type.into(),
            title: a.title,
            author: a.author,
            url: a.url,
            content_html: a.content_html,
            extracted_html: a.extracted_html,
            image_url: a.image_url,
            published_at: a.published_at,
            is_read: a.is_read,
            is_starred: a.is_starred,
            read_later: a.read_later,
            ai_summary: a.ai_summary,
            translated_html: a.translated_html,
            translated_lang: a.translated_lang,
            enclosures: a.enclosures.into_iter().map(Into::into).collect(),
            tags: a.tags.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<RefreshOptions> for papr_core::RefreshOptions {
    fn from(o: RefreshOptions) -> Self {
        Self {
            feed_ids: o.feed_ids,
            force: o.force,
        }
    }
}

impl From<papr_core::RefreshError> for RefreshError {
    fn from(e: papr_core::RefreshError) -> Self {
        Self {
            feed_id: e.feed_id,
            message: e.message,
        }
    }
}

impl From<papr_core::RefreshReport> for RefreshReport {
    fn from(r: papr_core::RefreshReport) -> Self {
        Self {
            total_feeds: r.total_feeds,
            new_articles: r.new_articles,
            errors: r.errors.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<papr_core::OpmlImportReport> for OpmlImportReport {
    fn from(r: papr_core::OpmlImportReport) -> Self {
        Self {
            imported_feeds: r.imported_feeds,
            failed_feeds: r.failed_feeds,
            errors: r.errors,
        }
    }
}

impl From<papr_core::SettingsSnapshot> for SettingsSnapshot {
    fn from(s: papr_core::SettingsSnapshot) -> Self {
        Self {
            theme: s.theme,
            language: s.language,
            refresh_interval_min: s.refresh_interval_min,
            notifications_enabled: s.notifications_enabled,
            notification_quiet_hours: s.notification_quiet_hours,
            reading: s.reading.into(),
        }
    }
}

impl From<papr_core::ReadingSettings> for ReadingSettings {
    fn from(s: papr_core::ReadingSettings) -> Self {
        Self {
            font: s.font,
            font_size: s.font_size,
            line_height: s.line_height,
            content_width: s.content_width,
            show_reading_time: s.show_reading_time,
            auto_extract: s.auto_extract,
        }
    }
}

impl From<ReadingSettings> for papr_core::ReadingSettings {
    fn from(s: ReadingSettings) -> Self {
        Self {
            font: s.font,
            font_size: s.font_size,
            line_height: s.line_height,
            content_width: s.content_width,
            show_reading_time: s.show_reading_time,
            auto_extract: s.auto_extract,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{parse_deep_link, AiRequestRegistry};

    #[test]
    fn deep_link_api_returns_only_valid_subscription_targets() {
        assert_eq!(
            parse_deep_link(
                "papr://subscribe?url=https%3A%2F%2Fexample.com%2Ffeed.xml".to_string()
            ),
            Some("https://example.com/feed.xml".to_string())
        );
        assert_eq!(parse_deep_link("https://example.com".to_string()), None);
    }

    #[test]
    fn ai_request_cancellation_is_idempotent_and_lease_cleans_up() {
        let registry = Arc::new(AiRequestRegistry::new());
        let lease = registry.register("request-1").unwrap();

        assert!(registry.cancel("request-1"));
        assert!(registry.cancel("request-1"));
        assert!(lease.cancellation.is_cancelled());

        drop(lease);
        assert!(!registry.cancel("request-1"));
    }

    #[test]
    fn duplicate_ai_request_id_does_not_replace_active_cancellation() {
        let registry = Arc::new(AiRequestRegistry::new());
        let lease = registry.register("request-1").unwrap();

        let error = match registry.register("request-1") {
            Ok(_) => panic!("duplicate request should fail"),
            Err(error) => error,
        };
        assert_eq!(error.code, "invalidInput");
        assert!(registry.cancel("request-1"));
        assert!(lease.cancellation.is_cancelled());
    }
}
