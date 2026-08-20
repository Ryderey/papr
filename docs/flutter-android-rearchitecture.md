# Flutter Android Rearchitecture

## Status

Draft design for the `feat/flutter-android-rearchitecture` branch.
Replaces `docs/android-mvp.md`, which targeted Tauri Mobile and is no longer the chosen route.

## Background

Papr is currently a Tauri v2 desktop RSS reader with a React/TypeScript frontend and a Rust backend (`src-tauri/src/`).
An earlier MVP plan proposed building an Android APK using Tauri Mobile (`docs/android-mvp.md`).
That approach was evaluated and rejected because the Tauri Mobile developer experience and runtime behaviour were unacceptable for this project.

This document records the decision to rebuild the Android experience with Flutter while reusing the existing Rust business logic.

## Goals

1. Build a first Flutter Android app that can run independently on Android 10+.
2. Reuse the existing Rust backend instead of rewriting the RSS/AI/sync logic in Dart.
3. Keep the Flutter work in the same repository as an independent module; do not force UI code sharing with the desktop React app.
4. Maintain the existing desktop app unchanged; all mobile work must be additive or platform-isolated.
5. Produce a demonstrable minimal reading loop in phase 1: feed list → article list → article detail.

## Non-Goals

- iOS support in phase 1.
- Replacing the desktop app or sharing UI between desktop and mobile.
- Rewriting business logic in Dart.
- Play Store release, AAB, production signing, or update channel.
- Background refresh, notifications, deep links, system share targets, file pickers.
- Newsletter/IMAP sources, Send to Kindle, SMTP.
- Lock-screen media controls, background audio.

## High-Level Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                        Desktop                               │
│  React/TypeScript (src/)  ↔  Tauri adapter (src-tauri/)    │
│                               ↓                              │
│                         papr-core                            │
└─────────────────────────────────────────────────────────────┘
                              ↑
┌─────────────────────────────────────────────────────────────┐
│                        Mobile (Android)                      │
│  Flutter (mobile/)  ↔  FRB bridge (crates/papr-flutter-bridge) │
│                               ↓                              │
│                         papr-core                            │
└─────────────────────────────────────────────────────────────┘
```

Dependency direction is fixed:

```text
src-tauri                  ──┐
crates/papr-flutter-bridge ──┼──> crates/papr-core
                             │
