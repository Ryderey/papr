# Spec: Papr Multi-LLM Provider Adapter Layer

## 0. Scope

Project path:

```text
D:\Work\SSG\papr
```

This document is a specification for a future Codex implementation task.

The goal is to refactor the current AI configuration, which only supports native Anthropic and OpenAI paths, into a multi-LLM provider adapter layer. The new design should support both Anthropic Messages-compatible APIs and OpenAI Chat Completions-compatible APIs, enabling dynamic model/profile switching similar to Claude Code-style provider selection and making it easier to connect external Agent backends.

This Spec is architecture-only. It must not be treated as implementation code.

Hard constraints for the implementation phase:

- Preserve all existing AI features.
- Preserve current streaming behavior and frontend event semantics.
- Preserve backward compatibility with existing settings.
- Do not introduce large SDK dependencies unless explicitly justified.
- Do not perform unrelated refactoring.
- Do not delete legacy settings during the first migration.

Existing features that must continue working:

- AI article summary
- AI summary follow-up Q&A (`ai_summarize_follow_up`)
- Ask-the-article / RAG question answering
- AI Digest
- LLM translation
- SSE streaming
- Existing `AiEvent` and `TranslateEvent` frontend event behavior
- Existing `ai_provider`, `ai_api_key`, `ai_model`, and `ai_base_url` settings

---

## 1. Current Architecture Summary

The current project already has a basic provider abstraction, but it is tightly bound to two built-in providers.

Relevant files:

```text
src-tauri/src/ai.rs
src-tauri/src/commands.rs
src/components/SettingsDialog.tsx
src/api.ts
```

Current backend shape:

```text
src-tauri/src/ai.rs
- Provider enum: Anthropic | OpenAi
- AiConfig: provider, api_key, model, base_url
- stream_chat(...)
- complete_chat(...)
- stream_anthropic(...)
- stream_openai(...)
- consume_sse(...)
- extract_delta(...)
- extract_error(...)
```

Current settings loading:

```text
src-tauri/src/commands.rs
- load_ai_config() reads:
  - ai_provider
  - ai_api_key
  - ai_model
  - ai_base_url
```

Current frontend limitation:

```text
src/components/SettingsDialog.tsx
- provider type is fixed to "anthropic" | "openai"
- provider selection UI only supports Anthropic and OpenAI
- switching provider clears model and base URL
```

Conclusion:

The current code already separates most AI calls behind `stream_chat` and `complete_chat`. The safest refactor is to keep those public call sites stable while replacing the internal `Provider` branching with a protocol-based adapter layer.

---

## 2. Design Goals

### 2.1 Primary Goals

Introduce a three-layer architecture:

```text
Business Feature Layer
  ai_summarize / ai_ask / ai_digest / ai_translate
        ↓
Unified LLM Interface Layer
  stream_chat / complete_chat / LlmRequest / LlmProfile / LlmStreamEvent
        ↓
Protocol Adapter Layer
  Anthropic Messages Adapter
  OpenAI Chat Completions Adapter
        ↓
Provider Profile Configuration
  Anthropic official
  OpenAI official
  OpenRouter
  DeepSeek
  Groq
  Ollama
  LiteLLM
  Hermes Gateway
  Custom compatible endpoint
```

### 2.2 Non-Goals

Do not implement the following in the first phase:

- Tool calling / function calling
- Multi-turn conversation persistence
- Embeddings
- Reranking
- Semantic search
- Online model list discovery
- Provider speed benchmark
- Token usage accounting UI
- Cloud sync for API keys
- Full Agent runtime orchestration
- A new RAG retrieval pipeline
- A rewrite of non-LLM translation engines

---

## 3. Core Design Principles

### 3.1 Separate Provider Profile from Protocol

The current `provider` value mixes two concepts:

```text
Provider brand: Anthropic, OpenAI, OpenRouter, DeepSeek, Groq, Ollama
Protocol shape: Anthropic Messages, OpenAI Chat Completions
```

The refactor should separate them.

A provider profile is a user-configured endpoint:

```text
Profile = name + protocol + base_url + api_key + model + optional headers
```

A protocol adapter defines how to call and parse that endpoint:

```text
Protocol = request body format + auth header format + SSE delta parser
```

