//! Reset, short-form rotation, and rotation-origin override-tag handlers.
//!
//! Implements [`TagHandler`] for the `\r`, `\fr`, and `\org` commands,
//! covering style reset, Z-axis rotation, and rotation origin points.

use super::validation::is_numeric;
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
