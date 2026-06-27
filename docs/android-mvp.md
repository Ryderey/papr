# Android APK MVP Plan

Status: planning only. Do not implement this plan until the user explicitly approves implementation.

Last reviewed: 2026-06-27

## Goal

Build a first Android APK for Papr that can be installed locally and used independently on Android 10 or newer.

The first release should preserve the core RSS reading loop:

- Local SQLite data on the phone.
- Add, remove, organize, and refresh RSS subscriptions.
- Article list, reader, full-text extraction, starred, read later, tags, and rules.
- FreshRSS or Miniflux sync for read state.
- Bring-your-own-key remote AI summaries, Q&A, and translation.
- Basic foreground audio playback.

The first release is not a Play Store release. It targets a local debug or locally installable APK first.

## Confirmed Product Decisions

- Target platform: Android APK only.
- Minimum supported Android version: Android 10+.
- First orientation target: phone portrait.
- Data model: fully local and independent on Android.
- No direct desktop SQLite migration.
- Subscription migration: OPML.
- Read state sync: existing FreshRSS or Miniflux support.
- UI approach: single-column mobile shell with bottom navigation and back-stack style flow.
- Build environment: Windows local machine with Android Studio or Android SDK/NDK, JDK, Rust Android targets, pnpm, and Tauri CLI.
- Implementation should use existing dependencies where possible.
- No new UI framework should be introduced for the MVP.

## Non-Goals For The First Android APK

The following features are intentionally out of scope for the first Android APK:

- Google Play release, AAB, formal production signing, or update channel.
- Desktop tray integration.
- Autostart.
- Desktop auto-updater.
- Background auto-refresh.
- Notifications.
- System share target from browsers or other apps.
- Android deep link or intent integration.
- In-app original-page child webview.
- Desktop browser extension deep-link flow.
- Newsletter or IMAP subscription sources.
- Send to Kindle or SMTP sending.
- Background audio playback.
- Lock-screen media controls.
- Headphone media buttons.
- Android file picker or full Storage Access Framework integration.

## Why Not web-to-app

The `shiahonb777/web-to-app` project is not the primary implementation route for Papr's Android MVP.

It is suitable for packaging a URL, static web app, media project, or a local runtime-backed web app into an Android WebView APK. Papr is different: the frontend relies heavily on Tauri IPC and Rust commands for the local database, feed fetching, AI calls, OPML, sync, and other application behavior.

Relevant project facts:

- `src/api.ts` imports `invoke` and `Channel` from `@tauri-apps/api/core`.
- Most frontend operations call Tauri commands rather than HTTP endpoints.
- `src-tauri/src/lib.rs` registers the Rust command surface and initializes local app state.
- `src-tauri/Cargo.toml` contains the SQLite, network, parsing, sync, and desktop integration dependencies.

Packaging only the Vite `dist` output in a generic Android WebView would not provide the Rust command surface. Making `web-to-app` work would require converting the Rust backend into an HTTP service or otherwise rewriting the app boundary, which is larger than using Tauri Mobile.

Preferred route: Tauri 2 Android Mobile.

Useful references:

- Tauri mobile development: https://v2.tauri.app/develop/
- Tauri CLI reference: https://v2.tauri.app/reference/cli/
- Tauri deep linking: https://v2.tauri.app/plugin/deep-linking/
- web-to-app repository: https://github.com/shiahonb777/web-to-app

## Current Codebase Signals

These existing project details make Tauri Android the lowest-risk route:

- `package.json` already uses Tauri 2 packages.
- `src-tauri/src/lib.rs` already marks the app entry with `#[cfg_attr(mobile, tauri::mobile_entry_point)]`.
- `src-tauri/Cargo.toml` has `crate-type = ["staticlib", "cdylib", "rlib"]`, which is compatible with mobile builds.
- Android and iOS icon folders already exist under `src-tauri/icons`.

These existing project details also show why the mobile MVP needs explicit work:

- `src-tauri/tauri.conf.json` is desktop-window oriented, including desktop minimum dimensions.
- `src/styles.css` uses a fixed desktop three-column layout: sidebar, article list, reader.
- `src/App.tsx` includes desktop keyboard shortcuts, tray events, updater checks, and desktop window background handling.
- `src/api.ts` exposes desktop-specific commands such as tray refresh and in-app page view.
- `src-tauri/src/lib.rs` initializes tray, scheduler, notifications, deep links, and desktop plugins.

