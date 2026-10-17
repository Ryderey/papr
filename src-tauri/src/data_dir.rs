//! Data-directory resolution, override bookkeeping, and the restart-time
//! database relocation.
//!
//! Papr stores everything in one SQLite file under the platform data dir
//! (`%APPDATA%\com.thomas.papr` on Windows). Users who keep large collections
//! can relocate that directory: the `data-dir.txt` marker file in the DEFAULT
//! directory names the configured one, and a `migration-pending.json` marker
//! schedules the actual copy for the next start.
//!
//! A configured directory that does not answer yet is probed a few times
//! before falling back (mechanical drives and boot-time mounts come up
//! slowly), and the reason for every fallback is recorded in
//! `data-dir-fallback.txt` for later investigation.
//!
//! All marker files live in the *default* directory — it is always known and
//! always writable, so there is no bootstrap problem. The source database is
//! never touched destructively: a failed or interrupted migration leaves it
//! intact and simply doesn't activate the new location.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const OVERRIDE_FILE: &str = "data-dir.txt";
pub const PENDING_FILE: &str = "migration-pending.json";
const ERROR_FILE: &str = "data-dir-migration-error.txt";
const FALLBACK_FILE: &str = "data-dir-fallback.txt";
const DB_FILE: &str = "papr.db";
const TMP_DB_FILE: &str = "papr.db.migrating";
const PREVIOUS_DIR_SETTING: &str = "data_dir_previous";

/// Startup grace for a configured directory that is not answering (yet):
/// probe count and interval before falling back. One failed probe would
/// silently drop the session onto the default directory, which mechanical
/// drives at boot regularly cause. Worst case this adds
/// `(OVERRIDE_PROBE_ATTEMPTS - 1) * OVERRIDE_PROBE_INTERVAL` to startup.
const OVERRIDE_PROBE_ATTEMPTS: u32 = 3;
const OVERRIDE_PROBE_INTERVAL: Duration = Duration::from_millis(1500);

/// The effective database location for this start.
pub struct DataPaths {
    /// Directory holding `papr.db` — the configured one if active, else the
    /// platform default.
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
}

/// A start-time data situation the user should know about. Surfaced as an
/// OS notification after the database is open (the UI language is only
/// readable then).
pub enum Notice {
    /// A scheduled relocation ran but failed; the source directory stays
    /// active and the request will be retried on the next start.
    MigrationFailed { to: PathBuf, error: String },
    /// The configured directory stayed unusable through the startup probe
    /// grace period (unreachable, or its `papr.db` is missing). The session
    /// runs on the default directory; the override file is left untouched so
    /// recovery is automatic once the directory is reachable again. The
    /// reason is recorded in `data-dir-fallback.txt`.
    FellBackToDefault { configured: PathBuf },
}

/// Localised notification body for a start-time notice, in the user's UI
/// language (read from settings after the database is open).
pub fn notice_body(lang: &str, notice: &Notice) -> String {
    match notice {
        Notice::MigrationFailed { to, error } => match lang {
            "zh" => format!(
                "数据目录迁移失败（目标 {}）：{error}。仍使用原目录，下次启动将自动重试。",
                to.display()
            ),
            "ja" => format!(
                "データディレクトリの移行に失敗しました（移動先 {}）：{error}。現在のディレクトリを使い続け、次回起動時に再試行します。",
                to.display()
            ),
            _ => format!(
                "Data directory migration failed (target {}): {error}. Keeping the current directory; it will be retried on next start.",
                to.display()
            ),
        },
        Notice::FellBackToDefault { configured } => match lang {
            "zh" => format!(
                "配置的数据目录 {} 当前不可用，本次启动已回退到默认目录；目录恢复后重启即可回到原目录。",
                configured.display()
            ),
            "ja" => format!(
                "設定されたデータディレクトリ {} が現在利用できません。今回は既定のディレクトリを使用します。",
                configured.display()
            ),
            _ => format!(
                "The configured data directory {} is unavailable right now, so this session uses the default directory. Restart once it is reachable to switch back.",
                configured.display()
            ),
        },
    }
}

