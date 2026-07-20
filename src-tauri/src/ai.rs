//! AI features: cloud LLM streaming for article summaries and RAG Q&A.
//! Provider-agnostic (Anthropic / OpenAI); a local backend can later implement
//! the same `stream_chat` contract.

use crate::error::{AppError, AppResult};
use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Request, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tauri::ipc::Channel;

/// Map a non-success HTTP response to an `AppError` naming the service and
/// carrying the response body; returns the response unchanged on success.
async fn ensure_success(resp: Response, service: &str) -> AppResult<Response> {
    if resp.status().is_success() {
        return Ok(resp);
    }
    let status = resp.status();
    let detail = resp.text().await.unwrap_or_default();
    Err(AppError::other(format!(
        "{service} error {status}: {detail}"
    )))
}

/// Per-request cap for AI streaming. The shared HTTP client carries the
/// feed-fetch timeout (~30s), which would truncate a long generation — so AI
/// requests override it with a generous bound.
const AI_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

/// Output token cap for summaries / Q&A / digests, applied to every provider so
/// a response stays bounded in length and cost. These all fit comfortably within
/// it. Translation overrides it with [`TRANSLATE_MAX_TOKENS`].
pub const MAX_TOKENS: u32 = 2048;

/// Output token cap for one translation batch. A batch's translated HTML tracks
/// its input length (tags are echoed too), so it needs far more room than a
/// summary. Paired with [`TRANSLATE_CHUNK_BUDGET`] so a batch fits under it.
pub const TRANSLATE_MAX_TOKENS: u32 = 4096;

/// Input character budget for one translation batch, used by
/// `translate::chunk_blocks`. Chosen alongside [`TRANSLATE_MAX_TOKENS`] so the
/// translated output of a full batch stays under the output cap.
pub const TRANSLATE_CHUNK_BUDGET: usize = 3000;

/// Hard cap on the SSE line buffer. A well-behaved provider delimits every
/// event with a newline, so the buffer never holds more than a single frame.
/// A misbehaving or non-SSE endpoint (the base URL can point at any
/// OpenAI-compatible server, including a local one) could instead stream bytes
/// with no newline at all — without this cap that response would accumulate in
/// memory unbounded. 8 MiB is far larger than any genuine SSE frame while
/// still stopping a runaway stream, mirroring `fetch::MAX_BODY_BYTES`.
const MAX_SSE_BUFFER: usize = 8 * 1024 * 1024;

/// Token-level events streamed to the frontend over an `ipc::Channel`.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum AiEvent {
    Delta(String),
    Done,
    Error(String),
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LlmProtocol {
    AnthropicMessages,
    #[serde(rename = "openai_chat_completions")]
    OpenAiChatCompletions,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LlmPurpose {
    Summary,
    Ask,
    Digest,
    Translate,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LlmAuthMode {
    Bearer,
    #[serde(rename = "x_api_key")]
    XApiKey,
    None,
}

#[derive(Clone, Serialize)]
pub struct LlmMessage {
    role: String,
    content: String,
}

#[allow(dead_code)]
#[derive(Clone, Default, Deserialize)]
pub struct LlmParams {
    temperature: Option<f32>,
    top_p: Option<f32>,
    max_tokens: Option<u32>,
    stop: Option<Vec<String>>,
    timeout_seconds: Option<u64>,
}

#[allow(dead_code)]
#[derive(Clone, Deserialize)]
pub struct LlmProfile {
    id: String,
    name: String,
    protocol: LlmProtocol,
    base_url: String,
    #[serde(default)]
    api_key: String,
    model: String,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default)]
    default_for: Vec<LlmPurpose>,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    params: LlmParams,
    #[serde(default)]
    auth: Option<LlmAuthMode>,
}

#[derive(Clone, Deserialize)]
pub struct LlmProfileInput {
    #[serde(default)]
    pub name: String,
    pub protocol: LlmProtocol,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub auth: Option<LlmAuthMode>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Deserialize)]
struct StoredAiConfig {
    version: u32,
    #[serde(default)]
    profiles: Vec<LlmProfile>,
    active_profile_id: Option<String>,
}

fn default_enabled() -> bool {
    true
}

/// Resolved AI configuration read from the settings table.
pub struct AiConfig {
    profile: LlmProfile,
}

impl AiConfig {
    /// Build a config from raw legacy settings, applying per-provider defaults.
    /// Retained for tests and direct legacy config construction.
    #[allow(dead_code)]
    pub fn new(
        provider: Option<String>,
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
    ) -> AppResult<Self> {
        Self::legacy(provider, api_key, model, base_url)
    }

    /// Resolve a selected profile from the new JSON settings, falling back to
    /// legacy settings when no enabled stored profile can be selected.
    pub fn from_settings(
        ai_profiles_json: Option<String>,
        active_profile_id: Option<String>,
        purpose_profile_id: Option<String>,
        legacy_provider: Option<String>,
        legacy_api_key: Option<String>,
        legacy_model: Option<String>,
        legacy_base_url: Option<String>,
        _purpose: LlmPurpose,
    ) -> AppResult<Self> {
        if let Some(raw) = ai_profiles_json
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let value: Value =
                serde_json::from_str(raw).map_err(|_| AppError::code("invalidAiConfig"))?;
            reject_unknown_protocol(&value)?;
            let stored: StoredAiConfig =
                serde_json::from_value(value).map_err(|_| AppError::code("invalidAiConfig"))?;
            let _version = stored.version;

            let selected = [
                purpose_profile_id.as_deref(),
                active_profile_id.as_deref(),
                stored.active_profile_id.as_deref(),
            ]
            .into_iter()
            .flatten()
            .filter_map(|id| {
                stored
                    .profiles
                    .iter()
                    .find(|profile| profile.enabled && profile.id == id)
            })
            .next()
            .or_else(|| stored.profiles.iter().find(|profile| profile.enabled));

            if let Some(profile) = selected {
                return Ok(Self {
                    profile: normalize_profile(profile)?,
                });
            }
        }

