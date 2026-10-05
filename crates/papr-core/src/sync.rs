//! Provider-neutral synchronization contracts.
//!
//! Network adapters implement [`SyncPort`]. Core keeps local changes durable in
//! SQLite and advances its cursor only after the adapter acknowledges a
//! contiguous prefix of a batch.

pub mod github;
pub mod greader;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::dto::{SyncChange, SyncEntity, SyncOperation};
use crate::error::{CoreError, ErrorCategory};

/// The mobile-supported GReader-compatible provider families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncProvider {
    FreshRss,
    Miniflux,
}

/// Secret-free configuration for one external reader service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncProfile {
    pub provider: SyncProvider,
    pub server_url: String,
    pub username: String,
    pub credential_ref: String,
}

impl SyncProfile {
    pub fn validate(&self) -> Result<(), CoreError> {
        let url = Url::parse(self.server_url.trim()).map_err(|_| invalid_profile())?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || self.username.trim().is_empty()
            || self.username.len() > 256
            || !is_sync_credential_ref(&self.credential_ref)
        {
            return Err(invalid_profile());
        }
        Ok(())
    }
}

fn invalid_profile() -> CoreError {
    CoreError::coded(ErrorCategory::Sync, "invalidSyncProfile", None)
}

fn is_sync_credential_ref(value: &str) -> bool {
    value.strip_prefix("papr.sync.").is_some_and(|suffix| {
        (1..=80).contains(&suffix.len())
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    })
}

/// A remote mutation returned by a provider pull operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteSyncChange {
    pub remote_id: String,
    pub entity: SyncEntity,
    pub operation: SyncOperation,
    pub field: Option<String>,
    pub value: Option<String>,
    /// Article URL for matching a remote item to an already-ingested article.
    pub url: Option<String>,
    /// Folder identity for a remote subscription, when one is assigned.
    pub folder_remote_id: Option<String>,
}

/// One cursor-scoped provider response. Credentials never appear in this DTO.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyncPull {
    pub cursor: Option<String>,
    pub changes: Vec<RemoteSyncChange>,
}

/// The minimal contract shared by FreshRSS, Miniflux, and the deterministic
/// Fake Provider. Real adapters may exchange any authentication material
/// internally, but this contract never persists or returns it.
#[allow(async_fn_in_trait)]
pub trait SyncPort {
    async fn validate(&mut self) -> Result<(), CoreError>;
    async fn push(&mut self, changes: &[SyncChange]) -> Result<Vec<i64>, CoreError>;
    async fn pull(&mut self, cursor: Option<&str>) -> Result<SyncPull, CoreError>;
    async fn acknowledge(&mut self, cursor: &str) -> Result<(), CoreError>;
}

/// Deterministic in-memory provider for Core contract tests.
#[derive(Debug, Default)]
pub struct FakeSyncPort {
    pull: SyncPull,
    pushed: BTreeMap<i64, SyncChange>,
    acknowledged_cursors: Vec<String>,
    fail_next: Option<CoreError>,
    fail_next_push: Option<CoreError>,
    fail_next_pull: Option<CoreError>,
    push_acknowledgements: Option<BTreeSet<i64>>,
}

impl FakeSyncPort {
    pub fn with_pull(pull: SyncPull) -> Self {
        Self {
            pull,
            ..Self::default()
        }
    }

    /// Make the next port operation fail without changing provider state.
    pub fn fail_next(&mut self, error: CoreError) {
        self.fail_next = Some(error);
    }

    /// Fail the next push after validation has succeeded.
    pub fn fail_next_push(&mut self, error: CoreError) {
        self.fail_next_push = Some(error);
    }

    /// Fail the next pull after the local push has completed.
    pub fn fail_next_pull(&mut self, error: CoreError) {
        self.fail_next_pull = Some(error);
    }

    /// Acknowledge only these sequence numbers in subsequent push calls.
    pub fn acknowledge_push_sequences(&mut self, sequences: &[i64]) {
        self.push_acknowledgements = Some(sequences.iter().copied().collect());
    }

    pub fn pushed_changes(&self) -> Vec<SyncChange> {
        self.pushed.values().cloned().collect()
    }

    pub fn acknowledged_cursors(&self) -> &[String] {
        &self.acknowledged_cursors
    }

