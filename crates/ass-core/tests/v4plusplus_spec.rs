//! Integration tests for ASS v4++ specification support
//!
//! Tests parsing, analysis, and rendering of v4++ format extensions including
//! separate top/bottom margins, `RelativeTo` positioning, and \kt karaoke tags.
//! Ensures backward compatibility with v4+ format while properly handling
//! new v4++ features.

#[path = "v4plusplus_spec/karaoke.rs"]
mod karaoke;
#[path = "v4plusplus_spec/parsing.rs"]
mod parsing;
#[path = "v4plusplus_spec/resolved.rs"]
mod resolved;
