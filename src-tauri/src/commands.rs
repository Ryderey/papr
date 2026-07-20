//! Tauri command surface — the typed IPC boundary the React frontend calls.
//! All SQL is delegated to `db`; commands only orchestrate.

use crate::ai::{self, AiConfig, AiEvent, LlmPurpose};
use crate::db::{self};
use crate::error::{AppError, AppResult};
use crate::extraction;
use crate::ingestion::discovery::{self, DiscoveryResult};
use crate::ingestion::newsletter::{self, NewsletterConfig};
use crate::ingestion::sources::{self, Normalized};
use crate::ingestion::{fetch, parse, scheduler};
use crate::models::*;
use crate::opml;
use crate::sanitize;
use crate::state::AppState;
use crate::translate;
use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, AppHandle, Emitter, Manager, State};
use url::Url;

// ─────────────────────────── folders ───────────────────────────

#[tauri::command]
pub async fn list_folders(state: State<'_, AppState>) -> AppResult<Vec<Folder>> {
    let conn = state.read().await;
    db::list_folders(&conn)
}

#[tauri::command]
pub async fn create_folder(state: State<'_, AppState>, name: String) -> AppResult<i64> {
    let conn = state.db.lock().await;
    db::create_folder(&conn, &name)
}

#[tauri::command]
pub async fn rename_folder(state: State<'_, AppState>, id: i64, name: String) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::rename_folder(&conn, id, &name)
}

#[tauri::command]
pub async fn delete_folder(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::delete_folder(&conn, id)
}

// ─────────────────────────── feeds ───────────────────────────

#[tauri::command]
pub async fn list_feeds(state: State<'_, AppState>) -> AppResult<Vec<Feed>> {
    let conn = state.read().await;
    db::list_feeds(&conn)
}

/// Subscribe to a feed. Accepts either a feed URL or a website URL (in which
/// case we auto-discover the feed). Also recognizes multi-source URLs —
/// YouTube channels, subreddits, Mastodon profiles — and rewrites them to the
/// real feed URL first (feature F5). Fetches once so the feed is immediately
/// populated, then returns the stored feed.
#[tauri::command]
pub async fn add_feed(
    state: State<'_, AppState>,
    url: String,
    folder_id: Option<i64>,
) -> AppResult<Feed> {
    let client = state.http();

    // Step 0a: expand an `rsshub://route` short link into a normal feed URL on
    // the configured RSSHub instance (default: the public rsshub.app). The
    // expansion is a plain HTTP feed URL, so the rest of the pipeline handles
    // it with no further special-casing. Only touch the DB when it's actually
    // an rsshub link, so the common case pays nothing.
    let url = if url.trim().get(..9).is_some_and(|s| s.eq_ignore_ascii_case("rsshub://")) {
        let instance = {
            let conn = state.db.lock().await;
            db::get_setting(&conn, "rsshub_instance")?
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| sources::DEFAULT_RSSHUB_INSTANCE.to_string())
        };
        sources::expand_rsshub(&url, &instance).unwrap_or(url)
    } else {
        url
    };

    // Step 0: multi-source normalization. If the pasted URL is a known
    // special source, rewrite it to its real feed URL. A YouTube vanity URL
    // needs the channel page fetched to learn its channel id; that single
    // network call lives here (the extraction logic itself is pure).
    let (effective_url, forced_type): (String, Option<SourceType>) =
        match sources::normalize_source(&url) {
            Normalized::Feed { url, source_type } => (url, Some(source_type)),
            Normalized::NeedsYoutubeResolution { page_url } => {
                let (page_bytes, ct, _) = fetch::get(&client, &page_url).await?;
                let html = fetch::decode_html(&page_bytes, ct.as_deref());
                let channel_id = sources::extract_channel_id(&html)
                    .ok_or_else(|| AppError::code("youtubeChannelNotFound"))?;
                (
                    sources::youtube_feed_url(&channel_id),
                    Some(SourceType::Youtube),
                )
            }
            Normalized::Untouched => (url.clone(), None),
        };

    // Step 1: fetch whatever the user gave us (or the normalized feed URL).
    let (bytes, ct, final_url) = fetch::get(&client, &effective_url).await?;

    // Step 2: if it is a feed use it directly, otherwise discover one.
    let (feed_url, feed_bytes) = if parse::looks_like_feed(&bytes) {
        (final_url, bytes)
    } else {
        let html = fetch::decode_html(&bytes, ct.as_deref());
        let candidates = parse::discover_feeds(&html, &final_url);
        let candidate = candidates
            .into_iter()
            .next()
            .ok_or_else(|| AppError::code("noFeedFound"))?;
        let (fb, _, _) = fetch::get(&client, &candidate).await?;
        (candidate, fb)
    };

    // Step 3: parse and classify. A normalization step that already pinned a
    // source type (YouTube / Reddit / Mastodon) wins over heuristic detection.
    let parsed = parse::parse_feed(&feed_bytes, &feed_url)?;
    let source_type = match forced_type {
        Some(t) => t,
        None => parse::refine_source_type(
            parse::detect_source_type(&feed_url),
            &parsed,
            &feed_url,
        ),
    };

    let title = parsed
        .title
        .clone()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| feed_url.clone());
    let favicon = parsed.icon.clone().or_else(|| {
        parsed
            .site_url
            .as_deref()
            .and_then(|s| Url::parse(s).ok())
            .and_then(|u| u.host_str().map(String::from))
            .map(|h| format!("https://www.google.com/s2/favicons?domain={h}&sz=64"))
    });

    // Step 4: persist.
    let conn = state.db.lock().await;
    if db::find_feed_by_url(&conn, &feed_url)?.is_some() {
        return Err(AppError::code("alreadySubscribed"));
    }
    let feed_id = db::insert_feed(
        &conn,
        &feed_url,
        parsed.site_url.as_deref(),
        &title,
        parsed.description.as_deref(),
        source_type,
        folder_id,
    )?;
    if let Some(fav) = &favicon {
        db::update_feed_meta(&conn, feed_id, None, None, None, Some(fav))?;
    }
    let dedup = db::setting_flag(&conn, "dedup_enabled", false);
    let rules = db::active_rules(&conn).unwrap_or_default();
    for article in &parsed.articles {
        db::upsert_article(&conn, feed_id, article, dedup, &rules)?;
    }
    // Record that the feed was just fetched. `add_feed` fetches the document
    // here in step 1/2, so without this `last_fetched_at` would stay NULL —
    // the feed would wrongly read as "never refreshed" until the next
    // scheduler tick, and the tick would also re-fetch it in full a moment
    // after this add. (The conditional-GET revalidators are not captured —
    // `fetch::get` does not surface ETag / Last-Modified — so the next poll
    // does one full GET before it can store them; that is a single missed
    // optimisation, not incorrect behaviour, and many feeds send no ETag at
    // all.)
    let _ = db::touch_feed(&conn, feed_id);
    let last_fetched_at = db::feed_last_fetched(&conn, feed_id).ok().flatten();
    // Count actual unread rows rather than tallying insertions: keeps the
    // returned `unread_count` aligned with the sidebar's `list_feeds` count
    // regardless of how filter rules pre-set article state.
    let unread = db::count_feed_unread(&conn, feed_id)?;
    drop(conn);

    Ok(Feed {
        id: feed_id,
        feed_url,
        site_url: parsed.site_url,
        title,
        description: parsed.description,
        favicon_url: favicon,
        folder_id,
        source_type: source_type.as_str().to_string(),
        last_fetched_at,
        fetch_error: None,
        unread_count: unread,
        refresh_interval_min: None,
    })
}

