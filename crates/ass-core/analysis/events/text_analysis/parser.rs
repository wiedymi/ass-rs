//! Core text analysis parsing implementation.
//!
//! Houses [`TextAnalysis::analyze_impl_with_registry`], which walks the
//! dialogue text, extracts plain text, parses override blocks, and computes
//! the Unicode complexity indicators stored in the result.

use super::TextAnalysis;
use crate::{
    analysis::events::tags::{parse_override_block, DiagnosticKind, TagDiagnostic},
    utils::{errors::resource::check_depth_limit, CoreError},
    Result,
};

#[cfg(feature = "plugins")]
use crate::analysis::events::tags::parse_override_block_with_registry;

#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;
use alloc::{string::String, vec::Vec};

impl<'a> TextAnalysis<'a> {
    /// Internal implementation of analysis with optional registry support
    pub(super) fn analyze_impl_with_registry(
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
}
