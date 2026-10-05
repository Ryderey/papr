# Design: P1 应用框架、订阅源与文件夹

## 1. 分层与迁移策略

```
桌面(已验证纯逻辑) ──机械下沉──► papr-core ──FRB 薄适配──► Flutter Repository/Provider ──► UI
                                                     └──► Android 平台（文档选择器/深链）
```

**下沉 = 机械移植**：桌面 `db.rs`/`opml.rs`/`sources.rs`/`discovery.rs` 已是平台无关（`&Connection` 参数、无 `AppHandle`），移植只做三件事：
1. `AppError` → `CoreError`（`AppError::code("x")` → `CoreError::coded(ErrorCategory::InvalidInput, "x", detail)`，见 [[papr-core-data-baseline-contracts]]）。
2. `crate::models::*` → `crate::dto::*`（`SourceType` 已是 `&str` 存库，两端枚举一致）。
3. 把 `feed-directory.json` 资源从 `src-tauri/resources/` 拷到 `crates/papr-core/resources/`（`include_str!` 路径改）。

## 2. papr-core 服务划分

- `FolderService`（新）：`list/create/rename/delete/move`。
- `FeedService`（扩）：`list/add/delete/rename/move/set_refresh_interval/refresh_one`。
- `IngestionService`（扩）：`add_feed` 完整流程（规范化→发现→首次抓取→解析→分类→事务入库）。
- `OpmlService`（扩）：`import_text`（保留）+ `export_text`（新）+ `parse/build` 纯函数。
- `DiscoveryService`（新）：`search_directory`、`looks_like_url`、`parse_deep_link`。

DB 写路径（folder/feed CRUD）接入 `Db::transact` + `append_change_log`（entity=`folder`/`feed`，op=`upsert`/`delete`），遵守 P0 事务契约。

## 3. 添加订阅完整流程（`IngestionService::add_feed`）

```
input → normalize_source（YouTube/Reddit/Mastodon → 真实 feed URL / NeedsYoutubeResolution）
      → expand_rsshub（rsshub:// 展开）
      → 若 looks_like_url 且非直接 feed：fetch 页面 → discover_feeds → 选第一个
      → conditional_get feed 文档 → parse → refine_source_type
      → 若 NeedsYoutubeResolution：fetch 页面 → extract_channel_id → youtube_feed_url
      → 事务内：insert_feed + 文章 upsert + change_log
      → 失败用稳定 code（feedAlreadyExists / invalidFeedUrl / feedNotFound / network / parse）
```

## 4. FRB API 表面（新增）

- 文件夹：`list_folders`、`create_folder(name)`、`rename_folder(id,name)`、`delete_folder(id)`。
- Feed：`delete_feed(id)`、`rename_feed(id,title)`、`move_feed(id,folder_id)`、`set_feed_refresh_interval(id,minutes)`、`refresh_feed(id)`。
- 添加订阅：`add_feed(input)` → 返回 `Feed`（含来源识别 + 首次抓取）。
- OPML：`export_opml() -> String`、`import_opml(text)`（已有）。
- 发现：`search_directory(query, lang)`、`parse_deep_link(url)`。
- 错误沿用 `PaprBridgeError{category,code,detail}`（P0 已建）。

DTO 补齐：`Folder`（含 position）、`Feed` 增 `refresh_interval_min`、`custom_title`、`DiscoveryResult`、`AddFeedInput`。

## 5. Flutter 自适应导航 + i18n + 主题

- **导航**：`NavigationBar`（手机）/ `NavigationRail`（平板，`LayoutBuilder` 宽度阈值 ~600dp）；顶层 4 入口 文章/订阅/已保存/设置。平板订阅页双栏（订阅列表 + 文章列表）。
- **i18n**：`flutter_localizations` + `intl` + `gen-l10n`；`lib/l10n/` 下 `app_zh.arb`/`app_en.arb`/`app_ja.arb`；错误码映射到本地化文案（`error.<code>`）。
- **主题**：`ThemeMode.system/light/dark`，设置页切换并持久化到 `SettingsService`。

## 6. Android 平台集成

- **OPML 文档选择**：`file_picker`（`FileType.custom`）或系统 SAF，返回文件内容交给 core `import_opml`；导出写临时文件交 `share_plus`/SAF 保存。不申请 `MANAGE_EXTERNAL_STORAGE`。
- **深链**：`AndroidManifest.xml` 加 `<intent-filter>`（`papr` scheme + `https` App Link 域），冷启动/热启动都路由到 `parse_deep_link` + 添加订阅对话框。

## 7. 兼容与回滚

- 桌面回归必须全绿（`cargo test -p papr` 252 项）：下沉是「复制 + 适配」不是「删除桌面」，桌面代码暂留，逐步切换。
- FRB 重生成幂等；`flutter analyze` 清零。
- 高风险回归：重复订阅、空名/重名、OPML bare-`&`/重复 URL、断网/超时/部分刷新失败、删文件夹不删 Feed。
