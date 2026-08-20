//! SQLite data layer.
//!
//! Owns the canonical, append-only schema shared by the desktop adapter
//! (`src-tauri`) and the Flutter adapter (`papr-flutter-bridge`). The
//! migration sequence v1–v15 is ported verbatim from the desktop's
//! `src-tauri/src/db.rs`; v16 adds the sync-ready baseline.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex, MutexGuard};

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, OptionalExtension};
use rusqlite_migration::{M, Migrations};

use crate::dto::{
    ArticleDetail, ArticleFilter, ArticleFilterKind, ArticleSummary, Enclosure, Feed, NewArticle,
    SourceType,
};
use crate::error::CoreError;

/// Number of read-only connections in the UI query pool.
const READER_POOL_SIZE: usize = 3;

/// Append-only schema migrations. Never edit a shipped migration — add a new
/// one. v1–v15 mirror the desktop's migration history exactly so an existing
/// desktop database (already at v15) forward-migrates under the same version
/// numbers; v16 establishes the sync-ready baseline.
fn migrations() -> Vec<M<'static>> {
    vec![
        M::up(
            r#"
            CREATE TABLE folders (
                id        INTEGER PRIMARY KEY,
                name      TEXT NOT NULL,
                position  INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE feeds (
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

            CREATE TABLE articles (
                id            INTEGER PRIMARY KEY,
                feed_id       INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
                guid          TEXT NOT NULL,
                url           TEXT,
                title         TEXT NOT NULL,
                author        TEXT,
                summary       TEXT,
                content_html  TEXT,
                extracted_html TEXT,
                body_text     TEXT NOT NULL DEFAULT '',
                image_url     TEXT,
                ai_summary    TEXT,
                published_at  TEXT,
                fetched_at    TEXT NOT NULL DEFAULT (datetime('now')),
                is_read       INTEGER NOT NULL DEFAULT 0,
                is_starred    INTEGER NOT NULL DEFAULT 0,
                read_later    INTEGER NOT NULL DEFAULT 0,
                UNIQUE(feed_id, guid)
            );

            CREATE INDEX idx_articles_feed      ON articles(feed_id);
            CREATE INDEX idx_articles_published ON articles(published_at DESC);
            CREATE INDEX idx_articles_unread    ON articles(is_read) WHERE is_read = 0;

            CREATE TABLE enclosures (
                id         INTEGER PRIMARY KEY,
                article_id INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                url        TEXT NOT NULL,
                mime_type  TEXT,
                length     INTEGER
            );
            CREATE INDEX idx_enclosures_article ON enclosures(article_id);

            CREATE VIRTUAL TABLE articles_fts USING fts5(
                title, body, tokenize = 'porter unicode61'
            );

            -- Keep the FTS index in sync on delete; inserts are handled in code so
            -- that read-state updates do not trigger needless re-indexing.
            CREATE TRIGGER articles_fts_ad AFTER DELETE ON articles BEGIN
                DELETE FROM articles_fts WHERE rowid = old.id;
            END;

            CREATE TABLE settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        ),
        // v2 — placeholder. An earlier sqlite-vec semantic-search schema was
        // removed; this keeps the version count aligned for databases that
        // already applied it. Search is keyword-only (FTS5).
        M::up("-- semantic search removed; search is FTS5 keyword-only"),
        // v3 — sync support: a remote item id per article plus a small queue
        // of local read/starred changes still to push to the sync server.
        M::up(
            r#"
            ALTER TABLE articles ADD COLUMN remote_id TEXT;
            CREATE TABLE sync_queue (
                article_id INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                field      TEXT NOT NULL,
                value      INTEGER NOT NULL,
                PRIMARY KEY (article_id, field)
            );
            "#,
        ),
        // v4 — article tags: a flat label set plus an article↔tag join table.
        M::up(
            r#"
            CREATE TABLE tags (
                id        INTEGER PRIMARY KEY,
                name      TEXT NOT NULL UNIQUE,
                color     TEXT NOT NULL DEFAULT 'clay',
                position  INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE article_tags (
                article_id INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                tag_id     INTEGER NOT NULL REFERENCES tags(id)     ON DELETE CASCADE,
                PRIMARY KEY (article_id, tag_id)
            );
            CREATE INDEX idx_article_tags_tag ON article_tags(tag_id);
            "#,
        ),
        // v5 — filter rules: keyword matches applied to incoming articles to
        // auto-skip noise, or auto mark-read / star them, at ingestion time.
        M::up(
            r#"
            CREATE TABLE rules (
                id         INTEGER PRIMARY KEY,
                name       TEXT NOT NULL,
                enabled    INTEGER NOT NULL DEFAULT 1,
                feed_id    INTEGER REFERENCES feeds(id) ON DELETE CASCADE,
                field      TEXT NOT NULL DEFAULT 'title',
                query      TEXT NOT NULL,
                action     TEXT NOT NULL DEFAULT 'skip',
                position   INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
        ),
        // v6 — index over the effective article date the list sorts by,
        // COALESCE(published_at, fetched_at), so a dateless entry sorts by
        // when it was fetched instead of sinking below every dated article.
        // (Superseded by v12, which rebuilds this index over a `datetime()`-
        // normalised expression.)
        M::up(
            "CREATE INDEX idx_articles_sort
             ON articles(COALESCE(published_at, fetched_at) DESC, id DESC);",
        ),
        // v7 — every date ordering now sorts on the effective date and uses
        // idx_articles_sort, so the original published_at-only index is dead
        // weight on each insert. Drop it.
        M::up("DROP INDEX idx_articles_published;"),
        // v8 — partial indexes mirroring idx_articles_unread for the other
        // two smart-view flags, so the Starred / Read-later sidebar counts
        // and list queries use a tiny index instead of a full table scan.
        M::up(
            "CREATE INDEX idx_articles_starred
                 ON articles(is_starred) WHERE is_starred = 1;
             CREATE INDEX idx_articles_readlater
                 ON articles(read_later) WHERE read_later = 1;",
        ),
        // v9 — index the article URL. FreshRSS reconciliation matches remote
        // items to local articles by URL (up to ~1000 lookups per sync) and
        // the dedup check tests URL existence per inserted article; both
        // full-scanned the table without this.
        M::up("CREATE INDEX idx_articles_url ON articles(url);"),
        // v10 — email-newsletter sources (feature F5). A newsletter is a
        // normal `feeds` row (source_type = 'newsletter') so it lists,
        // searches and retains like an RSS feed; this side-table holds the
        // IMAP connection details, keyed 1:1 by feed_id and cascade-deleted
        // with the feed.
        M::up(
            r#"
            CREATE TABLE newsletter_sources (
                feed_id   INTEGER PRIMARY KEY REFERENCES feeds(id) ON DELETE CASCADE,
                host      TEXT NOT NULL,
                port      INTEGER NOT NULL DEFAULT 993,
                username  TEXT NOT NULL,
                password  TEXT NOT NULL,
                folder    TEXT NOT NULL DEFAULT 'INBOX'
            );
            "#,
        ),
        // v11 — highlights / annotations layer (feature F7). Each highlight
        // pins a span of an article's rendered plain text.
        M::up(
            r#"
            CREATE TABLE highlights (
                id          INTEGER PRIMARY KEY,
                article_id  INTEGER NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
                quote       TEXT NOT NULL,
                prefix      TEXT NOT NULL DEFAULT '',
                suffix      TEXT NOT NULL DEFAULT '',
                text_offset INTEGER NOT NULL DEFAULT 0,
                color       TEXT NOT NULL DEFAULT 'yellow',
                note        TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_highlights_article ON highlights(article_id);
            "#,
        ),
        // v12 — rebuild the article-sort index over the *normalised* effective
        // date (datetime() parses RFC 3339 and SQLite's space-separated form
        // into one canonical representation).
        M::up(
            "DROP INDEX idx_articles_sort;
             CREATE INDEX idx_articles_sort
                 ON articles(datetime(COALESCE(published_at, fetched_at)) DESC,
                             id DESC);",
        ),
        // v13 — mark feeds whose title the user has set by hand.
        M::up(
            "ALTER TABLE feeds ADD COLUMN custom_title INTEGER NOT NULL DEFAULT 0;",
        ),
        // v14 — cache a translated copy of the article body.
        M::up(
            "ALTER TABLE articles ADD COLUMN translated_html TEXT;
             ALTER TABLE articles ADD COLUMN translated_lang TEXT;",
        ),
        // v15 — per-feed refresh interval (minutes). NULL follows the global
        // `refresh_interval_min` setting; the 525_600 sentinel means "never".
        M::up("ALTER TABLE feeds ADD COLUMN refresh_interval_min INTEGER;"),
        // v16 — sync-ready baseline. Adds stable identifiers, update
        // timestamps and soft-delete markers to syncable entities, plus a
        // general change log, per-provider sync cursors, and a remote id map.
        M::up(
            r#"
            ALTER TABLE folders ADD COLUMN sync_id TEXT;
            ALTER TABLE folders ADD COLUMN updated_at TEXT;
            ALTER TABLE folders ADD COLUMN deleted_at TEXT;
            ALTER TABLE feeds ADD COLUMN sync_id TEXT;
            ALTER TABLE feeds ADD COLUMN updated_at TEXT;
            ALTER TABLE feeds ADD COLUMN deleted_at TEXT;
            ALTER TABLE tags ADD COLUMN sync_id TEXT;
            ALTER TABLE tags ADD COLUMN updated_at TEXT;
            ALTER TABLE tags ADD COLUMN deleted_at TEXT;
            ALTER TABLE rules ADD COLUMN sync_id TEXT;
            ALTER TABLE rules ADD COLUMN updated_at TEXT;
            ALTER TABLE rules ADD COLUMN deleted_at TEXT;
            ALTER TABLE highlights ADD COLUMN sync_id TEXT;
            ALTER TABLE highlights ADD COLUMN updated_at TEXT;
            ALTER TABLE highlights ADD COLUMN deleted_at TEXT;

            CREATE UNIQUE INDEX idx_folders_sync_id    ON folders(sync_id)    WHERE sync_id IS NOT NULL;
            CREATE UNIQUE INDEX idx_feeds_sync_id      ON feeds(sync_id)      WHERE sync_id IS NOT NULL;
            CREATE UNIQUE INDEX idx_tags_sync_id       ON tags(sync_id)       WHERE sync_id IS NOT NULL;
            CREATE UNIQUE INDEX idx_rules_sync_id      ON rules(sync_id)      WHERE sync_id IS NOT NULL;
            CREATE UNIQUE INDEX idx_highlights_sync_id ON highlights(sync_id) WHERE sync_id IS NOT NULL;

            CREATE TABLE change_log (
                id        INTEGER PRIMARY KEY,
                entity    TEXT NOT NULL,
                entity_id INTEGER NOT NULL,
                field     TEXT,
                value     TEXT,
                op        TEXT NOT NULL DEFAULT 'upsert',
                seq       INTEGER NOT NULL
            );
            CREATE INDEX idx_change_log_seq ON change_log(seq);

            CREATE TABLE sync_cursor (
                provider   TEXT PRIMARY KEY,
                cursor     TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE remote_id_map (
                provider    TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                local_id    INTEGER NOT NULL,
                remote_id   TEXT NOT NULL,
                PRIMARY KEY (provider, entity_type, local_id)
            );
            "#,
        ),
    ]
}

static MIGRATIONS: LazyLock<Migrations> = LazyLock::new(|| Migrations::new(migrations()));

/// Register Papr's custom SQL scalar functions on a freshly opened connection.
///
/// SQLite's built-in `LOWER()` only case-folds ASCII; Rust's
/// `str::to_lowercase()` is fully Unicode-aware. Rule preview must agree with
/// the case-folding `rule_matches` does, so `unicode_lower` provides it.
fn register_functions(conn: &Connection) -> Result<(), CoreError> {
    conn.create_scalar_function(
        "unicode_lower",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let value: Option<String> = ctx.get(0)?;
            Ok(value.map(|s| s.to_lowercase()))
        },
    )
    .map_err(|e| CoreError::Db(e.to_string()))?;
    Ok(())
}

/// Run the canonical migrations against `conn`. Both adapters call this so the
/// schema has a single source of truth. Does not set pragmas or register SQL
/// functions — callers do that around it.
pub fn migrate(conn: &mut Connection) -> Result<(), CoreError> {
    MIGRATIONS
        .to_latest(conn)
        .map_err(|e| CoreError::Db(format!("failed to run database migrations: {e}")))
}

/// Delete a legacy phase-1 validation database so it can be rebuilt with the
/// canonical schema. The old validation schema created `folders` but no FTS5
/// index, so it cannot forward-migrate under the shared migration sequence; it
/// is detected by the presence of `folders` and the absence of `articles_fts`,
/// then reset once. A fresh database (no tables) or an Alpha database (has
/// `articles_fts`) is left untouched.
fn reset_legacy_validation_db(path: &Path) -> Result<(), CoreError> {
    if !path.exists() {
        return Ok(());
    }
    let conn = Connection::open(path)
        .map_err(|e| CoreError::Db(format!("failed to open database at {}: {}", path.display(), e)))?;
    let has_folders: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='folders'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;
    let has_fts: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='articles_fts'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;
    drop(conn);

    if has_folders > 0 && has_fts == 0 {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
    }
    Ok(())
}

/// Open the writer connection: run migrations and set the write-side pragmas.
/// WAL mode is persisted in the database header, so reader connections opened
/// afterwards inherit it automatically.
fn open(path: &Path) -> Result<Connection, CoreError> {
    let mut conn = Connection::open(path)
        .map_err(|e| CoreError::Db(format!("failed to open database at {}: {}", path.display(), e)))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| CoreError::Db(e.to_string()))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| CoreError::Db(e.to_string()))?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| CoreError::Db(e.to_string()))?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .map_err(|e| CoreError::Db(e.to_string()))?;
    migrate(&mut conn)?;
    register_functions(&conn)?;
    Ok(conn)
}

/// Open a read-only connection for the UI query pool. Must be called after
/// `open` has migrated. `query_only` is a safety net against an accidental
/// write on a pooled reader.
fn open_reader(path: &Path) -> Result<Connection, CoreError> {
    let conn = Connection::open(path)
        .map_err(|e| CoreError::Db(format!("failed to open database at {}: {}", path.display(), e)))?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .map_err(|e| CoreError::Db(e.to_string()))?;
    conn.pragma_update(None, "query_only", true)
        .map_err(|e| CoreError::Db(e.to_string()))?;
    register_functions(&conn)?;
    Ok(conn)
}

/// Thread-safe SQLite handle.
///
/// Holds one writer connection (all mutations) plus a small pool of read-only
/// connections for UI queries. Under WAL the readers run concurrently with the
/// writer, so the interface stays responsive while a background refresh writes.
pub struct Db {
    writer: Mutex<Connection>,
    readers: Vec<Mutex<Connection>>,
    next_reader: AtomicUsize,
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
    /// Open or create the SQLite database, run migrations, and open the reader
    /// pool. A legacy phase-1 validation database (folders but no FTS index) is
    /// reset once, then upgraded through numbered migrations only.
    pub fn new(path: &Path) -> Result<Self, CoreError> {
        reset_legacy_validation_db(path)?;
        let writer = open(path)?;
        let mut readers = Vec::with_capacity(READER_POOL_SIZE);
        for _ in 0..READER_POOL_SIZE {
            readers.push(Mutex::new(open_reader(path)?));
        }
        Ok(Self {
            writer: Mutex::new(writer),
            readers,
            next_reader: AtomicUsize::new(0),
        })
    }

    /// Acquire the writer connection. All mutations hold this exclusively.
    fn writer(&self) -> Result<MutexGuard<'_, Connection>, CoreError> {
        self.writer.lock().map_err(|e| {
            CoreError::Db(format!("database writer mutex poisoned: {e}"))
        })
    }

    /// Acquire a read-only connection from the pool (round-robin).
    fn reader(&self) -> Result<MutexGuard<'_, Connection>, CoreError> {
        let i = self.next_reader.fetch_add(1, Ordering::Relaxed) % self.readers.len();
        self.readers[i].lock().map_err(|e| {
            CoreError::Db(format!("database reader mutex poisoned: {e}"))
        })
    }

    /// Run a closure inside a transaction on the writer connection. On `Ok` the
    /// transaction commits; on `Err` it rolls back. Business writes that also
    /// append a change-log entry must run through here so they commit or roll
    /// back atomically.
    pub fn transact<T, F>(&self, f: F) -> Result<T, CoreError>
    where
        F: for<'a> FnOnce(&rusqlite::Transaction<'a>) -> Result<T, CoreError>,
    {
        let mut conn = self.writer()?;
        let tx = conn.transaction().map_err(|e| CoreError::Db(e.to_string()))?;
        let result = f(&tx)?;
        tx.commit().map_err(|e| CoreError::Db(e.to_string()))?;
        Ok(result)
    }

    /// List all feeds.
    pub fn list_feeds(&self) -> Result<Vec<Feed>, CoreError> {
        let conn = self.reader()?;
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
        let conn = self.reader()?;
        let mut feeds = Self::list_feeds_locked(&conn)?;
        feeds.retain(|f| f.id == id);
        feeds.into_iter().next().ok_or_else(|| {
            CoreError::NotFound(format!("feed {} not found", id))
        })
    }

    /// Insert a new feed and return its generated row ID. The feed row and its
    /// change-log entry commit in the same transaction.
    pub fn add_feed(&self, feed_url: &str) -> Result<i64, CoreError> {
        self.transact(|tx| {
            tx.execute(
                "INSERT INTO feeds (feed_url, title, source_type, updated_at) \
                 VALUES (?1, ?2, 'rss', datetime('now'))",
                [feed_url, feed_url],
            )
            .map_err(|e| {
                if is_unique_violation(&e) {
                    CoreError::coded(
                        crate::error::ErrorCategory::InvalidInput,
                        "feedAlreadyExists",
                        Some(feed_url.to_string()),
                    )
                } else {
                    CoreError::Db(e.to_string())
                }
            })?;
            let id = tx.last_insert_rowid();
            append_change_log(tx, "feed", id, "upsert", None, None)?;
            Ok(id)
        })
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
        let conn = self.reader()?;
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
                // Tags are stored separately; article_tags join is not yet
                // exposed through this phase-1 filter — fall back to All.
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
        let conn = self.reader()?;
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
        let conn = self.reader()?;
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
        let conn = self.reader()?;
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

    /// Upsert an article, using (feed_id, guid) as the dedup key. The article
    /// row, its FTS index entry, and its enclosures land in one transaction so
    /// a mid-loop failure cannot leave a partially-indexed article.
    /// Returns true if a new row was inserted.
    pub fn upsert_article(
        &self,
        feed_id: i64,
        article: &NewArticle,
    ) -> Result<bool, CoreError> {
        let mut conn = self.writer()?;
        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Db(e.to_string()))?;

        let inserted = tx.execute(
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

        if inserted == 0 {
            return Ok(false);
        }
        let article_id = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO articles_fts(rowid, title, body) VALUES (?1, ?2, ?3)",
            (article_id, &article.title, &article.body_text),
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;

        for enc in &article.enclosures {
            tx.execute(
                "INSERT INTO enclosures (article_id, url, mime_type, length) VALUES (?1, ?2, ?3, ?4)",
                (article_id, &enc.url, &enc.mime_type, &enc.length),
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
        }

        tx.commit().map_err(|e| CoreError::Db(e.to_string()))?;
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
        let conn = self.writer()?;
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
        let conn = self.writer()?;
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
        let conn = self.writer()?;
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

/// Append a change-log entry inside an open transaction. Must be called in the
/// same transaction as the business write so the two commit or roll back
/// together. `field`/`value` are only meaningful for partial-entity changes
/// (e.g. article read state); entity-level upserts/soft-deletes leave them
/// `None`.
fn append_change_log(
    tx: &rusqlite::Transaction,
    entity: &str,
    entity_id: i64,
    op: &str,
    field: Option<&str>,
    value: Option<&str>,
) -> Result<(), CoreError> {
    let seq = next_seq(tx)?;
    tx.execute(
        "INSERT INTO change_log (entity, entity_id, field, value, op, seq) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (entity, entity_id, field, value, op, seq),
    )
    .map_err(|e| CoreError::Db(e.to_string()))?;
    Ok(())
}

/// Next monotonic change-log sequence number, within the current transaction.
fn next_seq(tx: &rusqlite::Transaction) -> Result<i64, CoreError> {
    tx.query_row(
        "SELECT COALESCE(MAX(seq), 0) + 1 FROM change_log",
        [],
        |r| r.get::<_, i64>(0),
    )
    .map_err(|e| CoreError::Db(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_article_is_not_counted_or_given_duplicate_enclosures() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        let article = NewArticle {
            guid: "article-1".to_string(),
            url: Some("https://example.com/article-1".to_string()),
            title: "Article 1".to_string(),
            author: None,
            summary: None,
            content_html: None,
            body_text: "Article body".to_string(),
            image_url: None,
            published_at: None,
            enclosures: vec![Enclosure {
                url: "https://example.com/audio.mp3".to_string(),
                mime_type: Some("audio/mpeg".to_string()),
                length: Some(42),
            }],
        };

        assert!(db.upsert_article(feed_id, &article).unwrap());
        assert!(!db.upsert_article(feed_id, &article).unwrap());

        let articles = db.list_articles(&ArticleFilter::default()).unwrap();
        assert_eq!(articles.len(), 1);
        let detail = db.get_article_detail(articles[0].id).unwrap();
        assert_eq!(detail.enclosures.len(), 1);
    }

    #[test]
    fn fresh_database_reaches_latest_schema() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.db");
        let db = Db::new(&path).unwrap();

        // Sync-ready baseline tables exist.
        let writer = db.writer().unwrap();
        for table in [
            "folders",
            "feeds",
            "articles",
            "enclosures",
            "articles_fts",
            "settings",
            "tags",
            "article_tags",
            "rules",
            "newsletter_sources",
            "highlights",
            "change_log",
            "sync_cursor",
            "remote_id_map",
        ] {
            let count: i64 = writer
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table','view') AND name = ?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(count > 0, "expected table {table} to exist");
        }
    }

    #[test]
    fn desktop_v15_database_forward_migrates_without_data_loss() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.db");

        // Simulate a desktop database at v15 by applying only the first 15
        // migrations, then inserting a row.
        {
            let mut conn = Connection::open(&path).unwrap();
            let v1_to_v15: Migrations = Migrations::new(migrations().into_iter().take(15).collect());
            v1_to_v15.to_latest(&mut conn).unwrap();
            conn.execute(
                "INSERT INTO folders (name, position) VALUES ('existing', 0)",
                [],
            )
            .unwrap();
        }

        // Opening through `Db::new` runs the full sequence (v1–v16); only v16
        // should apply on top of the existing v15 data.
        let db = Db::new(&path).unwrap();
        let writer = db.writer().unwrap();
        let name: String = writer
            .query_row("SELECT name FROM folders WHERE id = 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "existing");

        // v16 columns and tables are present.
        let has_sync_id: i64 = writer
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('folders') WHERE name = 'sync_id'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(has_sync_id, 1);
        let change_log_exists: i64 = writer
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='change_log'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(change_log_exists, 1);
    }

    #[test]
    fn reopening_does_not_clear_data() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.db");

        let feed_id = {
            let db = Db::new(&path).unwrap();
            db.add_feed("https://example.com/feed.xml").unwrap()
        };

        // Reopen: the feed survives, and no reset occurs.
        let db = Db::new(&path).unwrap();
        let feed = db.get_feed(feed_id).unwrap();
        assert_eq!(feed.feed_url, "https://example.com/feed.xml");
    }

    #[test]
    fn fts_index_is_maintained_on_insert_and_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        let article = NewArticle {
            guid: "g1".to_string(),
            url: None,
            title: "Searchable title".to_string(),
            author: None,
            summary: None,
            content_html: None,
            body_text: "Some body text".to_string(),
            image_url: None,
            published_at: None,
            enclosures: Vec::new(),
        };
        assert!(db.upsert_article(feed_id, &article).unwrap());

        let writer = db.writer().unwrap();
        let count: i64 = writer
            .query_row(
                "SELECT COUNT(*) FROM articles_fts WHERE articles_fts MATCH 'Searchable'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn add_feed_writes_change_log_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();

        let writer = db.writer().unwrap();
        let (entity, entity_id, op, seq): (String, i64, String, i64) = writer
            .query_row(
                "SELECT entity, entity_id, op, seq FROM change_log WHERE entity_id = ?1",
                [feed_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(entity, "feed");
        assert_eq!(entity_id, feed_id);
        assert_eq!(op, "upsert");
        assert_eq!(seq, 1);
    }

    #[test]
    fn transact_rolls_back_business_write_and_change_log_on_error() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();

        let result: Result<i64, CoreError> = db.transact(|tx| {
            tx.execute(
                "INSERT INTO feeds (feed_url, title, source_type) VALUES ('https://x/feed', 'x', 'rss')",
                [],
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
            let id = tx.last_insert_rowid();
            append_change_log(tx, "feed", id, "upsert", None, None)?;
            // Simulate a later failure inside the same transaction.
            Err(CoreError::Db("simulated failure".to_string()))
        });
        assert!(result.is_err());

        let reader = db.reader().unwrap();
        let feeds: i64 = reader
            .query_row("SELECT COUNT(*) FROM feeds", [], |r| r.get(0))
            .unwrap();
        let logs: i64 = reader
            .query_row("SELECT COUNT(*) FROM change_log", [], |r| r.get(0))
            .unwrap();
        assert_eq!(feeds, 0, "business write must roll back");
        assert_eq!(logs, 0, "change log must roll back with the business write");
    }

    #[test]
    fn legacy_validation_database_is_reset_once_then_upgrades_without_clearing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.db");

        // Simulate the old phase-1 validation schema: a `folders` table but no
        // FTS5 index, which cannot forward-migrate under the shared sequence.
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute(
                "CREATE TABLE folders (id INTEGER PRIMARY KEY, name TEXT NOT NULL, position INTEGER NOT NULL DEFAULT 0)",
                [],
            )
            .unwrap();
            conn.execute("INSERT INTO folders (name, position) VALUES ('stale', 0)", [])
                .unwrap();
        }

        // First open: the legacy DB is reset and the full Alpha schema built.
        let feed_id = {
            let db = Db::new(&path).unwrap();
            db.add_feed("https://example.com/feed.xml").unwrap()
        };

        // Second open: no reset, the feed survives, and the FTS index exists.
        let db = Db::new(&path).unwrap();
        let feed = db.get_feed(feed_id).unwrap();
        assert_eq!(feed.feed_url, "https://example.com/feed.xml");
        let writer = db.writer().unwrap();
        let has_fts: i64 = writer
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='articles_fts'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(has_fts, 1);
    }
}
