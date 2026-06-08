//! Karaoke timing override-tag handler implementations.
//!
//! Implements [`TagHandler`] for the `\k`, `\kf`, `\ko`, and v4++ `\kt`
//! karaoke timing commands. Each handler validates that its argument is an
//! integer duration in hundredths of seconds (centiseconds).

use crate::plugin::{TagHandler, TagResult};
use alloc::string::String;

/// Handler for basic karaoke highlight tag (`\k`)
///
/// Highlights text for the specified duration in hundredths of seconds.
pub struct BasicKaraokeTagHandler;

impl TagHandler for BasicKaraokeTagHandler {
    fn name(&self) -> &'static str {
        "k"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Karaoke tag `k` expects duration in hundredths of seconds",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().parse::<u32>().is_ok()
    }
}

/// Handler for fill karaoke highlight tag (`\kf`)
///
/// Fills text progressively for the specified duration in hundredths of seconds.
pub struct FillKaraokeTagHandler;

impl TagHandler for FillKaraokeTagHandler {
    fn name(&self) -> &'static str {
        "kf"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Fill karaoke tag `kf` expects duration in hundredths of seconds",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().parse::<u32>().is_ok()
    }
}

/// Handler for outline karaoke highlight tag (`\ko`)
///
/// Highlights text outline for the specified duration in hundredths of seconds.
pub struct OutlineKaraokeTagHandler;

impl TagHandler for OutlineKaraokeTagHandler {
    fn name(&self) -> &'static str {
        "ko"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Outline karaoke tag `ko` expects duration in hundredths of seconds",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().parse::<u32>().is_ok()
    }
}

/// Handler for absolute karaoke timing tag (`\kt`)
///
/// This tag was introduced in the v4++ spec for absolute karaoke timing.
/// It expects an integer duration in centiseconds.
///
/// # Examples
///
/// ```rust
/// use ass_core::plugin::tags::karaoke::KaraokeTimingTagHandler;
/// use ass_core::plugin::{TagHandler, TagResult};
///
/// let handler = KaraokeTimingTagHandler;
/// assert_eq!(handler.process("500"), TagResult::Processed);
/// assert!(matches!(handler.process("invalid"), TagResult::Failed(_)));
/// ```
pub struct KaraokeTimingTagHandler;

impl TagHandler for KaraokeTimingTagHandler {
    fn name(&self) -> &'static str {
        "kt"
    }

    fn process(&self, args: &str) -> TagResult {
        if self.validate(args) {
            TagResult::Processed
        } else {
            TagResult::Failed(String::from(
                "Karaoke timing tag `kt` expects an integer duration in centiseconds",
            ))
        }
    }

    fn validate(&self, args: &str) -> bool {
        args.trim().parse::<u32>().is_ok()
    }
}
