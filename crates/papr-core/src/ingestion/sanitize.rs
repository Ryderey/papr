//! HTML sanitization and text extraction. Every piece of feed- or web-supplied
//! HTML passes through `sanitize` before it is ever stored or rendered.

use ammonia::{Builder, UrlRelative};
use lol_html::{element, rewrite_str, RewriteStrSettings};
use scraper::{Html, Selector};
use std::sync::LazyLock;
use url::Url;

/// Sanitize untrusted HTML for safe rendering inside the reader webview.
/// Relative URLs are rewritten against `base` so feed images/links resolve.
pub fn sanitize(html: &str, base: Option<&str>) -> String {
    // Recover lazy-loaded image URLs before ammonia runs.
    let html = promote_lazy_images(html);

    let mut builder = Builder::default();
    builder
        .link_rel(Some("noopener noreferrer nofollow"))
        .add_generic_attributes(["loading"])
        .add_tags(["video", "source"])
        .add_tag_attributes(
            "video",
            ["src", "poster", "width", "height", "preload", "loop", "muted", "playsinline"],
        )
        .add_tag_attributes("source", ["src", "type", "media"])
        .set_tag_attribute_value("video", "controls", "")
        .set_tag_attribute_value("img", "referrerpolicy", "no-referrer");

    let parsed_base = base.and_then(|b| Url::parse(b).ok());
    if let Some(b) = parsed_base {
        builder.url_relative(UrlRelative::RewriteWithBase(b));
    }
    builder.clean(&html).to_string()
}

/// Promote a lazy-loaded image's real URL into `src` so it survives `sanitize`.
fn promote_lazy_images(html: &str) -> String {
    let handler = element!("img", |el| {
        let has_real_src = el.get_attribute("src").is_some_and(|s| {
            let s = s.trim();
            !s.is_empty() && !s.starts_with("data:")
        });
        if !has_real_src {
            let recovered = [
                "data-src",
                "data-original",
                "data-actualsrc",
                "data-lazy-src",
            ]
            .iter()
            .find_map(|a| el.get_attribute(a))
            .or_else(|| {
                el.get_attribute("srcset").and_then(|ss| {
                    ss.split(',')
                        .next()
                        .and_then(|c| c.split_whitespace().next())
                        .map(str::to_string)
                })
            });
            if let Some(url) = recovered {
                let url = url.trim();
                if !url.is_empty() {
                    let _ = el.set_attribute("src", url);
                }
            }
        }
        Ok(())
    });
    rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![handler],
            ..Default::default()
        },
    )
    .unwrap_or_else(|_| html.to_string())
}

/// Tags whose text content is dropped wholesale.
const SKIP_TAGS: &[&str] = &["script", "style", "template", "noscript"];

/// Block-level tags: their edges are word boundaries.
const BLOCK_TAGS: &[&str] = &[
    "address", "article", "aside", "blockquote", "br", "caption", "dd", "div",
    "dl", "dt", "figcaption", "figure", "footer", "h1", "h2", "h3", "h4", "h5",
    "h6", "header", "hr", "li", "main", "nav", "ol", "p", "pre", "section",
    "table", "td", "th", "tr", "ul",
];

/// Strip all markup from HTML, yielding collapsed plain text.
pub fn html_to_text(html: &str) -> String {
    let frag = Html::parse_fragment(html);
    let mut out = String::new();
    let mut skip = 0u32;
    for node in frag.tree.root().descendants() {
        use scraper::node::Node;
        match node.value() {
            Node::Element(el) => {
                let name = el.name();
                if SKIP_TAGS.contains(&name) {
                    skip += 1;
                } else if skip == 0 && BLOCK_TAGS.contains(&name) {
                    out.push(' ');
                }
            }
            Node::Text(t) if skip == 0 => out.push_str(&t.text),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

static IMG_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("img").expect("img is a valid selector"));

/// The first usable image URL embedded in a block of (already-sanitized) HTML.
pub fn first_image(html: &str) -> Option<String> {
    let frag = Html::parse_fragment(html);
    frag.select(&IMG_SELECTOR).find_map(|el| {
        let src = el.value().attr("src")?.trim();
        (src.starts_with("http://") || src.starts_with("https://")).then(|| src.to_string())
    })
}

/// HTML-escape a string for safe interpolation into element text or attribute.
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
