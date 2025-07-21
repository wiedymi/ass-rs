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

use crate::Result;
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
}

/// Single ASS override tag with analysis results
///
/// Represents a parsed override tag like `{\b1}` or `{\pos(100,200)}`.
/// Contains zero-copy references to original text for efficiency.
#[derive(Debug, Clone)]
pub struct OverrideTag<'a> {
    /// Tag name (e.g., "b", "pos", "move")
    name: &'a str,
    /// Tag arguments as original text slice
    args: &'a str,
    /// Complexity score for rendering (0-5)
    complexity: u8,
    /// Byte position in original text
    position: usize,
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
                    Self::parse_override_block(tag_content, tag_start, &mut override_tags);
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
        })
    }

    /// Get plain text without override tags
    pub fn plain_text(&self) -> &str {
        &self.plain_text
    }

    /// Get Unicode character count
    pub fn char_count(&self) -> usize {
        self.char_count
    }

    /// Get line count after processing linebreaks
    pub fn line_count(&self) -> usize {
        self.line_count
    }

    /// Check if text contains bidirectional content
    pub fn has_bidi_text(&self) -> bool {
        self.has_bidi_text
    }

    /// Check if text contains complex Unicode beyond basic Latin
    pub fn has_complex_unicode(&self) -> bool {
        self.has_complex_unicode
    }

    /// Get parsed override tags
    pub fn override_tags(&self) -> &[OverrideTag<'a>] {
        &self.override_tags
    }

    /// Parse override tags within a tag block
    fn parse_override_block(content: &'a str, start_pos: usize, tags: &mut Vec<OverrideTag<'a>>) {
        let mut pos = 0;
        let chars: Vec<char> = content.chars().collect();

        while pos < chars.len() {
            if chars[pos] == '\\' && pos + 1 < chars.len() {
                let tag_start = pos;
                pos += 1;

                let name_start = pos;
                while pos < chars.len() && chars[pos].is_ascii_alphabetic() {
                    pos += 1;
                }

                if pos > name_start {
                    let name_end = pos;
                    let args_start = pos;

                    while pos < chars.len() && chars[pos] != '\\' {
                        pos += 1;
                    }

                    let tag_name = &content[name_start..name_end];
                    let args = &content[args_start..pos];
                    let complexity = Self::calculate_tag_complexity(tag_name);

                    tags.push(OverrideTag {
                        name: tag_name,
                        args,
                        complexity,
                        position: start_pos + tag_start,
                    });
                } else {
                    pos += 1;
                }
            } else {
                pos += 1;
            }
        }
    }

    /// Calculate rendering complexity for a tag
    fn calculate_tag_complexity(tag_name: &str) -> u8 {
        match tag_name {
            "b" | "i" | "u" | "s" | "c" | "1c" | "2c" | "3c" | "4c" | "alpha" | "1a" | "2a"
            | "3a" | "4a" | "fn" | "fs" => 1,
            "pos" | "an" | "a" | "org" | "be" | "blur" | "bord" | "shad" | "xbord" | "ybord"
            | "xshad" | "yshad" => 2,
            "move" | "fad" | "fade" | "frx" | "fry" | "frz" | "fscx" | "fscy" | "fsp" | "clip"
            | "iclip" => 3,
            "t" | "pbo" => 4,
            "p" => 5,
            _ => 2,
        }
    }

    /// Detect bidirectional text (RTL scripts)
    fn detect_bidi_text(text: &str) -> bool {
        text.chars().any(|ch| {
            matches!(ch as u32,
                0x0590..=0x05FF | // Hebrew
                0x0600..=0x06FF | // Arabic
                0x0750..=0x077F | // Arabic Supplement
                0x08A0..=0x08FF   // Arabic Extended-A
            )
        })
    }

    /// Detect complex Unicode beyond basic Latin
    fn detect_complex_unicode(text: &str) -> bool {
        text.chars().any(|ch| {
            let code = ch as u32;
            code > 0x00FF
                || matches!(code,
                    0x0000..=0x001F | // Control characters
                    0x007F..=0x009F | // Extended control
                    0x200C..=0x200D | // Zero-width joiners
                    0x2060..=0x206F   // Unicode controls
                )
        })
    }
}

impl<'a> OverrideTag<'a> {
    /// Get tag name
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Get tag arguments
    pub fn args(&self) -> &'a str {
        self.args
    }

    /// Get complexity score
    pub fn complexity(&self) -> u8 {
        self.complexity
    }

    /// Get position in original text
    pub fn position(&self) -> usize {
        self.position
    }
}
