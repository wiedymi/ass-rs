//! Miscellaneous tag handlers for ASS override tags
//!
//! Implements handlers for various utility tags including style reset,
//! rotation origin, and short-form rotation.
//!
//! # Supported Tags
//!
//! - `r`: Reset to style or default
//! - `fr`: Short form Z-axis rotation (alias for frz)
//! - `org`: Set rotation/transformation origin point
//!
//! # Performance
//!
//! - Zero allocations for simple tags
//! - Fast validation
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for reset tag (`\r`)
///
/// Resets override tags to either:
/// - Default style (no argument)
/// - Named style (with style name argument)
pub struct ResetTagHandler;

impl TagHandler for ResetTagHandler {
    fn name(&self) -> &'static str {
        "r"
    }

    fn process(&self, _args: &str) -> TagResult {
        // Reset accepts empty (reset to default) or style name
        TagResult::Processed
    }

    fn validate(&self, _args: &str) -> bool {
        // Any argument is valid (empty = default, non-empty = style name)
        true
    }
}

/// Handler for short-form rotation tag (`\fr`)
///
/// Alias for `\frz` - rotates text around Z-axis.
pub struct ShortRotationTagHandler;

impl TagHandler for ShortRotationTagHandler {
    fn name(&self) -> &'static str {
        "fr"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Rotation tag requires numeric degrees"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Must be a valid number (positive or negative)
        is_numeric(args)
    }
}

/// Handler for rotation origin tag (`\org`)
///
/// Sets the origin point for rotation transformations.
/// Format: `\org(x,y)`
pub struct OriginTagHandler;

impl TagHandler for OriginTagHandler {
    fn name(&self) -> &'static str {
        "org"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Origin tag requires (x,y) coordinates"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();

        // Must have parentheses
        if !args.starts_with('(') || !args.ends_with(')') {
            return false;
        }

        // Extract content between parentheses
        let content = &args[1..args.len() - 1];
        let parts: alloc::vec::Vec<&str> = content.split(',').map(str::trim).collect();

        // Must have exactly 2 coordinates
        if parts.len() != 2 {
            return false;
        }

        // Both must be numeric
        parts.iter().all(|part| is_numeric(part))
    }
}

/// Validate if a string represents a valid number
#[inline]
fn is_numeric(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // Check for optional sign
    let has_sign = first == '-' || first == '+';
    if has_sign && s.len() == 1 {
        return false;
    }

    let mut has_decimal = false;
    let start_idx = usize::from(has_sign);

    for (i, c) in s.chars().enumerate().skip(start_idx) {
        match c {
            '0'..='9' => {}
            '.' => {
                if has_decimal || i == start_idx || i == s.len() - 1 {
                    return false;
                }
                has_decimal = true;
            }
            _ => return false,
        }
    }

    true
}

