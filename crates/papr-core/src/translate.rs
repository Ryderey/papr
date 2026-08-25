//! Portable HTML translation helpers.
//!
//! Network engines and persistence deliberately live outside this module. These
//! helpers define the shared, testable rule: translate readable text while
//! preserving article HTML structure and code-like content.

use ego_tree::NodeId;
use scraper::node::Node;
use scraper::{ElementRef, Html};

const UNWRAP_TAGS: &[&str] = &["div", "article", "section", "main"];
const SKIP_TEXT_TAGS: &[&str] = &["script", "style", "code", "pre", "kbd", "samp"];

/// The human-readable name for the supported target language codes.
pub fn language_name(code: &str) -> &'static str {
    match code {
        "zh" => "Simplified Chinese",
        "ja" => "Japanese",
        _ => "English",
    }
}

/// Build the LLM instruction for translating one HTML fragment.
pub fn system_prompt(target: &str) -> String {
    format!(
        "You are a professional translator. Translate the text content of the \
         HTML fragment into {target}.\n\n\
         Rules:\n\
         - Preserve every HTML tag, attribute, and the overall structure exactly.\n\
         - Translate only human-readable text; do not translate code or URLs.\n\
         - Keep images, links, and all other markup intact.\n\
         - Output only the translated HTML fragment: no preamble, no code fences."
    )
}

/// Split HTML into whole top-level blocks under `budget` bytes where possible.
pub fn chunk_blocks(html: &str, budget: usize) -> Vec<String> {
    let fragment = Html::parse_fragment(html);
    let mut nodes: Vec<_> = fragment.root_element().children().collect();

    loop {
        let elements: Vec<_> = nodes
            .iter()
            .filter(|node| node.value().as_element().is_some())
            .copied()
            .collect();
        match elements.as_slice() {
            [only]
                if only
                    .value()
                    .as_element()
                    .is_some_and(|element| UNWRAP_TAGS.contains(&element.name())) =>
            {
                nodes = only.children().collect();
            }
            _ => break,
        }
    }

    let pieces = nodes.into_iter().filter_map(|node| match node.value() {
        Node::Element(_) => ElementRef::wrap(node).map(|element| element.html()),
        Node::Text(text) if !text.trim().is_empty() => Some(text.to_string()),
        _ => None,
    });

    let mut batches = Vec::new();
    let mut current = String::new();
    for piece in pieces {
        if !current.is_empty() && current.len() + piece.len() > budget {
            batches.push(std::mem::take(&mut current));
        }
        current.push_str(&piece);
    }
    if !current.is_empty() {
        batches.push(current);
    }
    batches
}

/// Remove a surrounding Markdown code fence from an LLM response.
pub fn strip_code_fence(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = rest.strip_suffix("```").unwrap_or(rest);
        let content = match rest.find('\n') {
            Some(index) => &rest[index + 1..],
            None => rest,
        };
        return content.trim().to_string();
    }
    trimmed.to_string()
}

struct TextSlot {
    id: NodeId,
    prefix: String,
    suffix: String,
}

fn collect_text(document: &Html) -> (Vec<TextSlot>, Vec<String>) {
    let mut slots = Vec::new();
    let mut text = Vec::new();
    for node in document.tree.nodes() {
        let Node::Text(value) = node.value() else {
            continue;
        };
        let value: &str = value;
        if value.trim().is_empty()
            || node.ancestors().any(|ancestor| {
                ancestor
                    .value()
                    .as_element()
                    .is_some_and(|element| SKIP_TEXT_TAGS.contains(&element.name()))
            })
        {
            continue;
        }
        let prefix = value[..value.len() - value.trim_start().len()].to_string();
        let suffix = value[value.trim_end().len()..].to_string();
        slots.push(TextSlot {
            id: node.id(),
            prefix,
            suffix,
        });
        text.push(value.trim().to_string());
    }
    (slots, text)
}

fn serialize_fragment(document: &Html) -> String {
    document
        .root_element()
        .children()
        .filter_map(|child| match child.value() {
            Node::Element(_) => ElementRef::wrap(child).map(|element| element.html()),
            Node::Text(text) => Some(text.to_string()),
            _ => None,
        })
        .collect()
}

/// Replace readable text nodes with positional translations, retaining markup.
/// Missing translations leave their original nodes unchanged rather than
/// shifting later translations onto the wrong text.
pub fn rewrite_fragment(html: &str, translated: &[String]) -> String {
    let mut document = Html::parse_fragment(html);
    let (slots, _) = collect_text(&document);
    for (slot, replacement) in slots.iter().zip(translated) {
        if let Some(mut node) = document.tree.get_mut(slot.id) {
            if let Node::Text(value) = node.value() {
                value.text = format!("{}{}{}", slot.prefix, replacement, slot.suffix).into();
            }
        }
    }
    serialize_fragment(&document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_nested_generic_wrappers_without_splitting_blocks() {
        let batches = chunk_blocks(
            "<article><div><p>first</p><p>second</p></div></article>",
            15,
        );

        assert_eq!(batches, ["<p>first</p>", "<p>second</p>"]);
    }

    #[test]
    fn rewrite_preserves_markup_whitespace_and_code() {
        let rewritten = rewrite_fragment(
            "<p> Hello <a href=\"/a\">world</a> <code>let x = 1;</code></p>",
            &["你好".to_string(), "世界".to_string()],
        );

        assert_eq!(
            rewritten,
            "<p> 你好 <a href=\"/a\">世界</a> <code>let x = 1;</code></p>"
        );
    }

    #[test]
    fn rewrite_keeps_unmatched_source_text() {
        let rewritten = rewrite_fragment("<p>one <strong>two</strong></p>", &["一".to_string()]);

        assert_eq!(rewritten, "<p>一 <strong>two</strong></p>");
    }

    #[test]
    fn strips_optional_markdown_fence() {
        assert_eq!(strip_code_fence("```html\n<p>译文</p>\n```"), "<p>译文</p>");
        assert_eq!(strip_code_fence(" <p>plain</p> "), "<p>plain</p>");
    }
}
