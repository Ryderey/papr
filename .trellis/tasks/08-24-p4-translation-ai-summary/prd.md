# P4 翻译与 AI 摘要

## Goal

把桌面端已验证的摘要、HTML 分块翻译和流式协议下沉到 `papr-core`，通过 FRB 在移动阅读器中提供摘要与翻译，并由 Android Keystore 保存 API Key。

## Requirements

1. 支持 Anthropic Messages 与 OpenAI Chat Completions 兼容端点，多 Profile、模型、Base URL、认证方式和连接测试。
2. SQLite 只保存 Profile 元数据与 Keystore 引用；密钥不得进入数据库、日志、错误、备份或同步。
3. 摘要支持模板、流式增量、缓存、重新生成、取消、重试；切换文章必须隔离状态。
4. 翻译支持 LLM、Google、DeepL、Bing，按 HTML 块处理并保留链接、图片、代码块与层级；缓存最近目标语言。
5. Core 拥有 HTTP、SSE、分块、验证和缓存写入；Flutter 只编排页面状态和稳定错误码。
6. 明确排除追问、文章问答、Ask/RAG、Digest 与向量数据库。

## Acceptance Criteria

- [ ] HTTP/SSE 测试覆盖分片、超时、取消、限流、非法响应和中途失败。
- [ ] 翻译后 HTML 结构与不可翻译节点保持正确，失败保留原文和旧缓存。
- [ ] API Key 不出现在 SQLite、日志、错误消息或同步负载。
- [ ] 摘要/翻译在文章切换、重试、取消后不串状态。
- [ ] FRB 生成幂等，Core/Bridge/Flutter/桌面回归与 Debug APK 通过。

## Dependencies / Out of Scope

依赖 P3 归档。遵守父任务“明确排除”清单；不借本任务恢复 Ask、RAG 或 Digest。
