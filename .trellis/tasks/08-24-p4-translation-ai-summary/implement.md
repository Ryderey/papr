# Implement: P4 翻译与 AI 摘要

1. [x] 盘点并测试桌面 AI/SSE/翻译语义，冻结 DTO、事件和错误码（Core 已覆盖双协议请求、SSE 分片/非法响应/中途限流、可唤醒取消、无密钥 Profile 与 success-only 缓存契约）。
2. [ ] 将 Profile、摘要、翻译、分块与缓存事务下沉 `papr-core`。（已完成 Profile、摘要/追问、HTML 分块基础与摘要缓存事务；待翻译 provider 适配和桌面复用。）
3. [x] 增加 Keystore 凭据适配与密钥泄漏回归（AES/GCM Android Keystore、私有密文存储、备份/迁移排除、平台通道输入测试）。
4. [x] 暴露 FRB 流式/取消 API 并验证二次生成幂等（摘要/追问 typed stream、缓存读取、请求注册自动清理、幂等取消与完整缓存原子替换）。
5. [ ] 实现 Flutter Profile、摘要、翻译和进度/取消 UI。（已完成 Profile 管理、右下角新增入口、Keystore 驱动的连接探测、全屏流式摘要、缓存/重生成/取消与内存追问；待翻译 UI。）
6. [ ] 运行 Core/Bridge/桌面/Flutter 测试、analyze 与 Debug APK。
7. [ ] 在实体机验证旋转、后台/前台切换、断网、取消和进程恢复。

## Rollback

依次按 Flutter/Android、FRB、Core 回滚；不得删除已有文章缓存字段。
