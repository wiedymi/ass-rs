//! Soft/hard line break and hard space override-tag handlers.
//!
//! Implements [`TagHandler`] for the `\n`, `\N`, and `\h` special-character
//! commands. These tags take no arguments but affect text layout and
//! rendering.

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for soft line break tag (`\n`)
///
/// Creates a line break except when smart wrapping is enabled.
pub struct SoftLineBreakTagHandler;

impl TagHandler for SoftLineBreakTagHandler {
    fn name(&self) -> &'static str {
        "n"
    }

    fn process(&self, args: &str) -> TagResult {
        if args.trim().is_empty() {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Line break tag takes no arguments"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().is_empty()
    }
}

/// Handler for hard line break tag (`\N`)
///
/// Always creates a line break regardless of wrapping mode.
pub struct HardLineBreakTagHandler;

impl TagHandler for HardLineBreakTagHandler {
    fn name(&self) -> &'static str {
        "N"
    }

    fn process(&self, args: &str) -> TagResult {
        if args.trim().is_empty() {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Line break tag takes no arguments"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().is_empty()
    }
}

/// Handler for hard space tag (`\h`)
///
/// Inserts a non-breaking space character.
pub struct HardSpaceTagHandler;

impl TagHandler for HardSpaceTagHandler {
    fn name(&self) -> &'static str {
        "h"
    }

    fn process(&self, args: &str) -> TagResult {
        if args.trim().is_empty() {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Hard space tag takes no arguments"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().is_empty()
    }
}
