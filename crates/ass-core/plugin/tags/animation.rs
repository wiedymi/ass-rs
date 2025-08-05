//! Animation and effect tag handlers for ASS override tags
//!
//! Implements handlers for animation and transition effects including
//! transforms, fading, and alpha animations. These are some of the most
//! complex tags in the ASS format.
//!
//! # Supported Tags
//!
//! - `t`: Animated style transformations
//! - `fade`: Complex alpha animation
//! - `fad`: Simple fade in/out
//!
//! # Performance
//!
//! - Efficient argument parsing
//! - Complex validation for multi-parameter tags
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for transform animation tag (`\t`)
///
/// Animates style modifiers over time. Format:
/// - `\t(<modifiers>)` - Transform over entire subtitle duration
/// - `\t(t1,t2,<modifiers>)` - Transform between t1 and t2
/// - `\t(t1,t2,accel,<modifiers>)` - With acceleration factor
pub struct TransformTagHandler;

impl TagHandler for TransformTagHandler {
    fn name(&self) -> &'static str {
        "t"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Transform tag requires (modifiers) or (t1,t2,[accel,]modifiers)",
            ))
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
        if content.is_empty() {
            return false;
        }

        // Check if it contains modifiers (simplified validation)
        // In real implementation, would parse timing and validate modifiers
        // For now, just ensure it's not empty and has some content
        !content.trim().is_empty()
    }
}

/// Handler for complex fade animation tag (`\fade`)
///
/// Animates alpha transparency with 7 parameters:
/// `\fade(a1,a2,a3,t1,t2,t3,t4)`
/// - a1,a2,a3: Alpha values (0-255)
/// - t1-t4: Time points (milliseconds)
pub struct FadeTagHandler;

