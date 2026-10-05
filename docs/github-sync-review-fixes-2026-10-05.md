# GitHub synchronization static-review fixes

Date: 2026-10-05. Branch: `codex/github-personal-sync`. Review base: `71176d7`; reviewed head: `422b75b`. These changes resolve the eight static-review findings. Real GitHub/physical-device acceptance remains A25 and is separate from automated regression evidence.

| Finding | Correction | Regression evidence |
| --- | --- | --- |
| Android native credential errors become unknown | Normalize `PlatformException` to a safe `AppException` code in the shared GitHub platform helper; report it in foreground and Worker paths. Never carry native message/details. | Flutter mock credential channel raises read/write failures; original codes survive and private details do not. Core scheduling test verifies credential errors stop foreground and background eligibility. |
| Automatic desktop import leaves folder/detail caches stale | Background feed-update events use the same reading-query invalidation helper as manual/bulk changes, including folders, open article detail and searches. | QueryClient regression verifies reading caches become stale while settings stay valid. |
| Archived unread sidebar counts disagree with lists | Smart unread count uses the same active-source-or-saved visibility predicate as article lists. | Existing populated desktop archive regression now also verifies smart counts before/after saving an archived article. |
| Token update does not resume automatic scheduling | After repository verification and successful secure storage, adapters call shared `credential_updated` for the active credential reference. It clears only credential errors/backoff; history/identity errors remain blocked. Worker respects Core background eligibility. | Core regression checks both schedulers resume, unrelated history errors remain and a wrong credential reference is rejected. |
| Concurrent first initializers create incompatible dataset IDs | An unpublished initializer can adopt the confirmed peer dataset under its lease; it preserves device ID, sequence, outbox and checkpoint origin. Confirmed connections still reject replacement datasets. | Two independent Core databases connect to the empty fake repository before either publishes; both article catalogs and saved flags converge, pending edits drain and the original secure checkpoint remains valid. A later dataset replacement is rejected. |
| Lost publication response can lose old rejection receipts | Persist the confirmed remote rejection receipt in a short guarded transaction before reducing/publishing a newer suffix. Final import reuses that receipt helper. | Lost ref response plus a rejected folder rename plus a later star edit: the next remote receipt is replaced, while the original local rejection remains recorded. |
| Retained folder tombstone deletes a reused local row ID | Tombstone and alias deletion verify `sync_id`; retire tombstone mappings; live mapping lookup and temporary-name vacating also verify stable identity. | SQLite explicitly reuses the old folder's row ID; two imports preserve the replacement folder and a local-only Newsletter's membership. |
| Future publication dates bypass 90 days | New catalog entries take first cloud confirmation time from the trusted HTTP clock. Future publication dates fall back to that time; refetch does not extend it. Saved states remain protected. | Future-dated article with an incorrect local first-seen clock expires after the window, does not gain life on refetch, and remains when starred. |

## Local migration and security boundary

The existing v18 migration is unchanged. Append-only v19 adds `github_connections.initializing` and `checkpoint_dataset_id`. v18 rows preserve their checkpoint origin; only an unacknowledged connection with no cached protocol files and an initial seed is marked initializing. New connections initialize these fields explicitly. The secure checkpoint remains `<checkpoint-origin-dataset>:<device>:<highest-reserved-sequence>`; adopting the peer's cloud dataset never changes that origin or permits sequence rollback.

Adoption requires an active matching lease, epoch 1, no confirmed acknowledgment or cached protocol, and the original seed. A different dataset already containing this device is rejected. Normal dataset/epoch/history/installation validation remains in effect. No token/body enters the new fields; no wire format, remote history, dependencies or signing configuration changed.

## Verification

- Full serial Rust regression: desktop 265, Core 143 (39 shared GitHub tests plus the v18-to-v19 migration regression), Bridge 3 pass.
- Frontend: 89 tests pass; TypeScript and production build pass.
- Flutter: analyze reports no issues; 35 tests pass, including native credential failure normalization.
- FRB generated bindings include `github_credential_updated`; repeat generation changes none of the generated Rust/Dart hashes.
- Windows debug executable build passes. Android all four release ABIs and the standard Debug APK build pass (8m 44s, 215 tasks). The existing isolated emulator test package passes foreground/headless Worker Keystore/Core bridge smoke again without touching the real package. Artifact hashes are recorded in the task.
- Build outputs and raw logs are not committed. No real app-level GitHub write, PAT test, physical Android/Doze/reboot or simultaneous real-account round trip was performed.

## Boundaries and follow-up

Back up data before the v19 upgrade; do not open the upgraded database with an older binary. Keep the task active for A25 real-device acceptance. The original mobile checkout's uncommitted signing/release work remains untouched. Debug APK upgrade still requires matching signing/version; use an independent test package rather than uninstalling the real app.

## Updated artifacts

- Standard Android APK: `mobile/build/github-sync/papr-sync-debug.apk` (com.papr.papr_mobile, 0.1.0+1); SHA256 02f78d0a2556fdcb20cb09ad9588e5d684a380caae169ff92582832fe51f11e2.
- Windows executable: `target/debug/papr.exe`; SHA256 eb6ebcbb1c7aea6a4d2070c239fe2d5790a2e0721a643bbf0d811dd839f5786a.
- Default Gradle output was subsequently replaced by the isolated smoke package; use the preserved standard APK above.