## Target Mobile UX

The Android MVP should not shrink the desktop three-column layout onto a phone.

Use a single-column shell:

- Library: smart views, subscriptions, folders, tags.
- Articles: article list for the selected view.
- Reader: article reading view.
- Settings: mobile settings page.

Navigation:

- Selecting a smart view, feed, folder, or tag opens Articles.
- Selecting an article opens Reader.
- Reader has a top back action to Articles.
- Articles has a path back to Library.
- Settings is accessible from mobile navigation.

Desktop layout should remain available for desktop builds.

## Feature Scope

### Keep In MVP

- Local database.
- Feed CRUD.
- Folder CRUD.
- Tags and tag assignment.
- Rules.
- Manual and startup foreground refresh.
- Article list and reader.
- Full-text extraction.
- Starred and read-later state.
- Read/unread state.
- FreshRSS and Miniflux sync.
- Remote BYOK AI summaries, Q&A, digest, and translation.
- External browser opening for original URLs.
- OPML text import and text export.
- Basic foreground audio playback.

### Degrade In MVP

- Original-page in-app view: open in external browser.
- OPML import/export: text paste and copy, not file picker.
- Audio: foreground only.
- Settings: single-column layout, mobile-safe controls.

### Hide Or Disable In MVP

- Autostart.
- Desktop updater.
- Tray.
- Desktop window and titlebar controls.
- Desktop shortcut-first help text.
- Background refresh.
- Notifications.
- Android system share target.
- Android deep links.
- Newsletter and IMAP sources.
- Send to Kindle and SMTP.

## Implementation Phases

### M1: Android Project Initialization

Objective: create and run the Android Tauri project locally.

Tasks:

- Confirm local requirements: Android Studio or SDK/NDK, JDK, Rust Android targets, pnpm, Tauri CLI.
- Run Tauri Android initialization.
- Build or run a basic Android target.
- Document local environment issues and exact commands.

Likely commands:

```powershell
pnpm tauri android init
pnpm tauri android dev
```

Acceptance criteria:

- Android project files exist.
- Tauri Android dev build starts on emulator or device.
- Any environment requirements are documented.

Expected touched areas:

- `src-tauri/gen/android/**`
- `src-tauri/tauri.conf.json`
- Documentation.

### M2: Mobile Compile Gating For Rust And Tauri

Objective: make the Rust/Tauri backend compile for Android without desktop-only features.

Tasks:

- Gate desktop-only plugins behind `#[cfg(desktop)]` or equivalent build conditions.
- Review plugin availability on mobile before including each plugin.
- Disable tray setup on mobile.
- Disable desktop updater and process relaunch on mobile.
- Disable autostart on mobile.
- Disable window-state behavior on mobile if unsupported or irrelevant.
- Disable page view commands on mobile or provide no-op/error responses that the UI will not expose.
- Disable newsletter/IMAP if it blocks Android compilation.
- Disable Send to Kindle/SMTP if it blocks Android compilation.
- Disable notification and badge code for the MVP if it increases Android risk.
- Keep SQLite, feed fetching, full-text extraction, AI HTTP requests, OPML, sync, tags, and rules.

Acceptance criteria:

- Android Rust build passes.
- Desktop build still passes.
- Disabled mobile features fail gracefully or are not reachable from the mobile UI.

Expected touched areas:

- `src-tauri/Cargo.toml`
- `src-tauri/src/lib.rs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/page_view.rs`
- `src-tauri/src/notify.rs`
- `src-tauri/src/tray.rs`
- Newsletter and SMTP related modules if needed.

### M3: Android Permissions And Runtime Behavior

Objective: ensure the APK can perform core runtime operations.

Tasks:

- Confirm Android internet permission.
- Confirm app data directory path and SQLite creation.
- Confirm HTTP and HTTPS feed fetching.
- Confirm image loading and proxied image fetching.
- Confirm external browser opening.
- Confirm AI HTTPS requests.
- Confirm FreshRSS or Miniflux requests.
- Set or document Android minimum SDK for Android 10+.

