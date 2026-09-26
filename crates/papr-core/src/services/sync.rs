use std::sync::Arc;

use chrono::{DateTime, Utc};
use reqwest::Client;

use crate::db::Db;
use crate::error::{CoreError, ErrorCategory};
use crate::sync::greader::GReaderSyncPort;
use crate::sync::{highest_contiguous_acknowledgement, SyncPort, SyncProfile};

use super::settings::SettingsService;

const SYNC_BATCH_SIZE: usize = 500;
const BACKGROUND_SYNC_INTERVAL_HOURS: i64 = 6;

/// Secret-free state shown by Flutter and queried by the existing Worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncStatus {
    pub profile: Option<SyncProfile>,
    pub last_success_at: Option<String>,
    pub last_error_code: Option<String>,
    pub background_due: bool,
}

/// Result of safely pushing one local sync batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncPushReport {
    pub attempted: usize,
    pub acknowledged: usize,
    pub advanced_to: Option<i64>,
}

/// Coordinates Core's durable local outbox with one provider port.
pub struct SyncService {
    db: Arc<Db>,
}

impl SyncService {
    pub fn new(db: Arc<Db>) -> Self {
        Self { db }
    }

    /// Probe credentials without changing the saved connection.
    pub async fn test_connection(
        &self,
        profile: &SyncProfile,
        credential: String,
        http: Arc<Client>,
    ) -> Result<(), CoreError> {
        let mut port = GReaderSyncPort::new(profile, credential, http)?;
        port.validate().await
    }

    /// Replace the saved metadata only after a successful network probe.
    pub async fn connect(
        &self,
        profile: SyncProfile,
        credential: String,
        http: Arc<Client>,
    ) -> Result<(), CoreError> {
        self.test_connection(&profile, credential, http).await?;
        SettingsService::new(Arc::clone(&self.db))
            .save_sync_profile(profile)
            .await
    }

    pub async fn status(&self) -> Result<SyncStatus, CoreError> {
        let profile = SettingsService::new(Arc::clone(&self.db))
            .get_sync_profile()
            .await?;
        let last_success_at = self
            .db
            .get_setting("sync_last_success_at")?
            .filter(|value| !value.is_empty());
        let last_error_code = self
            .db
            .get_setting("sync_last_error_code")?
            .filter(|value| !value.is_empty());
        let background_due = profile.is_some()
            && last_error_code.is_none()
            && last_success_at
                .as_deref()
                .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
                .is_none_or(|last| {
                    Utc::now().signed_duration_since(last).num_hours()
                        >= BACKGROUND_SYNC_INTERVAL_HOURS
                });
        Ok(SyncStatus {
            profile,
            last_success_at,
            last_error_code,
            background_due,
        })
    }

    /// Use the saved connection and one transient Keystore credential for a
    /// complete push/pull pass. Only the stable failure code is persisted.
    pub async fn sync_configured(
        &self,
        credential: String,
        http: Arc<Client>,
    ) -> Result<usize, CoreError> {
        let profile = SettingsService::new(Arc::clone(&self.db))
            .get_sync_profile()
            .await?
            .ok_or_else(|| CoreError::coded(ErrorCategory::Sync, "syncNotConnected", None))?;
        let result = async {
            let mut port = GReaderSyncPort::new(&profile, credential, http)?;
            self.sync_once(&profile.credential_ref, &mut port).await
        }
        .await;
        let db = Arc::clone(&self.db);
        match result {
            Ok(merged) => {
                let now = Utc::now().to_rfc3339();
                tokio::task::spawn_blocking(move || {
                    db.set_settings(&[
                        ("sync_last_success_at", now),
                        ("sync_last_error_code", String::new()),
                    ])
                })
                .await
                .map_err(blocking_error)??;
                Ok(merged)
            }
            Err(error) => {
                let code = error.code().to_string();
                let _ = tokio::task::spawn_blocking(move || {
                    db.set_setting("sync_last_error_code", &code)
                })
                .await;
                Err(error)
            }
        }
    }

