//! Built-in syntax highlighting extension for ASS/SSA files
//!
//! Provides syntax highlighting for ASS subtitle format, including:
//! - Section headers (`[Script Info]`, `[Styles]`, `[Events]`)
//! - Field names and values
//! - Override tags and their parameters
//! - Comments and special formatting

mod document;
mod editor_ext;
mod events;
mod extension;
mod sections;
mod tags;
mod types;

pub use extension::{SyntaxHighlightConfig, SyntaxHighlightExtension};
pub use types::{HighlightToken, TokenType};

#[cfg(test)]
mod tests;

// Include extended tests
#[cfg(test)]
#[path = "../syntax_highlight_tests.rs"]
mod extended_tests;
