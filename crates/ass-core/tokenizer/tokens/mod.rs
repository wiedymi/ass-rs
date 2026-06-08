//! Token definitions for ASS script tokenization
//!
//! Provides zero-copy token types for lexical analysis of ASS subtitle scripts.
//! All tokens maintain references to the original source text via lifetime parameters.
//!
//! # Token Design
//!
//! - Zero-copy via `&'a str` spans referencing source
//! - Location tracking for error reporting and editor integration
//! - Semantic token types for context-aware parsing
//! - Efficient discriminant matching for hot parsing paths
//!
//! # Example
//!
//! ```rust
//! use ass_core::tokenizer::{Token, TokenType};
//!
//! let source = "[Script Info]";
//! // Token would be created by tokenizer with span referencing source
//! let token = Token {
//!     token_type: TokenType::SectionHeader,
//!     span: &source[0..12], // "[Script Info"
//!     line: 1,
//!     column: 1,
//! };
//! ```

mod delimiter;
mod position;
mod token;
mod token_type;

#[cfg(test)]
mod delimiter_tests;
#[cfg(test)]
mod position_tests;
#[cfg(test)]
mod token_tests;
#[cfg(test)]
mod token_type_name_tests;
#[cfg(test)]
mod token_type_tests;

pub use delimiter::DelimiterType;
pub use position::TokenPosition;
pub use token::Token;
pub use token_type::TokenType;
