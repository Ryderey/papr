//! Multi-source URL normalization. Users paste many kinds of links — YouTube
//! channels, subreddits, Mastodon profiles — that are not themselves
//! subscribable feed documents. This module recognizes those patterns and
//! rewrites them to the real feed URL, reporting the resulting [`SourceType`].
//!
//! Everything here is a *pure* function; the one case that needs a page fetch
//! (resolving a YouTube vanity URL) is split into a pure HTML-extraction
//! function ([`extract_channel_id`]), leaving the fetch itself to the caller.
//!
//! Ported from the desktop's `src-tauri/src/ingestion/sources.rs`.

use crate::dto::SourceType;
use url::Url;

/// Outcome of running a pasted string through [`normalize_source`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Normalized {
    /// Recognized and rewritten to a directly-subscribable feed.
    Feed {
        url: String,
        source_type: SourceType,
    },
    /// A YouTube vanity link whose channel id must be learned by fetching the
    /// page, then calling [`extract_channel_id`] + [`youtube_feed_url`].
    NeedsYoutubeResolution { page_url: String },
    /// Not a recognized special source — hand back to the normal feed /
    /// auto-discovery flow.
    Untouched,
}

/// Inspect a user-pasted URL/string and, if it matches a known source pattern,
/// rewrite it to the subscribable feed URL.
pub fn normalize_source(input: &str) -> Normalized {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Normalized::Untouched;
    }

    let with_scheme = super::discovery::normalize_query_url(trimmed);
    let Ok(url) = Url::parse(&with_scheme) else {
        return Normalized::Untouched;
    };
    let host = url.host_str().unwrap_or("").to_lowercase();
    let path = url.path().to_string();

    if host.ends_with("youtube.com") || host == "youtu.be" || host.ends_with(".youtu.be") {
        return normalize_youtube(&host, &path, &url);
    }
    if host == "reddit.com" || host.ends_with(".reddit.com") {
        if let Some(feed) = normalize_reddit(&path) {
            return Normalized::Feed {
                url: feed,
                source_type: SourceType::Reddit,
            };
        }
    }
    if let Some(feed) = normalize_mastodon(&with_scheme, &path) {
        return Normalized::Feed {
            url: feed,
            source_type: SourceType::Mastodon,
        };
    }

    Normalized::Untouched
}

/// Build the canonical YouTube channel feed URL for a `UC…` channel id.
pub fn youtube_feed_url(channel_id: &str) -> String {
    format!("https://www.youtube.com/feeds/videos.xml?channel_id={channel_id}")
}

/// The public RSSHub instance used when the user hasn't configured their own.
pub const DEFAULT_RSSHUB_INSTANCE: &str = "https://rsshub.app";

/// Expand an `rsshub://route` short link into a full feed URL on `instance`.
/// Returns `None` for anything that isn't an `rsshub://` link or for an empty
/// route.
pub fn expand_rsshub(input: &str, instance: &str) -> Option<String> {
    let trimmed = input.trim();
    if !trimmed.get(..9)?.eq_ignore_ascii_case("rsshub://") {
        return None;
    }
    let route = trimmed[9..].trim_start_matches('/');
    if route.is_empty() {
        return None;
    }
    let base = instance.trim().trim_end_matches('/');
    Some(format!("{base}/{route}"))
}

/// True for the URL-safe id characters YouTube uses in channel/playlist ids.
fn is_id_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Split a URL path into its non-empty segments.
fn path_segments(path: &str) -> Vec<&str> {
    path.split('/').filter(|s| !s.is_empty()).collect()
}

/// True if `id` looks like a YouTube channel id (`UC` + 22 chars).
fn is_channel_id(id: &str) -> bool {
    id.len() == 24 && id.starts_with("UC") && id.chars().all(is_id_char)
}

