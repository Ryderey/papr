//! GitHub Git Data API transport. Credentials stay in memory; errors never include responses.

use super::{
    error,
    model::{is_hex_id, GitHubProfile, Snapshot, MAX_FILE_BYTES, MAX_SNAPSHOT_BYTES, PREFIX},
};
use crate::error::CoreError;
use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Method,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

const MAX_HTTP_JSON: usize = 8 * 1024 * 1024;
const MAX_TREE_REQUEST: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedFile {
    pub sha: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct RemoteSnapshot {
    pub head: String,
    pub tree: String,
    pub snapshot: Option<Snapshot>,
    pub files: BTreeMap<String, CachedFile>,
    pub trusted_now: Option<String>,
    pub repository_size_kib: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Publication {
    Accepted,
    HeadChanged,
}

pub trait SnapshotTransport {
    fn read(
        &mut self,
        cache: &BTreeMap<String, CachedFile>,
    ) -> impl std::future::Future<Output = Result<RemoteSnapshot, CoreError>> + Send;
    fn create_candidate(
        &mut self,
        base: &RemoteSnapshot,
        files: &BTreeMap<String, String>,
    ) -> impl std::future::Future<Output = Result<Option<String>, CoreError>> + Send;
    fn publish(
        &mut self,
        expected: &str,
        candidate: &str,
    ) -> impl std::future::Future<Output = Result<Publication, CoreError>> + Send;
    fn is_descendant(
        &mut self,
        older: &str,
        newer: &str,
    ) -> impl std::future::Future<Output = Result<bool, CoreError>> + Send;
    fn retry_at(&self) -> Option<String>;
}

pub struct GitHubTransport {
    client: Arc<Client>,
    profile: GitHubProfile,
    base: url::Url,
    authorization: HeaderValue,
    blocked_until: Option<tokio::time::Instant>,
    blocked_indefinitely: bool,
    retry_at: Option<String>,
    trusted_now: Option<String>,
    next_write: tokio::time::Instant,
    write_interval: Duration,
}

impl GitHubTransport {
    pub fn new(
        client: Arc<Client>,
        profile: GitHubProfile,
        token: &str,
    ) -> Result<Self, CoreError> {
        profile.validate()?;
        let token = token.trim();
        if token.is_empty() || token.len() > 4096 || !token.bytes().all(|c| (33..=126).contains(&c))
        {
            return Err(error("githubInvalidCredential"));
        }
        let mut authorization = HeaderValue::from_str(&format!("Bearer {token}"))
            .map_err(|_| error("githubInvalidCredential"))?;
        authorization.set_sensitive(true);
        let base = url::Url::parse(&format!(
            "https://api.github.com/repos/{}/{}/",
            profile.owner, profile.repo
        ))
        .map_err(|_| error("githubInvalidProfile"))?;
        Ok(Self {
            client,
            profile,
            base,
            authorization,
            blocked_until: None,
            blocked_indefinitely: false,
            retry_at: None,
            trusted_now: None,
            next_write: tokio::time::Instant::now(),
            write_interval: Duration::from_secs(1),
        })
    }

    pub async fn inspect(
        client: Arc<Client>,
        owner: String,
        repo: String,
        branch: Option<String>,
        credential_ref: String,
        token: &str,
    ) -> Result<GitHubProfile, CoreError> {
        let profile = GitHubProfile {
            repository_id: 1,
            owner,
            repo,
            branch: branch.clone().unwrap_or_else(|| "main".into()),
            credential_ref,
        };
        let mut transport = Self::new(client, profile, token)?;
        let repository: Repository = transport.get(&[], None).await?;
        if !repository.private {
            return Err(error("githubPrivateRepositoryRequired"));
        }
        transport.profile.repository_id = repository.id;
        transport.profile.branch = branch.unwrap_or(repository.default_branch);
        transport.profile.validate()?;
        transport.head().await?;
        Ok(transport.profile)
    }

    fn url(&self, segments: &[&str]) -> Result<url::Url, CoreError> {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| error("githubInvalidProfile"))?
            .pop_if_empty()
            .extend(segments);
        Ok(url)
    }

    fn record_headers(&mut self, headers: &HeaderMap) {
        let server_now = headers
            .get("date")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| DateTime::parse_from_rfc2822(s).ok())
            .map(|d| d.with_timezone(&Utc));
        if let Some(now) = server_now {
            self.trusted_now = Some(now.to_rfc3339());
        }
        let remaining_zero = headers
            .get("x-ratelimit-remaining")
            .and_then(|h| h.to_str().ok())
            == Some("0");
        let retry_after = headers
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let reset = if remaining_zero {
            headers
                .get("x-ratelimit-reset")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<i64>().ok())
                .map(|t| {
                    t.saturating_sub(server_now.unwrap_or_else(Utc::now).timestamp())
                        .max(1) as u64
                })
        } else {
            None
        };
        if let Some(seconds) =
            retry_after
                .or(reset)
                .or(if remaining_zero { Some(60) } else { None })
        {
            let duration = Duration::from_secs(seconds.max(1));
            // Overflow is fail-closed: a valid future delay must never be shortened.
            self.blocked_until = tokio::time::Instant::now().checked_add(duration);
            self.blocked_indefinitely = self.blocked_until.is_none();
            self.retry_at = i64::try_from(seconds)
                .ok()
                .and_then(chrono::Duration::try_seconds)
                .and_then(|d| server_now.unwrap_or_else(Utc::now).checked_add_signed(d))
                .map(|d| d.to_rfc3339());
        }
    }

    async fn request(
        &mut self,
        method: Method,
        segments: &[&str],
        query: Option<(&str, &str)>,
        body: Option<Value>,
        raw: bool,
    ) -> Result<String, CoreError> {
        if self.blocked_indefinitely
            || self
                .blocked_until
                .is_some_and(|t| tokio::time::Instant::now() < t)
        {
            return Err(error("githubRateLimited"));
        }
        let writes = method != Method::GET;
        if writes {
            tokio::time::sleep_until(self.next_write).await;
            self.next_write = tokio::time::Instant::now() + self.write_interval;
        }
        let mut request = self
            .client
            .request(method, self.url(segments)?)
            .header("User-Agent", "Papr/GitHubSyncV1")
            .header("Authorization", self.authorization.clone())
            .header(
                "Accept",
                if raw {
                    "application/vnd.github.raw+json"
                } else {
                    "application/vnd.github+json"
                },
            )
            .header("X-GitHub-Api-Version", "2026-03-10");
        if let Some(pair) = query {
            request = request.query(&[pair]);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let mut response = request.send().await.map_err(|_| error("githubNetwork"))?;
        self.record_headers(response.headers());
        if !response.status().is_success() {
            return Err(error(match response.status().as_u16() {
                401 => "githubAuthenticationFailed",
                403 if self.blocked_indefinitely
                    || self
                        .blocked_until
                        .is_some_and(|t| tokio::time::Instant::now() < t) =>
                {
                    "githubRateLimited"
                }
                403 => "githubPermissionDenied",
                404 => "githubRepositoryUnavailable",
                409 | 422 => "githubWriteRejected",
                429 => {
                    if self.blocked_until.is_none() {
                        self.blocked_until =
                            Some(tokio::time::Instant::now() + Duration::from_secs(60));
                        self.retry_at =
                            Some((Utc::now() + chrono::Duration::seconds(60)).to_rfc3339());
                    }
                    "githubRateLimited"
                }
                500..=599 => "githubNetwork",
                _ => "githubInvalidResponse",
            }));
        }
        let limit = if raw { MAX_FILE_BYTES } else { MAX_HTTP_JSON };
        if response.content_length().is_some_and(|n| n > limit as u64) {
            return Err(error("githubCapacityExceeded"));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| error("githubNetwork"))? {
            if bytes.len() + chunk.len() > limit {
                return Err(error("githubCapacityExceeded"));
            }
            bytes.extend_from_slice(&chunk);
        }
        String::from_utf8(bytes).map_err(|_| error("githubInvalidResponse"))
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &mut self,
        segments: &[&str],
        query: Option<(&str, &str)>,
    ) -> Result<T, CoreError> {
        let text = self
            .request(Method::GET, segments, query, None, false)
            .await?;
        serde_json::from_str(&text).map_err(|_| error("githubInvalidResponse"))
    }
    async fn post<T: serde::de::DeserializeOwned>(
        &mut self,
        segments: &[&str],
        body: Value,
    ) -> Result<T, CoreError> {
        let text = self
            .request(Method::POST, segments, None, Some(body), false)
            .await?;
        serde_json::from_str(&text).map_err(|_| error("githubInvalidResponse"))
    }
    async fn head(&mut self) -> Result<String, CoreError> {
        let branch = self.profile.branch.clone();
        let reference: Reference = self.get(&["git", "ref", "heads", &branch], None).await?;
        if reference.reference != format!("refs/heads/{branch}")
            || reference.object.kind != "commit"
            || !is_hex_id(&reference.object.sha, 40)
        {
            return Err(error("githubInvalidResponse"));
        }
        Ok(reference.object.sha)
    }
}