// ── Resolution ───────────────────────────────────────────────────────────

/// Resolve the effective data directory. Returns the active paths plus a
/// fallback notice when an override file exists but is unusable (relative
/// path, or its `papr.db` is missing — typically an unplugged drive or a
/// deleted folder).
pub fn resolve(default_dir: &Path) -> (DataPaths, Option<Notice>) {
    resolve_with_grace(
        default_dir,
        OVERRIDE_PROBE_ATTEMPTS,
        OVERRIDE_PROBE_INTERVAL,
    )
}

/// `resolve` with an explicit probe policy so tests don't sleep. The
/// configured directory is probed `attempts` times, `interval` apart — a
/// drive that is merely slow to come up must not end up in a fallback.
fn resolve_with_grace(
    default_dir: &Path,
    attempts: u32,
    interval: Duration,
) -> (DataPaths, Option<Notice>) {
    let attempts = attempts.max(1);
    let default_paths = DataPaths {
        data_dir: default_dir.to_path_buf(),
        db_path: default_dir.join(DB_FILE),
    };
    let Some(configured) = read_override(default_dir) else {
        return (default_paths, None);
    };
    if configured.is_absolute() {
        for probe in 1..=attempts {
            if configured.join(DB_FILE).is_file() {
                return (
                    DataPaths {
                        db_path: configured.join(DB_FILE),
                        data_dir: configured,
                    },
                    None,
                );
            }
            if probe < attempts {
                std::thread::sleep(interval);
            }
        }
    }
    let span_ms = interval.as_millis() * u128::from(attempts - 1);
    let reason = if !configured.is_absolute() {
        "override path is not absolute".to_string()
    } else if !configured.is_dir() {
        format!("directory unreachable after {attempts} probes spanning {span_ms} ms")
    } else {
        format!("{DB_FILE} missing after {attempts} probes spanning {span_ms} ms")
    };
    log::warn!(
        "data-dir override {configured:?} is unusable ({reason}); falling back to the default directory"
    );
    record_fallback(default_dir, &configured, &reason);
    (
        default_paths,
        Some(Notice::FellBackToDefault {
            configured: configured.clone(),
        }),
    )
}

fn read_override(default_dir: &Path) -> Option<PathBuf> {
    fs::read_to_string(default_dir.join(OVERRIDE_FILE))
        .ok()
        .and_then(|s| {
            let s = s.trim();
            (!s.is_empty()).then(|| PathBuf::from(s))
        })
}

/// Compare two directories for our purposes: canonicalize when both exist,
/// otherwise a case-insensitive (Windows) textual compare. `\\?\` and `\\?\UNC\`
/// prefixes appear only on canonicalized paths and could make the identical
/// location look different, so both sides are normalized.
fn norm(p: &Path) -> String {
    let s = p.to_string_lossy();
    let s = s.replace("\\\\?\\UNC\\", "\\\\").replace("\\\\?\\", "");
    #[cfg(windows)]
    {
        s.to_lowercase()
    }
    #[cfg(not(windows))]
    {
        s
    }
}

pub fn same_dir(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => norm(&a) == norm(&b),
        _ => norm(a) == norm(b),
    }
}

// ── Pending relocation ───────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct Pending {
    /// Directory the database should move to on next start.
    to: PathBuf,
}

/// Schedule a relocation for the next start. Written by the settings command
/// only after the target passed validation.
pub fn write_pending(default_dir: &Path, to: &Path) -> std::io::Result<()> {
    let json = serde_json::to_string(&Pending {
        to: to.to_path_buf(),
    })
    .expect("serializing the pending-relocation marker");
    fs::write(default_dir.join(PENDING_FILE), json)
}

pub(crate) fn read_pending_to(default_dir: &Path) -> Option<PathBuf> {
    let Ok(raw) = fs::read_to_string(default_dir.join(PENDING_FILE)) else {
        return None;
    };
    match serde_json::from_str::<Pending>(&raw) {
        Ok(p) => Some(p.to),
        Err(e) => {
            log::warn!("ignoring corrupt migration-pending marker: {e}");
            None
        }
    }
}

