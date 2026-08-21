//! Feed discovery: a bundled curated directory plus `papr://subscribe` deep-link
//! parsing. All pure and network-free, so they are unit-tested without the
//! filesystem or the network.
//!
//! Ported from the desktop's `src-tauri/src/ingestion/discovery.rs`.

use serde::Deserialize;

use crate::dto::DiscoveryResult;

/// The curated directory, embedded into the binary at compile time.
const DIRECTORY_JSON: &str = include_str!("../../resources/feed-directory.json");

/// One entry in the curated feed directory.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntry {
    pub title: String,
    pub feed_url: String,
    pub site_url: String,
    pub category: String,
    pub description: String,
    pub lang: String,
}

fn from_entry(e: &DirectoryEntry) -> DiscoveryResult {
    DiscoveryResult {
        title: e.title.clone(),
        feed_url: e.feed_url.clone(),
        site_url: Some(e.site_url.clone()),
        category: Some(e.category.clone()),
        description: Some(e.description.clone()),
        from_directory: true,
    }
}

/// Build a result from a feed URL discovered live on a scraped page.
pub fn from_scrape(feed_url: String, title: Option<String>) -> DiscoveryResult {
    DiscoveryResult {
        title: title.unwrap_or_else(|| feed_url.clone()),
        feed_url,
        site_url: None,
        category: None,
        description: None,
        from_directory: false,
    }
}

/// Parse the embedded directory JSON. Panics only on a malformed bundled asset.
pub fn directory() -> Vec<DirectoryEntry> {
    serde_json::from_str(DIRECTORY_JSON).expect("bundled feed-directory.json is valid JSON")
}

/// The primary subtag of a BCP-47-ish language tag, lowercased.
fn primary_lang(tag: &str) -> String {
    tag.split(['-', '_']).next().unwrap_or(tag).to_lowercase()
}

/// Case-insensitive search of the curated directory, scoped to one language.
/// Falls back to English when the directory has no slice for the requested
/// language. An empty query returns the whole language slice.
pub fn search_directory(query: &str, lang: &str) -> Vec<DiscoveryResult> {
    let needle = query.trim().to_lowercase();
    let dir = directory();
    let want = primary_lang(lang);
    let want = if dir.iter().any(|e| primary_lang(&e.lang) == want) {
        want
    } else {
        "en".to_string()
    };
    dir.iter()
        .filter(|e| primary_lang(&e.lang) == want)
        .filter(|e| {
            needle.is_empty()
                || e.title.to_lowercase().contains(&needle)
                || e.category.to_lowercase().contains(&needle)
                || e.description.to_lowercase().contains(&needle)
        })
        .map(from_entry)
        .collect()
}

/// True when the query looks like a URL or bare domain.
pub fn looks_like_url(query: &str) -> bool {
    let q = query.trim();
    if q.is_empty() || q.contains(char::is_whitespace) {
        return false;
    }
    if q.contains("://") {
        return true;
    }
    let host = q.split(['/', '?', '#']).next().unwrap_or(q);
    let labels: Vec<&str> = host.split('.').filter(|s| !s.is_empty()).collect();
    if labels.len() < 2 {
        return false;
    }
    let tld = labels.last().copied().unwrap_or("");
    tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic())
}

/// Normalize a URL-ish query into a fetchable absolute URL by adding a scheme
/// when the user typed a bare domain.
pub fn normalize_query_url(query: &str) -> String {
    let q = query.trim();
    if q.contains("://") {
        q.to_string()
    } else {
        format!("https://{q}")
    }
}

/// The outcome of parsing a `papr://` deep link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeepLink {
    Subscribe { url: String },
}

/// Parse a `papr://subscribe?url=<encoded>` deep link. Returns `None` for
/// anything that is not a well-formed subscribe link.
pub fn parse_deep_link(input: &str) -> Option<DeepLink> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = url::Url::parse(trimmed).ok()?;
    if parsed.scheme() != "papr" {
        return None;
    }
    let action = match parsed.host_str() {
        Some(h) if !h.is_empty() => h.to_string(),
        _ => parsed.path().trim_matches('/').to_string(),
    };
    if action != "subscribe" {
        return None;
    }
    let target = parsed
        .query_pairs()
        .find(|(k, _)| k == "url")
        .map(|(_, v)| v.into_owned())?;
    let target = target.trim();
    if target.is_empty() {
        return None;
    }
    Some(DeepLink::Subscribe {
        url: target.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_parses_and_is_substantial() {
        let dir = directory();
        assert!(dir.len() >= 40);
        for e in &dir {
            assert!(!e.title.is_empty());
            assert!(e.feed_url.starts_with("http"));
            assert!(!e.category.is_empty());
            assert!(!e.lang.is_empty());
        }
    }

    #[test]
    fn empty_query_returns_whole_language_slice() {
        assert!(!search_directory("", "en").is_empty());
        assert!(!search_directory("", "zh").is_empty());
        assert!(!search_directory("", "ja").is_empty());
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        assert_eq!(
            search_directory("", "ko").len(),
            search_directory("", "en").len()
        );
    }

    #[test]
    fn search_matches_title_case_insensitively() {
        let hits = search_directory("hacker news", "en");
        assert!(hits.iter().any(|r| r.title == "Hacker News"));
    }

    #[test]
    fn looks_like_url_detects_schemed_and_bare() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("example.com"));
        assert!(!looks_like_url("science news"));
        assert!(!looks_like_url(""));
    }

    #[test]
    fn normalize_query_url_adds_scheme_when_missing() {
        assert_eq!(normalize_query_url("example.com"), "https://example.com");
        assert_eq!(
            normalize_query_url("http://example.com"),
            "http://example.com"
        );
    }

    #[test]
    fn deep_link_basic_subscribe() {
        assert_eq!(
            parse_deep_link("papr://subscribe?url=https://example.com/feed.xml"),
            Some(DeepLink::Subscribe {
                url: "https://example.com/feed.xml".to_string()
            })
        );
    }

    #[test]
    fn deep_link_rejects_unknown_action_and_missing_url() {
        assert_eq!(
            parse_deep_link("papr://unsubscribe?url=https://x.com"),
            None
        );
        assert_eq!(parse_deep_link("papr://subscribe"), None);
        assert_eq!(parse_deep_link("not a url"), None);
    }
}