impl SnapshotTransport for GitHubTransport {
    async fn read(
        &mut self,
        cache: &BTreeMap<String, CachedFile>,
    ) -> Result<RemoteSnapshot, CoreError> {
        let repository: Repository = self.get(&[], None).await?;
        if repository.id != self.profile.repository_id {
            return Err(error("githubRepositoryChanged"));
        }
        if !repository.private {
            return Err(error("githubPrivateRepositoryRequired"));
        }
        let head = self.head().await?;
        let commit: Commit = self.get(&["git", "commits", &head], None).await?;
        if !is_hex_id(&commit.tree.sha, 40) {
            return Err(error("githubInvalidResponse"));
        }
        let tree: Tree = self
            .get(
                &["git", "trees", &commit.tree.sha],
                Some(("recursive", "1")),
            )
            .await?;
        if tree.truncated {
            return Err(error("githubTruncatedTree"));
        }
        let mut files = BTreeMap::new();
        let mut total = 0usize;
        let has_namespace = tree
            .tree
            .iter()
            .any(|entry| entry.path.starts_with("papr-sync/"));
        for entry in tree.tree {
            if !entry.path.starts_with(&format!("{PREFIX}/")) {
                continue;
            }
            if entry.kind == "tree" {
                continue;
            }
            if entry.kind != "blob" || entry.mode != "100644" || !is_hex_id(&entry.sha, 40) {
                return Err(error("githubInvalidSnapshot"));
            }
            if entry.size.is_some_and(|n| n > MAX_FILE_BYTES as u64) {
                return Err(error("githubCapacityExceeded"));
            }
            let content = match cache.get(&entry.path).filter(|file| file.sha == entry.sha) {
                Some(file) => file.content.clone(),
                None => {
                    self.request(Method::GET, &["git", "blobs", &entry.sha], None, None, true)
                        .await?
                }
            };
            total = total
                .checked_add(content.len())
                .ok_or_else(|| error("githubCapacityExceeded"))?;
            if total > MAX_SNAPSHOT_BYTES
                || files.len() >= 130
                || files
                    .insert(
                        entry.path,
                        CachedFile {
                            sha: entry.sha,
                            content,
                        },
                    )
                    .is_some()
            {
                return Err(error("githubCapacityExceeded"));
            }
        }
        let snapshot = if files.is_empty() {
            if has_namespace {
                return Err(error("githubUnsupportedFormat"));
            }
            None
        } else {
            Some(Snapshot::from_files(
                &files
                    .iter()
                    .map(|(p, f)| (p.clone(), f.content.clone()))
                    .collect(),
            )?)
        };
        Ok(RemoteSnapshot {
            head,
            tree: commit.tree.sha,
            snapshot,
            files,
            trusted_now: self.trusted_now.clone(),
            repository_size_kib: repository.size,
        })
    }

