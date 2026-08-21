//! Platform-neutral full-text extraction for the mobile and desktop adapters.

use std::sync::LazyLock;

use dom_smoothie::Readability;
use scraper::{Html, Selector};
use url::Url;

use crate::error::{CoreError, ErrorCategory};
use crate::ingestion::sanitize;

static LEAD_IMAGE_SELECTORS: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse(
        r#"meta[property="og:image"], meta[name="og:image"],
           meta[property="twitter:image"], meta[name="twitter:image"],
           meta[itemprop="image"], link[rel="image_src"]"#,
    )
    .expect("lead image selector is valid")
});

/// Extract and sanitize the main article body from a complete HTML page.
/// `dom_smoothie::Readability` is not `Send`; callers run this in
/// `spawn_blocking` and move only the input/output strings across the task.
pub fn extract_article(html: &str, url: &str) -> Result<String, CoreError> {
    let mut readability = Readability::new(html, Some(url), None)
        .map_err(|e| CoreError::Parse(format!("readability init: {e}")))?;
    let article = readability
        .parse()
        .map_err(|e| CoreError::Parse(format!("readability parse: {e}")))?;
    let content = article.content.to_string();
    if content.trim().is_empty() {
        return Err(CoreError::coded(
            ErrorCategory::Parse,
            "noExtractableContent",
            None,
        ));
    }
    Ok(sanitize::sanitize(&content, Some(url)))
}

/// Read a page-level lead image and resolve it against the final response URL.
pub fn lead_image(html: &str, base: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    doc.select(&LEAD_IMAGE_SELECTORS).find_map(|element| {
        let raw = element
            .value()
            .attr("content")
            .or_else(|| element.value().attr("href"))?
            .trim();
        resolve_http_url(raw, base)
    })
}

fn resolve_http_url(raw: &str, base: &str) -> Option<String> {
    if raw.is_empty() || raw.starts_with("data:") {
        return None;
    }
    let url = Url::parse(raw)
        .or_else(|_| Url::parse(base).and_then(|base| base.join(raw)))
        .ok()?;
    matches!(url.scheme(), "http" | "https").then(|| url.to_string())
}

#[cfg(test)]
mod tests {
    use super::{extract_article, lead_image};

    #[test]
    fn extraction_sanitizes_untrusted_markup() {
        let html = r#"
            <html><body><article>
              <h1>Useful title</h1>
              <p>This is a sufficiently useful article paragraph for extraction.</p>
              <script>alert('x')</script>
            </article></body></html>
        "#;
        let extracted = extract_article(html, "https://example.com/post").unwrap();
        assert!(extracted.contains("Useful title") || extracted.contains("useful article"));
        assert!(!extracted.contains("<script"));
    }

    #[test]
    fn lead_image_resolves_relative_metadata_url() {
        let html = r#"<meta property="og:image" content="/images/lead.jpg">"#;
        assert_eq!(
            lead_image(html, "https://example.com/posts/1").as_deref(),
            Some("https://example.com/images/lead.jpg")
        );
    }
}
