//! Absolute positioning override-tag handler.
//!
//! Implements [`TagHandler`] for the `\pos` command, which positions text at
//! absolute `(x,y)` coordinates. Arguments must be two comma-separated numbers.

use super::validation::is_numeric;
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
