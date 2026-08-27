# Design: P4 翻译与 AI 摘要

## Boundary

从 `src-tauri/src/ai.rs` 与 `src-tauri/src/translate.rs` 提取平台无关实现到 `papr-core`。Tauri 与 FRB 映射同一 DTO/事件；Android 平台通道只做 Keystore 读写。桌面当前把 `api_key` 放入设置 JSON 的兼容实现不得迁移到移动端。

## Contracts

- `AiProfile` 仅存入设置中的元数据：id、名称、协议、模型、base URL、认证方式、可选自定义请求头、enabled、`credential_ref` 与 Summary/Translate 默认用途；序列化模型中禁止 `api_key` 字段。移动端不支持 Ask/Digest 用途。
- 移动端只允许一个 Profile 处于生效状态。启用某项时 Core 在同一次设置写入中关闭其余项；关闭当前项时不选择备用项，因此可以没有生效的 AI 配置，摘要页此时只提示用户先启用配置而不发起网络请求。读取旧版多生效配置时，Core 按旧摘要选择顺序保留一个并持久化关闭其余项。
- Profile 连接探测从 Keystore 临时取用凭据，并调用与摘要完全相同的流式 Provider/SSE 路径，使用固定的最小请求提示；探测不接收文章正文、不创建请求缓存、不写入 SQLite。成功只返回完成状态，失败仍只透传稳定错误码。
- Android 以 `credential_ref` 为 Keystore alias。Flutter 仅在发起连接测试、摘要或 LLM 翻译的瞬间经平台通道读取密钥，并将其作为非持久化参数传入 FRB；Core 只在该请求生命周期内持有它，绝不写 SQLite、change_log、缓存、日志或错误 detail。Google/Bing 等无用户密钥引擎不读取 Keystore。
- 长任务由调用方生成 `request_id`，FRB 使用 `StreamSink<AiStreamEvent>` 传输 `delta`、`progress`、`completed` 与 `error`。取消按 request ID 幂等；完成、失败或取消后都必须释放请求状态。Flutter 以 `(article_id, request_id)` 作为页面状态键，切换文章时不复用前一篇的流。
- 摘要缓存写 `ai_summary`；翻译缓存写 `translated_html/translated_lang`，仅在完整成功后通过 Core 事务替换。网络、取消或解析失败不得覆盖旧缓存或原始正文。
- 稳定错误码为 `invalidAiProfile`、`noAiCredential`、`noArticleBody`、`emptyAiQuestion`、`aiAuth`、`aiRateLimited`、`aiNetwork`、`aiParse`、`aiCancelled`、`aiRequestNotFound`；Flutter 只本地化 stable code，且不展示 provider 原始响应体。

## Frozen portable seam

1. `papr-core::ai` owns profile validation, protocol-specific request construction, bounded SSE parsing, cancellation, summary prompts and success-only cache writes. It accepts an ephemeral `ResolvedAiCredential` supplied by the caller rather than depending on Android APIs.
2. `papr-core::translate` owns HTML block chunking, non-translatable-node preservation and engine adapters. It delegates LLM batches to `ai`; it reports batch-level progress, never token-level progress for a full article.
3. FRB converts typed DTOs and forwards stream events only. `mobile/lib/services/platform_service.dart` owns Keystore method-channel calls, and `mobile/lib/repositories` coordinates credential lookup, FRB invocation and provider invalidation.
4. Tauri remains an adapter during the extraction: its legacy settings loader supplies a transient credential to the new Core service so existing desktop behavior stays compatible without sharing the insecure mobile storage model.

## Mobile summary interaction

- 阅读器工具栏提供 AI 摘要入口，摘要以独立全屏页面承载，不把摘要面板与正文并排，也不使用受高度限制的底部面板。
- AI Profile 列表将新增 FAB 固定在右下角，与其他列表页一致；每个 Profile 行在配置弹窗外显示生效开关与连接探测入口，并在探测或切换期间禁用同类操作，避免重复的凭据请求或并发覆盖。
- 摘要页通过正常返回栈回到同一篇文章；阅读器拥有并恢复正文滚动位置，摘要页不得重建或重排正文内容。
- 命中完整摘要缓存时直接展示且不联网；无缓存时进入页面自动开始流式生成。重新生成必须由用户显式触发；离开生成中的页面会取消请求，残缺输出只存在内存且不得覆盖已有缓存。
- 全屏容器承载模板选择、流式正文、取消、重试、重新生成与针对当前摘要的连续追问；不加入全局 Ask/RAG 或 Digest 入口。
- 每次追问只向摘要用途 Profile 发送当前完整摘要、该摘要下本次已完成的问答历史和新问题，不读取或检索文章正文。系统提示要求超出摘要信息时明确说明边界；切换文章或重新生成摘要必须结束当前追问请求并清空其上下文。
- 追问问答只由摘要页状态持有，不定义持久化 DTO 或数据表。退出摘要页即取消进行中的追问并丢弃全部问答；再次进入只加载文章的摘要缓存。
- `articles` 增加可空的 `ai_summary_template`，与现有 `ai_summary` 组成单份“最近成功摘要”缓存。旧数据模板为空时仍展示摘要，并标记为未知/旧版模板；不为每个模板建立独立缓存行。
- 模板选择器区分“缓存实际模板”和“待生成模板”。选择新模板不发起请求；只有用户点击重新生成且流完整结束后，Core 才在同一事务内更新 `ai_summary` 与 `ai_summary_template`。取消、失败或非法响应保留二者旧值。
- `articles` 同时增加可空的 `ai_summary_lang`。摘要和追问使用界面语言（`zh` 为简体中文、`ja` 为日文，其余为英文）；成功缓存时在同一事务内写入摘要、模板和语言。界面语言变化只标记缓存语言差异，不自动发起请求。
- 全文提取成功会改变摘要输入源，因此 Core 在保存新 `extracted_html` 的同一事务内清除旧摘要、模板与语言缓存；提取失败仍保留旧正文和旧摘要。

## Security and Rollback

平台凭据写入成功后才提交 Profile 引用；删除 Profile 时先删除元数据，再尽力删除对应 Keystore 条目，不把删除失败的密钥内容带回 Flutter。Core、FRB、Flutter/Android 分层提交，可关闭入口而保留兼容数据。