papr-core does not depend on any adapter
```

## Module Definitions

### `crates/papr-core`

Pure Rust business logic. No dependencies on Tauri, Flutter, Dart, WebView, windowing, tray, notifications, auto-updater, deep links, or system plugins.

Responsibilities:
- Domain models and DTOs.
- SQLite data access (`db.rs`).
- Feed ingestion: fetching, parsing, discovery, deduplication, storage (`ingestion/`).
- OPML import/export.
- FreshRSS / Miniflux sync.
- AI provider calls, request construction, response parsing, error normalization.
- Translation.
- Tags, rules, automatic classification.
- General utilities (`feed_utils.rs`, `html.rs`).
- Decoupled application state (no Tauri `State`, no Flutter singleton).

Constraints:
- Platform paths (`data_dir`, `database_path`, `cache_dir`, `log_dir`) are injected by adapters; `papr-core` does not resolve them.
- Credential storage is handled by adapters in phase 1; core receives sensitive values as plain strings when needed.
- Async streaming outputs are modelled with platform-agnostic event types (e.g. `AiStreamEvent`), converted to Tauri events or Dart streams by adapters.

### `src-tauri` (logical `papr-tauri`)

Tauri adapter layer. The physical directory remains `src-tauri/` to preserve Tauri CLI defaults.

Responsibilities:
- Tauri command entry points (`commands.rs`).
- Tray, notifications, page view, window management.
- Deep links, autostart, auto-updater, plugin setup.
- `TauriAppState` holding an `Arc<PaprCore>` plus Tauri-specific handles.

Constraints:
- `commands.rs` must be thin: parameter conversion, call core, return result.
- No business logic lives here in the long term.

### `crates/papr-flutter-bridge`

Flutter Rust Bridge v2 adapter.

Responsibilities:
- Expose a minimal, explicit Rust API in `src/api.rs`.
- Hold `PaprCore` inside an FRB opaque wrapper (`PaprCoreBridge`).
- Map `papr-core` errors to `PaprBridgeError`.
- Convert DTOs for FRB compatibility.

Constraints:
- No business logic.
- `papr-core` must not depend on `flutter_rust_bridge`.
- Codegen is manual; generated Dart bindings are committed.

### `mobile/`

Flutter application.

Responsibilities:
- Android (and later iOS) UI.
- State management with Riverpod.
- Service/repository wrappers around FRB generated bindings.
- Platform path construction with `path_provider`.

Constraints:
- FRB generated bindings are consumed through a thin service layer before reaching UI providers.
- No direct FRB calls from widgets.

## Directory Layout

```text
papr/
├── Cargo.toml                       # Rust workspace root
├── package.json
├── pnpm-workspace.yaml
├── docs/
│   └── flutter-android-rearchitecture.md   # this document
│
├── crates/
│   ├── papr-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs            # PaprCoreConfig, Platform
│   │       ├── error.rs             # CoreError
│   │       ├── db.rs                # SQLite layer (phase-1 subset)
│   │       ├── dto.rs               # Phase-1 DTOs
│   │       └── services/
│   │           ├── mod.rs
│   │           ├── article.rs
│   │           ├── feed.rs
│   │           ├── ingestion.rs
│   │           ├── opml.rs
│   │           └── settings.rs
│   │
│   └── papr-flutter-bridge/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── api.rs               # FRB public API
│           └── error.rs             # PaprBridgeError
│
├── mobile/
│   ├── pubspec.yaml
│   ├── android/
│   ├── ios/
│   ├── lib/
│   │   ├── main.dart
│   │   ├── app.dart                 # MaterialApp + ProviderScope
│   │   ├── bridge/
│   │   │   └── generated/           # FRB generated files, committed
│   │   ├── core/
│   │   │   ├── config.dart          # PaprCoreConfig construction
│   │   │   ├── di.dart              # Riverpod providers
│   │   │   └── exceptions.dart      # AppException, AppErrorKind
│   │   ├── repositories/
│   │   │   ├── article_repository.dart
│   │   │   ├── feed_repository.dart
│   │   │   └── settings_repository.dart
│   │   ├── services/
│   │   │   └── papr_core_service.dart
│   │   └── ui/
│   │       ├── screens/
│   │       │   ├── feed_list_screen.dart
│   │       │   ├── article_list_screen.dart
│   │       │   ├── article_detail_screen.dart
│   │       │   └── settings_screen.dart
│   │       └── widgets/
│   └── rust/
│       └── frb_codegen_config.yaml  # FRB codegen configuration
│
├── src/                             # Existing React desktop frontend
├── src-tauri/                       # Existing Tauri project, becomes adapter
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── main.rs
│       ├── commands.rs
│       ├── tray.rs
│       ├── notify.rs
│       ├── page_view.rs
│       ├── window/
│       └── state.rs                 # TauriAppState
```

## Cargo Workspace

Root `Cargo.toml`:

```toml
[workspace]
members = [
    "src-tauri",
    "crates/papr-core",
    "crates/papr-flutter-bridge",
]
resolver = "2"
```

`src-tauri/Cargo.toml` depends on `papr-core`:

```toml
[dependencies]
papr-core = { path = "../crates/papr-core" }
```

`crates/papr-flutter-bridge/Cargo.toml` depends on `papr-core` and FRB:

```toml
[dependencies]
papr-core = { path = "../papr-core" }
flutter_rust_bridge = "2"
```

## Initialization Model

`papr-core` exposes:

```rust
#[derive(Debug, Clone)]
pub struct PaprCoreConfig {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub cache_dir: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub log_level: Option<String>,
    pub platform: Platform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Desktop,
    Android,
    Ios,
}

