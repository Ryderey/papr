//! Platform-neutral AI contracts and prompt construction.
//!
//! Provider credentials are deliberately separate from serializable profile
//! metadata. Android adapters resolve a Keystore alias only for the lifetime of
//! one request and pass the resulting credential to the HTTP layer.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{Client, Request, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Notify;
use url::Url;

use crate::dto::SummaryTemplate;
use crate::error::{CoreError, ErrorCategory};

pub const SUMMARY_INPUT_CHAR_LIMIT: usize = 8_000;
// Reasoning-capable OpenAI-compatible models count hidden reasoning toward this
// limit. Keep enough room for the visible summary or follow-up response.
pub const AI_MAX_OUTPUT_TOKENS: u32 = 2_000;
const AI_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_SSE_BUFFER: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProtocol {
    AnthropicMessages,
    OpenaiChatCompletions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiAuthMode {
    Bearer,
    XApiKey,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiPurpose {
    Summary,
    Translate,
}

/// Persistable AI provider metadata. It intentionally has no credential field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiProfile {
    pub id: String,
    pub name: String,
    pub protocol: AiProtocol,
    pub model: String,
    pub base_url: String,
    pub auth: AiAuthMode,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    pub credential_ref: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub default_for: Vec<AiPurpose>,
}

impl AiProfile {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() || self.model.trim().is_empty()
        {
            return Err(invalid_profile());
        }

        let url = Url::parse(self.base_url.trim()).map_err(|_| invalid_profile())?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(invalid_profile());
        }

        if self.auth != AiAuthMode::None
            && self
                .credential_ref
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
        {
            return Err(CoreError::coded(ErrorCategory::Ai, "noAiCredential", None));
        }

        for (name, value) in &self.headers {
            let header = HeaderName::from_bytes(name.as_bytes()).map_err(|_| invalid_profile())?;
            HeaderValue::from_str(value).map_err(|_| invalid_profile())?;
            if matches!(
                header.as_str(),
                "authorization" | "proxy-authorization" | "x-api-key" | "api-key" | "cookie"
            ) {
                return Err(invalid_profile());
            }
        }

        Ok(())
    }
}

/// A transient secret resolved from platform secure storage for one request.
/// This type is neither serializable nor printable.
pub struct ResolvedAiCredential(String);

impl ResolvedAiCredential {
    pub fn new(secret: String) -> Result<Self, CoreError> {
        let secret = secret.trim().to_string();
        if secret.is_empty() {
            return Err(CoreError::coded(ErrorCategory::Ai, "noAiCredential", None));
        }
        Ok(Self(secret))
    }

    pub(crate) fn secret(&self) -> &str {
        &self.0
    }
}

/// Cooperative cancellation shared by an adapter and one Core request.
#[derive(Clone, Default)]
pub struct AiCancellation {
    cancelled: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl AiCancellation {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        let notified = self.notify.notified();
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }
}

/// Events emitted by a summary or follow-up stream. Translation adds progress
/// events through the same cross-layer envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiStreamEvent {
    Delta {
        request_id: String,
        text: String,
    },
    Progress {
        request_id: String,
        completed: u32,
        total: u32,
    },
    Completed {
        request_id: String,
    },
    Error {
        request_id: String,
        code: String,
    },
}

fn default_enabled() -> bool {
    true
}

fn invalid_profile() -> CoreError {
    CoreError::coded(ErrorCategory::InvalidInput, "invalidAiProfile", None)
}

/// Normalize the UI language to the three response languages currently
/// supported by the app.
pub fn response_language_code(language: &str) -> &'static str {
    match language
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "zh" => "zh",
        "ja" => "ja",
        _ => "en",
    }
}

fn response_language_directive(language: &str) -> &'static str {
    match response_language_code(language) {
        "zh" => "\n\nAlways write your response in Simplified Chinese.",
        "ja" => "\n\nAlways write your response in Japanese.",
        _ => "\n\nAlways write your response in English.",
    }
}

