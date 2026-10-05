//! Shared SQL contracts used by both platform writers. Trigger capture commits
//! with the business statement, including bulk updates and rule-driven flags.
use super::transport::{CachedFile, RemoteSnapshot};
use super::{
    error,
    merge::{InitialData, Intent, Operation},
    model::*,
};
use crate::error::CoreError;
use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{functions::FunctionFlags, params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: i64,
    pub profile: GitHubProfile,
    pub dataset_id: String,
    pub device_id: String,
    pub epoch: u64,
    pub next_seq: u64,
    pub ack_seq: u64,
    pub head: Option<String>,
    pub binding: String,
}

pub fn connection(conn: &Connection) -> Result<Option<ConnectionInfo>, CoreError> {
    let row = conn.query_row("SELECT id,profile,dataset_id,device_id,epoch,next_seq,ack_seq,head,binding FROM github_connections WHERE active=1", [], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,u64>(4)?,r.get::<_,u64>(5)?,r.get::<_,u64>(6)?,r.get::<_,Option<String>>(7)?,r.get::<_,String>(8)?))).optional().map_err(db)?;
    row.map(
        |(id, profile, dataset_id, device_id, epoch, next_seq, ack_seq, head, binding)| {
            let profile: GitHubProfile =
                serde_json::from_str(&profile).map_err(|_| error("githubInvalidProfile"))?;
            profile.validate()?;
            Ok(ConnectionInfo {
                id,
                profile,
                dataset_id,
                device_id,
                epoch,
                next_seq,
                ack_seq,
                head,
                binding,
            })
        },
    )
    .transpose()
}

pub fn checkpoint(conn: &Connection, previous: Option<&str>) -> Result<String, CoreError> {
    let info = connection(conn)?.ok_or_else(|| error("githubNotConnected"))?;
    let current = info.next_seq.saturating_sub(1);
    match previous {
        Some(previous) => {
            let parts: Vec<_> = previous.split(':').collect();
            if parts.len() != 3
                || parts[0] != info.dataset_id
                || parts[1] != info.device_id
                || parts[2].parse::<u64>().map_or(true, |seq| seq > current)
            {
                return Err(error("githubRestoredDatabase"));
            }
        }
        None if info.ack_seq > 0 => return Err(error("githubRestoredDatabase")),
        None => {}
    }
    Ok(format!(
        "{}:{}:{}",
        info.dataset_id, info.device_id, current
    ))
}

pub fn ensure_backend_available(conn: &Connection) -> Result<(), CoreError> {
    let other: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM settings WHERE key IN ('sync_profile','freshrss_auth') AND trim(value) NOT IN ('','null'))", [], |r| r.get(0)).map_err(db)?;
    if other {
        Err(error("githubOtherBackendConnected"))
    } else {
        Ok(())
    }
}

pub fn initial_data(conn: &Connection) -> Result<InitialData, CoreError> {
    // Assign existing folder IDs before the first connection is activated.
    conn.execute(
        "UPDATE folders SET sync_id=lower(hex(randomblob(16))) WHERE sync_id IS NULL",
        [],
    )
    .map_err(db)?;
    let mut data = InitialData::default();
    let mut folders = conn
        .prepare("SELECT sync_id,name FROM folders ORDER BY position,name")
        .map_err(db)?;
    for row in folders
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(db)?
    {
        let (id, name) = row.map_err(db)?;
        data.subscriptions.folder_order.push(id.clone());
        data.subscriptions.folders.insert(
            id.clone(),
            Folder {
                folder_id: id,
                name,
                deleted: false,
                name_version: None,
            },
        );
    }
    let mut feeds = conn
        .prepare("SELECT subscription FROM github_feed_capture")
        .map_err(db)?;
    for row in feeds.query_map([], |r| r.get::<_, String>(0)).map_err(db)? {
        let mut subscription: Subscription =
            serde_json::from_str(&row.map_err(db)?).map_err(|_| error("githubInvalidSource"))?;
        subscription.generation = 1;
        data.subscriptions
            .feeds
            .insert(subscription.feed_key.clone(), subscription);
    }
    let mut articles = conn.prepare("SELECT c.entry,a.is_read,a.is_starred,a.read_later FROM github_article_capture c JOIN articles a ON a.id=c.id WHERE c.entry IS NOT NULL").map_err(db)?;
    for row in articles
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, bool>(1)?,
                r.get::<_, bool>(2)?,
                r.get::<_, bool>(3)?,
            ))
        })
        .map_err(db)?
    {
        let (entry, read, starred, later) = row.map_err(db)?;
        let entry: CatalogEntry =
            serde_json::from_str(&entry).map_err(|_| error("githubInvalidArticle"))?;
        let key = entry.key();
        data.articles.insert(key.clone(), entry);
        data.states.insert(
            key,
            ArticleState {
                read: Flag {
                    value: read,
                    version: None,
                },
                starred: Flag {
                    value: starred,
                    version: None,
                },
                read_later: Flag {
                    value: later,
                    version: None,
                },
            },
        );
    }
    Ok(data)
}

/// Called inside the adapter's writer transaction after the user confirms preview.
pub fn connect(
    conn: &Connection,
    profile: &GitHubProfile,
    remote: &RemoteSnapshot,
    binding: &str,
) -> Result<ConnectionInfo, CoreError> {
    profile.validate()?;
    ensure_backend_available(conn)?;
    if connection(conn)?.is_some() {
        return Err(error("githubAlreadyConnected"));
    }
    if binding.is_empty() || binding.len() > 256 {
        return Err(error("githubInvalidBinding"));
    }
    let data = initial_data(conn)?;
    let dataset_id = remote
        .snapshot
        .as_ref()
        .map(|s| s.manifest.dataset_id.clone())
        .unwrap_or(random_id(conn)?);
    let epoch = remote.snapshot.as_ref().map_or(1, |s| s.manifest.epoch);
    let device_id = random_id(conn)?;
    conn.execute("INSERT INTO github_connections(profile,dataset_id,device_id,epoch,head,binding) VALUES (?1,?2,?3,?4,?5,?6)",params![serde_json::to_string(profile).map_err(|_|error("githubInvalidProfile"))?,dataset_id,device_id,epoch,remote.head,binding]).map_err(db)?;
    let info = connection(conn)?.ok_or_else(|| error("githubNotConnected"))?;
    save_files(conn, info.id, &remote.files)?;
    conn.execute(
        "INSERT INTO github_outbox(connection_id,seq,payload) VALUES (?1,1,?2)",
        params![
            info.id,
            serde_json::to_string(&Intent::SeedInitial { data })
                .map_err(|_| error("githubInvalidOperation"))?
        ],
    )
    .map_err(db)?;
    // Seed maps preserve local body caches when the first confirmed projection arrives.
    conn.execute(
        "INSERT INTO github_entity_map SELECT ?1,'folder',sync_id,id,NULL FROM folders",
        [info.id],
    )
    .map_err(db)?;
    conn.execute("INSERT INTO github_entity_map SELECT ?1,'feed',entity_key,id,NULL FROM github_feed_capture",[info.id]).map_err(db)?;
    conn.execute("INSERT INTO github_entity_map SELECT ?1,'article',papr_article_key(entry),id,entry FROM github_article_capture WHERE entry IS NOT NULL",[info.id]).map_err(db)?;
    connection(conn)?.ok_or_else(|| error("githubNotConnected"))
}