fn clear_pending(default_dir: &Path) {
    let _ = fs::remove_file(default_dir.join(PENDING_FILE));
}

/// Details of the most recent failed relocation, or `None` after a clean
/// start. The file lives next to the markers (the database may not have
/// opened yet when a failure happens), so the settings UI can show why a
/// pending move keeps retrying instead of failing invisibly.
pub(crate) fn read_last_error(default_dir: &Path) -> Option<String> {
    fs::read_to_string(default_dir.join(ERROR_FILE))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn record_failure(default_dir: &Path, to: &Path, error: &str) {
    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let _ = fs::write(
        default_dir.join(ERROR_FILE),
        format!("{stamp} | {} | {error}", to.display()),
    );
}

/// Persistent record of why the last session fell back to the default
/// directory: written next to the other markers so a later investigation can
/// tell an unmounted drive from a deleted database file. Overwritten by the
/// next fallback.
fn record_fallback(default_dir: &Path, configured: &Path, reason: &str) {
    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let _ = fs::write(
        default_dir.join(FALLBACK_FILE),
        format!("{stamp} | {} | {reason}", configured.display()),
    );
}

// ── Startup ──────────────────────────────────────────────────────────────

/// Complete the startup database setup: run any pending relocation first
/// (before anything else has opened the database), then resolve the effective
/// location. `rebind` rewrites the local GitHub-sync installation binding for
/// the new path — only the path part of the binding depends on the database
/// location, so without it syncing would refuse the relocated copy.
///
/// Idempotent at every step: a crash between any two steps leaves the source
/// database untouched and the pending marker in place, so the next start
/// simply retries.
///
/// When the configured directory is unreachable, this session runs on the
/// default directory — and a pending relocation is deliberately DEFERRED:
/// running it would copy the stale fallback snapshot and mark the target as
/// configured, silently activating old data. Deferring also keeps a
/// "move back home" request alive instead of mistaking the fallback for the
/// already-active directory.
pub fn prepare_startup(
    default_dir: &Path,
    rebind: &dyn Fn(&Connection, &Path) -> Result<(), String>,
) -> (DataPaths, Option<Notice>) {
    prepare_startup_with_grace(
        default_dir,
        rebind,
        OVERRIDE_PROBE_ATTEMPTS,
        OVERRIDE_PROBE_INTERVAL,
    )
}

fn prepare_startup_with_grace(
    default_dir: &Path,
    rebind: &dyn Fn(&Connection, &Path) -> Result<(), String>,
    attempts: u32,
    interval: Duration,
) -> (DataPaths, Option<Notice>) {
    let (base, notice) = resolve_with_grace(default_dir, attempts, interval);
    if matches!(notice, Some(Notice::FellBackToDefault { .. })) {
        return (base, notice);
    }
    let Some(to) = read_pending_to(default_dir) else {
        return (base, notice);
    };
    if same_dir(&to, &base.data_dir) {
        // Stale marker pointing at the already-active directory — nothing to
        // move, just drop it.
        clear_pending(default_dir);
        return (base, notice);
    }
    match relocate(default_dir, &base, &to, rebind) {
        Ok(()) => {
            clear_pending(default_dir);
            let _ = fs::remove_file(default_dir.join(ERROR_FILE));
            let (final_paths, fallback) = resolve_with_grace(default_dir, attempts, interval);
            return (final_paths, fallback);
        }
        Err(error) => {
            record_failure(default_dir, &to, &error);
            return (base, Some(Notice::MigrationFailed { to, error }));
        }
    }
}

fn relocate(
    default_dir: &Path,
    source: &DataPaths,
    to: &Path,
    rebind: &dyn Fn(&Connection, &Path) -> Result<(), String>,
) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("creating target directory: {e}"))?;

    // ── Copy the database page by page ──
    // Backing up into a scratch file keeps any target-side accident from
    // touching the real destination name. Large databases: big steps, tiny
    // pauses — speed matters at startup.
    let mut src = Connection::open(&source.db_path).map_err(|e| format!("opening source: {e}"))?;
    let tmp = to.join(TMP_DB_FILE);
    let _ = fs::remove_file(&tmp);
    let mut dst =
        Connection::open(&tmp).map_err(|e| format!("opening target scratch copy: {e}"))?;
    backup(&mut src, &mut dst)?;
    verify(&src, &dst)?;

    // Flush and release both sides before touching file names. Best-effort
    // WAL checkpoint on the source keeps its sidecar files small.
    let _ = src.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)");
    drop(src);
    drop(dst);

    // ── Activate ──
    let final_db = to.join(DB_FILE);
    if final_db.exists() {
        // Rebuilding into a configured directory (e.g. moving back to the
        // default): the existing file is the stale pre-move snapshot and the
        // pending request explicitly asked for this destination.
        fs::remove_file(&final_db).map_err(|e| format!("clearing old target copy: {e}"))?;
    }
    fs::rename(&tmp, &final_db).map_err(|e| format!("activating target copy: {e}"))?;

    // ── Post-copy bookkeeping (fails here leave the source active) ──
    let conn = Connection::open(&final_db).map_err(|e| format!("opening relocated copy: {e}"))?;
    rebind(&conn, &final_db)?;
    conn.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![PREVIOUS_DIR_SETTING, source.data_dir.to_string_lossy()],
    )
    .map_err(|e| format!("recording the old directory: {e}"))?;
    drop(conn);

    if same_dir(to, default_dir) {
        // Moving back home: the override marker has served its purpose.
        let _ = fs::remove_file(default_dir.join(OVERRIDE_FILE));
    } else {
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            to.to_string_lossy().as_bytes(),
        )
        .map_err(|e| format!("writing the override marker: {e}"))?;
    }
    Ok(())
}