/// Build the system prompt for one of the six desktop-compatible templates.
pub fn build_summary_prompt(template: SummaryTemplate, language: &str) -> String {
    let lang = response_language_directive(language);
    let prompt = match template {
        SummaryTemplate::Classic => {
            "You are a sharp news editor. Summarize the article so a reader can decide whether to read it in full.\n\nFormat the response in markdown using exactly this shape:\n**TL;DR** — One sentence capturing the single most important point.\n\n- Key fact, finding, or claim (under ~20 words)\n- Another key point\n- 3 to 5 bullets total, one idea each, no nested bullets\n\nOutput only this structure. No preamble, no closing remarks, no section headers, no extra prose."
        }
        SummaryTemplate::News5w1h => {
            "You are a senior news editor. Produce a structured summary that extracts the core facts.\n\nUse exactly this markdown shape:\n**一句话概述** — One sentence summarizing the core subject.\n\n**5W1H 速览**\n- **Who（谁）**：People, companies, or organizations involved\n- **What（什么）**：The event, product, or finding\n- **When（何时）**：Time, date, or version cycle\n- **Where（何地）**：Location, platform, or scope\n- **Why（为什么）**：Motivation, background, or cause\n- **How（如何）**：Method, approach, or impact path\n\n**关键细节**：1–3 essential facts or data points.\n\n**价值判断**：Is the article worth reading in full? One sentence.\n\nOutput only this structure. No preamble or extra prose."
        }
        SummaryTemplate::Decision => {
            "You are an efficient information-filtering assistant. Help the reader decide in 5 seconds whether the article is worth reading.\n\nUse exactly this markdown shape:\n**核心命题** — What problem or viewpoint does the article address?\n\n**适合谁读** — Which reader will find this most valuable?\n\n**为什么现在读** — What is timely or uniquely valuable?\n\n**值不值得读** — High quality, worth a careful read / Clickbait, the summary is enough / Useful only for a specific audience\n\n**如果只看一句话**：The single most memorable conclusion.\n\nOutput only this structure. No extra prose."
        }
        SummaryTemplate::Funnel => {
            "You are an information architect. Compress the article into a three-level progressive summary.\n\nUse exactly this markdown shape:\n**第一层：30 秒速览** (≤ 30 words)\nOne extremely short sentence stating the core conclusion.\n\n**第二层：2 分钟精华** (3–4 bullets)\nThe most important evidence or findings, each ≤ 25 words.\n\n**第三层：深度线索** (1–2 items)\nWhich sections deserve deeper reading?\n\n**适合场景**：When is this article best read?\n\nOutput only this structure. No extra prose."
        }
        SummaryTemplate::Argument => {
            "You are a logic analyst. Deconstruct the article's core argument.\n\nUse exactly this markdown shape:\n**作者的核心观点** — The one thing the author most wants the reader to believe.\n\n**主要论据** — 2–4 key pieces of evidence or reasoning.\n\n**潜在前提** — What unstated assumptions does the argument rely on?\n\n**不同视角** — What would a skeptic question?\n\n**我的判断** — Is the argument solid and worth trusting?\n\nOutput only this structure. No extra prose."
        }
        SummaryTemplate::Minimal => {
            "You are an ultra-concise summary editor. Reduce the article to a single sentence, at most 50 words.\n\nRequirements:\n- Include the core conclusion or key fact\n- Do not start with 'This article discusses...' or similar empty phrases\n- Deliver the substance directly\n\nIf the article is extremely short or lacks substance, output exactly: 「内容较浅，建议跳过。」\n\nOutput only that one sentence."
        }
    };
    format!("{prompt}{lang}")
}

/// Build the bounded article message used by summary requests.
pub fn build_summary_user_message(title: &str, body: &str) -> Result<String, CoreError> {
    if body.trim().is_empty() {
        return Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            "noArticleBody",
            None,
        ));
    }
    let body: String = body.chars().take(SUMMARY_INPUT_CHAR_LIMIT).collect();
    Ok(format!("Title: {}\n\n{body}", title.trim()))
}

pub fn build_follow_up_system_prompt(language: &str) -> String {
    format!(
        "You are a helpful reading assistant. Answer using only the information present in the supplied AI-generated summary and completed Q&A history. If the question goes beyond that information, say so plainly. Keep answers concise.{}",
        response_language_directive(language)
    )
}