pub fn disconnect(conn: &Connection) -> Result<Option<String>, CoreError> {
    let info = connection(conn)?;
    if let Some(info) = &info {
        let busy:bool=conn.query_row("SELECT lease_until IS NOT NULL AND julianday(lease_until)>julianday('now') FROM github_connections WHERE id=?1",[info.id],|r|r.get(0)).map_err(db)?;
        if busy {
            return Err(error("githubSyncBusy"));
        }
        conn.execute(
            "UPDATE github_connections SET active=0,lease=NULL,lease_until=NULL WHERE id=?1",
            [info.id],
        )
        .map_err(db)?;
    }
    Ok(info.map(|i| i.profile.credential_ref))
}

pub fn files(conn: &Connection, id: i64) -> Result<BTreeMap<String, CachedFile>, CoreError> {
    let mut statement = conn
        .prepare("SELECT path,sha,content FROM github_files WHERE connection_id=?1")
        .map_err(db)?;
    let rows = statement
        .query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                CachedFile {
                    sha: r.get(1)?,
                    content: r.get(2)?,
                },
            ))
        })
        .map_err(db)?;
    rows.collect::<Result<BTreeMap<_, _>, _>>().map_err(db)
}
fn save_files(
    conn: &Connection,
    id: i64,
    files: &BTreeMap<String, CachedFile>,
) -> Result<(), CoreError> {
    conn.execute("DELETE FROM github_files WHERE connection_id=?1", [id])
        .map_err(db)?;
    for (path, file) in files {
        conn.execute(
            "INSERT INTO github_files VALUES (?1,?2,?3,?4)",
            params![id, path, file.sha, file.content],
        )
        .map_err(db)?;
    }
    Ok(())
}
pub fn operations(
    conn: &Connection,
    info: &ConnectionInfo,
    after: u64,
    limit: usize,
) -> Result<Vec<Operation>, CoreError> {
    let mut statement=conn.prepare("SELECT seq,payload FROM github_outbox WHERE connection_id=?1 AND seq>?2 ORDER BY seq LIMIT ?3").map_err(db)?;
    let rows = statement
        .query_map(params![info.id, after, limit], |r| {
            Ok((r.get::<_, u64>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(db)?;
    rows.map(|row| {
        let (seq, payload) = row.map_err(db)?;
        Ok(Operation {
            dataset_id: info.dataset_id.clone(),
            device_id: info.device_id.clone(),
            seq,
            intent: serde_json::from_str(&payload).map_err(|_| error("githubInvalidOperation"))?,
        })
    })
    .collect()
}

pub fn acquire(conn: &Connection, binding: &str) -> Result<(ConnectionInfo, String), CoreError> {
    let info = connection(conn)?.ok_or_else(|| error("githubNotConnected"))?;
    if info.binding != binding {
        return Err(error("githubDatabaseCloneDetected"));
    }
    let lease = random_id(conn)?;
    let changed=conn.execute("UPDATE github_connections SET lease=?2,lease_until=datetime('now','+10 minutes') WHERE id=?1 AND (lease_until IS NULL OR julianday(lease_until)<=julianday('now')) AND (retry_at IS NULL OR julianday(retry_at)<=julianday('now'))",params![info.id,lease]).map_err(db)?;
    if changed == 0 {
        let limited:bool=conn.query_row("SELECT retry_at IS NOT NULL AND julianday(retry_at)>julianday('now') FROM github_connections WHERE id=?1",[info.id],|r|r.get(0)).map_err(db)?;
        return Err(error(if limited {
            "githubRateLimited"
        } else {
            "githubSyncBusy"
        }));
    }
    Ok((info, lease))
}
fn guard(conn: &Connection, id: i64, lease: &str) -> Result<(), CoreError> {
    let valid:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM github_connections WHERE id=?1 AND active=1 AND lease=?2 AND julianday(lease_until)>julianday('now'))",params![id,lease],|r|r.get(0)).map_err(db)?;
    if !valid {
        return Err(error("githubConnectionChanged"));
    }
    Ok(())
}
pub fn record_attempt(
    conn: &Connection,
    id: i64,
    lease: &str,
    head: &str,
    max_seq: u64,
    candidate: Option<&str>,
) -> Result<(), CoreError> {
    guard(conn, id, lease)?;
    conn.execute("INSERT INTO github_attempt VALUES (?1,?2,?3,?4) ON CONFLICT(connection_id) DO UPDATE SET base_head=excluded.base_head,max_seq=excluded.max_seq,candidate=excluded.candidate",params![id,head,max_seq,candidate]).map_err(db)?;
    conn.execute(
        "UPDATE github_connections SET lease_until=datetime('now','+10 minutes') WHERE id=?1",
        [id],
    )
    .map_err(db)?;
    if candidate.is_some() {
        conn.execute(
            "UPDATE github_connections SET last_publish_at=datetime('now') WHERE id=?1",
            [id],
        )
        .map_err(db)?;
    }
    Ok(())
}
pub fn fail(
    conn: &Connection,
    id: i64,
    lease: &str,
    code: &str,
    retry_at: Option<&str>,
) -> Result<(), CoreError> {
    conn.execute("UPDATE github_connections SET lease=NULL,lease_until=NULL,last_error_code=?3,retry_at=COALESCE(?4,datetime('now','+60 seconds')) WHERE id=?1 AND lease=?2",params![id,lease,code,retry_at]).map_err(db)?;
    Ok(())
}
pub fn release(conn: &Connection, id: i64, lease: &str) -> Result<(), CoreError> {
    conn.execute(
        "UPDATE github_connections SET lease=NULL,lease_until=NULL WHERE id=?1 AND lease=?2",
        params![id, lease],
    )
    .map_err(db)?;
    Ok(())
}

pub fn publish_due(conn: &Connection, id: i64) -> Result<bool, CoreError> {
    conn.query_row("SELECT last_publish_at IS NULL OR julianday(last_publish_at)<=julianday('now','-60 seconds') FROM github_connections WHERE id=?1",[id],|r|r.get(0)).map_err(db)
}
pub fn rejected_publication(conn: &Connection, id: i64, lease: &str) -> Result<(), CoreError> {
    guard(conn, id, lease)?;
    conn.execute(
        "UPDATE github_connections SET last_publish_at=NULL WHERE id=?1",
        [id],
    )
    .map_err(db)?;
    Ok(())
}

fn map_entity(
    conn: &Connection,
    id: i64,
    kind: &str,
    key: &str,
    local: i64,
    catalog: Option<&CatalogEntry>,
) -> Result<(), CoreError> {
    let catalog = catalog
        .map(serde_json::to_string)
        .transpose()
        .map_err(|_| error("githubInvalidArticle"))?;
    conn.execute("INSERT INTO github_entity_map VALUES (?1,?2,?3,?4,?5) ON CONFLICT(connection_id,kind,entity_key) DO UPDATE SET local_id=excluded.local_id,catalog=excluded.catalog",params![id,kind,key,local,catalog]).map_err(db)?;
    Ok(())
}
fn mapped(conn: &Connection, id: i64, kind: &str, key: &str) -> Result<Option<i64>, CoreError> {
    conn.query_row("SELECT local_id FROM github_entity_map WHERE connection_id=?1 AND kind=?2 AND entity_key=?3",params![id,kind,key],|r|r.get(0)).optional().map_err(db)
}

fn import_projection(conn: &Connection, id: i64, snapshot: &Snapshot) -> Result<(), CoreError> {
    // Remove tombstones and temporarily vacate names inside the import transaction.
    // Final valid cloud names can otherwise collide with the previous local projection.
    for (key, folder) in &snapshot.subscriptions.folders {
        if folder.deleted && !snapshot.subscriptions.folder_aliases.contains_key(key) {
            if let Some(local) = mapped(conn, id, "folder", key)? {
                conn.execute("DELETE FROM folders WHERE id=?1", [local])
                    .map_err(db)?;
            }
        }
    }
    conn.execute("UPDATE folders SET name='papr-sync-'||lower(hex(randomblob(16))) WHERE id IN (SELECT local_id FROM github_entity_map WHERE connection_id=?1 AND kind='folder')",[id]).map_err(db)?;
    let mut folder_ids = BTreeMap::new();
    for key in &snapshot.subscriptions.folder_order {
        let folder = &snapshot.subscriptions.folders[key];
        let mut local = mapped(conn, id, "folder", key)?.filter(|local| {
            conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM folders WHERE id=?1)",
                [local],
                |r| r.get::<_, bool>(0),
            )
            .unwrap_or(false)
        });
        if local.is_none() {
            local = conn
                .query_row(
                    "SELECT id FROM folders WHERE sync_id=?1 OR name=?2 COLLATE NOCASE LIMIT 1",
                    params![key, folder.name],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db)?;
        }
        let local = match local {
            Some(local) => local,
            None => {
                conn.execute(
                    "INSERT INTO folders(name,sync_id) VALUES (?1,?2)",
                    params![folder.name, key],
                )
                .map_err(db)?;
                conn.last_insert_rowid()
            }
        };
        folder_ids.insert(key.clone(), local);
        map_entity(conn, id, "folder", key, local, None)?;
    }
    for (alias, _) in &snapshot.subscriptions.folder_aliases {
        let target = snapshot.resolve_folder(alias)?;
        if let Some(&target_local) = folder_ids.get(&target) {
            if let Some(old) = mapped(conn, id, "folder", alias)? {
                if old != target_local {
                    conn.execute(
                        "UPDATE feeds SET folder_id=?2 WHERE folder_id=?1",
                        params![old, target_local],
                    )
                    .map_err(db)?;
                    conn.execute("DELETE FROM folders WHERE id=?1", [old])
                        .map_err(db)?;
                }
            }
            map_entity(conn, id, "folder", alias, target_local, None)?;
        }
    }
    for (position, key) in snapshot.subscriptions.folder_order.iter().enumerate() {
        conn.execute(
            "UPDATE folders SET name=?2,position=?3,sync_id=?4 WHERE id=?1",
            params![
                folder_ids[key],
                snapshot.subscriptions.folders[key].name,
                position,
                key
            ],
        )
        .map_err(db)?;
    }
    let mut feed_ids = BTreeMap::new();
    for (key, feed) in &snapshot.subscriptions.feeds {
        let folder = feed
            .folder_id
            .as_ref()
            .map(|id| snapshot.resolve_folder(id))
            .transpose()?
            .and_then(|id| folder_ids.get(&id).copied());
        let existing = conn
            .query_row(
                "SELECT id FROM feeds WHERE papr_feed_key(feed_url)=?1 LIMIT 1",
                [key],
                |r| r.get::<_, i64>(0),
            )
            .optional()
            .map_err(db)?;
        let local = match existing {
            Some(local) => local,
            None => {
                conn.execute("INSERT INTO feeds(feed_url,title,source_type,folder_id,subscription_active,github_generation,custom_title) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![feed.feed_url,feed.display_title,feed.source_type,folder,feed.active,feed.generation,feed.custom_title]).map_err(db)?;
                conn.last_insert_rowid()
            }
        };
        conn.execute("UPDATE feeds SET title=?2,custom_title=?3,folder_id=?4,subscription_active=?5,github_generation=?6 WHERE id=?1",params![local,feed.display_title,feed.custom_title,folder,feed.active,feed.generation]).map_err(db)?;
        feed_ids.insert(key.clone(), local);
        map_entity(conn, id, "feed", key, local, None)?;
    }
    let mut mappings=conn.prepare("SELECT entity_key,local_id FROM github_entity_map WHERE connection_id=?1 AND kind='article'").map_err(db)?;
    let known = mappings
        .query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))
        .map_err(db)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(db)?;
    for (key, local) in known {
        if !snapshot.articles.contains_key(&key) {
            conn.execute(
                "UPDATE articles SET is_starred=0,read_later=0 WHERE id=?1",
                [local],
            )
            .map_err(db)?;
        }
    }
    for (key, entry) in &snapshot.articles {
        let state = snapshot.states.get(key).cloned().unwrap_or_default();
        let suppressed:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM github_suppressed WHERE connection_id=?1 AND entity_key=?2)",params![id,key],|r|r.get(0)).map_err(db)?;
        if suppressed && !state.protected() {
            continue;
        }
        let feed_id = feed_ids[&entry.feed_key];
        let mut local = mapped(conn, id, "article", key)?.filter(|local| {
            conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM articles WHERE id=?1 AND feed_id=?2)",
                params![local, feed_id],
                |r| r.get::<_, bool>(0),
            )
            .unwrap_or(false)
        });
        if local.is_none() && !entry.guid.is_empty() {
            local = conn
                .query_row(
                    "SELECT id FROM articles WHERE feed_id=?1 AND guid=?2",
                    params![feed_id, entry.guid],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db)?;
        }
        let local = match local {
            Some(local) => local,
            None => {
                let guid = if entry.guid.is_empty() {
                    format!("papr-sync-url:{key}")
                } else {
                    entry.guid.clone()
                };
                conn.execute("INSERT INTO articles(feed_id,guid,url,title,published_at,fetched_at,metadata_only) VALUES (?1,?2,?3,?4,?5,?6,1)",params![feed_id,guid,entry.url,entry.title,entry.published_at,entry.first_seen_at]).map_err(db)?;
                let local = conn.last_insert_rowid();
                conn.execute(
                    "INSERT INTO articles_fts(rowid,title,body) VALUES (?1,?2,'')",
                    params![local, entry.title],
                )
                .map_err(db)?;
                local
            }
        };
        conn.execute(
            "UPDATE articles SET is_read=?2,is_starred=?3,read_later=?4 WHERE id=?1",
            params![
                local,
                state.read.value,
                state.starred.value,
                state.read_later.value
            ],
        )
        .map_err(db)?;
        map_entity(conn, id, "article", key, local, Some(entry))?;
    }
    Ok(())
}

