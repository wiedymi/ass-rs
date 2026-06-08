//! Alpha (transparency) override-tag handlers.
//!
//! Implements [`TagHandler`] for the `\alpha`, `\1a`, `\2a`, `\3a`, and `\4a`
//! transparency commands. Alpha values use the `&Haa&` hex format per the ASS
//! specification.

use super::validation::validate_alpha_args;
use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for all alpha channels tag (`\alpha`)
pub struct AlphaTagHandler;

impl TagHandler for AlphaTagHandler {
    fn name(&self) -> &'static str {
        "alpha"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Handler for primary alpha tag (`\1a`)
pub struct Alpha1TagHandler;

impl TagHandler for Alpha1TagHandler {
    fn name(&self) -> &'static str {
        "1a"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Handler for secondary alpha tag (`\2a`)
pub struct Alpha2TagHandler;

impl TagHandler for Alpha2TagHandler {
    fn name(&self) -> &'static str {
        "2a"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Handler for outline alpha tag (`\3a`)
pub struct Alpha3TagHandler;

impl TagHandler for Alpha3TagHandler {
    fn name(&self) -> &'static str {
        "3a"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}

/// Handler for shadow alpha tag (`\4a`)
pub struct Alpha4TagHandler;

impl TagHandler for Alpha4TagHandler {
    fn name(&self) -> &'static str {
        "4a"
    }

    fn process(&self, args: &str) -> TagResult {
        if validate_alpha_args(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Alpha tag requires &Haa& format"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        validate_alpha_args(args)
    }
}
