# Implementation

- [x] Inspect branch divergence and shared Core dependencies.
- [x] Merge into isolated codex/integrate-desktop-mobile worktree and resolve AGENTS conflict.
- [x] Move existing Release profile to workspace root.
- [x] Desktop frontend build and 88 tests; Flutter analyze and 34 tests.
- [x] Rust final serial gate: desktop 262, Core 103 (including populated desktop v15 upgrade), Bridge 3 tests pass. Initial parallel run had two transient Core AI network fixture failures; serial rerun passed.
- [x] FRB 2.12.0 regeneration produces no generated-file diff. All four Rust Android ABIs compile; Android Debug APK builds using the cross-drive workaround below.
- [x] Complete integration commit and fast-forward optimize-bugfix; create codex/github-personal-sync.
- [x] Shared protocol/model, reducer, identities, retention, and malformed-data checks.
- [x] Append-only migrations, transactional capture/outbox, catalog import/hydration, and recovery.
- [x] GitHub transport, atomic publication, rate-limit handling, and fault injection tests.
- [x] Windows secure credentials, settings, status, preview, manual/automatic orchestration.
- [x] Bridge generation, Android secure credentials, settings, and foreground/background orchestration.
- [x] Full integration gates, contract acceptance, documentation, and accurate external-validation limitations.

## Validation commands

`cargo test -p papr-core -p papr-flutter-bridge -p papr --offline -- --test-threads=1`, `pnpm build`, `pnpm test`, `flutter analyze --no-pub`, `flutter test --no-pub`, `flutter build apk --debug --no-pub`. Regenerate FRB through the existing YAML config and compare generated files; installed 2.12.0 CLI has no --check option.

## Boundaries

Do not read signing keys or user tokens, change the original checkout, publish releases, reset branches, force-push, clear user databases, or create remote repositories. Stop treating unavailable real-network/device evidence as an automated-test substitute.

## Integration build evidence

Initial `flutter build apk --debug --no-pub` failed in WorkManager Kotlin compilation because the worktree is on C: and the pub cache is on D:; Kotlin could not relativize source paths while closing incremental caches. Retried from `mobile/android` with `./gradlew.bat assembleDebug '-Pkotlin.incremental=false' '-Pkotlin.compiler.execution.strategy=in-process'`: BUILD SUCCESSFUL (215 tasks, 4m 8s). These are temporary command-line parameters; no Gradle or environment settings were changed. PowerShell requires quoting the dotted -P arguments; an initial unquoted attempt was rejected as an unknown task. APK installation/device behavior was not tested in this integration phase.

## Feature gate — 2026-10-05

- Integrated baseline: 71176d7 on local optimize-bugfix. Implementation branch: codex/github-personal-sync.
- Repository target verified read-only: Ryderey/papr-sync; private; repository ID 1405209831; default branch main. No remote sync writes or pushes performed.
- Final Rust serial regression: desktop 265, Core 137, Bridge 3 tests pass. Core includes 34 GitHub protocol/storage/transport/fault tests. After the final duplicate-connect credential protection change, both isolated Windows credential tests, Flutter analyze and all 34 Flutter tests pass again.
- Desktop: pnpm test 88 tests pass; production frontend build passes; pnpm tauri build --debug --no-bundle succeeds.
- Android: Flutter analyze reports no issues; all 34 existing Flutter tests pass. FRB regeneration after all API additions changes none of the generated Rust/Dart files.
- Final standard Android build: all four Rust release ABIs and Debug APK build successfully (8m 38s, 215 tasks). After the adapter-only credential protection change, the standard APK was rebuilt in 10 seconds using the already verified native libraries (-x buildRustBridge); its normal application ID was verified and the preserved artifact updated. Use that standard-package artifact below.
- Independent emulator package com.papr.papr_mobile.sync_smoke_20261005 installs and launches on emulator-5554 (sdk_gphone16k_x86_64). Foreground Keystore round trip and GitHub bridge status pass. A one-off headless WorkManager callback reads/deletes the non-secret marker and initializes Core/GitHub status successfully. Both PAPR_SYNC_SMOKE_FOREGROUND_PASS and PAPR_SYNC_SMOKE_WORKER_PASS observed. Reader sync/GitHub form renders without runtime exceptions. Existing com.papr.papr_mobile was not upgraded or cleared.
- Emulator smoke source: validation/android_smoke.dart. Build uses an isolated application ID via a temporary Gradle init script and skips rebuilding already verified native libraries; no production Gradle/signing configuration changed.
- Real app-level GitHub publication/convergence, production GitHub-connected Worker, physical Android, Doze/reboot behavior and token-expiry acceptance remain pending (A25). Task remains in_progress for that manual acceptance; no archive or feature-branch merge into main yet.
- Full evidence and known UI/history-management limits: docs/github-sync-implementation-2026-10-05.md.

## Review artifacts

- Windows executable: target/debug/papr.exe — SHA256 4831e8d312fd2d9907391ff72763c53da7de8e07521d1cb89098735caee98afe.
- Standard Android APK: mobile/build/github-sync/papr-sync-debug.apk — package com.papr.papr_mobile, version 0.1.0+1; SHA256 02e4f3a2b343079140454b0ff3229f907043cf5673e0ec851a6893ecaa8714f9.
- Independent emulator test APK: mobile/build/github-sync/papr-sync-emulator-smoke.apk; this is a different package/entrypoint and is not the user's install artifact.
- Emulator UI screenshot: mobile/build/github-sync/android-github-settings.png.
- Debug APK signing/version may differ from the user's installed app. Do not uninstall the real app to resolve an upgrade mismatch; preserve its database and finish the existing signing/version release work separately.
- Existing FRB cfg, Gradle deprecation, and frontend bundle-size warnings remain; no unrelated dependency/configuration changes were introduced.