/// Import, acknowledgement, cursor and cache commit together. New edits made
/// during network I/O are projected on top before touching business rows.
pub fn finish(
    conn: &Connection,
    id: i64,
    lease: &str,
    remote: &RemoteSnapshot,
) -> Result<(), CoreError> {
    guard(conn, id, lease)?;
    let info = connection(conn)?
        .filter(|i| i.id == id)
        .ok_or_else(|| error("githubConnectionChanged"))?;
    let snapshot = remote
        .snapshot
        .as_ref()
        .ok_or_else(|| error("githubDatasetMissing"))?;
    snapshot.validate()?;
    if snapshot.manifest.dataset_id != info.dataset_id || snapshot.manifest.epoch != info.epoch {
        return Err(error("githubDatasetChanged"));
    }
    let ack = snapshot
        .manifest
        .devices
        .get(&info.device_id)
        .map_or(0, |d| d.processed_seq);
    if ack < info.ack_seq || ack >= info.next_seq {
        return Err(error("githubRestoredDatabase"));
    }
    let pending = operations(conn, &info, ack, usize::MAX.min(i64::MAX as usize))?;
    // Apply pending batches in order; the reducer itself has a 500-op bound.
    let mut projection = snapshot.clone();
    for batch in pending.chunks(BATCH_SIZE) {
        projection = super::merge::merge(&projection, batch, None)?.snapshot;
    }
    conn.execute("UPDATE github_connections SET applying=1 WHERE id=?1", [id])
        .map_err(db)?;
    import_projection(conn, id, &projection)?;
    conn.execute("DELETE FROM github_versions WHERE connection_id=?1", [id])
        .map_err(db)?;
    let put =
        |kind: &str, key: &str, field: &str, version: Option<&str>| -> Result<(), CoreError> {
            conn.execute(
                "INSERT INTO github_versions VALUES (?1,?2,?3,?4,?5)",
                params![id, kind, key, field, version],
            )
            .map_err(db)?;
            Ok(())
        };
    for (key, folder) in &projection.subscriptions.folders {
        put("folder", key, "name", folder.name_version.as_deref())?;
    }
    put(
        "folder_order",
        "",
        "order",
        projection.subscriptions.order_version.as_deref(),
    )?;
    for (key, feed) in &projection.subscriptions.feeds {
        for (field, version) in &feed.field_versions {
            put("feed", key, field, Some(version))?;
        }
    }
    for (key, state) in &projection.states {
        for (field, flag) in [
            ("read", &state.read),
            ("starred", &state.starred),
            ("read_later", &state.read_later),
        ] {
            put("article", key, field, flag.version.as_deref())?;
        }
    }
    if let Some(watermark) = snapshot.manifest.devices.get(&info.device_id) {
        for rejection in &watermark.last_rejections {
            conn.execute(
                "INSERT OR IGNORE INTO github_rejections VALUES (?1,?2,?3)",
                params![id, rejection.seq, rejection.code],
            )
            .map_err(db)?;
        }
    }
    save_files(conn, id, &remote.files)?;
    conn.execute(
        "DELETE FROM github_outbox WHERE connection_id=?1 AND seq<=?2",
        params![id, ack],
    )
    .map_err(db)?;
    conn.execute(
        "DELETE FROM github_attempt WHERE connection_id=?1 AND max_seq<=?2",
        params![id, ack],
    )
    .map_err(db)?;
    conn.execute("UPDATE github_connections SET applying=0,ack_seq=?2,head=?3,last_success_at=datetime('now'),last_error_code=NULL,retry_at=NULL WHERE id=?1",params![id,ack,remote.head]).map_err(db)?;
    conn.execute("UPDATE github_connections SET pending_since=NULL WHERE id=?1 AND NOT EXISTS(SELECT 1 FROM github_outbox WHERE connection_id=?1)",[id]).map_err(db)?;
    Ok(())
}