    async fn create_candidate(
        &mut self,
        base: &RemoteSnapshot,
        files: &BTreeMap<String, String>,
    ) -> Result<Option<String>, CoreError> {
        Snapshot::from_files(files)?;
        if !is_hex_id(&base.head, 40) || !is_hex_id(&base.tree, 40) {
            return Err(error("githubInvalidResponse"));
        }
        let paths: BTreeSet<_> = files.keys().chain(base.files.keys()).collect();
        let mut changes = Vec::new();
        for path in paths {
            if !path.starts_with(&format!("{PREFIX}/")) {
                return Err(error("githubInvalidSnapshot"));
            }
            if base.files.get(path).map(|f| &f.content) == files.get(path) {
                continue;
            }
            changes.push(match files.get(path) {
                Some(content) => {
                    json!({"path":path,"mode":"100644","type":"blob","content":content})
                }
                None => json!({"path":path,"mode":"100644","type":"blob","sha":null}),
            });
        }
        if changes.is_empty() {
            return Ok(None);
        }
        let mut body = json!({"base_tree":base.tree,"tree":changes});
        if serde_json::to_vec(&body)
            .map_err(|_| error("githubInvalidSnapshot"))?
            .len()
            > MAX_TREE_REQUEST
        {
            for entry in body["tree"]
                .as_array_mut()
                .ok_or_else(|| error("githubInvalidSnapshot"))?
            {
                if let Some(content) = entry
                    .get("content")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                {
                    let blob: Object = self
                        .post(
                            &["git", "blobs"],
                            json!({"content":content,"encoding":"utf-8"}),
                        )
                        .await?;
                    if !is_hex_id(&blob.sha, 40) {
                        return Err(error("githubInvalidResponse"));
                    }
                    entry
                        .as_object_mut()
                        .ok_or_else(|| error("githubInvalidSnapshot"))?
                        .remove("content");
                    entry["sha"] = json!(blob.sha);
                }
            }
        }
        let tree: Object = self.post(&["git", "trees"], body).await?;
        if !is_hex_id(&tree.sha, 40) {
            return Err(error("githubInvalidResponse"));
        }
        let commit: Object = self
            .post(
                &["git", "commits"],
                json!({"message":"Papr sync","tree":tree.sha,"parents":[base.head]}),
            )
            .await?;
        if !is_hex_id(&commit.sha, 40) {
            return Err(error("githubInvalidResponse"));
        }
        Ok(Some(commit.sha))
    }