impl TagHandler for FadeTagHandler {
    fn name(&self) -> &'static str {
        "fade"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Fade tag requires (a1,a2,a3,t1,t2,t3,t4) - 7 numeric parameters",
            ))
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

        // Must have exactly 7 parameters
        if parts.len() != 7 {
            return false;
        }

        // First 3 are alpha values (0-255)
        for part in parts.iter().take(3) {
            match part.parse::<u32>() {
                Ok(alpha) => {
                    if alpha > 255 {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }

        // Last 4 are time values (non-negative)
        for part in parts.iter().take(7).skip(3) {
            match part.parse::<i32>() {
                Ok(time) => {
                    if time < 0 {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }

        true
    }
}

/// Handler for simple fade in/out tag (`\fad`)
///
/// Simple fade effect with 2 parameters:
/// `\fad(t1,t2)`
/// - t1: Fade in duration (milliseconds)
/// - t2: Fade out duration (milliseconds)
pub struct SimpleFadeTagHandler;

impl TagHandler for SimpleFadeTagHandler {
    fn name(&self) -> &'static str {
        "fad"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Simple fade tag requires (t1,t2) - fade in and out durations",
            ))
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

        // Must have exactly 2 parameters
        if parts.len() != 2 {
            return false;
        }

        // Both must be non-negative integers
        for part in parts {
            match part.parse::<u32>() {
                Ok(_) => {}
                Err(_) => return false,
            }
        }

        true
    }
}

/// Create all animation tag handlers
///
/// Returns a vector of boxed tag handlers for animation operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::animation::create_animation_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_animation_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_animation_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(TransformTagHandler),
        alloc::boxed::Box::new(FadeTagHandler),
        alloc::boxed::Box::new(SimpleFadeTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_handler_valid() {
        let handler = TransformTagHandler;
        // Simple transform with modifiers
        assert_eq!(handler.process("(\\fs20)"), TagResult::Processed);
        assert_eq!(handler.process("(\\c&H0000FF&)"), TagResult::Processed);
        // With timing
        assert_eq!(handler.process("(100,500,\\fs30)"), TagResult::Processed);
        // With timing and acceleration
        assert_eq!(
            handler.process("(0,1000,2.5,\\frz360)"),
            TagResult::Processed
        );
        // Multiple modifiers
        assert_eq!(
            handler.process("(\\fs20\\c&HFF0000&)"),
            TagResult::Processed
        );
    }

    #[test]
    fn transform_handler_invalid() {
        let handler = TransformTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("\\fs20"), TagResult::Failed(_))); // No parentheses
        assert!(matches!(handler.process("()"), TagResult::Failed(_))); // Empty
        assert!(matches!(handler.process("(  )"), TagResult::Failed(_))); // Only whitespace

        if let TagResult::Failed(msg) = handler.process("no_parens") {
            assert_eq!(
                msg,
                "Transform tag requires (modifiers) or (t1,t2,[accel,]modifiers)"
            );
        }
    }

    #[test]
    fn fade_handler_valid() {
        let handler = FadeTagHandler;
        // Standard fade
        assert_eq!(
            handler.process("(0,255,0,0,500,1000,1500)"),
            TagResult::Processed
        );
        // All zeros
        assert_eq!(handler.process("(0,0,0,0,0,0,0)"), TagResult::Processed);
        // Max alpha values
        assert_eq!(
            handler.process("(255,255,255,100,200,300,400)"),
            TagResult::Processed
        );
    }

    #[test]
    fn fade_handler_invalid() {
        let handler = FadeTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(
            handler.process("(0,0,0,0,0,0)"),
            TagResult::Failed(_)
        )); // Too few
        assert!(matches!(
            handler.process("(0,0,0,0,0,0,0,0)"),
            TagResult::Failed(_)
        )); // Too many
        assert!(matches!(
            handler.process("(256,0,0,0,0,0,0)"),
            TagResult::Failed(_)
        )); // Alpha > 255
        assert!(matches!(
            handler.process("(0,0,0,-1,0,0,0)"),
            TagResult::Failed(_)
        )); // Negative time
        assert!(matches!(
            handler.process("(a,b,c,d,e,f,g)"),
            TagResult::Failed(_)
        )); // Non-numeric

        if let TagResult::Failed(msg) = handler.process("(1,2,3)") {
            assert_eq!(
                msg,
                "Fade tag requires (a1,a2,a3,t1,t2,t3,t4) - 7 numeric parameters"
            );
        }
    }

    #[test]
    fn simple_fade_handler_valid() {
        let handler = SimpleFadeTagHandler;
        assert_eq!(handler.process("(500,500)"), TagResult::Processed);
        assert_eq!(handler.process("(0,1000)"), TagResult::Processed);
        assert_eq!(handler.process("(1000,0)"), TagResult::Processed);
        assert_eq!(handler.process("( 100 , 200 )"), TagResult::Processed); // With spaces
    }

    #[test]
    fn simple_fade_handler_invalid() {
        let handler = SimpleFadeTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("(500)"), TagResult::Failed(_))); // Too few
        assert!(matches!(
            handler.process("(500,500,500)"),
            TagResult::Failed(_)
        )); // Too many
        assert!(matches!(
            handler.process("(-500,500)"),
            TagResult::Failed(_)
        )); // Negative
        assert!(matches!(handler.process("(abc,def)"), TagResult::Failed(_))); // Non-numeric

        if let TagResult::Failed(msg) = handler.process("500,500") {
            assert_eq!(
                msg,
                "Simple fade tag requires (t1,t2) - fade in and out durations"
            );
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(TransformTagHandler.name(), "t");
        assert_eq!(FadeTagHandler.name(), "fade");
        assert_eq!(SimpleFadeTagHandler.name(), "fad");
    }

    #[test]
    fn create_animation_handlers_returns_all() {
        let handlers = create_animation_handlers();
        assert_eq!(handlers.len(), 3);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
        assert!(names.contains(&"t"));
        assert!(names.contains(&"fade"));
        assert!(names.contains(&"fad"));
    }

    #[test]
    fn transform_validation() {
        let handler = TransformTagHandler;
        assert!(handler.validate("(\\fs20)"));
        assert!(handler.validate("(100,500,\\fs30)"));
        assert!(handler.validate("(0,1000,2.5,\\frz360)"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("no_parens"));
        assert!(!handler.validate("()"));
    }

    #[test]
    fn fade_validation() {
        let handler = FadeTagHandler;
        assert!(handler.validate("(0,255,0,0,500,1000,1500)"));
        assert!(handler.validate("(255,255,255,0,0,0,0)"));
        assert!(!handler.validate("(0,0,0,0,0,0)")); // Too few
        assert!(!handler.validate("(256,0,0,0,0,0,0)")); // Alpha > 255
        assert!(!handler.validate("(0,0,0,-1,0,0,0)")); // Negative time
    }

    #[test]
    fn simple_fade_validation() {
        let handler = SimpleFadeTagHandler;
        assert!(handler.validate("(500,500)"));
        assert!(handler.validate("(0,0)"));
        assert!(handler.validate("(9999,9999)"));
        assert!(!handler.validate("(500)")); // Too few
        assert!(!handler.validate("(500,500,500)")); // Too many
        assert!(!handler.validate("(-500,500)")); // Negative not allowed
    }

    #[test]
    fn fade_edge_cases() {
        let handler = FadeTagHandler;

        // Boundary alpha values
        assert_eq!(handler.process("(0,0,0,0,0,0,0)"), TagResult::Processed);
        assert_eq!(
            handler.process("(255,255,255,0,0,0,0)"),
            TagResult::Processed
        );

        // Large time values
        assert_eq!(
            handler.process("(128,128,128,0,99999,199999,299999)"),
            TagResult::Processed
        );

        // Whitespace handling
        assert_eq!(
            handler.process("( 100 , 200 , 100 , 0 , 500 , 1000 , 1500 )"),
            TagResult::Processed
        );
    }

    #[test]
    fn transform_complex_cases() {
        let handler = TransformTagHandler;

        // Nested parentheses (in real implementation would need proper parsing)
        assert_eq!(
            handler.process("(\\clip(0,0,100,100))"),
            TagResult::Processed
        );

        // Multiple transformations
        assert_eq!(
            handler.process("(\\fs20\\frz30\\c&HFF0000&)"),
            TagResult::Processed
        );

        // With complex timing
        assert_eq!(
            handler.process("(100,2000,0.5,\\fscx120\\fscy120)"),
            TagResult::Processed
        );
    }
}