fn db(error: rusqlite::Error) -> CoreError {
    CoreError::Db(error.to_string())
}
pub fn random_id(conn: &Connection) -> Result<String, CoreError> {
    conn.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))
        .map_err(db)
}

/// Metadata imported from another device is hydrated before evaluating local
/// ingestion rules. It remains the same article and never counts as new.
pub fn hydrate(
    conn: &Connection,
    feed_id: i64,
    article: &crate::dto::NewArticle,
) -> Result<bool, CoreError> {
    let mut local = conn
        .query_row(
            "SELECT id FROM articles WHERE feed_id=?1 AND guid=?2 AND metadata_only=1",
            params![feed_id, article.guid],
            |r| r.get::<_, i64>(0),
        )
        .optional()
        .map_err(db)?;
    if local.is_none() {
        if let Some(url) = article.url.as_deref().and_then(|s| canonical_url(s).ok()) {
            let mut statement=conn.prepare("SELECT id FROM articles WHERE feed_id=?1 AND metadata_only=1 AND url=?2 AND guid LIKE 'papr-sync-url:%'").map_err(db)?;
            let matches = statement
                .query_map(params![feed_id, url], |r| r.get::<_, i64>(0))
                .map_err(db)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(db)?;
            if matches.len() == 1 {
                local = Some(matches[0]);
            }
        }
    }
    let Some(local) = local else { return Ok(false) };
    conn.execute("UPDATE articles SET guid=?2,title=?3,author=?4,summary=?5,content_html=?6,body_text=?7,image_url=?8,published_at=COALESCE(?9,published_at),metadata_only=0 WHERE id=?1",params![local,article.guid,article.title,article.author,article.summary,article.content_html,article.body_text,article.image_url,article.published_at]).map_err(db)?;
    conn.execute("DELETE FROM articles_fts WHERE rowid=?1", [local])
        .map_err(db)?;
    conn.execute(
        "INSERT INTO articles_fts(rowid,title,body) VALUES (?1,?2,?3)",
        params![local, article.title, article.body_text],
    )
    .map_err(db)?;
    conn.execute("DELETE FROM enclosures WHERE article_id=?1", [local])
        .map_err(db)?;
    for enclosure in &article.enclosures {
        conn.execute(
            "INSERT INTO enclosures(article_id,url,mime_type,length) VALUES (?1,?2,?3,?4)",
            params![local, enclosure.url, enclosure.mime_type, enclosure.length],
        )
        .map_err(db)?;
    }
    Ok(true)
}
fn date(value: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(value)
        .map(|d| d.with_timezone(&Utc).to_rfc3339())
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|d| d.and_utc().to_rfc3339())
        })
}

