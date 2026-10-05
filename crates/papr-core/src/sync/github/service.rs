//! Shared orchestration. Network awaits never hold a SQLite writer lock.
use super::{
    error,
    merge::{merge, merge_with_confirmed_times, Intent, Operation},
    model::*,
    storage,
    transport::{CachedFile, GitHubTransport, Publication, RemoteSnapshot, SnapshotTransport},
};
use crate::{db::Db, error::CoreError};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub trait LocalStore: Clone + Send + Sync {
    fn with<T: Send + 'static>(
        &self,
        action: impl FnOnce(&Connection) -> Result<T, CoreError> + Send + 'static,
    ) -> impl Future<Output = Result<T, CoreError>> + Send;
}
#[derive(Clone)]
pub struct CoreStore(pub Arc<Db>);
impl LocalStore for CoreStore {
    async fn with<T: Send + 'static>(
        &self,
        action: impl FnOnce(&Connection) -> Result<T, CoreError> + Send + 'static,
    ) -> Result<T, CoreError> {
        let db = Arc::clone(&self.0);
        tokio::task::spawn_blocking(move || db.transact(|tx| action(tx)))
            .await
            .map_err(|_| error("githubLocalTaskFailed"))?
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    pub profile: GitHubProfile,
    pub head: String,
    pub local_feeds: usize,
    pub remote_feeds: usize,
    pub local_articles: usize,
    pub remote_articles: usize,
    pub excluded_feeds: usize,
    pub excluded_articles: usize,
    pub warning_count: usize,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Status {
    pub profile: Option<GitHubProfile>,
    pub pending: usize,
    pub rejected: usize,
    pub metadata_only: usize,
    pub last_success_at: Option<String>,
    pub last_error_code: Option<String>,
    pub retry_at: Option<String>,
    pub busy: bool,
    pub uncertain_publication: bool,
    pub background_due: bool,
    pub automatic_due: bool,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncReport {
    pub acknowledged: usize,
    pub rejected: usize,
    pub retries: usize,
    pub pending: usize,
}

#[derive(Clone, Default)]
pub struct Cancellation {
    flag: Arc<AtomicBool>,
    wake: Arc<tokio::sync::Notify>,
}
impl Cancellation {
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
        self.wake.notify_one();
    }
    fn check(&self) -> Result<(), CoreError> {
        if self.flag.load(Ordering::Relaxed) {
            Err(error("githubSyncCancelled"))
        } else {
            Ok(())
        }
    }
    async fn wait<F: Future>(&self, future: F) -> Result<F::Output, CoreError> {
        self.check()?;
        tokio::select! {biased;_ = self.wake.notified()=>Err(error("githubSyncCancelled")),output=future=>Ok(output)}
    }
}

pub struct GitHubService<S: LocalStore> {
    store: S,
    client: Arc<reqwest::Client>,
}
impl<S: LocalStore> GitHubService<S> {
    pub fn new(store: S, client: Arc<reqwest::Client>) -> Self {
        Self { store, client }
    }
    pub async fn status(&self) -> Result<Status, CoreError> {
        self.store.with(|conn|{
        let Some(info)=storage::connection(conn)? else{return Ok(Status::default())};
        let (last_success_at,last_error_code,retry_at,busy,background_due)=conn.query_row("SELECT last_success_at,last_error_code,retry_at,lease_until IS NOT NULL AND julianday(lease_until)>julianday('now'),last_success_at IS NULL OR julianday(last_success_at)<julianday('now','-6 hours') OR EXISTS(SELECT 1 FROM github_outbox WHERE connection_id=github_connections.id) FROM github_connections WHERE id=?1",[info.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).map_err(sql)?;
        let count=|table:&str|conn.query_row(&format!("SELECT COUNT(*) FROM {table} WHERE connection_id=?1"),[info.id],|r|r.get::<_,usize>(0)).map_err(sql);
        let ready: bool=conn.query_row("SELECT (last_error_code IS NULL OR last_error_code IN ('githubNetwork','githubRateLimited','githubSyncCancelled','githubConcurrentRetryLimit','githubSyncBusy')) AND (retry_at IS NULL OR julianday(retry_at)<=julianday('now')) AND (lease_until IS NULL OR julianday(lease_until)<=julianday('now')) FROM github_connections WHERE id=?1",[info.id],|r|r.get(0)).map_err(sql)?;
        let automatic_due=conn.query_row("SELECT (last_error_code IS NULL OR last_error_code IN ('githubNetwork','githubRateLimited','githubSyncCancelled','githubConcurrentRetryLimit','githubSyncBusy')) AND (retry_at IS NULL OR julianday(retry_at)<=julianday('now')) AND (lease_until IS NULL OR julianday(lease_until)<=julianday('now')) AND ((pending_since IS NOT NULL AND (julianday(last_edit_at)<=julianday('now','-10 seconds') OR julianday(pending_since)<=julianday('now','-60 seconds')) AND (last_publish_at IS NULL OR julianday(last_publish_at)<=julianday('now','-60 seconds'))) OR last_success_at IS NULL OR julianday(last_success_at)<=julianday('now','-5 minutes')) FROM github_connections WHERE id=?1",[info.id],|r|r.get(0)).map_err(sql)?;
        Ok(Status{profile:Some(info.profile),pending:count("github_outbox")?,rejected:count("github_rejections")?,metadata_only:conn.query_row("SELECT COUNT(*) FROM articles WHERE metadata_only=1",[],|r|r.get(0)).map_err(sql)?,last_success_at,last_error_code,retry_at,busy,uncertain_publication:count("github_attempt")?>0,background_due:background_due && ready,automatic_due})
    }).await
    }

    pub async fn preview(
        &self,
        owner: String,
        repo: String,
        branch: Option<String>,
        credential_ref: String,
        token: &str,
    ) -> Result<Preview, CoreError> {
        self.store.with(storage::ensure_backend_available).await?;
        let profile = GitHubTransport::inspect(
            Arc::clone(&self.client),
            owner,
            repo,
            branch,
            credential_ref,
            token,
        )
        .await?;
        let mut transport = GitHubTransport::new(Arc::clone(&self.client), profile.clone(), token)?;
        let remote = transport.read(&BTreeMap::new()).await?;
        let (data, excluded_feeds, excluded_articles) = self
            .store
            .with(|conn| {
                let data = storage::initial_data(conn)?;
                let feeds = conn
                    .query_row("SELECT COUNT(*) FROM feeds", [], |r| r.get::<_, usize>(0))
                    .map_err(sql)?;
                let articles = conn
                    .query_row("SELECT COUNT(*) FROM articles", [], |r| {
                        r.get::<_, usize>(0)
                    })
                    .map_err(sql)?;
                let excluded_feeds = feeds.saturating_sub(data.subscriptions.feeds.len());
                let excluded_articles = articles.saturating_sub(data.articles.len());
                Ok((data, excluded_feeds, excluded_articles))
            })
            .await?;
        let mut preview = Preview {
            profile,
            head: remote.head.clone(),
            local_feeds: data.subscriptions.feeds.len(),
            local_articles: data.articles.len(),
            remote_feeds: 0,
            remote_articles: 0,
            excluded_feeds,
            excluded_articles,
            warning_count: 0,
        };
        let base = remote.snapshot.clone().unwrap_or_else(|| {
            Snapshot::empty(
                "d".repeat(32),
                cutoff(remote.trusted_now.as_deref()).unwrap_or_else(epoch),
            )
        });
        preview.remote_feeds = base
            .subscriptions
            .feeds
            .values()
            .filter(|f| f.active)
            .count();
        preview.remote_articles = base.articles.len();
        let op = Operation {
            dataset_id: base.manifest.dataset_id.clone(),
            device_id: "e".repeat(32),
            seq: base
                .manifest
                .devices
                .get(&"e".repeat(32))
                .map_or(1, |d| d.processed_seq + 1),
            intent: Intent::SeedInitial { data },
        };
        let report = merge(
            &base,
            &[op],
            cutoff(remote.trusted_now.as_deref()).as_deref(),
        )?;
        report.snapshot.files()?;
        preview.warning_count = report.rejected.len();
        Ok(preview)
    }

    pub async fn connect(
        &self,
        preview: Preview,
        token: &str,
        binding: String,
    ) -> Result<(), CoreError> {
        let mut transport =
            GitHubTransport::new(Arc::clone(&self.client), preview.profile.clone(), token)?;
        let remote = transport.read(&BTreeMap::new()).await?;
        if remote.head != preview.head {
            return Err(error("githubPreviewChanged"));
        }
        self.store
            .with(move |conn| {
                storage::connect(conn, &preview.profile, &remote, &binding).map(|_| ())
            })
            .await
    }
    pub async fn disconnect(&self) -> Result<Option<String>, CoreError> {
        self.store.with(storage::disconnect).await
    }
    pub async fn checkpoint(&self, previous: Option<String>) -> Result<String, CoreError> {
        self.store
            .with(move |conn| storage::checkpoint(conn, previous.as_deref()))
            .await
    }
    pub fn record_platform_error(
        &self,
        code: &'static str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send + '_ {
        self.store.with(move|conn|{conn.execute("UPDATE github_connections SET last_error_code=?1,retry_at=COALESCE(retry_at,datetime('now','+60 seconds')) WHERE active=1 AND lease IS NULL",[code]).map_err(sql)?;Ok(())})
    }

    pub async fn verify_credential(&self, token: &str) -> Result<(), CoreError> {
        let info = self
            .store
            .with(storage::connection)
            .await?
            .ok_or_else(|| error("githubNotConnected"))?;
        let mut transport = GitHubTransport::new(Arc::clone(&self.client), info.profile, token)?;
        let remote = transport.read(&BTreeMap::new()).await?;
        self.store
            .with(move |conn| {
                storage::validate_credential_snapshot(conn, info.id, remote.snapshot.as_ref())
            })
            .await
    }

    /// Call only after verification and successful secure-store replacement.
    /// Credential repair cannot clear an unrelated history/identity failure.
    pub async fn credential_updated(&self, credential_ref: String) -> Result<(), CoreError> {
        self.store.with(move |conn| {
            let info = storage::connection(conn)?.ok_or_else(|| error("githubNotConnected"))?;
            if info.profile.credential_ref != credential_ref {
                return Err(error("githubConnectionChanged"));
            }
            conn.execute("UPDATE github_connections SET last_error_code=NULL,retry_at=NULL WHERE id=?1 AND last_error_code IN ('githubAuthenticationFailed','githubPermissionDenied','githubInvalidCredential','githubRepositoryUnavailable','syncCredentialMissing','credentialReadFailed','credentialWriteFailed')",[info.id]).map_err(sql)?;
            Ok(())
        }).await
    }

    pub fn sync(
        &self,
        token: String,
        binding: String,
        cancel: Cancellation,
    ) -> impl Future<Output = Result<SyncReport, CoreError>> + Send + '_ {
        async move {
            let (info, lease) = self
                .store
                .with(move |conn| storage::acquire(conn, &binding))
                .await?;
            let mut transport = match GitHubTransport::new(
                Arc::clone(&self.client),
                info.profile.clone(),
                &token,
            ) {
                Ok(transport) => transport,
                Err(err) => {
                    let id = info.id;
                    let code = err.code();
                    self.store
                        .with(move |conn| storage::fail(conn, id, &lease, code, None))
                        .await?;
                    return Err(err);
                }
            };
            let result = run(&self.store, &mut transport, &info, &lease, &cancel).await;
            let id = info.id;
            let code: Option<&'static str> = match &result {
                Err(error) => Some(error.code()),
                Ok(_) => None,
            };
            let retry = transport.retry_at();
            self.store
                .with(move |conn| match code {
                    Some(code) => storage::fail(conn, id, &lease, code, retry.as_deref()),
                    None => storage::release(conn, id, &lease),
                })
                .await?;
            result
        }
    }
}
fn sql(error: rusqlite::Error) -> CoreError {
    CoreError::Db(error.to_string())
}
fn epoch() -> String {
    "1970-01-01T00:00:00Z".into()
}
fn cutoff(now: Option<&str>) -> Option<String> {
    now.and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| (d.with_timezone(&Utc) - chrono::Duration::days(90)).to_rfc3339())
}

