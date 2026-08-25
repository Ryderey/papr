# Implement: P4 翻译与 AI 摘要

1. [ ] 盘点并测试桌面 AI/SSE/翻译语义，冻结 DTO、事件和错误码（已冻结移动端无密钥持久化、按请求临时凭据、request-id 流事件和 success-only 缓存契约；待将桌面语义测试迁入 Core）。
2. [ ] 将 Profile、摘要、翻译、分块与缓存事务下沉 `papr-core`。
3. [ ] 增加 Keystore 凭据适配与密钥泄漏回归。
4. [ ] 暴露 FRB 流式/取消 API 并验证二次生成幂等。
5. [ ] 实现 Flutter Profile、摘要、翻译和进度/取消 UI。
6. [ ] 运行 Core/Bridge/桌面/Flutter 测试、analyze 与 Debug APK。
7. [ ] 在实体机验证旋转、后台/前台切换、断网、取消和进程恢复。

## Rollback

依次按 Flutter/Android、FRB、Core 回滚；不得删除已有文章缓存字段。