    /// Validate a provider, send its next durable local batch, and advance its
    /// push cursor only through an acknowledged prefix.
    pub async fn push_pending<P: SyncPort>(
        &self,
        provider: &str,
        port: &mut P,
    ) -> Result<SyncPushReport, CoreError> {
        port.validate().await?;
        let cursor_key = format!("{provider}:push");
        let db = Arc::clone(&self.db);
        let cursor_key_for_read = cursor_key.clone();
        let after_sequence =
            tokio::task::spawn_blocking(move || db.get_sync_cursor(&cursor_key_for_read))
                .await
                .map_err(blocking_error)?
                .and_then(parse_push_cursor)?;

        let provider_key = provider.to_string();
        let db = Arc::clone(&self.db);
        let changes = tokio::task::spawn_blocking(move || {
            db.pending_sync_changes(&provider_key, after_sequence, SYNC_BATCH_SIZE)
        })
        .await
        .map_err(blocking_error)??;
        if changes.is_empty() {
            return Ok(SyncPushReport {
                attempted: 0,
                acknowledged: 0,
                advanced_to: None,
            });
        }

        let acknowledged = port.push(&changes).await?;
        let advanced_to = highest_contiguous_acknowledgement(&changes, &acknowledged);
        if let Some(sequence) = advanced_to {
            let db = Arc::clone(&self.db);
            tokio::task::spawn_blocking(move || {
                db.set_sync_cursor(&cursor_key, &sequence.to_string())
            })
            .await
            .map_err(blocking_error)??;
        }
        let acknowledged = changes
            .iter()
            .take_while(|change| advanced_to.is_some_and(|sequence| change.sequence <= sequence))
            .count();
        Ok(SyncPushReport {
            attempted: changes.len(),
            acknowledged,
            advanced_to,
        })
    }

    /// Push local changes before applying a remote delta. If a push is only
    /// partially acknowledged, the merge protects every remaining local
    /// article mutation from the remote snapshot.
    pub async fn sync_once<P: SyncPort>(
        &self,
        provider: &str,
        port: &mut P,
    ) -> Result<usize, CoreError> {
        self.drain_pending(provider, port).await?;
        let push_key = format!("{provider}:push");
        let pull_key = format!("{provider}:pull");
        let db = Arc::clone(&self.db);
        let (push_cursor, pull_cursor) = tokio::task::spawn_blocking(move || {
            Ok::<_, CoreError>((
                parse_push_cursor(db.get_sync_cursor(&push_key)?)?,
                db.get_sync_cursor(&pull_key)?,
            ))
        })
        .await
        .map_err(blocking_error)??;
        if let Some(cursor) = pull_cursor.as_deref() {
            port.acknowledge(cursor).await?;
        }
        let pull = port.pull(pull_cursor.as_deref()).await?;
        let next_cursor = pull.cursor.clone();
        let provider_key = provider.to_string();
        let db = Arc::clone(&self.db);
        let merged = tokio::task::spawn_blocking(move || {
            db.apply_sync_pull(&provider_key, &pull, push_cursor)
        })
        .await
        .map_err(blocking_error)??;
        if let Some(cursor) = next_cursor.as_deref() {
            port.acknowledge(cursor).await?;
        }
        // The pull can establish a remote article ID needed by a pending
        // local read/star mutation. Send it before reporting this pass done.
        self.drain_pending(provider, port).await?;
        Ok(merged)
    }

    async fn drain_pending<P: SyncPort>(
        &self,
        provider: &str,
        port: &mut P,
    ) -> Result<(), CoreError> {
        loop {
            let report = self.push_pending(provider, port).await?;
            if report.attempted == 0 || report.acknowledged < report.attempted {
                break;
            }
        }
        Ok(())
    }
}

fn parse_push_cursor(cursor: Option<String>) -> Result<i64, CoreError> {
    match cursor {
        None => Ok(0),
        Some(cursor) => cursor
            .parse::<i64>()
            .ok()
            .filter(|value| *value >= 0)
            .ok_or_else(|| CoreError::coded(ErrorCategory::Sync, "invalidSyncPushCursor", None)),
    }
}

