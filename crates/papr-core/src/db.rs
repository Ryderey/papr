//! SQLite data layer.
//!
//! This module is a phase-1 subset focused on the six bridge APIs. It will grow
//! as more features migrate from `src-tauri/src/db.rs`.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension};
use rusqlite_migration::{M, Migrations};

use crate::dto::{
    ArticleDetail, ArticleFilter, ArticleFilterKind, ArticleSummary, Enclosure, Feed, NewArticle,
    SourceType,
};
use crate::error::CoreError;

/// Thread-safe SQLite handle.
///
/// `rusqlite::Connection` is `Send` but not `Sync`; wrapping it in a `Mutex`
/// makes `Db` both `Send` and `Sync`, which is required because `PaprCore`
/// (which owns a `Db`) is held across the Flutter Rust Bridge as an opaque
/// object and may be accessed from multiple threads.
pub struct Db {
    conn: Mutex<Connection>,
}

/// Information needed to refresh one feed.
#[derive(Debug, Clone)]
pub struct FeedRefreshInfo {
    pub id: i64,
    pub feed_url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl Db {
    /// Open or create the SQLite database and run migrations.
    pub fn new(path: &Path) -> Result<Self, CoreError> {
        let mut conn = Connection::open(path).map_err(|e| {
            CoreError::Db(format!("failed to open database at {}: {}", path.display(), e))
        })?;

        let migrations = Migrations::new(vec![M::up(
            r#"
            CREATE TABLE IF NOT EXISTS folders (
                id        INTEGER PRIMARY KEY,
                name      TEXT NOT NULL,
                position  INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS feeds (
                id              INTEGER PRIMARY KEY,
                feed_url        TEXT NOT NULL UNIQUE,
                site_url        TEXT,
                title           TEXT NOT NULL,
                description     TEXT,
                favicon_url     TEXT,
                folder_id       INTEGER REFERENCES folders(id) ON DELETE SET NULL,
                source_type     TEXT NOT NULL DEFAULT 'rss',
                etag            TEXT,
                last_modified   TEXT,
                last_fetched_at TEXT,
                fetch_error     TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS articles (
                id             INTEGER PRIMARY KEY,
                feed_id        INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
                guid           TEXT NOT NULL,
                url            TEXT,
                title          TEXT NOT NULL,
                author         TEXT,
                summary        TEXT,
                content_html   TEXT,
                extracted_html TEXT,
                body_text      TEXT NOT NULL DEFAULT '',
                image_url      TEXT,
                ai_summary     TEXT,
                published_at   TEXT,
                fetched_at     TEXT NOT NULL DEFAULT (datetime('now')),
                is_read        INTEGER NOT NULL DEFAULT 0,
                is_starred     INTEGER NOT NULL DEFAULT 0,
                read_later     INTEGER NOT NULL DEFAULT 0,
                UNIQUE(feed_id, guid)
            );

            CREATE INDEX IF NOT EXISTS idx_articles_feed      ON articles(feed_id);
            CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published_at DESC);
            CREATE INDEX IF NOT EXISTS idx_articles_unread    ON articles(is_read) WHERE is_read = 0;

            CREATE TABLE IF NOT EXISTS enclosures (
                id         INTEGER PRIMARY KEY,
                article_id INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                url        TEXT NOT NULL,
                mime_type  TEXT,
                length     INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_enclosures_article ON enclosures(article_id);

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )]);

        migrations.to_latest(&mut conn).map_err(|e| {
            CoreError::Db(format!("failed to run database migrations: {}", e))
        })?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, CoreError> {
        self.conn.lock().map_err(|e| {
            CoreError::Db(format!("database mutex poisoned: {}", e))
        })
    }

    /// List all feeds.
    pub fn list_feeds(&self) -> Result<Vec<Feed>, CoreError> {
        let conn = self.lock()?;
        Self::list_feeds_locked(&conn)
    }

    fn list_feeds_locked(conn: &Connection) -> Result<Vec<Feed>, CoreError> {
        let mut stmt = conn.prepare(
            "SELECT id, feed_url, site_url, title, description, favicon_url, folder_id, source_type, last_fetched_at, fetch_error FROM feeds ORDER BY title"
        ).map_err(|e| CoreError::Db(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let source_type: String = row.get(7)?;
            Ok(Feed {
                id: row.get(0)?,
                feed_url: row.get(1)?,
                site_url: row.get(2)?,
                title: row.get(3)?,
                description: row.get(4)?,
                favicon_url: row.get(5)?,
                folder_id: row.get(6)?,
                source_type: parse_source_type(&source_type),
                last_fetched_at: row.get(8)?,
                fetch_error: row.get(9)?,
                unread_count: 0, // computed below
            })
        }).map_err(|e| CoreError::Db(e.to_string()))?;

        let mut feeds = Vec::new();
        for row in rows {
            feeds.push(row.map_err(|e| CoreError::Db(e.to_string()))?);
        }

        drop(stmt);

        // Compute unread counts.
        for feed in &mut feeds {
            feed.unread_count = Self::count_unread_articles_locked(conn, feed.id)?;
        }

        Ok(feeds)
    }

    /// Fetch a single feed by ID.
    pub fn get_feed(&self, id: i64) -> Result<Feed, CoreError> {
        let conn = self.lock()?;
        let mut feeds = Self::list_feeds_locked(&conn)?;
        feeds.retain(|f| f.id == id);
        feeds.into_iter().next().ok_or_else(|| {
            CoreError::NotFound(format!("feed {} not found", id))
        })
    }

    /// Insert a new feed and return its generated row ID.
    pub fn add_feed(&self, feed_url: &str) -> Result<i64, CoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO feeds (feed_url, title, source_type) VALUES (?1, ?2, 'rss')",
            [feed_url, feed_url],
        )
        .map_err(|e| {
            if is_unique_violation(&e) {
                CoreError::InvalidInput(format!("feed already exists: {}", feed_url))
            } else {
                CoreError::Db(e.to_string())
            }
        })?;
        Ok(conn.last_insert_rowid())
    }

    fn count_unread_articles_locked(
        conn: &Connection,
        feed_id: i64,
    ) -> Result<i64, CoreError> {
        conn.query_row(
            "SELECT COUNT(*) FROM articles WHERE feed_id = ?1 AND is_read = 0",
            [feed_id],
            |row| row.get::<usize, i64>(0),
        )
        .map_err(|e| CoreError::Db(e.to_string()))
    }

    /// List articles matching a filter.
    pub fn list_articles(
        &self, filter: &ArticleFilter) -> Result<Vec<ArticleSummary>, CoreError> {
        let conn = self.lock()?;
        let mut clauses = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        match &filter.kind {
            ArticleFilterKind::All => {}
            ArticleFilterKind::Unread => {
                clauses.push("articles.is_read = 0".to_string());
            }
            ArticleFilterKind::Starred => {
                clauses.push("articles.is_starred = 1".to_string());
            }
            ArticleFilterKind::ReadLater => {
                clauses.push("articles.read_later = 1".to_string());
            }
            ArticleFilterKind::Feed(id) => {
                clauses.push("articles.feed_id = ?".to_string());
                params.push((*id).into());
            }
            ArticleFilterKind::Folder(id) => {
                clauses.push("feeds.folder_id = ?".to_string());
                params.push((*id).into());
            }
            ArticleFilterKind::Tag(_id) => {
                // Phase 1: tags are not yet stored separately; fall back to All.
            }
        }

        let where_sql = if clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", clauses.join(" AND "))
        };

        let mut sql = format!(
            "SELECT articles.id, articles.feed_id, feeds.title, feeds.source_type, articles.title, articles.author, articles.summary, articles.image_url, articles.url, articles.published_at, articles.is_read, articles.is_starred, articles.read_later \
             FROM articles \
             JOIN feeds ON feeds.id = articles.feed_id \
             {where_sql} \
             ORDER BY articles.published_at DESC NULLS LAST"
        );

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();

        let mut stmt = conn.prepare(&sql).map_err(|e| CoreError::Db(e.to_string()))?;

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let source_type: String = row.get(3)?;
            Ok(ArticleSummary {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                feed_title: row.get(2)?,
                source_type: parse_source_type(&source_type),
                title: row.get(4)?,
                author: row.get(5)?,
                snippet: row.get(6)?,
                image_url: row.get(7)?,
                url: row.get(8)?,
                published_at: row.get(9)?,
                is_read: row.get::<usize, i64>(10)? != 0,
                is_starred: row.get::<usize, i64>(11)? != 0,
                read_later: row.get::<usize, i64>(12)? != 0,
            })
        }).map_err(|e| CoreError::Db(e.to_string()))?;

