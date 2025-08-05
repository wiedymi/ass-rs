//! Special character tag handlers for ASS override tags
//!
//! Implements handlers for special characters like line breaks and
//! non-breaking spaces. These tags don't take arguments but affect
//! text layout and rendering.
//!
//! # Supported Tags
//!
//! - `n`: Soft line break (ignored with smart wrapping)
//! - `N`: Hard line break (always creates new line)
//! - `h`: Hard space (non-breaking space)
//!
//! # Performance
//!
//! - Zero allocations
//! - O(1) validation and processing
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for soft line break tag (`\n`)
///
/// Creates a line break except when smart wrapping is enabled.
pub struct SoftLineBreakTagHandler;

impl TagHandler for SoftLineBreakTagHandler {
    fn name(&self) -> &'static str {
        "n"
    }

    fn process(&self, args: &str) -> TagResult {
        if args.trim().is_empty() {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Line break tag takes no arguments"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().is_empty()
    }
}

/// Handler for hard line break tag (`\N`)
///
/// Always creates a line break regardless of wrapping mode.
pub struct HardLineBreakTagHandler;

impl TagHandler for HardLineBreakTagHandler {
    fn name(&self) -> &'static str {
        "N"
    }

    fn process(&self, args: &str) -> TagResult {
        if args.trim().is_empty() {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Line break tag takes no arguments"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().is_empty()
    }
}

/// Handler for hard space tag (`\h`)
///
/// Inserts a non-breaking space character.
pub struct HardSpaceTagHandler;

impl TagHandler for HardSpaceTagHandler {
    fn name(&self) -> &'static str {
        "h"
    }

    fn process(&self, args: &str) -> TagResult {
        if args.trim().is_empty() {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Hard space tag takes no arguments"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().is_empty()
    }
}

/// Create all special character tag handlers
///
/// Returns a vector of boxed tag handlers for special character operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::special::create_special_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_special_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_special_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(SoftLineBreakTagHandler),
        alloc::boxed::Box::new(HardLineBreakTagHandler),
        alloc::boxed::Box::new(HardSpaceTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn soft_line_break_valid() {
        let handler = SoftLineBreakTagHandler;
        assert_eq!(handler.process(""), TagResult::Processed);
        assert_eq!(handler.process("  "), TagResult::Processed);
        assert_eq!(handler.process("\t"), TagResult::Processed);
    }

    #[test]
    fn soft_line_break_invalid() {
        let handler = SoftLineBreakTagHandler;
        assert!(matches!(handler.process("arg"), TagResult::Failed(_)));
        assert!(matches!(handler.process("123"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Line break tag takes no arguments");
        }
    }

    #[test]
    fn hard_line_break_valid() {
        let handler = HardLineBreakTagHandler;
        assert_eq!(handler.process(""), TagResult::Processed);
        assert_eq!(handler.process("  "), TagResult::Processed);
        assert_eq!(handler.process("\t"), TagResult::Processed);
    }

    #[test]
    fn hard_line_break_invalid() {
        let handler = HardLineBreakTagHandler;
        assert!(matches!(handler.process("arg"), TagResult::Failed(_)));
        assert!(matches!(handler.process("123"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Line break tag takes no arguments");
        }
    }

    #[test]
    fn hard_space_valid() {
        let handler = HardSpaceTagHandler;
        assert_eq!(handler.process(""), TagResult::Processed);
        assert_eq!(handler.process("  "), TagResult::Processed);
        assert_eq!(handler.process("\t"), TagResult::Processed);
    }

    #[test]
    fn hard_space_invalid() {
        let handler = HardSpaceTagHandler;
        assert!(matches!(handler.process("arg"), TagResult::Failed(_)));
        assert!(matches!(handler.process("123"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Hard space tag takes no arguments");
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(SoftLineBreakTagHandler.name(), "n");
        assert_eq!(HardLineBreakTagHandler.name(), "N");
        assert_eq!(HardSpaceTagHandler.name(), "h");
    }

    #[test]
    fn create_special_handlers_returns_all() {
        let handlers = create_special_handlers();
        assert_eq!(handlers.len(), 3);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
        assert!(names.contains(&"n"));
        assert!(names.contains(&"N"));
        assert!(names.contains(&"h"));
    }

    #[test]
    fn validation_consistency() {
        let handlers = create_special_handlers();

        for handler in &handlers {
            // All special handlers accept empty args
            assert!(handler.validate(""));
            assert!(handler.validate("  "));
            assert_eq!(handler.process(""), TagResult::Processed);

            // All reject non-empty args
            assert!(!handler.validate("something"));
            assert!(matches!(handler.process("something"), TagResult::Failed(_)));
        }
    }

    #[test]
    fn whitespace_handling() {
        let handlers = create_special_handlers();

        for handler in &handlers {
            // Various whitespace should be accepted
            assert_eq!(handler.process(""), TagResult::Processed);
            assert_eq!(handler.process(" "), TagResult::Processed);
            assert_eq!(handler.process("  "), TagResult::Processed);
            assert_eq!(handler.process("\t"), TagResult::Processed);
            assert_eq!(handler.process("\n"), TagResult::Processed);
            assert_eq!(handler.process(" \t\n "), TagResult::Processed);
        }
    }
}
