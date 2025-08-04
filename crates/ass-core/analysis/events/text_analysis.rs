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
    utils::{errors::resource::check_depth_limit, CoreError},
    Result,
};

#[cfg(feature = "plugins")]
use crate::analysis::events::tags::parse_override_block_with_registry;

#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;
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
    ///
    /// # Errors
    ///
    /// Returns an error if text parsing fails or contains invalid override tags.
    pub fn analyze(text: &'a str) -> Result<Self> {
        #[cfg(feature = "plugins")]
        return Self::analyze_with_registry(text, None);
        #[cfg(not(feature = "plugins"))]
        return Self::analyze_impl(text);
    }

    /// Analyze dialogue text content with extension registry support
    ///
    /// Same as [`analyze`](Self::analyze) but allows custom tag handlers via registry.
    /// Unhandled tags fall back to standard processing.
    ///
    /// # Arguments
    ///
    /// * `text` - Original dialogue text with potential override tags
    /// * `registry` - Optional registry for custom tag handlers
    ///
    /// # Returns
    ///
    /// Complete text analysis results or parsing error.
    ///
    /// # Errors
    ///
    /// Returns an error if text parsing fails or contains invalid override tags.
    #[cfg(feature = "plugins")]
    pub fn analyze_with_registry(
        text: &'a str,
        registry: Option<&ExtensionRegistry>,
    ) -> Result<Self> {
        Self::analyze_impl_with_registry(text, registry)
    }

    /// Internal implementation without plugins support
    #[cfg(not(feature = "plugins"))]
    fn analyze_impl(text: &'a str) -> Result<Self> {
        Self::analyze_impl_with_registry(text)
    }

    /// Internal implementation of analysis with optional registry support
    fn analyze_impl_with_registry(
        text: &'a str,
        #[cfg(feature = "plugins")] registry: Option<&ExtensionRegistry>,
    ) -> Result<Self> {
        const MAX_BRACE_DEPTH: usize = 100; // Prevent DoS with deeply nested braces

        let mut override_tags = Vec::new();
        let mut parse_diagnostics = Vec::new();

        let mut plain_text = String::new();
        let mut position = 0;
        let mut drawing_mode = false;

        let mut chars = text.chars();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut brace_count = 1;
                let tag_start = position + ch.len_utf8();

                for inner_ch in chars.by_ref() {
                    position += inner_ch.len_utf8();

                    if inner_ch == '{' {
                        brace_count += 1;
                        // Check for excessive nesting depth to prevent DoS
                        if check_depth_limit(brace_count, MAX_BRACE_DEPTH).is_err() {
                            return Err(CoreError::parse("Maximum brace nesting depth exceeded"));
                        }
                    } else if inner_ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                }

                if position > tag_start {
                    let tag_content = &text[tag_start..position];

                    #[cfg(feature = "plugins")]
                    if let Some(registry) = registry {
                        parse_override_block_with_registry(
                            tag_content,
                            tag_start,
                            &mut override_tags,
                            &mut parse_diagnostics,
                            Some(registry),
                        );
                    } else {
                        parse_override_block(
                            tag_content,
                            tag_start,
                            &mut override_tags,
                            &mut parse_diagnostics,
                        );
                    }

                    #[cfg(not(feature = "plugins"))]
                    parse_override_block(
                        tag_content,
                        tag_start,
                        &mut override_tags,
                        &mut parse_diagnostics,
                    );

                    // Check for drawing mode changes in this tag block
                    drawing_mode = Self::update_drawing_mode(tag_content, drawing_mode);
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
                        'n' | 'N' => {
                            if !drawing_mode {
                                plain_text.push('\n');
                            }
                        }
                        'h' => {
                            if !drawing_mode {
                                plain_text.push('\u{00A0}');
                            }
                        }
                        _ => {
                            if !drawing_mode {
                                plain_text.push(ch);
                                plain_text.push(next_ch);
                            }
                        }
                    }
                }
            } else if !drawing_mode {
                plain_text.push(ch);
            }

            position += ch.len_utf8();
        }

        let char_count = plain_text.chars().count();
        let line_count = Self::count_lines(&plain_text);
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
    #[must_use]
    pub fn plain_text(&self) -> &str {
        &self.plain_text
    }

    /// Get Unicode character count
    #[must_use]
    pub const fn char_count(&self) -> usize {
        self.char_count
    }

    /// Get line count after processing linebreaks
    #[must_use]
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    /// Check if text contains bidirectional content
    #[must_use]
    pub const fn has_bidi_text(&self) -> bool {
        self.has_bidi_text
    }

    /// Check if text contains complex Unicode beyond basic Latin
    #[must_use]
    pub const fn has_complex_unicode(&self) -> bool {
        self.has_complex_unicode
    }

    /// Get parsed override tags
    #[must_use]
    pub fn override_tags(&self) -> &[OverrideTag<'a>] {
        &self.override_tags
    }

    /// Get parse diagnostics collected during analysis
    #[must_use]
    pub fn diagnostics(&self) -> &[TagDiagnostic<'a>] {
        &self.parse_diagnostics
    }

    /// Update drawing mode state based on override tag content
    fn update_drawing_mode(tag_content: &str, current_mode: bool) -> bool {
        let mut pos = 0;
        let chars: Vec<char> = tag_content.chars().collect();
        let mut drawing_mode = current_mode;

        while pos < chars.len() {
            if chars[pos] == '\\' && pos + 1 < chars.len() && chars[pos + 1] == 'p' {
                pos += 2;
                let mut number_str = String::new();

                while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '-') {
                    number_str.push(chars[pos]);
                    pos += 1;
                }

                if let Ok(p_value) = number_str.parse::<i32>() {
                    drawing_mode = p_value > 0;
                }
            } else {
                pos += 1;
            }
        }

        drawing_mode
    }

    /// Count lines correctly, handling empty lines and trailing newlines
    fn count_lines(text: &str) -> usize {
        if text.is_empty() {
            return 1;
        }

        // For ASS subtitles, count newlines and add 1, but handle special cases
        let newline_count = text.chars().filter(|&ch| ch == '\n').count();

        if newline_count == 0 {
            // No newlines means 1 line
            1
        } else if text.trim_end_matches('\n').is_empty() {
            // Text is only newlines - each newline creates a line boundary
            newline_count + 1
        } else {
            // Text has content - trailing newlines don't create additional lines
            // Use lines() count which handles this correctly
            text.lines().count().max(1)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_analysis_simple_text() {
        let text = "Hello world!";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Hello world!");
        assert_eq!(analysis.char_count(), 12);
        assert_eq!(analysis.line_count(), 1);
        assert!(!analysis.has_bidi_text());
        assert!(!analysis.has_complex_unicode());
        assert!(analysis.override_tags().is_empty());
        assert!(analysis.diagnostics().is_empty());
    }

    #[test]
    fn text_analysis_with_override_tags() {
        let text = "Hello {\\b1}bold{\\b0} world!";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Hello bold world!");
        assert_eq!(analysis.char_count(), 17);
        assert_eq!(analysis.line_count(), 1);
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_nested_braces() {
        let text = "Text {\\pos(100,{\\some}200)} more text";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Text  more text");
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_line_breaks() {
        let text = "First line\\NSecond line\\nThird line";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "First line\nSecond line\nThird line");
        assert_eq!(analysis.line_count(), 3);
    }

    #[test]
    fn text_analysis_hard_spaces() {
        let text = "Text\\hwith\\hhard\\hspaces";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(
            analysis.plain_text(),
            "Text\u{00A0}with\u{00A0}hard\u{00A0}spaces"
        );
    }

    #[test]
    fn text_analysis_mixed_escapes() {
        let text = "Line 1\\NLine 2\\hspace\\nLine 3";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Line 1\nLine 2\u{00A0}space\nLine 3");
        assert_eq!(analysis.line_count(), 3);
    }

    #[test]
    fn text_analysis_bidi_text_arabic() {
        let text = "Hello مرحبا world";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert!(analysis.has_bidi_text());
        assert!(analysis.has_complex_unicode());
    }

    #[test]
    fn text_analysis_bidi_text_hebrew() {
        let text = "Hello שלום world";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert!(analysis.has_bidi_text());
        assert!(analysis.has_complex_unicode());
    }

    #[test]
    fn text_analysis_complex_unicode_emoji() {
        let text = "Hello 🌍 world";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert!(!analysis.has_bidi_text());
        assert!(analysis.has_complex_unicode());
    }

    #[test]
    fn text_analysis_complex_unicode_control_chars() {
        let text = "Text\u{200C}with\u{200D}controls";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert!(analysis.has_complex_unicode());
    }

    #[test]
    fn text_analysis_basic_latin_only() {
        let text = "Basic ASCII text 123!@#";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert!(!analysis.has_bidi_text());
        assert!(!analysis.has_complex_unicode());
    }

    #[test]
    fn text_analysis_extended_latin() {
        let text = "Café naïve résumé";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert!(!analysis.has_bidi_text());
        assert!(!analysis.has_complex_unicode()); // These are still in Latin-1 range
    }

    #[test]
    fn text_analysis_empty_override_blocks() {
        let text = "Text {} more text";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Text  more text");
        // Should have diagnostic for empty override
        assert!(!analysis.diagnostics().is_empty());
    }

    #[test]
    fn text_analysis_unmatched_braces() {
        let text = "Text {\\b1 unmatched";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Text ");
        // Should handle unmatched braces gracefully
    }

    #[test]
    fn text_analysis_multiple_override_blocks() {
        let text = "{\\b1}Bold{\\b0} and {\\i1}italic{\\i0} text";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Bold and italic text");
        assert_eq!(analysis.override_tags().len(), 4);
    }

    #[test]
    fn text_analysis_complex_tags() {
        let text = "{\\move(0,0,100,100)}{\\t(0,1000,\\fscx120)}{\\fade(255,0,0,0,800,900,1000)}Animated text";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Animated text");
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_drawing_commands() {
        let text = "{\\p1}m 0 0 l 100 0 100 100 0 100{\\p0}Square";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Square");
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_color_tags() {
        let text = "{\\c&H0000FF&}Red text{\\c} and {\\1c&H00FF00&}green text";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Red text and green text");
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_mixed_content() {
        let text = "Start {\\b1}bold\\N{\\i1}italic{\\i0}{\\b0}\\hnormal end";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(
            analysis.plain_text(),
            "Start bold\nitalic\u{00A0}normal end"
        );
        assert_eq!(analysis.line_count(), 2);
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_whitespace_only() {
        let text = "   \t\n  ";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "   \t\n  ");
        assert_eq!(analysis.char_count(), 7);
        assert_eq!(analysis.line_count(), 2);
    }

    #[test]
    fn text_analysis_empty_text() {
        let text = "";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "");
        assert_eq!(analysis.char_count(), 0);
        assert_eq!(analysis.line_count(), 1); // Minimum 1 line
        assert!(analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_only_override_tags() {
        let text = "{\\b1}{\\i1}{\\u1}";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "");
        assert_eq!(analysis.char_count(), 0);
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_escape_sequences() {
        let text = "Test`[Events]`backslash and \\{brace and \\}close";
        let analysis = TextAnalysis::analyze(text).unwrap();

        // These should be treated as literal characters, not escape sequences
        assert_eq!(
            analysis.plain_text(),
            "Test`[Events]`backslash and \\{brace and \\}close"
        );
    }

    #[test]
    fn text_analysis_karaoke_tags() {
        let text = "{\\k50}Ka{\\k30}ra{\\k70}o{\\k40}ke";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Karaoke");
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_position_and_rotation() {
        let text = "{\\pos(320,240)}{\\frz45}Rotated positioned text";
        let analysis = TextAnalysis::analyze(text).unwrap();

        assert_eq!(analysis.plain_text(), "Rotated positioned text");
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_very_long_text() {
        let text = "A".repeat(1000);
        let analysis = TextAnalysis::analyze(&text).unwrap();

        assert_eq!(analysis.char_count(), 1000);
        assert_eq!(analysis.plain_text().len(), 1000);
    }

    #[test]
    fn text_analysis_line_count_edge_cases() {
        // Text ending with newline
        let text1 = "Line 1\\nLine 2\\n";
        let analysis1 = TextAnalysis::analyze(text1).unwrap();
        assert_eq!(analysis1.line_count(), 2);

        // Multiple consecutive newlines
        let text2 = "Line 1\\n\\n\\nLine 2";
        let analysis2 = TextAnalysis::analyze(text2).unwrap();
        assert_eq!(analysis2.line_count(), 4);

        // Only newlines
        let text3 = "\\n\\N\\n";
        let analysis3 = TextAnalysis::analyze(text3).unwrap();
        assert_eq!(analysis3.line_count(), 4);
    }

    #[test]
    fn text_analysis_excessive_brace_nesting() {
        // Create deeply nested braces to trigger depth limit error
        let mut text = String::new();
        for _ in 0..110 {
            text.push('{');
        }
        text.push_str("\\b1");
        for _ in 0..110 {
            text.push('}');
        }

        let result = TextAnalysis::analyze(&text);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Maximum brace nesting depth exceeded"));
    }

    #[test]
    fn text_analysis_drawing_mode_escape_sequences() {
        // Test escape sequences in drawing mode - they should not be processed
        let text = "{\\p1}Line1\\nLine2\\hSpace\\NNewline{\\p0}Normal\\ntext";
        let analysis = TextAnalysis::analyze(text).unwrap();

        // In drawing mode, text is ignored entirely from plain_text
        // After {p0}, normal processing resumes
        assert_eq!(analysis.plain_text(), "Normal\ntext");
        assert!(!analysis.override_tags().is_empty());
    }

    #[test]
    fn text_analysis_drawing_mode_p_value_parsing() {
        // Test various p values to trigger drawing mode logic
        let text1 = "{\\p0}Not drawing mode";
        let analysis1 = TextAnalysis::analyze(text1).unwrap();
        assert_eq!(analysis1.plain_text(), "Not drawing mode");

        let text2 = "{\\p1}Drawing mode";
        let analysis2 = TextAnalysis::analyze(text2).unwrap();
        assert_eq!(analysis2.plain_text(), ""); // Drawing mode excludes text

        let text3 = "{\\p5}Also drawing mode";
        let analysis3 = TextAnalysis::analyze(text3).unwrap();
        assert_eq!(analysis3.plain_text(), ""); // Drawing mode excludes text
    }

    #[test]
    fn text_analysis_line_count_only_newlines() {
        // Test line counting when text is only newlines (line 252)
        let text = "\n\n\n";
        let analysis = TextAnalysis::analyze(text).unwrap();
        assert_eq!(analysis.line_count(), 4); // 3 newlines = 4 lines
    }

    #[test]
    fn text_analysis_drawing_mode_mixed_escapes() {
        // Test all escape sequence types in drawing mode
        let text = "{\\p1}Start\\nNew\\NLine\\hHard{\\p0}End\\nNormal";
        let analysis = TextAnalysis::analyze(text).unwrap();

        // Drawing mode excludes all text, normal mode processes escape sequences
        assert_eq!(analysis.plain_text(), "End\nNormal");
        assert!(!analysis.override_tags().is_empty());
    }
}
