//! Rotation override-tag handlers for the X, Y, and Z axes.
//!
//! Implements [`TagHandler`] for the `\frx`, `\fry`, and `\frz` rotation
//! commands. Each accepts a numeric degree value (integer or decimal).

use super::validation::validate_numeric_arg;
use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for Z-axis rotation tag (`\frz`)
pub struct RotationZTagHandler;

impl TagHandler for RotationZTagHandler {
    fn name(&self) -> &'static str {
        "frz"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Rotation tag requires numeric degrees"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for X-axis rotation tag (`\frx`)
pub struct RotationXTagHandler;

impl TagHandler for RotationXTagHandler {
    fn name(&self) -> &'static str {
        "frx"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Rotation tag requires numeric degrees"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}

/// Handler for Y-axis rotation tag (`\fry`)
pub struct RotationYTagHandler;

impl TagHandler for RotationYTagHandler {
    fn name(&self) -> &'static str {
        "fry"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_numeric_arg(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Rotation tag requires numeric degrees"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_numeric_arg(args)
    }
}
