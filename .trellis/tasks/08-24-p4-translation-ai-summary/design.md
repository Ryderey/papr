# Design: P4 翻译与 AI 摘要

## Boundary

从 `src-tauri/src/ai.rs` 与 `src-tauri/src/translate.rs` 提取平台无关实现到 `papr-core`。Tauri 与 FRB 映射同一 DTO/事件；Android 平台通道只做 Keystore 读写。桌面当前把 `api_key` 放入设置 JSON 的兼容实现不得迁移到移动端。

## Contracts

- `AiProfile` 仅存入设置中的元数据：id、名称、协议、模型、base URL、认证方式、可选自定义请求头、enabled、`credential_ref` 与 Summary/Translate 默认用途；序列化模型中禁止 `api_key` 字段。移动端不支持 Ask/Digest 用途。
- Android 以 `credential_ref` 为 Keystore alias。Flutter 仅在发起连接测试、摘要或 LLM 翻译的瞬间经平台通道读取密钥，并将其作为非持久化参数传入 FRB；Core 只在该请求生命周期内持有它，绝不写 SQLite、change_log、缓存、日志或错误 detail。Google/Bing 等无用户密钥引擎不读取 Keystore。
- 长任务由调用方生成 `request_id`，FRB 使用 `StreamSink<AiStreamEvent>` 传输 `delta`、`progress`、`completed` 与 `error`。取消按 request ID 幂等；完成、失败或取消后都必须释放请求状态。Flutter 以 `(article_id, request_id)` 作为页面状态键，切换文章时不复用前一篇的流。
- 摘要缓存写 `ai_summary`；翻译缓存写 `translated_html/translated_lang`，仅在完整成功后通过 Core 事务替换。网络、取消或解析失败不得覆盖旧缓存或原始正文。
- 稳定错误码为 `invalidAiProfile`、`noAiCredential`、`aiAuth`、`aiRateLimited`、`aiNetwork`、`aiParse`、`aiCancelled`、`aiRequestNotFound`；Flutter 只本地化 stable code，且不展示 provider 原始响应体。

## Frozen portable seam

1. `papr-core::ai` owns profile validation, protocol-specific request construction, bounded SSE parsing, cancellation, summary prompts and success-only cache writes. It accepts an ephemeral `ResolvedAiCredential` supplied by the caller rather than depending on Android APIs.
2. `papr-core::translate` owns HTML block chunking, non-translatable-node preservation and engine adapters. It delegates LLM batches to `ai`; it reports batch-level progress, never token-level progress for a full article.
3. FRB converts typed DTOs and forwards stream events only. `mobile/lib/services/platform_service.dart` owns Keystore method-channel calls, and `mobile/lib/repositories` coordinates credential lookup, FRB invocation and provider invalidation.
4. Tauri remains an adapter during the extraction: its legacy settings loader supplies a transient credential to the new Core service so existing desktop behavior stays compatible without sharing the insecure mobile storage model.

## Security and Rollback

平台凭据写入成功后才提交 Profile 引用；删除 Profile 时先删除元数据，再尽力删除对应 Keystore 条目，不把删除失败的密钥内容带回 Flutter。Core、FRB、Flutter/Android 分层提交，可关闭入口而保留兼容数据。