/// Feed discovery (feature F6). Searches the bundled curated directory for
/// entries matching `query`, scoped to `lang` (the user's UI language) so the
/// recommendations are in a language they read. When `query` looks like a URL
/// or bare domain we ALSO fetch that page and run `parse::discover_feeds` over
/// it, so the same box doubles as smart URL handling. Live page-scrape results
/// are returned first (most specific to what the user typed), then the
/// directory matches.
///
/// A failed page fetch is non-fatal — the directory results are still
/// returned — so a typo or an offline site does not break discovery.
#[tauri::command]
pub async fn search_feed_directory(
    state: State<'_, AppState>,
    query: String,
    lang: String,
) -> AppResult<Vec<DiscoveryResult>> {
    let mut results: Vec<DiscoveryResult> = Vec::new();

    // Live page scrape — only when the query genuinely looks like a URL.
    if discovery::looks_like_url(&query) {
        let target = discovery::normalize_query_url(&query);
        let client = state.http();
        if let Ok((bytes, ct, final_url)) = fetch::get(&client, &target).await {
            if parse::looks_like_feed(&bytes) {
                // The pasted URL is itself a feed — surface it directly.
                let title = parse::parse_feed(&bytes, &final_url)
                    .ok()
                    .and_then(|p| p.title);
                results.push(DiscoveryResult::from_scrape(final_url, title));
            } else {
                let html = fetch::decode_html(&bytes, ct.as_deref());
                for feed_url in parse::discover_feeds(&html, &final_url) {
                    results.push(DiscoveryResult::from_scrape(feed_url, None));
                }
            }
        }
    }

    // Curated directory matches (deduplicated against the scrape results),
    // scoped to the user's UI language so the recommendations read natively.
    for hit in discovery::search_directory(&query, &lang) {
        if !results.iter().any(|r| r.feed_url == hit.feed_url) {
            results.push(hit);
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn delete_feed(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::delete_feed(&conn, id)
}

#[tauri::command]
pub async fn move_feed(
    state: State<'_, AppState>,
    id: i64,
    folder_id: Option<i64>,
) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::move_feed(&conn, id, folder_id)
}

/// Set a feed's per-feed refresh interval. `None` reverts it to the global
/// interval; `Some(525600)` (the "off" sentinel) opts the feed out of
/// automatic refresh. The change is honoured on the scheduler's next tick.
#[tauri::command]
pub async fn set_feed_refresh_interval(
    state: State<'_, AppState>,
    id: i64,
    minutes: Option<i64>,
) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::set_feed_refresh_interval(&conn, id, minutes)
}

#[tauri::command]
pub async fn rename_feed(state: State<'_, AppState>, id: i64, title: String) -> AppResult<()> {
    // `db::rename_feed` trims and rejects an empty title — the one chokepoint.
    let conn = state.db.lock().await;
    db::rename_feed(&conn, id, &title)
}

/// Refresh every feed, streaming progress to the frontend over `on_progress`.
#[tauri::command]
pub async fn refresh_feeds(
    app: AppHandle,
    on_progress: Channel<RefreshProgress>,
) -> AppResult<usize> {
    scheduler::refresh_all(&app, Some(on_progress), false, scheduler::RefreshScope::All).await
}

// ─────────────────────────── articles ───────────────────────────

#[tauri::command]
pub async fn list_articles(
    state: State<'_, AppState>,
    query: ArticleQuery,
    unread_only: bool,
    search: Option<String>,
    oldest_first: bool,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<ArticleSummary>> {
    let conn = state.read().await;
    db::list_articles(
        &conn,
        &query,
        unread_only,
        search.as_deref(),
        oldest_first,
        limit,
        offset,
    )
}

#[tauri::command]
pub async fn get_article(state: State<'_, AppState>, id: i64) -> AppResult<ArticleDetail> {
    let conn = state.read().await;
    db::get_article(&conn, id)
}

/// Queue a read/starred change for FreshRSS, but only when a server is linked.
fn enqueue_if_connected(conn: &rusqlite::Connection, id: i64, field: &str, value: bool) {
    if db::is_freshrss_connected(conn) {
        let _ = db::enqueue_sync(conn, id, field, value);
    }
}

/// Refresh the two unread surfaces — the Dock badge and the menu-bar tray —
/// after an operation that changed the unread count.
async fn refresh_unread_surfaces(app: &AppHandle) {
    crate::notify::update_badge(app).await;
    crate::tray::refresh(app).await;
}

#[tauri::command]
pub async fn mark_read(app: AppHandle, id: i64, read: bool) -> AppResult<()> {
    {
        let state = app.state::<AppState>();
        let conn = state.db.lock().await;
        db::set_read(&conn, id, read)?;
        enqueue_if_connected(&conn, id, "read", read);
    }
    refresh_unread_surfaces(&app).await;
    Ok(())
}

#[tauri::command]
pub async fn mark_starred(app: AppHandle, id: i64, starred: bool) -> AppResult<()> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().await;
    db::set_starred(&conn, id, starred)?;
    enqueue_if_connected(&conn, id, "starred", starred);
    Ok(())
}

#[tauri::command]
pub async fn mark_read_later(state: State<'_, AppState>, id: i64, value: bool) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::set_read_later(&conn, id, value)
}

#[tauri::command]
pub async fn mark_all_read(app: AppHandle, query: ArticleQuery) -> AppResult<usize> {
    let n = {
        let state = app.state::<AppState>();
        let conn = state.db.lock().await;
        db::mark_all_read(&conn, &query, db::is_freshrss_connected(&conn))?
    };
    let _ = app.emit("feeds-updated", 0);
    refresh_unread_surfaces(&app).await;
    Ok(n)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartCounts {
    unread: i64,
    starred: i64,
    read_later: i64,
}

#[tauri::command]
pub async fn smart_counts(state: State<'_, AppState>) -> AppResult<SmartCounts> {
    let conn = state.read().await;
    let (unread, starred, read_later) = db::smart_counts(&conn)?;
    Ok(SmartCounts {
        unread,
        starred,
        read_later,
    })
}

// ─────────────────────────── full-text extraction ───────────────────────────

/// Fetch the article's source page and extract its full text (Readability).
/// Stores the result so subsequent reads are instant/offline.
#[tauri::command]
pub async fn extract_fulltext(state: State<'_, AppState>, article_id: i64) -> AppResult<String> {
    let url = {
        let conn = state.read().await;
        db::get_article(&conn, article_id)?
            .url
            .ok_or_else(|| AppError::code("noArticleUrl"))?
    };

    let http = state.http();
    let (bytes, ct, final_url) = fetch::get(&http, &url).await?;
    // Decode in the page's declared charset — a non-UTF-8 page (Shift-JIS,
    // GBK, ISO-8859-1, …) would otherwise become mojibake before Readability.
    let html = fetch::decode_html(&bytes, ct.as_deref());
    let lead_image = extraction::lead_image(&html, &final_url);

    // Readability is not Send — run it on the blocking pool.
    let extraction_url = final_url.clone();
    let extracted =
        tokio::task::spawn_blocking(move || extraction::extract_article(&html, &extraction_url))
            .await
            .map_err(|e| AppError::other(format!("extraction task: {e}")))??;
    let image_url = sanitize::first_image(&extracted).or(lead_image);

    let conn = state.db.lock().await;
    db::set_extracted_html(&conn, article_id, &extracted, image_url.as_deref())?;
    Ok(extracted)
}

// ─────────────────────────── image download ───────────────────────────

/// A mainstream browser User-Agent. Some image hosts gate on it in addition to
/// the `Referer`; sending a browser UA (and no Referer) mirrors what the webview
/// itself does when it renders the image, so a save succeeds wherever the inline
/// render did.
const IMAGE_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
    AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";

/// Upper bound on a saved image, so a hostile or mistyped URL can't balloon
/// memory. Well above any real article image.
const MAX_IMAGE_BYTES: u64 = 25 * 1024 * 1024;

/// The `Referer` values to try, in order, when fetching an image. Hotlink
/// protection cuts both ways: blacklist hosts (`*.sinaimg.cn`) 403 a foreign
/// Referer but serve a bare request, while others (`cdnfile.sspai.com`) 403 a
/// bare request and demand a Referer be present; strict whitelist CDNs accept
/// only their own site — which the article URL satisfies. No single value
/// works everywhere, so the fetch walks this chain until one succeeds.
fn referer_candidates(image_url: &str, page_url: Option<&str>) -> Vec<Option<String>> {
    let mut out = vec![None];
    if let Ok(u) = Url::parse(image_url) {
        let origin = u.origin().ascii_serialization();
        if origin != "null" {
            out.push(Some(format!("{origin}/")));
        }
    }
    if let Some(p) = page_url {
        if (p.starts_with("http://") || p.starts_with("https://")) && Url::parse(p).is_ok() {
            let candidate = Some(p.to_string());
            if !out.contains(&candidate) {
                out.push(candidate);
            }
        }
    }
    out
}

/// Fetch a feed image's bytes — for the reader's "Save image" action and as
/// the retry path for images the webview itself failed to load.
///
/// Done in Rust rather than via the webview so the request's `Referer` can be
/// controlled: it walks [`referer_candidates`] (none → image origin → article
/// URL) until the host serves the image, which covers both blacklist- and
/// whitelist-style hotlink protection. `page_url` is the article's link, used
/// as the final candidate. Transport errors abort the chain — a different
/// Referer can't fix an unreachable host — only HTTP status errors advance it.
#[tauri::command]
pub async fn fetch_image(
    state: State<'_, AppState>,
    url: String,
    page_url: Option<String>,
) -> AppResult<Vec<u8>> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(AppError::code("badImageUrl"));
    }
    let http = state.http();
    let mut last_err = AppError::code("badImageUrl");
    for referer in referer_candidates(&url, page_url.as_deref()) {
        let mut req = http.get(&url).header("User-Agent", IMAGE_UA);
        if let Some(r) = &referer {
            req = req.header("Referer", r.as_str());
        }
        match req.send().await?.error_for_status() {
            Err(e) => last_err = e.into(),
            Ok(resp) => {
                if resp.content_length().is_some_and(|n| n > MAX_IMAGE_BYTES) {
                    return Err(AppError::code("imageTooLarge"));
                }
                let bytes = resp.bytes().await?;
                if bytes.len() as u64 > MAX_IMAGE_BYTES {
                    return Err(AppError::code("imageTooLarge"));
                }
                return Ok(bytes.to_vec());
            }
        }
    }
    Err(last_err)
}