/// Build a follow-up message from the summary and in-memory conversation only.
pub fn build_follow_up_user_message(
    summary: &str,
    history: &[(String, String)],
    question: &str,
) -> Result<String, CoreError> {
    let question = question.trim();
    if question.is_empty() {
        return Err(CoreError::coded(
            ErrorCategory::InvalidInput,
            "emptyAiQuestion",
            None,
        ));
    }

    let mut user = format!("Summary:\n\n{}\n\n---\n\n", summary.trim());
    for (question, answer) in history {
        user.push_str("Q: ");
        user.push_str(question.trim());
        user.push_str("\nA: ");
        user.push_str(answer.trim());
        user.push_str("\n\n");
    }
    user.push_str("Q: ");
    user.push_str(question);
    user.push_str("\nA:");
    Ok(user)
}

/// Stream one provider-neutral chat request. Provider response bodies and
/// credentials never enter returned errors or events.
pub async fn stream_chat<F>(
    client: &Client,
    profile: &AiProfile,
    credential: Option<&ResolvedAiCredential>,
    request_id: &str,
    system: &str,
    user: &str,
    cancellation: &AiCancellation,
    mut emit: F,
) -> Result<String, CoreError>
where
    F: FnMut(AiStreamEvent) -> bool,
{
    let result = stream_chat_inner(
        client,
        profile,
        credential,
        request_id,
        system,
        user,
        cancellation,
        &mut emit,
    )
    .await;

    match &result {
        Ok(_) => {
            emit(AiStreamEvent::Completed {
                request_id: request_id.to_string(),
            });
        }
        Err(error) => {
            emit(AiStreamEvent::Error {
                request_id: request_id.to_string(),
                code: error.code().to_string(),
            });
        }
    }
    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn stream_chat_inner<F>(
    client: &Client,
    profile: &AiProfile,
    credential: Option<&ResolvedAiCredential>,
    request_id: &str,
    system: &str,
    user: &str,
    cancellation: &AiCancellation,
    emit: &mut F,
) -> Result<String, CoreError>
where
    F: FnMut(AiStreamEvent) -> bool,
{
    if cancellation.is_cancelled() {
        return Err(ai_error("aiCancelled"));
    }

    let request = build_stream_request(
        client,
        profile,
        credential,
        system,
        user,
        AI_MAX_OUTPUT_TOKENS,
    )?;
    let mut response = tokio::select! {
        response = client.execute(request) => response.map_err(|_| ai_error("aiNetwork"))?,
        _ = cancellation.cancelled() => return Err(ai_error("aiCancelled")),
    };
    if !response.status().is_success() {
        return Err(http_status_error(response.status()));
    }

    let mut decoder = SseDecoder::new(profile.protocol);
    let mut full = String::new();
    loop {
        let chunk = tokio::select! {
            chunk = response.chunk() => chunk.map_err(|_| ai_error("aiNetwork"))?,
            _ = cancellation.cancelled() => return Err(ai_error("aiCancelled")),
        };
        let Some(chunk) = chunk else {
            break;
        };
        for text in decoder.push(&chunk)? {
            full.push_str(&text);
            if !emit(AiStreamEvent::Delta {
                request_id: request_id.to_string(),
                text,
            }) {
                cancellation.cancel();
                return Err(ai_error("aiCancelled"));
            }
        }
    }
    for text in decoder.finish()? {
        full.push_str(&text);
        if !emit(AiStreamEvent::Delta {
            request_id: request_id.to_string(),
            text,
        }) {
            cancellation.cancel();
            return Err(ai_error("aiCancelled"));
        }
    }
    if full.trim().is_empty() {
        return Err(ai_error("aiParse"));
    }
    Ok(full)
}

fn build_stream_request(
    client: &Client,
    profile: &AiProfile,
    credential: Option<&ResolvedAiCredential>,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<Request, CoreError> {
    profile.validate()?;
    let base_url = profile.base_url.trim().trim_end_matches('/');
    let (url, body) = match profile.protocol {
        AiProtocol::AnthropicMessages => (
            format!("{base_url}/messages"),
            json!({
                "model": profile.model.trim(),
                "max_tokens": max_tokens,
                "system": system,
                "stream": true,
                "messages": [{"role": "user", "content": user}],
            }),
        ),
        AiProtocol::OpenaiChatCompletions => (
            format!("{base_url}/chat/completions"),
            json!({
                "model": profile.model.trim(),
                "max_tokens": max_tokens,
                "stream": true,
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": user},
                ],
            }),
        ),
    };

    let mut request = client
        .post(url)
        .timeout(AI_REQUEST_TIMEOUT)
        .json(&body)
        .build()
        .map_err(|_| invalid_profile())?;
    request
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    for (name, value) in &profile.headers {
        let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| invalid_profile())?;
        let value = HeaderValue::from_str(value).map_err(|_| invalid_profile())?;
        request.headers_mut().insert(name, value);
    }

    match profile.auth {
        AiAuthMode::Bearer => {
            let secret = credential
                .ok_or_else(|| ai_error("noAiCredential"))?
                .secret();
            let value = HeaderValue::from_str(&format!("Bearer {secret}"))
                .map_err(|_| ai_error("noAiCredential"))?;
            request.headers_mut().insert(AUTHORIZATION, value);
        }
        AiAuthMode::XApiKey => {
            let secret = credential
                .ok_or_else(|| ai_error("noAiCredential"))?
                .secret();
            let value = HeaderValue::from_str(secret).map_err(|_| ai_error("noAiCredential"))?;
            request.headers_mut().insert("x-api-key", value);
        }
        AiAuthMode::None => {}
    }
    if profile.protocol == AiProtocol::AnthropicMessages {
        request
            .headers_mut()
            .insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
    }
    Ok(request)
}

