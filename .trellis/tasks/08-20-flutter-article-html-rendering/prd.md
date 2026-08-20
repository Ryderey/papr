# 修复 Flutter 文章正文图片渲染

## Goal

在 Flutter Android 文章详情页正确渲染已清洗的 HTML 正文与内联图片。

## Background

`ArticleDetailScreen` 当前把 `extractedHtml ?? contentHtml` 直接交给 Flutter `Text`，导致 `<p>`、`<strong>` 和 `<img>` 等标签以普通文字显示。Rust ingestion 已清洗正文并保留绝对图片 URL，FRB `ArticleDetail` 也已传递这些字段，因此问题位于 Flutter 详情页渲染层。

## Requirements

- 保持正文选择顺序为 `extractedHtml` 优先、`contentHtml` 兜底。
- 正确渲染已清洗的常见 HTML 标签和正文中的全部网络图片。
- 图片不得超出正文宽度，并保持原始宽高比。
- 使用文章 URL 作为相对资源的基础地址。
- `null` 或空白正文继续显示 `No content`。
- 只修改 Flutter 详情页和所需依赖，不改动 Rust、数据库或 FRB DTO。
- 先通过 Widget 回归测试复现原始标签与图片无法渲染的问题，再实施修复。

## Acceptance Criteria

- [x] 详情页不再显示原始 `<p>`、`<img>` 等标签文本。
- [x] 段落、强调文字和正文内联图片能够正常渲染。
- [x] 宽图片在正文区域内无横向溢出。
- [x] 空正文显示 `No content`。
- [x] Widget 回归测试、`flutter test`、`flutter analyze` 和 Debug APK 构建通过。
- [ ] Android 模拟器使用 `https://www.ithome.com/rss/` 验证正文和内联图片正常显示。
- [x] 在模拟器通过宿主机夹具加载同一 IT 之家真实图片，验证 APK 级 HTML、粗体、网络图片及响应式布局链路。

在线验收项受测试 AVD 网络环境阻塞：AVD 没有默认路由，`ping 8.8.8.8` 返回 `Network is unreachable`，刷新记录为 `network error`。这不是应用代码失败。

## Out of Scope

- 文章列表缩略图。
- 外链跳转。
- 图片防盗链代理或桌面阅读器完整能力。
