//! Text segmentation for handling inline formatting changes

mod apply;
mod split;

use apply::process_tag_block;

use crate::utils::RenderError;
use ass_core::analysis::events::TextAnalysis;
use ass_core::ExtensionRegistry;

#[cfg(feature = "nostd")]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
#[cfg(not(feature = "nostd"))]
use std::{
    string::{String, ToString},
    vec::Vec,
};

/// A text segment with its own formatting
#[derive(Debug, Clone)]
pub struct TextSegment {
    /// The plain text content
    pub text: String,
    /// Starting position in original text
    pub start: usize,
    /// Ending position in original text  
    pub end: usize,
    /// Active tags for this segment
    pub tags: super::tag_processor::ProcessedTags,
}

/// Process text with inline tag changes into segments
pub fn segment_text_with_tags(
    text: &str,
    _registry: Option<&ExtensionRegistry>,
) -> Result<Vec<TextSegment>, RenderError> {
    // Analyze text to get tags and their positions
    #[cfg(feature = "plugins")]
    let analysis = TextAnalysis::analyze_with_registry(text, _registry)
        .map_err(|e| RenderError::InvalidScript(e.to_string()))?;

    #[cfg(not(feature = "plugins"))]
    let analysis =
        TextAnalysis::analyze(text).map_err(|e| RenderError::InvalidScript(e.to_string()))?;

    let tags = analysis.override_tags();
    if tags.is_empty() {
        // No tags, return single segment with plain text
        return Ok(vec![TextSegment {
            text: analysis.plain_text().to_string(),
            start: 0,
            end: text.len(),
            tags: super::tag_processor::ProcessedTags::default(),
        }]);
    }

    // Build segments based on tag positions
    let mut segments = Vec::new();
    let mut current_tags = super::tag_processor::ProcessedTags::default();
    let mut last_pos = 0;
    let mut current_text = String::new();

    // Process text character by character, tracking tag blocks
    let mut chars = text.chars();
    let mut pos = 0;
    let mut in_tag = false;
    let mut tag_start = 0;
    let mut brace_depth = 0;

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if !in_tag {
                // Start of tag block
                if !current_text.is_empty() {
                    // Save current segment
                    segments.push(TextSegment {
                        text: current_text.clone(),
                        start: last_pos,
                        end: pos,
                        tags: current_tags.clone(),
                    });
                    current_text.clear();
                    last_pos = pos;
                }
                in_tag = true;
                tag_start = pos;
                brace_depth = 1;
            } else {
                brace_depth += 1;
            }
        } else if ch == '}' && in_tag {
            brace_depth -= 1;
            if brace_depth == 0 {
                // End of tag block - process tags in this block
                let tag_content = &text[tag_start + 1..pos];

                // Check if this block contains karaoke tags
                let has_karaoke = tag_content.contains("\\k") || tag_content.contains("\\K");

                if has_karaoke && !current_text.is_empty() {
                    // If we have karaoke and accumulated text, save it as a segment first
                    segments.push(TextSegment {
                        text: current_text.clone(),
                        start: last_pos,
                        end: tag_start,
                        tags: current_tags.clone(),
                    });
                    current_text.clear();
                }

                process_tag_block(tag_content, &mut current_tags)?;
                in_tag = false;
                last_pos = pos + 1;
            }
        } else if !in_tag {
            // Regular text
            if ch == '\\' {
                // Check for escape sequences
                if let Some(next) = chars.next() {
                    pos += next.len_utf8();
                    match next {
                        'N' | 'n' => current_text.push('\n'),
                        'h' => current_text.push('\u{00A0}'),
                        _ => {
                            current_text.push(ch);
                            current_text.push(next);
                        }
                    }
                } else {
                    current_text.push(ch);
                }
            } else {
                current_text.push(ch);
            }
        }

        pos += ch.len_utf8();
    }

    // Add final segment if there's remaining text
    if !current_text.is_empty() || segments.is_empty() {
        segments.push(TextSegment {
            text: current_text,
            start: last_pos,
            end: text.len(),
            tags: current_tags,
        });
    }

    Ok(segments)
}

// Re-export helper functions from tag_processor
pub use super::tag_processor::{parse_alpha, parse_color, parse_move_args, parse_pos_args};
