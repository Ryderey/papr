# Design: P2 文章列表、智能视图与完整阅读闭环

## 1. 分层

```text
SQLite/FTS5 → papr-core ArticleService/SettingsService → FRB DTO/API
            → Flutter Repository → Riverpod/UI → Android MethodChannel
```

Core 拥有查询、状态事务、正文抓取/提取/清洗和设置验证；Flutter 只编排分页、乐观状态和展示；Android 只调用系统浏览器与分享 Intent。

## 2. Core 数据合同

- 扩展 `ArticleFilter`：`query`、`unread_only`、`oldest_first`、`limit`、`offset`。Core 将 limit 限制在 1..=200，默认 50，offset 最小为 0。
- `ArticleFilterKind::Tag` 使用 `article_tags` 子查询，不再退化为 All。
- 新增 `ArticleCounts` 与 `TagSummary`，支持全局智能视图计数、当前筛选计数和只读标签筛选列表。
- `set_read/set_starred/set_read_later` 统一通过一个 DB 状态写入点执行 `UPDATE + append_change_log`；不存在的文章返回稳定 `articleNotFound`。
- `mark_all_read(filter)` 复用同一筛选谓词，在事务内只更新未读行并为受影响文章追加 change log。
- `set_extracted_html` 在事务内更新正文、必要时补封面、刷新 FTS body、清空 `translated_html/translated_lang`。

不新增数据库迁移：正式 schema 已包含 FTS、文章状态、标签、提取正文、翻译缓存、settings 和 change_log。

## 3. 全文提取

从桌面机械下沉 `extraction.rs`：HTTP 获取源页面 → 捕获最终 URL → `dom_smoothie` Readability → Core sanitizer → 事务写入。解析放入 `spawn_blocking`，网络请求使用 `PaprCore` 共享 HTTP client。错误使用稳定 code：`articleNotFound`、`articleUrlMissing`、`noExtractableContent`、`network`、`parse`。

## 4. 阅读设置

`ReadingSettings` 默认值：system 字体、17sp、1.65 行距、680dp 最大宽度、显示阅读时长、关闭自动全文。Core 验证字体枚举与数值范围（字号 14–24、行距 1.3–2.0、宽度 320–840），存入现有 settings key/value 表；Flutter 通过单一设置 Repository 读取/写入。

## 5. Flutter 状态与 UI

- 文章主页使用可复用的分页控制器，筛选变化重置页；下一页按 offset 追加并按 ID 去重。
- 智能视图通过筛选菜单暴露，全局计数由 Core 返回；Feed/文件夹沿用 P1 数据，标签为只读列表。
- 单篇状态先更新当前列表/详情，再调用 Repository；失败恢复快照并刷新计数。
- 打开详情前标记已读；详情操作后返回列表会刷新相关分页与计数。
- 阅读器将标题、元信息、状态/提取/浏览器/分享操作和正文放入一个可滚动页面；网络图片失败显示图标占位。

## 6. Android 边界

扩展现有 `com.papr.papr_mobile/platform`：

- `openUrl({url}) -> bool`：仅接收 http/https，使用 `ACTION_VIEW`。
- `shareArticle({title,url}) -> bool`：使用 `ACTION_SEND text/plain` 和 chooser。

Flutter 在调用前检查 URL，平台异常映射为可恢复提示；不新增依赖和权限。

## 7. 兼容、回滚与风险

- Core 查询从桌面逻辑复制并适配 `CoreError`，桌面实现暂不删除。
- FRB 绑定与 UI 分批提交，任一层可独立回滚。
- 主要风险是动态 SQL/FTS 参数错位、分页竞态、乐观状态回滚和 Readability 在 Android ABI 上的构建；用单元测试、Widget 测试和 APK 构建覆盖。