Example:

```text
Profile ID: openrouter-claude
Name: OpenRouter Claude
Protocol: openai_chat_completions
Base URL: https://openrouter.ai/api/v1
Model: anthropic/claude-sonnet-4.5
```

Example:

```text
Profile ID: anthropic-official
Name: Anthropic Official
Protocol: anthropic_messages
Base URL: https://api.anthropic.com/v1
Model: claude-sonnet-4-6
```

This allows many providers to share one protocol adapter.

---

### 3.2 Preserve Existing Business-Level API

Existing business functions should not need to know the provider details.

The existing conceptual API is:

```text
stream_chat(client, cfg, system, user, channel, max_tokens)
complete_chat(client, cfg, system, user, max_tokens)
```

The implementation may internally convert this into a structured `LlmRequest`, but the first phase should avoid forcing all call sites to change at once.

Recommended first-phase strategy:

```text
Existing business call sites
        ↓
stream_chat / complete_chat compatibility wrapper
        ↓
LlmRequest
        ↓
Protocol adapter
```

---

### 3.3 Keep Streaming Semantics Stable

Current frontend events:

```text
AiEvent
- Delta(String)
- Done
- Error(String)
```

The new adapter layer can use an internal event type, but it must map back to the existing public event contract.

Recommended internal event type:

```text
LlmStreamEvent
- TextDelta(String)
- Completed
- Error(String)
```

The UI should continue receiving `AiEvent::Delta`, `AiEvent::Done`, and `AiEvent::Error` with the same behavior as before.

---

## 4. Unified Interface Definitions

The following definitions describe the intended contract. They are not final implementation code.

### 4.1 LlmProtocol

```text
LlmProtocol
- anthropic_messages
- openai_chat_completions
```

Reserved for future expansion:

```text
- openai_responses
- openai_legacy_completions
- google_genai
```

Rules:

- `anthropic_messages` is for Anthropic Messages API-compatible endpoints.
- `openai_chat_completions` is for OpenAI Chat Completions-compatible endpoints.
- OpenRouter, DeepSeek, Groq, LiteLLM, Ollama OpenAI mode, and many local gateways should normally use `openai_chat_completions`.
- Do not confuse provider brand with protocol.

---

### 4.2 LlmPurpose

```text
LlmPurpose
- summary
- ask
- digest
- translate
```

Purpose:

- Enables different features to use different profiles.
- Allows summary to use a cheaper model while translation uses a longer-context model.
- Provides a foundation for Claude Code-style dynamic model switching.

Future possible values:

```text
- agent
- rewrite
- extract
- custom
```

---

### 4.3 LlmMessage

```text
LlmMessage
- role: system | user | assistant
- content: string
```

Current project calls can still be represented as:

```text
system: "..."
messages:
  - role: user
    content: "..."
```

Adapter conversion rules:

Anthropic Messages:

```text
system -> top-level system field
messages -> messages array
```

OpenAI Chat Completions:

```text
system -> first message with role "system"
messages -> appended after the system message
```

---

### 4.4 LlmParams

```text
LlmParams
- temperature?: number
- top_p?: number
- max_tokens?: number
- stop?: string[]
- timeout_seconds?: number
```

Rules:

- The business layer's explicit `max_tokens` should take precedence over profile defaults.
- Unsupported params should be ignored or safely omitted by the adapter.
- Do not fail a request just because a profile includes an adapter-unsupported optional param.
- Keep the current AI timeout behavior as the default, currently 300 seconds.

---

### 4.5 LlmProfile

```text
LlmProfile
- id: string
- name: string
- protocol: LlmProtocol
- base_url: string
- api_key?: string
- api_key_env?: string
- model: string
- enabled: bool
- default_for?: LlmPurpose[]
- headers?: HeaderMap
- params?: LlmParams
- auth?: LlmAuthMode
```

Field meanings:

| Field | Meaning |
|---|---|
| `id` | Stable unique identifier used by active/default profile settings. |
| `name` | Human-readable label shown in UI. |
| `protocol` | Request/response protocol. Not the provider brand. |
| `base_url` | API root without the final request path. |
| `api_key` | User-entered credential. |
| `api_key_env` | Optional future env var lookup. |
| `model` | Model string sent to the upstream provider. |
| `enabled` | Whether this profile is selectable. |
| `default_for` | Optional list of feature purposes that default to this profile. |
| `headers` | Optional custom headers. |
| `params` | Optional model params. |
| `auth` | Optional auth strategy override. |

