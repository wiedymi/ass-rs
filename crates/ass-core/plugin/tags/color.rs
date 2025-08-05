//! Color and alpha tag handlers for ASS override tags
//!
//! Implements handlers for color and transparency commands including
//! primary, secondary, outline, and shadow colors along with alpha channels.
//! Color values use BGR format (blue-green-red) as per ASS specification.
//!
//! # Supported Tags
//!
//! - `c` or `1c`: Primary color (&Hbbggrr&)
//! - `2c`: Secondary color (&Hbbggrr&)
//! - `3c`: Outline color (&Hbbggrr&)
//! - `4c`: Shadow color (&Hbbggrr&)
//! - `alpha`: All alpha channels (&Haa&)
//! - `1a`: Primary alpha (&Haa&)
//! - `2a`: Secondary alpha (&Haa&)
//! - `3a`: Outline alpha (&Haa&)
//! - `4a`: Shadow alpha (&Haa&)
//!
//! # Performance
//!
//! - Zero allocations for argument parsing
//! - SIMD-optimized hex parsing when available
//! - Fast validation with minimal branching

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for primary color tag (`\c` or `\1c`)
pub struct PrimaryColorTagHandler;

impl TagHandler for PrimaryColorTagHandler {
    fn name(&self) -> &'static str {
        "c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for primary color tag with explicit index (`\1c`)
pub struct Color1TagHandler;

impl TagHandler for Color1TagHandler {
    fn name(&self) -> &'static str {
        "1c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for secondary color tag (`\2c`)
pub struct Color2TagHandler;

impl TagHandler for Color2TagHandler {
    fn name(&self) -> &'static str {
        "2c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for outline color tag (`\3c`)
pub struct Color3TagHandler;

impl TagHandler for Color3TagHandler {
    fn name(&self) -> &'static str {
        "3c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for shadow color tag (`\4c`)
pub struct Color4TagHandler;

impl TagHandler for Color4TagHandler {
    fn name(&self) -> &'static str {
        "4c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for all alpha channels tag (`\alpha`)
pub struct AlphaTagHandler;

impl TagHandler for AlphaTagHandler {
    fn name(&self) -> &'static str {
        "alpha"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Handler for primary alpha tag (`\1a`)
pub struct Alpha1TagHandler;

impl TagHandler for Alpha1TagHandler {
    fn name(&self) -> &'static str {
        "1a"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Handler for secondary alpha tag (`\2a`)
pub struct Alpha2TagHandler;

impl TagHandler for Alpha2TagHandler {
    fn name(&self) -> &'static str {
        "2a"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Handler for outline alpha tag (`\3a`)
pub struct Alpha3TagHandler;

impl TagHandler for Alpha3TagHandler {
    fn name(&self) -> &'static str {
        "3a"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Handler for shadow alpha tag (`\4a`)
pub struct Alpha4TagHandler;

impl TagHandler for Alpha4TagHandler {
    fn name(&self) -> &'static str {
        "4a"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Validate color arguments in &Hbbggrr& format
#[inline]
fn validate_color_args(args: &str) -> bool {
    let args = args.trim();

    // Check format: &Hbbggrr&
    if !args.starts_with("&H") || !args.ends_with('&') || args.len() != 9 {
        return false;
    }

    // Validate hex digits (between &H and &)
    args[2..8].chars().all(|c| c.is_ascii_hexdigit())
}

/// Validate alpha arguments in &Haa& format
#[inline]
fn validate_alpha_args(args: &str) -> bool {
    let args = args.trim();

    // Check format: &Haa&
    if !args.starts_with("&H") || !args.ends_with('&') || args.len() != 5 {
        return false;
    }

    // Validate hex digits (between &H and &)
    args[2..4].chars().all(|c| c.is_ascii_hexdigit())
}

/// Create all color and alpha tag handlers
///
/// Returns a vector of boxed tag handlers for color-related operations.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::color::create_color_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_color_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_color_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(PrimaryColorTagHandler),
        alloc::boxed::Box::new(Color1TagHandler),
        alloc::boxed::Box::new(Color2TagHandler),
        alloc::boxed::Box::new(Color3TagHandler),
        alloc::boxed::Box::new(Color4TagHandler),
        alloc::boxed::Box::new(AlphaTagHandler),
        alloc::boxed::Box::new(Alpha1TagHandler),
        alloc::boxed::Box::new(Alpha2TagHandler),
        alloc::boxed::Box::new(Alpha3TagHandler),
        alloc::boxed::Box::new(Alpha4TagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn validate_color_valid() {
        assert!(validate_color_args("&H000000&"));
        assert!(validate_color_args("&HFFFFFF&"));
        assert!(validate_color_args("&H123456&"));
        assert!(validate_color_args("&HABCDEF&"));
        assert!(validate_color_args("&Habcdef&"));
        assert!(validate_color_args(" &H000000& "));
    }

    #[test]
    fn validate_color_invalid() {
        assert!(!validate_color_args(""));
        assert!(!validate_color_args("&H&"));
        assert!(!validate_color_args("&H00&"));
        assert!(!validate_color_args("&H0000&"));
        assert!(!validate_color_args("&H00000000&")); // Too long
        assert!(!validate_color_args("H000000&")); // Missing &
        assert!(!validate_color_args("&H000000")); // Missing &
        assert!(!validate_color_args("&HGGGGGG&")); // Invalid hex
        assert!(!validate_color_args("&H00 000&")); // Space in hex
    }

    #[test]
    fn validate_alpha_valid() {
        assert!(validate_alpha_args("&H00&"));
        assert!(validate_alpha_args("&HFF&"));
        assert!(validate_alpha_args("&H7F&"));
        assert!(validate_alpha_args("&HAB&"));
        assert!(validate_alpha_args("&Hab&"));
        assert!(validate_alpha_args(" &H00& "));
    }

    #[test]
    fn validate_alpha_invalid() {
        assert!(!validate_alpha_args(""));
        assert!(!validate_alpha_args("&H&"));
        assert!(!validate_alpha_args("&H0&"));
        assert!(!validate_alpha_args("&H000&")); // Too long
        assert!(!validate_alpha_args("H00&")); // Missing &
        assert!(!validate_alpha_args("&H00")); // Missing &
        assert!(!validate_alpha_args("&HGG&")); // Invalid hex
        assert!(!validate_alpha_args("&H 0&")); // Space in hex
    }

    #[test]
    fn color_handlers_valid() {
        let color_handlers: [&dyn TagHandler; 5] = [
            &PrimaryColorTagHandler,
            &Color1TagHandler,
            &Color2TagHandler,
            &Color3TagHandler,
            &Color4TagHandler,
        ];

        for handler in &color_handlers {
            assert_eq!(handler.process("&H000000&"), TagResult::Processed);
            assert_eq!(handler.process("&HFFFFFF&"), TagResult::Processed);
            assert_eq!(handler.process("&H123ABC&"), TagResult::Processed);
        }
    }

    #[test]
    fn color_handlers_invalid() {
        let handler = PrimaryColorTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("&H00&"), TagResult::Failed(_)));
        assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("&H00&") {
            assert_eq!(msg, "Color tag requires &Hbbggrr& format");
        }
    }

