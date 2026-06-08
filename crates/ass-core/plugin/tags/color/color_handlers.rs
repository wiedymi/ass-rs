//! Primary/secondary/outline/shadow color override-tag handlers.
//!
//! Implements [`TagHandler`] for the `\c`/`\1c`, `\2c`, `\3c`, and `\4c`
//! color commands. Color values use BGR format (`&Hbbggrr&`) per the ASS
//! specification.

use super::validation::validate_color_args;
use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for primary color tag (`\c` or `\1c`)
pub struct PrimaryColorTagHandler;

impl TagHandler for PrimaryColorTagHandler {
    fn name(&self) -> &'static str {
        "c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for primary color tag with explicit index (`\1c`)
pub struct Color1TagHandler;

impl TagHandler for Color1TagHandler {
    fn name(&self) -> &'static str {
        "1c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for secondary color tag (`\2c`)
pub struct Color2TagHandler;

impl TagHandler for Color2TagHandler {
    fn name(&self) -> &'static str {
        "2c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for outline color tag (`\3c`)
pub struct Color3TagHandler;

impl TagHandler for Color3TagHandler {
    fn name(&self) -> &'static str {
        "3c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}

/// Handler for shadow color tag (`\4c`)
pub struct Color4TagHandler;

impl TagHandler for Color4TagHandler {
    fn name(&self) -> &'static str {
        "4c"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_color_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Color tag requires &Hbbggrr& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_color_args(args)
    }
}