Recommended auth modes:

```text
LlmAuthMode
- bearer
- x_api_key
- none
```

Default auth mapping:

```text
anthropic_messages -> x_api_key
openai_chat_completions -> bearer
```

Local endpoints such as Ollama may use `none` or a dummy key depending on compatibility behavior.

---

### 4.6 LlmRequest

```text
LlmRequest
- profile: LlmProfile
- purpose: LlmPurpose
- system: string
- messages: LlmMessage[]
- max_tokens: number
- stream: bool
```

The first implementation may keep the existing call signatures and build `LlmRequest` internally.

---

### 4.7 LlmAdapter

```text
LlmAdapter
- protocol: LlmProtocol
- build_request(profile, request) -> HttpRequestSpec
- parse_stream_event(json) -> Option<LlmStreamEvent>
- parse_error(json) -> Option<String>
```

`HttpRequestSpec`:

```text
HttpRequestSpec
- method: POST
- url: string
- headers: HeaderMap
- body: JsonValue
- timeout_seconds: number
```

Adapter responsibilities:

1. Build request URL.
2. Build headers.
3. Build JSON body.
4. Parse SSE events.
5. Parse upstream error payloads.

Adapter non-responsibilities:

- Article summarization prompt logic
- RAG retrieval
- Translation batching
- Frontend state management
- Settings UI rendering

---

## 5. Protocol Adapter Rules

### 5.1 Anthropic Messages Adapter

Request path:

```text
{base_url}/messages
```

Default base URL:

```text
https://api.anthropic.com/v1
```

Default headers:

```text
x-api-key: <api_key>
anthropic-version: 2023-06-01
content-type: application/json
```

Request body fields:

```text
model
max_tokens
system
stream
messages
```

Message conversion:

```text
LlmRequest.system -> top-level system
LlmRequest.messages -> messages array
```

Text delta parsing:

```text
type == "content_block_delta"
delta.text -> TextDelta
```

Error parsing:

```text
type == "error"
error.message -> Error
```

---

### 5.2 OpenAI Chat Completions Adapter

Request path:

```text
{base_url}/chat/completions
```

Default base URL:

```text
https://api.openai.com/v1
```

Default headers:

```text
Authorization: Bearer <api_key>
content-type: application/json
```

Request body fields:

```text
model
max_tokens
stream
messages
```

Message conversion:

```text
[
  { "role": "system", "content": system },
  ...request.messages
]
```

Text delta parsing:

```text
choices[0].delta.content -> TextDelta
```

Error parsing:

```text
error.message -> Error
```

Compatibility requirements:

- `error: null` must not be treated as an error.
- A chunk with no `choices` must be safely ignored.
- A final SSE frame without a trailing newline must not be dropped.
- `[DONE]` should be ignored as a protocol terminator, not rendered as text.

---

## 6. Settings and Configuration Format

The current project uses key-value settings. The safest first step is to store multi-profile configuration in a JSON setting while retaining old keys.

### 6.1 New Settings Keys

```text
ai_profiles_json
ai_active_profile_id
ai_default_summary_profile_id
ai_default_ask_profile_id
ai_default_digest_profile_id
ai_default_translate_profile_id
```

### 6.2 Legacy Settings Keys to Preserve

```text
ai_provider
ai_api_key
ai_model
ai_base_url
```

Do not delete these during the first migration.

---

### 6.3 Full `ai_profiles_json` Example

