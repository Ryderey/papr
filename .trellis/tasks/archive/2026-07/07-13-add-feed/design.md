# Design: 实现添加 feed 功能

## 总体思路
从 UI 到 Rust core 新增一条「添加订阅源」链路。Rust 侧负责 URL 基础校验、入库、返回 `Feed` 对象；Flutter 侧负责输入 UI、调用 repository、刷新列表。

## 数据流

```
FeedListScreen (FAB click)
  └── AddFeedDialog (StatefulWidget)
        └── FeedRepository.addFeed(url)
              └── bridge.addFeed(core, url)
                    └── PaprCoreBridge::add_feed(url)
                          └── FeedService::add_feed(feed_url)
                                └── Db::add_feed(feed_url)
                                      └── INSERT INTO feeds ...
```

## Rust 侧设计

### `papr-core`

1. **DTO 新增**
   - `dto.rs` 新增 `AddFeedInput { feed_url: String }`（可选，若直接传 String 可省略）。
   - 为简化跨层传递，直接传 `String`。

2. **Db 层**
   - 新增 `Db::add_feed(feed_url: &str) -> Result<i64, CoreError>`。
   - 插入时：
     - `title` 暂时用 feed_url（首次刷新后再更新真实 title）。
     - `source_type` 默认 `'rss'`。
     - `folder_id` 默认 `NULL`。
     - 利用 `feed_url UNIQUE` 约束捕获重复，转换为 `CoreError::InvalidInput`。
   - 新增 `Db::get_feed(id: i64) -> Result<Feed, CoreError>` 用于返回刚插入的完整记录。

3. **Service 层**
   - `FeedService::add_feed(feed_url: String) -> Result<Feed, CoreError>`。
   - 对 URL 做简单校验：非空、trim 后非空。
   - 调用 `db.add_feed` 后调用 `db.get_feed` 返回 `Feed`。

### `papr-flutter-bridge`

1. **DTO**
   - 无需新增 DTO，直接使用 `String` 参数。

2. **API**
   - `api.rs` 新增：
     ```rust
     pub async fn add_feed(
         core: &PaprCoreBridge,
         feed_url: String,
     ) -> Result<Feed, PaprBridgeError> {
         let feed = core.inner.feed_service().add_feed(feed_url).await?;
         Ok(feed.into())
     }
     ```

3. **错误映射**
   - `PaprBridgeError` 已覆盖 `InvalidInput`，无需新增错误类型。

## Flutter 侧设计

### Repository
- `FeedRepository.addFeed(String url) -> Future<Feed>`。
- 调用 `bridge.addFeed(core: core, feedUrl: url)`。
- 错误通过 `PaprCoreService.mapError` 转换。

### UI
- `FeedListScreen` 添加 `FloatingActionButton`，点击弹出 `AddFeedDialog`。
- `AddFeedDialog`：
  - TextField 输入 URL。
  - 按钮：取消 / 添加。
  - 添加时显示 CircularProgressIndicator，禁用输入。
  - 成功后 Navigator.pop(true)，外层刷新 `feedListProvider`。
  - 失败时显示 SnackBar 错误信息。

### Provider 刷新
- `feedListProvider` 是 `FutureProvider`，返回 `FeedListScreen` 时通过 `ref.invalidate(feedListProvider)` 或 `ref.refresh(feedListProvider)` 刷新。

## FRB 代码生成
- 修改 `crates/papr-flutter-bridge/src/api.rs` 后，运行 `flutter_rust_bridge_codegen generate`（或 `flutter pub run flutter_rust_bridge_codegen:generate`）重新生成 `mobile/lib/bridge/generated/*`。
- 生成步骤写入 `implement.md`。

## 边界与异常
- 空 URL：Flutter 前端拦截，按钮禁用或提示。
- 重复 URL：Rust 捕获 `UNIQUE` 约束冲突，返回 `InvalidInput`，Flutter 显示「已存在该订阅源」。
- 数据库错误：显示通用错误。

## 兼容性
- 不改变现有表结构（migrations 无需修改）。
- 不改变已有 API 签名。