/// True if `id` looks like a YouTube playlist id (`PL`, `UU`, `LL`, `FL`, …).
fn is_playlist_id(id: &str) -> bool {
    (id.len() >= 13)
        && id.chars().all(is_id_char)
        && (id.starts_with("PL")
            || id.starts_with("UU")
            || id.starts_with("LL")
            || id.starts_with("FL")
            || id.starts_with("OL"))
}

fn normalize_youtube(host: &str, path: &str, url: &Url) -> Normalized {
    // Already a feed document — leave it for the normal flow.
    if path.contains("/feeds/videos.xml") {
        return Normalized::Untouched;
    }

    // A `playlist?list=PL…` URL has a playlist feed endpoint.
    if path.starts_with("/playlist") {
        if let Some((_, list)) = url.query_pairs().find(|(k, _)| k == "list") {
            if is_playlist_id(&list) {
                return Normalized::Feed {
                    url: format!("https://www.youtube.com/feeds/videos.xml?playlist_id={list}"),
                    source_type: SourceType::Youtube,
                };
            }
        }
    }

    let segments = path_segments(path);

    // `youtube.com/channel/UC…` — the channel id is right there in the path.
    if segments.len() >= 2 && segments[0] == "channel" && is_channel_id(segments[1]) {
        return Normalized::Feed {
            url: youtube_feed_url(segments[1]),
            source_type: SourceType::Youtube,
        };
    }

    // Vanity URLs that need a page fetch.
    let is_vanity = segments
        .first()
        .map(|s| s.starts_with('@') || matches!(*s, "c" | "user"))
        .unwrap_or(false);
    let is_short_host = host == "youtu.be" || host.ends_with(".youtu.be");
    if is_vanity || (is_short_host && !segments.is_empty()) {
        return Normalized::NeedsYoutubeResolution {
            page_url: url.as_str().to_string(),
        };
    }

    Normalized::Untouched
}

/// Extract `r/SUBREDDIT` from a Reddit path and build its `.rss` feed URL.
fn normalize_reddit(path: &str) -> Option<String> {
    let segments = path_segments(path);
    if segments.len() < 2 || segments[0] != "r" {
        return None;
    }
    let sub = segments[1];
    if sub.is_empty() || !sub.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return None;
    }
    match segments.get(2).copied() {
        None => Some(format!("https://www.reddit.com/r/{sub}/.rss")),
        Some(listing @ ("hot" | "new" | "top" | "rising")) => {
            Some(format!("https://www.reddit.com/r/{sub}/{listing}/.rss"))
        }
        Some(s) if s.ends_with(".rss") => None,
        Some(_) => None,
    }
}

/// Recognize a Mastodon profile URL (`https://instance/@user`) and append the
/// `.rss` suffix Mastodon exposes for every account's public timeline.
fn normalize_mastodon(full_url: &str, path: &str) -> Option<String> {
    let segments = path_segments(path);
    if segments.len() != 1 {
        return None;
    }
    let handle = segments[0];
    if !handle.starts_with('@') || handle.len() < 2 {
        return None;
    }
    if handle.ends_with(".rss") {
        return None;
    }
    let base = Url::parse(full_url).ok()?;
    let host = base.host_str()?;
    let scheme = base.scheme();
    Some(format!("{scheme}://{host}/{handle}.rss"))
}

/// Pull a YouTube channel id (`UC…`) out of a channel page's HTML. Pure and
/// network-free. Owner-specific signals are tried before the ambiguous plain
/// `"channelId"` JSON key.
pub fn extract_channel_id(html: &str) -> Option<String> {
    for key in ["\"externalId\":\"", "\"externalChannelId\":\""] {
        if let Some(id) = find_after(html, key, '"') {
            if is_channel_id(&id) {
                return Some(id);
            }
        }
    }

    if let Some(id) = canonical_channel_id(html) {
        return Some(id);
    }

    if let Some(id) = channel_id_after_marker(html) {
        return Some(id);
    }

    if let Some(id) = find_after(html, "\"channelId\":\"", '"') {
        if is_channel_id(&id) {
            return Some(id);
        }
    }

    None
}

