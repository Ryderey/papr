//! Pure ordered-intent reducer. Clocks do not decide field winners.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::{error, model::*};
use crate::error::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateField {
    Read,
    Starred,
    ReadLater,
}

impl StateField {
    pub fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Starred => "starred",
            Self::ReadLater => "read_later",
        }
    }
    pub fn flag(self, state: &mut ArticleState) -> &mut Flag {
        match self {
            Self::Read => &mut state.read,
            Self::Starred => &mut state.starred,
            Self::ReadLater => &mut state.read_later,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "field", rename_all = "snake_case", deny_unknown_fields)]
pub enum SubscriptionField {
    Title {
        display_title: String,
        custom_title: bool,
    },
    Folder {
        folder_id: Option<String>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialData {
    pub subscriptions: Subscriptions,
    pub articles: BTreeMap<String, CatalogEntry>,
    pub states: BTreeMap<String, ArticleState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Intent {
    CreateFolder {
        folder_id: String,
        name: String,
    },
    RenameFolder {
        folder_id: String,
        name: String,
        base_version: Option<String>,
    },
    DeleteFolder {
        folder_id: String,
        base_version: Option<String>,
    },
    ReorderFolders {
        folder_ids: Vec<String>,
        base_version: Option<String>,
    },
    Subscribe {
        subscription: Subscription,
    },
    SetSubscriptionField {
        feed_key: String,
        generation: u64,
        value: SubscriptionField,
        base_version: Option<String>,
    },
    Unsubscribe {
        feed_key: String,
        generation: u64,
        base_version: Option<String>,
    },
    Resubscribe {
        subscription: Subscription,
        observed_generation: u64,
    },
    EnsureArticle {
        entry: CatalogEntry,
        generation: u64,
    },
    SetArticleState {
        entry: CatalogEntry,
        field: StateField,
        value: bool,
        base_version: Option<String>,
    },
    SeedInitial {
        data: InitialData,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub dataset_id: String,
    pub device_id: String,
    pub seq: u64,
    pub intent: Intent,
}

impl Operation {
    pub fn id(&self) -> String {
        format!("{}:{}", self.device_id, self.seq)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeReport {
    pub snapshot: Snapshot,
    pub processed: usize,
    pub rejected: Vec<Rejection>,
    pub concurrent_overwrites: usize,
}

/// Already-processed prefixes are skipped. Any sequence hole aborts the batch.
pub fn merge(
    remote: &Snapshot,
    operations: &[Operation],
    cutoff_at: Option<&str>,
) -> Result<MergeReport, CoreError> {
    remote.validate()?;
    if operations.len() > BATCH_SIZE {
        return Err(error("githubCapacityExceeded"));
    }
    let mut report = MergeReport {
        snapshot: remote.clone(),
        processed: 0,
        rejected: Vec::new(),
        concurrent_overwrites: 0,
    };
    if let Some(first) = operations.first() {
        let initial_seq = remote
            .manifest
            .devices
            .get(&first.device_id)
            .map_or(0, |d| d.processed_seq);
        let mut processed_seq = initial_seq;
        let mut previous = None;
        for op in operations {
            if op.dataset_id != remote.manifest.dataset_id
                || op.device_id != first.device_id
                || !is_hex_id(&op.device_id, 32)
                || op.seq == 0
                || op.seq > MAX_SEQUENCE
                || previous.is_some_and(|seq| op.seq <= seq)
            {
                return Err(error("githubInvalidOperation"));
            }
            previous = Some(op.seq);
            if op.seq <= processed_seq {
                continue;
            }
            if op.seq != processed_seq + 1 {
                return Err(error("githubSequenceGap"));
            }
            let mut overwrites = 0;
            // Scalar intents validate before mutation. Only the multi-entity
            // initial seed needs a temporary snapshot for operation rollback.
            let result = if matches!(op.intent, Intent::SeedInitial { .. }) {
                let mut next = report.snapshot.clone();
                let result = apply(&mut next, &op.intent, &op.id(), &mut overwrites);
                if result.is_ok() {
                    report.snapshot = next;
                }
                result
            } else {
                apply(&mut report.snapshot, &op.intent, &op.id(), &mut overwrites)
            };
            match result {
                Ok(()) => {
                    report.concurrent_overwrites += overwrites;
                    if let Intent::SeedInitial { data } = &op.intent {
                        if data.subscriptions.feeds.values().any(|feed| {
                            feed.active
                                && remote
                                    .subscriptions
                                    .feeds
                                    .get(&feed.feed_key)
                                    .is_some_and(|old| !old.active)
                        }) {
                            // The seed preserves saved states, but cannot revive
                            // subscriptions cancelled in the existing dataset.
                            report.rejected.push(Rejection {
                                seq: op.seq,
                                code: "githubInitialSubscriptionInactive".into(),
                            });
                        }
                    }
                }
                Err(code) => report.rejected.push(Rejection {
                    seq: op.seq,
                    code: code.into(),
                }),
            }
            processed_seq = op.seq;
            report.processed += 1;
        }
        if processed_seq > initial_seq {
            report.snapshot.manifest.devices.insert(
                first.device_id.clone(),
                DeviceWatermark {
                    processed_seq,
                    last_batch_id: Some(format!("{}:{processed_seq}", first.device_id)),
                    last_rejections: report.rejected.clone(),
                },
            );
        }
    }
    if let Some(cutoff) = cutoff_at {
        let new_cutoff = parse_date(cutoff)?;
        let trusted_now = new_cutoff + chrono::Duration::days(90);
        // Local fetch clocks do not define first cloud confirmation time.
        for (key, entry) in &mut report.snapshot.articles {
            if !remote.articles.contains_key(key) || parse_date(&entry.first_seen_at)? > trusted_now
            {
                entry.first_seen_at = trusted_now.to_rfc3339();
            }
        }
        let previous_cutoff = report.snapshot.manifest.retention.cutoff_at.clone();
        if new_cutoff > parse_date(&report.snapshot.manifest.retention.cutoff_at)? {
            report.snapshot.manifest.retention.cutoff_at = cutoff.into();
        }
        let cutoff = parse_date(&report.snapshot.manifest.retention.cutoff_at)?;
        let mut expired = Vec::new();
        for (key, entry) in &report.snapshot.articles {
            let protected = report
                .snapshot
                .states
                .get(key)
                .is_some_and(ArticleState::protected);
            let mut date = parse_date(
                entry
                    .published_at
                    .as_deref()
                    .unwrap_or(&entry.first_seen_at),
            )?;
            if date > trusted_now {
                date = parse_date(&entry.first_seen_at)?;
            }
            if !protected && date < cutoff {
                expired.push(key.clone());
            }
        }
        if expired.is_empty() && report.processed == 0 {
            report.snapshot.manifest.retention.cutoff_at = previous_cutoff;
        }
        for key in expired {
            report.snapshot.articles.remove(&key);
            report.snapshot.states.remove(&key);
        }
    }
    report.snapshot.validate()?;
    Ok(report)
}

type ApplyResult = Result<(), &'static str>;

fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 256
}

fn folder_target(
    snapshot: &Snapshot,
    folder: &Option<String>,
) -> Result<Option<String>, &'static str> {
    folder
        .as_deref()
        .map(|id| {
            let id = snapshot
                .resolve_folder(id)
                .map_err(|_| "githubUnknownFolder")?;
            Ok(if snapshot.subscriptions.folders[&id].deleted {
                None
            } else {
                Some(id)
            })
        })
        .transpose()
        .map(Option::flatten)
}

fn overwritten(current: Option<&str>, base: Option<&str>, count: &mut usize) {
    if current != base {
        *count += 1;
    }
}

fn ensure_entry(snapshot: &mut Snapshot, entry: &CatalogEntry) -> Result<String, &'static str> {
    entry.validate().map_err(|_| "githubInvalidArticle")?;
    if !snapshot.subscriptions.feeds.contains_key(&entry.feed_key) {
        return Err("githubUnknownSubscription");
    }
    let own_key = entry.key();
    // Only a proven URL-fallback identity can acquire a GUID alias. Two real
    // GUIDs sharing a URL remain separate articles.
    let aliases: Vec<String> =
        if entry.identity_kind == IdentityKind::Guid && !snapshot.articles.contains_key(&own_key) {
            entry
                .url
                .as_deref()
                .and_then(|url| canonical_url(url).ok())
                .map(|url| {
                    snapshot
                        .articles
                        .iter()
                        .filter(|(_, old)| {
                            old.feed_key == entry.feed_key
                                && old.identity_kind == IdentityKind::Url
                                && old.identity_value == url
                                && (old.guid.is_empty() || old.guid == entry.guid)
                        })
                        .map(|(key, _)| key.clone())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
    let key = if aliases.len() == 1 {
        aliases[0].clone()
    } else {
        own_key
    };
    let old = snapshot
        .articles
        .entry(key.clone())
        .or_insert_with(|| entry.clone());
    if old.url.is_none() {
        old.url = entry.url.clone();
    }
    if old.published_at.is_none() {
        old.published_at = entry.published_at.clone();
    }
    if old.identity_kind == IdentityKind::Url && old.guid.is_empty() {
        old.guid = entry.guid.clone();
    }
    Ok(key)
}

fn live_feed<'a>(
    snapshot: &'a mut Snapshot,
    key: &str,
    generation: u64,
) -> Result<&'a mut Subscription, &'static str> {
    let feed = snapshot
        .subscriptions
        .feeds
        .get_mut(key)
        .ok_or("githubUnknownSubscription")?;
    if feed.generation != generation {
        return Err("githubStaleSubscriptionGeneration");
    }
    if !feed.active {
        return Err("githubSubscriptionInactive");
    }
    Ok(feed)
}

fn validate_subscription(feed: &Subscription) -> ApplyResult {
    if feed_key(&feed.feed_url).map_err(|_| "githubInvalidSource")? != feed.feed_key
        || feed.generation == 0
        || feed.generation > MAX_SEQUENCE
        || feed.display_title.len() > 4096
        || !matches!(
            feed.source_type.as_str(),
            "rss" | "youtube" | "podcast" | "mastodon" | "bluesky" | "reddit"
        )
    {
        return Err("githubInvalidSource");
    }
    Ok(())
}

fn apply(
    snapshot: &mut Snapshot,
    intent: &Intent,
    version: &str,
    overwrites: &mut usize,
) -> ApplyResult {
    match intent {
        Intent::CreateFolder { folder_id, name } => {
            if !is_hex_id(folder_id, 32) || !valid_name(name) {
                return Err("githubInvalidFolder");
            }
            if snapshot.subscriptions.folders.contains_key(folder_id)
                || snapshot
                    .subscriptions
                    .folder_aliases
                    .contains_key(folder_id)
            {
                return Err("githubFolderAlreadyExists");
            }
            if let Some(existing) = snapshot.subscriptions.folders.values().find(|folder| {
                !folder.deleted && folder.name.trim().eq_ignore_ascii_case(name.trim())
            }) {
                snapshot
                    .subscriptions
                    .folder_aliases
                    .insert(folder_id.clone(), existing.folder_id.clone());
            } else {
                snapshot.subscriptions.folders.insert(
                    folder_id.clone(),
                    Folder {
                        folder_id: folder_id.clone(),
                        name: name.trim().into(),
                        deleted: false,
                        name_version: Some(version.into()),
                    },
                );
                snapshot.subscriptions.folder_order.push(folder_id.clone());
            }
        }
        Intent::RenameFolder {
            folder_id,
            name,
            base_version,
        } => {
            if !valid_name(name) {
                return Err("githubInvalidFolder");
            }
            let id = snapshot
                .resolve_folder(folder_id)
                .map_err(|_| "githubUnknownFolder")?;
            if snapshot.subscriptions.folders[&id].deleted {
                return Err("githubFolderDeleted");
            }
            if snapshot.subscriptions.folders.values().any(|f| {
                f.folder_id != id && !f.deleted && f.name.trim().eq_ignore_ascii_case(name.trim())
            }) {
                return Err("folderNameExists");
            }
            let folder = snapshot
                .subscriptions
                .folders
                .get_mut(&id)
                .ok_or("githubUnknownFolder")?;
            overwritten(
                folder.name_version.as_deref(),
                base_version.as_deref(),
                overwrites,
            );
            folder.name = name.trim().into();
            folder.name_version = Some(version.into());
        }
        Intent::DeleteFolder {
            folder_id,
            base_version,
        } => {
            let id = snapshot
                .resolve_folder(folder_id)
                .map_err(|_| "githubUnknownFolder")?;
            let aliases: BTreeSet<_> = snapshot
                .subscriptions
                .folder_aliases
                .keys()
                .filter(|alias| {
                    snapshot
                        .resolve_folder(alias)
                        .is_ok_and(|target| target == id)
                })
                .cloned()
                .chain(std::iter::once(id.clone()))
                .collect();
            let folder = snapshot
                .subscriptions
                .folders
                .get_mut(&id)
                .ok_or("githubUnknownFolder")?;
            overwritten(
                folder.name_version.as_deref(),
                base_version.as_deref(),
                overwrites,
            );
            folder.deleted = true;
            folder.name_version = Some(version.into());
            snapshot
                .subscriptions
                .folder_order
                .retain(|folder| folder != &id);
            for feed in snapshot.subscriptions.feeds.values_mut() {
                if feed
                    .folder_id
                    .as_ref()
                    .is_some_and(|folder| aliases.contains(folder))
                {
                    feed.folder_id = None;
                    feed.field_versions
                        .insert("folder_id".into(), version.into());
                }
            }
        }
        Intent::ReorderFolders {
            folder_ids,
            base_version,
        } => {
            overwritten(
                snapshot.subscriptions.order_version.as_deref(),
                base_version.as_deref(),
                overwrites,
            );
            let mut order = Vec::new();
            let mut seen = BTreeSet::new();
            for id in folder_ids {
                let id = snapshot
                    .resolve_folder(id)
                    .map_err(|_| "githubUnknownFolder")?;
                if !snapshot.subscriptions.folders[&id].deleted && seen.insert(id.clone()) {
                    order.push(id);
                }
            }
            for id in &snapshot.subscriptions.folder_order {
                if seen.insert(id.clone()) {
                    order.push(id.clone());
                }
            }
            snapshot.subscriptions.folder_order = order;
            snapshot.subscriptions.order_version = Some(version.into());
        }
        Intent::Subscribe { subscription } => {
            validate_subscription(subscription)?;
            if !subscription.active || subscription.generation != 1 {
                return Err("githubInvalidSource");
            }
            if let Some(old) = snapshot.subscriptions.feeds.get(&subscription.feed_key) {
                if !old.active {
                    return Err("githubSubscriptionInactive");
                }
                return Ok(());
            }
            let mut feed = subscription.clone();
            feed.folder_id = folder_target(snapshot, &feed.folder_id)?;
            feed.field_versions = BTreeMap::from([
                ("active".into(), version.into()),
                ("title".into(), version.into()),
                ("folder_id".into(), version.into()),
            ]);
            snapshot
                .subscriptions
                .feeds
                .insert(feed.feed_key.clone(), feed);
        }
        Intent::SetSubscriptionField {
            feed_key,
            generation,
            value,
            base_version,
        } => {
            let (field, folder) = match value {
                SubscriptionField::Title { display_title, .. } => {
                    if display_title.trim().is_empty() || display_title.len() > 4096 {
                        return Err("emptyFeedTitle");
                    }
                    ("title", None)
                }
                SubscriptionField::Folder { folder_id } => {
                    ("folder_id", folder_target(snapshot, folder_id)?)
                }
            };
            let feed = live_feed(snapshot, feed_key, *generation)?;
            overwritten(
                feed.field_versions.get(field).map(String::as_str),
                base_version.as_deref(),
                overwrites,
            );
            match value {
                SubscriptionField::Title {
                    display_title,
                    custom_title,
                } => {
                    feed.display_title = display_title.clone();
                    feed.custom_title = *custom_title;
                }
                SubscriptionField::Folder { .. } => feed.folder_id = folder,
            }
            feed.field_versions.insert(field.into(), version.into());
        }
        Intent::Unsubscribe {
            feed_key,
            generation,
            base_version,
        } => {
            let feed = snapshot
                .subscriptions
                .feeds
                .get_mut(feed_key)
                .ok_or("githubUnknownSubscription")?;
            if feed.generation != *generation {
                return Err("githubStaleSubscriptionGeneration");
            }
            overwritten(
                feed.field_versions.get("active").map(String::as_str),
                base_version.as_deref(),
                overwrites,
            );
            feed.active = false;
            feed.field_versions.insert("active".into(), version.into());
        }
        Intent::Resubscribe {
            subscription,
            observed_generation,
        } => {
            validate_subscription(subscription)?;
            let old = snapshot
                .subscriptions
                .feeds
                .get(&subscription.feed_key)
                .ok_or("githubUnknownSubscription")?;
            if *observed_generation >= MAX_SEQUENCE {
                return Err("githubInvalidSource");
            }
            if old.active && old.generation == observed_generation + 1 {
                return Ok(());
            }
            if old.active || old.generation != *observed_generation {
                return Err("githubStaleSubscriptionGeneration");
            }
            let mut feed = subscription.clone();
            feed.generation = observed_generation + 1;
            feed.active = true;
            feed.folder_id = folder_target(snapshot, &feed.folder_id)?;
            feed.field_versions = BTreeMap::from([
                ("active".into(), version.into()),
                ("title".into(), version.into()),
                ("folder_id".into(), version.into()),
            ]);
            snapshot
                .subscriptions
                .feeds
                .insert(feed.feed_key.clone(), feed);
        }
        Intent::EnsureArticle { entry, generation } => {
            live_feed(snapshot, &entry.feed_key, *generation)?;
            ensure_entry(snapshot, entry)?;
        }
        Intent::SetArticleState {
            entry,
            field,
            value,
            base_version,
        } => {
            let key = ensure_entry(snapshot, entry)?;
            let state = snapshot.states.entry(key).or_default();
            let flag = field.flag(state);
            overwritten(flag.version.as_deref(), base_version.as_deref(), overwrites);
            *flag = Flag {
                value: *value,
                version: Some(version.into()),
            };
        }
        Intent::SeedInitial { data } => {
            for id in &data.subscriptions.folder_order {
                let folder = data
                    .subscriptions
                    .folders
                    .get(id)
                    .ok_or("githubInvalidFolder")?;
                if snapshot
                    .subscriptions
                    .folders
                    .contains_key(&folder.folder_id)
                    || snapshot
                        .subscriptions
                        .folder_aliases
                        .contains_key(&folder.folder_id)
                {
                    continue;
                }
                apply(
                    snapshot,
                    &Intent::CreateFolder {
                        folder_id: folder.folder_id.clone(),
                        name: folder.name.clone(),
                    },
                    version,
                    overwrites,
                )?;
            }
            for source in data.subscriptions.feeds.values() {
                validate_subscription(source)?;
                if !snapshot.subscriptions.feeds.contains_key(&source.feed_key) {
                    let mut source = source.clone();
                    source.folder_id = folder_target(snapshot, &source.folder_id)?;
                    source.generation = 1;
                    source.field_versions = BTreeMap::from([
                        ("active".into(), version.into()),
                        ("title".into(), version.into()),
                        ("folder_id".into(), version.into()),
                    ]);
                    snapshot
                        .subscriptions
                        .feeds
                        .insert(source.feed_key.clone(), source);
                }
            }
            for (key, entry) in &data.articles {
                if &entry.key() != key {
                    return Err("githubInvalidArticle");
                }
                let resolved_key = ensure_entry(snapshot, entry)?;
                if let Some(local) = data.states.get(key) {
                    let state = snapshot.states.entry(resolved_key).or_default();
                    for field in [StateField::Read, StateField::Starred, StateField::ReadLater] {
                        let mut local = local.clone();
                        if field.flag(&mut local).value && !field.flag(state).value {
                            *field.flag(state) = Flag {
                                value: true,
                                version: Some(version.into()),
                            };
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(device: char, seq: u64, intent: Intent) -> Operation {
        Operation {
            dataset_id: "d".repeat(32),
            device_id: device.to_string().repeat(32),
            seq,
            intent,
        }
    }
    fn feed() -> Subscription {
        let url = "https://example.com/feed";
        Subscription {
            feed_key: feed_key(url).unwrap(),
            feed_url: url.into(),
            source_type: "rss".into(),
            display_title: "Feed".into(),
            custom_title: false,
            folder_id: None,
            active: true,
            generation: 1,
            field_versions: BTreeMap::new(),
        }
    }
    fn entry() -> CatalogEntry {
        CatalogEntry {
            feed_key: feed().feed_key,
            identity_kind: IdentityKind::Guid,
            identity_value: "item".into(),
            guid: "item".into(),
            title: "Article".into(),
            url: Some("https://example.com/article".into()),
            published_at: Some("2000-01-01T00:00:00Z".into()),
            first_seen_at: "2026-10-05T00:00:00Z".into(),
        }
    }
    fn base() -> Snapshot {
        let remote = Snapshot::empty("d".repeat(32), "1970-01-01T00:00:00Z".into());
        merge(
            &remote,
            &[
                op(
                    'a',
                    1,
                    Intent::Subscribe {
                        subscription: feed(),
                    },
                ),
                op(
                    'a',
                    2,
                    Intent::EnsureArticle {
                        entry: entry(),
                        generation: 1,
                    },
                ),
            ],
            None,
        )
        .unwrap()
        .snapshot
    }
    fn state_op(device: char, seq: u64, field: StateField, value: bool) -> Operation {
        op(
            device,
            seq,
            Intent::SetArticleState {
                entry: entry(),
                field,
                value,
                base_version: None,
            },
        )
    }

    #[test]
    fn independent_fields_merge_and_explicit_false_survives_retry() {
        let a = state_op('a', 3, StateField::Starred, true);
        let b = state_op('b', 1, StateField::ReadLater, true);
        let remote = merge(&base(), &[a], None).unwrap().snapshot;
        let remote = merge(&remote, &[b], None).unwrap().snapshot;
        let off = state_op('a', 4, StateField::Starred, false);
        let remote = merge(&remote, &[off.clone()], None).unwrap().snapshot;
        let state = &remote.states[&entry().key()];
        assert!(!state.starred.value);
        assert!(state.read_later.value);
        assert_eq!(merge(&remote, &[off], None).unwrap().snapshot, remote);
        assert_eq!(
            Snapshot::from_files(&remote.files().unwrap()).unwrap(),
            remote
        );
    }

    #[test]
    fn sequence_gap_does_not_mutate_remote() {
        let remote = base();
        assert_eq!(
            merge(
                &remote,
                &[state_op('a', 4, StateField::Starred, true)],
                None
            )
            .unwrap_err()
            .code(),
            "githubSequenceGap"
        );
        assert!(remote.states.is_empty());
    }

    #[test]
    fn retention_keeps_old_saved_items_and_releases_unprotected_items() {
        let saved = merge(
            &base(),
            &[state_op('a', 3, StateField::Starred, true)],
            Some("2026-07-07T00:00:00Z"),
        )
        .unwrap()
        .snapshot;
        assert!(saved.articles.contains_key(&entry().key()));
        let cleared = merge(
            &saved,
            &[state_op('a', 4, StateField::Starred, false)],
            Some("2026-07-07T00:00:00Z"),
        )
        .unwrap()
        .snapshot;
        assert!(cleared.articles.is_empty());
        assert!(cleared.states.is_empty());
        let restored = merge(
            &cleared,
            &[state_op('b', 1, StateField::ReadLater, true)],
            Some("2026-07-07T00:00:00Z"),
        )
        .unwrap()
        .snapshot;
        assert!(restored.articles.contains_key(&entry().key()));
    }

    #[test]
    fn tombstone_and_generation_stop_offline_resurrection() {
        let key = feed().feed_key;
        let deleted = merge(
            &base(),
            &[op(
                'a',
                3,
                Intent::Unsubscribe {
                    feed_key: key.clone(),
                    generation: 1,
                    base_version: None,
                },
            )],
            None,
        )
        .unwrap()
        .snapshot;
        let rejected = merge(
            &deleted,
            &[op(
                'b',
                1,
                Intent::Subscribe {
                    subscription: feed(),
                },
            )],
            None,
        )
        .unwrap();
        assert_eq!(rejected.rejected[0].code, "githubSubscriptionInactive");
        assert!(!rejected.snapshot.subscriptions.feeds[&key].active);
        let restored = merge(
            &rejected.snapshot,
            &[op(
                'a',
                4,
                Intent::Resubscribe {
                    subscription: feed(),
                    observed_generation: 1,
                },
            )],
            None,
        )
        .unwrap()
        .snapshot;
        let stale = merge(
            &restored,
            &[op(
                'b',
                2,
                Intent::Unsubscribe {
                    feed_key: key.clone(),
                    generation: 1,
                    base_version: None,
                },
            )],
            None,
        )
        .unwrap();
        assert!(stale.snapshot.subscriptions.feeds[&key].active);
        assert_eq!(stale.snapshot.subscriptions.feeds[&key].generation, 2);
        assert_eq!(stale.rejected[0].code, "githubStaleSubscriptionGeneration");
    }

    #[test]
    fn same_name_creation_aliases_but_rename_collision_is_rejected() {
        let first = "1".repeat(32);
        let second = "2".repeat(32);
        let third = "3".repeat(32);
        let remote = merge(
            &base(),
            &[
                op(
                    'a',
                    3,
                    Intent::CreateFolder {
                        folder_id: first.clone(),
                        name: "News".into(),
                    },
                ),
                op(
                    'a',
                    4,
                    Intent::CreateFolder {
                        folder_id: third.clone(),
                        name: "Other".into(),
                    },
                ),
            ],
            None,
        )
        .unwrap()
        .snapshot;
        let remote = merge(
            &remote,
            &[op(
                'b',
                1,
                Intent::CreateFolder {
                    folder_id: second.clone(),
                    name: " news ".into(),
                },
            )],
            None,
        )
        .unwrap()
        .snapshot;
        assert_eq!(remote.resolve_folder(&second).unwrap(), first);
        let report = merge(
            &remote,
            &[op(
                'b',
                2,
                Intent::RenameFolder {
                    folder_id: third.clone(),
                    name: "NEWS".into(),
                    base_version: None,
                },
            )],
            None,
        )
        .unwrap();
        assert_eq!(report.rejected[0].code, "folderNameExists");
        assert_eq!(report.snapshot.subscriptions.folders[&third].name, "Other");
    }

    #[test]
    fn lost_response_retry_does_not_overwrite_another_devices_later_edit() {
        let off = state_op('a', 3, StateField::Starred, false);
        let accepted = merge(&base(), &[off.clone()], None).unwrap().snapshot;
        let later = merge(
            &accepted,
            &[state_op('b', 1, StateField::Starred, true)],
            None,
        )
        .unwrap()
        .snapshot;
        let retried = merge(&later, &[off], None).unwrap();
        assert_eq!(retried.processed, 0);
        assert_eq!(retried.snapshot, later);
        assert!(retried.snapshot.states[&entry().key()].starred.value);
    }

    #[test]
    fn folder_delete_unclassifies_feeds_and_stale_move_cannot_restore_it() {
        let id = "1".repeat(32);
        let key = feed().feed_key;
        let remote = merge(
            &base(),
            &[
                op(
                    'a',
                    3,
                    Intent::CreateFolder {
                        folder_id: id.clone(),
                        name: "News".into(),
                    },
                ),
                op(
                    'a',
                    4,
                    Intent::SetSubscriptionField {
                        feed_key: key.clone(),
                        generation: 1,
                        value: SubscriptionField::Folder {
                            folder_id: Some(id.clone()),
                        },
                        base_version: None,
                    },
                ),
                op(
                    'a',
                    5,
                    Intent::DeleteFolder {
                        folder_id: id.clone(),
                        base_version: None,
                    },
                ),
            ],
            None,
        )
        .unwrap()
        .snapshot;
        assert_eq!(remote.subscriptions.feeds[&key].folder_id, None);
        let stale = merge(
            &remote,
            &[op(
                'b',
                1,
                Intent::SetSubscriptionField {
                    feed_key: key.clone(),
                    generation: 1,
                    value: SubscriptionField::Folder {
                        folder_id: Some(id),
                    },
                    base_version: None,
                },
            )],
            None,
        )
        .unwrap()
        .snapshot;
        assert_eq!(stale.subscriptions.feeds[&key].folder_id, None);
        assert!(stale.subscriptions.folder_order.is_empty());
    }

    #[test]
    fn initial_merge_preserves_saved_states_without_reviving_cancelled_sources() {
        let key = feed().feed_key;
        let remote = merge(
            &base(),
            &[
                state_op('a', 3, StateField::ReadLater, true),
                op(
                    'a',
                    4,
                    Intent::Unsubscribe {
                        feed_key: key.clone(),
                        generation: 1,
                        base_version: None,
                    },
                ),
            ],
            None,
        )
        .unwrap()
        .snapshot;
        let mut data = InitialData::default();
        data.subscriptions.feeds.insert(key.clone(), feed());
        data.articles.insert(entry().key(), entry());
        data.states.insert(
            entry().key(),
            ArticleState {
                starred: Flag {
                    value: true,
                    version: None,
                },
                ..ArticleState::default()
            },
        );
        let report = merge(&remote, &[op('b', 1, Intent::SeedInitial { data })], None).unwrap();
        let state = &report.snapshot.states[&entry().key()];
        assert!(state.starred.value && state.read_later.value);
        assert!(!report.snapshot.subscriptions.feeds[&key].active);
        assert_eq!(report.rejected[0].code, "githubInitialSubscriptionInactive");
    }

    #[test]
    fn invalid_initial_seed_rolls_back_entities_but_records_a_terminal_rejection() {
        let id = "1".repeat(32);
        let mut data = InitialData::default();
        data.subscriptions.folder_order.push(id.clone());
        data.subscriptions.folders.insert(
            id.clone(),
            Folder {
                folder_id: id,
                name: "New folder".into(),
                deleted: false,
                name_version: None,
            },
        );
        let mut invalid = feed();
        invalid.feed_url = "https://user:password@example.com/feed".into();
        data.subscriptions
            .feeds
            .insert(invalid.feed_key.clone(), invalid);
        let report = merge(&base(), &[op('b', 1, Intent::SeedInitial { data })], None).unwrap();
        assert!(report.snapshot.subscriptions.folders.is_empty());
        assert_eq!(report.rejected[0].code, "githubInvalidSource");
        assert_eq!(
            report.snapshot.manifest.devices[&"b".repeat(32)].processed_seq,
            1
        );
    }

    #[test]
    fn a_new_maintenance_clock_without_changes_does_not_create_a_commit() {
        let remote = Snapshot::empty("d".repeat(32), "2026-07-07T00:00:00Z".into());
        assert_eq!(
            merge(&remote, &[], Some("2026-07-08T00:00:00Z"))
                .unwrap()
                .snapshot,
            remote
        );
    }

    #[test]
    fn proven_url_fallback_gains_guid_without_merging_two_real_guids() {
        let mut fallback = entry();
        fallback.guid.clear();
        fallback.identity_kind = IdentityKind::Url;
        fallback.identity_value = fallback.url.clone().unwrap();
        let mut snapshot = base();
        snapshot.articles.clear();
        snapshot.states.clear();
        let saved = op(
            'b',
            1,
            Intent::SetArticleState {
                entry: fallback.clone(),
                field: StateField::Starred,
                value: true,
                base_version: None,
            },
        );
        snapshot = merge(&snapshot, &[saved], None).unwrap().snapshot;
        let real = entry();
        snapshot = merge(
            &snapshot,
            &[op(
                'b',
                2,
                Intent::EnsureArticle {
                    entry: real.clone(),
                    generation: 1,
                },
            )],
            None,
        )
        .unwrap()
        .snapshot;
        assert_eq!(snapshot.articles.len(), 1);
        assert_eq!(snapshot.articles[&fallback.key()].guid, real.guid);
        assert!(snapshot.states[&fallback.key()].starred.value);
        let mut other = real.clone();
        other.guid = "different-real-guid".into();
        other.identity_value = other.guid.clone();
        snapshot = merge(
            &snapshot,
            &[op(
                'b',
                3,
                Intent::EnsureArticle {
                    entry: other,
                    generation: 1,
                },
            )],
            None,
        )
        .unwrap()
        .snapshot;
        assert_eq!(snapshot.articles.len(), 2);
    }

    #[test]
    fn future_dates_use_first_cloud_confirmation_and_do_not_extend_on_refetch() {
        let mut first = entry();
        first.published_at = Some("2099-01-01T00:00:00Z".into());
        first.first_seen_at = "1900-01-01T00:00:00Z".into();
        let key = first.key();
        let now = "2026-10-05T00:00:00+00:00";
        let cutoff = "2026-07-07T00:00:00Z";
        let snapshot = merge(
            &Snapshot::empty("d".repeat(32), "1970-01-01T00:00:00Z".into()),
            &[
                op(
                    'a',
                    1,
                    Intent::Subscribe {
                        subscription: feed(),
                    },
                ),
                op(
                    'a',
                    2,
                    Intent::EnsureArticle {
                        entry: first.clone(),
                        generation: 1,
                    },
                ),
            ],
            Some(cutoff),
        )
        .unwrap()
        .snapshot;
        assert_eq!(snapshot.articles[&key].first_seen_at, now);
        first.first_seen_at = "2099-02-01T00:00:00Z".into();
        let refreshed = merge(
            &snapshot,
            &[op(
                'a',
                3,
                Intent::EnsureArticle {
                    entry: first,
                    generation: 1,
                },
            )],
            Some("2026-07-08T00:00:00Z"),
        )
        .unwrap()
        .snapshot;
        assert_eq!(refreshed.articles[&key].first_seen_at, now);
        assert!(merge(&refreshed, &[], Some("2026-10-06T00:00:00Z"))
            .unwrap()
            .snapshot
            .articles
            .is_empty());
        let saved = merge(
            &snapshot,
            &[state_op('a', 3, StateField::Starred, true)],
            None,
        )
        .unwrap()
        .snapshot;
        assert!(
            merge(&saved, &[], Some("2026-10-06T00:00:00Z"))
                .unwrap()
                .snapshot
                .states[&key]
                .starred
                .value
        );
    }
}
