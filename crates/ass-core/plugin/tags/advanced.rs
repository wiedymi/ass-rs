//! Advanced formatting tag handlers for ASS override tags
//!
//! Implements handlers for advanced text formatting including borders,
//! shadows, and edge blur effects. These handlers validate numeric
//! arguments according to ASS specifications.
//!
//! # Supported Tags
//!
//! - `bord`: Border width (pixels)
//! - `shad`: Shadow depth (pixels)
//! - `be`: Blur edges (0/1)
//!
//! # Performance
//!
//! - Zero allocations for validation
//! - Fast numeric validation
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for border width tag (`\bord`)
///
/// Sets the border/outline width in pixels. Must be non-negative.
pub struct BorderTagHandler;

impl TagHandler for BorderTagHandler {
    fn name(&self) -> &'static str {
        "bord"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Border tag requires non-negative numeric width",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Parse as non-negative number
        args.parse::<f32>().is_ok_and(|width| width >= 0.0)
    }
}

/// Handler for shadow depth tag (`\shad`)
///
/// Sets the shadow depth in pixels. Must be non-negative.
pub struct ShadowTagHandler;

impl TagHandler for ShadowTagHandler {
    fn name(&self) -> &'static str {
        "shad"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Shadow tag requires non-negative numeric depth",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Parse as non-negative number
        args.parse::<f32>().is_ok_and(|depth| depth >= 0.0)
    }
}

/// Handler for blur edges tag (`\be`)
///
/// Enables or disables edge blur effect (0 or 1).
pub struct BlurEdgesTagHandler;

impl TagHandler for BlurEdgesTagHandler {
    fn name(&self) -> &'static str {
        "be"
    }

    fn process(&self, args: &str) -> TagResult {
        match args.trim() {
            "0" | "1" => TagResult::Processed,
            _ => TagResult::Failed(String::from("Blur edges tag accepts only 0 or 1"))}
    }

    fn validate(&self, args: &str) -> bool {
        let trimmed = args.trim();
        trimmed == "0" || trimmed == "1"
    }
}

/// Create all advanced formatting tag handlers
///
/// Returns a vector of boxed tag handlers for advanced formatting operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::advanced::create_advanced_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_advanced_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_advanced_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(BorderTagHandler),
        alloc::boxed::Box::new(ShadowTagHandler),
        alloc::boxed::Box::new(BlurEdgesTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[test]
    fn border_handler_valid() {
        let handler = BorderTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process("2.5"), TagResult::Processed);
        assert_eq!(handler.process("10"), TagResult::Processed);
        assert_eq!(handler.process(" 3 "), TagResult::Processed);
    }

    #[test]
    fn border_handler_invalid() {
        let handler = BorderTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
        assert!(matches!(handler.process("2px"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("-1") {
            assert_eq!(msg, "Border tag requires non-negative numeric width");
        }
    }

    #[test]
    fn shadow_handler_valid() {
        let handler = ShadowTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process("2.5"), TagResult::Processed);
        assert_eq!(handler.process("10"), TagResult::Processed);
        assert_eq!(handler.process(" 3 "), TagResult::Processed);
    }

    #[test]
    fn shadow_handler_invalid() {
        let handler = ShadowTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("-1"), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
        assert!(matches!(handler.process("2px"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("-1") {
            assert_eq!(msg, "Shadow tag requires non-negative numeric depth");
        }
    }

    #[test]
    fn blur_edges_handler_valid() {
        let handler = BlurEdgesTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("1"), TagResult::Processed);
        assert_eq!(handler.process(" 0 "), TagResult::Processed);
        assert_eq!(handler.process(" 1 "), TagResult::Processed);
    }

    #[test]
    fn blur_edges_handler_invalid() {
        let handler = BlurEdgesTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("2"), TagResult::Failed(_)));
        assert!(matches!(handler.process("true"), TagResult::Failed(_)));
        assert!(matches!(handler.process("on"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("2") {
            assert_eq!(msg, "Blur edges tag accepts only 0 or 1");
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(BorderTagHandler.name(), "bord");
        assert_eq!(ShadowTagHandler.name(), "shad");
        assert_eq!(BlurEdgesTagHandler.name(), "be");
    }

    #[test]
    fn create_advanced_handlers_returns_all() {
        let handlers = create_advanced_handlers();
        assert_eq!(handlers.len(), 3);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
        assert!(names.contains(&"bord"));
        assert!(names.contains(&"shad"));
        assert!(names.contains(&"be"));
    }

    #[test]
    fn border_validation() {
        let handler = BorderTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("1.5"));
        assert!(handler.validate("100"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("-1"));
        assert!(!handler.validate("abc"));
    }

    #[test]
    fn shadow_validation() {
        let handler = ShadowTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("1.5"));
        assert!(handler.validate("100"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("-1"));
        assert!(!handler.validate("abc"));
    }

    #[test]
    fn blur_edges_validation() {
        let handler = BlurEdgesTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("1"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("2"));
        assert!(!handler.validate("true"));
    }

    #[test]
    fn edge_cases() {
        let border_handler = BorderTagHandler;
        let shadow_handler = ShadowTagHandler;

        // Zero values
        assert_eq!(border_handler.process("0"), TagResult::Processed);
        assert_eq!(shadow_handler.process("0"), TagResult::Processed);

        // Decimal values
        assert_eq!(border_handler.process("0.1"), TagResult::Processed);
        assert_eq!(shadow_handler.process("0.1"), TagResult::Processed);

        // Large values
        assert_eq!(border_handler.process("999"), TagResult::Processed);
        assert_eq!(shadow_handler.process("999"), TagResult::Processed);

        // Very precise decimals
        assert_eq!(border_handler.process("1.23456"), TagResult::Processed);
        assert_eq!(shadow_handler.process("1.23456"), TagResult::Processed);
    }

    #[test]
    fn whitespace_handling() {
        let handlers = create_advanced_handlers();

        // Border and shadow accept numeric with whitespace
        assert_eq!(handlers[0].process(" 2 "), TagResult::Processed);
        assert_eq!(handlers[1].process(" 3 "), TagResult::Processed);

        // Blur edges accepts 0/1 with whitespace
        assert_eq!(handlers[2].process(" 0 "), TagResult::Processed);
        assert_eq!(handlers[2].process(" 1 "), TagResult::Processed);
    }
}
