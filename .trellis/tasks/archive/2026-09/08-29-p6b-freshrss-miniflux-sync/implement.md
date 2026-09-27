# Implement: P6B FreshRSS 与 Miniflux 同步

## Ordered Work

1. [x] Inventory Core v16 sync schema/change-log writers and define the smallest durable outbox/cursor API without reusing `src-tauri`'s `sync_queue`.
2. [x] Add secret-free sync models, stable errors, `SyncPort`, and an injectable Fake Provider; cover push, ack, cursor, tombstone, replay, and conflict tests before the GReader adapter.
3. [x] Implement the FreshRSS/Miniflux GReader adapter in Core with shared URL/auth/request handling and redacted failures.
4. [x] Expose typed connection, status, connect/disconnect, and sync APIs through FRB; regenerate bindings and extend repositories.
5. [x] Generalize Android's encrypted credential store for isolated `papr.sync.*` aliases, preserving existing AI aliases and adding platform-channel validation/tests.
6. [x] Add the minimal Flutter Sync settings UI: provider, server/user, credential entry, test/connect, status, sync now/retry, and disconnect; localize all new strings.
7. [x] Integrate low-frequency due-sync into the existing P6A background task without adding a worker or blocking feed refresh; validate worker uniqueness.
8. [x] Run full Core/FRB/Flutter checks, build/install a Debug APK, and record Fake Provider plus device acceptance.

## Validation Commands

```powershell
cargo fmt --check
cargo test -p papr-core -- --test-threads=1
cargo check -p papr-flutter-bridge

Set-Location mobile
flutter test
flutter analyze
flutter build apk --debug
```

## Review Gates

- After steps 1-2: a replay after an injected failure must preserve every local mutation and cursor.
- After step 3: FreshRSS and Miniflux differ only at provider configuration, not conflict or queue logic.
- After step 5: an AI alias cannot be read, overwritten, or deleted through a sync credential action.
- After step 7: background refresh still runs with a missing credential or provider failure, and JobScheduler exposes one Papr worker.
- Before completion: logs, DTOs, SQLite, task records, and visible errors must contain no secret/token/provider response body.

## Rollback Points

- Core outbox and Fake Provider can land before Flutter settings and background integration.
- If a real GReader provider proves incompatible, retain the generic port/outbox and disable only that provider entry; do not store a fallback password in SQLite.
- If background sync is unstable, retain manual sync and leave the P6A worker responsible only for feed refresh until a safe low-frequency contract is restored.

## Progress — 2026-09-26

- Core reads the v16 transaction log as a bounded outbox and provides separate cursor keys; only the push cursor is wired so far. It does not use the desktop `sync_queue`.
- Fake Provider now supports a failed push after successful validation, partial acknowledgements, and idempotent replay. Core tests prove an unacknowledged suffix remains pending.
- Core now merges remote folder/feed additions and article states transactionally with the pull cursor. Remote tombstones preserve local content; a pending local article field wins until its push is confirmed. Failed pulls keep the pull cursor and merge state available for retry.
- Sync profile validation rejects query and fragment components so URL metadata cannot contain an embedded token.
- Step 2 remains open until the provider payload and identity mapping are finalized. The GReader adapter, FRB sync use case, UI, and background invocation remain open.
- Validation: `cargo test -p papr-core -- --test-threads=1` passed 93 tests; `cargo check -p papr-flutter-bridge` passed with existing FRB macro warnings; `flutter test --no-pub` passed 29 tests; `flutter analyze --no-pub` found no issues. Flutter required execution outside the workspace sandbox and cached dependency mode because the default command stalled during network resolution.

## Progress — 2026-09-27

- Core now completes push, pull, transactional merge, then one more outbox push for articles whose remote ID was learned during pull. A bounded GReader adapter handles both provider roots, authentication, subscriptions, folders, and article read/starred states with stable redacted errors.
- FRB exposes typed test/connect/status/sync/disconnect APIs. Flutter stores only the credential alias in Core; a local Android plugin registers Keystore access in both the UI and Workmanager's headless engine, sharing the existing encrypted store while restricting sync operations to `papr.sync.*` aliases.
- Settings now offers localized connection management and manual sync. The existing P6A Worker performs due sync after feed refresh and notification; a failed background attempt records its stable code and pauses until manual retry, so a frequent feed schedule cannot hammer the provider.
- Core's Fake Provider covers replay, partial acknowledgement, cursor, tombstone and conflict protection. An HTTP mock covers Miniflux GReader endpoints and assigning a local folder to an uncategorized remote feed. Emulator smoke testing launched the Debug APK, opened Sync settings, rejected an invalid connection without saving it, and showed one current Papr JobScheduler job.
- Latest checks before the final adapter refinement: Core 99 tests, FRB check, Flutter 29 tests, analyzer, and Debug APK build passed. Real FreshRSS/Miniflux account testing remains for a later device acceptance run with user-provided service credentials; no credentials were recorded in this task.
- Final gate: Core 101 tests, FRB check, Flutter 29 tests, analyzer, targeted Rust format, and Debug APK build passed. The final APK installed and launched on `emulator-5554`; JobScheduler showed one current Papr job. Repository-wide `cargo fmt --check` still fails on untouched `src-tauri` formatting, so only this task's Rust files were checked with `rustfmt --check`.
