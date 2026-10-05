# PRD: 实现 refreshFeeds 文章抓取展示闭环

## 背景
Flutter Android 端已支持添加 feed，但 `IngestionService::refresh_feeds` 还是 stub，不会真正抓取文章。点击 feed 进入文章列表时永远为空，无法形成阅读闭环。

## 目标
实现从 feed URL 抓取 RSS/Atom、解析文章、写入 SQLite、在 Flutter 端展示文章列表的完整流程。

## 验收标准
1. Rust core 的 `IngestionService::refresh_feeds` 真正抓取所有 feed。
2. 新文章被写入 `articles` 表，`enclosures` 表同步写入附件。
3. 抓取成功后更新 feed 的 `title`、`site_url`、`description`、`favicon_url`、`last_fetched_at`、`etag`、`last_modified`。
4. 单个 feed 抓取失败不影响其他 feed，失败原因写入 `fetch_error`。
5. Flutter `FeedListScreen` 支持下拉刷新或刷新按钮触发 `refreshFeeds`。
6. 刷新完成后文章列表自动更新，点击文章可进入详情页。
7. 构建并安装到 emulator-5554 后，添加 feed → 刷新 → 能看到文章列表。

## 非目标
- 不实现后台定时刷新（仅手动触发）。
- 不实现 newsletter、rules、sync、retention。
- 不实现全文提取（`extracted_html` 留空，先用 `content_html`）。
- 不实现 feed 自动发现（discovery）。
- 不实现文章去重（dedup）设置，默认按 `(feed_id, guid)` 去重即可。

## 约束
- 保持现有代码风格（Riverpod + repository + service 分层）。
- 保持 `papr-core` 不依赖 Tauri，自包含 HTTP client。
- 复用 `src-tauri/src/ingestion/fetch.rs` 和 `parse.rs` 的核心逻辑，但适配 `papr-core` 的错误类型和 DTO。
- 不引入新的 crates（reqwest、feed-rs、scraper 等已在 workspace）。

## 相关文件
- `crates/papr-core/src/services/ingestion.rs`
- `crates/papr-core/src/db.rs`
- `crates/papr-core/src/dto.rs`
- `crates/papr-core/src/error.rs`
- `crates/papr-core/src/lib.rs`
- `crates/papr-flutter-bridge/src/api.rs`
- `mobile/lib/repositories/feed_repository.dart`
- `mobile/lib/ui/screens/feed_list_screen.dart`
- `mobile/lib/ui/screens/article_list_screen.dart`
- `src-tauri/src/ingestion/fetch.rs`（参考）
- `src-tauri/src/ingestion/parse.rs`（参考）
