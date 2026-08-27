//! SQLite data layer.
//!
//! Owns the canonical, append-only schema shared by the desktop adapter
//! (`src-tauri`) and the Flutter adapter (`papr-flutter-bridge`). The
//! migration sequence v1–v15 is ported verbatim from the desktop's
//! `src-tauri/src/db.rs`; v16 adds the sync-ready baseline and later versions
//! remain shared by every adapter.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex, MutexGuard};

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};

use crate::dto::{
    AiSummaryCache, ArticleCounts, ArticleDetail, ArticleFilter, ArticleFilterKind, ArticleSummary,
    Enclosure, Feed, Folder, Highlight, HighlightInput, NewArticle, ResolvedHighlight, Rule,
    RuleInput, RulePreview, SourceType, SummaryTemplate, Tag, TagSummary,
};
use crate::error::{CoreError, ErrorCategory};

/// Number of read-only connections in the UI query pool.
const READER_POOL_SIZE: usize = 3;

/// The "never auto-refresh" sentinel (minutes ≈ one year). A per-feed interval
/// can carry it to opt one feed out of automatic refresh.
pub const REFRESH_OFF_MINUTES: i64 = 525_600;

const DEFAULT_ARTICLE_LIMIT: i64 = 50;
const MAX_ARTICLE_LIMIT: i64 = 200;

struct ArticleQueryParts {
    join_fts: bool,
    clauses: Vec<String>,
    params: Vec<rusqlite::types::Value>,
}

fn article_query_parts(filter: &ArticleFilter, force_unread: bool) -> ArticleQueryParts {
    let mut clauses = Vec::new();
    let mut params = Vec::new();

    match &filter.kind {
        ArticleFilterKind::All => {}
        ArticleFilterKind::Unread => clauses.push("a.is_read = 0".to_string()),
        ArticleFilterKind::Starred => clauses.push("a.is_starred = 1".to_string()),
        ArticleFilterKind::ReadLater => clauses.push("a.read_later = 1".to_string()),
        ArticleFilterKind::Feed(id) => {
            clauses.push("a.feed_id = ?".to_string());
            params.push((*id).into());
        }
        ArticleFilterKind::Folder(id) => {
            clauses.push("f.folder_id = ?".to_string());
            params.push((*id).into());
        }
        ArticleFilterKind::Tag(id) => {
            clauses
                .push("a.id IN (SELECT article_id FROM article_tags WHERE tag_id = ?)".to_string());
            params.push((*id).into());
        }
    }

    if (force_unread || filter.unread_only) && !matches!(filter.kind, ArticleFilterKind::Unread) {
        clauses.push("a.is_read = 0".to_string());
    }

    let mut join_fts = false;
    if let Some(search) = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if let Some(query) = fts_query(search) {
            join_fts = true;
            clauses.push("articles_fts MATCH ?".to_string());
            params.push(query.into());
        } else {
            // A punctuation-only query is a valid empty result, not an FTS
            // syntax error and not an accidental unfiltered list.
            clauses.push("0".to_string());
        }
    }

    ArticleQueryParts {
        join_fts,
        clauses,
        params,
    }
}

fn fts_query(input: &str) -> Option<String> {
    let terms: Vec<String> = input
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(|term| format!("\"{term}\"*"))
        .collect();
    (!terms.is_empty()).then(|| terms.join(" "))
}

#[derive(Clone, Copy)]
enum ArticleStateField {
    Read,
    Starred,
    ReadLater,
}

impl ArticleStateField {
    fn column(self) -> &'static str {
        match self {
            Self::Read => "is_read",
            Self::Starred => "is_starred",
            Self::ReadLater => "read_later",
        }
    }

    fn change_field(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Starred => "starred",
            Self::ReadLater => "read_later",
        }
    }
}

const TAG_COLORS: &[&str] = &[
    "clay", "amber", "pine", "teal", "indigo", "violet", "rose", "slate",
];
const HIGHLIGHT_COLORS: &[&str] = &["yellow", "green", "blue", "pink", "purple"];

fn ensure_tag_color(color: &str) -> Result<(), CoreError> {
    if TAG_COLORS.contains(&color) {
        Ok(())
    } else {
        Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            "invalidTagColor",
            None,
        ))
    }
}

fn ensure_highlight_color(color: &str) -> Result<(), CoreError> {
    if HIGHLIGHT_COLORS.contains(&color) {
        Ok(())
    } else {
        Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            "invalidHighlightColor",
            None,
        ))
    }
}

fn validate_rule(input: &RuleInput) -> Result<(), CoreError> {
    if input.name.trim().is_empty() {
        return Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            "emptyRuleName",
            None,
        ));
    }
    if input.query.trim().is_empty() {
        return Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            "emptyRuleQuery",
            None,
        ));
    }
    if !matches!(input.field.as_str(), "title" | "author" | "content" | "any") {
        return Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            "invalidRuleField",
            None,
        ));
    }
    if !matches!(input.action.as_str(), "skip" | "read" | "star") {
        return Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            "invalidRuleAction",
            None,
        ));
    }
    Ok(())
}

fn row_to_rule(row: &rusqlite::Row<'_>) -> rusqlite::Result<Rule> {
    Ok(Rule {
        id: row.get(0)?,
        name: row.get(1)?,
        enabled: row.get::<_, i64>(2)? != 0,
        feed_id: row.get(3)?,
        field: row.get(4)?,
        query: row.get(5)?,
        action: row.get(6)?,
        position: row.get(7)?,
    })
}

