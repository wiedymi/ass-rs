//! Unicode-aware soft-wrap opportunities (UAX #14).
//!
//! Feature-gated behind `unicode-wrap`. Provides the analysis-side equivalent
//! of libass 0.17.4's `ASS_FEATURE_WRAP_UNICODE`: it identifies valid line
//! break positions in plain text, including breaks between CJK/Kana/Hangul
//! characters that are not separated by spaces.
//!
//! The input is expected to be *plain* text — that is, with override tag blocks
//! removed and explicit `\N`/`\n` breaks already resolved (see
//! [`TextWithLineBreaks`](super::line_breaks::TextWithLineBreaks)). Each
//! returned offset is the byte index of the character that would start the new
//! line if a wrap is taken there.
//!
//! # Examples
//!
//! ```rust
//! use ass_core::analysis::events::unicode_wrap::soft_wrap_offsets;
//!
//! // A space provides a soft-wrap opportunity before the next word.
//! let offsets = soft_wrap_offsets("Hello world");
//! assert!(offsets.contains(&6));
//! ```

use alloc::vec::Vec;

/// A position in plain text where a line wrap may occur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrapOpportunity {
    /// Byte offset of the character that starts the new line if wrapped here.
    pub offset: usize,
    /// Whether the break is mandatory (e.g. a hard line feed or end of text).
    pub mandatory: bool,
}

/// Compute all Unicode line-break opportunities for `text` per UAX #14.
///
/// Includes the mandatory break at the end of the text. Use
/// [`soft_wrap_offsets`] for only the optional intra-text break points.
#[must_use]
pub fn wrap_opportunities(text: &str) -> Vec<WrapOpportunity> {
    unicode_linebreak::linebreaks(text)
        .map(|(offset, opportunity)| WrapOpportunity {
            offset,
            mandatory: matches!(opportunity, unicode_linebreak::BreakOpportunity::Mandatory),
        })
        .collect()
}

/// Return only the byte offsets at which an optional (soft) wrap may occur.
///
/// Mandatory breaks — including the implicit one at the end of the string — are
/// excluded, leaving the positions a renderer may choose to wrap at when a line
/// exceeds the available width.
#[must_use]
pub fn soft_wrap_offsets(text: &str) -> Vec<usize> {
    wrap_opportunities(text)
        .into_iter()
        .filter(|opportunity| !opportunity.mandatory)
        .map(|opportunity| opportunity.offset)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaks_after_space() {
        let offsets = soft_wrap_offsets("Hello world");
        // The break is permitted before "world" (offset of 'w').
        assert_eq!(offsets, alloc::vec![6]);
    }

    #[test]
    fn multiple_words() {
        let offsets = soft_wrap_offsets("one two three");
        assert_eq!(offsets, alloc::vec![4, 8]);
    }

    #[test]
    fn breaks_between_cjk_without_spaces() {
        // Japanese text has no spaces but permits inter-character wrapping.
        let text = "日本語字幕";
        let offsets = soft_wrap_offsets(text);
        // At least one break opportunity exists despite the absence of spaces.
        assert!(!offsets.is_empty());
        // Every offset must fall on a UTF-8 character boundary.
        assert!(offsets.iter().all(|&o| text.is_char_boundary(o)));
    }

    #[test]
    fn no_soft_break_in_single_token() {
        // A single unbroken word offers no interior soft-wrap point.
        assert!(soft_wrap_offsets("indivisible").is_empty());
    }

    #[test]
    fn final_break_is_mandatory() {
        let all = wrap_opportunities("hi there");
        let last = all.last().expect("at least one opportunity");
        assert_eq!(last.offset, "hi there".len());
        assert!(last.mandatory);
    }
}