Acceptance criteria:

- First launch creates the local database.
- Feed fetching works on device or emulator.
- External links open outside the app.
- Restarting the app preserves data.

Expected touched areas:

- Android manifest or generated Android config.
- `src-tauri/tauri.conf.json`
- Rust app setup only if mobile path differences appear.

### M4: Mobile Platform And Capability Model

Objective: expose a simple frontend way to detect mobile and feature availability.

Tasks:

- Add or extend platform detection.
- Define capabilities such as `canUseTray`, `canUseAutostart`, `canUseUpdater`, `canUsePageView`, `canUseNewsletter`, `canUseSendToKindle`, `canUseNotifications`, and `canUseFilePicker`.
- Use capabilities to hide or disable UI affordances.
- Keep the desktop behavior unchanged.

Acceptance criteria:

- Mobile build does not show first-release unsupported actions.
- Desktop build still shows existing desktop features.
- Unsupported features are not just visually hidden while still triggerable by command palette or shortcuts.

Expected touched areas:

- `src/lib/platform.ts`
- `src/App.tsx`
- `src/components/CommandPalette.tsx`
- `src/components/SettingsDialog.tsx`
- Related components that expose unsupported actions.

### M5: Mobile App Shell And Navigation

Objective: implement a usable phone portrait layout.

Tasks:

- Keep the desktop shell for desktop widths/platforms.
- Add a mobile shell that switches between Library, Articles, Reader, and Settings.
- Preserve selection state in the existing store where possible.
- Add mobile back actions.
- Ensure opening an article moves to the reader view.
- Ensure selecting a feed/folder/tag/smart view moves to articles view.
- Ensure settings opens as a full-page mobile view or mobile-safe modal.

Acceptance criteria:

- Phone portrait layout is single-column.
- Core navigation does not require desktop keyboard shortcuts.
- No essential controls are off-screen.
- Desktop three-column layout remains intact.

Expected touched areas:

- `src/App.tsx`
- `src/store.ts`
- `src/components/Sidebar.tsx`
- `src/components/ArticleList.tsx`
- `src/components/Reader.tsx`
- `src/components/SettingsDialog.tsx`
- `src/styles.css`

### M6: Mobile CSS And Touch Usability

Objective: make the mobile shell usable and readable on phone screens.

Tasks:

- Add mobile media queries or platform-scoped mobile styles.
- Replace fixed three-column layout with mobile single-column layout.
- Handle `100vh` or dynamic viewport issues on Android.
- Add safe-area padding where needed.
- Increase touch targets where needed.
- Avoid hover-only interactions on mobile.
- Hide hover previews on touch devices.
- Make modal/dialog sizes mobile-safe.
- Make reader toolbar and bottom navigation not overlap content.
- Validate text does not overflow buttons or controls.

Acceptance criteria:

- Usable on a phone portrait viewport.
- No critical overlap between toolbar, bottom nav, player, and content.
- Reader content is comfortable to scroll and read.
- Settings page remains reachable and scrollable.

Expected touched areas:

- `src/styles.css`
- Components with inline layout assumptions.

### M7: OPML Text Import And Export

Objective: support low-dependency OPML transfer on Android.

Tasks:

- Add mobile OPML import by text paste.
- Add mobile OPML export by displaying generated OPML text.
- Add copy-to-clipboard for exported OPML if available.
- Keep existing desktop import/export path unchanged if present.

Acceptance criteria:

- User can paste OPML text and import subscriptions.
- User can export OPML text and copy it.
- No Android file picker dependency is required.

Expected touched areas:

- `src/components/SettingsDialog.tsx`
- `src/api.ts` only if wrapper changes are needed.

### M8: AI And Sync Verification

Objective: verify retained network-backed features on Android.

Tasks:

- Test AI connection setup from mobile settings.
- Test AI summary streaming.
- Test AI follow-up Q&A.
- Test translation.
- Test FreshRSS or Miniflux connect, status, and sync.
- Confirm API keys and settings persist locally.

Acceptance criteria:

- Remote AI works with BYOK settings.
- Streaming UI updates correctly.
- FreshRSS or Miniflux sync works or documented blockers are identified.

Expected touched areas:

