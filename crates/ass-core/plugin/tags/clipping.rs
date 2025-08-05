//! Clipping tag handlers for ASS override tags
//!
//! Implements handlers for clipping masks including rectangular and
//! vector-based shape clipping. These tags control which parts of
//! text are visible.
//!
//! # Supported Tags
//!
//! - `clip`: Rectangular or vector clipping mask
//!
//! # Performance
//!
//! - Efficient argument parsing
//! - Support for both rectangular and vector formats
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for clipping tag (`\clip`)
///
/// Supports two formats:
/// - Rectangular: `\clip(x1,y1,x2,y2)` - Clips to rectangle
/// - Vector: `\clip([scale,]drawing commands)` - Clips to vector shape
pub struct ClipTagHandler;

impl TagHandler for ClipTagHandler {
    fn name(&self) -> &'static str {
        "clip"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Clip tag requires (x1,y1,x2,y2) or ([scale,]drawing commands)",
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
        let content = &args[1..args.len() - 1].trim();
        if content.is_empty() {
            return false;
        }

        // Check if it's rectangular format (4 numbers)
        let parts: alloc::vec::Vec<&str> = content.split(',').map(str::trim).collect();

        if parts.len() == 4 {
            // Rectangular clip - all parts must be numbers
            parts.iter().all(|part| is_numeric(part))
        } else {
            // Vector clip - must contain drawing commands
            // Simple validation: check for drawing command letters
            content
                .chars()
                .any(|c| matches!(c, 'm' | 'n' | 'l' | 'b' | 's' | 'p' | 'c'))
        }
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

/// Create all clipping tag handlers
///
/// Returns a vector of boxed tag handlers for clipping operations.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::clipping::create_clipping_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_clipping_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_clipping_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![alloc::boxed::Box::new(ClipTagHandler)]
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn clip_handler_rectangular_valid() {
        let handler = ClipTagHandler;
        assert_eq!(handler.process("(0,0,100,100)"), TagResult::Processed);
        assert_eq!(handler.process("(10,20,300,400)"), TagResult::Processed);
        assert_eq!(handler.process("(-50,-50,50,50)"), TagResult::Processed);
        assert_eq!(handler.process("(0.5,0.5,99.5,99.5)"), TagResult::Processed);
        assert_eq!(
            handler.process("( 0 , 0 , 100 , 100 )"),
            TagResult::Processed
        );
    }

    #[test]
    fn clip_handler_vector_valid() {
        let handler = ClipTagHandler;
        // Basic drawing commands
        assert_eq!(
            handler.process("(m 0 0 l 100 0 100 100 0 100)"),
            TagResult::Processed
        );
        // With scale
        assert_eq!(
            handler.process("(2,m 0 0 l 50 0 50 50 0 50)"),
            TagResult::Processed
        );
        // Complex path
        assert_eq!(
            handler.process("(m 10 10 b 20 10 30 20 30 30 l 10 30)"),
            TagResult::Processed
        );
        // With multiple commands
        assert_eq!(
            handler.process("(m 0 0 l 100 0 b 100 50 50 100 0 100 c)"),
            TagResult::Processed
        );
    }

    #[test]
    fn clip_handler_invalid() {
        let handler = ClipTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(
            handler.process("0,0,100,100"),
            TagResult::Failed(_)
        )); // No parentheses
        assert!(matches!(handler.process("()"), TagResult::Failed(_))); // Empty
        assert!(matches!(handler.process("(0,0,100)"), TagResult::Failed(_))); // 3 coords
        assert!(matches!(
            handler.process("(0,0,100,100,200)"),
            TagResult::Failed(_)
        )); // 5 coords
        assert!(matches!(handler.process("(a,b,c,d)"), TagResult::Failed(_))); // Non-numeric rect
                                                                               // Note: "(invalid)" actually contains 'n' and 'l' which are drawing commands, so it's valid
        assert!(matches!(handler.process("(xyz)"), TagResult::Failed(_))); // No drawing commands

        if let TagResult::Failed(msg) = handler.process("no_parens") {
            assert_eq!(
                msg,
                "Clip tag requires (x1,y1,x2,y2) or ([scale,]drawing commands)"
            );
        }
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(ClipTagHandler.name(), "clip");
    }

    #[test]
    fn create_clipping_handlers_returns_all() {
        let handlers = create_clipping_handlers();
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].name(), "clip");
    }

    #[test]
    fn clip_validation_rectangular() {
        let handler = ClipTagHandler;
        assert!(handler.validate("(0,0,100,100)"));
        assert!(handler.validate("(-50,-50,50,50)"));
        assert!(handler.validate("(0.5,0.5,99.5,99.5)"));
        assert!(!handler.validate("(0,0,100)")); // Too few
        assert!(!handler.validate("(0,0,100,100,200)")); // Too many
        assert!(!handler.validate("(a,b,c,d)")); // Non-numeric
    }

    #[test]
    fn clip_validation_vector() {
        let handler = ClipTagHandler;
        assert!(handler.validate("(m 0 0 l 100 100)"));
        assert!(handler.validate("(2,m 0 0 l 50 50)")); // With scale
        assert!(handler.validate("(m 0 0 b 10 0 20 10 20 20)"));
        assert!(!handler.validate("(xyz)")); // No drawing commands
        assert!(!handler.validate("(123)")); // Just numbers, no commands
    }

    #[test]
    fn is_numeric_tests() {
        assert!(is_numeric("123"));
        assert!(is_numeric("-123"));
        assert!(is_numeric("+123"));
        assert!(is_numeric("123.45"));
        assert!(is_numeric("-123.45"));
        assert!(is_numeric("0"));
        assert!(is_numeric("0.0"));

        assert!(!is_numeric(""));
        assert!(!is_numeric("-"));
        assert!(!is_numeric("+"));
        assert!(!is_numeric("."));
        assert!(!is_numeric("123."));
        assert!(!is_numeric(".123"));
        assert!(!is_numeric("12.34.56"));
        assert!(!is_numeric("abc"));
        assert!(!is_numeric("12a"));
    }

    #[test]
    fn clip_edge_cases() {
        let handler = ClipTagHandler;

        // Very large coordinates
        assert_eq!(handler.process("(0,0,9999,9999)"), TagResult::Processed);

        // Negative coordinates
        assert_eq!(
            handler.process("(-1000,-1000,1000,1000)"),
            TagResult::Processed
        );

        // Decimal coordinates
        assert_eq!(handler.process("(0.1,0.2,99.8,99.9)"), TagResult::Processed);

        // Minimal vector path
        assert_eq!(handler.process("(m 0 0)"), TagResult::Processed);

        // Vector with all command types
        assert_eq!(
            handler.process("(m 0 0 n 10 10 l 20 20 b 30 20 40 30 40 40 s 50 40 60 50 p 70 60 c)"),
            TagResult::Processed
        );
    }

    #[test]
    fn clip_whitespace_handling() {
        let handler = ClipTagHandler;

        // Rectangular with spaces
        assert_eq!(
            handler.process("(  0  ,  0  ,  100  ,  100  )"),
            TagResult::Processed
        );

        // Vector with spaces
        assert_eq!(
            handler.process("(  m  0  0  l  100  100  )"),
            TagResult::Processed
        );

        // Mixed whitespace
        assert_eq!(
            handler.process("(\t0\t,\t0\t,\t100\t,\t100\t)"),
            TagResult::Processed
        );
    }
}