    fn fail_if_requested(&mut self) -> Result<(), CoreError> {
        match self.fail_next.take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl SyncPort for FakeSyncPort {
    async fn validate(&mut self) -> Result<(), CoreError> {
        self.fail_if_requested()
    }

    async fn push(&mut self, changes: &[SyncChange]) -> Result<Vec<i64>, CoreError> {
        self.fail_if_requested()?;
        if let Some(error) = self.fail_next_push.take() {
            return Err(error);
        }
        let acknowledged = changes
            .iter()
            .filter(|change| {
                self.push_acknowledgements
                    .as_ref()
                    .is_none_or(|sequences| sequences.contains(&change.sequence))
            })
            .cloned()
            .collect::<Vec<_>>();
        for change in &acknowledged {
            self.pushed.insert(change.sequence, change.clone());
        }
        Ok(acknowledged.iter().map(|change| change.sequence).collect())
    }

    async fn pull(&mut self, _cursor: Option<&str>) -> Result<SyncPull, CoreError> {
        self.fail_if_requested()?;
        if let Some(error) = self.fail_next_pull.take() {
            return Err(error);
        }
        Ok(self.pull.clone())
    }

    async fn acknowledge(&mut self, cursor: &str) -> Result<(), CoreError> {
        self.fail_if_requested()?;
        self.acknowledged_cursors.push(cursor.to_string());
        Ok(())
    }
}

/// Return the highest contiguous local sequence confirmed by a provider.
///
/// An out-of-order acknowledgement cannot skip an earlier local mutation;
/// callers retain the whole suffix and safely replay it on retry.
pub fn highest_contiguous_acknowledgement(
    pending: &[SyncChange],
    acknowledged: &[i64],
) -> Option<i64> {
    let acknowledged: BTreeSet<_> = acknowledged.iter().copied().collect();
    pending
        .iter()
        .take_while(|change| acknowledged.contains(&change.sequence))
        .last()
        .map(|change| change.sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;

    fn change(sequence: i64) -> SyncChange {
        SyncChange {
            sequence,
            entity: SyncEntity::Article,
            local_id: sequence,
            operation: SyncOperation::Upsert,
            field: Some("read".to_string()),
            value: Some("1".to_string()),
            remote_id: None,
            url: None,
            folder: None,
        }
    }

    #[test]
    fn acknowledgement_never_skips_an_unconfirmed_change() {
        let pending = vec![change(10), change(11), change(12)];
        assert_eq!(
            highest_contiguous_acknowledgement(&pending, &[10, 12]),
            Some(10)
        );
        assert_eq!(
            highest_contiguous_acknowledgement(&pending, &[11, 12]),
            None
        );
        assert_eq!(
            highest_contiguous_acknowledgement(&pending, &[10, 11, 12]),
            Some(12)
        );
    }

    #[test]
    fn sync_profile_rejects_embedded_or_cross_namespace_credentials() {
        let profile = SyncProfile {
            provider: SyncProvider::FreshRss,
            server_url: "https://reader.example.com".to_string(),
            username: "reader".to_string(),
            credential_ref: "papr.sync.primary".to_string(),
        };
        assert!(profile.validate().is_ok());
        assert_eq!(
            SyncProfile {
                server_url: "https://reader:password@example.com".to_string(),
                ..profile.clone()
            }
            .validate()
            .unwrap_err()
            .code(),
            "invalidSyncProfile"
        );
        assert_eq!(
            SyncProfile {
                credential_ref: "papr.ai.primary".to_string(),
                ..profile.clone()
            }
            .validate()
            .unwrap_err()
            .code(),
            "invalidSyncProfile"
        );
        assert_eq!(
            SyncProfile {
                server_url: "https://reader.example.com/?token=private".to_string(),
                ..profile
            }
            .validate()
            .unwrap_err()
            .code(),
            "invalidSyncProfile"
        );
    }

    #[tokio::test]
    async fn fake_provider_replays_after_an_injected_failure() {
        let batch = vec![change(1), change(2)];
        let mut port = FakeSyncPort::default();
        port.fail_next(CoreError::coded(
            ErrorCategory::Network,
            "syncUnavailable",
            None,
        ));

        assert_eq!(
            port.push(&batch).await.unwrap_err().code(),
            "syncUnavailable"
        );
        assert!(port.pushed_changes().is_empty());

        assert_eq!(port.push(&batch).await.unwrap(), vec![1, 2]);
        assert_eq!(port.pushed_changes(), batch);
        assert_eq!(port.push(&batch).await.unwrap(), vec![1, 2]);
        assert_eq!(port.pushed_changes(), batch);
    }

    #[tokio::test]
    async fn fake_provider_carries_pull_cursor_and_tombstone() {
        let pull = SyncPull {
            cursor: Some("remote-2".to_string()),
            changes: vec![RemoteSyncChange {
                remote_id: "feed-42".to_string(),
                entity: SyncEntity::Feed,
                operation: SyncOperation::Tombstone,
                field: None,
                value: None,
                url: None,
                folder_remote_id: None,
            }],
        };
        let mut port = FakeSyncPort::with_pull(pull.clone());

        port.validate().await.unwrap();
        assert_eq!(port.pull(Some("remote-1")).await.unwrap(), pull);
        port.acknowledge("remote-2").await.unwrap();
        assert_eq!(port.acknowledged_cursors(), ["remote-2"]);
    }
}
