//! OPML import/export — the portable subscription-list format every reader
//! supports, so users can migrate in from (and out to) Reeder, Feedly, etc.
//!
//! This is a port of the desktop's `src-tauri/src/opml.rs` (pure functions,
//! no platform dependencies).

use crate::error::CoreError;
use opml::{Head, Outline, OPML};
use std::collections::BTreeMap;

/// A subscription parsed out of an OPML file.
pub struct ImportedFeed {
    pub feed_url: String,
    pub title: String,
    pub folder: Option<String>,
}

/// Parse an OPML document into a flat list of feeds with their folder names.
///
/// The document is run through [`tidy`] first: real-world OPML exports routinely
/// emit bare `&` characters inside feed URLs and titles rather than the
/// well-formed `&amp;`. The `opml` crate's strict XML parser rejects the entire
/// document on the first such `&`.
pub fn parse(content: &str) -> Result<Vec<ImportedFeed>, CoreError> {
    let doc = OPML::from_str(&tidy(content)).map_err(|e| CoreError::Parse(e.to_string()))?;
    // Real-world OPML exports routinely repeat the same feed URL across folders.
    // Deduplicate by `xml_url` and keep the first occurrence.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut feeds = Vec::new();
    let mut collect_into: Vec<ImportedFeed> = Vec::new();
    for outline in &doc.body.outlines {
        collect(outline, None, &mut collect_into);
    }
    for feed in collect_into {
        if seen.insert(feed.feed_url.clone()) {
            feeds.push(feed);
        }
    }
    Ok(feeds)
}

/// Escape bare `&` — those not already opening a valid XML entity — into `&amp;`,
/// so an otherwise non-well-formed real-world OPML file parses instead of being
/// rejected wholesale.
fn tidy(content: &str) -> String {
    let mut out = String::with_capacity(content.len() + 32);
    for (idx, c) in content.char_indices() {
        if c == '&' && !starts_valid_entity(&content[idx + 1..]) {
            out.push_str("&amp;");
        } else {
            out.push(c);
        }
    }
    out
}

/// Does `rest` (the text immediately following a `&`) begin a valid XML entity
/// reference — a named entity or a numeric one?
fn starts_valid_entity(rest: &str) -> bool {
    for name in ["amp;", "lt;", "gt;", "quot;", "apos;"] {
        if rest.starts_with(name) {
            return true;
        }
    }
    if let Some(after) = rest.strip_prefix('#') {
        let (body, hex) = match after.strip_prefix(['x', 'X']) {
            Some(b) => (b, true),
            None => (after, false),
        };
        if let Some(semi) = body.find(';') {
            return semi > 0
                && body[..semi].chars().all(|c| {
                    if hex {
                        c.is_ascii_hexdigit()
                    } else {
                        c.is_ascii_digit()
                    }
                });
        }
    }
    false
}

/// The human-facing label of an outline: its `text` attribute, falling back to
/// `title`. A whitespace-only attribute counts as absent.
fn outline_label(outline: &Outline) -> Option<&str> {
    let label = if !outline.text.trim().is_empty() {
        Some(outline.text.as_str())
    } else {
        outline.title.as_deref()
    };
    label.filter(|t| !t.trim().is_empty())
}

fn collect(outline: &Outline, folder: Option<&str>, out: &mut Vec<ImportedFeed>) {
    if let Some(url) = &outline.xml_url {
        let title = outline_label(outline).unwrap_or(url).to_string();
        out.push(ImportedFeed {
            feed_url: url.clone(),
            title,
            folder: folder.map(|s| s.to_string()),
        });
    }
    let child_folder = if outline.xml_url.is_none() && !outline.outlines.is_empty() {
        outline_label(outline).or(folder)
    } else {
        folder
    };
    for child in &outline.outlines {
        collect(child, child_folder, out);
    }
}