fn blocking_error(error: tokio::task::JoinError) -> CoreError {
    CoreError::Platform(format!("blocking task failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{SyncChange, SyncEntity, SyncOperation};
    use crate::error::ErrorCategory;
    use crate::sync::{FakeSyncPort, RemoteSyncChange, SyncProvider, SyncPull};

    struct MappingPort {
        pull: SyncPull,
        pushed: Vec<SyncChange>,
    }

    impl SyncPort for MappingPort {
        async fn validate(&mut self) -> Result<(), CoreError> {
            Ok(())
        }

        async fn push(&mut self, changes: &[SyncChange]) -> Result<Vec<i64>, CoreError> {
            let ready = changes
                .iter()
                .filter(|change| change.entity != SyncEntity::Article || change.remote_id.is_some())
                .cloned()
                .collect::<Vec<_>>();
            self.pushed.extend(ready.iter().cloned());
            Ok(ready.iter().map(|change| change.sequence).collect())
        }

        async fn pull(&mut self, _cursor: Option<&str>) -> Result<SyncPull, CoreError> {
            Ok(self.pull.clone())
        }

        async fn acknowledge(&mut self, _cursor: &str) -> Result<(), CoreError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn one_sync_pass_pushes_article_state_after_pull_maps_its_remote_id() {
        let temp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&temp.path().join("test.db")).unwrap());
        let feed_id = db.add_feed("https://example.com/feed.xml").unwrap();
        let article_id = db
            .transact(|tx| {
                tx.execute(
                    "INSERT INTO articles (feed_id, guid, url, title) VALUES (?1, 'article-1', 'https://example.com/a', 'Article')",
                    [feed_id],
                )
                .map_err(|error| CoreError::Db(error.to_string()))?;
                Ok(tx.last_insert_rowid())
            })
            .unwrap();
        db.set_article_read(article_id, true).unwrap();
        let mut port = MappingPort {
            pull: SyncPull {
                cursor: None,
                changes: vec![RemoteSyncChange {
                    remote_id: "remote-article".into(),
                    entity: SyncEntity::Article,
                    operation: SyncOperation::Upsert,
                    field: Some("read".into()),
                    value: Some("0".into()),
                    url: Some("https://example.com/a".into()),
                    folder_remote_id: None,
                }],
            },
            pushed: Vec::new(),
        };
        SyncService::new(Arc::clone(&db))
            .sync_once("fake", &mut port)
            .await
            .unwrap();
        assert_eq!(
            db.get_sync_cursor("fake:push").unwrap().as_deref(),
            Some("2")
        );
        assert_eq!(port.pushed.len(), 2);
        assert_eq!(port.pushed[1].remote_id.as_deref(), Some("remote-article"));
        assert_eq!(port.pushed[1].value.as_deref(), Some("1"));
    }

    #[tokio::test]
    async fn background_sync_waits_for_its_window_and_pauses_after_an_error() {
        let temp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&temp.path().join("test.db")).unwrap());
        let service = SyncService::new(Arc::clone(&db));
        assert!(!service.status().await.unwrap().background_due);
        SettingsService::new(Arc::clone(&db))
            .save_sync_profile(SyncProfile {
                provider: SyncProvider::Miniflux,
                server_url: "https://reader.example".into(),
                username: "reader".into(),
                credential_ref: "papr.sync.test".into(),
            })
            .await
            .unwrap();
        assert!(service.status().await.unwrap().background_due);

        db.set_setting("sync_last_success_at", &Utc::now().to_rfc3339())
            .unwrap();
        assert!(!service.status().await.unwrap().background_due);
        db.set_setting(
            "sync_last_success_at",
            &(Utc::now() - chrono::Duration::hours(7)).to_rfc3339(),
        )
        .unwrap();
        assert!(service.status().await.unwrap().background_due);
        db.set_setting("sync_last_error_code", "syncUnavailable")
            .unwrap();
        assert!(!service.status().await.unwrap().background_due);
    }

    #[tokio::test]
    async fn failed_push_replays_without_advancing_the_cursor() {
        let temp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&temp.path().join("test.db")).unwrap());
        db.add_feed("https://example.com/feed.xml").unwrap();
        let service = SyncService::new(Arc::clone(&db));
        let mut port = FakeSyncPort::default();
        port.fail_next_push(CoreError::coded(
            ErrorCategory::Network,
            "syncUnavailable",
            None,
        ));

        assert_eq!(
            service
                .push_pending("fake", &mut port)
                .await
                .unwrap_err()
                .code(),
            "syncUnavailable"
        );
        assert_eq!(db.get_sync_cursor("fake:push").unwrap(), None);

        let report = service.push_pending("fake", &mut port).await.unwrap();
        assert_eq!(report.attempted, 1);
        assert_eq!(report.acknowledged, 1);
        assert_eq!(report.advanced_to, Some(1));
        assert_eq!(
            db.get_sync_cursor("fake:push").unwrap().as_deref(),
            Some("1")
        );
    }

    #[tokio::test]
    async fn partial_acknowledgement_replays_only_the_unconfirmed_suffix() {
        let temp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&temp.path().join("test.db")).unwrap());
        db.add_feed("https://example.com/one.xml").unwrap();
        db.add_feed("https://example.com/two.xml").unwrap();
        db.add_feed("https://example.com/three.xml").unwrap();
        let service = SyncService::new(Arc::clone(&db));
        let mut port = FakeSyncPort::default();
        port.acknowledge_push_sequences(&[1, 3]);

        let report = service.push_pending("fake", &mut port).await.unwrap();
        assert_eq!(report.attempted, 3);
        assert_eq!(report.acknowledged, 1);
        assert_eq!(report.advanced_to, Some(1));
        assert_eq!(
            db.get_sync_cursor("fake:push").unwrap().as_deref(),
            Some("1")
        );

        port.acknowledge_push_sequences(&[2, 3]);
        let report = service.push_pending("fake", &mut port).await.unwrap();
        assert_eq!(report.attempted, 2);
        assert_eq!(report.acknowledged, 2);
        assert_eq!(report.advanced_to, Some(3));
        assert_eq!(port.pushed_changes().len(), 3);
        assert_eq!(
            db.get_sync_cursor("fake:push").unwrap().as_deref(),
            Some("3")
        );
    }

    #[tokio::test]
    async fn pull_merges_additions_without_overwriting_an_unconfirmed_local_state() {
        let temp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&temp.path().join("test.db")).unwrap());
        let feed_id = db.add_feed("https://example.com/local.xml").unwrap();
        let article_id = db.transact(|tx| {
            tx.execute(
                "INSERT INTO articles (feed_id, guid, url, title) VALUES (?1, 'article-1', 'https://example.com/article-1', 'Article')",
                [feed_id],
            ).map_err(|error| CoreError::Db(error.to_string()))?;
            Ok(tx.last_insert_rowid())
        }).unwrap();
        db.set_article_read(article_id, true).unwrap();
        let service = SyncService::new(Arc::clone(&db));
        let mut port = FakeSyncPort::with_pull(SyncPull {
            cursor: Some("remote-1".to_string()),
            changes: vec![
                RemoteSyncChange {
                    remote_id: "folder-1".into(),
                    entity: SyncEntity::Folder,
                    operation: SyncOperation::Upsert,
                    field: None,
                    value: Some("News".into()),
                    url: None,
                    folder_remote_id: None,
                },
                RemoteSyncChange {
                    remote_id: "feed-1".into(),
                    entity: SyncEntity::Feed,
                    operation: SyncOperation::Upsert,
                    field: None,
                    value: Some("https://example.com/remote.xml".into()),
                    url: None,
                    folder_remote_id: Some("folder-1".into()),
                },
                RemoteSyncChange {
                    remote_id: "article-1".into(),
                    entity: SyncEntity::Article,
                    operation: SyncOperation::Upsert,
                    field: Some("read".into()),
                    value: Some("0".into()),
                    url: Some("https://example.com/article-1".into()),
                    folder_remote_id: None,
                },
                RemoteSyncChange {
                    remote_id: "deleted-feed".into(),
                    entity: SyncEntity::Feed,
                    operation: SyncOperation::Tombstone,
                    field: None,
                    value: Some("https://example.com/local.xml".into()),
                    url: None,
                    folder_remote_id: None,
                },
            ],
        });
        port.acknowledge_push_sequences(&[1]);

        assert_eq!(service.sync_once("fake", &mut port).await.unwrap(), 3);
        assert_eq!(db.list_feeds().unwrap().len(), 2);
        assert_eq!(db.list_folders().unwrap()[0].name, "News");
        assert_eq!(
            db.get_sync_cursor("fake:pull").unwrap().as_deref(),
            Some("remote-1")
        );
        assert_eq!(
            db.pending_sync_changes("fake", 1, 50).unwrap()[0]
                .remote_id
                .as_deref(),
            Some("article-1")
        );
        let read = db
            .transact(|tx| {
                tx.query_row(
                    "SELECT is_read FROM articles WHERE id = ?1",
                    [article_id],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(|error| CoreError::Db(error.to_string()))
            })
            .unwrap();
        assert!(read);

        port.acknowledge_push_sequences(&[2]);
        assert_eq!(service.sync_once("fake", &mut port).await.unwrap(), 1);
        let read = db
            .transact(|tx| {
                tx.query_row(
                    "SELECT is_read FROM articles WHERE id = ?1",
                    [article_id],
                    |row| row.get::<_, bool>(0),
                )
                .map_err(|error| CoreError::Db(error.to_string()))
            })
            .unwrap();
        assert!(!read);
        assert_eq!(db.list_feeds().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn failed_pull_keeps_the_pull_cursor_and_local_state_for_retry() {
        let temp = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::new(&temp.path().join("test.db")).unwrap());
        db.add_feed("https://example.com/local.xml").unwrap();
        let service = SyncService::new(Arc::clone(&db));
        let mut port = FakeSyncPort::with_pull(SyncPull {
            cursor: Some("remote-1".into()),
            changes: vec![RemoteSyncChange {
                remote_id: "feed-1".into(),
                entity: SyncEntity::Feed,
                operation: SyncOperation::Upsert,
                field: None,
                value: Some("https://example.com/remote.xml".into()),
                url: None,
                folder_remote_id: None,
            }],
        });
        port.fail_next_pull(CoreError::coded(
            ErrorCategory::Network,
            "syncUnavailable",
            None,
        ));

        assert_eq!(
            service
                .sync_once("fake", &mut port)
                .await
                .unwrap_err()
                .code(),
            "syncUnavailable"
        );
        assert_eq!(db.get_sync_cursor("fake:pull").unwrap(), None);
        assert_eq!(db.list_feeds().unwrap().len(), 1);

        assert_eq!(service.sync_once("fake", &mut port).await.unwrap(), 1);
        assert_eq!(
            db.get_sync_cursor("fake:pull").unwrap().as_deref(),
            Some("remote-1")
        );
        assert_eq!(db.list_feeds().unwrap().len(), 2);
    }
}