// ─────────────────────────── OPML ───────────────────────────

#[tauri::command]
pub async fn import_opml(app: AppHandle, content: String) -> AppResult<usize> {
    let imported = opml::parse(&content)?;
    let count = {
        let state = app.state::<AppState>();
        let conn = state.db.lock().await;
        // One transaction for the whole import — a mid-list failure rolls
        // back rather than leaving feeds (and auto-created folders) partly
        // imported.
        let tx = conn.unchecked_transaction()?;
        let mut added = 0;
        for feed in imported {
            if db::find_feed_by_url(&tx, &feed.feed_url)?.is_some() {
                continue;
            }
            let folder_id = match &feed.folder {
                Some(name) => Some(db::folder_id_by_name(&tx, name)?),
                None => None,
            };
            let source_type = parse::detect_source_type(&feed.feed_url);
            db::insert_feed(
                &tx,
                &feed.feed_url,
                None,
                &feed.title,
                None,
                source_type,
                folder_id,
            )?;
            added += 1;
        }
        tx.commit()?;
        added
    };
    // Newly imported feeds have no articles yet — kick off a refresh. Pass
    // wait_if_busy so it queues behind any in-flight refresh instead of
    // skipping and leaving the imported feeds empty until the next tick.
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ =
            scheduler::refresh_all(&app2, None, true, scheduler::RefreshScope::All).await;
    });
    Ok(count)
}

#[tauri::command]
pub async fn export_opml(state: State<'_, AppState>) -> AppResult<String> {
    let conn = state.read().await;
    let feeds = db::feeds_for_export(&conn)?;
    opml::build(&feeds)
}

// ─────────────────────────── settings ───────────────────────────

#[tauri::command]
pub async fn get_setting(state: State<'_, AppState>, key: String) -> AppResult<Option<String>> {
    let conn = state.read().await;
    db::get_setting(&conn, &key)
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::set_setting(&conn, &key, &value)
}

// ─────────────────────────── AI ───────────────────────────

fn purpose_profile_setting_key(purpose: LlmPurpose) -> &'static str {
    match purpose {
        LlmPurpose::Summary => "ai_default_summary_profile_id",
        LlmPurpose::Ask => "ai_default_ask_profile_id",
        LlmPurpose::Digest => "ai_default_digest_profile_id",
        LlmPurpose::Translate => "ai_default_translate_profile_id",
    }
}