```json
{
  "version": 1,
  "profiles": [
    {
      "id": "anthropic-official",
      "name": "Anthropic Official",
      "protocol": "anthropic_messages",
      "base_url": "https://api.anthropic.com/v1",
      "api_key": "",
      "model": "claude-sonnet-4-6",
      "enabled": true,
      "default_for": ["summary", "ask", "digest"],
      "auth": "x_api_key"
    },
    {
      "id": "openai-official",
      "name": "OpenAI Official",
      "protocol": "openai_chat_completions",
      "base_url": "https://api.openai.com/v1",
      "api_key": "",
      "model": "gpt-4.1-mini",
      "enabled": true,
      "default_for": [],
      "auth": "bearer"
    },
    {
      "id": "openrouter-claude",
      "name": "OpenRouter Claude",
      "protocol": "openai_chat_completions",
      "base_url": "https://openrouter.ai/api/v1",
      "api_key": "",
      "model": "anthropic/claude-sonnet-4.5",
      "enabled": true,
      "headers": {
        "HTTP-Referer": "https://papr.local",
        "X-Title": "Papr"
      },
      "default_for": ["translate"],
      "auth": "bearer"
    },
    {
      "id": "local-ollama",
      "name": "Local Ollama",
      "protocol": "openai_chat_completions",
      "base_url": "http://127.0.0.1:11434/v1",
      "api_key": "ollama",
      "model": "qwen2.5:7b",
      "enabled": false,
      "default_for": [],
      "auth": "bearer"
    }
  ]
}
```

---

### 6.4 Minimal Single-Profile Example

```json
{
  "version": 1,
  "active_profile_id": "deepseek",
  "profiles": [
    {
      "id": "deepseek",
      "name": "DeepSeek",
      "protocol": "openai_chat_completions",
      "base_url": "https://api.deepseek.com/v1",
      "api_key": "sk-...",
      "model": "deepseek-chat",
      "enabled": true,
      "auth": "bearer"
    }
  ]
}
```

---

## 7. Dynamic Profile Selection

### 7.1 Phase 1: Active Profile Only

Minimum viable behavior:

```text
All AI features use ai_active_profile_id.
```

Fallback order:

```text
ai_active_profile_id
↓
first enabled profile in ai_profiles_json
↓
legacy ai_provider / ai_api_key / ai_model / ai_base_url
↓
noAiProfile or noAiKey error
```

---

### 7.2 Phase 2: Purpose-Aware Profile Selection

Target behavior:

```text
summary   -> ai_default_summary_profile_id
ask       -> ai_default_ask_profile_id
digest    -> ai_default_digest_profile_id
translate -> ai_default_translate_profile_id
fallback  -> ai_active_profile_id
```

Fallback order for each purpose:

```text
purpose-specific profile
↓
active profile
↓
first enabled profile
↓
legacy settings fallback
↓
error
```

This enables Claude Code-style model switching without making each feature manually manage endpoint details.

---

## 8. Backend Refactor Plan

### 8.1 Recommended Module Layout

The current implementation can remain in `ai.rs` for a smaller first change, but the preferred final layout is:

```text
src-tauri/src/ai.rs
  Public entry points:
  - stream_chat
  - complete_chat
  - AiEvent
  - ChatOutcome

src-tauri/src/ai/config.rs
  - StoredAiConfig
  - ResolvedAiConfig
  - LlmProfile
  - LlmProtocol
  - legacy settings fallback

src-tauri/src/ai/request.rs
  - LlmRequest
  - LlmMessage
  - LlmPurpose
  - LlmParams

src-tauri/src/ai/adapters/anthropic.rs
  - Anthropic Messages request builder
  - Anthropic stream parser

src-tauri/src/ai/adapters/openai.rs
  - OpenAI Chat Completions request builder
  - OpenAI-compatible stream parser

src-tauri/src/ai/stream.rs
  - shared SSE consumption
  - line buffer handling
  - channel-forwarding behavior
```

For a low-risk first implementation, Codex may keep these as internal sections in `src-tauri/src/ai.rs` and split later.

---

### 8.2 Config Model Refactor

Current shape:

```text
AiConfig
- provider
- api_key
- model
- base_url
```

Target conceptual shape:

```text
StoredAiConfig
- version
- profiles
- active_profile_id

ResolvedAiConfig
- profile
```

Recommended distinction:

```text
StoredAiConfig = raw persisted profile collection
ResolvedAiConfig = one selected profile for one request
```

---

### 8.3 `load_ai_config` Refactor

Current:

```text
load_ai_config(conn) -> AiConfig
```

Target:

```text
load_ai_config_for(conn, purpose) -> ResolvedAiConfig
```

Recommended compatibility approach:

