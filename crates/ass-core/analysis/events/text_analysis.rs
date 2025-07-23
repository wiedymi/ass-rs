//! Text content analysis for ASS dialogue events
//!
//! Provides comprehensive analysis of dialogue text including override tag parsing,
//! Unicode complexity detection, and character counting. Uses zero-copy design
//! with lifetime-generic references to original text.
//!
//! # Features
//!
//! - Override tag extraction and complexity scoring
//! - Plain text extraction (tags removed)
//! - Unicode bidirectional text detection
//! - Character and line counting
//! - Zero-copy tag argument references
//!
//! # Performance
//!
//! - Target: <0.5ms per event text analysis
//! - Memory: Minimal allocations via string slices
//! - Unicode: Efficient detection without full normalization

use crate::{
    analysis::events::tags::{parse_override_block, DiagnosticKind, OverrideTag, TagDiagnostic},
    Result,
};
use alloc::{string::String, vec::Vec};

/// Analysis results for dialogue text content
///
/// Contains extracted plain text, override tag information, and Unicode
/// complexity indicators. Uses zero-copy references where possible.
#[derive(Debug, Clone)]
pub struct TextAnalysis<'a> {
    /// Plain text with override tags removed
    plain_text: String,
    /// Unicode character count
    char_count: usize,
    /// Line count after processing linebreaks
    line_count: usize,
    /// Contains bidirectional text (RTL scripts)
    has_bidi_text: bool,
    /// Contains complex Unicode beyond basic Latin
    has_complex_unicode: bool,
    /// Parsed override tags
    override_tags: Vec<OverrideTag<'a>>,
    /// Parse diagnostics collected during analysis
    parse_diagnostics: Vec<TagDiagnostic<'a>>,
}

impl<'a> TextAnalysis<'a> {
    /// Analyze dialogue text content comprehensively
    ///
    /// Extracts plain text, parses override tags, and analyzes Unicode
    /// complexity. Uses zero-copy references for tag arguments.
    ///
    /// # Arguments
    ///
    /// * `text` - Original dialogue text with potential override tags
    ///
    /// # Returns
    ///
    /// Complete text analysis results or parsing error.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::analysis::events::text_analysis::TextAnalysis;
    /// let text = "Hello {\\b1}world{\\b0}!";
    /// let analysis = TextAnalysis::analyze(text)?;
    /// assert_eq!(analysis.plain_text(), "Hello world!");
    /// assert_eq!(analysis.override_tags().len(), 2);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn analyze(text: &'a str) -> Result<Self> {
        let mut override_tags = Vec::new();
        let mut parse_diagnostics = Vec::new();
        let mut plain_text = String::new();
        let mut position = 0;

        let mut chars = text.chars();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut brace_count = 1;
                let tag_start = position + ch.len_utf8();

                for inner_ch in chars.by_ref() {
                    position += inner_ch.len_utf8();

                    if inner_ch == '{' {
                        brace_count += 1;
                    } else if inner_ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                }

                if position > tag_start {
                    let tag_content = &text[tag_start..position];
                    parse_override_block(
                        tag_content,
                        tag_start,
                        &mut override_tags,
                        &mut parse_diagnostics,
                    );
                } else {
                    parse_diagnostics.push(TagDiagnostic {
                        span: &text[tag_start..position.max(tag_start + 1)],
                        offset: tag_start,
                        kind: DiagnosticKind::EmptyOverride,
                    });
                }
            } else if ch == '\\' {
                if let Some(next_ch) = chars.next() {
                    position += next_ch.len_utf8();
                    match next_ch {
                        'n' | 'N' => plain_text.push('\n'),
                        'h' => plain_text.push('\u{00A0}'),
                        _ => {
                            plain_text.push(ch);
                            plain_text.push(next_ch);
                        }
                    }
                }
            } else {
                plain_text.push(ch);
            }

            position += ch.len_utf8();
        }

        let char_count = plain_text.chars().count();
        let line_count = plain_text.lines().count().max(1);
        let has_bidi_text = Self::detect_bidi_text(&plain_text);
        let has_complex_unicode = Self::detect_complex_unicode(&plain_text);

        Ok(Self {
            plain_text,
            char_count,
            line_count,
            has_bidi_text,
            has_complex_unicode,
            override_tags,
            parse_diagnostics,
        })
    }

    /// Get plain text without override tags
    #[must_use] pub fn plain_text(&self) -> &str {
        &self.plain_text
    }

    /// Get Unicode character count
    #[must_use] pub const fn char_count(&self) -> usize {
        self.char_count
    }

    /// Get line count after processing linebreaks
    #[must_use] pub const fn line_count(&self) -> usize {
        self.line_count
    }

    /// Check if text contains bidirectional content
    #[must_use] pub const fn has_bidi_text(&self) -> bool {
        self.has_bidi_text
    }

    /// Check if text contains complex Unicode beyond basic Latin
    #[must_use] pub const fn has_complex_unicode(&self) -> bool {
        self.has_complex_unicode
    }

    /// Get parsed override tags
    #[must_use] pub fn override_tags(&self) -> &[OverrideTag<'a>] {
        &self.override_tags
    }

    /// Get parse diagnostics collected during analysis
    #[must_use] pub fn diagnostics(&self) -> &[TagDiagnostic<'a>] {
        &self.parse_diagnostics
    }

    /// Detect bidirectional text (RTL scripts)
    fn detect_bidi_text(text: &str) -> bool {
        text.chars().any(|ch| matches!(ch as u32, 0x0590..=0x05FF | 0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF))
    }

    /// Detect complex Unicode beyond basic Latin
    fn detect_complex_unicode(text: &str) -> bool {
        text.chars().any(|ch| {
            let code = ch as u32;
            code > 0x00FF || matches!(code, 0x0000..=0x001F | 0x007F..=0x009F | 0x200C..=0x200D | 0x2060..=0x206F)
        })
    }
}
