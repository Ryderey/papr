# Design: P4 翻译与 AI 摘要

## Boundary

从 `src-tauri/src/ai.rs` 与 `src-tauri/src/translate.rs` 提取平台无关实现到 `papr-core`。Tauri 与 FRB 映射同一 DTO/事件；Android 平台通道只做 Keystore 读写，Core 通过凭据解析接口按引用取密钥。

## Contracts

- `AiProfile` 不包含密钥，仅包含 provider、model、base URL、credential ref 与默认用途。
- 长任务返回 request ID，并产生 text/progress/completed/error 事件；取消按 request ID 幂等。
- 摘要缓存写 `ai_summary`；翻译缓存写 `translated_html/translated_lang`，仅成功完成后原子替换。
- 错误分为 validation/auth/rateLimit/network/parse/cancelled，Flutter 只本地化稳定码。

## Security and Rollback

平台凭据写入成功后才提交 Profile 引用；删除 Profile 同时删除对应 Keystore 条目。Core、FRB、Flutter/Android 分层提交，可关闭入口而保留兼容数据。
