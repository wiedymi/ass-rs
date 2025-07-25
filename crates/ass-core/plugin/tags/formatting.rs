//! Basic formatting tag handlers for ASS override tags
//!
//! Implements handlers for fundamental text formatting commands including
//! bold, italic, underline, and strikeout. These handlers validate arguments
//! and provide standardized processing for common ASS formatting operations.
//!
//! # Supported Tags
//!
//! - `b`: Bold formatting (0/1)
//! - `i`: Italic formatting (0/1)
//! - `u`: Underline formatting (0/1)
//! - `s`: Strikeout formatting (0/1)
//!
//! # Performance
//!
//! - Zero allocations for argument parsing
//! - O(1) validation and processing
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for bold formatting tag (`\b`)
pub struct BoldTagHandler;

impl TagHandler for BoldTagHandler {
    fn name(&self) -> &'static str {
        "b"
    }

    fn process(&self, args: &str) -> TagResult {
        let trimmed = args.trim();

        // Handle basic cases
        match trimmed {
            "" | "0" | "1" => return TagResult::Processed,
            _ => {}
        }

        // Handle font weight values (100-900)
        trimmed.parse::<u32>().map_or_else(
            |_| {
                TagResult::Failed(String::from(
                    "Bold tag accepts 0, 1, or font weight (100-900 in steps of 100)",
                ))
            },
            |weight| {
                if (100..=900).contains(&weight) && weight % 100 == 0 {
                    TagResult::Processed
                } else {
                    TagResult::Failed(String::from(
                        "Bold tag accepts 0, 1, or font weight (100-900 in steps of 100)",
                    ))
                }
            },
        )
    }

    fn validate(&self, args: &str) -> bool {
        let trimmed = args.trim();

        // Basic cases
        if trimmed.is_empty() || trimmed == "0" || trimmed == "1" {
            return true;
        }

        // Font weight values
        trimmed
            .parse::<u32>()
            .is_ok_and(|weight| (100..=900).contains(&weight) && weight % 100 == 0)
    }
}

/// Handler for italic formatting tag (`\i`)
pub struct ItalicTagHandler;

impl TagHandler for ItalicTagHandler {
    fn name(&self) -> &'static str {
        "i"
    }

    fn process(&self, args: &str) -> TagResult {
        match args.trim() {
            "0" | "1" | "" => TagResult::Processed,
            _ => TagResult::Failed(String::from("Italic tag accepts only 0, 1, or empty")),
        }
    }

    fn validate(&self, args: &str) -> bool {
        let trimmed = args.trim();
        trimmed.is_empty() || trimmed == "0" || trimmed == "1"
    }
}

/// Handler for underline formatting tag (`\u`)
pub struct UnderlineTagHandler;

impl TagHandler for UnderlineTagHandler {
    fn name(&self) -> &'static str {
        "u"
    }

    fn process(&self, args: &str) -> TagResult {
        match args.trim() {
            "0" | "1" | "" => TagResult::Processed,
            _ => TagResult::Failed(String::from("Underline tag accepts only 0, 1, or empty")),
        }
    }

    fn validate(&self, args: &str) -> bool {
        let trimmed = args.trim();
        trimmed.is_empty() || trimmed == "0" || trimmed == "1"
    }
}

/// Handler for strikeout formatting tag (`\s`)
pub struct StrikeoutTagHandler;

impl TagHandler for StrikeoutTagHandler {
    fn name(&self) -> &'static str {
        "s"
    }

    fn process(&self, args: &str) -> TagResult {
        match args.trim() {
            "0" | "1" | "" => TagResult::Processed,
            _ => TagResult::Failed(String::from("Strikeout tag accepts only 0, 1, or empty")),
        }
    }

    fn validate(&self, args: &str) -> bool {
        let trimmed = args.trim();
        trimmed.is_empty() || trimmed == "0" || trimmed == "1"
    }
}

