//! Border, shadow, and edge-blur override-tag handlers.
//!
//! Implements [`TagHandler`] for the `\bord`, `\shad`, and `\be` advanced
//! formatting commands. Each handler validates its numeric or boolean
//! argument according to the ASS specification.

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for border width tag (`\bord`)
///
/// Sets the border/outline width in pixels. Must be non-negative.
pub struct BorderTagHandler;

impl TagHandler for BorderTagHandler {
    fn name(&self) -> &'static str {
        "bord"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Border tag requires non-negative numeric width",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Parse as non-negative number
        args.parse::<f32>().is_ok_and(|width| width >= 0.0)
    }
}

/// Handler for shadow depth tag (`\shad`)
///
/// Sets the shadow depth in pixels. Must be non-negative.
pub struct ShadowTagHandler;

impl TagHandler for ShadowTagHandler {
    fn name(&self) -> &'static str {
        "shad"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Shadow tag requires non-negative numeric depth",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();
        if args.is_empty() {
            return false;
        }

        // Parse as non-negative number
        args.parse::<f32>().is_ok_and(|depth| depth >= 0.0)
    }
}

/// Handler for blur edges tag (`\be`)
///
/// Enables or disables edge blur effect (0 or 1).
pub struct BlurEdgesTagHandler;

impl TagHandler for BlurEdgesTagHandler {
    fn name(&self) -> &'static str {
        "be"
    }

    fn process(&self, args: &str) -> TagResult {
        match args.trim() {
            "0" | "1" => TagResult::Processed,
            _ => TagResult::Failed(String::from("Blur edges tag accepts only 0 or 1")),
        }
    }

    fn validate(&self, args: &str) -> bool {
        let trimmed = args.trim();
        trimmed == "0" || trimmed == "1"
    }
}
