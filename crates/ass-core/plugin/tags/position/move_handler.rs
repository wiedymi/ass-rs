//! Animated movement override-tag handler.
//!
//! Implements [`TagHandler`] for the `\move` command, which animates text from
//! `(x1,y1)` to `(x2,y2)` optionally between times `t1` and `t2`.

use super::validation::is_numeric;
use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

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
