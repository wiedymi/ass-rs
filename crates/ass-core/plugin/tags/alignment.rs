//! Alignment and layout tag handlers for ASS override tags
//!
//! Implements handlers for text alignment and wrapping style commands.
//! These handlers validate alignment codes according to ASS specifications.
//!
//! # Supported Tags
//!
//! - `a`: Legacy alignment (1-3 + 4/8 modifiers)
//! - `an`: Numpad-style alignment (1-9)
//! - `q`: Wrapping style (0-3)
//!
//! # Performance
//!
//! - Zero allocations for validation
//! - Fast integer validation
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for legacy alignment tag (`\a`)
///
/// Uses legacy alignment codes:
/// - 1 = left, 2 = center, 3 = right
/// - +4 = top, +0 = bottom, +8 = middle (vertical)
pub struct AlignmentTagHandler;

impl TagHandler for AlignmentTagHandler {
    fn name(&self) -> &'static str {
        "a"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Alignment tag requires valid alignment code (1-11)",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();

        // Valid values: 1-3 (bottom), 5-7 (top), 9-11 (middle)
        matches!(args, "1" | "2" | "3" | "5" | "6" | "7" | "9" | "10" | "11")
    }
}

/// Handler for numpad alignment tag (`\an`)
///
/// Uses numpad-style alignment (1-9):
/// ```text
/// 7 8 9
/// 4 5 6
/// 1 2 3
/// ```
pub struct NumpadAlignmentTagHandler;

impl TagHandler for NumpadAlignmentTagHandler {
    fn name(&self) -> &'static str {
        "an"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Numpad alignment tag requires value 1-9"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        matches!(
            args.trim(),
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
        )
    }
}

/// Handler for wrapping style tag (`\q`)
///
/// Controls text wrapping behavior:
/// - 0 = Smart wrapping
/// - 1 = End-of-line wrapping (\N only)
/// - 2 = No wrapping (\n, \N break)
/// - 3 = Smart wrapping, wider lower line
pub struct WrappingStyleTagHandler;

impl TagHandler for WrappingStyleTagHandler {
    fn name(&self) -> &'static str {
        "q"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Wrapping style tag requires value 0-3"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        matches!(args.trim(), "0" | "1" | "2" | "3")
    }
}

