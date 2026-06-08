//! Font name/size/encoding override-tag handlers.
//!
//! Implements [`TagHandler`] for the `\fn`, `\fs`, and `\fe` commands,
//! validating font name, size, and character-set arguments per the ASS
//! specification.

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for font name tag (`\fn`)
///
/// Sets the font face by name. Accepts any non-empty string.
pub struct FontNameTagHandler;

impl TagHandler for FontNameTagHandler {
    fn name(&self) -> &'static str {
        "fn"
    }

    fn process(&self, args: &str) -> TagResult {
        let args = args.trim();
        if args.is_empty() {
            TagResult::Failed(String::from("Font name tag requires a font name"))
        } else {
            TagResult::Processed
        }
    }

    fn validate(&self, args: &str) -> bool {
        !args.trim().is_empty()
    }
}

/// Handler for font size tag (`\fs`)
///
/// Sets the font size. Must be a positive number.
pub struct FontSizeTagHandler;

impl TagHandler for FontSizeTagHandler {
    fn name(&self) -> &'static str {
        "fs"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Font size tag requires positive numeric size"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Parse as positive number (integer or decimal)
        args.parse::<f32>().is_ok_and(|size| size > 0.0)
    }
}

/// Handler for font encoding tag (`\fe`)
///
/// Sets the font character set. Must be a valid encoding number.
pub struct FontEncodingTagHandler;

impl TagHandler for FontEncodingTagHandler {
    fn name(&self) -> &'static str {
        "fe"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Font encoding tag requires numeric charset"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Must be a non-negative integer
        args.parse::<u32>().is_ok()
    }
}
