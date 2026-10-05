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

- Standard Android APK: `mobile/build/github-sync/papr-sync-debug.apk` (com.papr.papr_mobile, 0.1.0+1); SHA256 a4e03ccaf4e6cbdac9a546893575cf4e57d3e1a55ae495a14a1a7405a5c02afa.
- Windows executable: `target/debug/papr.exe`; SHA256 16107dc8a32eb4bc3cb6630e45b0f6264429351e37dcec3bfe391e125a43eb18.
- The preserved standard APK above has been refreshed after the second-pass fixes; use it for the normal package.


## Second static-review fixes

Review baseline: 71176d7; reviewed head: 9320c07. All six second-pass findings are corrected.

| Finding | Correction | Regression |
| --- | --- | --- |
| Mounted Android lists/readers retain old data | Listen to repository replacement, refresh the current loaded range, and reload reader state without auto-marking remotely unread articles read. Defer refresh until optimistic writes complete. Probe one extra list row to detect an exhausted range. | Five Widget tests cover empty/populated lists, removal, pagination/scroll retention, remote reader flags and overlapping list/reader writes. |
| Canonically duplicate local feeds prevent connection | Deterministically choose one cloud mapping; OR initial flags for duplicate identities. Mirror cloud subscription fields and article flags across canonical local copies, preserving every cached row, body and highlight. Local duplicate rows remain available; no destructive deduplication is performed. | Duplicate source fragments and duplicate GUIDs connect successfully; unique cached articles, both local copies, highlight linkage, false states and unsubscribe survive. |
| Saving an expired dateless article resets its age | Persist confirmed article age separately from the expiring cloud catalog; the reducer receives that confirmed history. Save it before another publication can expire the entry. Recaptured metadata retains mapped age. | Reducer test and actual service/outbox/import rounds exercise expiration, star restoration and unstar expiration, preserving local cached body. |
| Peer initializer blocks valid token replacement | Credential verification shares the exact initial-adoption eligibility checks, without mutating the dataset/checkpoint. Actual adoption still requires the sync lease. | Eligible initial peer verification leaves identity/checkpoint untouched; confirmed/replacement dataset validation remains strict. |
| Repaired private-repository access leaves schedulers paused | Successful verified secure credential replacement also clears githubRepositoryUnavailable for the same active reference. History/identity errors remain blocked. | Existing scheduling regression now covers repaired 404 access and both eligibility paths, alongside unrelated fatal-error preservation. |
| Rate-limit persistence mixes clocks | Calculate the waiting duration using server headers, then persist received-local-time plus that duration. Server Date still supplies trusted retention time and reset-duration calculation. | Both Retry-After and primary reset headers yield the same receiving-clock deadline with server clocks ten minutes ahead/behind. |

### Local schema and compatibility

Append-only v20 adds github_article_confirmations (connection ID, stable key, first confirmed time only). Upgrade backfills available article ages from the validated cached cloud files; it does not invent confirmation evidence from ordinary local fetch timestamps. New receipts retain confirmation evidence after catalog expiration. The local clear-data transaction deletes this table before its parent connections. Wire protocol, existing v18/v19 migrations, credentials, dependencies and signing configuration remain unchanged. Back up the database before upgrade; an older binary must not open the upgraded database.

### Second-pass verification

- Final Rust serial regression: desktop 265, Core 149, Bridge 3 pass. Includes populated v19 upgrade, canonical duplicate preservation, credential validation, both clock-skew headers and the complete expiration/restoration round trip.
- Frontend 89 tests, TypeScript and production build pass. Flutter analyze reports no issues; all 40 tests pass.
- The initial new pagination regression exposed an exhausted-range loading indicator; the extra-row probe fixes it and the final regression passes.
- Windows debug executable builds successfully. Android all four release ABIs and the standard Debug APK build pass (12m 48s, 215 tasks). The final Flutter paging correction is included in a subsequent standard-package rebuild using those verified native libraries (8s, 207 tasks). Repeat FRB generation changes no generated Rust/Dart hashes.
- No actual GitHub app publication, new physical-device/background acceptance, push or main-branch merge is performed. A25 remains open; emulator evidence from the previous pass is not claimed as newly rerun.


### Second-pass changed files

- Core: crates/papr-core/src/db.rs and src/sync/github/{merge,service,storage,transport}.rs.
- Desktop: src-tauri/src/db.rs (clear confirmation metadata before connections).
- Android: mobile/lib/ui/screens/{article_list_screen,article_detail_screen}.dart and mobile/test/ui/screens/github_sync_refresh_test.dart.
- Documentation: this report, docs/github-sync-implementation-2026-10-05.md, .trellis/spec/frontend/mobile-cross-layer-contracts.md and task prd.md/implement.md/task.json. Trellis journal/index record the final evidence separately.
