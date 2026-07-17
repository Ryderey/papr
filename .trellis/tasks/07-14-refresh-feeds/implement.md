# Implement: 实现 refreshFeeds 文章抓取展示闭环

## 执行顺序

1. 修复 `crates/papr-core/src/dto.rs` 中重复的 `Enclosure` 定义。
2. 实现 `crates/papr-core/src/ingestion/parse.rs`：基于 `feed-rs` 解析 RSS/Atom，生成 `NewArticle`。
3. 创建 `crates/papr-core/src/ingestion/mod.rs`，导出 `fetch` / `parse` / `sanitize`。
4. 在 `crates/papr-core/src/db.rs` 中新增：
   - `feeds_to_refresh()`
   - `upsert_article()`
   - `update_feed_meta()`
   - `set_feed_fetch_state()`
   - `touch_feed()`
5. 实现 `IngestionService::refresh_feeds`：串行抓取所有 feed、解析、入库、更新 feed 元数据。
6. 在 `mobile/lib/repositories/feed_repository.dart` 添加 `refreshFeeds()`。
7. 在 `mobile/lib/ui/screens/feed_list_screen.dart` 添加下拉刷新 / AppBar 刷新按钮，刷新后 invalidate `feedListProvider` 与 `articleListProvider`。
8. 运行 `cargo check -p papr-core` 与 `cargo test -p papr-core` 验证。

## 关键约束

- 保持现有代码风格与错误类型 (`CoreError`)。
- 单个 feed 失败不影响其他 feed；失败原因写入 `fetch_error`。
- 文章按 `(feed_id, guid)` 去重：`ON CONFLICT DO NOTHING`。
- 复用已有的 `fetch.rs` 与 `sanitize.rs`。
- `extracted_html` 留空，先用 `content_html`。
- 不引入新依赖。

## 验证命令

```bash
cargo check -p papr-core
cargo test -p papr-core
```

## 验收检查点

- [ ] `cargo check -p papr-core` 通过。
- [ ] `cargo test -p papr-core` 通过。
- [ ] `IngestionService::refresh_feeds` 不再返回空报告。
- [ ] 刷新成功后 `articles` 与 `enclosures` 表有数据。
- [ ] `feeds` 表的 `last_fetched_at` / `etag` / `last_modified` 被更新。
- [ ] Flutter `FeedListScreen` 可触发刷新并更新列表。