/// Create all alignment tag handlers
///
/// Returns a vector of boxed tag handlers for alignment operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::alignment::create_alignment_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_alignment_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_alignment_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(AlignmentTagHandler),
        alloc::boxed::Box::new(NumpadAlignmentTagHandler),
        alloc::boxed::Box::new(WrappingStyleTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_handler_valid() {
        let handler = AlignmentTagHandler;
        // Bottom row
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process("2"), TagResult::Processed);
        assert_eq!(handler.process("3"), TagResult::Processed);
        // Top row
        assert_eq!(handler.process("5"), TagResult::Processed);
        assert_eq!(handler.process("6"), TagResult::Processed);
        assert_eq!(handler.process("7"), TagResult::Processed);
        // Middle row
        assert_eq!(handler.process("9"), TagResult::Processed);
        assert_eq!(handler.process("10"), TagResult::Processed);
        assert_eq!(handler.process("11"), TagResult::Processed);
    }

    #[test]
    fn alignment_handler_invalid() {
        let handler = AlignmentTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("0"), TagResult::Failed(_)));
        assert!(matches!(handler.process("4"), TagResult::Failed(_)));
        assert!(matches!(handler.process("8"), TagResult::Failed(_)));
        assert!(matches!(handler.process("12"), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("4") {
            assert_eq!(msg, "Alignment tag requires valid alignment code (1-11)");
        }
    }

    #[test]
    fn numpad_alignment_handler_valid() {
        let handler = NumpadAlignmentTagHandler;
        for i in 1..=9 {
            assert_eq!(handler.process(&i.to_string()), TagResult::Processed);
        }
        // With whitespace
        assert_eq!(handler.process(" 5 "), TagResult::Processed);
    }

    #[test]
    fn numpad_alignment_handler_invalid() {
        let handler = NumpadAlignmentTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("0"), TagResult::Failed(_)));
        assert!(matches!(handler.process("10"), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("0") {
            assert_eq!(msg, "Numpad alignment tag requires value 1-9");
        }
    }

    #[test]
    fn wrapping_style_handler_valid() {
        let handler = WrappingStyleTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process("2"), TagResult::Processed);
        assert_eq!(handler.process("3"), TagResult::Processed);
        assert_eq!(handler.process(" 2 "), TagResult::Processed);
    }

    #[test]
    fn wrapping_style_handler_invalid() {
        let handler = WrappingStyleTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("4"), TagResult::Failed(_)));
        assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("4") {
            assert_eq!(msg, "Wrapping style tag requires value 0-3");
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(AlignmentTagHandler.name(), "a");
        assert_eq!(NumpadAlignmentTagHandler.name(), "an");
        assert_eq!(WrappingStyleTagHandler.name(), "q");
    }

    #[test]
    fn create_alignment_handlers_returns_all() {
        let handlers = create_alignment_handlers();
        assert_eq!(handlers.len(), 3);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"an"));
        assert!(names.contains(&"q"));
    }

    #[test]
    fn alignment_validation() {
        let handler = AlignmentTagHandler;
        // Valid
        assert!(handler.validate("1"));
        assert!(handler.validate("2"));
        assert!(handler.validate("3"));
        assert!(handler.validate("5"));
        assert!(handler.validate("6"));
        assert!(handler.validate("7"));
        assert!(handler.validate("9"));
        assert!(handler.validate("10"));
        assert!(handler.validate("11"));
        // Invalid
        assert!(!handler.validate("0"));
        assert!(!handler.validate("4"));
        assert!(!handler.validate("8"));
        assert!(!handler.validate("12"));
        assert!(!handler.validate(""));
    }

    #[test]
    fn numpad_alignment_validation() {
        let handler = NumpadAlignmentTagHandler;
        // Valid
        for i in 1..=9 {
            assert!(handler.validate(&i.to_string()));
        }
        // Invalid
        assert!(!handler.validate("0"));
        assert!(!handler.validate("10"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("abc"));
    }

    #[test]
    fn wrapping_style_validation() {
        let handler = WrappingStyleTagHandler;
        // Valid
        assert!(handler.validate("0"));
        assert!(handler.validate("1"));
        assert!(handler.validate("2"));
        assert!(handler.validate("3"));
        // Invalid
        assert!(!handler.validate("4"));
        assert!(!handler.validate("-1"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("abc"));
    }

    #[test]
    fn whitespace_handling() {
        let alignment = AlignmentTagHandler;
        let numpad = NumpadAlignmentTagHandler;
        let wrapping = WrappingStyleTagHandler;

        assert_eq!(alignment.process(" 1 "), TagResult::Processed);
        assert_eq!(numpad.process(" 5 "), TagResult::Processed);
        assert_eq!(wrapping.process(" 2 "), TagResult::Processed);

        assert_eq!(alignment.process("\t3\t"), TagResult::Processed);
        assert_eq!(numpad.process("\t9\t"), TagResult::Processed);
        assert_eq!(wrapping.process("\t0\t"), TagResult::Processed);
    }

    #[test]
    fn alignment_semantics() {
        let handler = AlignmentTagHandler;

        // Bottom alignments (1-3)
        assert_eq!(handler.process("1"), TagResult::Processed); // Bottom-left
        assert_eq!(handler.process("2"), TagResult::Processed); // Bottom-center
        assert_eq!(handler.process("3"), TagResult::Processed); // Bottom-right

        // Top alignments (5-7)
        assert_eq!(handler.process("5"), TagResult::Processed); // Top-left
        assert_eq!(handler.process("6"), TagResult::Processed); // Top-center
        assert_eq!(handler.process("7"), TagResult::Processed); // Top-right

        // Middle alignments (9-11)
        assert_eq!(handler.process("9"), TagResult::Processed); // Middle-left
        assert_eq!(handler.process("10"), TagResult::Processed); // Middle-center
        assert_eq!(handler.process("11"), TagResult::Processed); // Middle-right
    }
}
