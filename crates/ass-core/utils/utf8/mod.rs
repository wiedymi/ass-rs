//! UTF-8 and text encoding utilities for ASS script processing
//!
//! Provides BOM handling, encoding detection, and UTF-8 validation utilities
//! optimized for ASS subtitle script processing with zero-copy design.
//!
//! # Features
//!
//! - BOM detection and stripping for common encodings
//! - UTF-8 validation with detailed error reporting
//! - Encoding detection for legacy ASS files
//! - `nostd` compatible implementation
//! - Zero-copy operations where possible
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::utf8::{strip_bom, detect_encoding, validate_utf8};
//!
//! // Strip BOM if present
//! let input = "\u{FEFF}[Script Info]\nTitle: Test";
//! let (stripped, had_bom) = strip_bom(input);
//! assert_eq!(stripped, "[Script Info]\nTitle: Test");
//! assert!(had_bom);
//!
//! // Detect encoding
//! let text = "[Script Info]\nTitle: Test";
//! let encoding = detect_encoding(text.as_bytes());
//! assert_eq!(encoding.encoding, "UTF-8");
//! assert!(encoding.confidence > 0.8);
//!
//! // Validate UTF-8
//! let valid_text = "Hello, 世界! 🎵";
//! assert!(validate_utf8(valid_text.as_bytes()).is_ok());
//! ```

mod bom;
mod encoding;
mod normalization;
mod validation;

// Re-export all public types and functions for API compatibility
pub use bom::{detect_bom, strip_bom, BomType};
pub use encoding::{detect_encoding, is_likely_ass_content, EncodingInfo};
pub use normalization::{
    normalize_line_endings, normalize_whitespace, remove_control_chars, trim_lines,
};
pub use validation::{
    count_replacement_chars, is_valid_ass_text, recover_utf8, truncate_at_char_boundary,
    validate_utf8,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_bom_detection() {
        let text_with_bom = "\u{FEFF}Hello World";
        let (stripped, had_bom) = strip_bom(text_with_bom);
        assert_eq!(stripped, "Hello World");
        assert!(had_bom);

        let bytes = &[0xEF, 0xBB, 0xBF, b'H', b'i'];
        let (bom_type, skip) = detect_bom(bytes).unwrap();
        assert_eq!(bom_type, BomType::Utf8);
        assert_eq!(skip, 3);
    }

    #[test]
    fn integration_encoding_detection() {
        let text = "[Script Info]\nTitle: Test Script";
        let encoding = detect_encoding(text.as_bytes());
        assert_eq!(encoding.encoding, "UTF-8");
        assert!(encoding.confidence > 0.9); // High confidence due to ASS patterns
        assert!(!encoding.has_bom);

        assert!(is_likely_ass_content(text));
        assert!(!is_likely_ass_content("Just regular text"));
    }

    #[test]
    fn integration_validation_and_recovery() {
        let valid_text = "Hello, 世界! 🎵";
        assert!(validate_utf8(valid_text.as_bytes()).is_ok());

        let invalid_bytes = &[b'H', b'i', 0xFF, b'!'];
        let (recovered, replacements) = recover_utf8(invalid_bytes);
        assert_eq!(recovered, "Hi�!");
        assert_eq!(replacements, 1);

        assert_eq!(count_replacement_chars(&recovered), 1);
    }

    #[test]
    fn integration_normalization() {
        let input = "Line 1\r\nLine 2\rLine 3\n";
        let normalized = normalize_line_endings(input);
        assert_eq!(normalized, "Line 1\nLine 2\nLine 3\n");

        let whitespace_text = "Hello    World   Test";
        let normalized_ws = normalize_whitespace(whitespace_text, true);
        assert_eq!(normalized_ws, "Hello World Test");

        let input_with_control = "Hello\x00World\x1FTest\nValid";
        let cleaned = remove_control_chars(input_with_control);
        assert_eq!(cleaned, "HelloWorldTest\nValid");
    }

    #[test]
    fn integration_text_validation() {
        assert!(is_valid_ass_text("Hello World"));
        assert!(is_valid_ass_text("Hello\tWorld\n"));
        assert!(is_valid_ass_text("Hello 世界"));
        assert!(!is_valid_ass_text("Hello\x00World")); // Null character
        assert!(!is_valid_ass_text("Hello\x1FWorld")); // Control character
    }

    #[test]
    fn integration_truncation() {
        let text = "Hello World";
        let (truncated, was_truncated) = truncate_at_char_boundary(text, 5);
        assert_eq!(truncated, "Hello");
        assert!(was_truncated);

        let unicode_text = "Hello 世界";
        let (truncated, was_truncated) = truncate_at_char_boundary(unicode_text, 8);
        assert_eq!(truncated, "Hello "); // Stops before the Unicode character
        assert!(was_truncated);

        let short_text = "Hi";
        let (truncated, was_truncated) = truncate_at_char_boundary(short_text, 10);
        assert_eq!(truncated, "Hi");
        assert!(!was_truncated);
    }

    #[test]
    fn integration_full_workflow() {
        // Simulate processing an ASS file with various encoding issues
        let input = "\u{FEFF}[Script Info]\r\nTitle: Test\x00Script\r\n\r\n[Events]\nDialogue: Hello    World";

        // Step 1: Strip BOM
        let (without_bom, had_bom) = strip_bom(input);
        assert!(had_bom);

        // Step 2: Normalize line endings
        let normalized = normalize_line_endings(without_bom);

        // Step 3: Remove control characters
        let cleaned = remove_control_chars(&normalized);

        // Step 4: Normalize whitespace
        let final_text = normalize_whitespace(&cleaned, true);

        // Verify final result
        assert!(final_text.contains("[Script Info]"));
        assert!(final_text.contains("Title: TestScript")); // Control char removed
        assert!(!final_text.contains('\r')); // No carriage returns
        assert!(final_text.contains("Dialogue: Hello World")); // Whitespace normalized
    }
}