/// Load the AI provider configuration from the settings table for one command
/// purpose, using the new profile resolver with legacy settings as fallback.
fn load_ai_config_for(conn: &rusqlite::Connection, purpose: LlmPurpose) -> AppResult<AiConfig> {
    AiConfig::from_settings(
        db::get_setting(conn, "ai_profiles_json")?,
        db::get_setting(conn, "ai_active_profile_id")?,
        db::get_setting(conn, purpose_profile_setting_key(purpose))?,
        db::get_setting(conn, "ai_provider")?,
        db::get_setting(conn, "ai_api_key")?,
        db::get_setting(conn, "ai_model")?,
        db::get_setting(conn, "ai_base_url")?,
        purpose,
    )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConnectionTestResult {
    message: String,
}

fn redact_ai_test_error(message: &str, api_key: &str) -> String {
    let key = api_key.trim();
    if key.is_empty() {
        message.to_string()
    } else {
        message.replace(key, "[redacted]")
    }
}

fn format_ai_test_error(message: &str, api_key: &str) -> String {
    let redacted = redact_ai_test_error(message, api_key);
    let lower = redacted.to_ascii_lowercase();
    if lower.contains("certificate")
        || lower.contains("unknown issuer")
        || lower.contains("untrusted")
        || lower.contains("tls")
    {
        format!("TLS certificate validation failed: {redacted}")
    } else {
        redacted
    }
}

#[tauri::command]
pub async fn test_ai_connection(
    state: State<'_, AppState>,
    profile: ai::LlmProfileInput,
) -> AppResult<AiConnectionTestResult> {
    let api_key = profile.api_key.clone();
    let cfg = AiConfig::from_profile_input(profile)?;
    let http = state.http();
    match ai::complete_chat(
        &http,
        &cfg,
        "You are testing an AI provider connection. Reply with exactly: OK",
        "Reply with exactly: OK",
        ai::MAX_TOKENS,
    )
    .await
    {
        Ok(text) => Ok(AiConnectionTestResult {
            message: truncate(text.trim(), 120),
        }),
        Err(e) => Err(AppError::other(format!(
            "AI connection test failed: {}",
            format_ai_test_error(&e.to_string(), &api_key)
        ))),
    }
}

/// Resolve a translation engine chosen by the caller (the reader's translate
/// switcher) into what it needs. `engine` is one of `google` / `bing` / `deepl`
/// / `llm`; anything else falls back to the LLM. Google, DeepL and Bing are
/// keyless free endpoints; only the LLM path needs credentials (the shared AI
/// provider config). The engine is picked per translation.
fn build_translate_selection(
    conn: &rusqlite::Connection,
    engine: &str,
) -> AppResult<translate::Selection> {
    Ok(match engine {
        "google" => translate::Selection::Google,
        "bing" => translate::Selection::Bing,
        "deepl" => translate::Selection::Deepl,
        _ => translate::Selection::Llm(load_ai_config_for(conn, LlmPurpose::Translate)?),
    })
}

/// Truncate to at most `max` characters without splitting a UTF-8 boundary.
fn truncate(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// A system-prompt directive so AI output matches the UI language rather than
/// defaulting to whatever language the source article happens to be in.
fn response_language(conn: &rusqlite::Connection) -> &'static str {
    match db::get_setting(conn, "language").ok().flatten().as_deref() {
        Some("zh") => "\n\nAlways write your response in Simplified Chinese.",
        Some("ja") => "\n\nAlways write your response in Japanese.",
        _ => "\n\nAlways write your response in English.",
    }
}

/// The article-translation target language code: the dedicated
/// `translate_target_lang` setting, falling back to the UI `language`, then
/// English. Stored as a code (`en` / `zh` / `ja`); `translate::language_name`
/// maps it to the name used in the prompt.
fn translate_target_lang(conn: &rusqlite::Connection) -> String {
    db::get_setting(conn, "translate_target_lang")
        .ok()
        .flatten()
        .or_else(|| db::get_setting(conn, "language").ok().flatten())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "en".to_string())
}

/// Stream an AI summary of one article; the full summary is also persisted.
#[tauri::command]
pub async fn ai_summarize(
    state: State<'_, AppState>,
    article_id: i64,
    template: SummaryTemplate,
    on_token: Channel<AiEvent>,
) -> AppResult<()> {
    let (title, body, cfg, lang) = {
        let conn = state.read().await;
        let (title, body) = db::article_text(&conn, article_id)?;
        (
            title,
            body,
            load_ai_config_for(&conn, LlmPurpose::Summary)?,
            response_language(&conn),
        )
    };
    if body.trim().is_empty() {
        return Err(AppError::code("noArticleBody"));
    }
    let system = build_summary_prompt(template, &lang);
    let user = format!("Title: {title}\n\n{}", truncate(&body, 8000));

    let http = state.http();
    let outcome = ai::stream_chat(&http, &cfg, &system, &user, &on_token, ai::MAX_TOKENS).await?;
    if outcome.completed && !outcome.text.trim().is_empty() {
        let conn = state.db.lock().await;
        db::set_ai_summary(&conn, article_id, outcome.text.trim())?;
    }
    Ok(())
}

/// Build the system prompt for a given summary template.
fn build_summary_prompt(template: SummaryTemplate, lang: &str) -> String {
    match template {
        SummaryTemplate::Classic => format!(
            "You are a sharp news editor. Summarize the article so a reader can \
             decide whether to read it in full.\n\n\
             Format the response in markdown using exactly this shape:\n\
             **TL;DR** — One sentence capturing the single most important point.\n\n\
             - Key fact, finding, or claim (under ~20 words)\n\
             - Another key point\n\
             - 3 to 5 bullets total, one idea each, no nested bullets\n\n\
             Output only this structure. No preamble, no closing remarks, no \
             section headers, no extra prose.{lang}",
            lang = lang
        ),
        SummaryTemplate::News5w1h => format!(
            "You are a senior news editor. Please produce a structured summary \
             that extracts the core facts from the article.\n\n\
             Format the response in markdown using exactly this shape:\n\
             **一句话概述** — One sentence summarizing the core subject.\n\n\
             **5W1H 速览**\n\
             - **Who（谁）**：The people, companies, or organizations involved\n\
             - **What（什么）**：The event, product, or finding\n\
             - **When（何时）**：Time, date, or version cycle\n\
             - **Where（何地）**：Location, platform, or scope\n\
             - **Why（为什么）**：Motivation, background, or cause\n\
             - **How（如何）**：Method, approach, or impact path\n\n\
             **关键细节**：1–3 pieces of essential information or data from the article.\n\n\
             **价值判断**：For the target reader, is this article worth reading in full? \
             One sentence.\n\n\
             Output only this structure. No preamble, no extra prose.{lang}",
            lang = lang
        ),
        SummaryTemplate::Decision => format!(
            "You are an efficient information-filtering assistant. Your job is \
             not to retell the article, but to help the reader decide in 5 \
             seconds whether it is worth reading.\n\n\
             Format the response in markdown using exactly this shape:\n\
             **核心命题** — One sentence: what problem or viewpoint does the \
             article address?\n\n\
             **适合谁读** — Which type of reader will find this most valuable?\n\n\
             **为什么现在读** — What is the timeliness or unique value?\n\n\
             **值不值得读** — High quality, worth a careful read / Clickbait, \
             the summary is enough / Useful only for a specific audience\n\n\
             **如果只看一句话**：The single most memorable sentence or conclusion.\n\n\
             Output only this structure. No extra prose.{lang}",
            lang = lang
        ),
        SummaryTemplate::Funnel => format!(
            "You are an information architect. Compress the article into a \
             three-level progressive summary, from coarse to fine.\n\n\
             Format the response in markdown using exactly this shape:\n\
             **第一层：30 秒速览** (≤ 30 words)\n\
             One extremely short sentence stating the core conclusion.\n\n\
             **第二层：2 分钟精华** (3–4 bullets)\n\
             The most important evidence or findings, each ≤ 25 words.\n\n\
             **第三层：深度线索** (1–2 items)\n\
             If the reader wants to go deeper, which sections of the article \
             should they focus on?\n\n\
             **适合场景**：When is this article best read? (e.g. commute skim, \
             technical research, weekend deep-dive)\n\n\
             Output only this structure. No extra prose.{lang}",
            lang = lang
        ),
        SummaryTemplate::Argument => format!(
            "You are a logic analyst. Deconstruct the article's core argument \
             structure so the reader can quickly grasp the author's reasoning.\n\n\
             Format the response in markdown using exactly this shape:\n\
             **作者的核心观点** — The one thing the author most wants you to believe.\n\n\
             **主要论据** — 2–4 key pieces of evidence or reasoning steps the \
             author uses.\n\n\
             **潜在前提** — What unstated assumptions does the argument rely on?\n\n\
             **不同视角** — What would a skeptic or opponent question about this article?\n\n\
             **我的判断** — Is the argument solid? Worth trusting, or take it \
             with a grain of salt?\n\n\
             Output only this structure. No extra prose.{lang}",
            lang = lang
        ),
        SummaryTemplate::Minimal => format!(
            "You are an ultra-concise summary editor. Reduce the article to a \
             single sentence, at most 50 words.\n\n\
             Requirements:\n\
             - Must include the core conclusion or key fact\n\
             - Do NOT start with 'This article discusses...' or similar empty phrases\n\
             - Deliver the substance directly\n\n\
             If the article is extremely short or lacks substance, output exactly:\n\
            「内容较浅，建议跳过。」\n\n\
             Output only that one sentence. No other text.{lang}",
            lang = lang
        ),
    }
}

