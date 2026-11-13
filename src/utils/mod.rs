//! Utility modules for ClickDown

pub mod clipboard;
pub mod copy_text;
pub mod deserializers;
pub mod query;
pub mod text;
pub mod url_generator;
pub mod url_parser;

pub use clipboard::ClipboardService;
pub use copy_text::comment_text_to_plain;
pub use query::QueryParams;
pub use text::{format_timestamp, truncate_chars, wrap_text};
pub use url_generator::{ClickUpUrlGenerator, UrlGenerator};
pub use url_parser::{ParsedUrl, UrlParser};
