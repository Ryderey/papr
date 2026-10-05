# Integrate desktop and Android, then implement GitHub personal sync

## Goal

Integrate the desktop main branch and the Android branch, then provide private GitHub repository sync for one user's Windows and Android Papr clients.

## Requirements

- Preserve desktop changes from optimize-bugfix and Android changes from feat/flutter-android-rearchitecture; preserve the original working tree's uncommitted files.
- Share protocol, merge rules, migrations, and HTTP transport in papr-core; use thin desktop and Flutter adapters.
- Sync subscriptions, flat folders, article titles/links/source/date/GUID, and read/starred/read-later state. Retain ordinary cloud catalog entries for 90 days and protected entries indefinitely.
- Keep bodies, themes, fonts, AI credentials/configuration, IMAP accounts, highlights, tags, and rules local.
- Preserve offline writes, explicit false states, atomic publication, and recovery after uncertain network results. Never force-push or rewrite remote history.
- Store credentials using Windows Credential Manager and the existing Android Keystore plugin; never persist tokens in SQLite or logs.
- Preserve the existing GReader backend and prevent simultaneous active backends.

## Acceptance Criteria

- [x] Integrated desktop build/tests, Core/Bridge tests, Flutter analyze/tests, and Android debug build pass; preserve migrated desktop and Android databases.
- [x] All contract scenarios A01-A24 in docs/github-personal-sync-design-2026-10-05.md have automated coverage or explicit evidence.
- [x] Windows and Android expose connection, preview, sync, credential update, status, and disconnect flows.
- [x] Generated bridge files match the bridge source.
- [x] Real private-repository and device/background checks (A25) are recorded accurately; do not claim checks without available credentials/devices.

## Background and constraints

The user approved the GitHub design and the order: integrate first, then develop sync. Local refs initially diverged by 12 desktop-only and 62 Android-only commits. The shared Core exists only on the Android branch. The original checkout contains unrelated Android signing/release work, which must remain untouched. Scope is personal use, Windows and Android only; no server provisioning or app store release.

## Remaining manual acceptance

A25 real repository publication and physical-device/background convergence remain open. The emulator smoke confirms platform credential/bridge availability, not a real GitHub round trip. Keep the task active until those checks are recorded.

## Static-review follow-up

- [x] Resolve the eight review findings and add focused regressions for initialization/checkpoint adoption, receipt recovery, folder ID reuse/local-only membership, future dates, credential scheduling/errors and desktop cache/count consistency.
- [x] Full Rust/frontend/Flutter checks pass and generated bridge reproducibility passes.
- [x] Updated Android APK and final artifact verification recorded.

See docs/github-sync-review-fixes-2026-10-05.md. A25 remains independently pending.


## Second static-review follow-up

- [x] Fix all six second-pass findings: mounted Android projections, canonical duplicate connection, restored catalog age, initializer credential validation, repaired repository-access scheduling and persisted rate-limit clock consistency.
- [x] Targeted regressions and full Rust/frontend/Flutter gates pass, including pagination/exhaustion and optimistic editing protection.
- [x] Refresh native artifacts and record the final generation/build gates.

A25 actual GitHub/account and physical-device acceptance remains independently pending.
