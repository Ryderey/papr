# Design: Flutter 文章正文 HTML 与图片渲染

## Boundary

修复限定在 `mobile/`。Rust sanitizer 已把正文清洗为安全 HTML、提升懒加载图片属性并补全相对 URL；Flutter 只负责展示，不重复解析或修改存储数据。

## Rendering

- 添加 `flutter_widget_from_html_core: ^0.17.2`，使用轻量 `HtmlWidget` 渲染正文，不引入视频、WebView、缓存图片或 URL launcher 扩展。
- `ArticleDetailScreen` 计算 `extractedHtml ?? contentHtml`；空白内容使用普通 `Text('No content')`。
- 非空内容交给 `HtmlWidget`，以 `detail.url` 作为 `baseUrl`，并为 `img` 设置 `max-width: 100%`、`height: auto`。
- 不单独显示 `detail.imageUrl`，避免正文已经包含首图时重复展示。

## Compatibility and Failure Handling

- 不改变 Flutter/Rust 公共接口、数据库 schema 或生成绑定。
- 网络图片继续使用 Flutter 默认网络加载；IT 之家示例图片不要求 Referer。
- 图片代理和链接打开留给后续独立任务。

## Test Seam

通过公开的 `ArticleDetailScreen` 和 Riverpod provider override 注入固定 `ArticleDetail`。测试从用户可见 UI 验证正文文本、原始标签消失、图片组件出现和空正文兜底，不测试 `HtmlWidget` 内部实现。
