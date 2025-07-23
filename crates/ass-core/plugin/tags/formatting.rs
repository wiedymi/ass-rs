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
        match args.trim() {
            "0" | "1" => TagResult::Processed,
            "" => TagResult::Processed, // Empty args defaults to toggle
            _ => TagResult::Failed(String::from("Bold tag accepts only 0, 1, or empty")),
        }
    }

    fn validate(&self, args: &str) -> bool {
        let trimmed = args.trim();
        trimmed.is_empty() || trimmed == "0" || trimmed == "1"
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
            "0" | "1" => TagResult::Processed,
            "" => TagResult::Processed,
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
            "0" | "1" => TagResult::Processed,
            "" => TagResult::Processed,
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
            "0" | "1" => TagResult::Processed,
            "" => TagResult::Processed,
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
#[must_use] pub fn create_formatting_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
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
        assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
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
}