```text
load_ai_config(conn) -> active/default ResolvedAiConfig
load_ai_config_for(conn, purpose) -> purpose-aware ResolvedAiConfig
```

Mapping:

```text
ai_summarize -> LlmPurpose::Summary
ai_ask       -> LlmPurpose::Ask
ai_digest    -> LlmPurpose::Digest
ai_translate -> LlmPurpose::Translate
```

---

## 9. Frontend Refactor Plan

### 9.1 Phase 1 UI

Replace the current provider dropdown with profile management.

Fields:

```text
AI Profile
- Active profile selector
- Create profile
- Delete profile
- Enable / disable profile
- Name
- Protocol
- Base URL
- API Key
- Model
- Custom headers
```

Protocol options:

```text
Anthropic Messages
OpenAI Chat Completions Compatible
```

Preset buttons:

```text
Anthropic
OpenAI
OpenRouter
DeepSeek
Groq
Ollama
Custom
```

Preset behavior:

- Fill protocol.
- Fill base URL placeholder or value.
- Fill model placeholder if useful.
- Never fill API key.
- Do not silently delete existing profiles.

---

### 9.2 Phase 2 UI

Add per-purpose model selection:

```text
Summary model
Ask model
Digest model
Translate model
```

Each selector should include:

```text
Use active profile
<enabled profile list>
```

---

## 10. Backward Compatibility and Migration

### 10.1 Legacy Migration Rule

If `ai_profiles_json` is empty or missing, read legacy settings:

```text
ai_provider
ai_api_key
ai_model
ai_base_url
```

Then build an in-memory profile:

```text
if ai_provider == "openai":
  id: legacy-openai
  name: OpenAI
  protocol: openai_chat_completions
  base_url: ai_base_url or https://api.openai.com/v1
  model: ai_model or gpt-4.1-mini
  api_key: ai_api_key
  auth: bearer

otherwise:
  id: legacy-anthropic
  name: Anthropic
  protocol: anthropic_messages
  base_url: ai_base_url or https://api.anthropic.com/v1
  model: ai_model or claude-sonnet-4-6
  api_key: ai_api_key
  auth: x_api_key
```

### 10.2 Persistence Strategy

Either strategy is acceptable:

```text
Lazy migration:
- Build legacy profile in memory.
- Do not write ai_profiles_json until the user saves AI settings.
```

or:

```text
Explicit migration:
- Generate ai_profiles_json on first successful settings save.
- Keep legacy keys untouched.
```

Do not delete old keys in the first implementation.

---

## 11. Error Handling

Suggested new error codes:

```text
noAiProfile
noAiKey
invalidAiProfile
invalidAiProtocol
invalidAiBaseUrl
invalidAiConfig
```

Error priority:

```text
1. No enabled profile -> noAiProfile
2. Missing key for a profile requiring auth -> noAiKey
3. Unknown protocol -> invalidAiProtocol
4. Empty or invalid base URL -> invalidAiBaseUrl
5. Upstream non-2xx HTTP response -> keep current AI API error format
6. Invalid or non-SSE stream -> keep current stream error behavior
```

Important compatibility rules:

- Preserve the existing `noAiKey` behavior where possible.
- Do not expose API keys in logs or error messages.
- Do not include full request headers in user-facing errors.
- Surface upstream response body only where current behavior already does so.

---

## 12. Testing Requirements

### 12.1 Config Tests

Required cases:

```text
- Legacy anthropic settings resolve to anthropic_messages.
- Legacy openai settings resolve to openai_chat_completions.
- Empty API key still returns noAiKey.
- API key is trimmed.
- Model name is trimmed.
- Base URL is trimmed.
- Base URL trailing slash is removed.
- Unknown protocol returns invalidAiProtocol.
- Disabled profiles are not selected.
- Purpose profile falls back to active profile when missing.
```

---

### 12.2 Adapter Request Tests

Anthropic adapter:

```text
- URL ends with /messages.
- Headers contain x-api-key.
- Headers contain anthropic-version.
- System prompt is top-level system.
- User message is in messages array.
```

OpenAI-compatible adapter:

```text
- URL ends with /chat/completions.
- Headers use Authorization: Bearer <key>.
- System prompt becomes first system message.
- Existing messages preserve order.
- Custom headers are included.
```

---

### 12.3 SSE Parser Tests

