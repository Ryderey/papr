//! FRB-annotated DTOs for the Flutter bridge.
//!
//! These mirror the types in `papr-core::dto` but carry `#[frb]` attributes.
//! Conversions between core and bridge DTOs live in `api.rs`.

use flutter_rust_bridge::frb;

/// Supported target platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[frb]
pub enum Platform {
    Desktop,
    Android,
    Ios,
}

/// Runtime configuration injected by the Flutter adapter.
#[derive(Debug, Clone)]
#[frb]
pub struct PaprCoreConfig {
    pub data_dir: String,
    pub database_path: String,
    pub cache_dir: Option<String>,
    pub log_dir: Option<String>,
    pub log_level: Option<String>,
    pub platform: Platform,
}

/// The kind of source a feed represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[frb]
pub enum SourceType {
    Rss,
    Youtube,
    Podcast,
    Mastodon,
    Bluesky,
    Reddit,
    Newsletter,
}

/// A feed or subscription.
#[derive(Debug, Clone)]
#[frb]
pub struct Feed {
    pub id: i64,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub folder_id: Option<i64>,
    pub source_type: SourceType,
    pub last_fetched_at: Option<String>,
    pub fetch_error: Option<String>,
    pub unread_count: i64,
    pub custom_title: bool,
    pub refresh_interval_min: Option<i64>,
}

/// A folder that groups feeds.
#[derive(Debug, Clone)]
#[frb]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub position: i64,
}

/// Input accepted by the full add-subscription pipeline.
#[derive(Debug, Clone)]
#[frb]
pub struct AddFeedInput {
    pub input: String,
}

/// A curated feed-directory result.
#[derive(Debug, Clone)]
#[frb]
pub struct DiscoveryResult {
    pub title: String,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub from_directory: bool,
}

/// A lightweight article row for lists.
#[derive(Debug, Clone)]
#[frb]
pub struct ArticleSummary {
    pub id: i64,
    pub feed_id: i64,
    pub feed_title: String,
    pub source_type: SourceType,
    pub title: String,
    pub author: Option<String>,
    pub snippet: Option<String>,
    pub image_url: Option<String>,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub read_later: bool,
}

/// A media enclosure attached to an article.
#[derive(Debug, Clone)]
#[frb]
pub struct Enclosure {
    pub url: String,
    pub mime_type: Option<String>,
    pub length: Option<i64>,
}

/// A lightweight tag representation.
#[derive(Debug, Clone)]
#[frb]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// Filter kind for article lists.
#[derive(Debug, Clone)]
#[frb]
pub enum ArticleFilterKind {
    All,
    Unread,
    Starred,
    ReadLater,
    Feed { feed_id: i64 },
    Folder { folder_id: i64 },
    Tag { tag_id: i64 },
}

/// Filter for paginated article lists.
#[derive(Debug, Clone)]
#[frb]
pub struct ArticleFilter {
    pub kind: ArticleFilterKind,
    pub search: Option<String>,
    pub unread_only: bool,
    pub oldest_first: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Counts shown beside the built-in article smart views.
#[derive(Debug, Clone)]
#[frb]
pub struct ArticleCounts {
    pub all: i64,
    pub unread: i64,
    pub starred: i64,
    pub read_later: i64,
}

/// A tag exposed for read-only article filtering.
#[derive(Debug, Clone)]
#[frb]
pub struct TagSummary {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub article_count: i64,
    pub position: i64,
}

/// A user-defined filter applied to incoming and existing articles.
#[derive(Debug, Clone)]
#[frb]
pub struct Rule {
    pub id: i64,
    pub name: String,
    pub enabled: bool,
    pub feed_id: Option<i64>,
    pub field: String,
    pub query: String,
    pub action: String,
    pub position: i64,
}

/// Rule form data accepted by Core validation.
#[derive(Debug, Clone)]
#[frb]
pub struct RuleInput {
    pub name: String,
    pub enabled: bool,
    pub feed_id: Option<i64>,
    pub field: String,
    pub query: String,
    pub action: String,
}

/// A rule preview against already persisted articles.
#[derive(Debug, Clone)]
#[frb]
pub struct RulePreview {
    pub count: i64,
    pub samples: Vec<String>,
}

/// A persisted reader annotation anchored to plain article text.
#[derive(Debug, Clone)]
#[frb]
pub struct Highlight {
    pub id: i64,
    pub article_id: i64,
    pub quote: String,
    pub prefix: String,
    pub suffix: String,
    pub text_offset: i64,
    pub color: String,
    pub note: String,
    pub created_at: String,
}

/// Input used to persist a new highlight from a reader selection.
#[derive(Debug, Clone)]
#[frb]
pub struct HighlightInput {
    pub article_id: i64,
    pub quote: String,
    pub prefix: String,
    pub suffix: String,
    pub text_offset: i64,
    pub color: String,
    pub note: String,
}

/// A highlight plus its current UTF-16 range, or unresolved coordinates.
#[derive(Debug, Clone)]
#[frb]
pub struct ResolvedHighlight {
    pub highlight: Highlight,
    pub start: Option<i64>,
    pub end: Option<i64>,
}

/// The full article shown in the reader.
#[derive(Debug, Clone)]
#[frb]
pub struct ArticleDetail {
    pub id: i64,
    pub feed_id: i64,
    pub feed_title: String,
    pub source_type: SourceType,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub content_html: Option<String>,
    pub extracted_html: Option<String>,
    pub image_url: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub read_later: bool,
    pub ai_summary: Option<String>,
    pub translated_html: Option<String>,
    pub translated_lang: Option<String>,
    pub enclosures: Vec<Enclosure>,
    pub tags: Vec<Tag>,
}

/// Options for a feed refresh run.
#[derive(Debug, Clone)]
#[frb]
pub struct RefreshOptions {
    pub feed_ids: Option<Vec<i64>>,
    pub force: bool,
}

/// Per-feed error inside a refresh report.
#[derive(Debug, Clone)]
#[frb]
pub struct RefreshError {
    pub feed_id: i64,
    pub message: String,
}

/// Result of a feed refresh run.
#[derive(Debug, Clone)]
#[frb]
pub struct RefreshReport {
    pub total_feeds: i64,
    pub new_articles: i64,
    pub errors: Vec<RefreshError>,
}

/// Result of an OPML text import.
#[derive(Debug, Clone)]
#[frb]
pub struct OpmlImportReport {
    pub imported_feeds: i64,
    pub failed_feeds: i64,
    pub errors: Vec<String>,
}

/// A snapshot of user-facing settings.
#[derive(Debug, Clone)]
#[frb]
pub struct SettingsSnapshot {
    pub theme: String,
    pub language: String,
    pub refresh_interval_min: i64,
    pub reading: ReadingSettings,
}

/// Reader appearance and behaviour settings.
#[derive(Debug, Clone)]
#[frb]
pub struct ReadingSettings {
    pub font: String,
    pub font_size: f64,
    pub line_height: f64,
    pub content_width: f64,
    pub show_reading_time: bool,
    pub auto_extract: bool,
}
