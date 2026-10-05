# Implement: 实现添加 feed 功能

## 执行清单

### Rust core 层
1. `crates/papr-core/src/db.rs`
   - [x] 新增 `add_feed(feed_url: &str) -> Result<i64, CoreError>`
   - [x] 新增 `get_feed(id: i64) -> Result<Feed, CoreError>`
   - [x] 新增 `is_unique_violation` 辅助函数

2. `crates/papr-core/src/services/feed.rs`
   - [x] 新增 `add_feed(feed_url: String) -> Result<Feed, CoreError>`
   - [x] 把 `block_in_place` 改为 `spawn_blocking` 避免 ANR

### Flutter bridge 层
3. `crates/papr-flutter-bridge/src/api.rs`
   - [x] 新增 `add_feed(core, feed_url) -> Result<Feed, PaprBridgeError>`

### Flutter 层
4. `mobile/lib/repositories/feed_repository.dart`
   - [x] 新增 `addFeed(String url) -> Future<Feed>`

5. `mobile/lib/ui/screens/feed_list_screen.dart`
   - [x] 添加 FloatingActionButton
   - [x] 实现 `_AddFeedDialog` StatefulWidget
   - [x] 添加成功后刷新 `feedListProvider`

6. `mobile/lib/repositories/article_repository.dart` 与 `settings_repository.dart`
   - [x] 移除未使用的 `../core/exceptions.dart` import

### 代码生成与验证
7. [x] 运行 FRB 代码生成
8. [x] `cd mobile && flutter build apk --debug`
9. [x] 安装到 emulator-5554
10. [x] 手动测试：通过 seed 验证 addFeed 端到端链路（数据库写入 + UI 列表刷新）
11. [ ] 通过真实 UI 输入验证（adb 自动化在模拟器上不稳定，已排除应用代码问题）

## 关键命令

```bash
# FRB 代码生成
cd mobile
flutter_rust_bridge_codegen generate

# 构建 APK
cd mobile
flutter build apk --debug

# 安装并启动
adb install -r build/app/outputs/flutter-apk/app-debug.apk
adb shell am start -n com.papr.papr_mobile/.MainActivity
```

## 回滚点
- 若 FRB 生成失败，回滚 `api.rs` 改动并检查 `flutter_rust_bridge_codegen` 版本。
- 若 APK 构建失败，优先检查 `papr-core` 编译错误。

## 验证步骤
1. 启动应用，确认 Feeds 页面为空。
2. 点击右下角 + 按钮，输入 `https://rsshub.app/anthropic/news` 或任意可用 RSS URL。
3. 确认对话框关闭，列表出现新 feed（title 暂时显示 URL）。
4. 再次添加相同 URL，确认弹出错误提示。
5. `adb shell run-as com.papr.papr_mobile sqlite3 app_flutter/papr/papr.db "SELECT * FROM feeds;"` 确认数据库有记录。