/// Build an OPML document from `(title, feed_url, folder)` tuples.
///
/// Empty `feed_url` entries are skipped. `folder` is treated as ungrouped if it
/// is `None`, empty, or whitespace-only — matching the `parse` side.
pub fn build(feeds: &[(String, String, Option<String>)]) -> Result<String, CoreError> {
    let mut doc = OPML {
        head: Some(Head {
            title: Some("Papr Subscriptions".to_string()),
            ..Head::default()
        }),
        ..OPML::default()
    };

    let mut by_folder: BTreeMap<Option<String>, Vec<Outline>> = BTreeMap::new();
    for (title, url, folder) in feeds {
        let trimmed_url = url.trim();
        if trimmed_url.is_empty() {
            continue;
        }
        let normalised_folder = folder
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let outline = Outline {
            text: title.clone(),
            xml_url: Some(trimmed_url.to_string()),
            r#type: Some("rss".to_string()),
            ..Outline::default()
        };
        by_folder
            .entry(normalised_folder)
            .or_default()
            .push(outline);
    }

    for (folder, outlines) in by_folder {
        match folder {
            Some(name) => doc.body.outlines.push(Outline {
                text: name,
                outlines,
                ..Outline::default()
            }),
            None => doc.body.outlines.extend(outlines),
        }
    }

    doc.to_string().map_err(|e| CoreError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{build, parse};

    #[test]
    fn parses_flat_feed_list() {
        let xml = r#"<opml version="2.0"><head/><body>
            <outline text="Blog A" xmlUrl="https://a.example/feed.xml"/>
            <outline text="Blog B" xmlUrl="https://b.example/feed.xml"/>
        </body></opml>"#;
        let feeds = parse(xml).expect("parse");
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[0].title, "Blog A");
        assert_eq!(feeds[0].feed_url, "https://a.example/feed.xml");
        assert!(feeds[0].folder.is_none());
    }

    #[test]
    fn parses_feeds_nested_in_a_folder() {
        let xml = r#"<opml version="2.0"><head/><body>
            <outline text="Tech">
                <outline text="Feed 1" xmlUrl="https://1.example/f"/>
                <outline text="Feed 2" xmlUrl="https://2.example/f"/>
            </outline>
        </body></opml>"#;
        let feeds = parse(xml).expect("parse");
        assert_eq!(feeds.len(), 2);
        assert!(feeds.iter().all(|f| f.folder.as_deref() == Some("Tech")));
    }

    #[test]
    fn build_round_trips_through_parse() {
        let input = vec![
            (
                "Folderless".to_string(),
                "https://x.example/f".to_string(),
                None,
            ),
            (
                "In Folder".to_string(),
                "https://y.example/f".to_string(),
                Some("Tech".to_string()),
            ),
        ];
        let xml = build(&input).expect("build");
        let feeds = parse(&xml).expect("re-parse");
        assert_eq!(feeds.len(), 2);
        let folderless = feeds
            .iter()
            .find(|f| f.feed_url == "https://x.example/f")
            .unwrap();
        assert!(folderless.folder.is_none());
        let foldered = feeds
            .iter()
            .find(|f| f.feed_url == "https://y.example/f")
            .unwrap();
        assert_eq!(foldered.title, "In Folder");
        assert_eq!(foldered.folder.as_deref(), Some("Tech"));
    }

    #[test]
    fn bare_ampersands_are_tolerated() {
        let xml = r#"<opml version="1.0"><head/><body>
            <outline title="Cell Death & Disease" type="rss"
                     xmlUrl="https://www.science.org/action/showFeed?type=etoc&feed=rss&jc=stm"/>
        </body></opml>"#;
        let feeds = parse(xml).expect("a bare & must not fail the parse");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].title, "Cell Death & Disease");
        assert_eq!(
            feeds[0].feed_url,
            "https://www.science.org/action/showFeed?type=etoc&feed=rss&jc=stm"
        );
    }

    #[test]
    fn duplicate_feed_url_is_collapsed() {
        let xml = r#"<opml version="1.0"><head/><body>
            <outline title="Today" xmlUrl="https://x.example/feed"/>
            <outline title="All Articles" xmlUrl="https://x.example/feed"/>
            <outline title="Other" xmlUrl="https://y.example/feed"/>
        </body></opml>"#;
        let feeds = parse(xml).expect("parse");
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[0].feed_url, "https://x.example/feed");
    }

    #[test]
    fn parse_rejects_malformed_document() {
        assert!(parse("not opml at all").is_err());
    }
}
