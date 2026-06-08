//! Legacy, numpad, and wrapping-style alignment override-tag handlers.
//!
//! Implements [`TagHandler`] for the `\a`, `\an`, and `\q` commands. Each
//! handler validates its argument against the ASS specification with zero
//! allocations and fast integer comparisons.

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for legacy alignment tag (`\a`)
///
/// Uses legacy alignment codes:
/// - 1 = left, 2 = center, 3 = right
/// - +4 = top, +0 = bottom, +8 = middle (vertical)
pub struct AlignmentTagHandler;

impl TagHandler for AlignmentTagHandler {
    fn name(&self) -> &'static str {
        "a"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Alignment tag requires valid alignment code (1-11)",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        let args = args.trim();

        // Valid values: 1-3 (bottom), 5-7 (top), 9-11 (middle)
        matches!(args, "1" | "2" | "3" | "5" | "6" | "7" | "9" | "10" | "11")
    }
}

/// Handler for numpad alignment tag (`\an`)
///
/// Uses numpad-style alignment (1-9):
/// ```text
/// 7 8 9
/// 4 5 6
/// 1 2 3
/// ```
pub struct NumpadAlignmentTagHandler;

impl TagHandler for NumpadAlignmentTagHandler {
    fn name(&self) -> &'static str {
        "an"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Numpad alignment tag requires value 1-9"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        matches!(
            args.trim(),
            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
        )
    }
}

/// Handler for wrapping style tag (`\q`)
///
/// Controls text wrapping behavior:
/// - 0 = Smart wrapping
/// - 1 = End-of-line wrapping (\N only)
/// - 2 = No wrapping (\n, \N break)
/// - 3 = Smart wrapping, wider lower line
pub struct WrappingStyleTagHandler;

impl TagHandler for WrappingStyleTagHandler {
    fn name(&self) -> &'static str {
        "q"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from("Wrapping style tag requires value 0-3"))
        }
    }

    fn validate(&self, args: &str) -> bool {
        matches!(args.trim(), "0" | "1" | "2" | "3")
    }
}