        Self::legacy(
            legacy_provider,
            legacy_api_key,
            legacy_model,
            legacy_base_url,
        )
    }

    pub fn from_profile_input(input: LlmProfileInput) -> AppResult<Self> {
        let model = input.model.trim();
        let base_url = input.base_url.trim();
        let profile = LlmProfile {
            id: "connection-test".to_string(),
            name: input.name.trim().to_string(),
            protocol: input.protocol,
            base_url: if base_url.is_empty() {
                input.protocol.default_base_url().to_string()
            } else {
                base_url.to_string()
            },
            api_key: input.api_key,
            model: if model.is_empty() {
                input.protocol.default_model().to_string()
            } else {
                model.to_string()
            },
            enabled: true,
            default_for: Vec::new(),
            headers: input.headers,
            params: LlmParams::default(),
            auth: input.auth,
        };
        Ok(Self {
            profile: normalize_profile(&profile)?,
        })
    }

    fn legacy(
        provider: Option<String>,
        api_key: Option<String>,
        model: Option<String>,
        base_url: Option<String>,
    ) -> AppResult<Self> {
        // Trim the key: it lands verbatim in an auth header (`x-api-key` /
        // `Authorization`). A pasted key routinely carries a trailing newline
        // or space from the clipboard — left in, that either trips reqwest's
        // header-value validation or earns a 401 from the provider. Trim like
        // `base_url` already does so the stored value is normalised at the one
        // chokepoint, independent of any frontend trimming.
        let api_key = api_key
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .ok_or_else(|| AppError::code("noAiKey"))?;
        let protocol = match provider.as_deref() {
            Some("openai") => LlmProtocol::OpenAiChatCompletions,
            _ => LlmProtocol::AnthropicMessages,
        };
        // Likewise trim the model name — it is JSON-serialised into the
        // request body, and a stray space/newline yields a "model not found".
        let model = model
            .map(|m| m.trim().to_string())
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| protocol.default_model().to_string());
        let base_url = base_url
            .map(|u| u.trim().trim_end_matches('/').to_string())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| protocol.default_base_url().to_string());
        Ok(AiConfig {
            profile: LlmProfile {
                id: legacy_profile_id(protocol).to_string(),
                name: legacy_profile_name(protocol).to_string(),
                protocol,
                base_url,
                api_key,
                model,
                enabled: true,
                default_for: Vec::new(),
                headers: HashMap::new(),
                params: LlmParams::default(),
                auth: Some(match protocol {
                    LlmProtocol::AnthropicMessages => LlmAuthMode::XApiKey,
                    LlmProtocol::OpenAiChatCompletions => LlmAuthMode::Bearer,
                }),
            },
        })
    }
}

impl LlmProtocol {
    /// The official API root for this provider, used when the user has not
    /// set a custom base URL.
    fn default_base_url(self) -> &'static str {
        match self {
            LlmProtocol::AnthropicMessages => "https://api.anthropic.com/v1",
            LlmProtocol::OpenAiChatCompletions => "https://api.openai.com/v1",
        }
    }

    fn default_model(self) -> &'static str {
        match self {
            LlmProtocol::AnthropicMessages => "claude-sonnet-4-6",
            LlmProtocol::OpenAiChatCompletions => "gpt-4.1-mini",
        }
    }
}

fn legacy_profile_id(protocol: LlmProtocol) -> &'static str {
    match protocol {
        LlmProtocol::AnthropicMessages => "legacy-anthropic",
        LlmProtocol::OpenAiChatCompletions => "legacy-openai",
    }
}

fn legacy_profile_name(protocol: LlmProtocol) -> &'static str {
    match protocol {
        LlmProtocol::AnthropicMessages => "Legacy Anthropic",
        LlmProtocol::OpenAiChatCompletions => "Legacy OpenAI",
    }
}

fn default_auth_for_protocol(protocol: LlmProtocol) -> LlmAuthMode {
    match protocol {
        LlmProtocol::AnthropicMessages => LlmAuthMode::XApiKey,
        LlmProtocol::OpenAiChatCompletions => LlmAuthMode::Bearer,
    }
}

fn reject_unknown_protocol(value: &Value) -> AppResult<()> {
    let Some(profiles) = value.get("profiles").and_then(Value::as_array) else {
        return Ok(());
    };
    for profile in profiles {
        let Some(protocol) = profile.get("protocol").and_then(Value::as_str) else {
            continue;
        };
        if !matches!(protocol, "anthropic_messages" | "openai_chat_completions") {
            return Err(AppError::code("invalidAiProtocol"));
        }
    }
    Ok(())
}

