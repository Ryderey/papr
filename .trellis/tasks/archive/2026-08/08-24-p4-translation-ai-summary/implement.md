# Implement: P4 翻译与 AI 摘要

1. [x] 盘点并测试桌面 AI/SSE/翻译语义，冻结 DTO、事件和错误码（Core 已覆盖双协议请求、SSE 分片/非法响应/中途限流、可唤醒取消、无密钥 Profile 与 success-only 缓存契约）。
2. [x] 将 Profile、摘要、翻译、分块与缓存事务下沉 `papr-core`。（已完成 Profile、摘要/追问、HTML 分块基础、摘要缓存事务和 LLM 批次翻译的完整成功缓存。产品决策明确不接入 Google/DeepL/Bing 的官方或非官方接口。）
3. [x] 增加 Keystore 凭据适配与密钥泄漏回归（AES/GCM Android Keystore、私有密文存储、备份/迁移排除、平台通道输入测试）。
4. [x] 暴露 FRB 流式/取消 API 并验证二次生成幂等（摘要/追问 typed stream、缓存读取、请求注册自动清理、幂等取消与完整缓存原子替换）。
5. [x] 实现 Flutter Profile、摘要、翻译和进度/取消 UI。（已完成 Profile 管理、右下角新增入口、Keystore 驱动的连接探测、全屏流式摘要、缓存/重生成/取消与内存追问，以及全屏 HTML 翻译、目标语言选择、批次进度与取消。）
6. [x] 运行 Core/Bridge/Flutter 测试、analyze 与 Debug APK。（Flutter analyze 与 24 项测试通过；Debug APK 构建成功。FRB 编译保留已有 `frb_expand` 宏警告，未新增。）
7. [x] 在实体机验证旋转、后台/前台切换、断网、取消和进程恢复。
8. [x] 修复 SenseNova 6.8 reasoning 占满通用输出上限后摘要没有可见正文的问题，并验证错误文案与 Provider 隔离。（Core reasoning-only SSE 回归通过；实体机日日新摘要验证通过。）

## Rollback

依次按 Flutter/Android、FRB、Core 回滚；不得删除已有文章缓存字段。
