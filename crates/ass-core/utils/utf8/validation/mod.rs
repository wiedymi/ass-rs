//! UTF-8 validation and recovery utilities for ASS subtitle processing
//!
//! Provides detailed UTF-8 validation with position-specific error reporting
//! and recovery mechanisms for handling invalid UTF-8 sequences. Designed
//! for robust processing of subtitle files with various encoding issues.
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::utf8::{validate_utf8, recover_utf8};
//!
//! let valid_text = "Hello, 世界! 🎵";
//! assert!(validate_utf8(valid_text.as_bytes()).is_ok());
//!
//! let invalid_bytes = &[b'H', b'i', 0xFF, b'!'];
//! let (recovered, replacements) = recover_utf8(invalid_bytes);
//! assert_eq!(recovered, "Hi�!");
//! assert_eq!(replacements, 1);
//! ```

mod recovery;
mod truncate;
mod validate;

#[cfg(test)]
mod tests;

pub use recovery::{count_replacement_chars, recover_utf8};
pub use truncate::truncate_at_char_boundary;
pub use validate::{is_valid_ass_text, validate_utf8};