/// Answer a question using the user's subscribed articles as RAG context.
/// Retrieval currently uses FTS5 keyword search (semantic search is Phase 5).
#[tauri::command]
pub async fn ai_ask(
    state: State<'_, AppState>,
    question: String,
    on_token: Channel<AiEvent>,
) -> AppResult<()> {
    let (cfg, context, lang) = {
        let conn = state.read().await;
        let cfg = load_ai_config_for(&conn, LlmPurpose::Ask)?;
        // RAG retrieval is recall-oriented: match articles that share *any* of
        // the question's keywords. `list_articles` AND-joins every search word,
        // which for a natural-language question matches nothing.
        let hits = db::search_articles_for_rag(&conn, &question, 6)?;
        let mut context = String::new();
        for (id, _title, feed_title) in hits {
            let (title, body) = db::article_text(&conn, id)?;
            context.push_str(&format!(
                "## {} — {}\n{}\n\n",
                title,
                feed_title,
                truncate(&body, 1200)
            ));
        }
        (cfg, context, response_language(&conn))
    };

    let system = format!(
        "You answer the user's question using only the provided \
         articles from their RSS subscriptions. Cite the article \
         titles you draw from. If the articles do not contain the \
         answer, say so plainly.{lang}"
    );
    let user = if context.trim().is_empty() {
        format!("No relevant articles were found.\n\nQuestion: {question}")
    } else {
        format!("Articles from the user's feeds:\n\n{context}---\n\nQuestion: {question}")
    };

    let http = state.http();
    ai::stream_chat(&http, &cfg, &system, &user, &on_token, ai::MAX_TOKENS).await?;
    Ok(())
}

/// Answer follow-up questions about an already-generated article summary.
/// The context is limited to the summary text plus the current Q&A history;
/// the original article body is not re-fetched.
#[tauri::command]
pub async fn ai_summarize_follow_up(
    state: State<'_, AppState>,
    summary_text: String,
    history: Vec<(String, String)>,
    question: String,
    on_token: Channel<AiEvent>,
) -> AppResult<()> {
    let (cfg, lang) = {
        let conn = state.read().await;
        (
            load_ai_config_for(&conn, LlmPurpose::Summary)?,
            response_language(&conn),
        )
    };

    let system = format!(
        "You are a helpful reading assistant. The user is looking at an \
         AI-generated article summary and wants to ask follow-up questions. \
         Answer using only the information present in the summary. If the \
         question goes beyond the summary, say so plainly. Keep answers \
         concise.{lang}",
        lang = lang
    );

    let mut user = format!("Summary:\n\n{}\n\n---\n\n", summary_text);
    for (q, a) in history {
        user.push_str(&format!("Q: {}\nA: {}\n\n", q, a));
    }
    user.push_str(&format!("Q: {}\nA:", question));

    let http = state.http();
    ai::stream_chat(&http, &cfg, &system, &user, &on_token, ai::MAX_TOKENS).await?;
    Ok(())
}

/// Stream an AI briefing that synthesizes the most recent articles by theme.
#[tauri::command]
pub async fn ai_digest(
    state: State<'_, AppState>,
    on_token: Channel<AiEvent>,
) -> AppResult<()> {
    let (cfg, articles, lang) = {
        let conn = state.read().await;
        (
            load_ai_config_for(&conn, LlmPurpose::Digest)?,
            db::digest_source(&conn, 30)?,
            response_language(&conn),
        )
    };
    if articles.is_empty() {
        return Err(AppError::code("noArticles"));
    }

    let mut corpus = String::new();
    for (title, feed, text) in &articles {
        corpus.push_str(&format!("- [{feed}] {title}: {}\n", truncate(text, 400)));
    }

    let system = format!(
        "You are the user's personal news briefer. From the recent \
         articles, write a crisp briefing: group related items into \
         2-4 themed sections with short headers, lead with what \
         matters most, and keep it skimmable. Plain prose, no preamble.{lang}"
    );
    let user = format!("Recent articles from my feeds:\n\n{corpus}");

    let http = state.http();
    ai::stream_chat(&http, &cfg, &system, &user, &on_token, ai::MAX_TOKENS).await?;
    Ok(())
}

/// Progress events streamed to the frontend during a translation. Reported once
/// per batch (a group of whole blocks), never per token: token-level IPC across
/// a full article would flood the webview's main thread and freeze the UI.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "data")]
pub enum TranslateEvent {
    /// The number of batches the body was split into, sent before any batch.
    Start { total: usize },
    /// One freshly translated, sanitized batch of HTML, plus how many batches
    /// have completed so far.
    Batch { html: String, done: usize },
    /// The full sanitized translation, sent once on completion.
    Done { html: String },
}

/// Translate one article's body into the configured target language, reporting
/// progress per batch over `on_event`. The body is split into batches of whole
/// blocks (so long articles and the per-request token cap are both handled) and
/// translated batch by batch with the HTML structure preserved. The reassembled,
/// sanitized result is cached on the article row and reused on the next open.
///
/// The work runs to completion independently of the reader view, so the frontend
/// can start several translations at once and switch articles without
/// interrupting any of them.
#[tauri::command]
pub async fn ai_translate(
    state: State<'_, AppState>,
    article_id: i64,
    lang: String,
    engine: String,
    on_event: Channel<TranslateEvent>,
) -> AppResult<()> {
    let (source_html, sel, target) = {
        let conn = state.read().await;
        let detail = db::get_article(&conn, article_id)?;
        // Translate the richest body available: the extracted full text when the
        // user has run extraction, otherwise the feed's own HTML.
        let source = detail
            .extracted_html
            .filter(|s| !s.trim().is_empty())
            .or(detail.content_html)
            .unwrap_or_default();
        // The reader picks the target language and engine per translation; fall
        // back to the stored default only if the caller sent none.
        let target = if lang.trim().is_empty() {
            translate_target_lang(&conn)
        } else {
            lang
        };
        (source, build_translate_selection(&conn, &engine)?, target)
    };
    if source_html.trim().is_empty() {
        return Err(AppError::code("noArticleBody"));
    }

    let http = state.http();
    // Resolve the chosen engine (fetching Bing's auth token, if selected) before
    // the loop, so a credential or network failure surfaces before any progress
    // is reported rather than mid-stream.
    let backend = translate::ready(&http, sel).await?;

    // Split by an engine-specific budget so a short article translates in one
    // request and a long one is chunked to fit the engine's per-call limits.
    let batches = translate::chunk_blocks(&source_html, translate::chunk_budget(&engine));
    let total = batches.len();
    let _ = on_event.send(TranslateEvent::Start { total });

    let system = translate::translate_system_prompt(translate::language_name(&target));
    let mut full = String::new();
    for (i, batch) in batches.iter().enumerate() {
        let raw = backend.translate_batch(&http, &system, batch, &target).await?;
        // Engine output (LLM or machine translation) is untrusted, so each batch
        // passes through the same sanitizer as feed HTML before it reaches the
        // webview or the database. Source URLs are already absolute (sanitized at
        // ingestion), so no base is needed.
        let clean = sanitize::sanitize(raw.trim(), None);
        full.push_str(&clean);
        full.push('\n');
        let _ = on_event.send(TranslateEvent::Batch { html: clean, done: i + 1 });
    }

    let final_html = full.trim().to_string();
    if !final_html.is_empty() {
        let conn = state.db.lock().await;
        db::set_translation(&conn, article_id, &final_html, &target)?;
    }
    let _ = on_event.send(TranslateEvent::Done { html: final_html });
    Ok(())
}

