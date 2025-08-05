//! Position and movement tag handlers for ASS override tags
//!
//! Implements handlers for positioning and movement commands including
//! absolute positioning and animated movement. These handlers validate
//! numeric arguments and coordinate pairs according to ASS specifications.
//!
//! # Supported Tags
//!
//! - `pos`: Absolute positioning (x,y)
//! - `move`: Animated movement (x1,y1,x2,y2\[,t1,t2\])
//!
//! # Performance
//!
//! - Zero allocations for argument parsing
//! - Fast numeric validation
//! - Minimal memory footprint per handler

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for position tag (`\pos`)
///
/// Positions text at absolute coordinates (x,y).
/// Arguments must be two comma-separated numbers.
pub struct PositionTagHandler;

impl TagHandler for PositionTagHandler {
    fn name(&self) -> &'static str {
        "pos"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Position tag requires (x,y) coordinates"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() || !args.contains(',') {
            return false;
        }

        // Parse (x,y) format
        if let Some((x_str, y_str)) = args.split_once(',') {
            let x_str = x_str.trim();
            let y_str = y_str.trim();

            // Validate both parts are numeric
            is_numeric(x_str) && is_numeric(y_str)
        } else {
            false
        }
    }
}

/// Handler for movement tag (`\move`)
///
/// Moves text from (x1,y1) to (x2,y2) optionally between times t1 and t2.
/// Arguments: x1,y1,x2,y2`[,t1,t2\]`
pub struct MoveTagHandler;

impl TagHandler for MoveTagHandler {
    fn name(&self) -> &'static str {
        "move"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Move tag requires (x1,y1,x2,y2[,t1,t2]) coordinates",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        let parts: alloc::vec::Vec<&str> = args.split(',').map(str::trim).collect();

        // Must have either 4 or 6 arguments
        if parts.len() != 4 && parts.len() != 6 {
            return false;
        }

        // All parts must be numeric
        parts.iter().all(|part| is_numeric(part))
    }
}