Preserve existing tests and add:

```text
- OpenAI-compatible chunk with error: null is not an error.
- OpenAI-compatible chunk with no choices is ignored.
- OpenAI-compatible real error object is surfaced.
- Anthropic content_block_delta is parsed.
- Anthropic error event is surfaced.
- Final SSE frame without trailing newline is parsed.
- [DONE] is ignored.
```

---

### 12.4 Regression Commands

Recommended verification commands:

```text
pnpm test
pnpm build
```

If Rust tests are relevant after implementation:

```text
cd src-tauri
cargo test
```

Codex must not claim tests passed unless they were actually run.

---

## 13. Recommended Implementation Phases

### Phase 1: Backend Adapter Abstraction

Goal:

```text
Support profile-based backend config while preserving existing UI as much as possible.
```

Tasks:

```text
1. Introduce LlmProtocol, LlmProfile, LlmRequest, and LlmPurpose.
2. Move request-building logic into protocol adapters.
3. Keep stream_chat and complete_chat public behavior stable.
4. Add ai_profiles_json parsing.
5. Add legacy settings fallback.
6. Add config and SSE tests.
```

Acceptance criteria:

```text
- Existing Anthropic official path still works.
- Existing OpenAI official path still works.
- Existing custom OpenAI-compatible base URL still works.
- Tests pass.
```

---

### Phase 2: Frontend Profile UI

Goal:

```text
Let users create, edit, delete, and select multiple LLM profiles.
```

Tasks:

```text
1. Replace provider dropdown with active profile selector.
2. Add profile editor fields.
3. Add protocol selector.
4. Add common provider presets.
5. Save to ai_profiles_json.
6. Preserve legacy settings display via generated legacy profile.
```

Acceptance criteria:

```text
- User can create an OpenAI-compatible profile.
- User can create an Anthropic-compatible profile.
- User can switch active profile.
- Existing single-provider settings are shown as a usable profile.
```

---

### Phase 3: Purpose-Aware Dynamic Switching

Goal:

```text
Support different models/profiles for summary, ask, digest, and translation.
```

Tasks:

```text
1. Add load_ai_config_for(conn, purpose).
2. Pass LlmPurpose from AI command handlers.
3. Add per-purpose profile selection UI.
4. Implement fallback to active profile.
5. Add tests for purpose fallback.
```

Acceptance criteria:

```text
- Summary and translation can use different profiles.
- Missing purpose-specific profile falls back to active profile.
- Deleted or disabled profile does not break all AI features.
```

---

## 14. Minimum Delivery Criteria

The implementation is acceptable when all of the following are true:

```text
1. Native Anthropic still works.
2. Native OpenAI still works.
3. OpenAI Chat Completions-compatible providers can be configured with base_url, model, and api_key.
4. Anthropic Messages-compatible providers can be configured with base_url, model, and api_key.
5. AI summary, AI ask, AI digest, and LLM translation all use the unified adapter layer.
6. Streaming output behavior is unchanged from the user's perspective.
7. Legacy settings remain usable.
8. No large unnecessary SDK dependency is added.
9. Tests cover config migration, adapter request construction, and SSE parsing.
```

---

## 15. Codex Implementation Prompt

Use the following prompt when asking Codex to implement this Spec:

```text
Read docs/multi-llm-provider-adapter-spec.md first. Implement the multi-LLM provider adapter layer for the Papr project.

Strict constraints:
- Do not remove existing AI features.
- Preserve existing Anthropic and OpenAI behavior.
- Preserve current streaming AiEvent semantics.
- Preserve old settings compatibility: ai_provider, ai_api_key, ai_model, ai_base_url.
- Prefer a protocol-based adapter design: anthropic_messages and openai_chat_completions.
- Do not add large SDK dependencies.
- Add tests for config parsing, migration, request construction, and SSE parsing.
- Keep the first implementation as small and safe as possible.

Recommended implementation phases:
1. Backend adapter abstraction and profile config.
2. Legacy settings fallback.
3. Tests.
4. Only then update SettingsDialog UI for multiple profiles.
5. Add purpose-aware profile selection after the base abstraction is stable.

Do not perform unrelated refactoring.
Report modified files and verification commands after implementation.
```
