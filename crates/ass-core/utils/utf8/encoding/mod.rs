//! Encoding detection utilities for ASS subtitle files
//!
//! Provides functionality for detecting text encodings, analyzing content
//! patterns, and validating encoding assumptions. Focuses on encodings
//! commonly used in ASS subtitle files with confidence scoring.
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::utf8::{detect_encoding, EncodingInfo};
//!
//! let text = "[Script Info]\nTitle: Test";
//! let encoding = detect_encoding(text.as_bytes());
//! assert_eq!(encoding.encoding, "UTF-8");
//! assert!(encoding.confidence > 0.8);
//! ```

mod detection;
mod info;

#[cfg(test)]
mod tests;

pub use detection::{detect_encoding, is_likely_ass_content};
pub use info::EncodingInfo;
