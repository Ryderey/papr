pub mod discovery;
pub mod fetch;
pub mod parse;
pub mod sanitize;
pub mod sources;

pub use fetch::{build_client, conditional_get, decode_html, get, Fetched, USER_AGENT};
pub use parse::{parse_feed, ParsedFeed};
pub use sanitize::{escape_html, first_image, html_to_text, sanitize};
