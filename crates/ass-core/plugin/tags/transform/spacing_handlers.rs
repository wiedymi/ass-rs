//! Letter-spacing override-tag handler.
//!
//! Implements [`TagHandler`] for the `\fsp` spacing command, which accepts
//! a numeric pixel value (integer or decimal).

use super::validation::validate_numeric_arg;
use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for letter spacing tag (`\fsp`)
pub struct SpacingTagHandler;

impl TagHandler for SpacingTagHandler {
    fn name(&self) -> &'static str {
        "fsp"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Spacing tag requires numeric pixels"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}
