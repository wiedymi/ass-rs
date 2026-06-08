//! Shear override-tag handlers for the X and Y axes.
//!
//! Implements [`TagHandler`] for the `\fax` and `\fay` shear commands.
//! Each accepts a numeric shear factor (integer or decimal).

use super::validation::validate_numeric_arg;
use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for X-axis shear tag (`\fax`)
pub struct ShearXTagHandler;

impl TagHandler for ShearXTagHandler {
    fn name(&self) -> &'static str {
        "fax"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Shear tag requires numeric factor"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for Y-axis shear tag (`\fay`)
pub struct ShearYTagHandler;

impl TagHandler for ShearYTagHandler {
    fn name(&self) -> &'static str {
        "fay"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Shear tag requires numeric factor"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}