/// Create all basic formatting tag handlers
///
/// Returns a vector of boxed tag handlers for all basic formatting operations.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::formatting::create_formatting_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_formatting_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_formatting_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(BoldTagHandler),
        alloc::boxed::Box::new(ItalicTagHandler),
        alloc::boxed::Box::new(UnderlineTagHandler),
        alloc::boxed::Box::new(StrikeoutTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bold_handler_valid_args() {
        let handler = BoldTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process(""), TagResult::Processed);
    }

    #[test]
    fn bold_handler_invalid_args() {
        let handler = BoldTagHandler;
        assert!(matches!(handler.process("2"), TagResult::Failed(_)));
        assert!(matches!(handler.process("50"), TagResult::Failed(_))); // Not a valid weight
        assert!(matches!(handler.process("150"), TagResult::Failed(_))); // Not multiple of 100
        assert!(matches!(handler.process("1000"), TagResult::Failed(_))); // Too high
        assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
    }

    #[test]
    fn bold_handler_font_weights() {
        let handler = BoldTagHandler;
        // Valid font weights (100-900 in steps of 100)
        assert_eq!(handler.process("100"), TagResult::Processed); // Thin
        assert_eq!(handler.process("200"), TagResult::Processed); // Extra Light
        assert_eq!(handler.process("300"), TagResult::Processed); // Light
        assert_eq!(handler.process("400"), TagResult::Processed); // Normal
        assert_eq!(handler.process("500"), TagResult::Processed); // Medium
        assert_eq!(handler.process("600"), TagResult::Processed); // Semi Bold
        assert_eq!(handler.process("700"), TagResult::Processed); // Bold
        assert_eq!(handler.process("800"), TagResult::Processed); // Extra Bold
        assert_eq!(handler.process("900"), TagResult::Processed); // Black
    }

    #[test]
    fn italic_handler_validation() {
        let handler = ItalicTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("1"));
        assert!(handler.validate(""));
        assert!(!handler.validate("2"));
    }

    #[test]
    fn all_handlers_have_correct_names() {
        assert_eq!(BoldTagHandler.name(), "b");
        assert_eq!(ItalicTagHandler.name(), "i");
        assert_eq!(UnderlineTagHandler.name(), "u");
        assert_eq!(StrikeoutTagHandler.name(), "s");
    }

    #[test]
    fn create_formatting_handlers_returns_four() {
        let handlers = create_formatting_handlers();
        assert_eq!(handlers.len(), 4);
    }

    #[test]
    fn bold_handler_validation() {
        let handler = BoldTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("1"));
        assert!(handler.validate(""));
        assert!(handler.validate(" 0 ")); // whitespace handling
        assert!(handler.validate(" 1 "));
        assert!(handler.validate("400")); // font weight
        assert!(handler.validate("700")); // font weight
        assert!(!handler.validate("2"));
        assert!(!handler.validate("invalid"));
        assert!(!handler.validate("-1"));
        assert!(!handler.validate("450")); // not multiple of 100
        assert!(!handler.validate("1000")); // too high
    }

    #[test]
    fn bold_handler_whitespace_handling() {
        let handler = BoldTagHandler;
        assert_eq!(handler.process(" 0 "), TagResult::Processed);
        assert_eq!(handler.process(" 1 "), TagResult::Processed);
        assert_eq!(handler.process("  "), TagResult::Processed);
        assert_eq!(handler.process("\t0\t"), TagResult::Processed);
    }

    #[test]
    fn bold_handler_error_messages() {
        let handler = BoldTagHandler;
        if let TagResult::Failed(msg) = handler.process("2") {
            assert_eq!(
                msg,
                "Bold tag accepts 0, 1, or font weight (100-900 in steps of 100)"
            );
        } else {
            panic!("Expected TagResult::Failed");
        }
    }

    #[test]
    fn italic_handler_process_valid() {
        let handler = ItalicTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process(""), TagResult::Processed);
    }

    #[test]
    fn italic_handler_process_invalid() {
        let handler = ItalicTagHandler;
        assert!(matches!(handler.process("2"), TagResult::Failed(_)));
        assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
        assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
    }

    #[test]
    fn italic_handler_whitespace_handling() {
        let handler = ItalicTagHandler;
        assert_eq!(handler.process(" 0 "), TagResult::Processed);
        assert_eq!(handler.process(" 1 "), TagResult::Processed);
        assert_eq!(handler.process("  "), TagResult::Processed);
        assert!(handler.validate(" 0 "));
        assert!(handler.validate(" 1 "));
    }

    #[test]
    fn italic_handler_error_messages() {
        let handler = ItalicTagHandler;
        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Italic tag accepts only 0, 1, or empty");
        } else {
            panic!("Expected TagResult::Failed");
        }
    }

    #[test]
    fn underline_handler_process_valid() {
        let handler = UnderlineTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process(""), TagResult::Processed);
    }

    #[test]
    fn underline_handler_process_invalid() {
        let handler = UnderlineTagHandler;
        assert!(matches!(handler.process("2"), TagResult::Failed(_)));
        assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
    }

    #[test]
    fn underline_handler_validation() {
        let handler = UnderlineTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("1"));
        assert!(handler.validate(""));
        assert!(handler.validate(" 0 "));
        assert!(!handler.validate("2"));
        assert!(!handler.validate("invalid"));
    }

    #[test]
    fn underline_handler_whitespace_handling() {
        let handler = UnderlineTagHandler;
        assert_eq!(handler.process(" 0 "), TagResult::Processed);
        assert_eq!(handler.process(" 1 "), TagResult::Processed);
        assert_eq!(handler.process("  "), TagResult::Processed);
    }

    #[test]
    fn underline_handler_error_messages() {
        let handler = UnderlineTagHandler;
        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Underline tag accepts only 0, 1, or empty");
        } else {
            panic!("Expected TagResult::Failed");
        }
    }

    #[test]
    fn strikeout_handler_process_valid() {
        let handler = StrikeoutTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process(""), TagResult::Processed);
    }

    #[test]
    fn strikeout_handler_process_invalid() {
        let handler = StrikeoutTagHandler;
        assert!(matches!(handler.process("2"), TagResult::Failed(_)));
        assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
    }

    #[test]
    fn strikeout_handler_validation() {
        let handler = StrikeoutTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("1"));
        assert!(handler.validate(""));
        assert!(handler.validate(" 1 "));
        assert!(!handler.validate("2"));
        assert!(!handler.validate("invalid"));
    }

    #[test]
    fn strikeout_handler_whitespace_handling() {
        let handler = StrikeoutTagHandler;
        assert_eq!(handler.process(" 0 "), TagResult::Processed);
        assert_eq!(handler.process(" 1 "), TagResult::Processed);
        assert_eq!(handler.process("  "), TagResult::Processed);
    }

    #[test]
    fn strikeout_handler_error_messages() {
        let handler = StrikeoutTagHandler;
        if let TagResult::Failed(msg) = handler.process("bad") {
            assert_eq!(msg, "Strikeout tag accepts only 0, 1, or empty");
        } else {
            panic!("Expected TagResult::Failed");
        }
    }

    #[test]
    fn all_handlers_consistency() {
        let handlers = create_formatting_handlers();

        // Test that all handlers except bold behave consistently
        // Skip bold handler (index 0) as it has extended functionality
        for handler in handlers.iter().skip(1) {
            assert_eq!(handler.process("0"), TagResult::Processed);
            assert_eq!(handler.process("1"), TagResult::Processed);
            assert_eq!(handler.process(""), TagResult::Processed);
            assert!(handler.validate("0"));
            assert!(handler.validate("1"));
            assert!(handler.validate(""));
            assert!(!handler.validate("2"));
            assert!(!handler.validate("invalid"));
        }
    }

    #[test]
    fn handlers_name_uniqueness() {
        let handlers = create_formatting_handlers();
        let mut names = std::collections::HashSet::new();

        for handler in &handlers {
            let name = handler.name();
            assert!(!names.contains(name), "Duplicate handler name: {name}");
            names.insert(name);
        }
    }

    #[test]
    fn edge_case_arguments() {
        let handlers = create_formatting_handlers();

        // Skip bold handler (index 0) as it has extended functionality
        for handler in handlers.iter().skip(1) {
            // Test extreme whitespace
            assert_eq!(handler.process("\n\t \r"), TagResult::Processed);
            assert!(handler.validate("\n\t \r"));

            // Test numeric edge cases
            assert!(matches!(handler.process("00"), TagResult::Failed(_)));
            assert!(matches!(handler.process("01"), TagResult::Failed(_)));
            assert!(matches!(handler.process("10"), TagResult::Failed(_)));
            assert!(matches!(handler.process("11"), TagResult::Failed(_)));
        }
    }
}
