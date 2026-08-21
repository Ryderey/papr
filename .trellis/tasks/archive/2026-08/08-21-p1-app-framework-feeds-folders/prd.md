# P1 应用框架、订阅源与文件夹

## Goal

建立移动端原生自适应导航与 i18n/主题，并把桌面端成熟的文件夹/Feed 全套 CRUD、添加订阅完整流程、OPML 导入导出、订阅目录搜索与 `papr://subscribe` 深链下沉到 `papr-core` 并通过 FRB 暴露给 Flutter。

## Background（已核实的现状差距）

**桌面已有成熟纯逻辑**（无 Tauri 依赖，参数为 `&Connection`，可机械下沉）：
- 文件夹 CRUD：`list_folders/create_folder/rename_folder/delete_folder/folder_id_by_name`（大小写不敏感去重、trim、空名拒绝、删文件夹 Feed 落未分类）。
- Feed CRUD：`insert_feed/list_feeds/delete_feed/rename_feed/move_feed/set_feed_refresh_interval/refine_feed_source_type/find_feed_by_url` 等。
- OPML：`parse`（bare-`&` 容错 + 按 `xml_url` 去重）+ `build`（导出，可往返）。
- 来源规范化 `sources.rs`：`normalize_source`（YouTube/Reddit/Mastodon 识别）、`expand_rsshub`、`extract_channel_id`/`youtube_feed_url`。
- 目录搜索与深链 `discovery.rs`：`search_directory`（内置 `feed-directory.json`）、`looks_like_url`/`normalize_query_url`、`parse_deep_link`。

**core 现状**：只有最小 `list_feeds/add_feed/get_feed` + `opml::import_text`（无导出）+ `ingestion`（fetch/parse/sanitize）。无文件夹 CRUD、无 Feed 编辑、无来源规范化、无目录搜索。

**Flutter 现状**：单一 `feed_list_screen` 入口 + 4 个平铺 screen，无自适应导航、无 i18n（硬编码英文）、无主题切换（设置页只读快照）。

## Requirements

1. **自适应导航**：手机用底部导航 + 抽屉/筛选面板 + 单栏详情；平板用 NavigationRail + 订阅/文章双栏。顶层入口固定为 文章 / 订阅 / 已保存 / 设置。
2. **i18n + 主题**：接入简体中文、英文、日文；系统/浅色/深色主题，设置可切换。
3. **文件夹 CRUD**：创建、重命名、删除、排序；删除文件夹时 Feed 移到未分类（`ON DELETE SET NULL`），不删除 Feed。
4. **Feed CRUD**：添加、删除、重命名（`custom_title=1`）、移动、单源刷新、单源刷新间隔（`REFRESH_OFF_MINUTES` 哨兵）。
5. **添加订阅完整流程**：复用桌面语义——Feed URL、网页 Feed 发现、`rsshub://`、来源识别（YouTube/Reddit/Mastodon/Bluesky/Podcast）；首次抓取/解析/分类/文章写入成功后再统一持久化；重复订阅/无效 URL/无 Feed/网络/解析错误用稳定错误码。
6. **订阅目录搜索与推荐**：`search_directory`（按 UI 语言切片、回退英文）。
7. **OPML 导入/导出**：通过 Android 系统文档选择器，不申请广泛存储权限；导入保留文件夹结构且不重复订阅。
8. **订阅深链**：`papr://subscribe?url=<encoded>` 或 Android App Link 入口。

## Acceptance Criteria

- [x] Feed 与文件夹全部 CRUD 在重启后保持正确（持久化）。
- [x] 删除 Feed 前显示确认；删除后关联文章/缓存按约束清理（`ON DELETE CASCADE`）。
- [x] OPML 导入保留文件夹结构且不重复订阅；导出文件可重新导入（往返测试）。
- [x] 离线、超时、证书错误、部分刷新失败不破坏本地已有数据。
- [x] 手机和平板均无布局溢出；TalkBack 能识别主要控件。
- [x] 错误码跨桥携带稳定 code（`emptyFolderName`/`folderNameExists`/`emptyFeedTitle`/`feedAlreadyExists` 等），Flutter 可本地化。
- [x] FRB 绑定重生成可重复；`cargo test -p papr-core`、`flutter analyze`/`flutter test`、桌面回归 `cargo test -p papr` 全绿。

## Out of Scope

- 文章列表/智能视图/全文搜索/阅读设置（P2）。
- 标签/规则/高亮（P3）。
- 翻译/AI（P4）、音频/Newsletter（P5）。
- FreshRSS/Miniflux 同步、后台刷新、通知（P6）。