impl PaprCore {
    pub async fn new(config: PaprCoreConfig) -> Result<Self, CoreError>;
}
```

- Adapters inject absolute paths and platform type.
- Runtime config is immutable after construction.
- `PaprCore::new` validates paths, creates directories, initialises the database, runs migrations, and constructs services.
- User settings are mutable at runtime through `SettingsService` and stored in SQLite.

Flutter initialization flow:

```text
path_provider → build PaprCoreConfig → FRB init_papr_core(config)
→ hold PaprCoreBridge in Riverpod provider
```

Tauri initialization flow:

```text
Tauri path resolver → build PaprCoreConfig → PaprCore::new(config)
→ store Arc<PaprCore> in TauriAppState
```

## Core Services

Phase 1 services:

```rust
pub struct PaprCore {
    db: Arc<Db>,
    config: PaprCoreConfig,
    feed_service: FeedService,
    article_service: ArticleService,
    ingestion_service: IngestionService,
    opml_service: OpmlService,
    settings_service: SettingsService,
}

impl PaprCore {
    pub fn feed_service(&self) -> &FeedService;
    pub fn article_service(&self) -> &ArticleService;
    pub fn ingestion_service(&self) -> &IngestionService;
    pub fn opml_service(&self) -> &OpmlService;
    pub fn settings_service(&self) -> &SettingsService;
}
```

Service methods (phase 1):

```rust
impl FeedService {
    pub fn new(db: Arc<Db>) -> Self;
    pub async fn list_feeds(&self) -> Result<Vec<Feed>, CoreError>;
}

impl ArticleService {
    pub fn new(db: Arc<Db>) -> Self;
    pub async fn list_articles(&self, filter: ArticleFilter) -> Result<Vec<ArticleSummary>, CoreError>;
    pub async fn get_article_detail(&self, article_id: i64) -> Result<ArticleDetail, CoreError>;
}

impl IngestionService {
    pub fn new(db: Arc<Db>) -> Self;
    pub async fn refresh_feeds(&self, options: RefreshOptions) -> Result<RefreshReport, CoreError>;
}

impl OpmlService {
    pub fn new(db: Arc<Db>) -> Self;
    pub async fn import_text(&self, opml_text: String) -> Result<OpmlImportReport, CoreError>;
}

impl SettingsService {
    pub fn new(db: Arc<Db>) -> Self;
    pub async fn get_settings(&self) -> Result<SettingsSnapshot, CoreError>;
}
```

`Db::new` is synchronous; service methods are `async` for future concurrency flexibility.

## Core DTOs (Phase 1)

### `Feed`

```rust
#[derive(Debug, Clone)]
pub struct Feed {
    pub id: i64,
    pub feed_url: String,
    pub site_url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub folder_id: Option<i64>,
    pub source_type: SourceType,
    pub last_fetched_at: Option<String>,
    pub fetch_error: Option<String>,
    pub unread_count: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Rss,
    Youtube,
    Podcast,
    Mastodon,
    Bluesky,
    Reddit,
    Newsletter,
}
```

### `ArticleSummary`

```rust
#[derive(Debug, Clone)]
pub struct ArticleSummary {
    pub id: i64,
    pub feed_id: i64,
    pub feed_title: String,
    pub source_type: SourceType,
    pub title: String,
    pub author: Option<String>,
    pub snippet: Option<String>,
    pub image_url: Option<String>,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub read_later: bool,
}
```

### `ArticleDetail`

```rust
#[derive(Debug, Clone)]
pub struct ArticleDetail {
    pub id: i64,
    pub feed_id: i64,
    pub feed_title: String,
    pub source_type: SourceType,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub content_html: Option<String>,
    pub extracted_html: Option<String>,
    pub image_url: Option<String>,
    pub published_at: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub read_later: bool,
    pub ai_summary: Option<String>,
    pub translated_html: Option<String>,
    pub translated_lang: Option<String>,
    pub enclosures: Vec<Enclosure>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone)]
pub struct Enclosure {
    pub url: String,
    pub mime_type: Option<String>,
    pub length: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}
