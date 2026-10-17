//! Thin desktop adapter over the shared GitHub synchronization service.
use crate::{
    error::{AppError, AppResult},
    github_credentials as credentials,
    state::AppState,
};
use papr_core::{
    error::CoreError,
    sync::github::{
        model::stable_key,
        service::{Cancellation, GitHubService, LocalStore, Preview, Status, SyncReport},
        storage,
    },
};
use rusqlite::Connection;
use std::{future::Future, path::Path, sync::Arc};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone)]
struct DesktopStore(AppHandle);
impl LocalStore for DesktopStore {
    fn with<T: Send + 'static>(
        &self,
        action: impl FnOnce(&Connection) -> Result<T, CoreError> + Send + 'static,
    ) -> impl Future<Output = Result<T, CoreError>> + Send {
        let app = self.0.clone();
        async move {
            let state = app.state::<AppState>();
            let mut conn = state.db.lock().await;
            let tx = conn
                .transaction()
                .map_err(|e| CoreError::Db(e.to_string()))?;
            let result = action(&tx)?;
            tx.commit().map_err(|e| CoreError::Db(e.to_string()))?;
            Ok(result)
        }
    }
}
fn service(app: &AppHandle) -> GitHubService<DesktopStore> {
    GitHubService::new(
        DesktopStore(app.clone()),
        Arc::new(app.state::<AppState>().http()),
    )
}
fn map(error: CoreError) -> AppError {
    AppError::code(error.code())
}

/// The canonical, install-stable binding string for a database location. Both
/// live binding generation and post-move regeneration share the normalization
/// (canonicalize + lowercase), so a relocated copy keeps matching only if both
/// sides compute the key identically.
fn installation_binding(marker: &str, db_path: &Path) -> AppResult<String> {
    let path = std::fs::canonicalize(db_path)
        .map_err(|_| AppError::code("githubInvalidBinding"))?
        .to_string_lossy()
        .to_lowercase();
    Ok(stable_key(&["installation:v1", marker, &path]))
}
async fn binding(app: &AppHandle, create: bool) -> AppResult<String> {
    let marker = match credentials::get("papr.sync.installation")? {
        Some(marker) => marker,
        None if create => {
            let state = app.state::<AppState>();
            let conn = state.db.lock().await;
            let marker = storage::random_id(&conn).map_err(map)?;
            credentials::set("papr.sync.installation", &marker)?;
            marker
        }
        None => return Err(AppError::code("githubInstallationMissing")),
    };
    let path = app.state::<AppState>().db_path.clone();
    Ok(installation_binding(&marker, &path)?)
}

/// Regenerate the local installation binding for a relocated database. The
/// binding is a local multi-copy fence keyed partly by the database path, so
/// a copy landed in a new directory would be rejected by `acquire()` until
/// its stored binding is rewritten. The installation marker lives in system
/// credential storage and is path-independent; without one, GitHub sync has
/// never been connected on this installation and there is nothing to fix.
///
/// Failure strings carry step detail — the caller relays them verbatim into
/// the migration error the user sees.
pub fn rebind_installation(conn: &Connection, db_path: &Path) -> Result<(), String> {
    let marker = match credentials::get("papr.sync.installation") {
        Ok(Some(marker)) => marker,
        Ok(None) => return Ok(()),
        Err(e) => return Err(format!("reading the installation marker failed: {e}")),
    };
    rebind_with_marker(conn, db_path, &marker)
}