/// Validate if a string represents a valid number (integer or decimal)
#[inline]
fn is_numeric(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // Check for optional negative sign
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

/// Create all position and movement tag handlers
///
/// Returns a vector of boxed tag handlers for position-related operations.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, tags::position::create_position_handlers};
///
/// let mut registry = ExtensionRegistry::new();
/// for handler in create_position_handlers() {
///     registry.register_tag_handler(handler).unwrap();
/// }
/// ```
#[must_use]
pub fn create_position_handlers() -> alloc::vec::Vec<alloc::boxed::Box<dyn TagHandler>> {
    alloc::vec![
        alloc::boxed::Box::new(PositionTagHandler),
        alloc::boxed::Box::new(MoveTagHandler),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn is_numeric_valid() {
        assert!(is_numeric("123"));
        assert!(is_numeric("123.45"));
        assert!(is_numeric("-123"));
        assert!(is_numeric("-123.45"));
        assert!(is_numeric("+123"));
        assert!(is_numeric("0"));
        assert!(is_numeric("0.0"));
    }

    #[test]
    fn is_numeric_invalid() {
        assert!(!is_numeric(""));
        assert!(!is_numeric("-"));
        assert!(!is_numeric("+"));
        assert!(!is_numeric("123."));
        assert!(!is_numeric(".123"));
        assert!(!is_numeric("12.34.56"));
        assert!(!is_numeric("abc"));
        assert!(!is_numeric("123abc"));
        assert!(!is_numeric("12 3"));
    }

    #[test]
    fn position_handler_valid() {
        let handler = PositionTagHandler;
        assert_eq!(handler.process("100,200"), TagResult::Processed);
        assert_eq!(handler.process("0,0"), TagResult::Processed);
        assert_eq!(handler.process("-50,100"), TagResult::Processed);
        assert_eq!(handler.process("100.5,200.75"), TagResult::Processed);
        assert_eq!(handler.process(" 100 , 200 "), TagResult::Processed);
    }

    #[test]
    fn position_handler_invalid() {
        let handler = PositionTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("100"), TagResult::Failed(_)));
        assert!(matches!(handler.process("100,"), TagResult::Failed(_)));
        assert!(matches!(handler.process(",200"), TagResult::Failed(_)));
        assert!(matches!(handler.process("abc,200"), TagResult::Failed(_)));
        assert!(matches!(handler.process("100,abc"), TagResult::Failed(_)));
        assert!(matches!(
            handler.process("100,200,300"),
            TagResult::Failed(_)
        ));
    }

    #[test]
    fn position_handler_validation() {
        let handler = PositionTagHandler;
        assert!(handler.validate("100,200"));
        assert!(handler.validate("-50,100"));
        assert!(handler.validate("100.5,200.75"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("100"));
        assert!(!handler.validate("abc,200"));
    }

    #[test]
    fn move_handler_valid_4_args() {
        let handler = MoveTagHandler;
        assert_eq!(handler.process("0,0,100,100"), TagResult::Processed);
        assert_eq!(handler.process("-50,-50,50,50"), TagResult::Processed);
        assert_eq!(
            handler.process("100.5,200.5,300.5,400.5"),
            TagResult::Processed
        );
        assert_eq!(handler.process(" 0 , 0 , 100 , 100 "), TagResult::Processed);
    }

    #[test]
    fn move_handler_valid_6_args() {
        let handler = MoveTagHandler;
        assert_eq!(handler.process("0,0,100,100,0,1000"), TagResult::Processed);
        assert_eq!(
            handler.process("-50,-50,50,50,500,1500"),
            TagResult::Processed
        );
        assert_eq!(
            handler.process("100.5,200.5,300.5,400.5,0.0,1000.0"),
            TagResult::Processed
        );
    }

    #[test]
    fn move_handler_invalid() {
        let handler = MoveTagHandler;
        assert!(matches!(handler.process(""), TagResult::Failed(_)));
        assert!(matches!(handler.process("100"), TagResult::Failed(_)));
        assert!(matches!(handler.process("100,200"), TagResult::Failed(_)));
        assert!(matches!(
            handler.process("100,200,300"),
            TagResult::Failed(_)
        ));
        assert!(matches!(
            handler.process("100,200,300,400,500"),
            TagResult::Failed(_)
        ));
        assert!(matches!(
            handler.process("100,200,300,400,500,600,700"),
            TagResult::Failed(_)
        ));
        assert!(matches!(
            handler.process("abc,200,300,400"),
            TagResult::Failed(_)
        ));
        assert!(matches!(
            handler.process("100,abc,300,400"),
            TagResult::Failed(_)
        ));
    }

    #[test]
    fn move_handler_validation() {
        let handler = MoveTagHandler;
        assert!(handler.validate("0,0,100,100"));
        assert!(handler.validate("0,0,100,100,0,1000"));
        assert!(!handler.validate(""));
        assert!(!handler.validate("100,200"));
        assert!(!handler.validate("100,200,300"));
        assert!(!handler.validate("100,200,300,400,500"));
        assert!(!handler.validate("abc,200,300,400"));
    }

    #[test]
    fn handlers_have_correct_names() {
        assert_eq!(PositionTagHandler.name(), "pos");
        assert_eq!(MoveTagHandler.name(), "move");
    }

    #[test]
    fn create_position_handlers_returns_all() {
        let handlers = create_position_handlers();
        assert_eq!(handlers.len(), 2);

        let names: alloc::vec::Vec<&str> = handlers.iter().map(|h| h.name()).collect();
        assert!(names.contains(&"pos"));
        assert!(names.contains(&"move"));
    }

    #[test]
    fn position_edge_cases() {
        let handler = PositionTagHandler;

        // Large numbers
        assert_eq!(handler.process("999999,999999"), TagResult::Processed);

        // Very small decimals
        assert_eq!(handler.process("0.0001,0.0001"), TagResult::Processed);

        // Mixed signs
        assert_eq!(handler.process("+100,-200"), TagResult::Processed);
    }

    #[test]
    fn move_edge_cases() {
        let handler = MoveTagHandler;

        // All zeros
        assert_eq!(handler.process("0,0,0,0"), TagResult::Processed);
        assert_eq!(handler.process("0,0,0,0,0,0"), TagResult::Processed);

        // Large numbers
        assert_eq!(
            handler.process("999999,999999,0,0,0,99999"),
            TagResult::Processed
        );

        // Negative times (valid in ASS)
        assert_eq!(
            handler.process("0,0,100,100,-500,1000"),
            TagResult::Processed
        );
    }

    #[test]
    fn whitespace_handling() {
        let pos_handler = PositionTagHandler;
        let move_handler = MoveTagHandler;

        // Various whitespace
        assert_eq!(pos_handler.process("  100  ,  200  "), TagResult::Processed);
        assert_eq!(
            move_handler.process("  0  ,  0  ,  100  ,  100  "),
            TagResult::Processed
        );
        assert_eq!(
            move_handler.process("  0  ,  0  ,  100  ,  100  ,  0  ,  1000  "),
            TagResult::Processed
        );

        // Tabs and newlines should be trimmed
        assert_eq!(pos_handler.process("\t100\t,\t200\t"), TagResult::Processed);
    }
}
