//! Scale override-tag handlers for the X and Y axes.
//!
//! Implements [`TagHandler`] for the `\fscx` and `\fscy` scale commands.
//! Each accepts a numeric percent value (integer or decimal).

use super::validation::validate_numeric_arg;
use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for X-axis scale tag (`\fscx`)
pub struct ScaleXTagHandler;

impl TagHandler for ScaleXTagHandler {
    fn name(&self) -> &'static str {
        "fscx"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Scale tag requires numeric percent"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for Y-axis scale tag (`\fscy`)
pub struct ScaleYTagHandler;

impl TagHandler for ScaleYTagHandler {
    fn name(&self) -> &'static str {
        "fscy"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Scale tag requires numeric percent"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}
