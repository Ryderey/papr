# Implement: P6A 后台刷新与新文章通知

## Ordered Work

1. [x] Add the shared refresh-off constant, due-feed query, validated background settings, and Core tests.
2. [x] Make `RefreshOptions.force` select all versus due feeds; retain optional feed filtering and manual-refresh behavior.
3. [x] Extend FRB DTO/API conversion, regenerate Dart bindings, and update repository tests/callers.
4. [x] Add `workmanager` and `flutter_local_notifications`; implement the top-level background dispatcher, Core initialization, retry mapping, unique scheduling, and notification summary.
5. [x] Add auto-refresh interval, notification, and fixed quiet-hours controls to Settings; request notification permission only after an explicit enable action.
6. [x] Add localized labels/errors and Flutter tests for setting persistence, scheduling decisions, permission denial, quiet hours, zero-new and summary cases.
7. [ ] Run full P6A checks and build/install a Debug APK for emulator/device acceptance.

## Validation Record — 2026-08-29

- `cargo test -p papr-core -- --test-threads=1`: 82 passed.
- `cargo check -p papr-flutter-bridge`: passed; existing `frb_expand` warnings remain.
- Package-scoped `cargo fmt --check`: passed. Workspace-wide formatting remains blocked by unrelated pre-existing `src-tauri` formatting drift.
- `flutter test`: 28 passed; `flutter analyze`: no issues.
- `flutter build apk --debug`: passed; APK installed on `emulator-5554` without clearing data.
- Android registered one network-constrained periodic WorkManager job. A foreground-created run and a run after `MainActivity` became invisible both returned `SUCCESS` and called `jobFinished`; cancel/re-enable left one next-period job.
- Emulator notification permission remained denied and notification preference remained off, so refresh completed without a notification. Pure tests cover permission-independent eligibility, zero-new suppression, fixed 22:00–08:00 quiet hours, localized summary text, and the 15–120 minute Android scheduling bounds.
- Remaining device acceptance: allow notifications and observe one real summary, verify quiet-hour suppression with device time, and verify a due refresh after Android naturally reclaims the app process.

## Validation Record — 2026-08-30 Notification Icon Fix

- Root cause: `AndroidInitializationSettings('ic_launcher')` referenced a name
  available only under `res/mipmap`, while `flutter_local_notifications`
  validates its default icon exclusively as a `drawable`; enabling the setting
  therefore failed with `PlatformException(invalid_icon, ...)` before the
  permission request.
- Added the app-owned `res/drawable/ic_notification.xml` white RSS silhouette,
  shared its resource name through `androidNotificationIconName`, and added a
  regression that proves the configured drawable exists.
- Pre-fix resource probe failed for `drawable/ic_launcher`; the same probe now
  resolves `drawable/ic_notification` successfully.
- `flutter test`: 29 passed; targeted background refresh service tests: 4
  passed; `flutter analyze`: no issues; `flutter build apk --debug`: passed.
- `aapt2 dump resources` confirms `drawable/ic_notification` is packaged as
  `res/drawable/ic_notification.xml` in the Debug APK.
- Emulator acceptance on `emulator-5554`: with `POST_NOTIFICATIONS` initially
  denied, tapping New article notifications opened the Android permission
  dialog without `invalid_icon` or `PlatformException`; allowing permission
  enabled the switch and the dependent quiet-hours control.
- The enabled preference and Android permission survived an app process
  restart. With permission already granted, off then on changed the switch
  cleanly, left it enabled, and retained exactly one network-constrained
  `BackgroundWorker` job.
- A namespaced `jobscheduler run` reached WorkManager, which correctly delayed
  the periodic work because its next run time had not arrived and replaced it
  with one rescheduled job. A real due refresh that inserts new articles, the
  resulting summary notification, and quiet-hour suppression remain pending.

## Validation Commands

```powershell
cargo fmt --check
cargo test -p papr-core
cargo check -p papr-flutter-bridge

Set-Location mobile
flutter pub get
flutter test
flutter analyze
flutter build apk --debug
```

Use WorkManager's Android debugging commands or the plugin's debug hook to run the unique task immediately, then inspect `adb logcat` and the database-backed article list. Repeat after force-stopping/dismissing the app, with notification permission denied, and during a temporarily simulated quiet-hour clock.

## Review Gates

- After steps 1–2: Core tests must prove manual and due refresh cannot be confused.
- After step 4: a headless run must open the same database path and must not require `MainActivity` to exist.
- After step 5: every successful settings write must reconcile the unique task; a failed write must leave the previous schedule/UI state.
- Before completion: confirm no URL, article body, credential, or provider response is emitted to logs or notification text.

## Rollback Points

- Core due selection is independent and may ship before Android scheduling.
- If a plugin prevents build or headless FRB initialization, remove only the platform adapter/dependencies and keep foreground/manual behavior intact; do not add a placeholder Worker that merely waits for the next app launch.
