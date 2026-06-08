//! Primary parse error type for ASS script parsing
//!
//! Contains the main `ParseError` enum representing unrecoverable parsing errors
//! that prevent script construction. These errors indicate fundamental issues
//! with the script structure or content that cannot be recovered from.

mod error;
mod formatting;

pub use error::ParseError;
