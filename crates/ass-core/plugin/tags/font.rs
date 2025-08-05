//! Font tag handlers for ASS override tags
//!
//! Implements handlers for font-related commands including font name,
//! size, and encoding. These handlers validate arguments according to
//! ASS specifications.
//!
//! # Supported Tags
//!
//! - `fn`: Font name
//! - `fs`: Font size
//! - `fe`: Font encoding (character set)
//!
//! # Performance
//!
//! - Zero allocations for validation
//! - Fast numeric validation for size
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for font name tag (`\fn`)
///
/// Sets the font face by name. Accepts any non-empty string.
pub struct FontNameTagHandler;

impl TagHandler for FontNameTagHandler {
    fn name(&self) -> &'static str {
        "fn"
    }

    fn process(&self, args: &str) -> TagResult {
        let args = args.trim();
        if args.is_empty() {
            TagResult::Failed(String::from("Font name tag requires a font name"))
        } else {
            TagResult::Processed
        }
    }

    fn validate(&self, args: &str) -> bool {
        !args.trim().is_empty()
    }
}

/// Handler for font size tag (`\fs`)
///
/// Sets the font size. Must be a positive number.
pub struct FontSizeTagHandler;

impl TagHandler for FontSizeTagHandler {
    fn name(&self) -> &'static str {
        "fs"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Font size tag requires positive numeric size"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Parse as positive number (integer or decimal)
        args.parse::<f32>().is_ok_and(|size| size > 0.0)
    }
}

/// Handler for font encoding tag (`\fe`)
///
/// Sets the font character set. Must be a valid encoding number.
pub struct FontEncodingTagHandler;

impl TagHandler for FontEncodingTagHandler {
    fn name(&self) -> &'static str {
        "fe"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Font encoding tag requires numeric charset"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Must be a non-negative integer
        args.parse::<u32>().is_ok()
    }
}

/// Create all font tag handlers
///
/// Returns a vector of boxed tag handlers for font-related operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::font::create_font_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_font_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_font_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(FontNameTagHandler),
        alloc::boxed::Box::new(FontSizeTagHandler),
        alloc::boxed::Box::new(FontEncodingTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_name_valid() {
        let handler = FontNameTagHandler;
        assert_eq!(handler.process("Arial"), TagResult::Processed);
        assert_eq!(handler.process("Times New Roman"), TagResult::Processed);
        assert_eq!(handler.process("MS UI Gothic"), TagResult::Processed);
        assert_eq!(handler.process(" Comic Sans MS "), TagResult::Processed);
        assert_eq!(handler.process("font-family"), TagResult::Processed);
    }

    #[test]
    fn font_name_invalid() {
        let handler = FontNameTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("  "), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("") {
            assert_eq!(msg, "Font name tag requires a font name");
        }
    }

    #[test]
    fn font_size_valid() {
        let handler = FontSizeTagHandler;
        assert_eq!(handler.process("12"), TagResult::Processed);
        assert_eq!(handler.process("24.5"), TagResult::Processed);
        assert_eq!(handler.process("100"), TagResult::Processed);
        assert_eq!(handler.process("0.5"), TagResult::Processed);
        assert_eq!(handler.process(" 16 "), TagResult::Processed);
    }

    #[test]
    fn font_size_invalid() {
        let handler = FontSizeTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("0"), TagResult::Failed(_)));
        assert!(matches!(handler.process("-12"), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
        assert!(matches!(handler.process("12px"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Font size tag requires positive numeric size");
        }
    }

    #[test]
    fn font_encoding_valid() {
        let handler = FontEncodingTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process("128"), TagResult::Processed);
        assert_eq!(handler.process("255"), TagResult::Processed);
        assert_eq!(handler.process(" 134 "), TagResult::Processed);
    }

    #[test]
    fn font_encoding_invalid() {
        let handler = FontEncodingTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
        assert!(matches!(handler.process("1.5"), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Font encoding tag requires numeric charset");
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(FontNameTagHandler.name(), "fn");
        assert_eq!(FontSizeTagHandler.name(), "fs");
        assert_eq!(FontEncodingTagHandler.name(), "fe");
    }

    #[test]
    fn create_font_handlers_returns_all() {
        let handlers = create_font_handlers();
        assert_eq!(handlers.len(), 3);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
        assert!(names.contains(&"fn"));
        assert!(names.contains(&"fs"));
        assert!(names.contains(&"fe"));
    }

    #[test]
    fn font_name_validation() {
        let handler = FontNameTagHandler;
        assert!(handler.validate("Arial"));
        assert!(handler.validate("Font Name With Spaces"));
        assert!(handler.validate("  trimmed  "));
        assert!(!handler.validate(""));
        assert!(!handler.validate("   "));
    }

    #[test]
    fn font_size_validation() {
        let handler = FontSizeTagHandler;
        assert!(handler.validate("12"));
        assert!(handler.validate("12.5"));
        assert!(handler.validate("0.1"));
        assert!(!handler.validate("0"));
        assert!(!handler.validate("-5"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("abc"));
    }

    #[test]
    fn font_encoding_validation() {
        let handler = FontEncodingTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("128"));
        assert!(handler.validate("255"));
        assert!(!handler.validate("-1"));
        assert!(!handler.validate("1.5"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("abc"));
    }

    #[test]
    fn font_name_edge_cases() {
        let handler = FontNameTagHandler;
        // Unicode font names
        assert_eq!(handler.process("ＭＳ ゴシック"), TagResult::Processed);
        // Font names with special characters
        assert_eq!(handler.process("Font_Name-123"), TagResult::Processed);
        // Very long font name
        assert_eq!(
            handler.process("A".repeat(100).as_str()),
            TagResult::Processed
        );
    }

    #[test]
    fn font_size_edge_cases() {
        let handler = FontSizeTagHandler;
        // Very small but positive
        assert_eq!(handler.process("0.001"), TagResult::Processed);
        // Very large
        assert_eq!(handler.process("9999"), TagResult::Processed);
        // Scientific notation is actually accepted by parse::<f32>()
        assert_eq!(handler.process("1e2"), TagResult::Processed); // 100
    }

    #[test]
    fn font_encoding_edge_cases() {
        let handler = FontEncodingTagHandler;
        // Common encoding values
        assert_eq!(handler.process("0"), TagResult::Processed); // ANSI
        assert_eq!(handler.process("1"), TagResult::Processed); // Default
        assert_eq!(handler.process("128"), TagResult::Processed); // Shift-JIS
        assert_eq!(handler.process("134"), TagResult::Processed); // GB2312
        assert_eq!(handler.process("136"), TagResult::Processed); // Big5
                                                                  // Max u32
        assert_eq!(handler.process("4294967295"), TagResult::Processed);
    }
}
