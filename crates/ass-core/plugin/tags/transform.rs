//! Transform and rotation tag handlers for ASS override tags
//!
//! Implements handlers for transformation commands including rotation,
//! scaling, shearing, and spacing. These handlers validate numeric
//! arguments and handle both integer and decimal values.
//!
//! # Supported Tags
//!
//! - `frz`: Z-axis rotation (degrees)
//! - `frx`: X-axis rotation (degrees)
//! - `fry`: Y-axis rotation (degrees)
//! - `fscx`: X-axis scale (percent)
//! - `fscy`: Y-axis scale (percent)
//! - `fax`: X-axis shear factor
//! - `fay`: Y-axis shear factor
//! - `fsp`: Letter spacing (pixels)
//!
//! # Performance
//!
//! - Zero allocations for argument parsing
//! - Fast numeric validation
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for Z-axis rotation tag (`\frz`)
pub struct RotationZTagHandler;

impl TagHandler for RotationZTagHandler {
    fn name(&self) -> &'static str {
        "frz"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Rotation tag requires numeric degrees"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for X-axis rotation tag (`\frx`)
pub struct RotationXTagHandler;

impl TagHandler for RotationXTagHandler {
    fn name(&self) -> &'static str {
        "frx"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Rotation tag requires numeric degrees"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for Y-axis rotation tag (`\fry`)
pub struct RotationYTagHandler;

impl TagHandler for RotationYTagHandler {
    fn name(&self) -> &'static str {
        "fry"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Rotation tag requires numeric degrees"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for X-axis scale tag (`\fscx`)
pub struct ScaleXTagHandler;

impl TagHandler for ScaleXTagHandler {
    fn name(&self) -> &'static str {
        "fscx"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Scale tag requires numeric percent"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for Y-axis scale tag (`\fscy`)
pub struct ScaleYTagHandler;

impl TagHandler for ScaleYTagHandler {
    fn name(&self) -> &'static str {
        "fscy"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Scale tag requires numeric percent"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for X-axis shear tag (`\fax`)
pub struct ShearXTagHandler;

impl TagHandler for ShearXTagHandler {
    fn name(&self) -> &'static str {
        "fax"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Shear tag requires numeric factor"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for Y-axis shear tag (`\fay`)
pub struct ShearYTagHandler;

impl TagHandler for ShearYTagHandler {
    fn name(&self) -> &'static str {
        "fay"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Shear tag requires numeric factor"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for letter spacing tag (`\fsp`)
pub struct SpacingTagHandler;

impl TagHandler for SpacingTagHandler {
    fn name(&self) -> &'static str {
        "fsp"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Spacing tag requires numeric pixels"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Validate if argument is a valid number (integer or decimal)
#[inline]
fn validate_numeric_arg(args: &str) -> bool {
    let args = args.trim();
    if args.is_empty() {
        return false;
    }

    let mut chars = args.chars();
    let first = chars.next().unwrap();

    // Check for optional sign
    let has_sign = first == '-' || first == '+';
    if has_sign && args.len() == 1 {
        return false;
    }

    let mut has_decimal = false;
    let start_idx = usize::from(has_sign);

    for (i, c) in args.chars().enumerate().skip(start_idx) {
        match c {
            '0'..='9' => {}
            '.' => {
                // No leading/trailing dot, only one decimal point
                if has_decimal || i == start_idx || i == args.len() - 1 {
                    return false;
                }
                has_decimal = true;
            }
            _ => return false,
        }
    }

    true
}

/// Create all transform and rotation tag handlers
///
/// Returns a vector of boxed tag handlers for transformation operations.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::transform::create_transform_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_transform_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_transform_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(RotationZTagHandler),
        alloc::boxed::Box::new(RotationXTagHandler),
        alloc::boxed::Box::new(RotationYTagHandler),
        alloc::boxed::Box::new(ScaleXTagHandler),
        alloc::boxed::Box::new(ScaleYTagHandler),
        alloc::boxed::Box::new(ShearXTagHandler),
        alloc::boxed::Box::new(ShearYTagHandler),
        alloc::boxed::Box::new(SpacingTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_numeric_valid() {
        assert!(validate_numeric_arg("0"));
        assert!(validate_numeric_arg("123"));
        assert!(validate_numeric_arg("-123"));
        assert!(validate_numeric_arg("+123"));
        assert!(validate_numeric_arg("123.45"));
        assert!(validate_numeric_arg("-123.45"));
        assert!(validate_numeric_arg("0.0"));
        assert!(!validate_numeric_arg(".5")); // Invalid - no leading digit
    }

    #[test]
    fn validate_numeric_invalid() {
        assert!(!validate_numeric_arg(""));
        assert!(!validate_numeric_arg("-"));
        assert!(!validate_numeric_arg("+"));
        assert!(!validate_numeric_arg("."));
        assert!(!validate_numeric_arg("123."));
        assert!(!validate_numeric_arg(".123"));
        assert!(!validate_numeric_arg("12.34.56"));
        assert!(!validate_numeric_arg("abc"));
        assert!(!validate_numeric_arg("123abc"));
        assert!(!validate_numeric_arg("12 3"));
        assert!(!validate_numeric_arg("1e5")); // No scientific notation
    }

    #[test]
    fn rotation_handlers_valid() {
        let handlers: [&dyn TagHandler; 3] = [
            &RotationZTagHandler,
            &RotationXTagHandler,
            &RotationYTagHandler,
        ];

        for handler in &handlers {
            assert_eq!(handler.process("0"), TagResult::Processed);
            assert_eq!(handler.process("90"), TagResult::Processed);
            assert_eq!(handler.process("-90"), TagResult::Processed);
            assert_eq!(handler.process("360"), TagResult::Processed);
            assert_eq!(handler.process("45.5"), TagResult::Processed);
        }
    }

    #[test]
    fn rotation_handlers_invalid() {
        let handler = RotationZTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc"), TagResult::Failed(_)));
        assert!(matches!(handler.process("90deg"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Rotation tag requires numeric degrees");
        }
    }

    #[test]
    fn scale_handlers_valid() {
        let handlers: [&dyn TagHandler; 2] = [&ScaleXTagHandler, &ScaleYTagHandler];

        for handler in &handlers {
            assert_eq!(handler.process("100"), TagResult::Processed);
            assert_eq!(handler.process("0"), TagResult::Processed);
            assert_eq!(handler.process("200"), TagResult::Processed);
            assert_eq!(handler.process("50.5"), TagResult::Processed);
            assert_eq!(handler.process("-100"), TagResult::Processed); // Negative scale flips
        }
    }

    #[test]
    fn scale_handlers_invalid() {
        let handler = ScaleXTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("100%"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Scale tag requires numeric percent");
        }
    }

    #[test]
    fn shear_handlers_valid() {
        let handlers: [&dyn TagHandler; 2] = [&ShearXTagHandler, &ShearYTagHandler];

        for handler in &handlers {
            assert_eq!(handler.process("0"), TagResult::Processed);
            assert_eq!(handler.process("0.5"), TagResult::Processed);
            assert_eq!(handler.process("-0.5"), TagResult::Processed);
            assert_eq!(handler.process("2"), TagResult::Processed);
        }
    }

    #[test]
    fn shear_handlers_invalid() {
        let handler = ShearXTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Shear tag requires numeric factor");
        }
    }

    #[test]
    fn spacing_handler_valid() {
        let handler = SpacingTagHandler;
        assert_eq!(handler.process("0"), TagResult::Processed);
        assert_eq!(handler.process("5"), TagResult::Processed);
        assert_eq!(handler.process("-2"), TagResult::Processed);
        assert_eq!(handler.process("1.5"), TagResult::Processed);
    }

    #[test]
    fn spacing_handler_invalid() {
        let handler = SpacingTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("5px"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("invalid") {
            assert_eq!(msg, "Spacing tag requires numeric pixels");
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(RotationZTagHandler.name(), "frz");
        assert_eq!(RotationXTagHandler.name(), "frx");
        assert_eq!(RotationYTagHandler.name(), "fry");
        assert_eq!(ScaleXTagHandler.name(), "fscx");
        assert_eq!(ScaleYTagHandler.name(), "fscy");
        assert_eq!(ShearXTagHandler.name(), "fax");
        assert_eq!(ShearYTagHandler.name(), "fay");
        assert_eq!(SpacingTagHandler.name(), "fsp");
    }

    #[test]
    fn create_transform_handlers_returns_all() {
        let handlers = create_transform_handlers();
        assert_eq!(handlers.len(), 8);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();

        assert!(names.contains(&"frz"));
        assert!(names.contains(&"frx"));
        assert!(names.contains(&"fry"));
        assert!(names.contains(&"fscx"));
        assert!(names.contains(&"fscy"));
        assert!(names.contains(&"fax"));
        assert!(names.contains(&"fay"));
        assert!(names.contains(&"fsp"));
    }

    #[test]
    fn numeric_edge_cases() {
        // Very large numbers
        assert!(validate_numeric_arg("999999999"));
        assert!(validate_numeric_arg("-999999999"));

        // Very small decimals
        assert!(validate_numeric_arg("0.00001"));
        assert!(validate_numeric_arg("-0.00001"));

        // Zero variations
        assert!(validate_numeric_arg("0"));
        assert!(validate_numeric_arg("0.0"));
        assert!(validate_numeric_arg("-0"));
        assert!(validate_numeric_arg("+0"));
    }

    #[test]
    fn whitespace_handling() {
        assert!(validate_numeric_arg(" 123 "));
        assert!(validate_numeric_arg("\t456\t"));
        assert!(validate_numeric_arg(" -789 "));

        // All transform handlers should handle whitespace
        let handler = RotationZTagHandler;
        assert_eq!(handler.process(" 90 "), TagResult::Processed);
    }

    #[test]
    fn validation_consistency() {
        let handlers = create_transform_handlers();

        for handler in &handlers {
            // Ensure validate() and process() agree
            let valid = "123.45";
            let invalid = "abc";

            assert!(handler.validate(valid));
            assert_eq!(handler.process(valid), TagResult::Processed);

            assert!(!handler.validate(invalid));
            assert!(matches!(handler.process(invalid), TagResult::Failed(_)));
        }
    }

    #[test]
    fn special_transform_values() {
        // Test special rotation values
        let rot_handler = RotationZTagHandler;
        assert_eq!(rot_handler.process("0"), TagResult::Processed); // No rotation
        assert_eq!(rot_handler.process("360"), TagResult::Processed); // Full rotation
        assert_eq!(rot_handler.process("-360"), TagResult::Processed); // Reverse full
        assert_eq!(rot_handler.process("720"), TagResult::Processed); // Double rotation

        // Test special scale values
        let scale_handler = ScaleXTagHandler;
        assert_eq!(scale_handler.process("0"), TagResult::Processed); // Invisible
        assert_eq!(scale_handler.process("100"), TagResult::Processed); // Normal
        assert_eq!(scale_handler.process("-100"), TagResult::Processed); // Flipped

        // Test special shear values
        let shear_handler = ShearXTagHandler;
        assert_eq!(shear_handler.process("0"), TagResult::Processed); // No shear
    }
}