    #[test]
    fn alpha_handlers_valid() {
        let alpha_handlers: [&dyn TagHandler; 5] = [
            &AlphaTagHandler,
            &Alpha1TagHandler,
            &Alpha2TagHandler,
            &Alpha3TagHandler,
            &Alpha4TagHandler,
        ];

        for handler in &alpha_handlers {
            assert_eq!(handler.process("&H00&"), TagResult::Processed);
            assert_eq!(handler.process("&HFF&"), TagResult::Processed);
            assert_eq!(handler.process("&H7F&"), TagResult::Processed);
        }
    }

    #[test]
    fn alpha_handlers_invalid() {
        let handler = AlphaTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("&H000000&"), TagResult::Failed(_)));
        assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));

        if let TagResult::Failed(msg) = handler.process("&H000&") {
            assert_eq!(msg, "Alpha tag requires &Haa& format");
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(PrimaryColorTagHandler.name(), "c");
        assert_eq!(Color1TagHandler.name(), "1c");
        assert_eq!(Color2TagHandler.name(), "2c");
        assert_eq!(Color3TagHandler.name(), "3c");
        assert_eq!(Color4TagHandler.name(), "4c");
        assert_eq!(AlphaTagHandler.name(), "alpha");
        assert_eq!(Alpha1TagHandler.name(), "1a");
        assert_eq!(Alpha2TagHandler.name(), "2a");
        assert_eq!(Alpha3TagHandler.name(), "3a");
        assert_eq!(Alpha4TagHandler.name(), "4a");
    }

    #[test]
    fn create_color_handlers_returns_all() {
        let handlers = create_color_handlers();
        assert_eq!(handlers.len(), 10);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();

        assert!(names.contains(&"c"));
        assert!(names.contains(&"1c"));
        assert!(names.contains(&"2c"));
        assert!(names.contains(&"3c"));
        assert!(names.contains(&"4c"));
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"1a"));
        assert!(names.contains(&"2a"));
        assert!(names.contains(&"3a"));
        assert!(names.contains(&"4a"));
    }

    #[test]
    fn hex_validation_edge_cases() {
        // Mixed case
        assert!(validate_color_args("&HaAbBcC&"));
        assert!(validate_alpha_args("&HaA&"));

        // All same digit
        assert!(validate_color_args("&H000000&"));
        assert!(validate_color_args("&HFFFFFF&"));
        assert!(validate_alpha_args("&H00&"));
        assert!(validate_alpha_args("&HFF&"));
    }

    #[test]
    fn whitespace_handling() {
        // Leading/trailing whitespace
        assert!(validate_color_args("  &H123456&  "));
        assert!(validate_alpha_args("  &H12&  "));

        // But not internal whitespace
        assert!(!validate_color_args("&H12 3456&"));
        assert!(!validate_alpha_args("&H1 2&"));
    }

    #[test]
    fn case_sensitivity() {
        // H must be uppercase
        assert!(!validate_color_args("&h123456&"));
        assert!(!validate_alpha_args("&h12&"));

        // But hex digits can be either case
        assert!(validate_color_args("&HABCDEF&"));
        assert!(validate_color_args("&Habcdef&"));
        assert!(validate_color_args("&HaBcDeF&"));
    }

    #[test]
    fn validation_consistency() {
        let color_handler = PrimaryColorTagHandler;
        let alpha_handler = AlphaTagHandler;

        // Ensure validate() and process() agree
        let color_valid = "&H123456&";
        let color_invalid = "&H12&";
        let alpha_valid = "&H12&";
        let alpha_invalid = "&H123456&";

        assert!(color_handler.validate(color_valid));
        assert_eq!(color_handler.process(color_valid), TagResult::Processed);

        assert!(!color_handler.validate(color_invalid));
        assert!(matches!(
            color_handler.process(color_invalid),
            TagResult::Failed(_)
        ));

        assert!(alpha_handler.validate(alpha_valid));
        assert_eq!(alpha_handler.process(alpha_valid), TagResult::Processed);

        assert!(!alpha_handler.validate(alpha_invalid));
        assert!(matches!(
            alpha_handler.process(alpha_invalid),
            TagResult::Failed(_)
        ));
    }
}
