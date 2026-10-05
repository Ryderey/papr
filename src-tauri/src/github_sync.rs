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
use std::{future::Future, sync::Arc};
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
    let path = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::code("githubInvalidBinding"))?
        .join("papr.db");
    let path = std::fs::canonicalize(path)
        .map_err(|_| AppError::code("githubInvalidBinding"))?
        .to_string_lossy()
        .to_lowercase();
    Ok(stable_key(&["installation:v1", &marker, &path]))
}
#[tauri::command]
pub async fn github_status(app: AppHandle) -> AppResult<Status> {
    service(&app).status().await.map_err(map)
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
    credentials::set(&profile.credential_ref, &token)
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
    if result.is_ok() {
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
        loop {
            tick.tick().await;
            if let Ok(status) = service(&app).status().await {
                if status.profile.is_some() && status.automatic_due {
                    let _ = github_sync_now(app.clone()).await;
                }
            }
        }
    });
}
