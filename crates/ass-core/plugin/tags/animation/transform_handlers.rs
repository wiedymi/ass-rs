//! Transform animation override-tag handler.
//!
//! Implements [`TagHandler`] for the `\t` command, which animates style
//! modifiers over a subtitle's duration or a specified time range.

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