fn ai_error(code: &'static str) -> CoreError {
    CoreError::coded(ErrorCategory::Ai, code, None)
}

fn http_status_error(status: StatusCode) -> CoreError {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => ai_error("aiAuth"),
        StatusCode::TOO_MANY_REQUESTS => ai_error("aiRateLimited"),
        _ => ai_error("aiNetwork"),
    }
}

struct SseDecoder {
    protocol: AiProtocol,
    buffer: Vec<u8>,
}

impl SseDecoder {
    fn new(protocol: AiProtocol) -> Self {
        Self {
            protocol,
            buffer: Vec::new(),
        }
    }

    fn push(&mut self, chunk: &[u8]) -> Result<Vec<String>, CoreError> {
        self.buffer.extend_from_slice(chunk);
        if self.buffer.len() > MAX_SSE_BUFFER {
            return Err(ai_error("aiParse"));
        }

        let mut deltas = Vec::new();
        while let Some(position) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.buffer.drain(..=position).collect();
            if let Some(delta) = parse_sse_line(&line, self.protocol)? {
                deltas.push(delta);
            }
        }
        Ok(deltas)
    }

    fn finish(self) -> Result<Vec<String>, CoreError> {
        if self.buffer.is_empty() {
            return Ok(Vec::new());
        }
        Ok(parse_sse_line(&self.buffer, self.protocol)?
            .into_iter()
            .collect())
    }
}

