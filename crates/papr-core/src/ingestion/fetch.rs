//! HTTP fetching with conditional GET (ETag / If-Modified-Since).

use reqwest::header::{CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use reqwest::{Client, StatusCode};
use std::time::Duration;

use crate::error::CoreError;

pub const USER_AGENT: &str = "Papr/0.1 (+https://github.com/papr-reader)";

const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

async fn read_capped(mut resp: reqwest::Response) -> Result<Vec<u8>, CoreError> {
    if resp
        .content_length()
        .is_some_and(|n| n > MAX_BODY_BYTES as u64)
    {
        return Err(CoreError::Network("response too large".to_string()));
    }
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| CoreError::Network(e.to_string()))?
    {
        if buf.len() + chunk.len() > MAX_BODY_BYTES {
            return Err(CoreError::Network("response too large".to_string()));
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(buf)
}

/// Build the shared HTTP client.
pub fn build_client(
    timeout_secs: u64,
    proxy: &str,
    user_agent: Option<&str>,
) -> Result<Client, CoreError> {
    let mut builder = Client::builder()
        .user_agent(user_agent.unwrap_or(USER_AGENT))
        .timeout(Duration::from_secs(timeout_secs.clamp(5, 300)))
        .connect_timeout(Duration::from_secs(10));

    match proxy {
        "system" | "" => {}
        "none" => builder = builder.no_proxy(),
        custom => {
            if let Ok(p) = reqwest::Proxy::all(custom) {
                builder = builder.proxy(p);
            }
        }
    }
    builder
        .build()
        .map_err(|e| CoreError::Network(format!("failed to build client: {}", e)))
}

/// Result of a conditional GET against a feed URL.
pub enum Fetched {
    NotModified,
    Body {
        bytes: Vec<u8>,
        etag: Option<String>,
        last_modified: Option<String>,
    },
}

/// Conditional GET. Sends `If-None-Match`/`If-Modified-Since` when we have them.
pub async fn conditional_get(
    client: &Client,
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<Fetched, CoreError> {
    let mut req = client.get(url);
    if let Some(e) = etag {
        req = req.header(IF_NONE_MATCH, e);
    }
    if let Some(lm) = last_modified {
        req = req.header(IF_MODIFIED_SINCE, lm);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| CoreError::Network(e.to_string()))?;
    if resp.status() == StatusCode::NOT_MODIFIED {
        return Ok(Fetched::NotModified);
    }
    let resp = resp
        .error_for_status()
        .map_err(|e| CoreError::Network(e.to_string()))?;
    let header = |name: reqwest::header::HeaderName| {
        resp.headers()
            .get(&name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    };
    let etag = header(ETAG);
    let last_modified = header(LAST_MODIFIED);
    let bytes = read_capped(resp).await?;
    Ok(Fetched::Body {
        bytes,
        etag,
        last_modified,
    })
}

/// Pull the `charset` parameter out of a `Content-Type` header value.
fn charset_from_content_type(content_type: &str) -> Option<String> {
    content_type
        .split(';')
        .filter_map(|p| p.split_once('='))
        .find(|(k, _)| k.trim().eq_ignore_ascii_case("charset"))
        .map(|(_, v)| v.trim().trim_matches('"').trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
}

fn parse_charset_value(tail: &str) -> Option<String> {
    let tail = tail.trim_start().strip_prefix('=').unwrap_or(tail);
    let value: String = tail
        .trim_start()
        .trim_start_matches(['"', '\''])
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    (!value.is_empty()).then_some(value)
}

fn charset_from_html(bytes: &[u8]) -> Option<String> {
    let head = &bytes[..bytes.len().min(2048)];
    let text = String::from_utf8_lossy(head).to_ascii_lowercase();
    let mut search = 0;
    while let Some(rel) = text[search..].find("<meta") {
        let tag_start = search + rel;
        let tag_end = text[tag_start..]
            .find('>')
            .map(|i| tag_start + i)
            .unwrap_or(text.len());
        let tag = &text[tag_start..tag_end];
        if let Some(i) = tag.find("charset") {
            if let Some(value) = parse_charset_value(&tag[i + "charset".len()..]) {
                return Some(value);
            }
        }
        search = tag_end;
    }
    None
}

/// Decode fetched HTML/text bytes into a `String`.
pub fn decode_html(bytes: &[u8], content_type: Option<&str>) -> String {
    let label = content_type
        .and_then(charset_from_content_type)
        .or_else(|| charset_from_html(bytes));
    let encoding = label
        .as_deref()
        .and_then(|l| encoding_rs::Encoding::for_label(l.as_bytes()))
        .unwrap_or(encoding_rs::UTF_8);
    let (text, _, _) = encoding.decode(bytes);
    text.into_owned()
}

/// Plain GET returning `(body, content_type, final_url)`.
pub async fn get(
    client: &Client,
    url: &str,
) -> Result<(Vec<u8>, Option<String>, String), CoreError> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| CoreError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| CoreError::Network(e.to_string()))?;
    let final_url = resp.url().to_string();
    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let bytes = read_capped(resp).await?;
    Ok((bytes, content_type, final_url))
}