    async fn publish(&mut self, expected: &str, candidate: &str) -> Result<Publication, CoreError> {
        if !is_hex_id(expected, 40) || !is_hex_id(candidate, 40) {
            return Err(error("githubInvalidResponse"));
        }
        let branch = self.profile.branch.clone();
        match self
            .request(
                Method::PATCH,
                &["git", "refs", "heads", &branch],
                None,
                Some(json!({"sha":candidate,"force":false})),
                false,
            )
            .await
        {
            Ok(text) => {
                let reference: Reference =
                    serde_json::from_str(&text).map_err(|_| error("githubInvalidResponse"))?;
                if reference.reference != format!("refs/heads/{branch}")
                    || reference.object.sha != candidate
                    || reference.object.kind != "commit"
                {
                    return Err(error("githubInvalidResponse"));
                }
                Ok(Publication::Accepted)
            }
            Err(err) if err.code() == "githubWriteRejected" => {
                if self.head().await? != expected {
                    Ok(Publication::HeadChanged)
                } else {
                    Err(err)
                }
            }
            Err(err) => Err(err),
        }
    }
    async fn is_descendant(&mut self, older: &str, newer: &str) -> Result<bool, CoreError> {
        if !is_hex_id(older, 40) || !is_hex_id(newer, 40) {
            return Err(error("githubInvalidResponse"));
        }
        if older == newer {
            return Ok(true);
        }
        let comparison: Comparison = self
            .get(&["compare", &format!("{older}...{newer}")], None)
            .await?;
        match comparison.status.as_str() {
            "ahead" | "identical" => Ok(true),
            "behind" | "diverged" => Ok(false),
            _ => Err(error("githubInvalidResponse")),
        }
    }
    fn retry_at(&self) -> Option<String> {
        self.retry_at.clone()
    }
}

