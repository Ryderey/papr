# GitHub personal synchronization implementation

This records the implementation following the approved [protocol design](github-personal-sync-design-2026-10-05.md). Desktop and Android share the protocol, merge engine, database capture/import helpers, and GitHub transport in `crates/papr-core/src/sync/github/`. Platform adapters provide existing database access, secure credentials, scheduling, and UI.

## Repository and branch state

- Sync repository: `Ryderey/papr-sync`, private, default branch `main`.
- Immutable GitHub repository ID: `1405209831`, verified through a read-only GitHub CLI request on 2026-10-05.
- Integration commit: `71176d7`; local `optimize-bugfix` was advanced to it by fast-forward.
- Feature branch: `codex/github-personal-sync`, based on that integrated commit.
- Implementation checkout: `C:/Users/Ryder/.codex/worktrees/integrate-desktop-mobile/papr`.
- Original mobile checkout and its uncommitted Android signing/release files remain untouched. No branch was pushed and no sync content was uploaded.

The app accepts repository coordinates in settings; this user's account is not hardcoded into the product. The GitHub CLI login was used only for repository metadata, not as the app's credential source.

## Set up both clients

1. Export an OPML backup and keep a backup of each device's pre-upgrade database before opening the new client. The append-only schema migration upgrades to v18; an older binary must not open that upgraded database. A rollback uses the old database backup.
2. Create a fine-grained personal access token, preferably one per device, with resource owner `Ryderey`, repository access limited to `papr-sync`, and repository **Contents: Read and write**. Metadata read access is included by GitHub. Choose an expiry you can maintain. No Actions, workflow, or administration permission is needed. See [GitHub token guidance](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens) and [Git reference permissions](https://docs.github.com/en/rest/git/refs?apiVersion=2026-03-10).
3. In Papr's synchronization settings, disconnect any GReader backend, then enter owner `Ryderey`, repository `papr-sync`, branch `main`, and the token. The form takes the repository name, not its `.git` URL.
4. Preview the local/remote counts and exclusions, then confirm connection. Preview and confirmation read GitHub; the first sync performs the initial publication. Saved states are initially combined, while an existing cloud unsubscribe is preserved.
5. Run a manual sync on the first device, then preview/connect and sync on the second. Bodies remain local; a metadata article can be read, saved, or hydrated through a normal fetch/full-text extraction.

Enter tokens only in the app's password input. Windows stores them in Credential Manager; Android uses the existing Keystore plugin. SQLite stores only credential references and sync metadata. Updating a token verifies the same repository before replacing the secure value. Disconnect keeps local reading data and disables that connection's queue. Repeated connection confirmation is rejected before writing credentials; failure cleanup retains any reference now owned by an active connection, and uncertain ownership preserves the original error.

## Implemented contracts and refinements

- Publication creates a tree from the observed `base_tree`, a commit with that exact observed head as parent, and updates the branch with `force=false`. It retries at most three head conflicts. It preserves unrelated repository files. [GitHub documents non-force updates and Contents write permission](https://docs.github.com/en/rest/git/refs?apiVersion=2026-03-10).
- Local mutations and their ordered outbox intents commit together through SQLite triggers. Bulk state edits, rule-driven initial flags, and OPML's existing feed/folder writers use the same capture path. Remote import suppresses capture within its transaction.
- Only a confirmed remote device watermark acknowledges intents. A persisted candidate/attempt allows recovery after a successful ref update whose response was lost. Edits made during network I/O remain pending and are projected over the confirmed state before import.
- Installation markers, database-path binding, and secure sequence checkpoints detect copied/restored writer identity. A changed connection, repository ID, dataset/epoch, or rewritten history prevents an old response from importing or acknowledging data.
- Ordinary cloud catalog entries expire after 90 days; starred or read-later entries remain indefinitely. Cloud retention never deletes local cached bodies. Local cache deletion records a suppression marker, produces no cloud delete, and does not repeatedly import an ordinary deleted item.
- Unsubscribe archives the local source row, stops fetching/exporting it, and hides ordinary articles. Saved articles remain accessible. Explicit re-subscription advances its generation. Stale unsubscribe/update intents cannot cancel a later generation.
- Imported article metadata uses a stable key rather than local row IDs. A unique, proven URL fallback can gain a GUID without merging two different real GUIDs. Hydration uses the same row, preserves flags, rebuilds FTS, and returns “not newly inserted” before ingestion rules or notifications.
- Folder imports temporarily vacate names inside the transaction before applying the final valid names. This handles name swaps and deleting/recreating the same name without transient SQLite uniqueness errors. Same-name creation aliases folders; a conflicting rename is acknowledged as a rejected intent, with a warning count.
- A local desktop “clear all data” first disconnects the device, refuses an active lease, then clears business and local GitHub metadata. It does not publish cloud deletion. The Windows adapter also deletes that connection's token.

The wire manifest now includes `file_hashes`, a sorted inventory of every non-manifest protocol file. Its values use the shared length-prefixed SHA-256 helper with `snapshot-file:v1` and the exact UTF-8 JSON content. It references neither its own content nor a commit SHA. Sparse empty shards remain omitted. This refinement detects missing article and state shards even when both disappear together, as well as altered content. Parsing validates the entire inventory before normalizing it out of the in-memory manifest. No previous version has been published to the user's repository.

## Scheduling, limits, and status

- Foreground coordinators check eligibility every 10 seconds. Pending edits debounce for 10 seconds, with a 60-second maximum wait; foreground pulls are due every five minutes. Ref publication is spaced by at least 60 seconds, except bounded retries of a known rejected publication.
- Android reuses its existing WorkManager refresh task. Sync-only background scheduling uses six hours when feed refresh is disabled. With feed refresh enabled, pending edits can sync on its existing periodic task and an otherwise idle pull is due after six hours. Android scheduling is inexact and depends on OS/background policy.
- Unknown network failures preserve the queue and back off at least 60 seconds. GitHub `Retry-After`/rate-reset headers extend the pause. Authentication, integrity, history, identity, and platform credential failures stop foreground automatic retries; the user can repair credentials or disconnect/reconnect and retry manually.
- Cancellation interrupts a pending network await. A cancelled publication may have reached GitHub; its persisted attempt and watermark recovery remain authoritative.
- Current bounds: 64 logical shards, at most 130 protocol files, 1 MiB per file, 50 MiB snapshot content, 500 intents per batch, 2 MiB inline tree request before separate blob creation, one-second API write spacing. Reads are sequential and reuse unchanged blob SHAs. No empty commit is created for an unchanged snapshot.
- Status exposes pending/rejected counts, metadata-only count, last success/error, retry deadline, busy state, and uncertain publication. Rejections are cumulative per connection. Detailed per-field conflict inspection and dismissing individual rejections are not implemented; the reducer computes overwrite diagnostics, while the current UI exposes rejection counts.
- Repository history grows normally; the client never rewrites it. Automatic Git-history size alarms and an automated migration to a replacement repository are not implemented. Repository switching is explicit disconnect/preview/connect.

## Acceptance evidence

Tests use temporary databases and fake transports or loopback HTTP fixtures. They do not use the user's reading database, real token, or remote repository writes.

| Design scenario | Evidence |
| --- | --- |
| A01–A02: different local IDs, remote-only metadata | `two_databases_import_stable_ids_and_metadata_then_local_cleanup_stays_local` verifies folder association, a different feed/folder ID, title FTS, metadata-only body, and independent saved flags. |
| A03–A05: independent fields, false, ordering | `independent_fields_merge_and_explicit_false_survives_retry`; operation versions contain device/sequence, not device timestamps. The two-database test also propagates explicit false. |
| A06: competing publication | `rejected_ref_rebases_only_when_head_changed`, `publication_preserves_tree_parent_and_never_forces_ref`, and bounded service retries. Independent-field convergence is exercised separately in the reducer; an actual simultaneous two-client GitHub race remains part of manual network validation. |
| A07: lost successful response | `lost_ref_response_recovers_watermark_without_replaying_after_other_device_edit`. |
| A08–A09: transaction rollback and in-flight writes | `capture_rolls_back_and_never_contains_bodies_or_configuration`, `import_ack_and_inflight_edit_commit_together_without_echo`, changed-dataset rollback, and the invalid-seed reducer rollback test. |
| A10: same-row hydration/extraction | `metadata_hydration_preserves_flags_and_never_emits_a_new_article_intent`; desktop archive/extraction regression. Core/desktop return false before rules and new-article counting. |
| A11–A14: folder/source lifecycle | Folder deletion/stale move, source tombstone/generation, initial inactive source, same-name alias/collision, folder swap/name-reuse, and Core business re-subscription tests. |
| A15–A17: retention and initial union | `retention_keeps_old_saved_items_and_releases_unprotected_items`, `cloud_retention_removes_catalog_without_deleting_local_cached_body`, initial saved-state union and inactive-source tests. |
| A18: failure/rate limit/cancel | Loopback HTTP rate-limit fixture, durable retry/backoff test, lost-response test, and `cancellation_interrupts_a_pending_network_future`. Expiring an actual PAT is not simulated against GitHub. |
| A19: malformed/incomplete format | Required files, duplicate JSON keys, unknown schema fields, strict shard paths, file inventory corruption/missing-file tests, and truncated-tree transport fixture. |
| A20–A21: identity/history/clone/restore | Repository ID fixture, rewritten-history service test, clone/lease/epoch guard test, stale response after reconnect, and secure checkpoint rollback test. |
| A22: bulk/rules/OPML/cache | `core_business_writes_capture_rules_bulk_read_and_resubscription`, folder/bulk SQL capture, and local-cleanup suppression. OPML reuses the tested feed/folder writer path; no additional network-dependent OPML import fixture was run for GitHub mode. |
| A23: exclusions | Outbox payload assertions exclude bodies/local private configuration; strict wire fields exclude token/unknown fields; transport errors omit credential and response content. Windows isolated secure-storage round trip succeeds and removes its test item. |
| A24: no empty commits | `unchanged_head_has_no_commit_and_cooldown_keeps_edits_pending`, `a_new_maintenance_clock_without_changes_does_not_create_a_commit`. |
| A25: real network and background | **Partially validated; real acceptance pending.** Repository privacy/ID/default branch verified read-only. An isolated Android emulator package passes foreground and headless WorkManager Keystore/Core bridge smoke checks and renders the GitHub form. No real PAT entered, no app-level remote publication, no production GitHub-connected Worker, no physical Android/Doze/reboot test, and no simultaneous Windows/Android network round trip yet. |

Final build/test counts and artifact paths are recorded in the active Trellis task's `implement.md`. Existing warnings include FRB `frb_expand` cfg warnings, Gradle deprecations, and the desktop bundle-size advisory.

## Manual acceptance checklist

- Back up local reading data, then enter scoped device tokens in the two apps.
- Add/move a feed and folder on Windows; confirm Android uses the same organization and article metadata.
- Star on one device and add read-later on the other; confirm both fields survive. Remove each flag and confirm false propagates.
- Edit the same field while one device is offline, then reconnect and confirm publication-order convergence.
- Disconnect the network during publication, retry, and verify the queue drains without restoring an older saved state.
- Open an imported metadata article and fetch full text; confirm the same item and flags remain, with no duplicate new-article notification.
- Test Android after reboot, in the background, and under battery restrictions. Record actual delay and secure credential availability; an APK build alone proves none of these.
- Keep GitHub history intact. A dedicated replacement repository/reconnection is the recovery path for an intentionally rebuilt dataset.

## Changed-file groups and final verification

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

Final Rust tests pass: desktop 265, Core 137 (34 GitHub tests), Bridge 3. Frontend 88 tests/build and Windows debug executable build pass. Flutter analyze and 34 tests pass; generated FRB files are reproducible. All four Android Rust ABIs and the standard Debug APK build successfully. The isolated emulator smoke logs foreground/Worker pass markers and shows the GitHub form. These results do not replace A25 real account/device acceptance.

Review artifacts are local, untracked build outputs: target/debug/papr.exe and mobile/build/github-sync/papr-sync-debug.apk. The latter is the standard com.papr.papr_mobile package at 0.1.0+1, preserved separately from the isolated test package and updated after the final adapter-only rebuild. Its Debug signing/version may not upgrade an existing installed app; preserve the original signing/version work and data instead of uninstalling to bypass an upgrade mismatch.
