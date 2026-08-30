//! Platform-agnostic data transfer objects for phase 1.

/// The kind of source a feed represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Rss,
    Youtube,
    Podcast,
    Mastodon,
    Bluesky,
    Reddit,
    Newsletter,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::Rss => "rss",
            SourceType::Youtube => "youtube",
            SourceType::Podcast => "podcast",
            SourceType::Mastodon => "mastodon",
            SourceType::Bluesky => "bluesky",
            SourceType::Reddit => "reddit",
            SourceType::Newsletter => "newsletter",
        }
    }
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A feed or subscription.
#[derive(Debug, Clone)]
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
    /// Whether the user manually renamed this feed.
    pub custom_title: bool,
    /// Per-feed refresh interval in minutes. `None` follows the global
    /// `refresh_interval_min`; the `525_600` sentinel means "never".
    pub refresh_interval_min: Option<i64>,
}

/// A folder that groups feeds.
#[derive(Debug, Clone)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub position: i64,
}

/// A media enclosure attached to an article.
#[derive(Debug, Clone)]
pub struct Enclosure {
    pub url: String,
    pub mime_type: Option<String>,
    pub length: Option<i64>,
}

/// A parsed article ready for insertion.
#[derive(Debug, Clone)]
pub struct NewArticle {
    pub guid: String,
    pub url: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub content_html: Option<String>,
    pub body_text: String,
    pub image_url: Option<String>,
    pub published_at: Option<String>,
    pub enclosures: Vec<Enclosure>,
}

/// A lightweight article row for lists.
#[derive(Debug, Clone)]
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

/// The full article shown in the reader.
#[derive(Debug, Clone)]
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

/// Supported formats for an AI-generated article summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryTemplate {
    Classic,
    News5w1h,
    Decision,
    Funnel,
    Argument,
    Minimal,
}

impl SummaryTemplate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::News5w1h => "news5w1h",
            Self::Decision => "decision",
            Self::Funnel => "funnel",
            Self::Argument => "argument",
            Self::Minimal => "minimal",
        }
    }
}

/// The most recent fully generated summary for one article.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSummaryCache {
    pub summary: String,
    /// `None` identifies a summary written before template metadata existed.
    pub template: Option<String>,
    /// `None` identifies a summary written before language metadata existed.
    pub language: Option<String>,
}

/// A lightweight tag representation.
#[derive(Debug, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// Filter kind for article lists.
#[derive(Debug, Clone)]
pub enum ArticleFilterKind {
    All,
    Unread,
    Starred,
    ReadLater,
    Feed(i64),
    Folder(i64),
    Tag(i64),
}

/// Filter for paginated article lists.
#[derive(Debug, Clone)]
pub struct ArticleFilter {
    pub kind: ArticleFilterKind,
    pub search: Option<String>,
    pub unread_only: bool,
    pub oldest_first: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for ArticleFilter {
    fn default() -> Self {
        Self {
            kind: ArticleFilterKind::All,
            search: None,
            unread_only: false,
            oldest_first: false,
            limit: Some(50),
            offset: Some(0),
        }
    }
}

/// Counts shown by the article smart views.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArticleCounts {
    pub all: i64,
    pub unread: i64,
    pub starred: i64,
    pub read_later: i64,
}

/// A tag exposed for read-only article filtering in phase 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagSummary {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub article_count: i64,
    pub position: i64,
}

/// A user-defined filter applied to incoming and existing articles.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// A draft rule submitted by an adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleInput {
    pub name: String,
    pub enabled: bool,
    pub feed_id: Option<i64>,
    pub field: String,
    pub query: String,
    pub action: String,
}

/// A dry-run of a rule against the existing article store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePreview {
    pub count: i64,
    pub samples: Vec<String>,
}

/// A persisted user annotation anchored to article plain text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Highlight {
    pub id: i64,
    pub article_id: i64,
    pub quote: String,
    pub prefix: String,
    pub suffix: String,
    /// UTF-16 code-unit offset, matching Flutter and the desktop webview.
    pub text_offset: i64,
    pub color: String,
    pub note: String,
    pub created_at: String,
}

/// Input for creating a highlight; IDs and timestamps are assigned by Core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightInput {
    pub article_id: i64,
    pub quote: String,
    pub prefix: String,
    pub suffix: String,
    pub text_offset: i64,
    pub color: String,
    pub note: String,
}

/// A stored highlight plus its current location in reader text, if found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedHighlight {
    pub highlight: Highlight,
    pub start: Option<i64>,
    pub end: Option<i64>,
}

/// Options for a feed refresh run.
#[derive(Debug, Clone, Default)]
pub struct RefreshOptions {
    /// Refresh only these feed IDs. `None` means refresh all feeds.
    pub feed_ids: Option<Vec<i64>>,
    /// Force refresh even if the feed was recently fetched.
    pub force: bool,
}

/// Per-feed error inside a refresh report.
#[derive(Debug, Clone)]
pub struct RefreshError {
    pub feed_id: i64,
    pub message: String,
}

/// Result of a feed refresh run.
#[derive(Debug, Clone)]
pub struct RefreshReport {
    pub total_feeds: i64,
    pub new_articles: i64,
    pub errors: Vec<RefreshError>,
}

/// Result of an OPML text import.
#[derive(Debug, Clone)]
pub struct OpmlImportReport {
    pub imported_feeds: i64,
    pub failed_feeds: i64,
    pub errors: Vec<String>,
}

/// A snapshot of user-facing settings.
#[derive(Debug, Clone)]
pub struct SettingsSnapshot {
    pub theme: String,
    pub language: String,
    pub refresh_interval_min: i64,
    pub notifications_enabled: bool,
    pub notification_quiet_hours: bool,
    pub reading: ReadingSettings,
}

/// Reader appearance and behaviour persisted by the Core settings service.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadingSettings {
    pub font: String,
    pub font_size: f64,
    pub line_height: f64,
    pub content_width: f64,
    pub show_reading_time: bool,
    pub auto_extract: bool,
}

impl Default for ReadingSettings {
    fn default() -> Self {
        Self {
            font: "system".to_string(),
            font_size: 17.0,
            line_height: 1.65,
            content_width: 680.0,
            show_reading_time: true,
            auto_extract: false,
        }
    }
}

impl Default for SettingsSnapshot {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "en".to_string(),
            refresh_interval_min: 30,
            notifications_enabled: false,
            notification_quiet_hours: false,
            reading: ReadingSettings::default(),
        }
    }
}

/// A single feed-discovery result surfaced to the UI.
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Display name of the feed.
    pub title: String,
    /// The subscribable feed URL — passed straight to `add_feed`.
    pub feed_url: String,
    /// The website the feed belongs to, when known.
    pub site_url: Option<String>,
    /// Category for directory entries; `None` for live page scrapes.
    pub category: Option<String>,
    /// Short description, when known.
    pub description: Option<String>,
    /// `true` when the result came from the curated directory, `false` when
    /// it was scraped live from a page the user pasted.
    pub from_directory: bool,
}