/// The vault-independent half of the rebind, split out so a test can drive
/// it with a real connection (the numbered placeholders here are a runtime
/// contract that only executing the statement catches).
fn rebind_with_marker(conn: &Connection, db_path: &Path, marker: &str) -> Result<(), String> {
    let binding = installation_binding(marker, db_path).map_err(|e| e.to_string())?;
    let changed = conn
        .execute(
            "UPDATE github_connections SET binding = ?1 WHERE active = 1",
            rusqlite::params![binding],
        )
        .map_err(|e| format!("rewriting the installation binding failed: {e}"))?;
    if changed > 0 {
        log::info!("github sync: installation binding rewritten for the relocated database");
    }
    Ok(())
}
#[tauri::command]
pub async fn github_status(app: AppHandle) -> AppResult<Status> {
    service(&app).status().await.map_err(map)
}
#[tauri::command]
pub async fn github_schedule(app: AppHandle) -> AppResult<papr_core::sync::github::schedule::Schedule> {
    service(&app).schedule().await.map_err(map)
}
#[tauri::command]
pub async fn github_set_schedule(app: AppHandle, schedule: papr_core::sync::github::schedule::Schedule) -> AppResult<()> {
    service(&app).set_schedule(schedule).await.map_err(map)
}
#[tauri::command]
pub async fn github_preview(
    app: AppHandle,
    owner: String,
    repo: String,
    branch: Option<String>,
    token: String,
) -> AppResult<Preview> {
    let reference = {
        let state = app.state::<AppState>();
        let conn = state.db.lock().await;
        format!("papr.sync.{}", storage::random_id(&conn).map_err(map)?)
    };
    service(&app)
        .preview(owner, repo, branch, reference, &token)
        .await
        .map_err(map)
}
#[tauri::command]
pub async fn github_connect(app: AppHandle, preview: Preview, token: String) -> AppResult<()> {
    let service = service(&app);
    if service.status().await.map_err(map)?.profile.is_some() {
        return Err(AppError::code("githubAlreadyConnected"));
    }
    let binding = binding(&app, true).await?;
    let reference = preview.profile.credential_ref.clone();
    credentials::set(&reference, &token)?;
    if let Err(error) = service.connect(preview, &token, binding).await {
        // A concurrent confirmation may now own this credential reference.
        if let Ok(status) = service.status().await {
            if status.profile.as_ref().map(|p| p.credential_ref.as_str())
                != Some(reference.as_str())
            {
                let _ = credentials::delete(&reference);
            }
        }
        return Err(map(error));
    }
    Ok(())
}
#[tauri::command]
pub async fn github_update_credential(app: AppHandle, token: String) -> AppResult<()> {
    let service = service(&app);
    let status = service.status().await.map_err(map)?;
    let profile = status
        .profile
        .ok_or_else(|| AppError::code("githubNotConnected"))?;
    service.verify_credential(&token).await.map_err(map)?;
    credentials::set(&profile.credential_ref, &token)?;
    service
        .credential_updated(profile.credential_ref)
        .await
        .map_err(map)
}
#[tauri::command]
pub async fn github_disconnect(app: AppHandle) -> AppResult<()> {
    if let Some(reference) = service(&app).disconnect().await.map_err(map)? {
        credentials::delete(&reference)?;
    }
    Ok(())
}
#[tauri::command]
pub async fn github_sync_now(app: AppHandle) -> AppResult<SyncReport> {
    let result = sync_now(&app).await;
    if let Err(AppError::Coded(code)) = &result {
        if *code != "githubSyncBusy" && *code != "githubSyncCancelled" {
            let _ = service(&app).record_platform_error(code).await;
        }
    }
    result
}
async fn sync_now(app: &AppHandle) -> AppResult<SyncReport> {
    let service = service(&app);
    let status = service.status().await.map_err(map)?;
    let profile = status
        .profile
        .ok_or_else(|| AppError::code("githubNotConnected"))?;
    let token = credentials::get(&profile.credential_ref)?
        .filter(|t| !t.is_empty())
        .ok_or_else(|| AppError::code("syncCredentialMissing"))?;
    let binding = binding(&app, false).await?;
    let cancel = Cancellation::default();
    let state = app.state::<AppState>();
    let checkpoint_ref = format!(
        "papr.sync.checkpoint_{}",
        profile.credential_ref.trim_start_matches("papr.sync.")
    );
    let checkpoint = service
        .checkpoint(credentials::get(&checkpoint_ref)?)
        .await
        .map_err(map)?;
    credentials::set(&checkpoint_ref, &checkpoint)?;
    {
        let mut active = state.github_cancel.lock().await;
        if active.is_some() {
            return Err(AppError::code("githubSyncBusy"));
        }
        *active = Some(cancel.clone());
    }
    let result = service.sync(token, binding, cancel).await.map_err(map);
    *state.github_cancel.lock().await = None;
    let checkpoint = service
        .checkpoint(credentials::get(&checkpoint_ref)?)
        .await
        .map_err(map)?;
    credentials::set(&checkpoint_ref, &checkpoint)?;
    if result.as_ref().is_ok_and(|report| !report.unchanged) {
        let _ = app.emit("feeds-updated", ());
        let _ = app.emit("articles-updated", ());
    }
    result
}
#[tauri::command]
pub async fn github_cancel_sync(app: AppHandle) -> AppResult<()> {
    if let Some(cancel) = app.state::<AppState>().github_cancel.lock().await.as_ref() {
        cancel.cancel();
    }
    Ok(())
}

pub fn spawn_scheduler(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(10));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            if let Ok(due) = service(&app).automatic_due().await {
                if due {
                    let _ = github_sync_now(app.clone()).await;
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drives the real SQL of the relocated-database rebind. The statement
    /// uses numbered placeholders, a contract that only fails at run time —
    /// parameter-count mismatches compile and pass review silently.
    #[test]
    fn rebind_rewrites_only_the_active_connection_binding() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE github_connections(active INTEGER NOT NULL, binding TEXT NOT NULL);
             INSERT INTO github_connections(active, binding) VALUES (1, 'stale-path-binding'), (0, 'dormant');",
        )
        .unwrap();

        // The binding is derived from the canonical database path, which
        // requires the file to exist.
        let dir = std::env::temp_dir().join(format!("papr-rebind-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("papr.db");
        std::fs::write(&db_path, b"").unwrap();

        rebind_with_marker(&conn, &db_path, "marker-1").expect("rebind must succeed");

        let expected = installation_binding("marker-1", &db_path).unwrap();
        let active: String = conn
            .query_row(
                "SELECT binding FROM github_connections WHERE active = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let dormant: String = conn
            .query_row(
                "SELECT binding FROM github_connections WHERE active = 0",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            active, expected,
            "the active row must carry the new binding"
        );
        assert_eq!(dormant, "dormant", "inactive rows must stay untouched");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
