//! Karaoke effect tag handlers for ASS override tags
//!
//! Implements handlers for karaoke timing tags like `\k`, `\kf`, `\ko`,
//! and the v4++ `\kt` tag for absolute timing.
//!
//! # Karaoke Tags
//!
//! - `\kt` - Absolute karaoke timing (v4++)
//!
//! # Examples
//!
//! ```rust
//! use ass_core::plugin::{ExtensionRegistry, tags::karaoke::create_karaoke_handlers};
//!
//! let mut registry = ExtensionRegistry::new();
//! for handler in create_karaoke_handlers() {
//!     registry.register_tag_handler(handler).unwrap();
//! }
//! ```

use crate::plugin::{TagHandler, TagResult};
use alloc::{boxed::Box, string::String, vec::Vec};

/// Handler for absolute karaoke timing tag (`\kt`)
///
/// This tag was introduced in the v4++ spec for absolute karaoke timing.
/// It expects an integer duration in centiseconds.
///
/// # Examples
///
/// ```rust
/// use ass_core::plugin::tags::karaoke::KaraokeTimingTagHandler;
/// use ass_core::plugin::{TagHandler, TagResult};
///
/// let handler = KaraokeTimingTagHandler;
/// assert_eq!(handler.process("500"), TagResult::Processed);
/// assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
/// ```
pub struct KaraokeTimingTagHandler;

impl TagHandler for KaraokeTimingTagHandler {
    fn name(&self) -> &'static str {
        "kt"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Karaoke timing tag `kt` expects an integer duration in centiseconds",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().parse::<u32>().is_ok()
    }
}

/// Create all karaoke tag handlers
///
/// Returns a vector of boxed tag handlers for all supported karaoke tags.
/// Currently includes only the `\kt` tag handler for v4++ compatibility.
///
/// # Examples
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::karaoke::create_karaoke_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_karaoke_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
///
/// // Test that kt handler is registered
/// assert!(registry.has_tag_handler("kt"));
/// ```
#[must_use]
pub fn create_karaoke_handlers() -> Vec<Box<dyn TagHandler>> {
    vec![Box::new(KaraokeTimingTagHandler)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kt_handler_name() {
        let handler = KaraokeTimingTagHandler;
        assert_eq!(handler.name(), "kt");
    }

    #[test]
    fn kt_handler_valid_args() {
        let handler = KaraokeTimingTagHandler;
        assert_eq!(handler.process("500"), TagResult::Processed);
        assert!(handler.validate("500"));
    }

    #[test]
    fn kt_handler_valid_zero() {
        let handler = KaraokeTimingTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert!(handler.validate("0"));
    }

    #[test]
    fn kt_handler_valid_large_number() {
        let handler = KaraokeTimingTagHandler;
        assert_eq!(handler.process("999999"), TagResult::Processed);
        assert!(handler.validate("999999"));
    }

    #[test]
    fn kt_handler_invalid_args() {
        let handler = KaraokeTimingTagHandler;
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
        assert!(!handler.validate("abc"));
    }

    #[test]
    fn kt_handler_invalid_negative() {
        let handler = KaraokeTimingTagHandler;
        assert!(matches!(handler.process("-100"), TagResult::Failed(_)));
        assert!(!handler.validate("-100"));
    }

    #[test]
    fn kt_handler_invalid_float() {
        let handler = KaraokeTimingTagHandler;
        assert!(matches!(handler.process("123.45"), TagResult::Failed(_)));
        assert!(!handler.validate("123.45"));
    }

    #[test]
    fn kt_handler_invalid_empty() {
        let handler = KaraokeTimingTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(!handler.validate(""));
    }

    #[test]
    fn kt_handler_invalid_whitespace_only() {
        let handler = KaraokeTimingTagHandler;
        assert!(matches!(handler.process("   "), TagResult::Failed(_)));
        assert!(!handler.validate("   "));
    }

    #[test]
    fn kt_handler_whitespace_trimming() {
        let handler = KaraokeTimingTagHandler;
        assert_eq!(handler.process("  500  "), TagResult::Processed);
        assert!(handler.validate("  500  "));
    }

    #[test]
    fn create_karaoke_handlers_contains_kt() {
        let handlers = create_karaoke_handlers();
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].name(), "kt");
    }

    #[test]
    fn create_karaoke_handlers_all_functional() {
        let handlers = create_karaoke_handlers();

        for handler in &handlers {
            // Test valid input
            assert_eq!(handler.process("100"), TagResult::Processed);

            // Test invalid input
            assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
        }
    }
}