pub fn register(conn: &Connection) -> Result<(), CoreError> {
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    conn.create_scalar_function("papr_feed_key", 1, flags, |ctx| {
        let url: String = ctx.get(0)?;
        Ok(feed_key(&url).ok())
    })
    .map_err(db)?;
    conn.create_scalar_function("papr_article_key", 1, flags, |ctx| {
        let value: Option<String> = ctx.get(0)?;
        Ok(value
            .and_then(|s| serde_json::from_str::<CatalogEntry>(&s).ok())
            .map(|entry| entry.key()))
    })
    .map_err(db)?;
    conn.create_scalar_function("papr_catalog", 8, flags, |ctx| {
        let feed: Option<String> = ctx.get(0)?;
        let guid: String = ctx.get(1)?;
        let url: Option<String> = ctx.get(2)?;
        let title: String = ctx.get(3)?;
        let published: Option<String> = ctx.get(4)?;
        let fetched: String = ctx.get(5)?;
        let mapped: Option<String> = ctx.get(6)?;
        let now: String = ctx.get(7)?;
        let Some(feed_key) = feed else {
            return Ok(None::<String>);
        };
        let mapped = mapped.and_then(|s| serde_json::from_str::<CatalogEntry>(&s).ok());
        let url = url.and_then(|s| canonical_url(&s).ok());
        let (identity_kind, identity_value) = match mapped {
            Some(ref entry) => (entry.identity_kind, entry.identity_value.clone()),
            None if !guid.is_empty() => (IdentityKind::Guid, guid.clone()),
            None => match &url {
                Some(url) => (IdentityKind::Url, url.clone()),
                None => return Ok(None),
            },
        };
        let guid = if guid.starts_with("papr-sync-url:") {
            mapped.as_ref().map(|e| e.guid.clone()).unwrap_or_default()
        } else {
            guid
        };
        let entry = CatalogEntry {
            feed_key,
            identity_kind,
            identity_value,
            guid,
            title,
            url,
            published_at: published.and_then(|s| date(&s)),
            first_seen_at: date(&fetched)
                .or_else(|| date(&now))
                .unwrap_or_else(|| "1970-01-01T00:00:00Z".into()),
        };
        Ok(if entry.validate().is_ok() {
            serde_json::to_string(&entry).ok()
        } else {
            None
        })
    })
    .map_err(db)?;
    Ok(())
}