/// Pull a valid channel id out of the text following the first `/channel/`
/// marker in `text`.
fn channel_id_after_marker(text: &str) -> Option<String> {
    let rest = text.split("/channel/").nth(1)?;
    let id: String = rest.chars().take_while(|c| is_id_char(*c)).collect();
    is_channel_id(&id).then_some(id)
}

/// Return the substring of `html` after the first occurrence of `marker`, up to
/// (but not including) the next `end` byte.
fn find_after(html: &str, marker: &str, end: char) -> Option<String> {
    let start = html.find(marker)? + marker.len();
    let tail = &html[start..];
    let stop = tail.find(end)?;
    Some(tail[..stop].to_string())
}

/// Find a `<link rel="canonical">` tag and pull a `/channel/UC…` id from its
/// `href`.
fn canonical_channel_id(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let mut search = 0;
    while let Some(rel) = lower[search..].find("rel=\"canonical\"") {
        let abs = search + rel;
        let Some(rel_tag_end) = lower[abs..].find('>') else {
            break;
        };
        let tag_end = rel_tag_end + abs;
        let Some(tag_start) = lower[..abs].rfind('<') else {
            search = tag_end;
            continue;
        };
        let tag = &html[tag_start..tag_end];
        if let Some(id) = channel_id_after_marker(tag) {
            return Some(id);
        }
        search = tag_end;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(n: Normalized) -> (String, SourceType) {
        match n {
            Normalized::Feed { url, source_type } => (url, source_type),
            other => panic!("expected Feed, got {other:?}"),
        }
    }

    #[test]
    fn youtube_channel_url_rewrites_to_feed() {
        let (url, st) = feed(normalize_source(
            "https://www.youtube.com/channel/UCXuqSBlHAE6Xw-yeJA0Tunw",
        ));
        assert_eq!(
            url,
            "https://www.youtube.com/feeds/videos.xml?channel_id=UCXuqSBlHAE6Xw-yeJA0Tunw"
        );
        assert_eq!(st, SourceType::Youtube);
    }

    #[test]
    fn reddit_subreddit_rewrites_to_rss() {
        let (url, st) = feed(normalize_source("https://www.reddit.com/r/rust"));
        assert_eq!(url, "https://www.reddit.com/r/rust/.rss");
        assert_eq!(st, SourceType::Reddit);
    }

    #[test]
    fn mastodon_profile_rewrites_to_rss() {
        let (url, st) = feed(normalize_source("https://mastodon.social/@Gargron"));
        assert_eq!(url, "https://mastodon.social/@Gargron.rss");
        assert_eq!(st, SourceType::Mastodon);
    }

    #[test]
    fn plain_feed_url_untouched() {
        assert_eq!(
            normalize_source("https://blog.rust-lang.org/feed.xml"),
            Normalized::Untouched
        );
    }

    #[test]
    fn expand_rsshub_maps_route_onto_instance() {
        assert_eq!(
            expand_rsshub(
                "rsshub://github/issue/DIYgod/RSSHub",
                DEFAULT_RSSHUB_INSTANCE
            )
            .as_deref(),
            Some("https://rsshub.app/github/issue/DIYgod/RSSHub")
        );
    }

    #[test]
    fn extract_channel_id_from_json_key() {
        let html = r#"<html><script>var x = {"channelId":"UCXuqSBlHAE6Xw-yeJA0Tunw","foo":1};</script></html>"#;
        assert_eq!(
            extract_channel_id(html).as_deref(),
            Some("UCXuqSBlHAE6Xw-yeJA0Tunw")
        );
    }

    #[test]
    fn extract_channel_id_none_when_absent() {
        assert_eq!(extract_channel_id("<html>no channel here</html>"), None);
    }
}