// ─────────────────────────── storage ───────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageStats {
    db_bytes: i64,
    article_count: i64,
    feed_count: i64,
}

#[tauri::command]
pub async fn storage_stats(state: State<'_, AppState>) -> AppResult<StorageStats> {
    let conn = state.read().await;
    let (db_bytes, article_count, feed_count) = db::storage_stats(&conn)?;
    Ok(StorageStats {
        db_bytes,
        article_count,
        feed_count,
    })
}

/// Delete read articles older than `days` (starred / read-later are kept).
#[tauri::command]
pub async fn cleanup_articles(app: AppHandle, days: i64) -> AppResult<usize> {
    let n = {
        let state = app.state::<AppState>();
        let conn = state.db.lock().await;
        db::cleanup_old_articles(&conn, days)?
    };
    let _ = app.emit("feeds-updated", 0);
    Ok(n)
}

/// Reclaim free database pages.
#[tauri::command]
pub async fn vacuum_db(state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::vacuum(&conn)
}

/// Clear every stored setting (AI keys, sync credentials, preferences).
#[tauri::command]
pub async fn reset_settings(state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::reset_settings(&conn)
}

/// Delete all feeds, folders and articles. Irreversible.
#[tauri::command]
pub async fn clear_all_data(app: AppHandle) -> AppResult<()> {
    {
        let state = app.state::<AppState>();
        let conn = state.db.lock().await;
        db::clear_all_data(&conn)?;
    }
    let _ = app.emit("feeds-updated", 0);
    refresh_unread_surfaces(&app).await;
    Ok(())
}

// ─────────────────────────── network ───────────────────────────

/// Rebuild the HTTP client from the persisted proxy / timeout settings so the
/// change takes effect without an app restart.
#[tauri::command]
pub async fn apply_network_settings(state: State<'_, AppState>) -> AppResult<()> {
    let client = {
        // Pure settings read — use the read pool, not the writer lock.
        let conn = state.read().await;
        fetch::build_client_from_settings(&conn)
    };
    state.set_http(client);
    Ok(())
}

// ─────────────────────────── FreshRSS sync ───────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FreshRssStatus {
    connected: bool,
    url: Option<String>,
    /// Which GReader-compatible backend is connected: "freshrss" or
    /// "miniflux". Always present (defaults to "freshrss") so the UI never
    /// has to guess for older installs.
    provider: String,
}

#[tauri::command]
pub async fn freshrss_connect(
    app: AppHandle,
    url: String,
    username: String,
    password: String,
    provider: Option<String>,
) -> AppResult<()> {
    crate::sync::connect(&app, &url, &username, &password, provider.as_deref()).await
}

#[tauri::command]
pub async fn freshrss_disconnect(app: AppHandle) -> AppResult<()> {
    crate::sync::disconnect(&app).await
}

#[tauri::command]
pub async fn freshrss_status(app: AppHandle) -> AppResult<FreshRssStatus> {
    let info = crate::sync::connected_url(&app).await?;
    let (url, provider) = match info {
        Some((u, p)) => (Some(u), p),
        None => (None, "freshrss".to_string()),
    };
    Ok(FreshRssStatus {
        connected: url.is_some(),
        url,
        provider,
    })
}

/// Run a full FreshRSS sync now; returns the number of reconciled articles.
#[tauri::command]
pub async fn freshrss_sync(app: AppHandle) -> AppResult<usize> {
    let n = crate::sync::sync_now(&app).await?;
    let _ = app.emit("feeds-updated", 0);
    refresh_unread_surfaces(&app).await;
    Ok(n)
}

/// Rebuild the tray menu — used after a language change.
#[tauri::command]
pub async fn refresh_tray(app: AppHandle) -> AppResult<()> {
    crate::tray::refresh(&app).await;
    Ok(())
}

/// Drain a `papr://subscribe` URL that was delivered before the webview could
/// receive the `deep-link-subscribe` event (a cold-start launch). The frontend
/// calls this once on mount; returns `None` when there is nothing pending.
#[tauri::command]
pub async fn take_pending_deep_link(state: State<'_, AppState>) -> AppResult<Option<String>> {
    Ok(state.take_pending_deep_link())
}

// ─────────────────────────── tags ───────────────────────────

#[tauri::command]
pub async fn list_tags(state: State<'_, AppState>) -> AppResult<Vec<Tag>> {
    let conn = state.read().await;
    db::list_tags(&conn)
}

#[tauri::command]
pub async fn create_tag(state: State<'_, AppState>, name: String) -> AppResult<i64> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::code("emptyTagName"));
    }
    let conn = state.db.lock().await;
    db::create_tag(&conn, name)
}

#[tauri::command]
pub async fn rename_tag(state: State<'_, AppState>, id: i64, name: String) -> AppResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::code("emptyTagName"));
    }
    let conn = state.db.lock().await;
    db::rename_tag(&conn, id, name)
}

#[tauri::command]
pub async fn set_tag_color(
    state: State<'_, AppState>,
    id: i64,
    color: String,
) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::set_tag_color(&conn, id, &color)
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::delete_tag(&conn, id)
}

/// Attach or detach a tag from one article.
#[tauri::command]
pub async fn set_article_tag(
    state: State<'_, AppState>,
    article_id: i64,
    tag_id: i64,
    on: bool,
) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::set_article_tag(&conn, article_id, tag_id, on)
}

// ─────────────────────────── filter rules ───────────────────────────

#[tauri::command]
pub async fn list_rules(state: State<'_, AppState>) -> AppResult<Vec<Rule>> {
    let conn = state.read().await;
    db::list_rules(&conn)
}

