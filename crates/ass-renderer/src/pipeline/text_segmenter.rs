//! Text segmentation for handling inline formatting changes

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

/// Process tags within a tag block and update current tags
fn process_tag_block(
    content: &str,
    current: &mut super::tag_processor::ProcessedTags,
) -> Result<(), RenderError> {
    // Parse tags more carefully to handle nested parentheses in \t tags
    let parts = split_tags_carefully(content);

    #[cfg(all(debug_assertions, not(feature = "nostd")))]
    eprintln!("DEBUG: Processing tag block: '{content}', parts: {parts:?}");

    for part in &parts {
        if part.is_empty() {
            continue;
        }

        // Extract tag name and arguments
        // Tag names can be alphabetic or start with a digit (like 1c, 3a, 4a)
        // Special handling for font name tag (fn) which can have letters immediately after
        let (name, args) = if part.starts_with("fn") && part.len() > 2 {
            // Special case for \fn tag - everything after "fn" is the font name
            ("fn", &part[2..])
        } else if part.starts_with(|c: char| c.is_ascii_digit()) {
            // Handle tags like 1c, 2c, 3a, 4a
            if let Some(idx) = part[1..]
                .find(|c: char| !c.is_ascii_alphabetic())
                .map(|i| i + 1)
            {
                (&part[..idx], &part[idx..])
            } else {
                (part.as_str(), "")
            }
        } else if let Some(idx) = part.find(|c: char| !c.is_ascii_alphabetic()) {
            (&part[..idx], &part[idx..])
        } else {
            (part.as_str(), "")
        };

        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        eprintln!("DEBUG: Tag name='{name}', args='{args}'");

        // Process based on tag name
        match name {
            // Color tags
            "c" | "1c" => {
                if let Some(color) = super::tag_processor::parse_color(args) {
                    current.colors.primary = Some(color);
                }
            }
            "2c" => {
                if let Some(color) = super::tag_processor::parse_color(args) {
                    current.colors.secondary = Some(color);
                }
            }
            "3c" => {
                if let Some(color) = super::tag_processor::parse_color(args) {
                    current.colors.outline = Some(color);
                }
            }
            "4c" => {
                if let Some(color) = super::tag_processor::parse_color(args) {
                    current.colors.shadow = Some(color);
                }
            }
            "alpha" => {
                if let Some(alpha) = super::tag_processor::parse_alpha(args) {
                    current.colors.alpha = Some(alpha);
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Set alpha={alpha} from args '{args}'");
                } else {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Failed to parse alpha from '{args}'");
                }
            }
            "1a" => {
                if let Some(alpha) = super::tag_processor::parse_alpha(args) {
                    current.colors.alpha1 = Some(alpha);
                }
            }
            "2a" => {
                if let Some(alpha) = super::tag_processor::parse_alpha(args) {
                    current.colors.alpha2 = Some(alpha);
                }
            }
            "3a" => {
                if let Some(alpha) = super::tag_processor::parse_alpha(args) {
                    current.colors.alpha3 = Some(alpha);
                }
            }
            "4a" => {
                if let Some(alpha) = super::tag_processor::parse_alpha(args) {
                    current.colors.alpha4 = Some(alpha);
                }
            }

            // Font tags
            "fn" => {
                // Empty \fn resets to style default
                if args.is_empty() {
                    current.font.name = None;
                } else {
                    current.font.name = Some(args.to_string());
                }
            }
            "fs" => {
                if let Ok(size) = args.parse::<f32>() {
                    current.font.size = Some(size);
                }
            }
            "b" => {
                current.formatting.bold = Some(args != "0");
            }
            "i" => {
                current.formatting.italic = Some(args != "0");
            }
            "u" => {
                current.formatting.underline = Some(args != "0");
            }
            "s" => {
                current.formatting.strikeout = Some(args != "0");
            }

            // Font scale and spacing tags
            "fscx" => {
                if let Ok(scale) = args.parse::<f32>() {
                    current.font.scale_x = Some(scale);
                }
            }
            "fscy" => {
                if let Ok(scale) = args.parse::<f32>() {
                    current.font.scale_y = Some(scale);
                }
            }
            "fsp" => {
                if let Ok(spacing) = args.parse::<f32>() {
                    current.font.spacing = Some(spacing);
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("DEBUG text_segmenter: Parsed \\fsp{spacing} tag");
                }
            }

            // Rotation tags
            "frz" | "fr" => {
                if let Ok(angle) = args.parse::<f32>() {
                    current.font.rotation_z = Some(angle);
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Set rotation_z={angle} from \\frz");
                }
            }
            "frx" => {
                if let Ok(angle) = args.parse::<f32>() {
                    current.font.rotation_x = Some(angle);
                }
            }
            "fry" => {
                if let Ok(angle) = args.parse::<f32>() {
                    current.font.rotation_y = Some(angle);
                }
            }

            // Alignment tags
            "a" | "an" => {
                if let Ok(align) = args.parse::<u8>() {
                    current.formatting.alignment = Some(align);
                }
            }

            // Position tags - these apply to the whole event, not segments
            "pos" => {
                if let Some((x, y)) = super::tag_processor::parse_pos_args(args) {
                    current.position = Some((x, y));
                }
            }
            "move" => {
                if let Some(data) = super::tag_processor::parse_move_args(args) {
                    current.movement = Some(data);
                }
            }

            // Fade effects
            "fad" | "fade" => {
                // Import the parse_fade_args function
                if let Some(fade) = super::tag_processor::parse_fade_args(args) {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Parsed fade tag with args '{}' -> FadeData {{ time_start: {}, time_end: {} }}", 
                        args, fade.time_start, fade.time_end);
                    current.fade = Some(fade);
                }
            }

            // Drawing mode
            "p" => {
                if let Ok(level) = args.parse::<u8>() {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Parsed drawing mode \\p{level}");
                    current.drawing_mode = Some(level);
                }
            }

            // Transform animation
            "t" => {
                // Transform tags need special handling for nested tags
                use crate::pipeline::tag_processor::TransformData;
                use crate::pipeline::transform::TransformAnimation;
                if let Some(animation) = TransformAnimation::parse(args) {
                    current.transforms.push(TransformData { animation });
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Added transform animation from args '{args}'");
                } else {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Failed to parse transform from '{args}'");
                }
            }

            // Karaoke tags
            "k" => {
                if let Ok(duration) = args.parse::<u32>() {
                    current.karaoke = Some(super::tag_processor::KaraokeData {
                        duration,
                        style: super::tag_processor::KaraokeStyle::Basic,
                        start_time: None,
                    });
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Set karaoke k={duration}");
                }
            }
            "kf" => {
                if let Ok(duration) = args.parse::<u32>() {
                    current.karaoke = Some(super::tag_processor::KaraokeData {
                        duration,
                        style: super::tag_processor::KaraokeStyle::Fill,
                        start_time: None,
                    });
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Set karaoke kf={duration}");
                }
            }
            "ko" => {
                if let Ok(duration) = args.parse::<u32>() {
                    current.karaoke = Some(super::tag_processor::KaraokeData {
                        duration,
                        style: super::tag_processor::KaraokeStyle::Outline,
                        start_time: None,
                    });
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Set karaoke ko={duration}");
                }
            }
            "K" => {
                if let Ok(duration) = args.parse::<u32>() {
                    current.karaoke = Some(super::tag_processor::KaraokeData {
                        duration,
                        style: super::tag_processor::KaraokeStyle::Sweep,
                        start_time: None,
                    });
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    eprintln!("TEXT_SEGMENTER: Set karaoke K={duration}");
                }
            }

            _ => {
                // Other tags can be added as needed
            }
        }
    }

    Ok(())
}

// Re-export helper functions from tag_processor
pub use super::tag_processor::{parse_alpha, parse_color, parse_move_args, parse_pos_args};

/// Split tags carefully, handling nested parentheses in \t tags
fn split_tags_carefully(content: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' && depth == 0 {
            // Start of a new tag
            if !current.is_empty() {
                parts.push(current);
                current = String::new();
            }
        } else {
            current.push(ch);

            // Track parentheses depth for \t tags
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;

                // If we closed all parentheses, this tag is complete
                if depth == 0 && !current.is_empty() {
                    // Check if next char is a backslash (new tag) or continue
                    if chars.peek() == Some(&'\\') {
                        parts.push(current.clone());
                        current = String::new();
                    }
                }
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}