fn normalize_profile(profile: &LlmProfile) -> AppResult<LlmProfile> {
    let mut profile = profile.clone();
    profile.api_key = profile.api_key.trim().to_string();
    profile.model = profile.model.trim().to_string();
    profile.base_url = profile.base_url.trim().trim_end_matches('/').to_string();
    profile
        .auth
        .get_or_insert_with(|| default_auth_for_protocol(profile.protocol));

    if profile.model.is_empty() || profile.base_url.is_empty() {
        return Err(AppError::code("invalidAiConfig"));
    }
    if !matches!(profile.auth, Some(LlmAuthMode::None)) && profile.api_key.is_empty() {
        return Err(AppError::code("noAiKey"));
    }

    Ok(profile)
}

/// The result of a streamed chat completion.
pub struct ChatOutcome {
    /// The accumulated response text.
    pub text: String,
    /// Whether the stream ran to completion. `false` when the frontend dropped
    /// the channel mid-stream (the user closed the AI panel) — the text is then
    /// a truncated fragment that callers must not persist as a finished result.
    pub completed: bool,
}

/// Stream a single-turn chat completion, forwarding each token to `channel`.
/// Returns the accumulated response text and whether the stream completed.
pub async fn stream_chat(
    client: &Client,
    cfg: &AiConfig,
    system: &str,
    user: &str,
    channel: &Channel<AiEvent>,
    max_tokens: u32,
) -> AppResult<ChatOutcome> {
    let messages = [LlmMessage {
        role: "user".to_string(),
        content: user.to_string(),
    }];
    let result = stream_llm(client, cfg, system, &messages, Some(channel), max_tokens).await;
    match &result {
        Ok(_) => {
            let _ = channel.send(AiEvent::Done);
        }
        Err(e) => {
            let _ = channel.send(AiEvent::Error(e.to_string()));
        }
    }
    result
}

/// Run a completion to the end and return its full text WITHOUT forwarding
/// per-token deltas to the frontend. Translation uses this and reports progress
/// once per batch instead of once per token — token-level IPC over a full
/// article would flood the webview's main thread and freeze the UI.
pub async fn complete_chat(
    client: &Client,
    cfg: &AiConfig,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> AppResult<String> {
    let messages = [LlmMessage {
        role: "user".to_string(),
        content: user.to_string(),
    }];
    let outcome = stream_llm(client, cfg, system, &messages, None, max_tokens).await?;
    Ok(outcome.text)
}

async fn stream_llm(
    client: &Client,
    cfg: &AiConfig,
    system: &str,
    messages: &[LlmMessage],
    channel: Option<&Channel<AiEvent>>,
    max_tokens: u32,
) -> AppResult<ChatOutcome> {
    let req = build_stream_request(client, cfg, system, messages, max_tokens)?;
    let resp = client.execute(req).await?;
    validate_chat_outcome(consume_sse(resp, channel, cfg.profile.protocol).await?)
}

fn validate_chat_outcome(outcome: ChatOutcome) -> AppResult<ChatOutcome> {
    if outcome.completed && outcome.text.trim().is_empty() {
        Err(AppError::other(
            "AI provider returned no displayable content",
        ))
    } else {
        Ok(outcome)
    }
}

fn build_stream_request(
    client: &Client,
    cfg: &AiConfig,
    system: &str,
    messages: &[LlmMessage],
    max_tokens: u32,
) -> AppResult<Request> {
    let timeout = cfg
        .profile
        .params
        .timeout_seconds
        .map(Duration::from_secs)
        .unwrap_or(AI_REQUEST_TIMEOUT);
    let (url, body) = match cfg.profile.protocol {
        LlmProtocol::AnthropicMessages => (
            format!("{}/messages", cfg.profile.base_url),
            json!({
                "model": cfg.profile.model,
                "max_tokens": max_tokens,
                "system": system,
                "stream": true,
                "messages": messages,
            }),
        ),
        LlmProtocol::OpenAiChatCompletions => {
            let mut request_messages = Vec::with_capacity(messages.len() + 1);
            request_messages.push(LlmMessage {
                role: "system".to_string(),
                content: system.to_string(),
            });
            request_messages.extend_from_slice(messages);
            (
                format!("{}/chat/completions", cfg.profile.base_url),
                json!({
                    "model": cfg.profile.model,
                    "max_tokens": max_tokens,
                    "stream": true,
                    "messages": request_messages,
                }),
            )
        }
    };

    let mut req = client.post(url).timeout(timeout).json(&body).build()?;
    for (name, value) in &cfg.profile.headers {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| AppError::other(format!("invalid AI header name: {name}")))?;
        let value = HeaderValue::from_str(value)
            .map_err(|_| AppError::other(format!("invalid AI header value for {name}")))?;
        req.headers_mut().insert(name, value);
    }

    req.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    match cfg
        .profile
        .auth
        .unwrap_or_else(|| default_auth_for_protocol(cfg.profile.protocol))
    {
        LlmAuthMode::Bearer => {
            let value = HeaderValue::from_str(&format!("Bearer {}", cfg.profile.api_key))
                .map_err(|_| AppError::code("noAiKey"))?;
            req.headers_mut().insert(AUTHORIZATION, value);
        }
        LlmAuthMode::XApiKey => {
            let value = HeaderValue::from_str(&cfg.profile.api_key)
                .map_err(|_| AppError::code("noAiKey"))?;
            req.headers_mut().insert("x-api-key", value);
        }
        LlmAuthMode::None => {}
    }
    if cfg.profile.protocol == LlmProtocol::AnthropicMessages {
        req.headers_mut()
            .insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    }

    Ok(req)
}

/// What to do after handling one SSE line.
#[derive(Debug)]
enum LineOutcome {
    /// Line handled; keep consuming the stream.
    Continue,
    /// The frontend dropped the channel — stop and report the result as
    /// interrupted so the caller does not persist a truncated fragment.
    ChannelClosed,
}