#[tauri::command]
pub async fn create_rule(
    state: State<'_, AppState>,
    name: String,
    feed_id: Option<i64>,
    field: String,
    query: String,
    action: String,
) -> AppResult<i64> {
    if query.trim().is_empty() {
        return Err(AppError::code("emptyRuleQuery"));
    }
    let conn = state.db.lock().await;
    db::create_rule(&conn, name.trim(), feed_id, &field, query.trim(), &action)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_rule(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    enabled: bool,
    feed_id: Option<i64>,
    field: String,
    query: String,
    action: String,
) -> AppResult<()> {
    if query.trim().is_empty() {
        return Err(AppError::code("emptyRuleQuery"));
    }
    let conn = state.db.lock().await;
    db::update_rule(&conn, id, name.trim(), enabled, feed_id, &field, query.trim(), &action)
}

#[tauri::command]
pub async fn delete_rule(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::delete_rule(&conn, id)
}

/// Apply a rule's action to the already-stored articles it matches and return
/// how many were acted on. The frontend calls this right after saving a rule so
/// an enabled rule affects the existing backlog, not just future articles. A
/// `skip` rule deletes its matches, so the UI confirms before invoking this.
#[tauri::command]
pub async fn apply_rule_to_existing(
    state: State<'_, AppState>,
    feed_id: Option<i64>,
    field: String,
    query: String,
    action: String,
) -> AppResult<usize> {
    if query.trim().is_empty() {
        return Err(AppError::code("emptyRuleQuery"));
    }
    let conn = state.db.lock().await;
    db::apply_rule_to_existing(&conn, feed_id, &field, query.trim(), &action)
}

/// Persist a reordered tag list (ids in the new display order).
#[tauri::command]
pub async fn reorder_tags(state: State<'_, AppState>, ids: Vec<i64>) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::reorder_tags(&conn, &ids)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RulePreview {
    count: i64,
    samples: Vec<String>,
}

/// Dry-run a draft rule against already-stored articles.
#[tauri::command]
pub async fn preview_rule(
    state: State<'_, AppState>,
    feed_id: Option<i64>,
    field: String,
    query: String,
) -> AppResult<RulePreview> {
    let conn = state.read().await;
    let (count, samples) = db::preview_rule(&conn, feed_id, &field, query.trim())?;
    Ok(RulePreview { count, samples })
}

// ─────────────────────────── newsletter sources ───────────────────────────

/// A configured email-newsletter source, as shown in the UI (no password).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsletterSource {
    feed_id: i64,
    title: String,
    host: String,
    port: u16,
    username: String,
    folder: String,
}

/// Payload for `add_newsletter_source` — the IMAP mailbox to start polling.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsletterInput {
    /// A display name for the source (falls back to the username).
    pub title: Option<String>,
    pub host: String,
    pub port: u16,
    pub username: String,
    /// IMAP app-password / token. Stored in the local DB only.
    pub password: String,
    /// Mailbox to poll, e.g. `INBOX` or `Newsletters`.
    pub folder: String,
}

/// Add an email-newsletter source. Verifies the IMAP credentials by polling
/// the mailbox once, ingests whatever it finds, and persists the source so the
/// background scheduler keeps polling it. Backed by a `feeds` row plus an
/// entry in `newsletter_sources` (see migration #10).
#[tauri::command]
pub async fn add_newsletter_source(
    state: State<'_, AppState>,
    input: NewsletterInput,
) -> AppResult<Feed> {
    let cfg = NewsletterConfig {
        host: input.host.trim().to_string(),
        port: input.port,
        username: input.username.trim().to_string(),
        password: input.password.clone(),
        folder: {
            let f = input.folder.trim();
            if f.is_empty() { "INBOX".to_string() } else { f.to_string() }
        },
    };
    if cfg.host.is_empty() || cfg.username.is_empty() || cfg.password.is_empty() {
        return Err(AppError::code("newsletterMissingFields"));
    }
    let feed_url = newsletter::synthetic_feed_url(&cfg);
    let title = input
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(String::from)
        .unwrap_or_else(|| cfg.username.clone());

    // Reject a duplicate mailbox before doing the (slow) IMAP round-trip.
    {
        let conn = state.read().await;
        if db::find_feed_by_url(&conn, &feed_url)?.is_some() {
            return Err(AppError::code("alreadySubscribed"));
        }
    }

    // Verify the credentials by actually connecting. The `imap` crate is
    // blocking, so the connection runs on the blocking pool — and it has no
    // per-operation timeout, so a server that completes the TCP/TLS handshake
    // but then stalls mid-command would block this command forever (the Add
    // dialog spinner never resolves, the blocking worker thread is leaked).
    // Bound the whole probe with the same wall-clock cap the scheduler's
    // background poll uses, so a wedged mailbox degrades to a clean error.
    let probe_cfg = cfg.clone();
    let messages = match tokio::time::timeout(
        std::time::Duration::from_secs(scheduler::NEWSLETTER_POLL_TIMEOUT_SECS),
        tokio::task::spawn_blocking(move || newsletter::fetch_recent(&probe_cfg, 30)),
    )
    .await
    {
        Ok(joined) => joined
            .map_err(|e| AppError::other(format!("newsletter poll task: {e}")))??,
        Err(_) => return Err(AppError::code("newsletterPollTimeout")),
    };

    // Persist the source, then ingest the messages just fetched.
    let conn = state.db.lock().await;
    let feed_id = db::insert_newsletter_source(&conn, &feed_url, &title, &cfg)?;
    let rules = db::active_rules(&conn).unwrap_or_default();
    // `upsert_article` returns `true` only for genuinely new *unread* rows, so
    // articles a `read` rule pre-marked read are correctly excluded from the
    // returned `unread_count` (matching the sidebar's `list_feeds` count).
    let mut unread = 0i64;
    for raw in &messages {
        if let Some(parsed) = newsletter::email_to_article(raw) {
            if db::upsert_article(&conn, feed_id, &parsed.article, false, &rules)? {
                unread += 1;
            }
        }
    }
    // Record that the mailbox was just polled. The IMAP fetch above is a
    // genuine, successful refresh of this source — without this the feed's
    // `last_fetched_at` stays NULL and the sidebar reads it as "never
    // refreshed" until the next scheduler tick (up to the refresh interval
    // away). Mirrors `touch_feed` in `scheduler::poll_newsletters` for the
    // background poll, and the same handling `add_feed` applies.
    let _ = db::touch_feed(&conn, feed_id);
    let last_fetched_at = db::feed_last_fetched(&conn, feed_id).ok().flatten();
    drop(conn);

    Ok(Feed {
        id: feed_id,
        feed_url,
        site_url: None,
        title,
        description: None,
        favicon_url: None,
        folder_id: None,
        source_type: SourceType::Newsletter.as_str().to_string(),
        last_fetched_at,
        fetch_error: None,
        unread_count: unread,
        refresh_interval_min: None,
    })
}

/// Every configured newsletter source (passwords omitted).
#[tauri::command]
pub async fn list_newsletter_sources(
    state: State<'_, AppState>,
) -> AppResult<Vec<NewsletterSource>> {
    let conn = state.read().await;
    Ok(db::list_newsletter_sources(&conn)?
        .into_iter()
        .map(|r| NewsletterSource {
            feed_id: r.feed_id,
            title: r.title,
            host: r.host,
            port: r.port,
            username: r.username,
            folder: r.folder,
        })
        .collect())
}

/// Remove a newsletter source and all of its ingested articles.
#[tauri::command]
pub async fn remove_newsletter_source(
    state: State<'_, AppState>,
    feed_id: i64,
) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::delete_newsletter_source(&conn, feed_id)
}

// ─────────────────────────── highlights (F7) ───────────────────────────

/// Create a highlight on an article. The frontend supplies the quote plus its
/// anchoring context (prefix / suffix window and the plain-text offset).
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn create_highlight(
    state: State<'_, AppState>,
    article_id: i64,
    quote: String,
    prefix: String,
    suffix: String,
    text_offset: i64,
    color: String,
    note: String,
) -> AppResult<i64> {
    if quote.trim().is_empty() {
        return Err(AppError::code("emptyHighlight"));
    }
    let conn = state.db.lock().await;
    db::insert_highlight(
        &conn,
        &db::NewHighlight {
            article_id,
            quote: &quote,
            prefix: &prefix,
            suffix: &suffix,
            text_offset,
            color: &color,
            note: &note,
        },
    )
}

