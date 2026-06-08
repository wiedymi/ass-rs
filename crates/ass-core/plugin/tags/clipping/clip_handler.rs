//! Clipping override-tag handler and numeric-argument validation.
//!
//! Implements [`TagHandler`] for the `\clip` command, supporting both
//! rectangular (`(x1,y1,x2,y2)`) and vector (`([scale,]drawing commands)`)
//! clipping masks.

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
pub(super) fn is_numeric(s: &str) -> bool {
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