/// Process a single SSE line: pull the `data:` payload, surface any provider
/// error, and forward a text delta to `channel` (appending it to `full`).
fn handle_sse_line(
    line: &str,
    protocol: LlmProtocol,
    full: &mut String,
    channel: Option<&Channel<AiEvent>>,
) -> AppResult<LineOutcome> {
    let Some(data) = line.trim().strip_prefix("data:") else {
        return Ok(LineOutcome::Continue);
    };
    let data = data.trim();
    if data.is_empty() || data == "[DONE]" {
        return Ok(LineOutcome::Continue);
    }
    let Ok(value) = serde_json::from_str::<Value>(data) else {
        return Ok(LineOutcome::Continue);
    };
    // Both providers can deliver an error mid-stream after a 200 OK
    // (rate limit, overload, content filter). Surface it instead of
    // ending the generation silently with a truncated summary.
    if let Some(msg) = extract_error(&value, protocol) {
        return Err(AppError::other(format!("AI stream error: {msg}")));
    }
    if let Some(text) = extract_delta(&value, protocol) {
        full.push_str(&text);
        // When a token channel is present, a send failure means the frontend
        // dropped it (the user closed the AI panel). Stop streaming instead of
        // downloading the rest of the response into a void. The silent path
        // (translation) passes `None` and simply accumulates into `full`.
        if let Some(ch) = channel {
            if ch.send(AiEvent::Delta(text)).is_err() {
                log::debug!("AI stream channel closed; aborting early");
                return Ok(LineOutcome::ChannelClosed);
            }
        }
    }
    Ok(LineOutcome::Continue)
}

/// Drive the Server-Sent-Events response, extracting text deltas per provider.
async fn consume_sse(
    resp: reqwest::Response,
    channel: Option<&Channel<AiEvent>>,
    protocol: LlmProtocol,
) -> AppResult<ChatOutcome> {
    let mut resp = ensure_success(resp, "AI API").await?;

    let mut buf: Vec<u8> = Vec::new();
    let mut full = String::new();

    while let Some(chunk) = resp.chunk().await? {
        buf.extend_from_slice(&chunk);
        // Guard against a provider that never delimits its frames: a buffer
        // this large is not a genuine SSE event, so fail instead of growing
        // memory without bound.
        if buf.len() > MAX_SSE_BUFFER {
            return Err(AppError::other(
                "AI stream error: response is not server-sent events",
            ));
        }
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let raw: Vec<u8> = buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&raw);
            match handle_sse_line(&line, protocol, &mut full, channel)? {
                LineOutcome::Continue => {}
                LineOutcome::ChannelClosed => {
                    return Ok(ChatOutcome {
                        text: full,
                        completed: false,
                    });
                }
            }
        }
    }
    // The stream ended. A standards-compliant provider newline-terminates
    // every frame, but a custom OpenAI-compatible endpoint (a local LLM
    // server, which the base-URL override explicitly allows) may close the
    // connection right after the final `data:` line with no trailing newline.
    // Without this, that last frame — carrying the closing token(s) of the
    // response — would be left unprocessed in `buf` and silently dropped.
    if !buf.is_empty() {
        let line = String::from_utf8_lossy(&buf);
        match handle_sse_line(&line, protocol, &mut full, channel)? {
            LineOutcome::Continue => {}
            LineOutcome::ChannelClosed => {
                return Ok(ChatOutcome {
                    text: full,
                    completed: false,
                });
            }
        }
    }
    Ok(ChatOutcome {
        text: full,
        completed: true,
    })
}

/// Detect a provider error object carried inside an SSE data frame.
///
/// For the OpenAI-compatible path the error must be a non-null object: many
/// compatible servers (OpenRouter and others) include a literal `"error": null`
/// alongside `choices` in their *successful* chunks. Treating that as a fault
/// would abort an otherwise-fine generation with a bogus "stream error".
fn extract_error(v: &Value, protocol: LlmProtocol) -> Option<String> {
    let err = match protocol {
        LlmProtocol::AnthropicMessages => (v["type"] == "error").then(|| &v["error"]),
        LlmProtocol::OpenAiChatCompletions => v.get("error").filter(|e| e.is_object()),
    }?;
    Some(
        err["message"]
            .as_str()
            .filter(|m| !m.is_empty())
            .unwrap_or("stream error")
            .to_string(),
    )
}