// These views expose only the allowlisted metadata needed to capture intents.
pub fn install_capture(conn: &Connection) -> Result<(), CoreError> {
    register(conn)?;
    conn.execute_batch(r#"
CREATE VIEW IF NOT EXISTS github_feed_capture AS
 SELECT f.id, papr_feed_key(f.feed_url) AS entity_key,
 json_object('feed_key',papr_feed_key(f.feed_url),'feed_url',f.feed_url,'source_type',f.source_type,
 'display_title',f.title,'custom_title',json(CASE WHEN f.custom_title THEN 'true' ELSE 'false' END),
 'folder_id',(SELECT sync_id FROM folders WHERE id=f.folder_id),'active',json(CASE WHEN f.subscription_active THEN 'true' ELSE 'false' END),
 'generation',f.github_generation,'field_versions',json('{}')) AS subscription
 FROM feeds f WHERE f.source_type IN ('rss','youtube','podcast','mastodon','bluesky','reddit') AND papr_feed_key(f.feed_url) IS NOT NULL;
CREATE VIEW IF NOT EXISTS github_article_capture AS
 SELECT a.id, f.entity_key AS feed_key, a.feed_id, x.github_generation AS generation,
 papr_catalog(f.entity_key,a.guid,a.url,a.title,a.published_at,a.fetched_at,
 (SELECT m.catalog FROM github_entity_map m JOIN github_connections c ON c.id=m.connection_id AND c.active=1 WHERE m.kind='article' AND m.local_id=a.id LIMIT 1),datetime('now')) AS entry
 FROM articles a JOIN feeds x ON x.id=a.feed_id JOIN github_feed_capture f ON f.id=a.feed_id;
CREATE TRIGGER IF NOT EXISTS github_outbox_sequence AFTER INSERT ON github_outbox BEGIN
 UPDATE github_connections SET next_seq=new.seq+1,pending_since=COALESCE(pending_since,datetime('now')),last_edit_at=datetime('now') WHERE id=new.connection_id;
END;
CREATE TRIGGER IF NOT EXISTS github_folder_identity AFTER INSERT ON folders WHEN new.sync_id IS NULL BEGIN
 UPDATE folders SET sync_id=lower(hex(randomblob(16))) WHERE id=new.id AND sync_id IS NULL;
END;
CREATE TRIGGER IF NOT EXISTS github_preserve_source BEFORE DELETE ON feeds
 WHEN EXISTS(SELECT 1 FROM github_connections WHERE active=1 AND applying=0)
 AND EXISTS(SELECT 1 FROM github_feed_capture WHERE id=old.id)
 BEGIN UPDATE feeds SET subscription_active=0 WHERE id=old.id; SELECT RAISE(IGNORE); END;
CREATE TRIGGER IF NOT EXISTS github_local_article_delete BEFORE DELETE ON articles
 WHEN EXISTS(SELECT 1 FROM github_connections WHERE active=1 AND applying=0) BEGIN
 INSERT OR IGNORE INTO github_suppressed SELECT connection_id,entity_key FROM github_entity_map WHERE kind='article' AND local_id=old.id;
 DELETE FROM github_entity_map WHERE kind='article' AND local_id=old.id;
END;
"#).map_err(db)?;
    let active = "c.active=1 AND c.applying=0";
    // No SQL runs inside a scalar function. All payloads are evaluated by the
    // writer's trigger; incoming import uses applying=1 in the same transaction.
    let version = |kind: &str, key: &str, field: &str| {
        format!("(SELECT version FROM github_versions WHERE connection_id=c.id AND kind='{kind}' AND entity_key={key} AND field='{field}')")
    };
    let install = |name: &str,
                   event: &str,
                   table: &str,
                   condition: &str,
                   payload: String,
                   from: &str,
                   key: Option<(&str, &str, &str)>|
     -> Result<(), CoreError> {
        let update = key.map(|(kind, entity_key, field)| format!("INSERT INTO github_versions(connection_id,kind,entity_key,field,version) SELECT c.id,'{kind}',{entity_key},'{field}',c.device_id||':'||(c.next_seq-1) FROM github_connections c {from} WHERE {active} AND {condition} ON CONFLICT(connection_id,kind,entity_key,field) DO UPDATE SET version=excluded.version;")).unwrap_or_default();
        let identity = if name == "folder_create" {
            "UPDATE folders SET sync_id=lower(hex(randomblob(16))) WHERE id=new.id AND sync_id IS NULL;"
        } else {
            ""
        };
        conn.execute_batch(&format!("CREATE TRIGGER IF NOT EXISTS github_{name} {event} ON {table} BEGIN {identity} INSERT INTO github_outbox(connection_id,seq,payload) SELECT c.id,c.next_seq,{payload} FROM github_connections c {from} WHERE {active} AND {condition}; {update} END;")).map_err(db)
    };
    install("folder_create", "AFTER INSERT", "folders", "1=1", "json_object('kind','create_folder','folder_id',(SELECT sync_id FROM folders WHERE id=new.id),'name',new.name)".into(), "", None)?;
    for (name, event, kind, value, condition) in [
        (
            "folder_rename",
            "AFTER UPDATE OF name",
            "rename_folder",
            ",'name',new.name",
            "new.name != old.name",
        ),
        ("folder_delete", "BEFORE DELETE", "delete_folder", "", "1=1"),
    ] {
        let row = if kind == "delete_folder" {
            "old"
        } else {
            "new"
        };
        let id = format!("{row}.sync_id");
        install(
            name,
            event,
            "folders",
            condition,
            format!(
                "json_object('kind','{kind}','folder_id',{id}{value},'base_version',{})",
                version("folder", &id, "name")
            ),
            "",
            Some(("folder", &id, "name")),
        )?;
    }
    install("folder_order","AFTER UPDATE OF position","folders","new.position != old.position",format!("json_object('kind','reorder_folders','folder_ids',json((SELECT json_group_array(sync_id) FROM (SELECT sync_id FROM folders ORDER BY position,name))),'base_version',{})",version("folder_order","''","order")),"",Some(("folder_order","''","order")))?;
    let feed_join = "JOIN github_feed_capture f ON f.id=new.id";
    install(
        "feed_create",
        "AFTER INSERT",
        "feeds",
        "1=1",
        "json_object('kind','subscribe','subscription',json(f.subscription))".into(),
        feed_join,
        None,
    )?;
    for (name,field,value,condition) in [("feed_title","title","json_object('field','title','display_title',new.title,'custom_title',json(CASE WHEN new.custom_title THEN 'true' ELSE 'false' END))","(new.title != old.title OR new.custom_title != old.custom_title) AND new.custom_title=1"),("feed_folder","folder_id","json_object('field','folder','folder_id',(SELECT sync_id FROM folders WHERE id=new.folder_id))","new.folder_id IS NOT old.folder_id")] {
        install(name,&format!("AFTER UPDATE OF {}",if field=="title" {"title,custom_title"}else{"folder_id"}),"feeds",condition,format!("json_object('kind','set_subscription_field','feed_key',f.entity_key,'generation',new.github_generation,'value',json({value}),'base_version',{})",version("feed","f.entity_key",field)),feed_join,Some(("feed","f.entity_key",field)))?;
    }
    install("unsubscribe","AFTER UPDATE OF subscription_active","feeds","new.subscription_active=0 AND old.subscription_active=1",format!("json_object('kind','unsubscribe','feed_key',f.entity_key,'generation',old.github_generation,'base_version',{})",version("feed","f.entity_key","active")),feed_join,Some(("feed","f.entity_key","active")))?;
    install("resubscribe","AFTER UPDATE OF subscription_active","feeds","new.subscription_active=1 AND old.subscription_active=0","json_object('kind','resubscribe','subscription',json(f.subscription),'observed_generation',old.github_generation)".into(),feed_join,Some(("feed","f.entity_key","active")))?;
    let article_join = "JOIN github_article_capture a ON a.id=new.id";
    install(
        "article_create",
        "AFTER INSERT",
        "articles",
        "a.entry IS NOT NULL",
        "json_object('kind','ensure_article','entry',json(a.entry),'generation',a.generation)"
            .into(),
        article_join,
        None,
    )?;
    for (field, column) in [
        ("read", "is_read"),
        ("starred", "is_starred"),
        ("read_later", "read_later"),
    ] {
        // The identity key is computed by a pure scalar from the catalog JSON.
        let key = "papr_article_key(a.entry)";
        let payload = format!("json_object('kind','set_article_state','entry',json(a.entry),'field','{field}','value',json(CASE WHEN new.{column} THEN 'true' ELSE 'false' END),'base_version',{})",version("article",key,field));
        install(
            &format!("state_{field}"),
            &format!("AFTER UPDATE OF {column}"),
            "articles",
            &format!("a.entry IS NOT NULL AND new.{column} != old.{column}"),
            payload.clone(),
            article_join,
            Some(("article", key, field)),
        )?;
        install(
            &format!("initial_{field}"),
            "AFTER INSERT",
            "articles",
            &format!("a.entry IS NOT NULL AND new.{column}=1"),
            payload,
            article_join,
            Some(("article", key, field)),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Connection, ConnectionInfo, Snapshot) {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        crate::db::migrate(&mut conn).unwrap();
        conn.execute(
            "INSERT INTO feeds(feed_url,title) VALUES ('https://example.com/feed','Feed')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO articles(feed_id,guid,title,content_html,body_text) VALUES (1,'guid','Article','<p>private body</p>','private body')",[]).unwrap();
        let snapshot = Snapshot::empty("d".repeat(32), "1970-01-01T00:00:00Z".into());
        let remote = RemoteSnapshot {
            head: "a".repeat(40),
            tree: "b".repeat(40),
            snapshot: Some(snapshot.clone()),
            files: snapshot
                .files()
                .unwrap()
                .into_iter()
                .map(|(p, content)| {
                    (
                        p,
                        CachedFile {
                            sha: "c".repeat(40),
                            content,
                        },
                    )
                })
                .collect(),
            trusted_now: None,
            repository_size_kib: None,
        };
        let tx = conn.unchecked_transaction().unwrap();
        let profile = GitHubProfile {
            repository_id: 1,
            owner: "owner".into(),
            repo: "repo".into(),
            branch: "main".into(),
            credential_ref: "papr.sync.test".into(),
        };
        let info = connect(&tx, &profile, &remote, "installation:path").unwrap();
        tx.commit().unwrap();
        (conn, info, snapshot)
    }
    fn remote(snapshot: Snapshot) -> RemoteSnapshot {
        let files = snapshot
            .files()
            .unwrap()
            .into_iter()
            .map(|(p, content)| {
                (
                    p,
                    CachedFile {
                        sha: String::new(),
                        content,
                    },
                )
            })
            .collect();
        RemoteSnapshot {
            head: "e".repeat(40),
            tree: "f".repeat(40),
            snapshot: Some(snapshot),
            files,
            trusted_now: None,
            repository_size_kib: None,
        }
    }

    #[test]
    fn capture_rolls_back_and_never_contains_bodies_or_configuration() {
        let (conn, info, _) = setup();
        {
            let tx = conn.unchecked_transaction().unwrap();
            tx.execute("UPDATE articles SET is_starred=1 WHERE id=1", [])
                .unwrap();
            assert_eq!(operations(&tx, &info, 0, 500).unwrap().len(), 2);
        }
        assert_eq!(operations(&conn, &info, 0, 500).unwrap().len(), 1);
        conn.execute(
            "UPDATE articles SET is_starred=1,read_later=1 WHERE id=1",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO settings VALUES ('ai_key','local-private-value')",
            [],
        )
        .unwrap();
        let ops = operations(&conn, &info, 0, 500).unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(connection(&conn).unwrap().unwrap().next_seq, 4);
        let json = serde_json::to_string(&ops).unwrap();
        assert!(!json.contains("private body"));
        assert!(!json.contains("local-private-value"));
        assert!(ops.iter().any(|o| matches!(
            o.intent,
            Intent::SetArticleState {
                field: super::super::merge::StateField::ReadLater,
                value: true,
                ..
            }
        )));
    }

    #[test]
    fn import_ack_and_inflight_edit_commit_together_without_echo() {
        let (conn, info, base) = setup();
        let (_, lease) = acquire(&conn, "installation:path").unwrap();
        let seed = operations(&conn, &info, 0, 500).unwrap();
        let accepted = super::super::merge::merge(&base, &seed, None)
            .unwrap()
            .snapshot;
        conn.execute("UPDATE articles SET is_starred=1 WHERE id=1", [])
            .unwrap();
        let tx = conn.unchecked_transaction().unwrap();
        finish(&tx, info.id, &lease, &remote(accepted)).unwrap();
        tx.commit().unwrap();
        let current = connection(&conn).unwrap().unwrap();
        assert_eq!(current.ack_seq, 1);
        assert_eq!(current.next_seq, 3);
        assert_eq!(operations(&conn, &current, 0, 500).unwrap().len(), 1);
        let state: (bool, String) = conn
            .query_row(
                "SELECT is_starred,body_text FROM articles WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(state, (true, "private body".into()));
        let files = files(&conn, info.id).unwrap();
        let confirmed =
            Snapshot::from_files(&files.into_iter().map(|(p, f)| (p, f.content)).collect())
                .unwrap();
        assert!(!confirmed.states.values().any(|s| s.starred.value));
    }

    #[test]
    fn folders_bulk_flags_and_unsubscribe_are_durable_intents() {
        let (conn, info, base) = setup();
        conn.execute("INSERT INTO folders(name) VALUES ('Tech')", [])
            .unwrap();
        conn.execute("UPDATE feeds SET folder_id=1 WHERE id=1", [])
            .unwrap();
        conn.execute("UPDATE articles SET is_read=1,read_later=1", [])
            .unwrap();
        conn.execute("DELETE FROM feeds WHERE id=1", []).unwrap();
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM articles", [], |r| r
                .get::<_, usize>(0))
                .unwrap(),
            1
        );
        assert!(!conn
            .query_row("SELECT subscription_active FROM feeds", [], |r| r
                .get::<_, bool>(0))
            .unwrap());
        let ops = operations(&conn, &info, 0, 500).unwrap();
        let report = super::super::merge::merge(&base, &ops, None).unwrap();
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);
        assert!(
            !report
                .snapshot
                .subscriptions
                .feeds
                .values()
                .next()
                .unwrap()
                .active
        );
        assert!(
            report
                .snapshot
                .states
                .values()
                .next()
                .unwrap()
                .read_later
                .value
        );
    }

    #[test]
    fn clone_lease_and_changed_dataset_fail_without_partial_import() {
        let (conn, info, mut base) = setup();
        assert_eq!(
            acquire(&conn, "other-installation").unwrap_err().code(),
            "githubDatabaseCloneDetected"
        );
        let (_, lease) = acquire(&conn, "installation:path").unwrap();
        assert_eq!(
            acquire(&conn, "installation:path").unwrap_err().code(),
            "githubSyncBusy"
        );
        base.manifest.dataset_id = "e".repeat(32);
        {
            let tx = conn.unchecked_transaction().unwrap();
            assert_eq!(
                finish(&tx, info.id, &lease, &remote(base))
                    .unwrap_err()
                    .code(),
                "githubDatasetChanged"
            );
        }
        assert_eq!(connection(&conn).unwrap().unwrap().ack_seq, 0);
        assert_eq!(operations(&conn, &info, 0, 500).unwrap().len(), 1);
    }

    #[test]
    fn secure_checkpoint_detects_restored_sequence_and_wrong_device() {
        let (conn, _, _) = setup();
        let point = checkpoint(&conn, None).unwrap();
        conn.execute("UPDATE articles SET is_read=1", []).unwrap();
        let newer = checkpoint(&conn, Some(&point)).unwrap();
        conn.execute("UPDATE github_connections SET next_seq=2", [])
            .unwrap();
        assert_eq!(
            checkpoint(&conn, Some(&newer)).unwrap_err().code(),
            "githubRestoredDatabase"
        );
        assert_eq!(
            checkpoint(&conn, Some("different:device:1"))
                .unwrap_err()
                .code(),
            "githubRestoredDatabase"
        );
    }

    #[test]
    fn metadata_hydration_preserves_flags_and_never_emits_a_new_article_intent() {
        let (conn, info, _) = setup();
        conn.execute(
            "UPDATE articles SET metadata_only=1,is_starred=1,read_later=1",
            [],
        )
        .unwrap();
        let before = operations(&conn, &info, 0, 500).unwrap().len();
        let article = crate::dto::NewArticle {
            guid: "guid".into(),
            url: Some("https://example.com/article".into()),
            title: "Hydrated article".into(),
            author: None,
            summary: None,
            content_html: Some("<p>new local body</p>".into()),
            body_text: "new local body".into(),
            image_url: None,
            published_at: None,
            enclosures: Vec::new(),
        };
        {
            let tx = conn.unchecked_transaction().unwrap();
            assert!(hydrate(&tx, 1, &article).unwrap());
            tx.commit().unwrap();
        }
        let state: (bool, bool, bool, String) = conn
            .query_row(
                "SELECT metadata_only,is_starred,read_later,body_text FROM articles WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(state, (false, true, true, "new local body".into()));
        assert_eq!(operations(&conn, &info, 0, 500).unwrap().len(), before);
        assert!(!hydrate(&conn, 1, &article).unwrap());
    }
    #[test]
    fn two_databases_import_stable_ids_and_metadata_then_local_cleanup_stays_local() {
        let (conn, info, base) = setup();
        conn.execute("INSERT INTO folders(name) VALUES ('Tech')", [])
            .unwrap();
        conn.execute("UPDATE feeds SET folder_id=1 WHERE id=1", [])
            .unwrap();
        conn.execute(
            "UPDATE articles SET is_starred=1,read_later=1 WHERE id=1",
            [],
        )
        .unwrap();
        let accepted =
            super::super::merge::merge(&base, &operations(&conn, &info, 0, 500).unwrap(), None)
                .unwrap()
                .snapshot;
        let accepted_remote = remote(accepted.clone());
        let mut second = Connection::open_in_memory().unwrap();
        second.pragma_update(None, "foreign_keys", true).unwrap();
        crate::db::migrate(&mut second).unwrap();
        second.execute("INSERT INTO feeds(feed_url,title) VALUES ('https://unrelated.example/feed','Unrelated')",[]).unwrap();
        second
            .execute("INSERT INTO folders(name) VALUES ('Other')", [])
            .unwrap();
        let mut profile = info.profile.clone();
        profile.credential_ref = "papr.sync.second".into();
        let other = {
            let tx = second.unchecked_transaction().unwrap();
            let other = connect(&tx, &profile, &accepted_remote, "second-installation").unwrap();
            tx.commit().unwrap();
            other
        };
        let (_, lease) = acquire(&second, "second-installation").unwrap();
        let combined = super::super::merge::merge(
            &accepted,
            &operations(&second, &other, 0, 500).unwrap(),
            None,
        )
        .unwrap()
        .snapshot;
        {
            let tx = second.unchecked_transaction().unwrap();
            finish(&tx, other.id, &lease, &remote(combined.clone())).unwrap();
            tx.commit().unwrap();
        }
        let (feed_id,folder_id,article_id,starred,later,placeholder,body):(i64,i64,i64,bool,bool,bool,String)=second.query_row("SELECT f.id,f.folder_id,a.id,a.is_starred,a.read_later,a.metadata_only,a.body_text FROM articles a JOIN feeds f ON f.id=a.feed_id WHERE a.guid='guid'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).unwrap();
        assert_ne!(feed_id, 1);
        assert_ne!(folder_id, 1);
        assert!(starred && later && placeholder);
        assert!(body.is_empty());
        assert_eq!(
            second
                .query_row("SELECT name FROM folders WHERE id=?1", [folder_id], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap(),
            "Tech"
        );
        assert_eq!(
            second
                .query_row(
                    "SELECT COUNT(*) FROM articles_fts WHERE articles_fts MATCH 'Article'",
                    [],
                    |r| r.get::<_, usize>(0)
                )
                .unwrap(),
            1
        );
        release(&second, other.id, &lease).unwrap();
        // Explicit false from the receiving device survives the next confirmed snapshot.
        second
            .execute(
                "UPDATE articles SET is_starred=0,read_later=0 WHERE id=?1",
                [article_id],
            )
            .unwrap();
        let current = connection(&second).unwrap().unwrap();
        let cleared = super::super::merge::merge(
            &combined,
            &operations(&second, &current, current.ack_seq, 500).unwrap(),
            None,
        )
        .unwrap()
        .snapshot;
        let (_, lease) = acquire(&second, "second-installation").unwrap();
        {
            let tx = second.unchecked_transaction().unwrap();
            finish(&tx, other.id, &lease, &remote(cleared.clone())).unwrap();
            tx.commit().unwrap();
        }
        let before = operations(&second, &connection(&second).unwrap().unwrap(), 0, 500)
            .unwrap()
            .len();
        second
            .execute("DELETE FROM articles WHERE id=?1", [article_id])
            .unwrap();
        assert_eq!(
            operations(&second, &connection(&second).unwrap().unwrap(), 0, 500)
                .unwrap()
                .len(),
            before
        );
        {
            let tx = second.unchecked_transaction().unwrap();
            finish(&tx, other.id, &lease, &remote(cleared)).unwrap();
            tx.commit().unwrap();
        }
        assert_eq!(
            second
                .query_row("SELECT COUNT(*) FROM articles", [], |r| r
                    .get::<_, usize>(0))
                .unwrap(),
            0
        );
        assert!(accepted
            .states
            .values()
            .any(|s| s.starred.value && s.read_later.value));
    }

    #[test]
    fn reconnect_isolates_old_response_and_errors_back_off() {
        let (conn, info, base) = setup();
        let (_, lease) = acquire(&conn, "installation:path").unwrap();
        fail(&conn, info.id, &lease, "githubNetwork", None).unwrap();
        assert_eq!(
            acquire(&conn, "installation:path").unwrap_err().code(),
            "githubRateLimited"
        );
        disconnect(&conn).unwrap();
        let other = connect(&conn, &info.profile, &remote(base), "installation:path").unwrap();
        assert_ne!(info.id, other.id);
        assert_ne!(info.device_id, other.device_id);
        assert_eq!(
            finish(
                &conn,
                info.id,
                &lease,
                &remote(Snapshot::empty(
                    info.dataset_id,
                    "1970-01-01T00:00:00Z".into()
                ))
            )
            .unwrap_err()
            .code(),
            "githubConnectionChanged"
        );
        assert_eq!(connection(&conn).unwrap().unwrap().ack_seq, 0);
    }

    #[test]
    fn folder_name_swaps_and_deleted_name_reuse_import_atomically() {
        let (conn, info, base) = setup();
        conn.execute("INSERT INTO folders(name) VALUES ('Alpha'),('Beta')", [])
            .unwrap();
        let initial =
            super::super::merge::merge(&base, &operations(&conn, &info, 0, 500).unwrap(), None)
                .unwrap()
                .snapshot;
        let (_, lease) = acquire(&conn, "installation:path").unwrap();
        {
            let tx = conn.unchecked_transaction().unwrap();
            finish(&tx, info.id, &lease, &remote(initial.clone())).unwrap();
            tx.commit().unwrap();
        }
        let alpha = initial
            .subscriptions
            .folders
            .values()
            .find(|f| f.name == "Alpha")
            .unwrap()
            .folder_id
            .clone();
        let beta = initial
            .subscriptions
            .folders
            .values()
            .find(|f| f.name == "Beta")
            .unwrap()
            .folder_id
            .clone();
        let operation = |seq, intent| Operation {
            dataset_id: info.dataset_id.clone(),
            device_id: "f".repeat(32),
            seq,
            intent,
        };
        let swapped = super::super::merge::merge(
            &initial,
            &[
                operation(
                    1,
                    Intent::RenameFolder {
                        folder_id: alpha.clone(),
                        name: "Temporary".into(),
                        base_version: None,
                    },
                ),
                operation(
                    2,
                    Intent::RenameFolder {
                        folder_id: beta.clone(),
                        name: "Alpha".into(),
                        base_version: None,
                    },
                ),
                operation(
                    3,
                    Intent::RenameFolder {
                        folder_id: alpha.clone(),
                        name: "Beta".into(),
                        base_version: None,
                    },
                ),
            ],
            None,
        )
        .unwrap()
        .snapshot;
        {
            let tx = conn.unchecked_transaction().unwrap();
            finish(&tx, info.id, &lease, &remote(swapped.clone())).unwrap();
            tx.commit().unwrap();
        }
        assert_eq!(
            conn.query_row("SELECT name FROM folders WHERE sync_id=?1", [&alpha], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "Beta"
        );
        let replacement = "e".repeat(32);
        let reused = super::super::merge::merge(
            &swapped,
            &[
                operation(
                    4,
                    Intent::DeleteFolder {
                        folder_id: alpha,
                        base_version: None,
                    },
                ),
                operation(
                    5,
                    Intent::CreateFolder {
                        folder_id: replacement.clone(),
                        name: "Beta".into(),
                    },
                ),
            ],
            None,
        )
        .unwrap()
        .snapshot;
        {
            let tx = conn.unchecked_transaction().unwrap();
            finish(&tx, info.id, &lease, &remote(reused)).unwrap();
            tx.commit().unwrap();
        }
        assert_eq!(
            conn.query_row(
                "SELECT name FROM folders WHERE sync_id=?1",
                [replacement],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "Beta"
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM folders", [], |r| r.get::<_, usize>(0))
                .unwrap(),
            2
        );
    }
    #[test]
    fn cloud_retention_removes_catalog_without_deleting_local_cached_body() {
        let (conn, info, base) = setup();
        let (_, lease) = acquire(&conn, "installation:path").unwrap();
        let mut accepted =
            super::super::merge::merge(&base, &operations(&conn, &info, 0, 500).unwrap(), None)
                .unwrap()
                .snapshot;
        accepted.articles.values_mut().next().unwrap().published_at =
            Some("2000-01-01T00:00:00Z".into());
        {
            let tx = conn.unchecked_transaction().unwrap();
            finish(&tx, info.id, &lease, &remote(accepted.clone())).unwrap();
            tx.commit().unwrap();
        }
        let expired = super::super::merge::merge(&accepted, &[], Some("2026-07-07T00:00:00Z"))
            .unwrap()
            .snapshot;
        assert!(expired.articles.is_empty());
        {
            let tx = conn.unchecked_transaction().unwrap();
            finish(&tx, info.id, &lease, &remote(expired)).unwrap();
            tx.commit().unwrap();
        }
        assert_eq!(
            conn.query_row("SELECT body_text FROM articles WHERE id=1", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "private body"
        );
        assert_eq!(
            operations(&conn, &connection(&conn).unwrap().unwrap(), 0, 500)
                .unwrap()
                .len(),
            0
        );
    }
}