fn backup(src: &mut Connection, dst: &mut Connection) -> Result<(), String> {
    let backup =
        rusqlite::backup::Backup::new(src, dst).map_err(|e| format!("starting backup: {e}"))?;
    backup
        .run_to_completion(8192, Duration::from_millis(1), None)
        .map_err(|e| format!("backing up: {e}"))
}

/// The copy is only accepted as a replacement of the original when SQLite's
/// own integrity check passes and every business row made it across; a
/// partial restart (power loss) leaves the source intact instead.
fn verify(src: &Connection, dst: &Connection) -> Result<(), String> {
    let integrity: String = dst
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|e| format!("integrity check: {e}"))?;
    if integrity != "ok" {
        return Err(format!(
            "integrity check on the copy returned {integrity:?}"
        ));
    }
    for table in ["articles", "feeds"] {
        let count = |conn: &Connection| -> Result<i64, String> {
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .map_err(|e| format!("counting {table}: {e}"))
        };
        let (a, b) = (count(src)?, count(dst)?);
        if a != b {
            return Err(format!(
                "{table} row count differs between source ({a}) and copy ({b})"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    /// A unique scratch root per test, under the system temp directory.
    fn scratch(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("papr-data-dir-{}-{}", process::id(), name));
        let _ = fs::remove_dir_all(&root);
        let default_dir = root.join("default");
        let source_dir = default_dir.clone(); // source *is* the default dir unless overridden
        let target_dir = root.join("target");
        fs::create_dir_all(&default_dir).unwrap();
        (default_dir, source_dir, target_dir)
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }

    fn seed_db(path: &Path) {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(
            "CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT);
             CREATE TABLE articles(id INTEGER PRIMARY KEY);
             CREATE TABLE feeds(id INTEGER PRIMARY KEY);
             INSERT INTO articles(id) VALUES (1),(2),(3);
             INSERT INTO feeds(id) VALUES (1);",
        )
        .unwrap();
    }

    fn no_rebind(_: &Connection, _: &Path) -> Result<(), String> {
        Ok(())
    }

    #[test]
    fn resolve_defaults_without_override() {
        let (default_dir, _, _) = scratch("no-override");
        let (paths, notice) = resolve(&default_dir);
        assert_eq!(paths.data_dir, default_dir);
        assert!(notice.is_none());
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn resolve_uses_override_when_the_database_exists() {
        let (default_dir, _, target_dir) = scratch("override");
        fs::create_dir_all(&target_dir).unwrap();
        seed_db(&target_dir.join(DB_FILE));
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            target_dir.to_string_lossy().as_bytes(),
        )
        .unwrap();
        let (paths, notice) = resolve(&default_dir);
        assert_eq!(paths.db_path, target_dir.join(DB_FILE));
        assert!(notice.is_none());
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn resolve_falls_back_when_override_db_is_missing() {
        let (default_dir, _, target_dir) = scratch("fallback");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            target_dir.to_string_lossy().as_bytes(),
        )
        .unwrap();
        let (paths, notice) = resolve_with_grace(&default_dir, 2, Duration::from_millis(1));
        assert_eq!(paths.data_dir, default_dir);
        match notice {
            Some(Notice::FellBackToDefault { configured }) => assert_eq!(configured, target_dir),
            _ => panic!("expected a fallback notice"),
        }
        let recorded = fs::read_to_string(default_dir.join(FALLBACK_FILE)).unwrap();
        assert!(recorded.contains("papr.db missing"), "got: {recorded}");
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn relocation_moves_the_database_and_rewrites_the_markers() {
        let (default_dir, _, target_dir) = scratch("move");
        seed_db(&default_dir.join(DB_FILE));
        write_pending(&default_dir, &target_dir).unwrap();

        let (paths, notice) = prepare_startup(&default_dir, &no_rebind);
        assert!(notice.is_none());
        assert_eq!(paths.db_path, target_dir.join(DB_FILE));

        // The copy is complete and consistent.
        let conn = Connection::open(&paths.db_path).unwrap();
        let articles: i64 = conn
            .query_row("SELECT COUNT(*) FROM articles", [], |r| r.get(0))
            .unwrap();
        assert_eq!(articles, 3);
        let previous: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'data_dir_previous'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(previous, default_dir.to_string_lossy());
        drop(conn);

        // Override points at the new directory; the pending marker is gone.
        let override_text = fs::read_to_string(default_dir.join(OVERRIDE_FILE)).unwrap();
        assert_eq!(override_text.trim(), target_dir.to_string_lossy());
        assert!(!default_dir.join(PENDING_FILE).exists());
        assert!(read_last_error(&default_dir).is_none());

        // The source snapshot survives untouched as the rollback path.
        assert!(default_dir.join(DB_FILE).is_file());
        cleanup(paths.data_dir.parent().unwrap());
    }

    #[test]
    fn relocation_failure_keeps_the_source_active() {
        let (default_dir, _, target_dir) = scratch("fail");
        seed_db(&default_dir.join(DB_FILE));
        write_pending(&default_dir, &target_dir).unwrap();

        let failing = |_: &Connection, _: &Path| Err("boom".to_string());
        let (_, notice) = prepare_startup(&default_dir, &failing);
        match notice {
            Some(Notice::MigrationFailed { error, .. }) => assert!(error.contains("boom")),
            _ => panic!("expected a migration-failed notice"),
        }
        // Request stands so the next start retries; nothing was activated,
        // and the failure is recorded for the settings UI.
        assert!(default_dir.join(PENDING_FILE).exists());
        assert!(!default_dir.join(OVERRIDE_FILE).exists());
        assert!(default_dir.join(DB_FILE).is_file());
        let recorded = read_last_error(&default_dir).expect("failure must be recorded");
        assert!(recorded.contains("boom"));
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn stale_pending_targeting_the_active_directory_is_dropped() {
        let (default_dir, _, _) = scratch("stale");
        seed_db(&default_dir.join(DB_FILE));
        write_pending(&default_dir, &default_dir).unwrap();
        let (paths, notice) = prepare_startup(&default_dir, &no_rebind);
        assert_eq!(paths.data_dir, default_dir);
        assert!(notice.is_none());
        assert!(!default_dir.join(PENDING_FILE).exists());
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn pending_relocation_is_deferred_while_the_configured_dir_is_unreachable() {
        let (default_dir, _, target_dir) = scratch("defer");
        seed_db(&default_dir.join(DB_FILE));
        // The configured directory "disappeared": the override marker still
        // names it, but its papr.db is gone → resolve falls back.
        let gone_dir = default_dir.parent().unwrap().join("gone");
        fs::create_dir_all(&gone_dir).unwrap();
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            gone_dir.to_string_lossy().as_bytes(),
        )
        .unwrap();
        write_pending(&default_dir, &target_dir).unwrap();

        let (paths, notice) =
            prepare_startup_with_grace(&default_dir, &no_rebind, 2, Duration::from_millis(1));
        assert_eq!(paths.data_dir, default_dir);
        match notice {
            Some(Notice::FellBackToDefault { configured }) => assert_eq!(configured, gone_dir),
            _ => panic!("expected a fallback notice"),
        }
        // The request survives for a start where the real source is back;
        // nothing was copied from the stale fallback snapshot, and the
        // override marker is untouched.
        assert!(default_dir.join(PENDING_FILE).exists());
        assert!(!target_dir.join(DB_FILE).exists());
        let override_text = fs::read_to_string(default_dir.join(OVERRIDE_FILE)).unwrap();
        assert_eq!(override_text.trim(), gone_dir.to_string_lossy());
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn move_home_request_survives_a_fallback_start() {
        let (default_dir, _, _) = scratch("defer-home");
        seed_db(&default_dir.join(DB_FILE));
        let gone_dir = default_dir.parent().unwrap().join("gone-home");
        fs::create_dir_all(&gone_dir).unwrap();
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            gone_dir.to_string_lossy().as_bytes(),
        )
        .unwrap();
        write_pending(&default_dir, &default_dir).unwrap();

        let (paths, notice) =
            prepare_startup_with_grace(&default_dir, &no_rebind, 2, Duration::from_millis(1));
        assert_eq!(paths.data_dir, default_dir);
        assert!(matches!(notice, Some(Notice::FellBackToDefault { .. })));
        // The fallback directory must NOT be mistaken for the configured one:
        // the request is kept instead of being dropped as a stale marker.
        assert!(default_dir.join(PENDING_FILE).exists());
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn relocation_carries_uncheckpointed_wal_content() {
        // The real application database is always WAL (db::open sets it), and
        // a freshly closed app leaves committed rows in the -wal sidecar. The
        // copy must carry them; a plain-file copy would silently lose them.
        let (default_dir, _, target_dir) = scratch("wal");
        let keep = Connection::open(default_dir.join(DB_FILE)).unwrap();
        keep.pragma_update(None, "journal_mode", "WAL").unwrap();
        keep.execute_batch(
            "CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT);
             CREATE TABLE articles(id INTEGER PRIMARY KEY);
             CREATE TABLE feeds(id INTEGER PRIMARY KEY);
             INSERT INTO articles(id) VALUES (1),(2),(3);
             INSERT INTO feeds(id) VALUES (1);",
        )
        .unwrap();
        // Hold `keep` open so nothing checkpoints the WAL before the copy.

        write_pending(&default_dir, &target_dir).unwrap();
        let (paths, notice) = prepare_startup(&default_dir, &no_rebind);
        assert!(
            notice.is_none(),
            "migration must handle a WAL source, got: {:?}",
            notice.is_some()
        );
        assert_eq!(paths.db_path, target_dir.join(DB_FILE));

        drop(keep);
        let conn = Connection::open(&paths.db_path).unwrap();
        let articles: i64 = conn
            .query_row("SELECT COUNT(*) FROM articles", [], |r| r.get(0))
            .unwrap();
        assert_eq!(articles, 3, "committed WAL rows must reach the copy");
        drop(conn);
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn relocation_back_to_the_default_clears_the_override() {
        let root = std::env::temp_dir().join(format!("papr-data-dir-{}-back", process::id()));
        let _ = fs::remove_dir_all(&root);
        let default_dir = root.join("default");
        let home_dir = root.join("home");
        fs::create_dir_all(&default_dir).unwrap();
        fs::create_dir_all(&home_dir).unwrap();
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            home_dir.to_string_lossy().as_bytes(),
        )
        .unwrap();
        seed_db(&home_dir.join(DB_FILE)); // the live copy sits in the configured dir
        write_pending(&default_dir, &default_dir).unwrap();

        let (paths, notice) = prepare_startup(&default_dir, &no_rebind);
        assert!(notice.is_none());
        assert_eq!(paths.db_path, default_dir.join(DB_FILE));
        assert!(!default_dir.join(OVERRIDE_FILE).exists());
        // The stale snapshot in the default directory has been replaced by
        // the live one.
        let conn = Connection::open(paths.db_path).unwrap();
        let articles: i64 = conn
            .query_row("SELECT COUNT(*) FROM articles", [], |r| r.get(0))
            .unwrap();
        assert_eq!(articles, 3);
        drop(conn);
        cleanup(&root);
    }

    #[test]
    fn a_slow_directory_is_waited_for_instead_of_falling_back() {
        let (default_dir, _, target_dir) = scratch("slow");
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            target_dir.to_string_lossy().as_bytes(),
        )
        .unwrap();
        // The drive "comes up" after the first probe: the directory and its
        // database only appear while resolve is waiting between probes.
        let target = target_dir.clone();
        let writer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            fs::create_dir_all(&target).unwrap();
            seed_db(&target.join(DB_FILE));
        });
        let (paths, notice) = resolve_with_grace(&default_dir, 8, Duration::from_millis(200));
        writer.join().unwrap();
        assert!(notice.is_none(), "a slow directory must not fall back");
        assert_eq!(paths.db_path, target_dir.join(DB_FILE));
        assert!(!default_dir.join(FALLBACK_FILE).exists());
        cleanup(default_dir.parent().unwrap());
    }

    #[test]
    fn every_fallback_records_why_it_happened() {
        let (default_dir, _, target_dir) = scratch("reason");
        let interval = Duration::from_millis(1);

        // A relative override path is reported, never probed.
        fs::write(default_dir.join(OVERRIDE_FILE), "relative\\dir").unwrap();
        let (_, notice) = resolve_with_grace(&default_dir, 2, interval);
        assert!(matches!(notice, Some(Notice::FellBackToDefault { .. })));
        let recorded = fs::read_to_string(default_dir.join(FALLBACK_FILE)).unwrap();
        assert!(recorded.contains("not absolute"), "got: {recorded}");

        // The configured directory itself is gone (unmounted drive).
        let missing = default_dir.parent().unwrap().join("not-mounted");
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            missing.to_string_lossy().as_bytes(),
        )
        .unwrap();
        let (_, notice) = resolve_with_grace(&default_dir, 2, interval);
        assert!(matches!(notice, Some(Notice::FellBackToDefault { .. })));
        let recorded = fs::read_to_string(default_dir.join(FALLBACK_FILE)).unwrap();
        assert!(recorded.contains("unreachable"), "got: {recorded}");
        assert!(recorded.contains(&*missing.to_string_lossy()));

        // The directory exists but its database file is gone.
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(
            default_dir.join(OVERRIDE_FILE),
            target_dir.to_string_lossy().as_bytes(),
        )
        .unwrap();
        let (_, notice) = resolve_with_grace(&default_dir, 2, interval);
        assert!(matches!(notice, Some(Notice::FellBackToDefault { .. })));
        let recorded = fs::read_to_string(default_dir.join(FALLBACK_FILE)).unwrap();
        assert!(recorded.contains("papr.db missing"), "got: {recorded}");
        assert!(recorded.contains(&*target_dir.to_string_lossy()));

        cleanup(default_dir.parent().unwrap());
    }
}