async fn run<S: LocalStore, T: SnapshotTransport>(
    store: &S,
    transport: &mut T,
    info: &storage::ConnectionInfo,
    lease: &str,
    cancel: &Cancellation,
) -> Result<SyncReport, CoreError> {
    let mut info = info.clone();
    let id = info.id;
    let cache = store.with(move |conn| storage::files(conn, id)).await?;
    let batch_info = info.clone();
    let mut batch = store
        .with(move |conn| storage::operations(conn, &batch_info, batch_info.ack_seq, BATCH_SIZE))
        .await?;
    let max_seq = batch.last().map_or(info.ack_seq, |o| o.seq);
    for retry in 0..3 {
        cancel.check()?;
        let remote = cancel.wait(transport.read(&cache)).await??;
        if let Some(head) = &info.head {
            if !cancel
                .wait(transport.is_descendant(head, &remote.head))
                .await??
            {
                return Err(error("githubHistoryRewritten"));
            }
        }
        let snapshot = match remote.snapshot.as_ref() {
            Some(snapshot) => snapshot.clone(),
            None => {
                if info.ack_seq > 0 || !cache.is_empty() {
                    return Err(error("githubDatasetMissing"));
                }
                Snapshot::empty(
                    info.dataset_id.clone(),
                    cutoff(remote.trusted_now.as_deref()).unwrap_or_else(epoch),
                )
            }
        };
        if remote.snapshot.is_some() && info.initializing {
            let lease_owned = lease.to_string();
            let known = snapshot.clone();
            info = store
                .with(move |conn| storage::adopt_initial_dataset(conn, id, &lease_owned, &known))
                .await?;
            for operation in &mut batch {
                operation.dataset_id = info.dataset_id.clone();
            }
        }
        if snapshot.manifest.dataset_id != info.dataset_id || snapshot.manifest.epoch != info.epoch
        {
            return Err(error("githubDatasetChanged"));
        }
        let ack = snapshot
            .manifest
            .devices
            .get(&info.device_id)
            .map_or(0, |d| d.processed_seq);
        let current = store
            .with(storage::connection)
            .await?
            .filter(|c| c.id == info.id)
            .ok_or_else(|| error("githubConnectionChanged"))?;
        if ack < info.ack_seq || ack >= current.next_seq {
            return Err(error("githubRestoredDatabase"));
        }
        let lease_owned = lease.to_string();
        let receipt = snapshot.clone();
        let confirmed_times = store
            .with(move |conn| {
                storage::save_rejections(conn, id, &lease_owned, &receipt)?;
                storage::confirmed_times(conn, id)
            })
            .await?;
        let report = merge_with_confirmed_times(
            &snapshot,
            &batch,
            cutoff(remote.trusted_now.as_deref()).as_deref(),
            &confirmed_times,
        )?;
        let files = report.snapshot.files()?;
        let publication_due = store
            .with(move |conn| storage::publish_due(conn, id))
            .await?;
        if publication_due {
            let lease_owned = lease.to_string();
            let head = remote.head.clone();
            store
                .with(move |conn| {
                    storage::record_attempt(conn, id, &lease_owned, &head, max_seq, None)
                })
                .await?;
        }
        cancel.check()?;
        let candidate = if publication_due {
            cancel
                .wait(transport.create_candidate(&remote, &files))
                .await??
        } else {
            None
        };
        let confirmed = if let Some(candidate) = candidate {
            let head = remote.head.clone();
            let lease_owned = lease.to_string();
            let candidate_owned = candidate.clone();
            store
                .with(move |conn| {
                    storage::record_attempt(
                        conn,
                        id,
                        &lease_owned,
                        &head,
                        max_seq,
                        Some(&candidate_owned),
                    )
                })
                .await?;
            cancel.check()?;
            if cancel
                .wait(transport.publish(&remote.head, &candidate))
                .await??
                == Publication::HeadChanged
            {
                let lease_owned = lease.to_string();
                store
                    .with(move |conn| storage::rejected_publication(conn, id, &lease_owned))
                    .await?;
                continue;
            }
            RemoteSnapshot {
                head: candidate,
                tree: remote.tree,
                snapshot: Some(report.snapshot),
                files: files
                    .into_iter()
                    .map(|(path, content)| {
                        let sha = remote
                            .files
                            .get(&path)
                            .filter(|old| old.content == content)
                            .map(|old| old.sha.clone())
                            .unwrap_or_default();
                        (path, CachedFile { sha, content })
                    })
                    .collect(),
                trusted_now: remote.trusted_now,
                repository_size_kib: remote.repository_size_kib,
            }
        } else {
            remote
        };
        let lease_owned = lease.to_string();
        let confirmed_owned = confirmed.clone();
        store
            .with(move |conn| storage::finish(conn, id, &lease_owned, &confirmed_owned))
            .await?;
        let device = info.device_id.clone();
        let new_ack = confirmed
            .snapshot
            .as_ref()
            .and_then(|s| s.manifest.devices.get(&device))
            .map_or(info.ack_seq, |d| d.processed_seq);
        let pending_info = info.clone();
        let pending = store
            .with(move |conn| {
                storage::operations(conn, &pending_info, new_ack, i64::MAX as usize)
                    .map(|ops| ops.len())
            })
            .await?;
        return Ok(SyncReport {
            acknowledged: (new_ack - info.ack_seq) as usize,
            rejected: report.rejected.len(),
            retries: retry,
            pending,
        });
    }
    Err(error("githubConcurrentRetryLimit"))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FakeTransport {
        remote: RemoteSnapshot,
        candidate: Option<Snapshot>,
        publications: usize,
        lose_response: bool,
        conflicts: usize,
        descendant: bool,
    }
    impl SnapshotTransport for FakeTransport {
        async fn read(
            &mut self,
            _cache: &BTreeMap<String, CachedFile>,
        ) -> Result<RemoteSnapshot, CoreError> {
            Ok(self.remote.clone())
        }
        async fn create_candidate(
            &mut self,
            _base: &RemoteSnapshot,
            files: &BTreeMap<String, String>,
        ) -> Result<Option<String>, CoreError> {
            if self
                .remote
                .files
                .iter()
                .map(|(p, f)| (p.clone(), f.content.clone()))
                .collect::<BTreeMap<_, _>>()
                == *files
            {
                return Ok(None);
            }
            self.candidate = Some(Snapshot::from_files(files)?);
            Ok(Some("c".repeat(40)))
        }
        async fn publish(
            &mut self,
            _expected: &str,
            _candidate: &str,
        ) -> Result<Publication, CoreError> {
            self.publications += 1;
            if self.conflicts > 0 {
                self.conflicts -= 1;
                return Ok(Publication::HeadChanged);
            }
            let snapshot = self.candidate.take().unwrap();
            self.remote.head = "c".repeat(40);
            self.remote.files = snapshot
                .files()?
                .into_iter()
                .map(|(p, content)| {
                    (
                        p,
                        CachedFile {
                            sha: "f".repeat(40),
                            content,
                        },
                    )
                })
                .collect();
            self.remote.snapshot = Some(snapshot);
            if self.lose_response {
                self.lose_response = false;
                return Err(error("githubNetwork"));
            }
            Ok(Publication::Accepted)
        }
        async fn is_descendant(&mut self, _old: &str, _new: &str) -> Result<bool, CoreError> {
            Ok(self.descendant)
        }
        fn retry_at(&self) -> Option<String> {
            None
        }
    }
    async fn fixture() -> (tempfile::TempDir, CoreStore, FakeTransport) {
        let dir = tempfile::tempdir().unwrap();
        let store = CoreStore(Arc::new(Db::new(&dir.path().join("papr.db")).unwrap()));
        store
            .with(|conn| {
                conn.execute(
                    "INSERT INTO feeds(feed_url,title) VALUES ('https://example.com/feed','Feed')",
                    [],
                )
                .map_err(sql)?;
                conn.execute(
                    "INSERT INTO articles(feed_id,guid,title) VALUES (1,'guid','Article')",
                    [],
                )
                .map_err(sql)?;
                Ok(())
            })
            .await
            .unwrap();
        let snapshot = Snapshot::empty("d".repeat(32), epoch());
        let remote = RemoteSnapshot {
            head: "a".repeat(40),
            tree: "b".repeat(40),
            files: snapshot
                .files()
                .unwrap()
                .into_iter()
                .map(|(p, content)| {
                    (
                        p,
                        CachedFile {
                            sha: "e".repeat(40),
                            content,
                        },
                    )
                })
                .collect(),
            snapshot: Some(snapshot),
            trusted_now: None,
            repository_size_kib: None,
        };
        let owned = remote.clone();
        store
            .with(move |conn| {
                storage::connect(
                    conn,
                    &GitHubProfile {
                        repository_id: 1,
                        owner: "owner".into(),
                        repo: "repo".into(),
                        branch: "main".into(),
                        credential_ref: "papr.sync.test".into(),
                    },
                    &owned,
                    "binding",
                )
                .map(|_| ())
            })
            .await
            .unwrap();
        (
            dir,
            store,
            FakeTransport {
                remote,
                candidate: None,
                publications: 0,
                lose_response: false,
                conflicts: 0,
                descendant: true,
            },
        )
    }
    async fn attempt(
        store: &CoreStore,
        transport: &mut FakeTransport,
        cancel: Cancellation,
    ) -> Result<SyncReport, CoreError> {
        let (info, lease) = store.with(|conn| storage::acquire(conn, "binding")).await?;
        let result = run(store, transport, &info, &lease, &cancel).await;
        let id = info.id;
        store
            .with(move |conn| storage::release(conn, id, &lease))
            .await
            .unwrap();
        result
    }

    #[tokio::test]
    async fn lost_ref_response_recovers_watermark_without_replaying_after_other_device_edit() {
        let (_dir, store, mut transport) = fixture().await;
        store
            .with(|conn| {
                conn.execute("UPDATE articles SET is_starred=1", [])
                    .map_err(sql)?;
                Ok(())
            })
            .await
            .unwrap();
        transport.lose_response = true;
        assert_eq!(
            attempt(&store, &mut transport, Cancellation::default())
                .await
                .unwrap_err()
                .code(),
            "githubNetwork"
        );
        let mut remote = transport.remote.snapshot.clone().unwrap();
        let entry = remote.articles.values().next().unwrap().clone();
        remote = merge(
            &remote,
            &[Operation {
                dataset_id: remote.manifest.dataset_id.clone(),
                device_id: "f".repeat(32),
                seq: 1,
                intent: Intent::SetArticleState {
                    entry,
                    field: super::super::merge::StateField::Starred,
                    value: false,
                    base_version: None,
                },
            }],
            None,
        )
        .unwrap()
        .snapshot;
        transport.remote.files = remote
            .files()
            .unwrap()
            .into_iter()
            .map(|(p, content)| {
                (
                    p,
                    CachedFile {
                        sha: "f".repeat(40),
                        content,
                    },
                )
            })
            .collect();
        transport.remote.snapshot = Some(remote);
        let report = attempt(&store, &mut transport, Cancellation::default())
            .await
            .unwrap();
        assert_eq!(report.acknowledged, 2);
        assert_eq!(transport.publications, 1);
        assert_eq!(report.pending, 0);
        assert!(!store
            .with(|conn| conn
                .query_row("SELECT is_starred FROM articles WHERE id=1", [], |r| r
                    .get::<_, bool>(0))
                .map_err(sql))
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn unchanged_head_has_no_commit_and_cooldown_keeps_edits_pending() {
        let (_dir, store, mut transport) = fixture().await;
        attempt(&store, &mut transport, Cancellation::default())
            .await
            .unwrap();
        assert_eq!(transport.publications, 1);
        attempt(&store, &mut transport, Cancellation::default())
            .await
            .unwrap();
        assert_eq!(transport.publications, 1);
        store
            .with(|conn| {
                conn.execute("UPDATE articles SET read_later=1", [])
                    .map_err(sql)?;
                Ok(())
            })
            .await
            .unwrap();
        let report = attempt(&store, &mut transport, Cancellation::default())
            .await
            .unwrap();
        assert_eq!(report.pending, 1);
        assert_eq!(transport.publications, 1);
        assert!(store
            .with(|conn| conn
                .query_row("SELECT read_later FROM articles WHERE id=1", [], |r| r
                    .get::<_, bool>(0))
                .map_err(sql))
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn conflicts_are_bounded_and_cancellation_and_rewritten_history_preserve_queue() {
        let (_dir, store, mut transport) = fixture().await;
        transport.conflicts = 5;
        transport.descendant = false;
        assert_eq!(
            attempt(&store, &mut transport, Cancellation::default())
                .await
                .unwrap_err()
                .code(),
            "githubHistoryRewritten"
        );
        assert_eq!(transport.publications, 0);
        transport.descendant = true;
        let cancel = Cancellation::default();
        cancel.cancel();
        assert_eq!(
            attempt(&store, &mut transport, cancel)
                .await
                .unwrap_err()
                .code(),
            "githubSyncCancelled"
        );
        assert_eq!(
            store
                .with(|conn| {
                    let info = storage::connection(conn)?.unwrap();
                    storage::operations(conn, &info, 0, 500).map(|ops| ops.len())
                })
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            attempt(&store, &mut transport, Cancellation::default())
                .await
                .unwrap_err()
                .code(),
            "githubConcurrentRetryLimit"
        );
        assert_eq!(transport.publications, 3);
    }
    #[tokio::test]
    async fn cancellation_interrupts_a_pending_network_future() {
        let cancel = Cancellation::default();
        let trigger = cancel.clone();
        let (result, ()) = tokio::join!(
            tokio::time::timeout(
                std::time::Duration::from_secs(1),
                cancel.wait(std::future::pending::<()>())
            ),
            async move {
                tokio::task::yield_now().await;
                trigger.cancel();
            }
        );
        assert_eq!(
            result
                .expect("cancellation must wake a pending request")
                .unwrap_err()
                .code(),
            "githubSyncCancelled"
        );
    }

    #[tokio::test]
    async fn core_business_writes_capture_rules_bulk_read_and_resubscription() {
        let (_dir, store, mut transport) = fixture().await;
        store.with(|conn|{conn.execute("INSERT INTO rules(name,field,query,action) VALUES ('Save','title','New','star')",[]).map_err(sql)?;Ok(())}).await.unwrap();
        let article = crate::dto::NewArticle {
            guid: "new".into(),
            url: Some("https://example.com/new".into()),
            title: "New article".into(),
            author: None,
            summary: None,
            content_html: Some("<p>local-only body</p>".into()),
            body_text: "local-only body".into(),
            image_url: None,
            published_at: None,
            enclosures: Vec::new(),
        };
        assert!(store.0.upsert_article(1, &article).unwrap());
        assert_eq!(
            store
                .0
                .mark_all_read(&crate::dto::ArticleFilter::default())
                .unwrap(),
            2
        );
        attempt(&store, &mut transport, Cancellation::default())
            .await
            .unwrap();
        let snapshot = transport.remote.snapshot.as_ref().unwrap();
        let key = article_key(
            &feed_key("https://example.com/feed").unwrap(),
            IdentityKind::Guid,
            "new",
        );
        assert!(snapshot.states[&key].starred.value && snapshot.states[&key].read.value);
        store.0.delete_feed(1).unwrap();
        assert!(store.0.list_feeds().unwrap().is_empty());
        assert_eq!(store.0.article_counts().unwrap().all, 1);
        assert_eq!(
            store
                .0
                .insert_feed(
                    "https://example.com/feed",
                    None,
                    "Feed",
                    None,
                    crate::dto::SourceType::Rss,
                    None
                )
                .unwrap(),
            1
        );
        let ops = store
            .with(|conn| {
                let info = storage::connection(conn)?.unwrap();
                storage::operations(conn, &info, info.ack_seq, 500)
            })
            .await
            .unwrap();
        assert!(ops
            .iter()
            .any(|op| matches!(op.intent, Intent::Unsubscribe { generation: 1, .. })));
        assert!(ops.iter().any(|op| matches!(
            op.intent,
            Intent::Resubscribe {
                observed_generation: 1,
                ..
            }
        )));
        let merged = merge(snapshot, &ops, None).unwrap();
        assert!(merged.rejected.is_empty());
        assert_eq!(
            merged
                .snapshot
                .subscriptions
                .feeds
                .values()
                .next()
                .unwrap()
                .generation,
            2
        );
        assert!(
            merged
                .snapshot
                .subscriptions
                .feeds
                .values()
                .next()
                .unwrap()
                .active
        );
    }

    #[tokio::test]
    async fn competing_initializers_adopt_winner_without_losing_checkpoint_or_edits() {
        let (_a_dir, a, mut transport) = fixture().await;
        let (_b_dir, b, _) = fixture().await;
        transport.remote.snapshot = None;
        transport.remote.files.clear();
        for (store, second) in [(&a, false), (&b, true)] {
            let empty = transport.remote.clone();
            store
                .with(move |conn| {
                    let old = storage::connection(conn)?.unwrap();
                    storage::disconnect(conn)?;
                    if second {
                        conn.execute(
                            "UPDATE articles SET guid='second',title='Second',is_starred=1",
                            [],
                        )
                        .map_err(sql)?;
                    }
                    storage::connect(conn, &old.profile, &empty, "binding")?;
                    if second {
                        conn.execute("UPDATE articles SET read_later=1", [])
                            .map_err(sql)?;
                    }
                    Ok(())
                })
                .await
                .unwrap();
        }
        let before = b
            .with(|conn| storage::checkpoint(conn, None))
            .await
            .unwrap();
        let old_dataset = b
            .with(|conn| Ok(storage::connection(conn)?.unwrap().dataset_id))
            .await
            .unwrap();
        attempt(&a, &mut transport, Cancellation::default())
            .await
            .unwrap();
        let winner = transport
            .remote
            .snapshot
            .as_ref()
            .unwrap()
            .manifest
            .dataset_id
            .clone();
        assert_ne!(old_dataset, winner);
        let report = attempt(&b, &mut transport, Cancellation::default())
            .await
            .unwrap();
        assert_eq!(report.pending, 0);
        assert_eq!(report.acknowledged, 2);
        assert_eq!(
            transport.remote.snapshot.as_ref().unwrap().articles.len(),
            2
        );
        let before_copy = before.clone();
        let after = b
            .with(move |conn| storage::checkpoint(conn, Some(&before_copy)))
            .await
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(
            b.with(|conn| Ok(storage::connection(conn)?.unwrap().dataset_id))
                .await
                .unwrap(),
            winner
        );
        let state = transport
            .remote
            .snapshot
            .as_ref()
            .unwrap()
            .states
            .values()
            .find(|s| s.read_later.value)
            .unwrap();
        assert!(state.starred.value);
        // A confirmed connection never adopts a replacement dataset.
        transport
            .remote
            .snapshot
            .as_mut()
            .unwrap()
            .manifest
            .dataset_id = "f".repeat(32);
        assert_eq!(
            attempt(&b, &mut transport, Cancellation::default())
                .await
                .unwrap_err()
                .code(),
            "githubDatasetChanged"
        );
    }

    #[tokio::test]
    async fn lost_ref_receipt_is_saved_before_new_suffix_replaces_it() {
        let (_dir, store, mut transport) = fixture().await;
        store
            .with(|conn| {
                let info = storage::connection(conn)?.unwrap();
                let payload = serde_json::to_string(&Intent::RenameFolder {
                    folder_id: "a".repeat(32),
                    name: "Missing".into(),
                    base_version: None,
                })
                .unwrap();
                conn.execute(
                    "INSERT INTO github_outbox(connection_id,seq,payload) VALUES (?1,?2,?3)",
                    rusqlite::params![info.id, info.next_seq, payload],
                )
                .map_err(sql)?;
                Ok(())
            })
            .await
            .unwrap();
        transport.lose_response = true;
        assert_eq!(
            attempt(&store, &mut transport, Cancellation::default())
                .await
                .unwrap_err()
                .code(),
            "githubNetwork"
        );
        let device = store
            .with(|conn| Ok(storage::connection(conn)?.unwrap().device_id))
            .await
            .unwrap();
        assert_eq!(
            transport.remote.snapshot.as_ref().unwrap().manifest.devices[&device]
                .last_rejections
                .len(),
            1
        );
        store
            .with(|conn| {
                conn.execute("UPDATE articles SET is_starred=1", [])
                    .map_err(sql)?;
                conn.execute(
                    "UPDATE github_connections SET last_publish_at=datetime('now','-61 seconds')",
                    [],
                )
                .map_err(sql)?;
                Ok(())
            })
            .await
            .unwrap();
        attempt(&store, &mut transport, Cancellation::default())
            .await
            .unwrap();
        assert!(
            transport.remote.snapshot.as_ref().unwrap().manifest.devices[&device]
                .last_rejections
                .is_empty()
        );
        assert_eq!(
            store
                .with(|conn| conn
                    .query_row(
                        "SELECT COUNT(*) FROM github_rejections WHERE seq=2",
                        [],
                        |r| r.get::<_, usize>(0)
                    )
                    .map_err(sql))
                .await
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn credential_repair_resumes_both_schedulers_but_preserves_identity_errors() {
        let (_dir, store, _) = fixture().await;
        let service = GitHubService::new(store.clone(), Arc::new(reqwest::Client::new()));
        store.with(|conn| {conn.execute("UPDATE github_connections SET last_error_code='credentialReadFailed',retry_at=datetime('now','+10 minutes')",[]).map_err(sql)?;Ok(())}).await.unwrap();
        let status = service.status().await.unwrap();
        assert!(!status.automatic_due && !status.background_due);
        service
            .credential_updated("papr.sync.test".into())
            .await
            .unwrap();
        let status = service.status().await.unwrap();
        assert!(status.automatic_due && status.background_due);
        assert!(status.last_error_code.is_none() && status.retry_at.is_none());
        store.with(|conn| {conn.execute("UPDATE github_connections SET last_error_code='githubRepositoryUnavailable',retry_at=datetime('now','+10 minutes')",[]).map_err(sql)?;Ok(())}).await.unwrap();
        let remote = store
            .with(|conn| {
                let info = storage::connection(conn)?.unwrap();
                Ok(Snapshot::empty(
                    info.dataset_id,
                    "1970-01-01T00:00:00Z".into(),
                ))
            })
            .await
            .unwrap();
        let remote_owned = remote.clone();
        store
            .with(move |conn| {
                let info = storage::connection(conn)?.unwrap();
                storage::validate_credential_snapshot(conn, info.id, Some(&remote_owned))
            })
            .await
            .unwrap();
        service
            .credential_updated("papr.sync.test".into())
            .await
            .unwrap();
        let status = service.status().await.unwrap();
        assert!(status.automatic_due && status.background_due);
        assert!(status.last_error_code.is_none() && status.retry_at.is_none());
        store
            .with(|conn| {
                conn.execute(
                    "UPDATE github_connections SET last_error_code='githubHistoryRewritten'",
                    [],
                )
                .map_err(sql)?;
                Ok(())
            })
            .await
            .unwrap();
        service
            .credential_updated("papr.sync.test".into())
            .await
            .unwrap();
        let status = service.status().await.unwrap();
        assert!(!status.automatic_due && !status.background_due);
        assert_eq!(
            status.last_error_code.as_deref(),
            Some("githubHistoryRewritten")
        );
        assert_eq!(
            service
                .credential_updated("papr.sync.other".into())
                .await
                .unwrap_err()
                .code(),
            "githubConnectionChanged"
        );
    }
    #[tokio::test]
    async fn expired_article_restoration_uses_durable_confirmation_history() {
        let (_dir, store, mut transport) = fixture().await;
        store
            .with(|conn| {
                conn.execute(
                    "UPDATE articles SET fetched_at='1900-01-01 00:00:00',body_text='cached body'",
                    [],
                )
                .map_err(sql)?;
                Ok(())
            })
            .await
            .unwrap();
        transport.remote.trusted_now = Some("2026-01-01T00:00:00Z".into());
        attempt(&store, &mut transport, Cancellation::default())
            .await
            .unwrap();
        let key = transport
            .remote
            .snapshot
            .as_ref()
            .unwrap()
            .articles
            .keys()
            .next()
            .unwrap()
            .clone();
        let confirmed = transport.remote.snapshot.as_ref().unwrap().articles[&key]
            .first_seen_at
            .clone();
        for (date, starred, should_exist) in [
            ("2026-04-02T00:00:00Z", None, false),
            ("2026-04-03T00:00:00Z", Some(true), true),
            ("2026-04-04T00:00:00Z", Some(false), false),
        ] {
            store
                .with(move |conn| {
                    conn.execute("UPDATE github_connections SET last_publish_at=NULL", [])
                        .map_err(sql)?;
                    if let Some(starred) = starred {
                        conn.execute("UPDATE articles SET is_starred=?1", [starred])
                            .map_err(sql)?;
                    }
                    Ok(())
                })
                .await
                .unwrap();
            transport.remote.trusted_now = Some(date.into());
            attempt(&store, &mut transport, Cancellation::default())
                .await
                .unwrap();
            let cloud = transport.remote.snapshot.as_ref().unwrap();
            assert_eq!(cloud.articles.contains_key(&key), should_exist);
            if should_exist {
                assert_eq!(cloud.articles[&key].first_seen_at, confirmed);
            }
        }
        let key_owned = key.clone();
        let (age, body) = store
            .with(move |conn| {
                let info = storage::connection(conn)?.unwrap();
                let times = storage::confirmed_times(conn, info.id)?;
                Ok((
                    times[&key_owned].clone(),
                    conn.query_row("SELECT body_text FROM articles WHERE id=1", [], |r| {
                        r.get::<_, String>(0)
                    })
                    .map_err(sql)?,
                ))
            })
            .await
            .unwrap();
        assert_eq!(age, confirmed);
        assert_eq!(body, "cached body");
    }
}
