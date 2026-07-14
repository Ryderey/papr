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

/// A media enclosure attached to an article.
#[derive(Debug, Clone)]
pub struct Enclosure {
    pub url: String,
    pub mime_type: Option<String>,
    pub length: Option<i64>,
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
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for ArticleFilter {
    fn default() -> Self {
        Self {
            kind: ArticleFilterKind::All,
            limit: None,
            offset: None,
        }
    }
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
}

impl Default for SettingsSnapshot {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "en".to_string(),
            refresh_interval_min: 30,
        }
    }
}
