# GitHub personal synchronization implementation

This records the implementation following the approved [protocol design](github-personal-sync-design-2026-10-05.md). Desktop and Android share the protocol, merge engine, database capture/import helpers, and GitHub transport in `crates/papr-core/src/sync/github/`. Platform adapters provide existing database access, secure credentials, scheduling, and UI.

## Repository configuration

Sync runs against a dedicated private repository (for example `<owner>/papr-sync`), default branch `main`. The app accepts repository coordinates in settings; this user's account is not hardcoded into the product. The GitHub CLI login was used only for repository metadata, not as the app's credential source.

## Set up both clients

1. Export an OPML backup and keep a backup of each device's pre-upgrade database before opening the new client. The append-only schema migration upgrades to v20; an older binary must not open that upgraded database. A rollback uses the old database backup.
2. Create a fine-grained personal access token, preferably one per device, with resource owner set to the repository owner, repository access limited to the sync repository, and repository **Contents: Read and write**. Metadata read access is included by GitHub. Choose an expiry you can maintain. No Actions, workflow, or administration permission is needed. See [GitHub token guidance](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens) and [Git reference permissions](https://docs.github.com/en/rest/git/refs?apiVersion=2026-03-10).
3. In Papr's synchronization settings, disconnect any GReader backend, then enter the owner, the repository name, branch `main`, and the token. The form takes the repository name, not its `.git` URL.
4. Preview the local/remote counts and exclusions, then confirm connection. Preview and confirmation read GitHub; the first sync performs the initial publication. Saved states are initially combined, while an existing cloud unsubscribe is preserved.
5. Run a manual sync on the first device, then preview/connect and sync on the second. Bodies remain local; a metadata article can be read, saved, or hydrated through a normal fetch/full-text extraction.

Enter tokens only in the app's password input. Windows stores them in Credential Manager; Android uses the existing Keystore plugin. SQLite stores only credential references and sync metadata. Updating a token verifies the same repository before replacing the secure value, then clears only credential-related failures so scheduling can resume. Disconnect keeps local reading data and disables that connection's queue. Repeated connection confirmation is rejected before writing credentials; failure cleanup retains any reference now owned by an active connection, and uncertain ownership preserves the original error.

## Implemented contracts and refinements

- Publication creates a tree from the observed `base_tree`, a commit with that exact observed head as parent, and updates the branch with `force=false`. It retries at most three head conflicts. It preserves unrelated repository files. [GitHub documents non-force updates and Contents write permission](https://docs.github.com/en/rest/git/refs?apiVersion=2026-03-10).
- Local mutations and their ordered outbox intents commit together through SQLite triggers. Bulk state edits, rule-driven initial flags, and OPML's existing feed/folder writers use the same capture path. Remote import suppresses capture within its transaction.
- Only a confirmed remote device watermark acknowledges intents. A persisted candidate/attempt allows recovery after a successful ref update whose response was lost. Edits made during network I/O remain pending and are projected over the confirmed state before import.
- Installation markers, database-path binding, and secure sequence checkpoints detect copied/restored writer identity. Checkpoint origin remains immutable when a never-confirmed initializer adopts the peer dataset. A changed connection, repository ID, dataset/epoch, or rewritten history prevents an old response from importing or acknowledging data.
- Ordinary cloud catalog entries expire after 90 days; starred or read-later entries remain indefinitely. Cloud retention never deletes local cached bodies. Local cache deletion records a suppression marker, produces no cloud delete, and does not repeatedly import an ordinary deleted item.
- Unsubscribe archives the local source row, stops fetching/exporting it, and hides ordinary articles. Saved articles remain accessible. Explicit re-subscription advances its generation. Stale unsubscribe/update intents cannot cancel a later generation.
- Imported article metadata uses a stable key rather than local row IDs. A unique, proven URL fallback can gain a GUID without merging two different real GUIDs. Hydration uses the same row, preserves flags, rebuilds FTS, and returns “not newly inserted” before ingestion rules or notifications.
- Folder imports temporarily vacate names inside the transaction before applying the final valid names. This handles name swaps and deleting/recreating the same name without transient SQLite uniqueness errors. Same-name creation aliases folders; a conflicting rename is acknowledged as a rejected intent, with a warning count.
- A local desktop “clear all data” first disconnects the device, refuses an active lease, then clears business and local GitHub metadata. It does not publish cloud deletion. The Windows adapter also deletes that connection's token.

The wire manifest now includes `file_hashes`, a sorted inventory of every non-manifest protocol file. Its values use the shared length-prefixed SHA-256 helper with `snapshot-file:v1` and the exact UTF-8 JSON content. It references neither its own content nor a commit SHA. Sparse empty shards remain omitted. This refinement detects missing article and state shards even when both disappear together, as well as altered content. Parsing validates the entire inventory before normalizing it out of the in-memory manifest. No previous version has been published to the user's repository.

## Scheduling, limits, and status

- Foreground coordinators check lightweight eligibility every 10 seconds without loading article statistics or cached snapshots. Per-device settings default to 30-second upload debounce and 10-minute cloud polling; continuous edits flush after max(60 seconds, twice debounce). Ref publication is spaced by at least 60 seconds, except bounded retries of a known rejected publication. Manual-only mode preserves queued edits and explicit sync.
- Android reuses its existing WorkManager refresh task at the minimum enabled RSS/GitHub cadence. The default GitHub background interval is 60 minutes, configurable from 15 to 360 minutes; pending edits can sync at an eligible Worker execution. Manual-only mode preserves RSS refresh while disabling automatic GitHub work. Android scheduling is inexact and depends on OS/background policy.
- Clean confirmed connections check repository privacy/identity and branch head with two GETs. Unchanged heads skip snapshot decoding/import and reading-list invalidation; pending/uncertain operations, concurrent edits, or 24-hour maintenance require the full path. Settings remain local; v21 adds the metadata-statistics index and full-maintenance timestamp without changing protocol v1.
- Unknown network failures preserve the queue and back off at least 60 seconds. GitHub `Retry-After`/rate-reset headers extend the pause. Authentication, integrity, history, identity, and platform credential failures stop foreground automatic retries; the user can repair credentials or disconnect/reconnect and retry manually.
- Cancellation interrupts a pending network await. A cancelled publication may have reached GitHub; its persisted attempt and watermark recovery remain authoritative.
- Current bounds: 64 logical shards, at most 130 protocol files, 1 MiB per file, 50 MiB snapshot content, 500 intents per batch, 2 MiB inline tree request before separate blob creation, one-second API write spacing. Reads are sequential and reuse unchanged blob SHAs. No empty commit is created for an unchanged snapshot.
- Status exposes pending/rejected counts, metadata-only count, last success/error, retry deadline, busy state, and uncertain publication. Rejections are cumulative per connection. Detailed per-field conflict inspection and dismissing individual rejections are not implemented; the reducer computes overwrite diagnostics, while the current UI exposes rejection counts.
- Repository history grows normally; the client never rewrites it. Automatic Git-history size alarms and an automated migration to a replacement repository are not implemented. Repository switching is explicit disconnect/preview/connect.

## Manual acceptance checklist

- Back up local reading data, then enter scoped device tokens in the two apps.
- Add/move a feed and folder on Windows; confirm Android uses the same organization and article metadata.
- Star on one device and add read-later on the other; confirm both fields survive. Remove each flag and confirm false propagates.
- Edit the same field while one device is offline, then reconnect and confirm publication-order convergence.
- Disconnect the network during publication, retry, and verify the queue drains without restoring an older saved state.
- Open an imported metadata article and fetch full text; confirm the same item and flags remain, with no duplicate new-article notification.
- Test Android after reboot, in the background, and under battery restrictions. Record actual delay and secure credential availability; an APK build alone proves none of these.
- Keep GitHub history intact. A dedicated replacement repository/reconnection is the recovery path for an intentionally rebuilt dataset.

## Changed-file groups

| Files | Purpose |
| --- | --- |
| crates/papr-core/src/sync/github/{model,merge,storage,service,transport,mod}.rs and schema.sql | Shared protocol, identities/inventory, reducer, durable capture/import, orchestration and Git Data API with tests |
| crates/papr-core/src/{db,lib,sync}.rs and services/settings.rs | Migration/functions, archived-feed queries, hydration/extraction and mutually exclusive backends |
| src-tauri/src/github_{credentials,sync}.rs, db.rs, commands.rs, lib.rs, state.rs, sync.rs | Windows secure credentials, existing writer adapter, scheduler/commands, local wipe and GReader guards |
| src/api.ts, components/{GitHubSyncSection,SettingsDialog}.tsx, lib/errors.ts, locales/{en,zh,ja}.json | Typed desktop UI, preview/status/actions and localized errors |
| crates/papr-flutter-bridge/src/{api,github_dto,lib,frb_generated}.rs; mobile/lib/bridge/generated/*.dart | Thin bridge APIs/DTOs and regenerated bindings |
| mobile/lib/repositories/{github_sync,settings}_repository.dart; services/{github_sync_credentials,background_refresh_service}.dart | Secure platform access/checkpoints and foreground/background eligibility |
| mobile/lib/ui/{screens/github_sync_panel,screens/sync_settings_screen,navigation/github_sync_coordinator}.dart and app.dart | Android sync settings, lifecycle coordinator and data-cache refresh |
| mobile/lib/l10n/*.arb, app_localizations*.dart, l10n.dart | English/Chinese/Japanese messages and generated localization |
| crates/papr-core/Cargo.toml, src-tauri/Cargo.toml, Cargo.lock, Trellis task/spec, this document | Reuse existing transitive sha2/windows-sys as direct dependencies, record contracts/evidence |