fn extract_delta(v: &Value, protocol: LlmProtocol) -> Option<String> {
    match protocol {
        LlmProtocol::AnthropicMessages => {
            if v["type"] == "content_block_delta" {
                v["delta"]["text"].as_str().map(String::from)
            } else {
                None
            }
        }
        LlmProtocol::OpenAiChatCompletions => v["choices"][0]["delta"]["content"]
            .as_str()
            .map(String::from),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_stream_request, extract_delta, extract_error, validate_chat_outcome, AiConfig,
        ChatOutcome, LlmMessage, LlmProfileInput, LlmProtocol, LlmPurpose,
    };
    use reqwest::header::AUTHORIZATION;
    use reqwest::Client;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn openai_null_error_field_is_not_an_error() {
        // OpenRouter and other OpenAI-compatible servers ship `"error": null`
        // inside ordinary successful chunks — it must not abort the stream.
        let chunk = json!({
            "choices": [{ "delta": { "content": "hello" } }],
            "error": null,
        });
        assert_eq!(
            extract_error(&chunk, LlmProtocol::OpenAiChatCompletions),
            None
        );
        assert_eq!(
            extract_delta(&chunk, LlmProtocol::OpenAiChatCompletions).as_deref(),
            Some("hello")
        );
    }

    #[test]
    fn completed_empty_chat_is_rejected_without_misreporting_interruption() {
        assert!(validate_chat_outcome(ChatOutcome {
            text: "  ".to_string(),
            completed: true,
        })
        .is_err());

        let completed = validate_chat_outcome(ChatOutcome {
            text: "summary".to_string(),
            completed: true,
        })
        .unwrap();
        assert_eq!(completed.text, "summary");

        let interrupted = validate_chat_outcome(ChatOutcome {
            text: String::new(),
            completed: false,
        })
        .unwrap();
        assert!(!interrupted.completed);
    }

    #[test]
    fn openai_real_error_object_is_surfaced() {
        let chunk = json!({ "error": { "message": "rate limit exceeded" } });
        assert_eq!(
            extract_error(&chunk, LlmProtocol::OpenAiChatCompletions).as_deref(),
            Some("rate limit exceeded")
        );
    }

    #[test]
    fn openai_error_object_without_message_falls_back() {
        let chunk = json!({ "error": { "code": 500 } });
        assert_eq!(
            extract_error(&chunk, LlmProtocol::OpenAiChatCompletions).as_deref(),
            Some("stream error")
        );
    }

    #[test]
    fn openai_plain_delta_chunk_has_no_error() {
        let chunk = json!({ "choices": [{ "delta": { "content": "x" } }] });
        assert_eq!(
            extract_error(&chunk, LlmProtocol::OpenAiChatCompletions),
            None
        );
    }

    #[test]
    fn anthropic_error_event_is_surfaced() {
        let chunk = json!({ "type": "error", "error": { "message": "overloaded" } });
        assert_eq!(
            extract_error(&chunk, LlmProtocol::AnthropicMessages).as_deref(),
            Some("overloaded")
        );
    }

    #[test]
    fn anthropic_content_delta_is_not_an_error() {
        let chunk = json!({
            "type": "content_block_delta",
            "delta": { "type": "text_delta", "text": "world" },
        });
        assert_eq!(extract_error(&chunk, LlmProtocol::AnthropicMessages), None);
        assert_eq!(
            extract_delta(&chunk, LlmProtocol::AnthropicMessages).as_deref(),
            Some("world")
        );
    }

    // --- handle_sse_line: per-line parsing, including the final unterminated
    //     frame a non-compliant endpoint may close the stream on. ---

    use super::{handle_sse_line, AiEvent, LineOutcome};
    use std::sync::{Arc, Mutex};
    use tauri::ipc::{Channel, InvokeResponseBody};

    /// A `Channel<AiEvent>` whose every sent delta is recorded into the
    /// returned buffer — lets the line parser be exercised without a webview.
    fn recording_channel() -> (Channel<AiEvent>, Arc<Mutex<Vec<String>>>) {
        let received = Arc::new(Mutex::new(Vec::new()));
        let sink = received.clone();
        let channel = Channel::new(move |body: InvokeResponseBody| {
            let json = match body {
                InvokeResponseBody::Json(s) => s,
                InvokeResponseBody::Raw(b) => String::from_utf8_lossy(&b).into_owned(),
            };
            let v: serde_json::Value = serde_json::from_str(&json).unwrap();
            if v["type"] == "delta" {
                sink.lock()
                    .unwrap()
                    .push(v["data"].as_str().unwrap().to_string());
            }
            Ok(())
        });
        (channel, received)
    }

    #[test]
    fn sse_line_forwards_a_delta() {
        let (channel, got) = recording_channel();
        let mut full = String::new();
        let line = "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n";
        let out = handle_sse_line(
            line,
            LlmProtocol::OpenAiChatCompletions,
            &mut full,
            Some(&channel),
        )
        .unwrap();
        assert!(matches!(out, LineOutcome::Continue));
        assert_eq!(full, "hi");
        assert_eq!(*got.lock().unwrap(), vec!["hi"]);
    }

    #[test]
    fn sse_line_ignores_non_data_and_done_lines() {
        let (channel, got) = recording_channel();
        let mut full = String::new();
        for line in [": keep-alive comment\n", "data: [DONE]\n", "\n"] {
            handle_sse_line(
                line,
                LlmProtocol::OpenAiChatCompletions,
                &mut full,
                Some(&channel),
            )
            .unwrap();
        }
        assert!(full.is_empty());
        assert!(got.lock().unwrap().is_empty());
    }

    #[test]
    fn sse_line_surfaces_a_mid_stream_error() {
        let (channel, _got) = recording_channel();
        let mut full = String::new();
        let line = "data: {\"error\":{\"message\":\"rate limited\"}}\n";
        let err = handle_sse_line(
            line,
            LlmProtocol::OpenAiChatCompletions,
            &mut full,
            Some(&channel),
        )
        .unwrap_err();
        assert!(err.to_string().contains("rate limited"));
    }

    #[test]
    fn sse_final_frame_without_trailing_newline_is_not_dropped() {
        // A custom OpenAI-compatible endpoint may close the connection right
        // after the last `data:` line with no trailing `\n`. The closing
        // token must still be parsed — `handle_sse_line` is fed the leftover
        // buffer verbatim, exactly as `consume_sse` does after the read loop.
        let (channel, got) = recording_channel();
        let mut full = String::new();
        let last = "data: {\"choices\":[{\"delta\":{\"content\":\"!\"}}]}";
        handle_sse_line(
            last,
            LlmProtocol::OpenAiChatCompletions,
            &mut full,
            Some(&channel),
        )
        .unwrap();
        assert_eq!(full, "!");
        assert_eq!(*got.lock().unwrap(), vec!["!"]);
    }

    #[test]
    fn openai_chunk_without_choices_is_ignored() {
        let (channel, got) = recording_channel();
        let mut full = String::new();
        let line = "data: {\"id\":\"chunk-without-choices\"}\n";
        let out = handle_sse_line(
            line,
            LlmProtocol::OpenAiChatCompletions,
            &mut full,
            Some(&channel),
        )
        .unwrap();

        assert!(matches!(out, LineOutcome::Continue));
        assert!(full.is_empty());
        assert!(got.lock().unwrap().is_empty());
    }

    // --- AiConfig::new: normalising pasted credentials. ---

    #[test]
    fn legacy_anthropic_settings_resolve_to_anthropic_messages() {
        let cfg = AiConfig::new(Some("anthropic".into()), Some("sk-key".into()), None, None)
            .expect("legacy Anthropic settings should resolve");
        assert_eq!(cfg.profile.protocol, LlmProtocol::AnthropicMessages);
    }

    #[test]
    fn legacy_openai_settings_resolve_to_openai_chat_completions() {
        let cfg = AiConfig::new(Some("openai".into()), Some("sk-key".into()), None, None)
            .expect("legacy OpenAI settings should resolve");
        assert_eq!(cfg.profile.protocol, LlmProtocol::OpenAiChatCompletions);
    }

    #[test]
    fn config_trims_api_key_model_and_base_url() {
        // A key copied from a webpage commonly carries a trailing newline /
        // spaces; left in, it breaks the auth header.
        let cfg = AiConfig::new(
            Some("openai".into()),
            Some("  sk-abc123\n".into()),
            Some(" gpt-4.1-mini\n".into()),
            Some(" https://example.test/v1/// ".into()),
        )
        .expect("a key with surrounding whitespace is still a usable key");
        assert_eq!(cfg.profile.api_key, "sk-abc123");
        assert_eq!(cfg.profile.model, "gpt-4.1-mini");
        assert_eq!(cfg.profile.base_url, "https://example.test/v1");
    }

    #[test]
    fn config_rejects_a_whitespace_only_api_key() {
        // Trimmed to empty — treated as "no key set", not a usable credential.
        // `AiConfig` deliberately holds no `Debug` impl (it carries a secret),
        // so match the result rather than `unwrap_err`.
        match AiConfig::new(Some("openai".into()), Some("   \n".into()), None, None) {
            Ok(_) => panic!("a whitespace-only key must not be accepted"),
            Err(e) => assert!(e.to_string().contains("noAiKey")),
        }
    }

    #[test]
    fn config_falls_back_to_the_default_model_for_a_blank_one() {
        // A whitespace-only model name trims to empty and must yield the
        // provider default, not an empty string in the request body.
        let cfg = AiConfig::new(
            Some("openai".into()),
            Some("sk-key".into()),
            Some("  ".into()),
            None,
        )
        .unwrap();
        assert_eq!(cfg.profile.model, "gpt-4.1-mini");
    }

    #[test]
    fn unknown_protocol_in_profiles_json_returns_invalid_ai_protocol() {
        let json = r#"{
            "version": 1,
            "profiles": [{
                "id": "bad",
                "name": "Bad",
                "protocol": "made_up",
                "base_url": "https://example.test/v1",
                "api_key": "sk-key",
                "model": "model",
                "enabled": true,
                "auth": "bearer"
            }]
        }"#;

        match AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            Some("legacy-key".into()),
            None,
            None,
            LlmPurpose::Ask,
        ) {
            Ok(_) => panic!("unknown protocol must be rejected"),
            Err(e) => assert!(e.to_string().contains("invalidAiProtocol")),
        }
    }

    #[test]
    fn malformed_profiles_json_returns_invalid_ai_config() {
        match AiConfig::from_settings(
            Some("{ not valid json".into()),
            None,
            None,
            None,
            Some("legacy-key".into()),
            None,
            None,
            LlmPurpose::Ask,
        ) {
            Ok(_) => panic!("malformed profile JSON must be rejected"),
            Err(e) => assert!(e.to_string().contains("invalidAiConfig")),
        }
    }

    #[test]
    fn stored_auth_none_profile_does_not_require_api_key_or_send_auth_headers() {
        let client = Client::new();
        let json = r#"{
            "version": 1,
            "profiles": [{
                "id": "local",
                "name": "Local",
                "protocol": "openai_chat_completions",
                "base_url": "http://localhost:11434/v1",
                "api_key": "   ",
                "model": "local-model",
                "enabled": true,
                "auth": "none"
            }]
        }"#;
        let cfg = AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            None,
            None,
            None,
            LlmPurpose::Ask,
        )
        .unwrap();
        let req = build_stream_request(
            &client,
            &cfg,
            "sys",
            &[LlmMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            32,
        )
        .unwrap();

        assert!(req.headers().get(AUTHORIZATION).is_none());
        assert!(req.headers().get("x-api-key").is_none());
    }

    #[test]
    fn stored_bearer_profile_with_whitespace_only_api_key_returns_no_ai_key() {
        let json = r#"{
            "version": 1,
            "profiles": [{
                "id": "bad-key",
                "name": "Bad Key",
                "protocol": "openai_chat_completions",
                "base_url": "https://openai.test/v1",
                "api_key": "   ",
                "model": "gpt-test",
                "enabled": true,
                "auth": "bearer"
            }]
        }"#;

        match AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            None,
            None,
            None,
            LlmPurpose::Ask,
        ) {
            Ok(_) => panic!("whitespace-only bearer key must be rejected"),
            Err(e) => assert!(e.to_string().contains("noAiKey")),
        }
    }

    #[test]
    fn stored_x_api_key_profile_with_whitespace_only_api_key_returns_no_ai_key() {
        let json = r#"{
            "version": 1,
            "profiles": [{
                "id": "bad-key",
                "name": "Bad Key",
                "protocol": "anthropic_messages",
                "base_url": "https://anthropic.test/v1",
                "api_key": "   ",
                "model": "claude-test",
                "enabled": true,
                "auth": "x_api_key"
            }]
        }"#;

        match AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            None,
            None,
            None,
            LlmPurpose::Ask,
        ) {
            Ok(_) => panic!("whitespace-only x-api-key must be rejected"),
            Err(e) => assert!(e.to_string().contains("noAiKey")),
        }
    }

    #[test]
    fn disabled_profile_is_skipped_and_later_enabled_profile_can_be_selected() {
        let json = r#"{
            "version": 1,
            "active_profile_id": "disabled",
            "profiles": [
                {
                    "id": "disabled",
                    "name": "Disabled",
                    "protocol": "openai_chat_completions",
                    "base_url": "https://disabled.test/v1",
                    "api_key": "sk-disabled",
                    "model": "disabled-model",
                    "enabled": false,
                    "auth": "bearer"
                },
                {
                    "id": "enabled",
                    "name": "Enabled",
                    "protocol": "anthropic_messages",
                    "base_url": "https://enabled.test/v1",
                    "api_key": "sk-enabled",
                    "model": "enabled-model",
                    "enabled": true,
                    "auth": "x_api_key"
                }
            ]
        }"#;

        let cfg = AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            Some("legacy-key".into()),
            None,
            None,
            LlmPurpose::Ask,
        )
        .unwrap();

        assert_eq!(cfg.profile.id, "enabled");
        assert_eq!(cfg.profile.protocol, LlmProtocol::AnthropicMessages);
    }

    #[test]
    fn missing_purpose_profile_falls_back_to_active_profile() {
        let json = r#"{
            "version": 1,
            "profiles": [
                {
                    "id": "first",
                    "name": "First",
                    "protocol": "anthropic_messages",
                    "base_url": "https://first.test/v1",
                    "api_key": "sk-first",
                    "model": "first-model",
                    "enabled": true,
                    "auth": "x_api_key"
                },
                {
                    "id": "active",
                    "name": "Active",
                    "protocol": "openai_chat_completions",
                    "base_url": "https://active.test/v1",
                    "api_key": "sk-active",
                    "model": "active-model",
                    "enabled": true,
                    "auth": "bearer"
                }
            ]
        }"#;

        let cfg = AiConfig::from_settings(
            Some(json.into()),
            Some("active".into()),
            Some("missing-purpose".into()),
            None,
            Some("legacy-key".into()),
            None,
            None,
            LlmPurpose::Summary,
        )
        .unwrap();

        assert_eq!(cfg.profile.id, "active");
        assert_eq!(cfg.profile.protocol, LlmProtocol::OpenAiChatCompletions);
    }

    #[test]
    fn anthropic_adapter_request_has_expected_url_headers_and_body() {
        let client = Client::new();
        let cfg = AiConfig::new(
            Some("anthropic".into()),
            Some(" sk-ant ".into()),
            Some(" claude-test ".into()),
            Some(" https://anthropic.test/v1/ ".into()),
        )
        .unwrap();
        let req = build_stream_request(
            &client,
            &cfg,
            "sys",
            &[LlmMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            77,
        )
        .unwrap();

        assert_eq!(req.method(), reqwest::Method::POST);
        assert_eq!(req.url().as_str(), "https://anthropic.test/v1/messages");
        assert_eq!(req.headers()["x-api-key"], "sk-ant");
        assert_eq!(req.headers()["anthropic-version"], "2023-06-01");
        assert_eq!(req.headers()["content-type"], "application/json");

        let body: serde_json::Value =
            serde_json::from_slice(req.body().unwrap().as_bytes().unwrap()).unwrap();
        assert_eq!(body["model"], "claude-test");
        assert_eq!(body["max_tokens"], 77);
        assert_eq!(body["system"], "sys");
        assert_eq!(body["stream"], true);
        assert_eq!(
            body["messages"],
            json!([{ "role": "user", "content": "hello" }])
        );
    }

    #[test]
    fn openai_adapter_request_has_expected_url_headers_body_and_message_order() {
        let client = Client::new();
        let cfg = AiConfig::new(
            Some("openai".into()),
            Some(" sk-openai ".into()),
            Some(" gpt-test ".into()),
            Some(" https://openai.test/v1/ ".into()),
        )
        .unwrap();
        let req = build_stream_request(
            &client,
            &cfg,
            "sys",
            &[
                LlmMessage {
                    role: "user".into(),
                    content: "first".into(),
                },
                LlmMessage {
                    role: "assistant".into(),
                    content: "second".into(),
                },
            ],
            88,
        )
        .unwrap();

        assert_eq!(req.method(), reqwest::Method::POST);
        assert_eq!(
            req.url().as_str(),
            "https://openai.test/v1/chat/completions"
        );
        assert_eq!(req.headers()["authorization"], "Bearer sk-openai");
        assert_eq!(req.headers()["content-type"], "application/json");

        let body: serde_json::Value =
            serde_json::from_slice(req.body().unwrap().as_bytes().unwrap()).unwrap();
        assert_eq!(body["model"], "gpt-test");
        assert_eq!(body["max_tokens"], 88);
        assert_eq!(body["stream"], true);
        assert_eq!(
            body["messages"],
            json!([
                { "role": "system", "content": "sys" },
                { "role": "user", "content": "first" },
                { "role": "assistant", "content": "second" }
            ])
        );
    }

    #[test]
    fn profile_input_builds_openai_compatible_connection_test_request() {
        let client = Client::new();
        let cfg = AiConfig::from_profile_input(LlmProfileInput {
            name: "Agnes".into(),
            protocol: LlmProtocol::OpenAiChatCompletions,
            base_url: " https://apihub.agnes-ai.com/v1/ ".into(),
            api_key: " sk-test ".into(),
            model: " agnes-2.0-flash ".into(),
            auth: None,
            headers: HashMap::new(),
        })
        .unwrap();
        let req = build_stream_request(
            &client,
            &cfg,
            "sys",
            &[LlmMessage {
                role: "user".into(),
                content: "ping".into(),
            }],
            8,
        )
        .unwrap();

        assert_eq!(
            req.url().as_str(),
            "https://apihub.agnes-ai.com/v1/chat/completions"
        );
        assert_eq!(req.headers()["authorization"], "Bearer sk-test");
        let body: serde_json::Value =
            serde_json::from_slice(req.body().unwrap().as_bytes().unwrap()).unwrap();
        assert_eq!(body["model"], "agnes-2.0-flash");
    }

    #[test]
    fn custom_headers_are_included_without_removing_required_headers() {
        let client = Client::new();
        let json = r#"{
            "version": 1,
            "profiles": [{
                "id": "custom",
                "name": "Custom",
                "protocol": "openai_chat_completions",
                "base_url": "https://custom.test/v1",
                "api_key": "sk-custom",
                "model": "custom-model",
                "enabled": true,
                "auth": "bearer",
                "headers": {
                    "x-extra": "trace-1",
                    "authorization": "Bearer wrong"
                }
            }]
        }"#;
        let cfg = AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            None,
            None,
            None,
            LlmPurpose::Ask,
        )
        .unwrap();
        let req = build_stream_request(
            &client,
            &cfg,
            "sys",
            &[LlmMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            32,
        )
        .unwrap();

        assert_eq!(req.headers()["x-extra"], "trace-1");
        assert_eq!(req.headers()["authorization"], "Bearer sk-custom");
    }

    #[test]
    fn stored_anthropic_profile_without_auth_defaults_to_x_api_key() {
        let client = Client::new();
        let json = r#"{
            "version": 1,
            "profiles": [{
                "id": "anthropic",
                "name": "Anthropic",
                "protocol": "anthropic_messages",
                "base_url": "https://anthropic.test/v1",
                "api_key": "sk-anthropic",
                "model": "claude-test",
                "enabled": true
            }]
        }"#;
        let cfg = AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            None,
            None,
            None,
            LlmPurpose::Ask,
        )
        .unwrap();
        let req = build_stream_request(
            &client,
            &cfg,
            "sys",
            &[LlmMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            32,
        )
        .unwrap();

        assert_eq!(req.headers()["x-api-key"], "sk-anthropic");
        assert!(req.headers().get(AUTHORIZATION).is_none());
    }

    #[test]
    fn stored_openai_profile_without_auth_defaults_to_bearer() {
        let client = Client::new();
        let json = r#"{
            "version": 1,
            "profiles": [{
                "id": "openai",
                "name": "OpenAI",
                "protocol": "openai_chat_completions",
                "base_url": "https://openai.test/v1",
                "api_key": "sk-openai",
                "model": "gpt-test",
                "enabled": true
            }]
        }"#;
        let cfg = AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            None,
            None,
            None,
            LlmPurpose::Ask,
        )
        .unwrap();
        let req = build_stream_request(
            &client,
            &cfg,
            "sys",
            &[LlmMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            32,
        )
        .unwrap();

        assert_eq!(req.headers()["authorization"], "Bearer sk-openai");
    }

    #[test]
    fn profile_params_max_tokens_does_not_override_explicit_request_cap() {
        let client = Client::new();
        let json = r#"{
            "version": 1,
            "profiles": [{
                "id": "tokens",
                "name": "Tokens",
                "protocol": "openai_chat_completions",
                "base_url": "https://openai.test/v1",
                "api_key": "sk-openai",
                "model": "gpt-test",
                "enabled": true,
                "auth": "bearer",
                "params": {
                    "max_tokens": 999
                }
            }]
        }"#;
        let cfg = AiConfig::from_settings(
            Some(json.into()),
            None,
            None,
            None,
            None,
            None,
            None,
            LlmPurpose::Ask,
        )
        .unwrap();
        let req = build_stream_request(
            &client,
            &cfg,
            "sys",
            &[LlmMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            32,
        )
        .unwrap();
        let body: serde_json::Value =
            serde_json::from_slice(req.body().unwrap().as_bytes().unwrap()).unwrap();

        assert_eq!(body["max_tokens"], 32);
    }
}
