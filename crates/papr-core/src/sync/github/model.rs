use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;

use chrono::DateTime;
use serde::{de, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use super::error;
use crate::error::CoreError;

pub const PREFIX: &str = "papr-sync/v1";
pub const SHARD_COUNT: u8 = 64;
pub const MAX_SEQUENCE: u64 = (1 << 53) - 1;
pub const MAX_FILE_BYTES: usize = 1024 * 1024;
pub const MAX_SNAPSHOT_BYTES: usize = 50 * 1024 * 1024;
pub const BATCH_SIZE: usize = 500;

pub fn is_hex_id(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Length prefixes are unsigned 32-bit big-endian UTF-8 byte lengths.
pub fn stable_key(parts: &[&str]) -> String {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u32).to_be_bytes());
        hash.update(part.as_bytes());
    }
    format!("{:x}", hash.finalize())
}

pub fn canonical_url(input: &str) -> Result<String, CoreError> {
    let mut url = Url::parse(input.trim()).map_err(|_| error("githubInvalidSource"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || input.len() > 8192
    {
        return Err(error("githubInvalidSource"));
    }
    url.set_fragment(None);
    Ok(url.to_string())
}

pub fn feed_key(url: &str) -> Result<String, CoreError> {
    Ok(stable_key(&["feed:v1", &canonical_url(url)?]))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdentityKind {
    Guid,
    Url,
}

pub fn article_key(feed: &str, kind: IdentityKind, value: &str) -> String {
    stable_key(&[
        "article:v1",
        feed,
        match kind {
            IdentityKind::Guid => "guid",
            IdentityKind::Url => "url",
        },
        value,
    ])
}

pub fn shard(key: &str) -> Result<String, CoreError> {
    if !is_hex_id(key, 64) {
        return Err(error("githubInvalidData"));
    }
    let first = u8::from_str_radix(&key[..2], 16).map_err(|_| error("githubInvalidData"))?;
    Ok(format!("{:02x}", first & 63))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitHubProfile {
    pub repository_id: u64,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub credential_ref: String,
}

impl GitHubProfile {
    pub fn validate(&self) -> Result<(), CoreError> {
        let valid_name = |name: &str| {
            !name.is_empty()
                && name.len() <= 100
                && name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
                && !matches!(name, "." | "..")
        };
        let branch = &self.branch;
        let valid_branch = !branch.is_empty()
            && branch.len() <= 255
            && !branch.starts_with('/')
            && !branch.ends_with('/')
            && !branch.ends_with('.')
            && !branch.contains("..")
            && !branch.contains("@{")
            && !branch.contains("//")
            && branch != "@"
            && !branch
                .bytes()
                .any(|b| b <= 32 || b == 127 || b"~^:?*[\\".contains(&b))
            && branch
                .split('/')
                .all(|part| !part.starts_with('.') && !part.ends_with(".lock"));
        let valid_ref = self
            .credential_ref
            .strip_prefix("papr.sync.")
            .is_some_and(|s| {
                !s.is_empty()
                    && s.len() <= 80
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
            });
        if self.repository_id == 0
            || self.repository_id > MAX_SEQUENCE
            || !valid_name(&self.owner)
            || !valid_name(&self.repo)
            || !valid_branch
            || !valid_ref
        {
            return Err(error("githubInvalidProfile"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retention {
    pub ordinary_days: u16,
    pub protected_forever: bool,
    pub cutoff_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rejection {
    pub seq: u64,
    pub code: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceWatermark {
    pub processed_seq: u64,
    pub last_batch_id: Option<String>,
    pub last_rejections: Vec<Rejection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub protocol_version: u16,
    pub dataset_id: String,
    pub epoch: u64,
    pub shard_count: u8,
    pub retention: Retention,
    #[serde(deserialize_with = "unique_map")]
    pub devices: BTreeMap<String, DeviceWatermark>,
    /// Wire-only inventory of every non-manifest protocol file. Parsed snapshots
    /// normalize this to empty; serialization derives it from the current data.
    #[serde(deserialize_with = "unique_map")]
    pub file_hashes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Folder {
    pub folder_id: String,
    pub name: String,
    pub deleted: bool,
    pub name_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subscription {
    pub feed_key: String,
    pub feed_url: String,
    pub source_type: String,
    pub display_title: String,
    pub custom_title: bool,
    pub folder_id: Option<String>,
    pub active: bool,
    pub generation: u64,
    #[serde(deserialize_with = "unique_map")]
    pub field_versions: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subscriptions {
    #[serde(deserialize_with = "unique_map")]
    pub folders: BTreeMap<String, Folder>,
    #[serde(deserialize_with = "unique_map")]
    pub feeds: BTreeMap<String, Subscription>,
    pub folder_order: Vec<String>,
    pub order_version: Option<String>,
    #[serde(deserialize_with = "unique_map")]
    pub folder_aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntry {
    pub feed_key: String,
    pub identity_kind: IdentityKind,
    pub identity_value: String,
    pub guid: String,
    pub title: String,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub first_seen_at: String,
}

impl CatalogEntry {
    pub fn key(&self) -> String {
        article_key(&self.feed_key, self.identity_kind, &self.identity_value)
    }
    pub fn validate(&self) -> Result<(), CoreError> {
        if !is_hex_id(&self.feed_key, 64)
            || self.title.is_empty()
            || self.title.len() > 4096
            || self.guid.len() > 8192
            || self.identity_value.is_empty()
            || self.identity_value.len() > 8192
            || (self.identity_kind == IdentityKind::Guid && self.guid != self.identity_value)
        {
            return Err(error("githubInvalidData"));
        }
        if self.identity_kind == IdentityKind::Url
            && canonical_url(&self.identity_value)? != self.identity_value
        {
            return Err(error("githubInvalidData"));
        }
        if let Some(url) = &self.url {
            canonical_url(url)?;
        }
        parse_date(&self.first_seen_at)?;
        if let Some(date) = &self.published_at {
            parse_date(date)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flag {
    pub value: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArticleState {
    #[serde(default)]
    pub read: Flag,
    #[serde(default)]
    pub starred: Flag,
    #[serde(default)]
    pub read_later: Flag,
}

impl ArticleState {
    pub fn protected(&self) -> bool {
        self.starred.value || self.read_later.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub manifest: Manifest,
    pub subscriptions: Subscriptions,
    pub articles: BTreeMap<String, CatalogEntry>,
    pub states: BTreeMap<String, ArticleState>,
}

pub fn parse_date(value: &str) -> Result<DateTime<chrono::FixedOffset>, CoreError> {
    if value.len() > 40 {
        return Err(error("githubInvalidData"));
    }
    DateTime::parse_from_rfc3339(value).map_err(|_| error("githubInvalidData"))
}

impl Snapshot {
    pub fn empty(dataset_id: String, cutoff_at: String) -> Self {
        Self {
            manifest: Manifest {
                protocol_version: 1,
                dataset_id,
                epoch: 1,
                shard_count: SHARD_COUNT,
                retention: Retention {
                    ordinary_days: 90,
                    protected_forever: true,
                    cutoff_at,
                },
                devices: BTreeMap::new(),
                file_hashes: BTreeMap::new(),
            },
            subscriptions: Subscriptions::default(),
            articles: BTreeMap::new(),
            states: BTreeMap::new(),
        }
    }

    pub fn resolve_folder(&self, id: &str) -> Result<String, CoreError> {
        let mut current = id;
        let mut seen = BTreeSet::new();
        while let Some(next) = self.subscriptions.folder_aliases.get(current) {
            if !seen.insert(current) {
                return Err(error("githubInvalidData"));
            }
            current = next;
        }
        if !self.subscriptions.folders.contains_key(current) {
            return Err(error("githubInvalidData"));
        }
        Ok(current.to_string())
    }

    fn validate_version(&self, version: Option<&str>) -> Result<(), CoreError> {
        if let Some(version) = version {
            let (device, seq) = version
                .split_once(':')
                .ok_or_else(|| error("githubInvalidData"))?;
            let seq = seq.parse::<u64>().map_err(|_| error("githubInvalidData"))?;
            if !is_hex_id(device, 32)
                || seq == 0
                || seq > MAX_SEQUENCE
                || self
                    .manifest
                    .devices
                    .get(device)
                    .is_none_or(|watermark| watermark.processed_seq < seq)
                || version != format!("{device}:{seq}")
            {
                return Err(error("githubInvalidData"));
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        let m = &self.manifest;
        if m.protocol_version != 1
            || m.shard_count != SHARD_COUNT
            || m.retention.ordinary_days != 90
            || !m.retention.protected_forever
        {
            return Err(error("githubUnsupportedFormat"));
        }
        if !is_hex_id(&m.dataset_id, 32) || m.epoch == 0 || m.epoch > MAX_SEQUENCE {
            return Err(error("githubInvalidData"));
        }
        parse_date(&m.retention.cutoff_at)?;
        for (id, device) in &m.devices {
            if !is_hex_id(id, 32)
                || device.processed_seq > MAX_SEQUENCE
                || device.last_rejections.len() > BATCH_SIZE
                || device
                    .last_rejections
                    .iter()
                    .any(|r| r.seq == 0 || r.seq > device.processed_seq || r.code.len() > 80)
            {
                return Err(error("githubInvalidData"));
            }
            self.validate_version(device.last_batch_id.as_deref())?;
            if let Some(batch) = &device.last_batch_id {
                if batch != &format!("{id}:{}", device.processed_seq) {
                    return Err(error("githubInvalidData"));
                }
            }
        }
        let mut names = BTreeSet::new();
        for (id, folder) in &self.subscriptions.folders {
            if id != &folder.folder_id
                || !is_hex_id(id, 32)
                || folder.name.trim().is_empty()
                || folder.name.len() > 256
                || (!folder.deleted && !names.insert(folder.name.trim().to_ascii_lowercase()))
            {
                return Err(error("githubInvalidData"));
            }
            self.validate_version(folder.name_version.as_deref())?;
        }
        let expected: BTreeSet<_> = self
            .subscriptions
            .folders
            .iter()
            .filter(|(_, f)| !f.deleted)
            .map(|(id, _)| id)
            .collect();
        let actual: BTreeSet<_> = self.subscriptions.folder_order.iter().collect();
        if expected != actual || actual.len() != self.subscriptions.folder_order.len() {
            return Err(error("githubInvalidData"));
        }
        self.validate_version(self.subscriptions.order_version.as_deref())?;
        for alias in self.subscriptions.folder_aliases.keys() {
            if !is_hex_id(alias, 32) || self.subscriptions.folders.contains_key(alias) {
                return Err(error("githubInvalidData"));
            }
            self.resolve_folder(alias)?;
        }
        for (key, feed) in &self.subscriptions.feeds {
            if key != &feed.feed_key
                || feed_key(&feed.feed_url)? != *key
                || feed.generation == 0
                || feed.generation > MAX_SEQUENCE
                || feed.display_title.len() > 4096
                || !matches!(
                    feed.source_type.as_str(),
                    "rss" | "youtube" | "podcast" | "mastodon" | "bluesky" | "reddit"
                )
            {
                return Err(error("githubInvalidData"));
            }
            if let Some(id) = &feed.folder_id {
                let resolved = self.resolve_folder(id)?;
                if self.subscriptions.folders[&resolved].deleted {
                    return Err(error("githubInvalidData"));
                }
            }
            for (field, version) in &feed.field_versions {
                if !matches!(field.as_str(), "title" | "folder_id" | "active") {
                    return Err(error("githubInvalidData"));
                }
                self.validate_version(Some(version))?;
            }
        }
        for (key, entry) in &self.articles {
            entry.validate()?;
            if entry.key() != *key || !self.subscriptions.feeds.contains_key(&entry.feed_key) {
                return Err(error("githubInvalidData"));
            }
        }
        for (key, state) in &self.states {
            if !self.articles.contains_key(key) {
                return Err(error("githubInvalidData"));
            }
            for flag in [&state.read, &state.starred, &state.read_later] {
                self.validate_version(flag.version.as_deref())?;
                if flag.value && flag.version.is_none() {
                    return Err(error("githubInvalidData"));
                }
            }
        }
        Ok(())
    }

    pub fn files(&self) -> Result<BTreeMap<String, String>, CoreError> {
        self.validate()?;
        let mut files = BTreeMap::new();
        files.insert(
            format!("{PREFIX}/subscriptions.json"),
            json(&self.subscriptions)?,
        );
        let mut articles: BTreeMap<String, BTreeMap<&String, &CatalogEntry>> = BTreeMap::new();
        let mut states: BTreeMap<String, BTreeMap<&String, &ArticleState>> = BTreeMap::new();
        for (key, value) in &self.articles {
            articles.entry(shard(key)?).or_default().insert(key, value);
        }
        for (key, value) in &self.states {
            states.entry(shard(key)?).or_default().insert(key, value);
        }
        for (partition, entries) in articles {
            files.insert(
                format!("{PREFIX}/articles/{partition}.json"),
                json(&entries)?,
            );
        }
        for (partition, entries) in states {
            files.insert(format!("{PREFIX}/states/{partition}.json"), json(&entries)?);
        }
        let mut manifest = self.manifest.clone();
        manifest.file_hashes = files
            .iter()
            .map(|(path, content)| (path.clone(), stable_key(&["snapshot-file:v1", content])))
            .collect();
        files.insert(format!("{PREFIX}/manifest.json"), json(&manifest)?);
        if files.values().map(String::len).sum::<usize>() > MAX_SNAPSHOT_BYTES {
            return Err(error("githubCapacityExceeded"));
        }
        Ok(files)
    }

    pub fn from_files(files: &BTreeMap<String, String>) -> Result<Self, CoreError> {
        if files.values().map(String::len).sum::<usize>() > MAX_SNAPSHOT_BYTES {
            return Err(error("githubCapacityExceeded"));
        }
        let required = |name: &str| {
            files
                .get(&format!("{PREFIX}/{name}.json"))
                .ok_or_else(|| error("githubIncompleteSnapshot"))
        };
        let mut snapshot = Self {
            manifest: decode(required("manifest")?)?,
            subscriptions: decode(required("subscriptions")?)?,
            articles: BTreeMap::new(),
            states: BTreeMap::new(),
        };
        let inventory: BTreeMap<String, String> = files
            .iter()
            .filter(|(path, _)| *path != &format!("{PREFIX}/manifest.json"))
            .map(|(path, content)| (path.clone(), stable_key(&["snapshot-file:v1", content])))
            .collect();
        if snapshot.manifest.file_hashes != inventory {
            return Err(error("githubIncompleteSnapshot"));
        }
        snapshot.manifest.file_hashes.clear();
        for (path, content) in files {
            if path == &format!("{PREFIX}/manifest.json")
                || path == &format!("{PREFIX}/subscriptions.json")
            {
                continue;
            }
            let suffix = path
                .strip_prefix(&format!("{PREFIX}/"))
                .ok_or_else(|| error("githubUnsupportedFormat"))?;
            let (kind, partition) = suffix
                .split_once('/')
                .ok_or_else(|| error("githubUnsupportedFormat"))?;
            let partition = partition
                .strip_suffix(".json")
                .ok_or_else(|| error("githubUnsupportedFormat"))?;
            let number =
                u8::from_str_radix(partition, 16).map_err(|_| error("githubInvalidData"))?;
            if number >= SHARD_COUNT || partition != format!("{number:02x}") {
                return Err(error("githubInvalidData"));
            }
            match kind {
                "articles" => {
                    let entries: UniqueEntries<CatalogEntry> = decode(content)?;
                    for (key, value) in entries.0 {
                        if shard(&key)? != partition
                            || snapshot.articles.insert(key, value).is_some()
                        {
                            return Err(error("githubInvalidData"));
                        }
                    }
                }
                "states" => {
                    let entries: UniqueEntries<ArticleState> = decode(content)?;
                    for (key, value) in entries.0 {
                        if shard(&key)? != partition || snapshot.states.insert(key, value).is_some()
                        {
                            return Err(error("githubInvalidData"));
                        }
                    }
                }
                _ => return Err(error("githubUnsupportedFormat")),
            }
        }
        snapshot.validate()?;
        Ok(snapshot)
    }
}

pub(crate) fn json<T: Serialize>(value: &T) -> Result<String, CoreError> {
    let mut text = serde_json::to_string_pretty(value).map_err(|_| error("githubInvalidData"))?;
    text.push('\n');
    if text.len() > MAX_FILE_BYTES {
        return Err(error("githubCapacityExceeded"));
    }
    Ok(text)
}

pub(crate) fn decode<T: de::DeserializeOwned>(text: &str) -> Result<T, CoreError> {
    if text.len() > MAX_FILE_BYTES {
        return Err(error("githubCapacityExceeded"));
    }
    serde_json::from_str(text).map_err(|_| error("githubInvalidData"))
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct UniqueEntries<T>(#[serde(deserialize_with = "unique_map")] BTreeMap<String, T>);

fn unique_map<'de, D, T>(deserializer: D) -> Result<BTreeMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Visitor<T>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>> de::Visitor<'de> for Visitor<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an object with unique keys")
        }
        fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, T>()? {
                if result.insert(key, value).is_some() {
                    return Err(de::Error::custom("duplicate key"));
                }
            }
            Ok(result)
        }
    }
    deserializer.deserialize_map(Visitor(PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_preserve_query_and_source_boundaries() {
        let a = feed_key(" HTTPS://EXAMPLE.com:443/Feed?a=1&b=2#fragment ").unwrap();
        assert_eq!(
            a,
            "e96736758964656f0cf3a392aa25a44cb50297757c7882010cfb7792d2acf34c"
        );
        assert_eq!(a, feed_key("https://example.com/Feed?a=1&b=2").unwrap());
        assert_ne!(a, feed_key("https://example.com/feed?a=1&b=2").unwrap());
        assert_ne!(a, feed_key("https://example.com/Feed?b=2&a=1").unwrap());
        assert_ne!(stable_key(&["ab", "c"]), stable_key(&["a", "bc"]));
        assert!(canonical_url("https://user:secret@example.com/feed").is_err());
        assert!(canonical_url("file:///tmp/feed").is_err());
        let key = article_key(&a, IdentityKind::Guid, "urn:post:123");
        assert_ne!(
            key,
            article_key(
                &feed_key("https://other.example/feed").unwrap(),
                IdentityKind::Guid,
                "urn:post:123"
            )
        );
        assert!(u8::from_str_radix(&shard(&key).unwrap(), 16).unwrap() < SHARD_COUNT);
    }

    #[test]
    fn empty_snapshot_round_trips_and_rejects_incomplete_or_unknown_format() {
        let snapshot = Snapshot::empty("a".repeat(32), "2026-07-07T00:00:00Z".into());
        let mut files = snapshot.files().unwrap();
        assert_eq!(Snapshot::from_files(&files).unwrap(), snapshot);
        files.insert(format!("{PREFIX}/other.json"), "{}".into());
        assert!(Snapshot::from_files(&files).is_err());
        files.remove(&format!("{PREFIX}/other.json"));
        files.remove(&format!("{PREFIX}/subscriptions.json"));
        assert_eq!(
            Snapshot::from_files(&files).unwrap_err().code(),
            "githubIncompleteSnapshot"
        );
    }

    #[test]
    fn duplicate_json_keys_and_sensitive_schema_fields_are_rejected() {
        assert!(decode::<UniqueEntries<u64>>("{\"key\":1,\"key\":2}").is_err());
        let mut value = serde_json::to_value(
            Snapshot::empty("a".repeat(32), "2026-07-07T00:00:00Z".into()).manifest,
        )
        .unwrap();
        value["token"] = serde_json::json!("example-only");
        assert!(serde_json::from_value::<Manifest>(value).is_err());
    }
    #[test]
    fn inventory_rejects_missing_shards_and_modified_content() {
        let snapshot = Snapshot::empty("a".repeat(32), "2026-07-07T00:00:00Z".into());
        let mut files = snapshot.files().unwrap();
        files.insert(format!("{PREFIX}/articles/00.json"), "{}\n".into());
        assert_eq!(
            Snapshot::from_files(&files).unwrap_err().code(),
            "githubIncompleteSnapshot"
        );
        let mut files = snapshot.files().unwrap();
        let manifest = files.get_mut(&format!("{PREFIX}/manifest.json")).unwrap();
        let mut wire: Manifest = decode(manifest).unwrap();
        wire.file_hashes.insert(
            format!("{PREFIX}/articles/00.json"),
            stable_key(&["snapshot-file:v1", "{}\n"]),
        );
        *manifest = json(&wire).unwrap();
        assert_eq!(
            Snapshot::from_files(&files).unwrap_err().code(),
            "githubIncompleteSnapshot"
        );
        let mut files = snapshot.files().unwrap();
        files
            .get_mut(&format!("{PREFIX}/subscriptions.json"))
            .unwrap()
            .push(' ');
        assert_eq!(
            Snapshot::from_files(&files).unwrap_err().code(),
            "githubIncompleteSnapshot"
        );
    }
}
