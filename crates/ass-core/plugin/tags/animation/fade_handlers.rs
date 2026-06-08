//! Fade animation override-tag handlers.
//!
//! Implements [`TagHandler`] for the complex `\fade` and simple `\fad`
//! alpha-transparency animation commands.

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

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
