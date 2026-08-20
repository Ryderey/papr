# Implement: Flutter 文章正文 HTML 与图片渲染

## Execution Order

1. [x] 新增 `ArticleDetailScreen` Widget 回归测试，使用含段落、强调标签和内联 data URI 图片的 DTO；运行目标测试并确认当前实现失败。
2. [x] 添加 `flutter_widget_from_html_core: ^0.17.2` 并更新依赖锁文件。
3. [x] 用 `HtmlWidget` 替换详情页正文 `Text`，保留内容优先级、空正文兜底、基础 URL 与响应式图片样式。
4. [x] 运行目标测试、完整 Flutter 测试和静态检查。
5. [x] 构建 Debug APK，并使用真实 IT 之家图片夹具完成模拟器端到端渲染验证。

## Validation Commands

```powershell
flutter test test/ui/screens/article_detail_screen_test.dart
flutter test
flutter analyze
flutter build apk --debug
```

## Rollback Point

改动只涉及任务资料、`mobile/pubspec.yaml`、`mobile/pubspec.lock`、详情页及对应测试；若依赖在当前 Flutter 工具链不兼容，撤回本任务新增文件和依赖，不触碰现有 Rust/FRB 变更。

## Verification Results

- RED：原实现的 Widget 测试找到包含 `<p>` 的 `Text`，没有图片组件。
- GREEN：目标测试 2/2 通过，完整 `flutter test` 通过。
- `flutter analyze`：无问题。
- `flutter build apk --debug`：成功生成 `build/app/outputs/flutter-apk/app-debug.apk`；仅有既存 FRB cfg 警告。
- 模拟器：正文标签不再裸露、`strong` 正常加粗、真实 IT 之家图片正常显示且未超出正文宽度。
- 环境限制：两个本地 AVD 均无可用外网默认路由，live RSS 刷新无法作为在线验收证据。