        let mut articles = Vec::new();
        for row in rows {
            articles.push(row.map_err(|e| CoreError::Db(e.to_string()))?);
        }

        Ok(articles)
    }

    /// Fetch a single article by ID.
    pub fn get_article_detail(
        &self, article_id: i64) -> Result<ArticleDetail, CoreError> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT articles.id, articles.feed_id, feeds.title, feeds.source_type, articles.title, articles.author, articles.url, articles.content_html, articles.extracted_html, articles.image_url, articles.published_at, articles.is_read, articles.is_starred, articles.read_later, articles.ai_summary, articles.translated_html, articles.translated_lang \
             FROM articles \
             JOIN feeds ON feeds.id = articles.feed_id \
             WHERE articles.id = ?1"
        ).map_err(|e| CoreError::Db(e.to_string()))?;

        let mut row = stmt.query_row([article_id], |row| {
            let source_type: String = row.get(3)?;
            Ok(ArticleDetail {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                feed_title: row.get(2)?,
                source_type: parse_source_type(&source_type),
                title: row.get(4)?,
                author: row.get(5)?,
                url: row.get(6)?,
                content_html: row.get(7)?,
                extracted_html: row.get(8)?,
                image_url: row.get(9)?,
                published_at: row.get(10)?,
                is_read: row.get::<usize, i64>(11)? != 0,
                is_starred: row.get::<usize, i64>(12)? != 0,
                read_later: row.get::<usize, i64>(13)? != 0,
                ai_summary: row.get(14)?,
                translated_html: row.get(15)?,
                translated_lang: row.get(16)?,
                enclosures: Vec::new(), // loaded below
                tags: Vec::new(),       // phase 1: not loaded
            })
        }).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CoreError::NotFound(format!("article {} not found", article_id))
            }
            _ => CoreError::Db(e.to_string()),
        })?;

        row.enclosures = Self::load_enclosures(&conn, article_id)?;

        Ok(row)
    }

    /// Read a single settings value.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, CoreError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [key],
            |row| row.get::<usize, String>(0),
        )
        .optional()
        .map_err(|e| CoreError::Db(e.to_string()))
    }

    /// Returns all feeds that should be refreshed.
    pub fn feeds_to_refresh(&self,
    ) -> Result<Vec<FeedRefreshInfo>, CoreError> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, feed_url, etag, last_modified FROM feeds ORDER BY title",
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(FeedRefreshInfo {
                    id: row.get::<usize, i64>(0)?,
                    feed_url: row.get::<usize, String>(1)?,
                    etag: row.get::<usize, Option<String>>(2)?,
                    last_modified: row.get::<usize, Option<String>>(3)?,
                })
            })
            .map_err(|e| CoreError::Db(e.to_string()))?;

        let mut feeds = Vec::new();
        for row in rows {
            feeds.push(row.map_err(|e| CoreError::Db(e.to_string()))?);
        }
        Ok(feeds)
    }

    /// Upsert an article, using (feed_id, guid) as the dedup key.
    /// Returns true if a new row was inserted.
    pub fn upsert_article(
        &self,
        feed_id: i64,
        article: &NewArticle,
    ) -> Result<bool, CoreError> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO articles \
             (feed_id, guid, url, title, author, summary, content_html, body_text, image_url, published_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
             ON CONFLICT(feed_id, guid) DO NOTHING",
            (
                feed_id,
                &article.guid,
                &article.url,
                &article.title,
                &article.author,
                &article.summary,
                &article.content_html,
                &article.body_text,
                &article.image_url,
                &article.published_at,
            ),
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;

        let article_id = conn.last_insert_rowid();
        if article_id == 0 {
            return Ok(false);
        }

        for enc in &article.enclosures {
            conn.execute(
                "INSERT INTO enclosures (article_id, url, mime_type, length) VALUES (?1, ?2, ?3, ?4)",
                (article_id, &enc.url, &enc.mime_type, &enc.length),
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
        }
        Ok(true)
    }

    /// Update editable feed metadata after a successful refresh.
    pub fn update_feed_meta(
        &self,
        feed_id: i64,
        title: Option<&str>,
        site_url: Option<&str>,
        description: Option<&str>,
        favicon_url: Option<&str>,
    ) -> Result<(), CoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE feeds SET \
             title = COALESCE(?1, title), \
             site_url = COALESCE(?2, site_url), \
             description = COALESCE(?3, description), \
             favicon_url = COALESCE(?4, favicon_url) \
             WHERE id = ?5",
            (title, site_url, description, favicon_url, feed_id),
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Update fetch-related state after a refresh attempt.
    pub fn set_feed_fetch_state(
        &self,
        feed_id: i64,
        etag: Option<&str>,
        last_modified: Option<&str>,
        fetch_error: Option<&str>,
    ) -> Result<(), CoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE feeds SET \
             etag = COALESCE(?1, etag), \
             last_modified = COALESCE(?2, last_modified), \
             fetch_error = ?3 \
             WHERE id = ?4",
            (etag, last_modified, fetch_error, feed_id),
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Touch a feed's last_fetched_at timestamp.
    pub fn touch_feed(&self, feed_id: i64) -> Result<(), CoreError> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE feeds SET last_fetched_at = datetime('now') WHERE id = ?1",
            [feed_id],
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Load enclosures for a given article.
    fn load_enclosures(
        conn: &Connection,
        article_id: i64,
    ) -> Result<Vec<Enclosure>, CoreError> {
        let mut stmt = conn
            .prepare("SELECT url, mime_type, length FROM enclosures WHERE article_id = ?1")
            .map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([article_id], |row| {
                Ok(Enclosure {
                    url: row.get(0)?,
                    mime_type: row.get(1)?,
                    length: row.get(2)?,
                })
            })
            .map_err(|e| CoreError::Db(e.to_string()))?;
        rows.into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))
    }
}

fn parse_source_type(s: &str) -> SourceType {
    match s {
        "youtube" => SourceType::Youtube,
        "podcast" => SourceType::Podcast,
        "mastodon" => SourceType::Mastodon,
        "bluesky" => SourceType::Bluesky,
        "reddit" => SourceType::Reddit,
        "newsletter" => SourceType::Newsletter,
        _ => SourceType::Rss,
    }
}

fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(sqlite_err, _) if sqlite_err.code == rusqlite::ErrorCode::ConstraintViolation
    )
}