#[derive(Deserialize)]
struct Repository {
    id: u64,
    private: bool,
    default_branch: String,
    size: Option<u64>,
}
#[derive(Deserialize)]
struct Object {
    sha: String,
}
#[derive(Deserialize)]
struct RefObject {
    sha: String,
    #[serde(rename = "type")]
    kind: String,
}
#[derive(Deserialize)]
struct Reference {
    #[serde(rename = "ref")]
    reference: String,
    object: RefObject,
}
#[derive(Deserialize)]
struct Commit {
    tree: Object,
}
#[derive(Deserialize)]
struct Tree {
    truncated: bool,
    tree: Vec<TreeEntry>,
}
#[derive(Deserialize)]
struct TreeEntry {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    kind: String,
    sha: String,
    size: Option<u64>,
}
#[derive(Deserialize)]
struct Comparison {
    status: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // Read the complete request before responding, including large JSON bodies.
    async fn fixture(
        responses: Vec<(u16, String, String)>,
    ) -> (GitHubTransport, tokio::task::JoinHandle<Vec<String>>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let mut requests = Vec::new();
            for (status, headers, body) in responses {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                loop {
                    let mut buffer = [0; 4096];
                    let count = stream.read(&mut buffer).await.unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&buffer[..count]);
                    if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&bytes[..end]);
                        let length = head
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .and_then(|v| v.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if bytes.len() >= end + 4 + length {
                            break;
                        }
                    }
                }
                requests.push(String::from_utf8(bytes).unwrap());
                stream.write_all(format!("HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}", body.len()).as_bytes()).await.unwrap();
            }
            requests
        });
        let mut transport = GitHubTransport::new(
            Arc::new(Client::new()),
            GitHubProfile {
                repository_id: 7,
                owner: "owner".into(),
                repo: "repo".into(),
                branch: "main".into(),
                credential_ref: "papr.sync.test".into(),
            },
            "test-secret-never-returned",
        )
        .unwrap();
        transport.base = url::Url::parse(&format!("http://{address}/repos/owner/repo/")).unwrap();
        transport.write_interval = Duration::ZERO;
        (transport, task)
    }
    fn response(body: Value) -> (u16, String, String) {
        (200, String::new(), body.to_string())
    }
    fn reference(sha: &str) -> Value {
        json!({"ref":"refs/heads/main","object":{"type":"commit","sha":sha}})
    }

    #[tokio::test]
    async fn publication_preserves_tree_parent_and_never_forces_ref() {
        let old = "a".repeat(40);
        let tree = "b".repeat(40);
        let candidate = "c".repeat(40);
        let (mut transport, task) = fixture(vec![
            response(json!({"sha":tree})),
            response(json!({"sha":candidate})),
            response(reference(&candidate)),
        ])
        .await;
        let snapshot = Snapshot::empty("d".repeat(32), "2026-10-05T00:00:00Z".into());
        let base = RemoteSnapshot {
            head: old.clone(),
            tree: "e".repeat(40),
            snapshot: None,
            files: BTreeMap::new(),
            trusted_now: None,
            repository_size_kib: None,
        };
        assert_eq!(
            transport
                .create_candidate(&base, &snapshot.files().unwrap())
                .await
                .unwrap(),
            Some(candidate.clone())
        );
        assert_eq!(
            transport.publish(&old, &candidate).await.unwrap(),
            Publication::Accepted
        );
        let requests = task.await.unwrap();
        let body = |index: usize| {
            serde_json::from_str::<Value>(requests[index].split("\r\n\r\n").nth(1).unwrap())
                .unwrap()
        };
        assert_eq!(body(0)["base_tree"], base.tree);
        assert_eq!(body(1)["parents"], json!([old]));
        assert_eq!(body(2)["force"], false);
        assert!(requests[2].starts_with("PATCH /repos/owner/repo/git/refs/heads/main "));
    }

    #[tokio::test]
    async fn rejected_ref_rebases_only_when_head_changed() {
        for changed in [false, true] {
            let old = "a".repeat(40);
            let next = if changed { "b".repeat(40) } else { old.clone() };
            let (mut transport, task) = fixture(vec![
                (422, String::new(), "sensitive-response".into()),
                response(reference(&next)),
            ])
            .await;
            let result = transport.publish(&old, &"c".repeat(40)).await;
            if changed {
                assert_eq!(result.unwrap(), Publication::HeadChanged);
            } else {
                assert_eq!(result.unwrap_err().code(), "githubWriteRejected");
            }
            assert_eq!(task.await.unwrap().len(), 2);
        }
    }

    #[tokio::test]
    async fn truncated_tree_and_repository_identity_abort_before_blobs() {
        let head = "a".repeat(40);
        let tree = "b".repeat(40);
        let (mut transport, task) = fixture(vec![
            response(json!({"id":7,"private":true,"default_branch":"main","size":1})),
            response(reference(&head)),
            response(json!({"tree":{"sha":tree}})),
            response(json!({"truncated":true,"tree":[]})),
        ])
        .await;
        assert_eq!(
            transport.read(&BTreeMap::new()).await.unwrap_err().code(),
            "githubTruncatedTree"
        );
        assert_eq!(task.await.unwrap().len(), 4);
        let (mut transport, task) = fixture(vec![response(
            json!({"id":8,"private":true,"default_branch":"main"}),
        )])
        .await;
        assert_eq!(
            transport.read(&BTreeMap::new()).await.unwrap_err().code(),
            "githubRepositoryChanged"
        );
        assert_eq!(task.await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn rate_limit_does_not_retry_or_expose_credential_or_response() {
        let (mut transport, task) = fixture(vec![(
            403,
            "Retry-After: 120\r\nDate: Mon, 05 Oct 2026 00:00:00 GMT\r\n".into(),
            "test-secret-never-returned".into(),
        )])
        .await;
        let first = transport.get::<Value>(&[], None).await.unwrap_err();
        assert_eq!(first.code(), "githubRateLimited");
        assert_eq!(first.detail(), None);
        assert_eq!(
            transport.retry_at(),
            Some("2026-10-05T00:02:00+00:00".into())
        );
        assert_eq!(
            transport.get::<Value>(&[], None).await.unwrap_err().code(),
            "githubRateLimited"
        );
        assert_eq!(task.await.unwrap().len(), 1);
    }
}