- Mostly validation.
- Settings UI if mobile layout issues appear.
- Rust network code only if Android-specific failures appear.

### M9: APK Packaging And Manual QA

Objective: produce and verify a locally installable APK.

Tasks:

- Build debug APK.
- Install on emulator or Android 10+ physical device.
- Run manual test matrix.
- Record APK location and build command.
- Document known limitations.

Likely commands:

```powershell
pnpm tauri android build
```

Acceptance criteria:

- APK installs locally.
- APK launches and completes core reading flow.
- Limitations are documented.

Expected touched areas:

- Documentation.
- Any final mobile-specific bug fixes.

## Validation Plan

Run narrow validation after each implementation phase, then broader validation before APK handoff.

Core checks:

```powershell
pnpm build
pnpm test
```

Rust checks, depending on what is available after Android init:

```powershell
cd src-tauri
cargo test
```

Android checks:

```powershell
pnpm tauri android dev
pnpm tauri android build
```

Manual QA checklist:

- Install APK on Android 10+ device or emulator.
- Launch app for the first time.
- Confirm local database is created.
- Add an RSS feed.
- Refresh feeds.
- Open article list.
- Open article reader.
- Mark read and unread.
- Star article.
- Toggle read later.
- Add and remove tag.
- Create and apply rule.
- Extract full text.
- Open original article in external browser.
- Import OPML by pasted text.
- Export OPML as text.
- Configure AI provider.
- Test AI connection.
- Generate AI summary.
- Ask follow-up question.
- Translate article.
- Connect FreshRSS or Miniflux.
- Sync and verify read state.
- Restart app and confirm data persists.
- Rotate or resume app and confirm no critical state loss.
- Verify phone portrait layout has no unreachable critical controls.

## Risk Register

### Android Cross-Compilation

Risk: Rust crates or Tauri plugins may fail under Android targets.

Mitigation:

- Gate desktop-only code.
- Disable non-core features for MVP.
- Keep retained dependencies as close to existing code as possible.

### Desktop Regression

Risk: mobile gating could accidentally remove desktop behavior.

Mitigation:

- Keep desktop code paths explicit.
- Validate desktop build and tests.
- Avoid broad refactors.

### Mobile UX Regression

Risk: desktop components may be technically visible but unpleasant or broken on a phone.

Mitigation:

- Add a mobile shell instead of compressing the desktop grid.
- Test phone portrait early.
- Hide hover-only interactions.

### Background Expectations

Risk: users may expect automatic background refresh and notifications.

Mitigation:

- Document that the MVP uses foreground/manual refresh.
- Add background refresh as a later milestone.

### File Handling

Risk: Android file picker and export flows can add platform complexity.

Mitigation:

- Use text OPML import/export first.
- Add Storage Access Framework support later if needed.

### Deep Link And Share Target Complexity

Risk: Android intents and cold-start routing can expand scope.

Mitigation:

- Defer system share and deep links.
- Keep manual add-feed URL entry.

## Later Milestones

After the Android APK MVP is stable, consider these follow-ups:

- Android file picker for OPML import.
- Android share sheet export for OPML.
- Android system share target to add feed URLs.
- Mobile deep links.
- Background refresh using Android-appropriate scheduling.
- Notifications and notification permission flow.
- Background audio playback and media controls.
- Production signing and release APK.
- AAB and Google Play release.
- iOS feasibility pass.
- Tablet and landscape layout.
- Desktop-to-mobile migration tooling if still needed.

## Implementation Order Summary

1. Initialize Android project and confirm environment.
2. Gate desktop-only Rust/Tauri code.
3. Confirm Android database, network, and app-data behavior.
4. Add mobile capability model.
5. Add single-column mobile shell.
6. Add mobile CSS and touch fixes.
7. Add OPML text import/export.
8. Verify AI and FreshRSS/Miniflux.
9. Build APK and run manual QA.

## Stop Conditions

Pause implementation and ask for review if any of these occur:

- A retained core feature requires adding a major new framework.
- Android build requires removing a core MVP feature.
- SQLite cannot run reliably in Android app data.
- Feed fetching fails because of a broad Android networking issue.
- Tauri Mobile cannot support the current IPC/channel pattern without significant redesign.
- Implementation would require changing the desktop app architecture broadly.