/// Every highlight on one article, in reading order.
#[tauri::command]
pub async fn list_highlights(
    state: State<'_, AppState>,
    article_id: i64,
) -> AppResult<Vec<Highlight>> {
    let conn = state.read().await;
    db::list_highlights(&conn, article_id)
}

/// Every highlight across all articles — for the Highlights browser.
#[tauri::command]
pub async fn list_all_highlights(state: State<'_, AppState>) -> AppResult<Vec<Highlight>> {
    let conn = state.read().await;
    db::list_all_highlights(&conn)
}

/// Replace a highlight's note (an empty string clears it).
#[tauri::command]
pub async fn update_highlight_note(
    state: State<'_, AppState>,
    id: i64,
    note: String,
) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::update_highlight_note(&conn, id, &note)
}

/// Change a highlight's colour (a palette key).
#[tauri::command]
pub async fn set_highlight_color(
    state: State<'_, AppState>,
    id: i64,
    color: String,
) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::set_highlight_color(&conn, id, &color)
}

#[tauri::command]
pub async fn delete_highlight(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let conn = state.db.lock().await;
    db::delete_highlight(&conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_conn() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn
    }

    fn put_setting(conn: &rusqlite::Connection, key: &str, value: &str) {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )
        .unwrap();
    }

    fn two_profile_json(active_key: &str, translate_key: &str) -> String {
        format!(
            r#"{{
                "version": 1,
                "profiles": [
                    {{
                        "id": "active",
                        "name": "Active",
                        "protocol": "openai_chat_completions",
                        "base_url": "https://active.test/v1",
                        "api_key": "{active_key}",
                        "model": "active-model",
                        "enabled": true,
                        "auth": "bearer"
                    }},
                    {{
                        "id": "translate",
                        "name": "Translate",
                        "protocol": "openai_chat_completions",
                        "base_url": "https://translate.test/v1",
                        "api_key": "{translate_key}",
                        "model": "translate-model",
                        "enabled": true,
                        "auth": "bearer"
                    }}
                ]
            }}"#
        )
    }

    fn assert_err_code<T>(result: AppResult<T>, code: &str) {
        match result {
            Ok(_) => panic!("expected {code} error"),
            Err(e) => assert!(e.to_string().contains(code), "unexpected error: {e}"),
        }
    }

    #[test]
    fn ai_connection_test_error_redacts_api_key() {
        let msg = redact_ai_test_error(
            "upstream rejected Authorization: Bearer sk-secret-token",
            "sk-secret-token",
        );

        assert!(!msg.contains("sk-secret-token"));
        assert!(msg.contains("[redacted]"));
    }

    #[test]
    fn load_ai_config_for_missing_purpose_profile_falls_back_to_active_profile() {
        let conn = settings_conn();
        put_setting(
            &conn,
            "ai_profiles_json",
            r#"{
                "version": 1,
                "profiles": [
                    {
                        "id": "first",
                        "name": "First",
                        "protocol": "openai_chat_completions",
                        "base_url": "https://first.test/v1",
                        "api_key": "sk-first",
                        "model": "first-model",
                        "enabled": true,
                        "auth": "bearer"
                    },
                    {
                        "id": "active",
                        "name": "Active",
                        "protocol": "openai_chat_completions",
                        "base_url": "https://active.test/v1",
                        "api_key": "   ",
                        "model": "active-model",
                        "enabled": true,
                        "auth": "bearer"
                    }
                ]
            }"#,
        );
        put_setting(&conn, "ai_active_profile_id", "active");
        put_setting(&conn, "ai_default_summary_profile_id", "missing-summary");

        assert_err_code(
            load_ai_config_for(&conn, ai::LlmPurpose::Summary),
            "noAiKey",
        );
    }

    #[test]
    fn load_ai_config_for_prefers_purpose_profile_before_active_profile() {
        let conn = settings_conn();
        put_setting(
            &conn,
            "ai_profiles_json",
            &two_profile_json("sk-active", "   "),
        );
        put_setting(&conn, "ai_active_profile_id", "active");
        put_setting(&conn, "ai_default_translate_profile_id", "translate");

        assert_err_code(
            load_ai_config_for(&conn, ai::LlmPurpose::Translate),
            "noAiKey",
        );
    }

    #[test]
    fn load_ai_config_for_without_profiles_json_falls_back_to_legacy_settings() {
        let conn = settings_conn();
        put_setting(&conn, "ai_provider", "openai");
        put_setting(&conn, "ai_api_key", "sk-legacy");

        assert!(load_ai_config_for(&conn, ai::LlmPurpose::Ask).is_ok());
    }

    #[test]
    fn build_translate_selection_keyless_engines_do_not_require_ai_settings() {
        let conn = settings_conn();

        assert!(matches!(
            build_translate_selection(&conn, "google").unwrap(),
            translate::Selection::Google
        ));
        assert!(matches!(
            build_translate_selection(&conn, "bing").unwrap(),
            translate::Selection::Bing
        ));
        assert!(matches!(
            build_translate_selection(&conn, "deepl").unwrap(),
            translate::Selection::Deepl
        ));
    }

    #[test]
    fn build_translate_selection_llm_uses_translate_profile() {
        let conn = settings_conn();
        put_setting(
            &conn,
            "ai_profiles_json",
            &two_profile_json("sk-active", "   "),
        );
        put_setting(&conn, "ai_active_profile_id", "active");
        put_setting(&conn, "ai_default_translate_profile_id", "translate");

        assert_err_code(build_translate_selection(&conn, "llm"), "noAiKey");
    }

    // --- referer_candidates: the Referer fallback chain for image fetches. ---

    #[test]
    fn referer_candidates_tries_bare_then_origin_then_page() {
        // Order matters: no Referer defeats blacklist-style protection
        // (*.sinaimg.cn), the image's own origin satisfies hosts that demand a
        // Referer be present (cdnfile.sspai.com), and the article URL is what
        // a strict whitelist CDN expects.
        let got = referer_candidates(
            "https://cdnfile.sspai.com/2025/a.png?imageMogr2/thumbnail",
            Some("https://sspai.com/post/110992"),
        );
        assert_eq!(
            got,
            vec![
                None,
                Some("https://cdnfile.sspai.com/".to_string()),
                Some("https://sspai.com/post/110992".to_string()),
            ]
        );
    }

    #[test]
    fn referer_candidates_without_page_url() {
        let got = referer_candidates("https://wx1.sinaimg.cn/large/a.jpg", None);
        assert_eq!(got, vec![None, Some("https://wx1.sinaimg.cn/".to_string())]);
    }

    #[test]
    fn referer_candidates_skips_non_http_page_url() {
        // A feed can ship anything as the article link; only an http(s) URL
        // is a plausible Referer.
        let got = referer_candidates("https://ex.com/a.png", Some("mailto:editor@ex.com"));
        assert_eq!(got, vec![None, Some("https://ex.com/".to_string())]);
    }

    #[test]
    fn referer_candidates_dedupes_page_equal_to_origin() {
        // Article link identical to the image origin would be a wasted retry.
        let got = referer_candidates("https://ex.com/a.png", Some("https://ex.com/"));
        assert_eq!(got, vec![None, Some("https://ex.com/".to_string())]);
    }
}
