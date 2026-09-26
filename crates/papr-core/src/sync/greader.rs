//! FreshRSS and Miniflux adapter for the shared Google Reader API subset.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::dto::{SyncChange, SyncEntity, SyncOperation};
use crate::error::{CoreError, ErrorCategory};
use crate::sync::{RemoteSyncChange, SyncPort, SyncProfile, SyncProvider, SyncPull};

const READ_TAG: &str = "user/-/state/com.google/read";
const STARRED_TAG: &str = "user/-/state/com.google/starred";
const READING_LIST: &str = "user/-/state/com.google/reading-list";
const ITEMS_PER_PAGE: usize = 500;
const MAX_ITEMS: usize = 20_000;
const MAX_PAGES: usize = 100;

struct Session {
    auth: String,
    edit_token: String,
}

/// A transient authenticated connection. The password and auth tokens are
/// never returned to FRB, persisted, logged, or included in errors.
pub struct GReaderSyncPort {
    http: Arc<Client>,
    base: String,
    username: String,
    password: String,
    session: Option<Session>,
}

impl GReaderSyncPort {
    pub fn new(
        profile: &SyncProfile,
        password: String,
        http: Arc<Client>,
    ) -> Result<Self, CoreError> {
        profile.validate()?;
        if password.trim().is_empty() || password.len() > 8192 {
            return Err(sync_error("syncCredentialMissing"));
        }
        Ok(Self {
            http,
            base: greader_base(profile),
            username: profile.username.clone(),
            password,
            session: None,
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/reader/api/0/{path}", self.base)
    }

    fn authenticated_get(&self, path: &str) -> Result<RequestBuilder, CoreError> {
        let auth = &self
            .session
            .as_ref()
            .ok_or_else(|| sync_error("syncAuthFailed"))?
            .auth;
        Ok(self
            .http
            .get(self.endpoint(path))
            .header("Authorization", format!("GoogleLogin auth={auth}")))
    }

    fn authenticated_post(&self, path: &str) -> Result<RequestBuilder, CoreError> {
        let auth = &self
            .session
            .as_ref()
            .ok_or_else(|| sync_error("syncAuthFailed"))?
            .auth;
        Ok(self
            .http
            .post(self.endpoint(path))
            .header("Authorization", format!("GoogleLogin auth={auth}")))
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, CoreError> {
        let response = checked(self.authenticated_get(path)?.send().await).await?;
        response
            .json()
            .await
            .map_err(|_| sync_error("syncInvalidResponse"))
    }

    async fn post_form(&self, path: &str, form: &[(&str, &str)]) -> Result<Response, CoreError> {
        checked(self.authenticated_post(path)?.form(form).send().await).await
    }

    async fn subscriptions(&self) -> Result<SubscriptionList, CoreError> {
        self.get_json("subscription/list?output=json").await
    }

    async fn pull_items(&self) -> Result<Vec<RemoteSyncChange>, CoreError> {
        let mut changes = Vec::new();
        let mut continuation: Option<String> = None;
        let mut seen = HashSet::new();
        let mut fetched = 0;
        let mut pages = 0;
        loop {
            pages += 1;
            if pages > MAX_PAGES {
                return Err(sync_error("syncTooManyItems"));
            }
            let mut request = self.authenticated_get("stream/items/ids")?.query(&[
                ("output", "json"),
                ("s", READING_LIST),
                ("n", "500"),
            ]);
            if let Some(cursor) = continuation.as_deref() {
                request = request.query(&[("c", cursor)]);
            }
            let page: ItemIds = checked(request.send().await)
                .await?
                .json()
                .await
                .map_err(|_| sync_error("syncInvalidResponse"))?;
            fetched += page.item_refs.len();
            if page.item_refs.len() > ITEMS_PER_PAGE || fetched > MAX_ITEMS {
                // ponytail: bounded full snapshot; switch to a provider delta
                // strategy if large accounts need more than 20k items.
                return Err(sync_error("syncTooManyItems"));
            }
            let edit_token = self
                .session
                .as_ref()
                .ok_or_else(|| sync_error("syncAuthFailed"))?
                .edit_token
                .as_str();
            for chunk in page.item_refs.chunks(100) {
                let mut form = vec![("T", edit_token), ("output", "json")];
                form.extend(chunk.iter().map(|item| ("i", item.id.as_str())));
                let contents: ItemContents = self
                    .post_form("stream/items/contents", &form)
                    .await?
                    .json()
                    .await
                    .map_err(|_| sync_error("syncInvalidResponse"))?;
                for item in contents.items {
                    let Some(url) = item
                        .canonical
                        .first()
                        .or_else(|| item.alternate.first())
                        .map(|href| href.href.clone())
                    else {
                        continue;
                    };
                    for (field, tag) in [
                        ("read", "/state/com.google/read"),
                        ("starred", "/state/com.google/starred"),
                    ] {
                        changes.push(RemoteSyncChange {
                            remote_id: item.id.clone(),
                            entity: SyncEntity::Article,
                            operation: SyncOperation::Upsert,
                            field: Some(field.to_string()),
                            value: Some(
                                if item
                                    .categories
                                    .iter()
                                    .any(|category| category.ends_with(tag))
                                {
                                    "1"
                                } else {
                                    "0"
                                }
                                .to_string(),
                            ),
                            url: Some(url.clone()),
                            folder_remote_id: None,
                        });
                    }
                }
            }
            match page.continuation.filter(|cursor| !cursor.is_empty()) {
                Some(cursor) if seen.insert(cursor.clone()) => continuation = Some(cursor),
                Some(_) => return Err(sync_error("syncInvalidResponse")),
                None => break,
            }
        }
        Ok(changes)
    }
}

impl SyncPort for GReaderSyncPort {
    async fn validate(&mut self) -> Result<(), CoreError> {
        if self.session.is_some() {
            return Ok(());
        }
        let response = checked(
            self.http
                .post(format!("{}/accounts/ClientLogin", self.base))
                .form(&[
                    ("Email", self.username.as_str()),
                    ("Passwd", self.password.as_str()),
                ])
                .send()
                .await,
        )
        .await?;
        let body = response
            .text()
            .await
            .map_err(|_| sync_error("syncInvalidResponse"))?;
        let auth = body
            .lines()
            .find_map(|line| line.strip_prefix("Auth="))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| sync_error("syncAuthFailed"))?
            .to_string();
        let response = checked(
            self.http
                .get(self.endpoint("token"))
                .header("Authorization", format!("GoogleLogin auth={auth}"))
                .send()
                .await,
        )
        .await?;
        let edit_token = response
            .text()
            .await
            .map_err(|_| sync_error("syncInvalidResponse"))?
            .trim()
            .to_string();
        if edit_token.is_empty() {
            return Err(sync_error("syncInvalidResponse"));
        }
        self.session = Some(Session { auth, edit_token });
        Ok(())
    }

    async fn push(&mut self, changes: &[SyncChange]) -> Result<Vec<i64>, CoreError> {
        self.validate().await?;
        let mut acknowledged = Vec::new();
        let mut remote_feeds: Option<HashMap<String, (Option<String>, bool)>> = None;
        for change in changes {
            if change.operation == SyncOperation::Tombstone || change.entity == SyncEntity::Folder {
                // GReader has no safe standalone-folder or confirmed-delete
                // operation in this release. Keep both sides' content.
                acknowledged.push(change.sequence);
                continue;
            }
            let edit_token = self
                .session
                .as_ref()
                .ok_or_else(|| sync_error("syncAuthFailed"))?
                .edit_token
                .as_str();
            match change.entity {
                SyncEntity::Feed => {
                    let Some(url) = change.url.as_deref() else {
                        acknowledged.push(change.sequence); // feed was deleted locally
                        continue;
                    };
                    if !url.starts_with("http://") && !url.starts_with("https://") {
                        acknowledged.push(change.sequence); // local-only source
                        continue;
                    }
                    if remote_feeds.is_none() {
                        remote_feeds = Some(
                            self.subscriptions()
                                .await?
                                .subscriptions
                                .into_iter()
                                .filter_map(|sub| {
                                    let labeled = sub
                                        .categories
                                        .iter()
                                        .any(|category| category.name().is_some());
                                    sub.url.map(|url| (url, (Some(sub.id), labeled)))
                                })
                                .collect(),
                        );
                    }
                    let feeds = remote_feeds.as_mut().expect("loaded above");
                    if !feeds.contains_key(url) {
                        let stream = format!("feed/{url}");
                        let label = change
                            .folder
                            .as_deref()
                            .map(|name| format!("user/-/label/{name}"));
                        let mut form = vec![
                            ("T", edit_token),
                            ("ac", "subscribe"),
                            ("s", stream.as_str()),
                        ];
                        if let Some(label) = label.as_deref() {
                            form.push(("a", label));
                        }
                        self.post_form("subscription/edit", &form).await?;
                        feeds.insert(url.to_string(), (None, label.is_some()));
                    } else if let (Some(folder), Some((Some(remote_id), false))) =
                        (change.folder.as_deref(), feeds.get(url))
                    {
                        // Assign an uncategorized remote feed without removing
                        // a category chosen independently on the provider.
                        let label = format!("user/-/label/{folder}");
                        self.post_form(
                            "subscription/edit",
                            &[
                                ("T", edit_token),
                                ("ac", "edit"),
                                ("s", remote_id),
                                ("a", label.as_str()),
                            ],
                        )
                        .await?;
                        if let Some((_, labeled)) = feeds.get_mut(url) {
                            *labeled = true;
                        }
                    }
                    acknowledged.push(change.sequence);
                }
                SyncEntity::Article => {
                    let Some(remote_id) = change.remote_id.as_deref() else {
                        continue; // pull first to establish the remote ID
                    };
                    let tag = match change.field.as_deref() {
                        Some("read") => READ_TAG,
                        Some("starred") => STARRED_TAG,
                        _ => return Err(sync_error("invalidSyncChange")),
                    };
                    let action = match change.value.as_deref() {
                        Some("1") => "a",
                        Some("0") => "r",
                        _ => return Err(sync_error("invalidSyncChange")),
                    };
                    self.post_form(
                        "edit-tag",
                        &[("T", edit_token), ("i", remote_id), (action, tag)],
                    )
                    .await?;
                    acknowledged.push(change.sequence);
                }
                SyncEntity::Folder => unreachable!(),
            }
        }
        Ok(acknowledged)
    }

    async fn pull(&mut self, _cursor: Option<&str>) -> Result<SyncPull, CoreError> {
        self.validate().await?;
        let mut changes = Vec::new();
        for subscription in self.subscriptions().await?.subscriptions {
            let Some(url) = subscription.url.filter(|url| !url.is_empty()) else {
                continue;
            };
            let folder = subscription.categories.iter().find_map(Category::name);
            if let Some((id, name)) = folder.as_ref() {
                changes.push(RemoteSyncChange {
                    remote_id: id.clone(),
                    entity: SyncEntity::Folder,
                    operation: SyncOperation::Upsert,
                    field: None,
                    value: Some(name.clone()),
                    url: None,
                    folder_remote_id: None,
                });
            }
            changes.push(RemoteSyncChange {
                remote_id: subscription.id,
                entity: SyncEntity::Feed,
                operation: SyncOperation::Upsert,
                field: None,
                value: Some(url),
                url: None,
                folder_remote_id: folder.map(|(id, _)| id),
            });
        }
        changes.extend(self.pull_items().await?);
        // GReader exposes a snapshot, not a durable remote change cursor.
        Ok(SyncPull {
            cursor: None,
            changes,
        })
    }

    async fn acknowledge(&mut self, _cursor: &str) -> Result<(), CoreError> {
        Ok(())
    }
}

fn greader_base(profile: &SyncProfile) -> String {
    let server = profile.server_url.trim().trim_end_matches('/');
    match profile.provider {
        SyncProvider::FreshRss if !server.ends_with("/api/greader.php") => {
            format!("{server}/api/greader.php")
        }
        _ => server.to_string(),
    }
}

async fn checked(response: Result<Response, reqwest::Error>) -> Result<Response, CoreError> {
    let response = response.map_err(|_| sync_error("syncUnavailable"))?;
    match response.status() {
        status if status.is_success() => Ok(response),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(sync_error("syncAuthFailed")),
        _ => Err(sync_error("syncProviderFailed")),
    }
}

fn sync_error(code: &'static str) -> CoreError {
    CoreError::coded(ErrorCategory::Sync, code, None)
}

#[derive(Deserialize)]
struct SubscriptionList {
    subscriptions: Vec<Subscription>,
}

#[derive(Deserialize)]
struct Subscription {
    id: String,
    url: Option<String>,
    #[serde(default)]
    categories: Vec<Category>,
}

#[derive(Deserialize)]
struct Category {
    id: String,
    label: Option<String>,
}

impl Category {
    fn name(&self) -> Option<(String, String)> {
        if !self.id.contains("/label/") {
            return None;
        }
        let name = self
            .label
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .or_else(|| self.id.rsplit_once("/label/").map(|(_, name)| name))?
            .trim();
        if name.is_empty() || name.eq_ignore_ascii_case("Uncategorized") {
            None
        } else {
            Some((self.id.clone(), name.to_string()))
        }
    }
}

#[derive(Deserialize)]
struct ItemIds {
    #[serde(rename = "itemRefs")]
    item_refs: Vec<ItemRef>,
    continuation: Option<String>,
}

#[derive(Deserialize)]
struct ItemRef {
    id: String,
}

#[derive(Deserialize)]
struct ItemContents {
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    id: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    canonical: Vec<Href>,
    #[serde(default)]
    alternate: Vec<Href>,
}

#[derive(Deserialize)]
struct Href {
    href: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn serve_responses(
        responses: Vec<(&'static str, &'static str)>,
    ) -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let mut requests = Vec::new();
            for (status, body) in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                loop {
                    let mut chunk = [0_u8; 4096];
                    let read = socket.read(&mut chunk).await.unwrap();
                    assert!(read > 0);
                    bytes.extend_from_slice(&chunk[..read]);
                    let Some(header_end) =
                        bytes.windows(4).position(|window| window == b"\r\n\r\n")
                    else {
                        continue;
                    };
                    let header = String::from_utf8_lossy(&bytes[..header_end]);
                    let content_length = header
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length: ")
                                .and_then(|value| value.parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= header_end + 4 + content_length {
                        break;
                    }
                }
                requests.push(String::from_utf8(bytes).unwrap());
                let response = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                socket.write_all(response.as_bytes()).await.unwrap();
            }
            requests
        });
        (base, server)
    }

    fn profile(provider: SyncProvider, server_url: &str) -> SyncProfile {
        SyncProfile {
            provider,
            server_url: server_url.to_string(),
            username: "reader".into(),
            credential_ref: "papr.sync.test".into(),
        }
    }

    #[test]
    fn provider_roots_preserve_subpaths_without_double_appending() {
        assert_eq!(
            greader_base(&profile(SyncProvider::FreshRss, "https://x.example/p")),
            "https://x.example/p/api/greader.php"
        );
        assert_eq!(
            greader_base(&profile(
                SyncProvider::FreshRss,
                "https://x.example/p/api/greader.php/"
            )),
            "https://x.example/p/api/greader.php"
        );
        assert_eq!(
            greader_base(&profile(SyncProvider::Miniflux, "https://x.example/p/")),
            "https://x.example/p"
        );
    }

    #[test]
    fn category_skips_the_provider_uncategorized_label() {
        assert!(Category {
            id: "user/-/label/Uncategorized".into(),
            label: None
        }
        .name()
        .is_none());
        assert!(Category {
            id: "user/1/state/com.google/read".into(),
            label: Some("Read".into()),
        }
        .name()
        .is_none());
        assert_eq!(
            Category {
                id: "user/1/label/Tech".into(),
                label: Some("Tech".into())
            }
            .name(),
            Some(("user/1/label/Tech".into(), "Tech".into()))
        );
    }

    #[tokio::test]
    async fn existing_uncategorized_subscription_gets_a_local_folder() {
        let (base, server) = serve_responses(vec![
            ("200 OK", "SID=reader\nAuth=opaque-auth\n"),
            ("200 OK", "edit-token"),
            (
                "200 OK",
                r#"{"subscriptions":[{"id":"feed/42","url":"https://example.org/feed.xml","categories":[]}]}"#,
            ),
            ("200 OK", "OK"),
        ])
        .await;
        let mut port = GReaderSyncPort::new(
            &profile(SyncProvider::Miniflux, &base),
            "private-password".into(),
            Arc::new(Client::new()),
        )
        .unwrap();
        let acknowledged = port
            .push(&[SyncChange {
                sequence: 1,
                entity: SyncEntity::Feed,
                local_id: 1,
                operation: SyncOperation::Upsert,
                field: None,
                value: None,
                remote_id: None,
                url: Some("https://example.org/feed.xml".into()),
                folder: Some("Tech".into()),
            }])
            .await
            .unwrap();
        assert_eq!(acknowledged, vec![1]);
        let requests = server.await.unwrap();
        assert!(requests[3].starts_with("POST /reader/api/0/subscription/edit "));
        assert!(requests[3].contains("ac=edit"));
        assert!(requests[3].contains("s=feed%2F42"));
        assert!(requests[3].contains("a=user%2F-%2Flabel%2FTech"));
    }

    #[tokio::test]
    async fn freshrss_uses_its_greader_root_for_auth_and_pull() {
        let (base, server) = serve_responses(vec![
            ("200 OK", "SID=reader\nAuth=opaque-auth\n"),
            ("200 OK", "edit-token"),
            ("200 OK", r#"{"subscriptions":[]}"#),
            ("200 OK", r#"{"itemRefs":[]}"#),
        ])
        .await;
        let mut port = GReaderSyncPort::new(
            &profile(SyncProvider::FreshRss, &base),
            "private-password".into(),
            Arc::new(Client::new()),
        )
        .unwrap();
        assert!(port.pull(None).await.unwrap().changes.is_empty());
        let requests = server.await.unwrap();
        assert!(requests[0].starts_with("POST /api/greader.php/accounts/ClientLogin "));
        assert!(requests[1].starts_with("GET /api/greader.php/reader/api/0/token "));
        assert!(requests[2].starts_with("GET /api/greader.php/reader/api/0/subscription/list?"));
        assert!(requests[3].starts_with("GET /api/greader.php/reader/api/0/stream/items/ids?"));
    }

    #[tokio::test]
    async fn miniflux_round_trip_uses_shared_greader_endpoints_without_persisting_tokens() {
        let (base, server) = serve_responses(vec![
            ("200 OK", "SID=reader\nAuth=opaque-auth\n"),
            ("200 OK", "edit-token"),
            ("200 OK", r#"{"subscriptions":[]}"#),
            ("200 OK", "OK"),
            ("200 OK", "OK"),
            ("200 OK", r#"{"subscriptions":[{"id":"feed/42","url":"https://example.org/feed.xml","categories":[{"id":"user/1/label/Tech","label":"Tech"}]}]}"#),
            ("200 OK", r#"{"itemRefs":[{"id":"7"}]}"#),
            ("200 OK", r#"{"items":[{"id":"remote-article","alternate":[{"href":"https://example.org/a"}],"categories":["user/1/state/com.google/read","user/1/state/com.google/starred"]}]}"#),
        ]).await;
        let http = Arc::new(Client::new());
        let mut port = GReaderSyncPort::new(
            &profile(SyncProvider::Miniflux, &base),
            "private-password".into(),
            http,
        )
        .unwrap();
        port.validate().await.unwrap();
        let changes = vec![
            SyncChange {
                sequence: 1,
                entity: SyncEntity::Feed,
                local_id: 1,
                operation: SyncOperation::Upsert,
                field: None,
                value: None,
                remote_id: None,
                url: Some("https://example.org/feed.xml".into()),
                folder: Some("Tech".into()),
            },
            SyncChange {
                sequence: 2,
                entity: SyncEntity::Article,
                local_id: 2,
                operation: SyncOperation::Upsert,
                field: Some("read".into()),
                value: Some("1".into()),
                remote_id: Some("remote-article".into()),
                url: Some("https://example.org/a".into()),
                folder: None,
            },
        ];
        assert_eq!(port.push(&changes).await.unwrap(), vec![1, 2]);
        let pull = port.pull(None).await.unwrap();
        assert_eq!(pull.changes.len(), 4);
        assert_eq!(pull.changes[0].entity, SyncEntity::Folder);
        assert_eq!(pull.changes[1].entity, SyncEntity::Feed);
        assert_eq!(pull.changes[2].value.as_deref(), Some("1"));
        assert_eq!(pull.changes[3].field.as_deref(), Some("starred"));

        let requests = server.await.unwrap();
        let paths = requests
            .iter()
            .map(|request| request.lines().next().unwrap())
            .collect::<Vec<_>>();
        assert!(paths[0].starts_with("POST /accounts/ClientLogin "));
        assert!(paths[1].starts_with("GET /reader/api/0/token "));
        assert!(paths[3].starts_with("POST /reader/api/0/subscription/edit "));
        assert!(paths[4].starts_with("POST /reader/api/0/edit-tag "));
        assert!(paths[6].starts_with("GET /reader/api/0/stream/items/ids?"));
        assert!(paths[7].starts_with("POST /reader/api/0/stream/items/contents "));
        assert!(requests[3].contains("a=user%2F-%2Flabel%2FTech"));
        assert!(requests[4].contains("T=edit-token"));
    }

    #[tokio::test]
    async fn authentication_failure_exposes_only_a_stable_code() {
        let (base, server) =
            serve_responses(vec![("401 Unauthorized", "password=private-password")]).await;
        let mut port = GReaderSyncPort::new(
            &profile(SyncProvider::Miniflux, &base),
            "private-password".into(),
            Arc::new(Client::new()),
        )
        .unwrap();
        let error = port.validate().await.unwrap_err();
        assert_eq!(error.code(), "syncAuthFailed");
        assert!(!error.to_string().contains("private-password"));
        server.await.unwrap();
    }
}