```

### `ArticleFilter`

```rust
#[derive(Debug, Clone)]
pub struct ArticleFilter {
    pub kind: ArticleFilterKind,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub enum ArticleFilterKind {
    All,
    Unread,
    Starred,
    ReadLater,
    Feed(i64),
    Folder(i64),
    Tag(i64),
}
```

### `RefreshOptions` / `RefreshReport`

```rust
#[derive(Debug, Clone)]
pub struct RefreshOptions {
    pub feed_ids: Option<Vec<i64>>,
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct RefreshReport {
    pub total_feeds: i64,
    pub new_articles: i64,
    pub errors: Vec<RefreshError>,
}

#[derive(Debug, Clone)]
pub struct RefreshError {
    pub feed_id: i64,
    pub message: String,
}
```

### `OpmlImportReport`

```rust
#[derive(Debug, Clone)]
pub struct OpmlImportReport {
    pub imported_feeds: i64,
    pub failed_feeds: i64,
    pub errors: Vec<String>,
}
```

### `SettingsSnapshot`

```rust
#[derive(Debug, Clone)]
pub struct SettingsSnapshot {
    pub theme: String,
    pub language: String,
    pub refresh_interval_min: i64,
}
```

Defaults when keys are missing:
- `theme`: `"system"`
- `language`: `"en"`
- `refresh_interval_min`: `30`

## Error Model

### `papr-core`

```rust
#[derive(thiserror::Error, Debug, Clone)]
pub enum CoreError {
    #[error("database error: {0}")]
    Db(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("AI error: {0}")]
    Ai(String),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("unknown error: {0}")]
    Unknown(String),
}
```

### `papr-flutter-bridge`

```rust
#[derive(Debug)]
#[frb]
pub enum PaprBridgeError {
    Database { message: String },
    Network { message: String },
    Parse { message: String },
    InvalidInput { message: String },
    NotFound { message: String },
    Ai { message: String },
    Platform { message: String },
    Unknown { message: String },
}

impl From<papr_core::CoreError> for PaprBridgeError {
    fn from(e: papr_core::CoreError) -> Self {
        let message = e.to_string();
        match e {
            CoreError::Db(_) => Self::Database { message },
            CoreError::Network(_) => Self::Network { message },
            CoreError::Parse(_) => Self::Parse { message },
            CoreError::InvalidInput(_) => Self::InvalidInput { message },
            CoreError::NotFound(_) => Self::NotFound { message },
            CoreError::Ai(_) => Self::Ai { message },
            CoreError::Platform(_) => Self::Platform { message },
            CoreError::Unknown(_) => Self::Unknown { message },
        }
    }
}
```

### Dart

```dart
enum AppErrorKind {
  database,
  network,
  parse,
  invalidInput,
  notFound,
  ai,
  platform,
  unknown,
}

class AppException implements Exception {
  final AppErrorKind kind;
  final String message;
  AppException(this.kind, this.message);
}
```

## FRB Bridge API (Phase 1)

`crates/papr-flutter-bridge/src/api.rs`:

```rust
use std::sync::Arc;
use flutter_rust_bridge::frb;
use papr_core::{
    PaprCore, PaprCoreConfig,
    ArticleFilter, ArticleSummary, ArticleDetail,
    Feed, RefreshOptions, RefreshReport,
    OpmlImportReport, SettingsSnapshot,
};
use crate::error::PaprBridgeError;

#[frb(opaque)]
pub struct PaprCoreBridge {
    inner: Arc<PaprCore>,
}

#[frb(init)]
pub fn init_app() {
    // Optional logger / panic hook setup.
}

pub async fn init_papr_core(
    config: PaprCoreConfig,
) -> Result<PaprCoreBridge, PaprBridgeError> {
    let core = PaprCore::new(config).await?;
    Ok(PaprCoreBridge {
        inner: Arc::new(core),
    })
}

pub async fn get_feeds(
    core: &PaprCoreBridge,
) -> Result<Vec<Feed>, PaprBridgeError> {
    Ok(core.inner.feed_service().list_feeds().await?)
}

pub async fn get_articles(
    core: &PaprCoreBridge,
    filter: ArticleFilter,
) -> Result<Vec<ArticleSummary>, PaprBridgeError> {
    Ok(core.inner.article_service().list_articles(filter).await?)
}

pub async fn get_article_detail(
    core: &PaprCoreBridge,
    article_id: i64,
) -> Result<ArticleDetail, PaprBridgeError> {
    Ok(core.inner.article_service().get_article_detail(article_id).await?)
}

pub async fn refresh_feeds(
    core: &PaprCoreBridge,
    options: RefreshOptions,
) -> Result<RefreshReport, PaprBridgeError> {
    Ok(core.inner.ingestion_service().refresh_feeds(options).await?)
}

pub async fn import_opml(
    core: &PaprCoreBridge,
    opml_text: String,
) -> Result<OpmlImportReport, PaprBridgeError> {
    Ok(core.inner.opml_service().import_text(opml_text).await?)
}

pub async fn get_settings(
    core: &PaprCoreBridge,
) -> Result<SettingsSnapshot, PaprBridgeError> {
    Ok(core.inner.settings_service().get_settings().await?)
}
```

### Codegen Strategy

- Use `flutter_rust_bridge_codegen generate` manually.
- Explicit `rust_input`: `crates/papr-flutter-bridge/src/api.rs`.
- Generated Dart bindings live in `mobile/lib/bridge/generated/` and are committed.
- Do not run codegen automatically from `build.rs` in phase 1.
- CI should verify generated files are up to date.

### Android Rust Build

- Use a Gradle cargo build task or the `rust-android-gradle` plugin.
- Flutter Android builds must package `libpapr_flutter_bridge.so` automatically.
- Developers should not need to manually copy `.so` files for normal builds.

## Flutter Layer

### Dependencies (Phase 1)

```yaml
dependencies:
  flutter:
    sdk: flutter
  flutter_rust_bridge: ^2.0.0
  flutter_riverpod: ^2.0.0
  path_provider: ^2.0.0

dev_dependencies:
  flutter_test:
    sdk: flutter
  flutter_lints: ^3.0.0
```

No `freezed`, no `go_router` in phase 1.

### Layering

```text
FRB generated bindings
  ↓
PaprCoreService
  ↓
Repository
  ↓
Riverpod providers
  ↓
Flutter UI
```

### Key Providers (Phase 1)

```dart
final paprCoreServiceProvider = FutureProvider<PaprCoreService>((ref) async { ... });
final feedRepositoryProvider = Provider<FeedRepository>((ref) { ... });
final articleRepositoryProvider = Provider<ArticleRepository>((ref) { ... });
final settingsRepositoryProvider = Provider<SettingsRepository>((ref) { ... });
final feedListProvider = FutureProvider<List<Feed>>((ref) { ... });
final articleListProvider = FutureProvider.family<List<ArticleSummary>, ArticleFilter>((ref, filter) { ... });
final articleDetailProvider = FutureProvider.family<ArticleDetail, int>((ref, id) { ... });
final settingsProvider = FutureProvider<SettingsSnapshot>((ref) { ... });
```

## Migration Strategy

Phase 1 uses a "temporary duplication" strategy (A + C):

- Create new `papr-core/src/db.rs` containing only the queries needed for the 6 phase-1 APIs.
- Create new `papr-core/src/dto.rs` with the phase-1 DTOs.
- Keep the existing `src-tauri/src/db.rs` and `src-tauri/src/models.rs` untouched so the desktop app continues to work.
- Once `papr-core` is stable and the Flutter app runs, gradually port the Tauri adapter to use `papr-core` and remove the duplicated code.

This avoids a high-risk big-bang migration and lets each side evolve independently during the initial phase.

## Implementation Order

1. Create `feat/flutter-android-rearchitecture` branch.
2. Delete `docs/android-mvp.md`.
3. Write this design document.
4. Create `crates/papr-core/` skeleton with `Cargo.toml`, `lib.rs`, `config.rs`, `error.rs`.
5. Add phase-1 DTOs and services to `papr-core`.
6. Create `crates/papr-flutter-bridge/` skeleton with FRB opaque wrapper and the 6 API functions.
7. Update root `Cargo.toml` workspace.
8. Create `mobile/` Flutter project and add dependencies.
9. Configure FRB codegen and Android Gradle cargo build.
10. Implement Riverpod providers and minimal UI screens.
11. Build and run a first end-to-end flow on Android.

## Stop Conditions

Pause and ask for review if any of the following occur:

- Rust or Tauri build breaks in a way that affects the desktop app.
- `papr-core` cannot be compiled without Tauri dependencies.
- FRB codegen cannot represent a required DTO or error type.
- Android SQLite cannot be created in app data.
- Feed fetching fails because of a broad Android networking issue.
- Implementing the 6 phase-1 APIs requires adding a major new framework.

## Risks

| Risk | Mitigation |
|------|-----------|
| Rust code is tightly coupled to Tauri | Extract small, pure-Rust services incrementally; gate platform code behind adapters. |
| `state.rs` refactor is error-prone | Split into core state and Tauri state explicitly; avoid moving Tauri handles into core. |
| FRB v2 learning curve | Use manual codegen, commit generated files, and keep API surface small in phase 1. |
| Desktop regression | Keep desktop build passing; do not modify desktop code paths unless required for isolation. |
| Background expectations | Document that phase 1 uses foreground/manual refresh only. |

## Later Milestones

After phase 1 is stable:

- Feed CRUD, article CRUD, tag/rule management.
- Sync configuration and FreshRSS/Miniflux verification.
- AI summaries, Q&A, translation (initially non-streaming).
- AI streaming via platform-agnostic events.
- OPML export and text copy.
- Android file picker for OPML import.
- Background refresh and notifications.
- Tablet and landscape layout.
- Evaluate migrating the Tauri adapter to `crates/papr-tauri/`.

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-07-09 | Reject Tauri Mobile | Runtime and developer experience were unacceptable. |
| 2026-07-09 | Use Flutter for Android | Independent mobile stack, good ecosystem, long-term iOS potential. |
| 2026-07-09 | Reuse Rust backend | Avoid duplicating feed/AI/sync logic in Dart. |
| 2026-07-09 | Same repo, independent module | Keep project together without forcing UI sharing. |
| 2026-07-09 | `flutter_rust_bridge` v2 | Minimises FFI boilerplate for a large API surface. |
| 2026-07-09 | Riverpod for Flutter state | Scales with the app; keeps bridge API framework-agnostic. |
| 2026-07-09 | FRB opaque `PaprCoreBridge` | Keeps `papr-core` free of FRB dependencies. |
| 2026-07-09 | Phase 1 = 6 APIs | Forms a demonstrable feed → article → detail reading loop. |
| 2026-07-09 | Temporary duplication | Lowers risk by not immediately breaking the desktop app. |
| 2026-07-09 | Bridge owns FRB DTOs, not `papr-core` | `papr-core` must not depend on `flutter_rust_bridge`; bridge crate defines `#[frb]` DTOs and conversion functions. |
| 2026-07-09 | `Db` wrapped in `Mutex` | `rusqlite::Connection` is not `Sync`; FRB opaque wrapper requires `Send + Sync`. |

## Phase 1 Implementation Status

### Completed

- [x] Created `feat/flutter-android-rearchitecture` branch from `codex/multi-llm-provider-adapter`.
- [x] Deleted obsolete `docs/android-mvp.md`.
- [x] Created `docs/flutter-android-rearchitecture.md` (this document).
- [x] Created root `Cargo.toml` workspace containing `src-tauri`, `crates/papr-core`, `crates/papr-flutter-bridge`.
- [x] Created `crates/papr-core/`:
  - `Cargo.toml`
  - `src/lib.rs` — `PaprCore` + service accessors
  - `src/config.rs` — `PaprCoreConfig`, `Platform`
  - `src/error.rs` — `CoreError`
  - `src/dto.rs` — phase-1 DTOs (`Feed`, `ArticleSummary`, `ArticleDetail`, etc.)
  - `src/db.rs` — SQLite layer with migration + 6 API queries (wrapped in `Mutex`)
  - `src/services/` — `FeedService`, `ArticleService`, `IngestionService`, `OpmlService`, `SettingsService`
- [x] Created `crates/papr-flutter-bridge/`:
  - `Cargo.toml`
  - `src/lib.rs`
  - `src/api.rs` — `PaprCoreBridge` opaque wrapper + 6 FRB API functions + DTO conversions
  - `src/dto.rs` — `#[frb]` DTOs mirrored from `papr-core`
  - `src/error.rs` — `PaprBridgeError`
- [x] Created `mobile/` Flutter project (regenerated with Flutter 3.44 template):
  - `pubspec.yaml` with `flutter_rust_bridge`, `flutter_riverpod`, `path_provider`, `freezed_annotation`, `freezed`, `build_runner`
  - `lib/` app code, core/config/di/exceptions, services, repositories, UI screens
  - Generated FRB bindings in `lib/bridge/generated/`
  - Freezed outputs generated via `build_runner`
- [x] Configured `flutter_rust_bridge_codegen`:
  - Installed CLI (`cargo install flutter_rust_bridge_codegen --version "^2.0"`)
  - Config file at `rust_frb_codegen.yaml`
  - Generated Dart bindings committed
- [x] Added Android NDK linkers to `C:\Users\Ryder\.cargo\config.toml`.
- [x] Verified `cargo check -p papr-core -p papr-flutter-bridge` passes.
- [x] Verified `cargo build -p papr-flutter-bridge --target aarch64-linux-android` produces `libpapr_flutter_bridge.so`.
- [x] Verified `flutter analyze --no-pub` passes with no issues.

### Blocked / Not Yet Completed

- [x] **Flutter Android APK build**: resolved by configuring Alibaba Cloud Gradle mirror, upgrading to Gradle 9.1.0, and overriding `:jni` / app NDK version to the locally intact `30.0.14904198`.
- [x] **Android Gradle Rust integration**: replaced the upstream-incompatible `rust-android-gradle` plugin with a custom Gradle `buildRustBridge` task in `mobile/android/app/build.gradle.kts` that runs `cargo build` for each ABI and copies the resulting `.so` files into `src/main/jniLibs`.
- [x] **Rust bridge for other Android ABIs**: `libpapr_flutter_bridge.so` is now built and packaged for `arm64-v8a`, `armeabi-v7a`, `x86`, and `x86_64`.
- [x] **Feed refresh business logic**: `IngestionService.refresh_feeds` performs conditional HTTP fetch, RSS/Atom parsing, article and enclosure insertion, feed metadata updates, and per-feed error isolation.
- [ ] **Stub business logic**: `OpmlService.import_text` is still an empty stub.
- [x] **Runtime verification on emulator**: Debug APK installed on `emulator-5554`; add Feed → refresh → article list → article detail was verified on 2026-08-20, including a repeated refresh with no duplicate article or enclosure.
- [ ] **Desktop regression check**: `src-tauri` has not been switched to `papr-core`; current desktop build should still work but should be verified.

### Files Modified / Created (Current Branch)

```text
Cargo.toml                                      (new)
docs/flutter-android-rearchitecture.md          (new)
docs/android-mvp.md                             (deleted)
crates/papr-core/Cargo.toml                     (new)
crates/papr-core/src/lib.rs                     (new)
crates/papr-core/src/config.rs                  (new)
crates/papr-core/src/error.rs                   (new)
crates/papr-core/src/dto.rs                     (new)
crates/papr-core/src/db.rs                      (new)
crates/papr-core/src/services/mod.rs            (new)
crates/papr-core/src/services/article.rs        (new)
crates/papr-core/src/services/feed.rs           (new)
crates/papr-core/src/services/ingestion.rs      (new)
crates/papr-core/src/services/opml.rs           (new)
crates/papr-core/src/services/settings.rs       (new)
crates/papr-flutter-bridge/Cargo.toml           (new)
crates/papr-flutter-bridge/src/lib.rs           (new)
crates/papr-flutter-bridge/src/api.rs           (new)
crates/papr-flutter-bridge/src/dto.rs           (new)
crates/papr-flutter-bridge/src/error.rs         (new)
crates/papr-flutter-bridge/src/frb_generated.rs (FRB generated)
mobile/                                           (new/regenerated)
  pubspec.yaml
  analysis_options.yaml
  .gitignore
  android/...                                     (Flutter 3.44 template)
  ios/...                                         (Flutter 3.44 template)
  lib/...                                         (custom app code)
  lib/bridge/generated/...                        (FRB + freezed generated)
rust_frb_codegen.yaml                           (new)
C:\Users\Ryder\.cargo\config.toml               (added Android NDK linkers)
```

### Immediate Next Steps (When Work Resumes)

1. **Implement OPML import** in `OpmlService.import_text`.
2. **Render article HTML as content** instead of displaying markup as plain text.
3. **Verify refresh against public feeds** on a device or emulator with working external networking.
4. **Verify desktop build still passes** (`pnpm build`, `cargo check` in `src-tauri`).
5. **Commit the completed refresh pipeline changes**.

### Known Warnings to Address Later

- `papr-flutter-bridge`: `unexpected_cfgs` warnings from FRB macros — normal for FRB v2, can be silenced with `#![allow(unexpected_cfgs)]` if desired.
