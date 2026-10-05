# Implement: P1 应用框架、订阅源与文件夹

## 执行顺序（每块可独立评审/提交）

### 块 A：Rust core 下沉（folder/feed CRUD + OPML + 来源/发现）

1. [x] 移植文件夹 CRUD 到 `papr-core`：`list_folders/create_folder/rename_folder/delete_folder/reorder_folders`（trim/大小写不敏感去重/空名拒绝/删文件夹 Feed 落未分类/排序事务化）。
2. [x] 移植 Feed CRUD：`insert_feed(list 含 unread/interval)/delete_feed/rename_feed(custom_title)/move_feed/set_feed_refresh_interval/refine_feed_source_type/find_feed_by_url/feeds_for_export`。
3. [x] 移植 `opml.rs`：`parse`（tidy + 去重）+ `build`（导出），`OpmlService` 实装 `import_text` + `export_text`。
4. [x] 移植 `sources.rs` + `discovery.rs`：`normalize_source/expand_rsshub/extract_channel_id/youtube_feed_url/search_directory/looks_like_url/normalize_query_url/parse_deep_link`；拷 `feed-directory.json` 资源。
5. [x] 新增 `FolderService`；`parse` 补 `detect_source_type/refine_source_type/discover_feeds/looks_like_feed`；`IngestionService::add_feed` 完整流程（规范化→发现→首次抓取→解析→分类→持久化）。
6. [x] 全部改用 `CoreError` + 稳定错误码（`emptyFolderName/folderNameExists/emptyFeedTitle/feedAlreadyExists/emptyFeedUrl/feedNotFound`）；folder/feed 写路径接 `Db::transact`+`append_change_log`。
7. [x] 移植/新增测试（opml/sources/discovery/parse/CRUD/添加订阅事务回滚/外观持久化/文件夹排序）；`cargo test -p papr-core` **42 全绿**。

**块 A 验证结果**：`cargo test -p papr-core` 41 通过（含强制文章写入失败时 Feed、文章与 change_log 同事务回滚）；`cargo test -p papr` 252 通过；`cargo test -p papr-flutter-bridge` 1 通过。

### 块 B：FRB 暴露 + 绑定重生成

8. [x] `papr-flutter-bridge`：新增 `Folder`/`DiscoveryResult`/`AddFeedInput` DTO + 上述 API 的 `#[frb]` 暴露 + DTO 转换；`Feed` 补齐 `custom_title`/`refresh_interval_min`。
9. [x] 重生成绑定（连续两次生成 diff hash 一致）；`cargo test -p papr-flutter-bridge` 1 通过；`flutter analyze` 清零。

**块 B 验证结果**：FRB 2.12.0 生成幂等；`flutter test` 2 通过；`flutter build apk --debug` 成功生成 `app-debug.apk`。Bridge 的 `add_feed` 已切换到 Core 完整抓取/发现/解析/原子持久化链路。

### 块 C：Flutter 框架 + UI

10. [x] 自适应导航（`NavigationBar`/`NavigationRail` + 双栏），顶层 4 入口。
11. [x] i18n（`gen-l10n`，zh/en/ja）+ 主题切换（system/light/dark 持久化）。
12. [x] 文件夹管理 UI（创建/重命名/删除/拖拽排序）+ Feed 管理 UI（删除确认/重命名/移动/单源刷新/刷新间隔）。
13. [x] 添加订阅流程 UI（URL 输入 + 发现 + 目录搜索）+ OPML 导入/导出 UI。
14. [x] `flutter test` + `flutter analyze` 通过。

**块 C 验证结果**：`flutter analyze` 无问题；`flutter test` 7 项通过（手机底栏、平板侧栏、Feed 管理、目录添加、文件夹可排序 UI、阅读器）；三语错误码映射与主题/语言 SQLite 持久化已接通。`reorder_folders` 会校验完整 ID 集，并在同一事务中更新 position 与每个受影响文件夹的 change_log。

### 块 D：Android 集成 + 真机验证

15. [x] OPML 文档选择器（原生 SAF，无新增依赖/无存储权限）+ `papr://subscribe` intent-filter（冷/热启动路由）。
16. [x] 真机验证 CRUD 重启持久化、OPML 往返、深链、无布局溢出。

**块 D 验证结果**：Android 模拟器冷启动深链已自动打开并预填添加订阅对话框；系统 `ACTION_OPEN_DOCUMENT`/`ACTION_CREATE_DOCUMENT` 均成功拉起，导出保持 `.opml` 后缀且刚导出的文件可再次选择导入；Debug APK 构建成功。2026-08-21 物理设备验收通过，覆盖 CRUD 重启持久化、真实 Feed/OPML 往返、深链与布局检查。

## 验证命令

```powershell
cargo test -p papr-core
cargo check -p papr-flutter-bridge
cargo test -p papr          # 桌面回归（下沉不破坏桌面）

Set-Location mobile
flutter analyze
flutter test
flutter build apk --debug
```

## 评审门禁

- 块 A：`cargo test -p papr-core` 全绿 + 桌面 `cargo test -p papr` 不回归。
- 块 B：FRB 重生成幂等 + `flutter analyze` 清零。
- 块 C：`flutter test`/`analyze` 通过。
- 块 D：真机验收（CRUD/OPML/深链/布局）。

## 回滚点

- 每块独立提交；块 A 是「复制 + 适配」不动桌面，可随时回退。
- 若 FRB 新 API 与现有绑定冲突，回退块 B 只保留 core 测试。

## 交接必记

- 修改的公共 DTO、迁移（无新迁移，复用 v16）、错误码、平台接口变化。
- 实际运行的检查与结果。
- 已知风险与 P2 入口。
