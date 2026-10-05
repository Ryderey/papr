# Design: 实现 refreshFeeds 文章抓取展示闭环

## 总体思路
把 `src-tauri` 中成熟的 feed 抓取/解析逻辑下沉到 `papr-core`，在 `IngestionService` 中实现真正的 `refresh_feeds`；Flutter 端在 FeedListScreen 增加刷新入口，刷新成功后刷新文章列表。

## 数据流

```
FeedListScreen
  └── FeedRepository.refreshFeeds()
        └── bridge.refreshFeeds(core, options)
              └── PaprCoreBridge::refresh_feeds(options)
                    └── IngestionService::refresh_feeds(options)
                          ├── for each feed: fetch conditional_get
                          ├── parse_feed
                          ├── Db::upsert_articles
                          └── Db::update_feed_meta / set_feed_fetch_state
```

## Rust 侧设计

### 新增模块
- `crates/papr-core/src/ingestion/fetch.rs` — HTTP 条件 GET，复用 `src-tauri` 逻辑。
- `crates/papr-core/src/ingestion/parse.rs` — feed 解析、文章映射，复用 `src-tauri` 逻辑。
- `crates/papr-core/src/ingestion/mod.rs` — 导出。

### 依赖
在 `papr-core/Cargo.toml` 中添加：
- `reqwest`（已存在 workspace）
- `feed-rs`（已存在 workspace）
- `scraper`（已存在 workspace）
- `url`（已存在 workspace）
- `encoding_rs`（已存在 workspace）
- `html-escape` 或直接用现有 `ammonia`/`dom_smoothie`？`src-tauri` 使用自定义 `sanitize` 模块。为简化，先只复用核心解析，HTML sanitize 可先用 `ammonia`。

实际上 `src-tauri/src/sanitize.rs` 也有成熟逻辑。为了不过度扩大范围，本阶段仅实现基础文章入库：
- 用 `feed-rs` 解析 title/guid/url/published/content/summary/author。
- 用 `ammonia` 清洗 content_html。
- body_text 从 content_html 提取纯文本（可用 `ammonia` + 简单 strip tags，或复用 `src-tauri/sanitize::html_to_text`）。

为减少代码量和风险，优先复用 `src-tauri/src/ingestion/parse.rs` 和 `sanitize.rs` 的核心函数，移到 `papr-core`。

### Db 层新增
- `Db::feeds_to_refresh() -> Result<Vec<(i64, String, Option<String>, Option<String>)>, CoreError>`：返回所有待刷新 feed。
- `Db::upsert_article(feed_id, article) -> Result<bool, CoreError>`：按 `(feed_id, guid)` 去重插入。
- `Db::update_feed_meta(feed_id, title?, site_url?, description?, favicon_url?) -> Result<(), CoreError>`
- `Db::set_feed_fetch_state(feed_id, etag?, last_modified?, fetch_error?) -> Result<(), CoreError>`
- `Db::touch_feed(feed_id) -> Result<(), CoreError>`：更新 `last_fetched_at`。

### Service 层
- `IngestionService::refresh_feeds(options: RefreshOptions) -> Result<RefreshReport, CoreError>`
  - 如果 `options.feed_ids` 有值，只刷新指定 feed；否则刷新全部。
  - 串行或有限并发抓取。为简单先串行；若太慢可后续加并发。
  - 返回 `RefreshReport { total_feeds, new_articles, errors }`。

### DTO 新增
- `papr-core::dto::NewArticle { guid, url, title, author, summary, content_html, body_text, image_url, published_at, enclosures }`
- `papr-core::dto::Enclosure { url, mime_type, length }`

### 错误处理
- 单个 feed 失败收集为 `RefreshError { feed_id, message }`，不中断整体流程。

## Flutter bridge 层
- `api.rs` 中 `refresh_feeds` 已存在，只需 Rust 侧实现真正逻辑。
- 无需新增 DTO。

## Flutter 层
- `FeedRepository.refreshFeeds() -> Future<RefreshReport>`
- `FeedListScreen`：
  - 添加 `RefreshIndicator` 包裹列表，支持下拉刷新。
  - 或添加 AppBar 右上角刷新按钮。
  - 刷新成功后 `ref.invalidate(feedListProvider)` 和 `ref.invalidate(articleListProvider)`。
- `ArticleListScreen`：已能展示文章列表，无需大改。

## 边界与异常
- 网络失败：记录到 `fetch_error`。
- 解析失败：记录到 `fetch_error`。
- 空 feed：正常结束，new_articles = 0。
- 重复 guid：`ON CONFLICT DO NOTHING`。

## 兼容性
- 不改变现有表结构。
- `RefreshOptions` 和 `RefreshReport` 已存在，保持签名。
