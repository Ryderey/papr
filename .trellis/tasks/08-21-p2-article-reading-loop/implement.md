# Implement: P2 文章列表、智能视图与完整阅读闭环

## 块 A：Core 查询、状态与全文提取

1. [x] 扩展文章筛选 DTO 与 DB 查询：标签、FTS、隐藏已读、排序、分页边界和有效日期排序。
2. [x] 新增智能计数/当前筛选计数/只读标签列表。
3. [x] 新增单篇已读/收藏/稍后读与当前筛选批量已读，写入和 change_log 同事务。
4. [x] 下沉 Readability 全文提取，写入时刷新 FTS/缩略图并清翻译缓存。
5. [x] 扩展阅读设置与验证；补 Core 回归（分页/搜索/状态回滚/计数/提取写入/设置持久化）。

## 块 B：FRB

6. [x] 新增/扩展 DTO 与 API：查询选项、计数、标签、状态、批量已读、提取、阅读设置。
7. [x] 重生成绑定并验证连续生成幂等；Bridge 测试通过。

## 块 C：Flutter 阅读闭环

8. [x] Repository 接入分页/搜索/计数/标签/状态/提取/阅读设置。
9. [x] 文章主页实现智能视图、实时计数、搜索、排序、隐藏已读、50 条分页与缩略图占位。
10. [x] 详情实现打开即已读、状态操作、提取/重新提取、阅读设置、阅读时长、enclosure 展示。
11. [x] 乐观更新失败回滚并刷新列表/计数/详情；补 Widget 测试。

## 块 D：Android 与验收

12. [x] 原生 MethodChannel 增加系统浏览器和分享 Intent；无 URL 时 UI 禁用。
13. [x] `cargo test -p papr-core`、Bridge、桌面回归、`flutter analyze/test`、FRB 幂等、Debug APK 全绿。
14. [ ] 实体机验证浏览器/分享、返回、旋转、进程恢复和大列表滚动。

## 自动验证结果（2026-08-21）

- FRB codegen 连续生成 SHA-256 一致。
- `cargo test -p papr-core --quiet`：46 passed。
- `cargo test -p papr-flutter-bridge --quiet`：1 passed；仅保留既有 `frb_expand` cfg 警告。
- `cargo test -p papr --quiet`：252 passed。
- `flutter analyze`：No issues found。
- `flutter test`：10 passed。
- `flutter build apk --debug`：成功，产物 `mobile/build/app/outputs/flutter-apk/app-debug.apk`。

## 回滚点

- A：Core 独立提交，不删除桌面逻辑。
- B：FRB 绑定独立提交。
- C：Flutter UI 独立提交。
- D：Android 平台适配独立提交。

## 交接必记

- 公共 DTO/API 变化；无数据库迁移。
- 运行的检查及实际数量。
- 实体机未覆盖项、分页/提取性能风险与 P3 入口。