fn parse_sse_line(line: &[u8], protocol: AiProtocol) -> Result<Option<String>, CoreError> {
    let line = std::str::from_utf8(line).map_err(|_| ai_error("aiParse"))?;
    let line = line.trim();
    let data = if let Some(data) = line.strip_prefix("data:") {
        data.trim()
    } else if line.starts_with('{') {
        // Some OpenAI-compatible providers return one completed JSON response
        // even when `stream: true` is requested. Treat it as a single delta.
        line
    } else {
        return Ok(None);
    };
    if data.is_empty() || data == "[DONE]" {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(data).map_err(|_| ai_error("aiParse"))?;
    if let Some(code) = provider_error_code(&value, protocol) {
        return Err(ai_error(code));
    }
    Ok(extract_delta(&value, protocol))
}

fn extract_delta(value: &Value, protocol: AiProtocol) -> Option<String> {
    match protocol {
        AiProtocol::AnthropicMessages if value["type"] == "content_block_delta" => {
            value["delta"]["text"].as_str().map(ToOwned::to_owned)
        }
        AiProtocol::AnthropicMessages => None,
        AiProtocol::OpenaiChatCompletions => value["choices"][0]["delta"]["content"]
            .as_str()
            .or_else(|| value["data"]["choices"][0]["delta"].as_str())
            .or_else(|| value["data"]["choices"][0]["delta"]["content"].as_str())
            .or_else(|| value["choices"][0]["message"].as_str())
            .or_else(|| value["choices"][0]["message"]["content"].as_str())
            .or_else(|| value["data"]["choices"][0]["message"].as_str())
            .or_else(|| value["data"]["choices"][0]["message"]["content"].as_str())
            .map(ToOwned::to_owned),
    }
}

fn provider_error_code(value: &Value, protocol: AiProtocol) -> Option<&'static str> {
    let error = match protocol {
        AiProtocol::AnthropicMessages => (value["type"] == "error").then(|| &value["error"]),
        AiProtocol::OpenaiChatCompletions => value.get("error").filter(|error| error.is_object()),
    }?;
    let text = format!("{} {}", error["type"], error["code"]).to_ascii_lowercase();
    if text.contains("rate") || text.contains("429") {
        Some("aiRateLimited")
    } else if text.contains("auth") || text.contains("api_key") || text.contains("401") {
        Some("aiAuth")
    } else {
        Some("aiNetwork")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> AiProfile {
        AiProfile {
            id: "summary-main".into(),
            name: "Summary".into(),
            protocol: AiProtocol::OpenaiChatCompletions,
            model: "gpt-4.1-mini".into(),
            base_url: "https://api.openai.com/v1".into(),
            auth: AiAuthMode::Bearer,
            headers: BTreeMap::new(),
            credential_ref: Some("papr.ai.summary-main".into()),
            enabled: true,
            default_for: vec![AiPurpose::Summary],
        }
    }

    #[test]
    fn serializable_profile_never_contains_a_secret_field() {
        let json = serde_json::to_string(&profile()).unwrap();
        assert!(!json.contains("api_key"));
        assert!(!json.contains("secret"));
        assert!(json.contains("credential_ref"));
    }

    #[test]
    fn profile_rejects_missing_credentials_and_sensitive_stored_headers() {
        let mut value = profile();
        value.credential_ref = None;
        assert_eq!(value.validate().unwrap_err().code(), "noAiCredential");

        value.credential_ref = Some("alias".into());
        value
            .headers
            .insert("Authorization".into(), "secret".into());
        assert_eq!(value.validate().unwrap_err().code(), "invalidAiProfile");
    }

    #[test]
    fn summary_prompts_cover_templates_and_interface_language() {
        for template in [
            SummaryTemplate::Classic,
            SummaryTemplate::News5w1h,
            SummaryTemplate::Decision,
            SummaryTemplate::Funnel,
            SummaryTemplate::Argument,
            SummaryTemplate::Minimal,
        ] {
            assert!(build_summary_prompt(template, "zh-CN").contains("Simplified Chinese"));
        }
        assert_eq!(response_language_code("ja-JP"), "ja");
        assert_eq!(response_language_code("fr"), "en");
    }

    #[test]
    fn summary_message_is_utf8_safe_and_bounded() {
        let body = "中".repeat(SUMMARY_INPUT_CHAR_LIMIT + 10);
        let message = build_summary_user_message(" 标题 ", &body).unwrap();
        assert!(message.starts_with("Title: 标题\n\n"));
        assert_eq!(
            message.chars().filter(|c| *c == '中').count(),
            SUMMARY_INPUT_CHAR_LIMIT
        );
        assert_eq!(
            build_summary_user_message("Title", "  ")
                .unwrap_err()
                .code(),
            "noArticleBody"
        );
    }

    #[test]
    fn follow_up_message_contains_only_summary_history_and_question() {
        let message = build_follow_up_user_message(
            "cached summary",
            &[("first question".into(), "first answer".into())],
            " next question ",
        )
        .unwrap();
        assert_eq!(
            message,
            "Summary:\n\ncached summary\n\n---\n\nQ: first question\nA: first answer\n\nQ: next question\nA:"
        );
        assert_eq!(
            build_follow_up_user_message("summary", &[], " ")
                .unwrap_err()
                .code(),
            "emptyAiQuestion"
        );
    }

    #[test]
    fn openai_request_uses_transient_credential_without_serializing_it() {
        let client = Client::new();
        let credential = ResolvedAiCredential::new("  sk-secret  ".into()).unwrap();
        let request = build_stream_request(
            &client,
            &profile(),
            Some(&credential),
            "system",
            "user",
            AI_MAX_OUTPUT_TOKENS,
        )
        .unwrap();
        assert_eq!(
            request.headers()[AUTHORIZATION].to_str().unwrap(),
            "Bearer sk-secret"
        );
        assert_eq!(
            request.url().as_str(),
            "https://api.openai.com/v1/chat/completions"
        );
        let body = std::str::from_utf8(request.body().unwrap().as_bytes().unwrap()).unwrap();
        assert!(!body.contains("sk-secret"));
    }

    #[test]
    fn sse_decoder_handles_utf8_chunk_boundaries_and_unterminated_final_frame() {
        let mut decoder = SseDecoder::new(AiProtocol::OpenaiChatCompletions);
        let stream = "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\n\
                      data: {\"choices\":[{\"delta\":{\"content\":\"!\"}}]}";
        let bytes = stream.as_bytes();
        let split = stream.find('好').unwrap() + 1;
        let mut deltas = decoder.push(&bytes[..split]).unwrap();
        deltas.extend(decoder.push(&bytes[split..]).unwrap());
        deltas.extend(decoder.finish().unwrap());
        assert_eq!(deltas, ["你好", "!"]);
    }

    #[test]
    fn sse_decoder_accepts_sensenova_data_wrapped_openai_compatible_deltas() {
        let delta = parse_sse_line(
            b"data: {\"data\":{\"choices\":[{\"delta\":\"summary\"}]},\"status\":{\"code\":0}}",
            AiProtocol::OpenaiChatCompletions,
        )
        .unwrap();

        assert_eq!(delta.as_deref(), Some("summary"));
    }

    #[test]
    fn sse_decoder_accepts_completed_openai_compatible_json_responses() {
        let delta = parse_sse_line(
            b"{\"data\":{\"choices\":[{\"message\":\"summary\"}]}}",
            AiProtocol::OpenaiChatCompletions,
        )
        .unwrap();

        assert_eq!(delta.as_deref(), Some("summary"));
    }

    #[test]
    fn sse_decoder_maps_provider_errors_without_exposing_response_text() {
        let mut decoder = SseDecoder::new(AiProtocol::OpenaiChatCompletions);
        let error = decoder
            .push(b"data: {\"error\":{\"type\":\"rate_limit_error\",\"message\":\"private provider detail\"}}\n")
            .unwrap_err();
        assert_eq!(error.code(), "aiRateLimited");
        assert!(!error.to_string().contains("private provider detail"));

        let null_error = parse_sse_line(
            b"data: {\"error\":null,\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}",
            AiProtocol::OpenaiChatCompletions,
        )
        .unwrap();
        assert_eq!(null_error.as_deref(), Some("ok"));
    }

    #[test]
    fn malformed_sse_and_http_statuses_have_stable_codes() {
        assert_eq!(
            parse_sse_line(b"data: not-json", AiProtocol::AnthropicMessages)
                .unwrap_err()
                .code(),
            "aiParse"
        );
        assert_eq!(http_status_error(StatusCode::UNAUTHORIZED).code(), "aiAuth");
        assert_eq!(
            http_status_error(StatusCode::TOO_MANY_REQUESTS).code(),
            "aiRateLimited"
        );
        assert_eq!(
            http_status_error(StatusCode::INTERNAL_SERVER_ERROR).code(),
            "aiNetwork"
        );
    }

    #[tokio::test]
    async fn cancelled_request_never_reaches_the_network_and_emits_error() {
        let cancellation = AiCancellation::default();
        cancellation.cancel();
        let credential = ResolvedAiCredential::new("secret".into()).unwrap();
        let mut events = Vec::new();
        let error = stream_chat(
            &Client::new(),
            &profile(),
            Some(&credential),
            "request-1",
            "system",
            "user",
            &cancellation,
            |event| {
                events.push(event);
                true
            },
        )
        .await
        .unwrap_err();
        assert_eq!(error.code(), "aiCancelled");
        assert_eq!(
            events,
            [AiStreamEvent::Error {
                request_id: "request-1".into(),
                code: "aiCancelled".into(),
            }]
        );
    }
}