/// Create all miscellaneous tag handlers
///
/// Returns a vector of boxed tag handlers for misc operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::misc::create_misc_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_misc_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_misc_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(ResetTagHandler),
        alloc::boxed::Box::new(ShortRotationTagHandler),
        alloc::boxed::Box::new(OriginTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_handler_valid() {
        let handler = ResetTagHandler;
        // Empty (reset to default)
        assert_eq!(handler.process(""), TagResult::Processed);
        // Style name
        assert_eq!(handler.process("Default"), TagResult::Processed);
        assert_eq!(handler.process("Main"), TagResult::Processed);
        assert_eq!(handler.process("Subtitle-Style"), TagResult::Processed);
        assert_eq!(handler.process(" Karaoke "), TagResult::Processed);
    }

    #[test]
    fn reset_handler_always_valid() {
        let handler = ResetTagHandler;
        // Reset accepts any input
        assert!(handler.validate(""));
        assert!(handler.validate("StyleName"));
        assert!(handler.validate("123"));
        assert!(handler.validate("Style With Spaces"));
    }

    #[test]
    fn short_rotation_handler_valid() {
        let handler = ShortRotationTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("90"), TagResult::Processed);
        assert_eq!(handler.process("-90"), TagResult::Processed);
        assert_eq!(handler.process("360"), TagResult::Processed);
        assert_eq!(handler.process("45.5"), TagResult::Processed);
        assert_eq!(handler.process(" 180 "), TagResult::Processed);
    }

    #[test]
    fn short_rotation_handler_invalid() {
        let handler = ShortRotationTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
        assert!(matches!(handler.process("90deg"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Rotation tag requires numeric degrees");
        }
    }

    #[test]
    fn origin_handler_valid() {
        let handler = OriginTagHandler;
        assert_eq!(handler.process("(100,200)"), TagResult::Processed);
        assert_eq!(handler.process("(0,0)"), TagResult::Processed);
        assert_eq!(handler.process("(-50,100)"), TagResult::Processed);
        assert_eq!(handler.process("(640,360)"), TagResult::Processed);
        assert_eq!(handler.process("(100.5,200.5)"), TagResult::Processed);
        assert_eq!(handler.process("( 50 , 100 )"), TagResult::Processed);
    }

    #[test]
    fn origin_handler_invalid() {
        let handler = OriginTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("100,200"), TagResult::Failed(_))); // No parentheses
        assert!(matches!(handler.process("(100)"), TagResult::Failed(_))); // Only one coord
        assert!(matches!(
            handler.process("(100,200,300)"),
            TagResult::Failed(_)
        )); // Too many
        assert!(matches!(handler.process("(abc,def)"), TagResult::Failed(_))); // Non-numeric

        if let TagResult::Failed(msg) = handler.process("no_parens") {
            assert_eq!(msg, "Origin tag requires (x,y) coordinates");
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(ResetTagHandler.name(), "r");
        assert_eq!(ShortRotationTagHandler.name(), "fr");
        assert_eq!(OriginTagHandler.name(), "org");
    }

    #[test]
    fn create_misc_handlers_returns_all() {
        let handlers = create_misc_handlers();
        assert_eq!(handlers.len(), 3);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
        assert!(names.contains(&"r"));
        assert!(names.contains(&"fr"));
        assert!(names.contains(&"org"));
    }

    #[test]
    fn reset_validation() {
        let handler = ResetTagHandler;
        // Always returns true
        assert!(handler.validate(""));
        assert!(handler.validate("Default"));
        assert!(handler.validate("Any String"));
        assert!(handler.validate("123"));
        assert!(handler.validate("!@#$%^&*()"));
    }

    #[test]
    fn short_rotation_validation() {
        let handler = ShortRotationTagHandler;
        assert!(handler.validate("0"));
        assert!(handler.validate("90"));
        assert!(handler.validate("-180"));
        assert!(handler.validate("45.5"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("abc"));
        assert!(!handler.validate("90deg"));
    }

    #[test]
    fn origin_validation() {
        let handler = OriginTagHandler;
        assert!(handler.validate("(0,0)"));
        assert!(handler.validate("(100,200)"));
        assert!(handler.validate("(-50,50)"));
        assert!(handler.validate("(1.5,2.5)"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("0,0")); // No parentheses
        assert!(!handler.validate("(0)")); // Too few
        assert!(!handler.validate("(0,0,0)")); // Too many
    }

    #[test]
    fn is_numeric_edge_cases() {
        assert!(is_numeric("0"));
        assert!(is_numeric("-0"));
        assert!(is_numeric("+0"));
        assert!(is_numeric("123"));
        assert!(is_numeric("-123"));
        assert!(is_numeric("123.45"));
        assert!(is_numeric("0.001"));
        assert!(is_numeric("999999"));

        assert!(!is_numeric(""));
        assert!(!is_numeric("-"));
        assert!(!is_numeric("."));
        assert!(!is_numeric("123."));
        assert!(!is_numeric(".123"));
        assert!(!is_numeric("1.2.3"));
        assert!(!is_numeric("1e5")); // No scientific notation
    }

    #[test]
    fn reset_style_names() {
        let handler = ResetTagHandler;
        // Various valid style names
        assert_eq!(handler.process("Default"), TagResult::Processed);
        assert_eq!(handler.process("Main-Style"), TagResult::Processed);
        assert_eq!(handler.process("Style_123"), TagResult::Processed);
        assert_eq!(handler.process("日本語スタイル"), TagResult::Processed); // Unicode
        assert_eq!(handler.process("Style With Spaces"), TagResult::Processed);
    }

    #[test]
    fn origin_coordinate_ranges() {
        let handler = OriginTagHandler;
        // Screen coordinates
        assert_eq!(handler.process("(0,0)"), TagResult::Processed); // Top-left
        assert_eq!(handler.process("(1920,1080)"), TagResult::Processed); // Full HD
        assert_eq!(handler.process("(3840,2160)"), TagResult::Processed); // 4K
                                                                          // Negative coordinates (off-screen)
        assert_eq!(handler.process("(-100,-100)"), TagResult::Processed);
        // Center points
        assert_eq!(handler.process("(640,360)"), TagResult::Processed); // 720p center
        assert_eq!(handler.process("(960,540)"), TagResult::Processed); // 1080p center
    }
}