fn row_to_highlight(row: &rusqlite::Row<'_>) -> rusqlite::Result<Highlight> {
    Ok(Highlight {
        id: row.get(0)?,
        article_id: row.get(1)?,
        quote: row.get(2)?,
        prefix: row.get(3)?,
        suffix: row.get(4)?,
        text_offset: row.get(5)?,
        color: row.get(6)?,
        note: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn rule_matches(rule: &Rule, feed_id: i64, article: &NewArticle) -> bool {
    if rule.feed_id.is_some_and(|id| id != feed_id) {
        return false;
    }
    let author = article.author.as_deref().unwrap_or("").to_lowercase();
    let fields = match rule.field.as_str() {
        "author" => vec![author],
        "content" => vec![article.body_text.to_lowercase()],
        "any" => vec![
            article.title.to_lowercase(),
            author,
            article.body_text.to_lowercase(),
        ],
        _ => vec![article.title.to_lowercase()],
    };
    rule.query
        .split(',')
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .any(|term| fields.iter().any(|field| field.contains(&term)))
}

fn rule_match_where(
    field: &str,
    query: &str,
    feed_id: Option<i64>,
) -> Option<(String, Vec<rusqlite::types::Value>)> {
    let terms: Vec<_> = query
        .split(',')
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect();
    if terms.is_empty() {
        return None;
    }
    let columns: &[&str] = match field {
        "author" => &["author"],
        "content" => &["body_text"],
        "any" => &["title", "author", "body_text"],
        _ => &["title"],
    };
    let mut clauses = Vec::new();
    let mut values = Vec::new();
    for term in terms {
        let escaped = term
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        for column in columns {
            clauses.push(format!(
                "unicode_lower(COALESCE({column}, '')) LIKE ? ESCAPE '\\'"
            ));
            values.push(rusqlite::types::Value::Text(format!("%{escaped}%")));
        }
    }
    let mut sql = format!("({})", clauses.join(" OR "));
    if let Some(feed_id) = feed_id {
        sql.push_str(" AND feed_id = ?");
        values.push(feed_id.into());
    }
    Some((sql, values))
}

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
        M::up("ALTER TABLE feeds ADD COLUMN custom_title INTEGER NOT NULL DEFAULT 0;"),
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
        // v17 — identify the template and output language belonging to the
        // single most-recent successful AI summary cache.
        M::up(
            "ALTER TABLE articles ADD COLUMN ai_summary_template TEXT;
             ALTER TABLE articles ADD COLUMN ai_summary_lang TEXT;",
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
    let conn = Connection::open(path).map_err(|e| {
        CoreError::Db(format!(
            "failed to open database at {}: {}",
            path.display(),
            e
        ))
    })?;
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
    let mut conn = Connection::open(path).map_err(|e| {
        CoreError::Db(format!(
            "failed to open database at {}: {}",
            path.display(),
            e
        ))
    })?;
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
    let conn = Connection::open(path).map_err(|e| {
        CoreError::Db(format!(
            "failed to open database at {}: {}",
            path.display(),
            e
        ))
    })?;
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
        self.writer
            .lock()
            .map_err(|e| CoreError::Db(format!("database writer mutex poisoned: {e}")))
    }

    /// Acquire a read-only connection from the pool (round-robin).
    fn reader(&self) -> Result<MutexGuard<'_, Connection>, CoreError> {
        let i = self.next_reader.fetch_add(1, Ordering::Relaxed) % self.readers.len();
        self.readers[i]
            .lock()
            .map_err(|e| CoreError::Db(format!("database reader mutex poisoned: {e}")))
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
        let tx = conn
            .transaction()
            .map_err(|e| CoreError::Db(e.to_string()))?;
        let result = f(&tx)?;
        tx.commit().map_err(|e| CoreError::Db(e.to_string()))?;
        Ok(result)
    }

    /// List all folders, ordered by position then name.
    pub fn list_folders(&self) -> Result<Vec<Folder>, CoreError> {
        let conn = self.reader()?;
        let mut stmt = conn
            .prepare("SELECT id, name, position FROM folders ORDER BY position, name")
            .map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([], |r| {
                Ok(Folder {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    position: r.get(2)?,
                })
            })
            .map_err(|e| CoreError::Db(e.to_string()))?;
        rows.into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))
    }

    /// Create a folder, returning an existing same-name folder's id when present
    /// (case-insensitive). Rejects empty/whitespace-only names.
    pub fn create_folder(&self, name: &str) -> Result<i64, CoreError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "emptyFolderName",
                None,
            ));
        }
        self.transact(|tx| {
            if let Some(id) = tx
                .query_row(
                    "SELECT id FROM folders WHERE name = ?1 COLLATE NOCASE",
                    [name],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| CoreError::Db(e.to_string()))?
            {
                return Ok(id);
            }
            tx.execute(
                "INSERT INTO folders (name, position) \
                 VALUES (?1, (SELECT COALESCE(MAX(position), 0) + 1 FROM folders))",
                [name],
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
            let id = tx.last_insert_rowid();
            append_change_log(tx, "folder", id, "upsert", None, None)?;
            Ok(id)
        })
    }

    /// Rename a folder, rejecting a name that collides with a *different* folder.
    pub fn rename_folder(&self, id: i64, name: &str) -> Result<(), CoreError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "emptyFolderName",
                None,
            ));
        }
        self.transact(|tx| {
            let clash: Option<i64> = tx
                .query_row(
                    "SELECT id FROM folders WHERE name = ?1 COLLATE NOCASE AND id != ?2",
                    (name, id),
                    |r| r.get(0),
                )
                .optional()
                .map_err(|e| CoreError::Db(e.to_string()))?;
            if clash.is_some() {
                return Err(CoreError::coded(
                    ErrorCategory::InvalidInput,
                    "folderNameExists",
                    None,
                ));
            }
            tx.execute("UPDATE folders SET name = ?2 WHERE id = ?1", (id, name))
                .map_err(|e| CoreError::Db(e.to_string()))?;
            append_change_log(tx, "folder", id, "upsert", None, None)?;
            Ok(())
        })
    }

    /// Delete a folder. Its feeds move to uncategorised (`folder_id = NULL` via
    /// `ON DELETE SET NULL`), never deleted.
    pub fn delete_folder(&self, id: i64) -> Result<(), CoreError> {
        self.transact(|tx| {
            tx.execute("DELETE FROM folders WHERE id = ?1", [id])
                .map_err(|e| CoreError::Db(e.to_string()))?;
            append_change_log(tx, "folder", id, "delete", None, None)?;
            Ok(())
        })
    }

    /// Persist the complete folder order. The supplied IDs must contain every
    /// current folder exactly once.
    pub fn reorder_folders(&self, folder_ids: &[i64]) -> Result<(), CoreError> {
        self.transact(|tx| {
            let mut stmt = tx
                .prepare("SELECT id FROM folders ORDER BY position, name")
                .map_err(|e| CoreError::Db(e.to_string()))?;
            let current = stmt
                .query_map([], |row| row.get::<_, i64>(0))
                .map_err(|e| CoreError::Db(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| CoreError::Db(e.to_string()))?;

            let mut expected = current;
            let mut supplied = folder_ids.to_vec();
            expected.sort_unstable();
            supplied.sort_unstable();
            if supplied != expected {
                return Err(CoreError::coded(
                    ErrorCategory::InvalidInput,
                    "invalidFolderOrder",
                    None,
                ));
            }

            for (position, id) in folder_ids.iter().enumerate() {
                tx.execute(
                    "UPDATE folders SET position = ?2 WHERE id = ?1",
                    (*id, position as i64),
                )
                .map_err(|e| CoreError::Db(e.to_string()))?;
                append_change_log(tx, "folder", *id, "upsert", None, None)?;
            }
            Ok(())
        })
    }

    /// List all feeds.
    pub fn list_feeds(&self) -> Result<Vec<Feed>, CoreError> {
        let conn = self.reader()?;
        Self::list_feeds_locked(&conn)
    }

    fn list_feeds_locked(conn: &Connection) -> Result<Vec<Feed>, CoreError> {
        let mut stmt = conn.prepare(
            "SELECT id, feed_url, site_url, title, description, favicon_url, folder_id, source_type, last_fetched_at, fetch_error, custom_title, refresh_interval_min FROM feeds ORDER BY title COLLATE NOCASE"
        ).map_err(|e| CoreError::Db(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
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
                    custom_title: row.get(10)?,
                    refresh_interval_min: row.get(11)?,
                })
            })
            .map_err(|e| CoreError::Db(e.to_string()))?;

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
        feeds
            .into_iter()
            .next()
            .ok_or_else(|| CoreError::NotFound(format!("feed {} not found", id)))
    }

    /// Insert a feed with full metadata and return its generated row ID. The
    /// feed row and its change-log entry commit in the same transaction.
    pub fn insert_feed(
        &self,
        feed_url: &str,
        site_url: Option<&str>,
        title: &str,
        description: Option<&str>,
        source_type: SourceType,
        folder_id: Option<i64>,
    ) -> Result<i64, CoreError> {
        self.transact(|tx| {
            Self::insert_feed_tx(
                tx,
                feed_url,
                site_url,
                title,
                description,
                source_type,
                folder_id,
            )
        })
    }

    /// Insert a feed and its initial articles atomically. A failed article,
    /// enclosure, or FTS write rolls back both the feed and its change log.
    pub fn insert_feed_with_articles(
        &self,
        feed_url: &str,
        site_url: Option<&str>,
        title: &str,
        description: Option<&str>,
        source_type: SourceType,
        folder_id: Option<i64>,
        articles: &[NewArticle],
    ) -> Result<i64, CoreError> {
        self.transact(|tx| {
            let feed_id = Self::insert_feed_tx(
                tx,
                feed_url,
                site_url,
                title,
                description,
                source_type,
                folder_id,
            )?;
            for article in articles {
                Self::upsert_article_tx(tx, feed_id, article)?;
            }
            Ok(feed_id)
        })
    }

    fn insert_feed_tx(
        tx: &rusqlite::Transaction<'_>,
        feed_url: &str,
        site_url: Option<&str>,
        title: &str,
        description: Option<&str>,
        source_type: SourceType,
        folder_id: Option<i64>,
    ) -> Result<i64, CoreError> {
        tx.execute(
            "INSERT INTO feeds (feed_url, site_url, title, description, source_type, folder_id, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
            (
                feed_url,
                site_url,
                title,
                description,
                source_type.as_str(),
                folder_id,
            ),
        )
        .map_err(|e| {
            if is_unique_violation(&e) {
                CoreError::coded(
                    ErrorCategory::InvalidInput,
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
    }

    /// Insert a new feed and return its generated row ID (simple path).
    pub fn add_feed(&self, feed_url: &str) -> Result<i64, CoreError> {
        self.insert_feed(feed_url, None, feed_url, None, SourceType::Rss, None)
    }

    /// Find a feed's id by its URL, if it exists.
    pub fn find_feed_by_url(&self, url: &str) -> Result<Option<i64>, CoreError> {
        let conn = self.reader()?;
        conn.query_row("SELECT id FROM feeds WHERE feed_url = ?1", [url], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|e| CoreError::Db(e.to_string()))
    }

    /// Promote a feed's `source_type` once its real kind is known — but only
    /// when it is still the generic `'rss'` (never demote an existing type).
    pub fn refine_feed_source_type(
        &self,
        id: i64,
        source_type: SourceType,
    ) -> Result<(), CoreError> {
        if source_type == SourceType::Rss {
            return Ok(());
        }
        let conn = self.writer()?;
        conn.execute(
            "UPDATE feeds SET source_type = ?2 WHERE id = ?1 AND source_type = 'rss'",
            (id, source_type.as_str()),
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Delete a feed. Its articles and other dependent rows cascade-delete.
    pub fn delete_feed(&self, id: i64) -> Result<(), CoreError> {
        self.transact(|tx| {
            tx.execute("DELETE FROM feeds WHERE id = ?1", [id])
                .map_err(|e| CoreError::Db(e.to_string()))?;
            append_change_log(tx, "feed", id, "delete", None, None)?;
            Ok(())
        })
    }

    /// Set a feed's display title to a user-chosen value, marking it so a later
    /// refresh does not overwrite the rename from the feed document.
    pub fn rename_feed(&self, id: i64, title: &str) -> Result<(), CoreError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "emptyFeedTitle",
                None,
            ));
        }
        self.transact(|tx| {
            tx.execute(
                "UPDATE feeds SET title = ?2, custom_title = 1 WHERE id = ?1",
                (id, title),
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
            append_change_log(tx, "feed", id, "upsert", None, None)?;
            Ok(())
        })
    }

    /// Move a feed into (or out of) a folder. `None` files it under "uncategorised".
    pub fn move_feed(&self, id: i64, folder_id: Option<i64>) -> Result<(), CoreError> {
        self.transact(|tx| {
            tx.execute(
                "UPDATE feeds SET folder_id = ?2 WHERE id = ?1",
                (id, folder_id),
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
            append_change_log(tx, "feed", id, "upsert", None, None)?;
            Ok(())
        })
    }

    /// Set (or clear) a feed's per-feed refresh interval. `None` reverts to the
    /// global interval; `Some(REFRESH_OFF_MINUTES)` opts it out entirely.
    pub fn set_feed_refresh_interval(
        &self,
        id: i64,
        minutes: Option<i64>,
    ) -> Result<(), CoreError> {
        self.transact(|tx| {
            tx.execute(
                "UPDATE feeds SET refresh_interval_min = ?2 WHERE id = ?1",
                (id, minutes),
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
            append_change_log(tx, "feed", id, "upsert", None, None)?;
            Ok(())
        })
    }

    /// Feeds for OPML export as `(title, feed_url, folder)` tuples. Newsletter
    /// sources are excluded (their `feed_url` is a synthetic `imap://` string).
    pub fn feeds_for_export(&self) -> Result<Vec<(String, String, Option<String>)>, CoreError> {
        let conn = self.reader()?;
        let mut stmt = conn
            .prepare(
                "SELECT f.title, f.feed_url, fo.name \
                 FROM feeds f LEFT JOIN folders fo ON fo.id = f.folder_id \
                 WHERE f.source_type != 'newsletter' \
                 ORDER BY fo.name, f.title",
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| CoreError::Db(e.to_string()))?;
        rows.into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))
    }

    fn count_unread_articles_locked(conn: &Connection, feed_id: i64) -> Result<i64, CoreError> {
        conn.query_row(
            "SELECT COUNT(*) FROM articles WHERE feed_id = ?1 AND is_read = 0",
            [feed_id],
            |row| row.get::<usize, i64>(0),
        )
        .map_err(|e| CoreError::Db(e.to_string()))
    }

    /// List articles matching a filter.
    pub fn list_articles(&self, filter: &ArticleFilter) -> Result<Vec<ArticleSummary>, CoreError> {
        let conn = self.reader()?;
        let mut parts = article_query_parts(filter, false);
        let where_sql = if parts.clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", parts.clauses.join(" AND "))
        };
        let fts_join = if parts.join_fts {
            "JOIN articles_fts ON articles_fts.rowid = a.id"
        } else {
            ""
        };
        let order_sql = if parts.join_fts {
            "articles_fts.rank, a.id DESC"
        } else if filter.oldest_first {
            "datetime(COALESCE(a.published_at, a.fetched_at)) ASC, a.id ASC"
        } else {
            "datetime(COALESCE(a.published_at, a.fetched_at)) DESC, a.id DESC"
        };
        let limit = filter
            .limit
            .unwrap_or(DEFAULT_ARTICLE_LIMIT)
            .clamp(1, MAX_ARTICLE_LIMIT);
        let offset = filter.offset.unwrap_or(0).max(0);
        parts.params.push(limit.into());
        parts.params.push(offset.into());
        let sql = format!(
            "SELECT a.id, a.feed_id, f.title, f.source_type, a.title, a.author, \
                    substr(a.body_text, 1, 280), a.image_url, a.url, \
                    COALESCE(a.published_at, a.fetched_at), \
                    a.is_read, a.is_starred, a.read_later \
             FROM articles a \
             JOIN feeds f ON f.id = a.feed_id \
             {fts_join} \
             {where_sql} \
             ORDER BY {order_sql} LIMIT ? OFFSET ?"
        );

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CoreError::Db(e.to_string()))?;

        let rows = stmt
            .query_map(rusqlite::params_from_iter(parts.params), |row| {
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
            })
            .map_err(|e| CoreError::Db(e.to_string()))?;

        let mut articles = Vec::new();
        for row in rows {
            articles.push(row.map_err(|e| CoreError::Db(e.to_string()))?);
        }

        Ok(articles)
    }

    /// Count articles matching the same unbounded filter used by the list.
    pub fn count_articles(&self, filter: &ArticleFilter) -> Result<i64, CoreError> {
        let conn = self.reader()?;
        let parts = article_query_parts(filter, false);
        let fts_join = if parts.join_fts {
            "JOIN articles_fts ON articles_fts.rowid = a.id"
        } else {
            ""
        };
        let where_sql = if parts.clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", parts.clauses.join(" AND "))
        };
        conn.query_row(
            &format!(
                "SELECT COUNT(*) FROM articles a JOIN feeds f ON f.id = a.feed_id \
                 {fts_join} {where_sql}"
            ),
            rusqlite::params_from_iter(parts.params),
            |row| row.get(0),
        )
        .map_err(|e| CoreError::Db(e.to_string()))
    }

    pub fn article_counts(&self) -> Result<ArticleCounts, CoreError> {
        let conn = self.reader()?;
        conn.query_row(
            "SELECT COUNT(*), \
                    SUM(CASE WHEN is_read = 0 THEN 1 ELSE 0 END), \
                    SUM(CASE WHEN is_starred = 1 THEN 1 ELSE 0 END), \
                    SUM(CASE WHEN read_later = 1 THEN 1 ELSE 0 END) \
             FROM articles",
            [],
            |row| {
                Ok(ArticleCounts {
                    all: row.get(0)?,
                    unread: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    starred: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                    read_later: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                })
            },
        )
        .map_err(|e| CoreError::Db(e.to_string()))
    }

    pub fn list_tag_summaries(&self) -> Result<Vec<TagSummary>, CoreError> {
        let conn = self.reader()?;
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.name, t.color, COUNT(at.article_id), t.position \
                 FROM tags t LEFT JOIN article_tags at ON at.tag_id = t.id \
                 WHERE t.deleted_at IS NULL \
                 GROUP BY t.id ORDER BY t.position, t.name COLLATE NOCASE",
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(TagSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    article_count: row.get(3)?,
                    position: row.get(4)?,
                })
            })
            .map_err(|e| CoreError::Db(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))
    }

    pub fn create_tag(&self, name: &str) -> Result<i64, CoreError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "emptyTagName",
                None,
            ));
        }
        self.transact(|tx| {
            if let Some(id) = tx.query_row(
                "SELECT id FROM tags WHERE name = ?1 COLLATE NOCASE AND deleted_at IS NULL",
                [name], |row| row.get(0),
            ).optional().map_err(|e| CoreError::Db(e.to_string()))? {
                return Ok(id);
            }
            let position: i64 = tx.query_row(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM tags", [], |row| row.get(0),
            ).map_err(|e| CoreError::Db(e.to_string()))?;
            let color = TAG_COLORS[position as usize % TAG_COLORS.len()];
            tx.execute(
                "INSERT INTO tags(name, color, position, updated_at) VALUES (?1, ?2, ?3, datetime('now'))",
                (name, color, position),
            ).map_err(|e| CoreError::Db(e.to_string()))?;
            let id = tx.last_insert_rowid();
            append_change_log(tx, "tag", id, "upsert", None, None)?;
            Ok(id)
        })
    }

    pub fn rename_tag(&self, id: i64, name: &str) -> Result<(), CoreError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "emptyTagName",
                None,
            ));
        }
        self.transact(|tx| {
            let clash: Option<i64> = tx.query_row(
                "SELECT id FROM tags WHERE name = ?1 COLLATE NOCASE AND id != ?2 AND deleted_at IS NULL",
                (name, id), |row| row.get(0),
            ).optional().map_err(|e| CoreError::Db(e.to_string()))?;
            if clash.is_some() {
                return Err(CoreError::coded(ErrorCategory::InvalidInput, "tagNameExists", None));
            }
            ensure_changed(tx.execute(
                "UPDATE tags SET name = ?2, updated_at = datetime('now') WHERE id = ?1 AND deleted_at IS NULL",
                (id, name),
            ).map_err(|e| CoreError::Db(e.to_string()))?, "tagNotFound", id)?;
            append_change_log(tx, "tag", id, "upsert", None, None)
        })
    }

    pub fn set_tag_color(&self, id: i64, color: &str) -> Result<(), CoreError> {
        ensure_tag_color(color)?;
        self.transact(|tx| {
            ensure_changed(tx.execute(
                "UPDATE tags SET color = ?2, updated_at = datetime('now') WHERE id = ?1 AND deleted_at IS NULL",
                (id, color),
            ).map_err(|e| CoreError::Db(e.to_string()))?, "tagNotFound", id)?;
            append_change_log(tx, "tag", id, "upsert", None, None)
        })
    }

    pub fn reorder_tags(&self, ids: &[i64]) -> Result<(), CoreError> {
        self.transact(|tx| {
            let mut current = tx
                .prepare("SELECT id FROM tags WHERE deleted_at IS NULL")
                .map_err(|e| CoreError::Db(e.to_string()))?
                .query_map([], |row| row.get::<_, i64>(0))
                .map_err(|e| CoreError::Db(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| CoreError::Db(e.to_string()))?;
            let mut supplied = ids.to_vec();
            current.sort_unstable();
            supplied.sort_unstable();
            if current != supplied {
                return Err(CoreError::coded(
                    ErrorCategory::InvalidInput,
                    "invalidTagOrder",
                    None,
                ));
            }
            for (position, id) in ids.iter().enumerate() {
                tx.execute(
                    "UPDATE tags SET position = ?2, updated_at = datetime('now') WHERE id = ?1",
                    (*id, position as i64),
                )
                .map_err(|e| CoreError::Db(e.to_string()))?;
                append_change_log(tx, "tag", *id, "upsert", None, None)?;
            }
            Ok(())
        })
    }

    pub fn delete_tag(&self, id: i64) -> Result<(), CoreError> {
        self.transact(|tx| {
            ensure_changed(
                tx.execute("DELETE FROM tags WHERE id = ?1", [id])
                    .map_err(|e| CoreError::Db(e.to_string()))?,
                "tagNotFound",
                id,
            )?;
            append_change_log(tx, "tag", id, "delete", None, None)
        })
    }

    pub fn set_article_tag(
        &self,
        article_id: i64,
        tag_id: i64,
        attached: bool,
    ) -> Result<(), CoreError> {
        self.transact(|tx| {
            let article_exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM articles WHERE id = ?1)", [article_id], |row| row.get(0))
                .map_err(|e| CoreError::Db(e.to_string()))?;
            if !article_exists {
                return Err(CoreError::coded(ErrorCategory::NotFound, "articleNotFound", Some(article_id.to_string())));
            }
            let tag_exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM tags WHERE id = ?1 AND deleted_at IS NULL)", [tag_id], |row| row.get(0))
                .map_err(|e| CoreError::Db(e.to_string()))?;
            if !tag_exists {
                return Err(CoreError::coded(ErrorCategory::NotFound, "tagNotFound", Some(tag_id.to_string())));
            }
            let sql = if attached {
                "INSERT INTO article_tags(article_id, tag_id) VALUES (?1, ?2) ON CONFLICT DO NOTHING"
            } else {
                "DELETE FROM article_tags WHERE article_id = ?1 AND tag_id = ?2"
            };
            let changed = tx
                .execute(sql, (article_id, tag_id))
                .map_err(|e| CoreError::Db(e.to_string()))?;
            if changed == 0 {
                return Ok(());
            }
            append_change_log(
                tx,
                "article",
                article_id,
                "upsert",
                Some("tags"),
                Some(if attached { "attach" } else { "detach" }),
            )
        })
    }

    fn tags_for_article(conn: &Connection, article_id: i64) -> Result<Vec<Tag>, CoreError> {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color FROM tags t JOIN article_tags at ON at.tag_id = t.id \
             WHERE at.article_id = ?1 AND t.deleted_at IS NULL ORDER BY t.position, t.name COLLATE NOCASE",
        ).map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([article_id], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                })
            })
            .map_err(|e| CoreError::Db(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))
    }

    /// Fetch a single article by ID.
    pub fn get_article_detail(&self, article_id: i64) -> Result<ArticleDetail, CoreError> {
        let conn = self.reader()?;
        let mut stmt = conn.prepare(
            "SELECT articles.id, articles.feed_id, feeds.title, feeds.source_type, articles.title, articles.author, articles.url, articles.content_html, articles.extracted_html, articles.image_url, COALESCE(articles.published_at, articles.fetched_at), articles.is_read, articles.is_starred, articles.read_later, articles.ai_summary, articles.translated_html, articles.translated_lang \
             FROM articles \
             JOIN feeds ON feeds.id = articles.feed_id \
             WHERE articles.id = ?1"
        ).map_err(|e| CoreError::Db(e.to_string()))?;

        let mut row = stmt
            .query_row([article_id], |row| {
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
                    tags: Vec::new(),
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => CoreError::coded(
                    ErrorCategory::NotFound,
                    "articleNotFound",
                    Some(article_id.to_string()),
                ),
                _ => CoreError::Db(e.to_string()),
            })?;

        row.enclosures = Self::load_enclosures(&conn, article_id)?;
        row.tags = Self::tags_for_article(&conn, article_id)?;

        Ok(row)
    }

    /// Load the latest complete AI summary cache without exposing partial
    /// generation state. Metadata may be absent for summaries written before
    /// migration v17.
    pub fn get_ai_summary_cache(
        &self,
        article_id: i64,
    ) -> Result<Option<AiSummaryCache>, CoreError> {
        let conn = self.reader()?;
        let row = conn
            .query_row(
                "SELECT ai_summary, ai_summary_template, ai_summary_lang \
                 FROM articles WHERE id = ?1",
                [article_id],
                |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| CoreError::Db(e.to_string()))?;

        match row {
            None => Err(CoreError::coded(
                ErrorCategory::NotFound,
                "articleNotFound",
                Some(article_id.to_string()),
            )),
            Some((None, _, _)) => Ok(None),
            Some((Some(summary), template, language)) => Ok(Some(AiSummaryCache {
                summary,
                template,
                language,
            })),
        }
    }

    /// Return the title and authoritative plain text used for AI summaries.
    /// Extracted full text wins over the often-truncated feed body.
    pub fn article_text(&self, article_id: i64) -> Result<(String, String), CoreError> {
        let conn = self.reader()?;
        conn.query_row(
            "SELECT title, body_text, extracted_html FROM articles WHERE id = ?1",
            [article_id],
            |row| {
                let title: String = row.get(0)?;
                let body: String = row.get(1)?;
                let extracted: Option<String> = row.get(2)?;
                let text = extracted
                    .filter(|html| !html.trim().is_empty())
                    .map(|html| crate::ingestion::sanitize::html_to_text(&html))
                    .unwrap_or(body);
                Ok((title, text))
            },
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => CoreError::coded(
                ErrorCategory::NotFound,
                "articleNotFound",
                Some(article_id.to_string()),
            ),
            _ => CoreError::Db(error.to_string()),
        })
    }

    /// Atomically replace the single complete summary cache and its identifying
    /// metadata. Callers invoke this only after a stream completes successfully.
    pub fn set_ai_summary_cache(
        &self,
        article_id: i64,
        summary: &str,
        template: SummaryTemplate,
        language: &str,
    ) -> Result<(), CoreError> {
        let summary = summary.trim();
        if summary.is_empty() {
            return Err(CoreError::coded(ErrorCategory::Ai, "aiParse", None));
        }
        let language = crate::ai::response_language_code(language);
        self.transact(|tx| {
            let changed = tx
                .execute(
                    "UPDATE articles \
                     SET ai_summary = ?2, ai_summary_template = ?3, ai_summary_lang = ?4 \
                     WHERE id = ?1",
                    (article_id, summary, template.as_str(), language),
                )
                .map_err(|e| CoreError::Db(e.to_string()))?;
            if changed == 0 {
                return Err(CoreError::coded(
                    ErrorCategory::NotFound,
                    "articleNotFound",
                    Some(article_id.to_string()),
                ));
            }
            Ok(())
        })
    }

    pub fn set_article_read(&self, article_id: i64, value: bool) -> Result<(), CoreError> {
        self.set_article_state(article_id, ArticleStateField::Read, value)
    }

    pub fn set_article_starred(&self, article_id: i64, value: bool) -> Result<(), CoreError> {
        self.set_article_state(article_id, ArticleStateField::Starred, value)
    }

    pub fn set_article_read_later(&self, article_id: i64, value: bool) -> Result<(), CoreError> {
        self.set_article_state(article_id, ArticleStateField::ReadLater, value)
    }

    fn set_article_state(
        &self,
        article_id: i64,
        field: ArticleStateField,
        value: bool,
    ) -> Result<(), CoreError> {
        self.transact(|tx| {
            let current = tx
                .query_row(
                    &format!("SELECT {} FROM articles WHERE id = ?1", field.column()),
                    [article_id],
                    |row| row.get::<_, bool>(0),
                )
                .optional()
                .map_err(|e| CoreError::Db(e.to_string()))?
                .ok_or_else(|| {
                    CoreError::coded(
                        ErrorCategory::NotFound,
                        "articleNotFound",
                        Some(article_id.to_string()),
                    )
                })?;
            if current == value {
                return Ok(());
            }
            let changed = tx
                .execute(
                    &format!("UPDATE articles SET {} = ?2 WHERE id = ?1", field.column()),
                    (article_id, value),
                )
                .map_err(|e| CoreError::Db(e.to_string()))?;
            debug_assert_eq!(changed, 1);
            append_change_log(
                tx,
                "article",
                article_id,
                "upsert",
                Some(field.change_field()),
                Some(if value { "1" } else { "0" }),
            )
        })
    }

    /// Mark every unread article in the unbounded filter as read.
    pub fn mark_all_read(&self, filter: &ArticleFilter) -> Result<i64, CoreError> {
        self.transact(|tx| {
            let parts = article_query_parts(filter, true);
            let fts_join = if parts.join_fts {
                "JOIN articles_fts ON articles_fts.rowid = a.id"
            } else {
                ""
            };
            let where_sql = if parts.clauses.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", parts.clauses.join(" AND "))
            };
            let ids = {
                let mut stmt = tx
                    .prepare(&format!(
                        "SELECT a.id FROM articles a JOIN feeds f ON f.id = a.feed_id \
                         {fts_join} {where_sql}"
                    ))
                    .map_err(|e| CoreError::Db(e.to_string()))?;
                let rows = stmt
                    .query_map(rusqlite::params_from_iter(parts.params), |row| {
                        row.get::<_, i64>(0)
                    })
                    .map_err(|e| CoreError::Db(e.to_string()))?;
                rows.collect::<Result<Vec<_>, _>>()
                    .map_err(|e| CoreError::Db(e.to_string()))?
            };

            for id in &ids {
                tx.execute("UPDATE articles SET is_read = 1 WHERE id = ?1", [id])
                    .map_err(|e| CoreError::Db(e.to_string()))?;
                append_change_log(tx, "article", *id, "upsert", Some("read"), Some("1"))?;
            }
            Ok(ids.len() as i64)
        })
    }

    /// Persist sanitized extracted HTML, refresh FTS and invalidate any
    /// translation generated from the previous body in one transaction.
    pub fn set_extracted_html(
        &self,
        article_id: i64,
        html: &str,
        image_url: Option<&str>,
    ) -> Result<(), CoreError> {
        let plain_text = crate::ingestion::sanitize::html_to_text(html);
        self.transact(|tx| {
            let changed = tx
                .execute(
                    "UPDATE articles \
                     SET extracted_html = ?2, \
                         image_url = CASE \
                             WHEN ?3 IS NOT NULL AND (image_url IS NULL OR trim(image_url) = '') \
                             THEN ?3 ELSE image_url END, \
                         translated_html = NULL, translated_lang = NULL, \
                         ai_summary = NULL, ai_summary_template = NULL, \
                         ai_summary_lang = NULL \
                     WHERE id = ?1",
                    (article_id, html, image_url),
                )
                .map_err(|e| CoreError::Db(e.to_string()))?;
            if changed == 0 {
                return Err(CoreError::coded(
                    ErrorCategory::NotFound,
                    "articleNotFound",
                    Some(article_id.to_string()),
                ));
            }
            tx.execute(
                "UPDATE articles_fts SET body = ?2 WHERE rowid = ?1",
                (article_id, &plain_text),
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
            Ok(())
        })
    }

    pub fn list_rules(&self) -> Result<Vec<Rule>, CoreError> {
        let conn = self.reader()?;
        Self::list_rules_locked(&conn, false)
    }

    fn list_rules_locked(conn: &Connection, active_only: bool) -> Result<Vec<Rule>, CoreError> {
        let where_sql = if active_only {
            "WHERE enabled = 1 AND deleted_at IS NULL"
        } else {
            "WHERE deleted_at IS NULL"
        };
        let mut stmt = conn.prepare(&format!(
            "SELECT id, name, enabled, feed_id, field, query, action, position FROM rules {where_sql} ORDER BY position, id"
        )).map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([], row_to_rule)
            .map_err(|e| CoreError::Db(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))
    }

    pub fn create_rule(&self, input: &RuleInput) -> Result<i64, CoreError> {
        validate_rule(input)?;
        self.transact(|tx| {
            let position: i64 = tx.query_row(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM rules WHERE deleted_at IS NULL", [], |row| row.get(0),
            ).map_err(|e| CoreError::Db(e.to_string()))?;
            tx.execute(
                "INSERT INTO rules(name, enabled, feed_id, field, query, action, position, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
                (input.name.trim(), input.enabled, input.feed_id, &input.field, input.query.trim(), &input.action, position),
            ).map_err(|e| CoreError::Db(e.to_string()))?;
            let id = tx.last_insert_rowid();
            append_change_log(tx, "rule", id, "upsert", None, None)?;
            Ok(id)
        })
    }

    pub fn update_rule(&self, id: i64, input: &RuleInput) -> Result<(), CoreError> {
        validate_rule(input)?;
        self.transact(|tx| {
            ensure_changed(tx.execute(
                "UPDATE rules SET name = ?2, enabled = ?3, feed_id = ?4, field = ?5, query = ?6, action = ?7, \
                 updated_at = datetime('now') WHERE id = ?1 AND deleted_at IS NULL",
                (id, input.name.trim(), input.enabled, input.feed_id, &input.field, input.query.trim(), &input.action),
            ).map_err(|e| CoreError::Db(e.to_string()))?, "ruleNotFound", id)?;
            append_change_log(tx, "rule", id, "upsert", None, None)
        })
    }

    pub fn delete_rule(&self, id: i64) -> Result<(), CoreError> {
        self.transact(|tx| {
            ensure_changed(
                tx.execute("DELETE FROM rules WHERE id = ?1", [id])
                    .map_err(|e| CoreError::Db(e.to_string()))?,
                "ruleNotFound",
                id,
            )?;
            append_change_log(tx, "rule", id, "delete", None, None)
        })
    }

    pub fn preview_rule(&self, input: &RuleInput) -> Result<RulePreview, CoreError> {
        validate_rule(input)?;
        let Some((where_sql, values)) =
            rule_match_where(&input.field, input.query.trim(), input.feed_id)
        else {
            return Ok(RulePreview {
                count: 0,
                samples: Vec::new(),
            });
        };
        let conn = self.reader()?;
        let count = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM articles WHERE {where_sql}"),
                rusqlite::params_from_iter(values.iter().cloned()),
                |row| row.get(0),
            )
            .map_err(|e| CoreError::Db(e.to_string()))?;
        let mut stmt = conn.prepare(&format!(
            "SELECT title FROM articles WHERE {where_sql} ORDER BY datetime(COALESCE(published_at, fetched_at)) DESC, id DESC LIMIT 5"
        )).map_err(|e| CoreError::Db(e.to_string()))?;
        let samples = stmt
            .query_map(rusqlite::params_from_iter(values), |row| row.get(0))
            .map_err(|e| CoreError::Db(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))?;
        Ok(RulePreview { count, samples })
    }

    pub fn apply_rule_to_existing(&self, input: &RuleInput) -> Result<i64, CoreError> {
        validate_rule(input)?;
        self.transact(|tx| {
            let Some((where_sql, values)) = rule_match_where(&input.field, input.query.trim(), input.feed_id) else {
                return Ok(0);
            };
            let protect = "is_starred = 0 AND read_later = 0 AND NOT EXISTS(SELECT 1 FROM highlights h WHERE h.article_id = articles.id)";
            let sql = match input.action.as_str() {
                "skip" => format!("SELECT id FROM articles WHERE ({where_sql}) AND {protect}"),
                "read" => format!("SELECT id FROM articles WHERE ({where_sql}) AND is_read = 0"),
                "star" => format!("SELECT id FROM articles WHERE ({where_sql}) AND is_starred = 0"),
                _ => unreachable!(),
            };
            let ids = tx.prepare(&sql).map_err(|e| CoreError::Db(e.to_string()))?
                .query_map(rusqlite::params_from_iter(values), |row| row.get::<_, i64>(0))
                .map_err(|e| CoreError::Db(e.to_string()))?
                .collect::<Result<Vec<_>, _>>().map_err(|e| CoreError::Db(e.to_string()))?;
            for id in &ids {
                match input.action.as_str() {
                    "skip" => { tx.execute("DELETE FROM articles WHERE id = ?1", [id]).map_err(|e| CoreError::Db(e.to_string()))?; append_change_log(tx, "article", *id, "delete", None, None)?; }
                    "read" => { tx.execute("UPDATE articles SET is_read = 1 WHERE id = ?1", [id]).map_err(|e| CoreError::Db(e.to_string()))?; append_change_log(tx, "article", *id, "upsert", Some("read"), Some("1"))?; }
                    "star" => { tx.execute("UPDATE articles SET is_starred = 1 WHERE id = ?1", [id]).map_err(|e| CoreError::Db(e.to_string()))?; append_change_log(tx, "article", *id, "upsert", Some("starred"), Some("1"))?; }
                    _ => unreachable!(),
                }
            }
            Ok(ids.len() as i64)
        })
    }

    pub fn create_highlight(&self, input: &HighlightInput) -> Result<i64, CoreError> {
        let quote = input.quote.trim();
        if quote.is_empty() {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "emptyHighlight",
                None,
            ));
        }
        if input.text_offset < 0 {
            return Err(CoreError::coded(
                ErrorCategory::InvalidInput,
                "invalidHighlightOffset",
                None,
            ));
        }
        ensure_highlight_color(&input.color)?;
        self.transact(|tx| {
            let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM articles WHERE id = ?1)", [input.article_id], |row| row.get(0))
                .map_err(|e| CoreError::Db(e.to_string()))?;
            if !exists {
                return Err(CoreError::coded(ErrorCategory::NotFound, "articleNotFound", Some(input.article_id.to_string())));
            }
            tx.execute(
                "INSERT INTO highlights(article_id, quote, prefix, suffix, text_offset, color, note, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
                (input.article_id, quote, &input.prefix, &input.suffix, input.text_offset, &input.color, &input.note),
            ).map_err(|e| CoreError::Db(e.to_string()))?;
            let id = tx.last_insert_rowid();
            append_change_log(tx, "highlight", id, "upsert", None, None)?;
            Ok(id)
        })
    }

    pub fn list_highlights(&self, article_id: i64) -> Result<Vec<Highlight>, CoreError> {
        let conn = self.reader()?;
        let mut stmt = conn.prepare(
            "SELECT id, article_id, quote, prefix, suffix, text_offset, color, note, created_at FROM highlights \
             WHERE article_id = ?1 AND deleted_at IS NULL ORDER BY text_offset, id",
        ).map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([article_id], row_to_highlight)
            .map_err(|e| CoreError::Db(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))
    }

    pub fn list_all_highlights(&self) -> Result<Vec<Highlight>, CoreError> {
        let conn = self.reader()?;
        let mut stmt = conn.prepare(
            "SELECT id, article_id, quote, prefix, suffix, text_offset, color, note, created_at FROM highlights \
             WHERE deleted_at IS NULL ORDER BY created_at DESC, id DESC",
        ).map_err(|e| CoreError::Db(e.to_string()))?;
        let rows = stmt
            .query_map([], row_to_highlight)
            .map_err(|e| CoreError::Db(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CoreError::Db(e.to_string()))
    }

    pub fn update_highlight_note(&self, id: i64, note: &str) -> Result<(), CoreError> {
        self.transact(|tx| {
            ensure_changed(tx.execute(
                "UPDATE highlights SET note = ?2, updated_at = datetime('now') WHERE id = ?1 AND deleted_at IS NULL",
                (id, note),
            ).map_err(|e| CoreError::Db(e.to_string()))?, "highlightNotFound", id)?;
            append_change_log(tx, "highlight", id, "upsert", Some("note"), Some(note))
        })
    }

    pub fn set_highlight_color(&self, id: i64, color: &str) -> Result<(), CoreError> {
        ensure_highlight_color(color)?;
        self.transact(|tx| {
            ensure_changed(tx.execute(
                "UPDATE highlights SET color = ?2, updated_at = datetime('now') WHERE id = ?1 AND deleted_at IS NULL",
                (id, color),
            ).map_err(|e| CoreError::Db(e.to_string()))?, "highlightNotFound", id)?;
            append_change_log(tx, "highlight", id, "upsert", Some("color"), Some(color))
        })
    }

    pub fn delete_highlight(&self, id: i64) -> Result<(), CoreError> {
        self.transact(|tx| {
            ensure_changed(
                tx.execute("DELETE FROM highlights WHERE id = ?1", [id])
                    .map_err(|e| CoreError::Db(e.to_string()))?,
                "highlightNotFound",
                id,
            )?;
            append_change_log(tx, "highlight", id, "delete", None, None)
        })
    }

    pub fn resolve_highlights(
        &self,
        article_id: i64,
        text: &str,
    ) -> Result<Vec<ResolvedHighlight>, CoreError> {
        self.list_highlights(article_id).map(|highlights| {
            highlights
                .into_iter()
                .map(|highlight| {
                    let range = resolve_highlight_anchor(text, &highlight);
                    ResolvedHighlight {
                        highlight,
                        start: range.map(|r| r.0),
                        end: range.map(|r| r.1),
                    }
                })
                .collect()
        })
    }

    /// Read a single settings value.
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, CoreError> {
        let conn = self.reader()?;
        conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get::<usize, String>(0)
        })
        .optional()
        .map_err(|e| CoreError::Db(e.to_string()))
    }

    /// Persist one validated setting value.
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), CoreError> {
        let conn = self.writer()?;
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            (key, value),
        )
        .map_err(|e| CoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Persist a coherent settings group atomically.
    pub fn set_settings(&self, values: &[(&str, String)]) -> Result<(), CoreError> {
        self.transact(|tx| {
            for (key, value) in values {
                tx.execute(
                    "INSERT INTO settings(key, value) VALUES (?1, ?2) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    (key, value),
                )
                .map_err(|e| CoreError::Db(e.to_string()))?;
            }
            Ok(())
        })
    }

    /// Returns all feeds that should be refreshed.
    pub fn feeds_to_refresh(&self) -> Result<Vec<FeedRefreshInfo>, CoreError> {
        let conn = self.reader()?;
        let mut stmt = conn
            .prepare("SELECT id, feed_url, etag, last_modified FROM feeds ORDER BY title")
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
    pub fn upsert_article(&self, feed_id: i64, article: &NewArticle) -> Result<bool, CoreError> {
        self.transact(|tx| Self::upsert_article_tx(tx, feed_id, article))
    }

    fn upsert_article_tx(
        tx: &rusqlite::Transaction<'_>,
        feed_id: i64,
        article: &NewArticle,
    ) -> Result<bool, CoreError> {
        let rules = Self::list_rules_locked(&*tx, true)?;
        let (mut is_read, mut is_starred) = (false, false);
        for rule in &rules {
            if !rule_matches(rule, feed_id, article) {
                continue;
            }
            match rule.action.as_str() {
                "skip" => return Ok(false),
                "read" => is_read = true,
                "star" => is_starred = true,
                _ => unreachable!(),
            }
        }
        let inserted = tx.execute(
            "INSERT INTO articles \
             (feed_id, guid, url, title, author, summary, content_html, body_text, image_url, published_at, is_read, is_starred) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) \
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
                is_read,
                is_starred,
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

        if is_read {
            append_change_log(tx, "article", article_id, "upsert", Some("read"), Some("1"))?;
        }
        if is_starred {
            append_change_log(
                tx,
                "article",
                article_id,
                "upsert",
                Some("starred"),
                Some("1"),
            )?;
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
    fn load_enclosures(conn: &Connection, article_id: i64) -> Result<Vec<Enclosure>, CoreError> {
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

fn ensure_changed(changed: usize, code: &'static str, id: i64) -> Result<(), CoreError> {
    if changed == 0 {
        Err(CoreError::coded(
            ErrorCategory::NotFound,
            code,
            Some(id.to_string()),
        ))
    } else {
        Ok(())
    }
}

/// Locate a persisted highlight in current reader text. Offsets are UTF-16
/// code units so the result matches Flutter and the desktop webview.
pub fn resolve_highlight_anchor(text: &str, highlight: &Highlight) -> Option<(i64, i64)> {
    if highlight.quote.is_empty() {
        return None;
    }
    if let Some(start) = byte_index_at_utf16_offset(text, highlight.text_offset) {
        let end = start.checked_add(highlight.quote.len())?;
        if text.get(start..end) == Some(highlight.quote.as_str()) {
            return Some((
                highlight.text_offset,
                highlight.text_offset + highlight.quote.encode_utf16().count() as i64,
            ));
        }
    }

    let candidates = quote_occurrences(text, &highlight.quote);
    if candidates.is_empty() {
        return None;
    }
    let start = candidates.into_iter().max_by_key(|start| {
        let end = *start + highlight.quote.len();
        let score = common_suffix_len(&text[..*start], &highlight.prefix)
            + common_prefix_len(&text[end..], &highlight.suffix);
        let offset = text[..*start].encode_utf16().count() as i64;
        (
            score,
            std::cmp::Reverse((offset - highlight.text_offset).abs()),
        )
    })?;
    let start_utf16 = text[..start].encode_utf16().count() as i64;
    let end_utf16 = start_utf16 + highlight.quote.encode_utf16().count() as i64;
    Some((start_utf16, end_utf16))
}

fn byte_index_at_utf16_offset(text: &str, target: i64) -> Option<usize> {
    if target < 0 {
        return None;
    }
    let target = target as usize;
    let mut offset = 0;
    for (index, ch) in text.char_indices() {
        if offset == target {
            return Some(index);
        }
        offset += ch.len_utf16();
    }
    (offset == target).then_some(text.len())
}

fn quote_occurrences(text: &str, quote: &str) -> Vec<usize> {
    let mut results = Vec::new();
    let mut from = 0;
    while let Some(relative) = text[from..].find(quote) {
        let start = from + relative;
        results.push(start);
        from = start
            + text[start..]
                .chars()
                .next()
                .expect("quote is non-empty")
                .len_utf8();
    }
    results
}

fn common_suffix_len(a: &str, b: &str) -> usize {
    a.chars()
        .rev()
        .zip(b.chars().rev())
        .take_while(|(left, right)| left == right)
        .count()
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(left, right)| left == right)
        .count()
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

    fn test_article(guid: &str, title: &str) -> NewArticle {
        NewArticle {
            guid: guid.to_string(),
            url: Some(format!("https://example.com/{guid}")),
            title: title.to_string(),
            author: Some("CAFÉ Author".to_string()),
            summary: None,
            content_html: None,
            body_text: "A plain article body".to_string(),
            image_url: None,
            published_at: None,
            enclosures: Vec::new(),
        }
    }

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
            let v1_to_v15: Migrations =
                Migrations::new(migrations().into_iter().take(15).collect());
            v1_to_v15.to_latest(&mut conn).unwrap();
            conn.execute(
                "INSERT INTO folders (name, position) VALUES ('existing', 0)",
                [],
            )
            .unwrap();
        }

        // Opening through `Db::new` runs the full sequence; v16+ should apply
        // on top of the existing v15 data without rebuilding old tables.
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
        for column in ["ai_summary_template", "ai_summary_lang"] {
            let exists: i64 = writer
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info('articles') WHERE name = ?1",
                    [column],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1, "expected articles.{column} to exist");
        }
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
    fn initial_feed_and_articles_roll_back_together() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        {
            let writer = db.writer().unwrap();
            writer
                .execute_batch(
                    "CREATE TRIGGER fail_initial_article \
                     BEFORE INSERT ON articles \
                     BEGIN SELECT RAISE(ABORT, 'forced article failure'); END;",
                )
                .unwrap();
        }
        let article = NewArticle {
            guid: "g1".to_string(),
            url: None,
            title: "Article".to_string(),
            author: None,
            summary: None,
            content_html: None,
            body_text: "Body".to_string(),
            image_url: None,
            published_at: None,
            enclosures: Vec::new(),
        };

        let result = db.insert_feed_with_articles(
            "https://example.com/feed.xml",
            None,
            "Feed",
            None,
            SourceType::Rss,
            None,
            &[article],
        );
        assert!(result.is_err());

        let reader = db.reader().unwrap();
        let feeds: i64 = reader
            .query_row("SELECT COUNT(*) FROM feeds", [], |row| row.get(0))
            .unwrap();
        let logs: i64 = reader
            .query_row("SELECT COUNT(*) FROM change_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!(feeds, 0);
        assert_eq!(logs, 0);
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
            conn.execute(
                "INSERT INTO folders (name, position) VALUES ('stale', 0)",
                [],
            )
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

    #[test]
    fn folder_crud_dedups_and_rejects_collisions() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();

        // Create is idempotent on name (case-insensitive).
        let id = db.create_folder("Tech").unwrap();
        assert_eq!(db.create_folder("tech").unwrap(), id);
        assert_eq!(db.create_folder("  tech  ").unwrap(), id);

        // Empty name rejected.
        assert!(db.create_folder("   ").is_err());

        // Rename collision with a different folder rejected.
        let other = db.create_folder("News").unwrap();
        assert!(db.rename_folder(other, "Tech").is_err());
        db.rename_folder(other, "World").unwrap();

        // List reflects the folders.
        let folders = db.list_folders().unwrap();
        assert_eq!(folders.len(), 2);
        assert!(folders.iter().any(|f| f.name == "Tech"));
        assert!(folders.iter().any(|f| f.name == "World"));

        db.reorder_folders(&[other, id]).unwrap();
        let reordered = db.list_folders().unwrap();
        assert_eq!(
            reordered.iter().map(|f| f.id).collect::<Vec<_>>(),
            [other, id]
        );
        assert!(db.reorder_folders(&[id]).is_err());
    }

    #[test]
    fn delete_folder_keeps_feeds_ungrouped_and_delete_feed_cascades() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();

        let folder_id = db.create_folder("Tech").unwrap();
        let feed_id = db
            .insert_feed(
                "https://example.com/feed.xml",
                None,
                "Example",
                None,
                SourceType::Rss,
                Some(folder_id),
            )
            .unwrap();

        // Delete the folder: the feed stays, now ungrouped.
        db.delete_folder(folder_id).unwrap();
        let feed = db.get_feed(feed_id).unwrap();
        assert_eq!(feed.folder_id, None);

        // Feed rename sets custom_title and reflects in list.
        db.rename_feed(feed_id, "Renamed").unwrap();
        assert!(db.rename_feed(feed_id, "   ").is_err());
        let feed = db.get_feed(feed_id).unwrap();
        assert_eq!(feed.title, "Renamed");

        // Move + refresh interval round-trip.
        let f2 = db.create_folder("Other").unwrap();
        db.move_feed(feed_id, Some(f2)).unwrap();
        assert_eq!(db.get_feed(feed_id).unwrap().folder_id, Some(f2));
        db.set_feed_refresh_interval(feed_id, Some(60)).unwrap();
        assert_eq!(db.get_feed(feed_id).unwrap().refresh_interval_min, Some(60));

        // Delete feed removes it.
        db.delete_feed(feed_id).unwrap();
        assert!(db.get_feed(feed_id).is_err());
    }

    #[test]
    fn article_filters_counts_tags_and_paging_share_one_contract() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db
            .insert_feed(
                "https://example.com/feed.xml",
                None,
                "Example",
                None,
                SourceType::Rss,
                None,
            )
            .unwrap();
        for (guid, title, body, date) in [
            (
                "rust",
                "Rust guide",
                "A complete Rust language guide",
                "2024-01-01T00:00:00Z",
            ),
            (
                "food",
                "Cooking notes",
                "A seasonal cooking notebook",
                "2025-01-01T00:00:00Z",
            ),
        ] {
            db.upsert_article(
                feed_id,
                &NewArticle {
                    guid: guid.to_string(),
                    url: Some(format!("https://example.com/{guid}")),
                    title: title.to_string(),
                    author: None,
                    summary: None,
                    content_html: None,
                    body_text: body.to_string(),
                    image_url: None,
                    published_at: Some(date.to_string()),
                    enclosures: Vec::new(),
                },
            )
            .unwrap();
        }

        let all = db.list_articles(&ArticleFilter::default()).unwrap();
        assert_eq!(
            all.iter().map(|a| a.title.as_str()).collect::<Vec<_>>(),
            ["Cooking notes", "Rust guide",]
        );
        let rust_id = all.iter().find(|a| a.title == "Rust guide").unwrap().id;
        let food_id = all.iter().find(|a| a.title == "Cooking notes").unwrap().id;

        let tag_id = {
            let writer = db.writer().unwrap();
            writer
                .execute(
                    "INSERT INTO tags(name, color, position) VALUES ('Tech', 'blue', 0)",
                    [],
                )
                .unwrap();
            let id = writer.last_insert_rowid();
            writer
                .execute(
                    "INSERT INTO article_tags(article_id, tag_id) VALUES (?1, ?2)",
                    (rust_id, id),
                )
                .unwrap();
            id
        };

        let searched = db
            .list_articles(&ArticleFilter {
                search: Some("rust language".to_string()),
                ..ArticleFilter::default()
            })
            .unwrap();
        assert_eq!(searched.len(), 1);
        assert_eq!(searched[0].id, rust_id);
        assert!(db
            .list_articles(&ArticleFilter {
                search: Some("---".to_string()),
                ..ArticleFilter::default()
            })
            .unwrap()
            .is_empty());

        let tagged = db
            .list_articles(&ArticleFilter {
                kind: ArticleFilterKind::Tag(tag_id),
                oldest_first: true,
                ..ArticleFilter::default()
            })
            .unwrap();
        assert_eq!(tagged.iter().map(|a| a.id).collect::<Vec<_>>(), [rust_id]);
        assert_eq!(db.list_tag_summaries().unwrap()[0].article_count, 1);

        let one = db
            .list_articles(&ArticleFilter {
                limit: Some(-5),
                ..ArticleFilter::default()
            })
            .unwrap();
        assert_eq!(one.len(), 1, "invalid limits are clamped to one row");
        assert_eq!(db.count_articles(&ArticleFilter::default()).unwrap(), 2);

        db.set_article_starred(rust_id, true).unwrap();
        db.set_article_read_later(food_id, true).unwrap();
        assert_eq!(
            db.article_counts().unwrap(),
            ArticleCounts {
                all: 2,
                unread: 2,
                starred: 1,
                read_later: 1,
            }
        );
    }

    #[test]
    fn article_state_and_extracted_body_update_transactional_projections() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        db.upsert_article(
            feed_id,
            &NewArticle {
                guid: "article".to_string(),
                url: Some("https://example.com/article".to_string()),
                title: "Article".to_string(),
                author: None,
                summary: None,
                content_html: Some("<p>Short feed body</p>".to_string()),
                body_text: "Short feed body".to_string(),
                image_url: None,
                published_at: None,
                enclosures: Vec::new(),
            },
        )
        .unwrap();
        let article_id = db.list_articles(&ArticleFilter::default()).unwrap()[0].id;

        db.set_article_starred(article_id, true).unwrap();
        db.set_article_starred(article_id, true).unwrap();
        let changed = db
            .mark_all_read(&ArticleFilter {
                kind: ArticleFilterKind::Starred,
                ..ArticleFilter::default()
            })
            .unwrap();
        assert_eq!(changed, 1);
        assert_eq!(
            db.mark_all_read(&ArticleFilter {
                kind: ArticleFilterKind::Starred,
                ..ArticleFilter::default()
            })
            .unwrap(),
            0,
            "bulk state updates are idempotent"
        );
        assert!(db.get_article_detail(article_id).unwrap().is_read);
        assert_eq!(
            db.set_article_read(999_999, true).unwrap_err().code(),
            "articleNotFound"
        );

        {
            let writer = db.writer().unwrap();
            writer
                .execute(
                    "UPDATE articles SET translated_html = '<p>old</p>', translated_lang = 'ja' WHERE id = ?1",
                    [article_id],
                )
                .unwrap();
        }
        db.set_extracted_html(
            article_id,
            "<p>Complete extracted searchable body</p>",
            Some("https://example.com/lead.jpg"),
        )
        .unwrap();
        let detail = db.get_article_detail(article_id).unwrap();
        assert_eq!(detail.translated_html, None);
        assert_eq!(detail.translated_lang, None);
        assert_eq!(
            detail.image_url.as_deref(),
            Some("https://example.com/lead.jpg")
        );
        assert_eq!(
            db.count_articles(&ArticleFilter {
                search: Some("searchable".to_string()),
                ..ArticleFilter::default()
            })
            .unwrap(),
            1
        );

        let reader = db.reader().unwrap();
        let fields: Vec<String> = reader
            .prepare("SELECT field FROM change_log WHERE entity = 'article' ORDER BY seq")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(fields, ["starred", "read"]);
    }

    #[test]
    fn ai_summary_cache_replaces_text_template_and_language_together() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        db.upsert_article(feed_id, &test_article("summary", "Summary article"))
            .unwrap();
        let article_id = db.list_articles(&ArticleFilter::default()).unwrap()[0].id;

        assert_eq!(db.get_ai_summary_cache(article_id).unwrap(), None);
        db.set_ai_summary_cache(
            article_id,
            "  cached answer  ",
            SummaryTemplate::News5w1h,
            "zh-CN",
        )
        .unwrap();
        assert_eq!(
            db.get_ai_summary_cache(article_id).unwrap(),
            Some(AiSummaryCache {
                summary: "cached answer".into(),
                template: Some("news5w1h".into()),
                language: Some("zh".into()),
            })
        );

        assert_eq!(
            db.set_ai_summary_cache(article_id, " ", SummaryTemplate::Minimal, "ja")
                .unwrap_err()
                .code(),
            "aiParse"
        );
        assert_eq!(
            db.get_ai_summary_cache(article_id)
                .unwrap()
                .unwrap()
                .template
                .as_deref(),
            Some("news5w1h"),
            "a rejected replacement keeps the previous complete cache"
        );
        assert_eq!(
            db.set_ai_summary_cache(999_999, "answer", SummaryTemplate::Classic, "en")
                .unwrap_err()
                .code(),
            "articleNotFound"
        );

        db.set_extracted_html(article_id, "<p>new authoritative body</p>", None)
            .unwrap();
        assert_eq!(
            db.get_ai_summary_cache(article_id).unwrap(),
            None,
            "changing the source body invalidates its derived summary"
        );
    }

    #[test]
    fn tags_are_normalized_ordered_and_do_not_delete_articles() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        db.upsert_article(feed_id, &test_article("tagged", "Tagged article"))
            .unwrap();
        let article_id = db.list_articles(&ArticleFilter::default()).unwrap()[0].id;

        let rust = db.create_tag(" Rust ").unwrap();
        assert_eq!(db.create_tag("rust").unwrap(), rust);
        let news = db.create_tag("News").unwrap();
        db.set_tag_color(rust, "indigo").unwrap();
        db.reorder_tags(&[news, rust]).unwrap();
        db.set_article_tag(article_id, rust, true).unwrap();

        let tags = db.list_tag_summaries().unwrap();
        assert_eq!(
            tags.iter().map(|tag| tag.id).collect::<Vec<_>>(),
            [news, rust]
        );
        assert_eq!(tags[1].color, "indigo");
        assert_eq!(tags[1].article_count, 1);
        assert_eq!(
            db.get_article_detail(article_id).unwrap().tags[0].name,
            "Rust"
        );

        assert_eq!(
            db.rename_tag(news, "RUST").unwrap_err().code(),
            "tagNameExists"
        );
        db.delete_tag(rust).unwrap();
        assert!(db.get_article_detail(article_id).is_ok());
        assert!(db.get_article_detail(article_id).unwrap().tags.is_empty());
    }

    #[test]
    fn rules_share_unicode_matching_and_protect_saved_articles_from_skip() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        for (guid, title) in [
            ("ordinary", "CAFÉ %_ ordinary"),
            ("starred", "café %_ starred"),
            ("later", "Café %_ later"),
            ("highlighted", "Café %_ highlighted"),
        ] {
            db.upsert_article(feed_id, &test_article(guid, title))
                .unwrap();
        }
        let articles = db.list_articles(&ArticleFilter::default()).unwrap();
        let find_id = |title: &str| {
            articles
                .iter()
                .find(|article| article.title == title)
                .unwrap()
                .id
        };
        db.set_article_starred(find_id("café %_ starred"), true)
            .unwrap();
        db.set_article_read_later(find_id("Café %_ later"), true)
            .unwrap();
        db.create_highlight(&HighlightInput {
            article_id: find_id("Café %_ highlighted"),
            quote: "highlighted".to_string(),
            prefix: "Café %_ ".to_string(),
            suffix: String::new(),
            text_offset: 9,
            color: "yellow".to_string(),
            note: String::new(),
        })
        .unwrap();

        let skip = RuleInput {
            name: "Discard literal café pattern".to_string(),
            enabled: true,
            feed_id: Some(feed_id),
            field: "title".to_string(),
            query: "café %_".to_string(),
            action: "skip".to_string(),
        };
        let preview = db.preview_rule(&skip).unwrap();
        assert_eq!(preview.count, 4);
        assert_eq!(db.apply_rule_to_existing(&skip).unwrap(), 1);
        assert_eq!(db.count_articles(&ArticleFilter::default()).unwrap(), 3);

        let star_existing = RuleInput {
            name: "Star saved article".to_string(),
            enabled: true,
            feed_id: Some(feed_id),
            field: "title".to_string(),
            query: "later".to_string(),
            action: "star".to_string(),
        };
        assert_eq!(db.preview_rule(&star_existing).unwrap().count, 1);
        assert_eq!(db.apply_rule_to_existing(&star_existing).unwrap(), 1);
        assert!(
            db.get_article_detail(find_id("Café %_ later"))
                .unwrap()
                .is_starred
        );

        let incoming = RuleInput {
            name: "Mark incoming".to_string(),
            enabled: true,
            feed_id: Some(feed_id),
            field: "title".to_string(),
            query: "incoming".to_string(),
            action: "star".to_string(),
        };
        db.create_rule(&incoming).unwrap();
        assert!(db
            .upsert_article(feed_id, &test_article("incoming", "INCOMING story"))
            .unwrap());
        assert!(
            db.list_articles(&ArticleFilter::default())
                .unwrap()
                .iter()
                .find(|article| article.title == "INCOMING story")
                .unwrap()
                .is_starred
        );
        let starred_logs: i64 = db
            .reader()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM change_log WHERE entity = 'article' AND field = 'starred'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            starred_logs >= 2,
            "incoming rule state is logged with the write"
        );
    }

    #[test]
    fn highlights_are_mutable_and_reanchor_by_context_or_utf16_offset() {
        let tmp = tempfile::tempdir().unwrap();
        let db = Db::new(&tmp.path().join("test.db")).unwrap();
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        db.upsert_article(feed_id, &test_article("highlight", "Highlight article"))
            .unwrap();
        let article_id = db.list_articles(&ArticleFilter::default()).unwrap()[0].id;
        let id = db
            .create_highlight(&HighlightInput {
                article_id,
                quote: "target".to_string(),
                prefix: "nearby ".to_string(),
                suffix: " tail".to_string(),
                text_offset: 0,
                color: "yellow".to_string(),
                note: String::new(),
            })
            .unwrap();
        db.update_highlight_note(id, "Keep this").unwrap();
        db.set_highlight_color(id, "blue").unwrap();
        let highlight = db.list_highlights(article_id).unwrap().pop().unwrap();
        assert_eq!(highlight.note, "Keep this");
        assert_eq!(highlight.color, "blue");

        let text = "begin target finish, nearby target tail";
        let start = text.find("nearby target").unwrap() + "nearby ".len();
        let expected = text[..start].encode_utf16().count() as i64;
        assert_eq!(
            resolve_highlight_anchor(text, &highlight),
            Some((expected, expected + 6))
        );

        let emoji = Highlight {
            text_offset: 3,
            prefix: String::new(),
            suffix: String::new(),
            ..highlight
        };
        assert_eq!(resolve_highlight_anchor("🙂 target", &emoji), Some((3, 9)));
        db.delete_highlight(id).unwrap();
        assert!(db.list_all_highlights().unwrap().is_empty());
    }
}
