# PRD: 实现添加 feed 功能

## 背景
Flutter Android 端 MVP 已经能初始化 PaprCore、展示空的 Feeds 列表。为了让应用具备最小可用闭环，需要支持用户手动输入 RSS feed URL 并订阅。

## 目标
在 Flutter 端提供「添加订阅源」功能：用户输入 feed URL，调用 Rust core 入库，成功后返回 Feeds 列表并显示新订阅。

## 验收标准
1. Rust core 暴露 `add_feed(feed_url: String) -> Result<Feed, CoreError>` API。
2. Flutter 端 `FeedListScreen` 提供添加 feed 的入口（FAB + 对话框）。
3. 输入有效 RSS URL 后，feed 被写入 SQLite `feeds` 表，列表自动刷新。
4. 输入重复 URL 时显示明确错误（不崩溃）。
5. 输入无效/空 URL 时前端做基础校验并提示。
6. 构建、安装到 emulator-5554 后，手动添加一个 feed 成功，列表非空。

## 非目标
- 不实现 feed 自动抓取文章内容（后续 refreshFeeds 任务处理）。
- 不实现 OPML 导入以外的批量导入。
- 不实现文件夹选择（固定为 null）。
- 不实现 source type 自动探测（默认 rss）。

## 约束
- 保持现有代码风格（Riverpod + ConsumerWidget + repository 层）。
- 保持 Rust 侧 `papr-core` / `papr-flutter-bridge` 分层。
- 新增代码必须伴随 FRB 代码生成；生成文件可提交。
- 不引入新的 Flutter/Rust 依赖。

## 相关文件
- `crates/papr-core/src/db.rs`
- `crates/papr-core/src/services/feed.rs`
- `crates/papr-core/src/dto.rs`
- `crates/papr-core/src/error.rs`
- `crates/papr-flutter-bridge/src/api.rs`
- `crates/papr-flutter-bridge/src/dto.rs`
- `crates/papr-flutter-bridge/src/error.rs`
- `mobile/lib/repositories/feed_repository.dart`
- `mobile/lib/ui/screens/feed_list_screen.dart`
- `mobile/lib/bridge/generated/*`（FRB 生成）
