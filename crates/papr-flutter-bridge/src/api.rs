//! Flutter Rust Bridge v2 public API.
//!
//! This file declares the exact Rust surface exposed to Dart. All business
//! logic lives in `papr-core`; this crate is only a thin wrapper plus DTO
//! conversion.

use std::path::PathBuf;
use std::sync::Arc;

use flutter_rust_bridge::frb;
use papr_core::PaprCore;

use crate::dto::{
    ArticleDetail, ArticleFilter, ArticleFilterKind, ArticleSummary, Enclosure, Feed,
    OpmlImportReport, PaprCoreConfig, Platform, RefreshError, RefreshOptions, RefreshReport,
    SettingsSnapshot, SourceType, Tag,
};
use crate::error::PaprBridgeError;

/// Opaque handle to a `PaprCore` instance.
///
/// Dart owns the lifetime of this handle via Riverpod. The inner `PaprCore`
/// is reference-counted so it can be used across multiple bridge calls.
#[frb(opaque)]
pub struct PaprCoreBridge {
    inner: Arc<PaprCore>,
}

/// Optional initialisation hook called by FRB before the first API use.
#[frb(init)]
pub fn init_app() {
    // Placeholder for panic hooks or logging initialisation.
}

/// Initialise `PaprCore` with platform-provided configuration.
pub async fn init_papr_core(
    config: PaprCoreConfig,
) -> Result<PaprCoreBridge, PaprBridgeError> {
    let core = PaprCore::new(config.into()).await?;
    Ok(PaprCoreBridge {
        inner: Arc::new(core),
    })
}

/// List all feeds.
pub async fn get_feeds(core: &PaprCoreBridge) -> Result<Vec<Feed>, PaprBridgeError> {
    let feeds = core.inner.feed_service().list_feeds().await?;
    Ok(feeds.into_iter().map(Into::into).collect())
}

/// Add a new feed by URL.
pub async fn add_feed(
    core: &PaprCoreBridge,
    feed_url: String,
) -> Result<Feed, PaprBridgeError> {
    let feed = core.inner.feed_service().add_feed(feed_url).await?;
    Ok(feed.into())
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

/// Read a snapshot of user settings.
pub async fn get_settings(
    core: &PaprCoreBridge,
) -> Result<SettingsSnapshot, PaprBridgeError> {
    let snapshot = core.inner.settings_service().get_settings().await?;
    Ok(snapshot.into())
}

// ---------------------------------------------------------------------------
// DTO conversions: bridge DTOs ↔ papr-core DTOs
// ---------------------------------------------------------------------------

impl From<PaprCoreConfig> for papr_core::PaprCoreConfig {
    fn from(c: PaprCoreConfig) -> Self {
        Self {
            data_dir: PathBuf::from(c.data_dir),
            database_path: PathBuf::from(c.database_path),
            cache_dir: c.cache_dir.map(PathBuf::from),
            log_dir: c.log_dir.map(PathBuf::from),
            log_level: c.log_level,
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
            limit: f.limit,
            offset: f.offset,
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
        }
    }
}
