//! ASS script tokenizer module
//!
//! Provides zero-copy lexical analysis of ASS subtitle scripts with incremental tokenization.
//! Supports SIMD-accelerated delimiter scanning and hex parsing for optimal performance.
//!
//! # Performance
//!
//! - Target: <1ms/1KB tokenization with zero allocations
//! - SIMD: 20-30% faster delimiter scanning when enabled
//! - Memory: Zero-copy via `&'a str` spans referencing source
//!
//! # Example
//!
//! ```rust
//! use ass_core::tokenizer::AssTokenizer;
//!
//! let source = "[Script Info]\nTitle: Example";
//! let mut tokenizer = AssTokenizer::new(source);
//!
//! while let Some(token) = tokenizer.next_token()? {
//!     println!("{:?}", token);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod ass_tokenizer;
mod next_token;
pub mod scanner;
#[cfg(feature = "simd")]
pub mod simd;
pub mod state;
pub mod tokens;

// Re-export public API
pub use ass_tokenizer::AssTokenizer;
pub use scanner::{CharNavigator, TokenScanner};
pub use state::{IssueCollector, IssueLevel, TokenContext, TokenIssue};
pub use tokens::{DelimiterType, Token, TokenType};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod inline1_tests;
#[cfg(test)]
mod inline2_tests;
#[cfg(test)]
mod inline3_tests;
#[cfg(test)]
mod inline4_tests;
#[cfg(test)]
mod inline5_tests;
#[cfg(test)]
mod inline6_tests;
