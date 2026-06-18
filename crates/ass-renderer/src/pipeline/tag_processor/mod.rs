//! Tag processing for events using ass-core's tag parsing

mod parse;
mod types;

pub use parse::{
    parse_alpha, parse_clip_args, parse_color, parse_fade_args, parse_move_args, parse_pos_args,
};
pub use types::{
    ClipData, ColorOverrides, FadeData, FontOverrides, FormattingOverrides, KaraokeData,
    KaraokeStyle, LineBreakType, ProcessedTags, TransformData,
};

use crate::pipeline::transform::TransformAnimation;
use crate::utils::RenderError;
#[cfg(feature = "nostd")]
use alloc::string::{String, ToString};
use ass_core::analysis::events::{OverrideTag, TextAnalysis};
use ass_core::ExtensionRegistry;
#[cfg(not(feature = "nostd"))]
use std::string::{String, ToString};

/// Process tags from event text
pub fn process_event_tags(
    text: &str,
    _registry: Option<&ExtensionRegistry>,
) -> Result<(ProcessedTags, String), RenderError> {
    // Analyze text to extract tags
    #[cfg(feature = "plugins")]
    let analysis = TextAnalysis::analyze_with_registry(text, _registry)
        .map_err(|e| RenderError::InvalidScript(e.to_string()))?;

    #[cfg(not(feature = "plugins"))]
    let analysis =
        TextAnalysis::analyze(text).map_err(|e| RenderError::InvalidScript(e.to_string()))?;

    let mut processed = ProcessedTags::default();

    // Process each tag
    for tag in analysis.override_tags() {
        process_single_tag(tag, &mut processed)?;
    }

    // Process line breaks and special characters
    // Note: ass-core's TextAnalysis already converts \N, \n, \h to actual characters
    let plain_text = analysis.plain_text();
    let mut clean_text = String::new();

    for ch in plain_text.chars() {
        match ch {
            '\n' => {
                // Track line break positions
                // We can't distinguish between \N (hard) and \n (soft) after conversion
                // For now, treat all as hard breaks
                processed
                    .line_breaks
                    .push(LineBreakType::Hard(clean_text.len()));
                clean_text.push('\n');
            }
            '\u{00A0}' => {
                // Non-breaking space (from \h)
                processed.nbsp_positions.push(clean_text.len());
                clean_text.push('\u{00A0}');
            }
            _ => {
                clean_text.push(ch);
            }
        }
    }

    Ok((processed, clean_text))
}

fn process_single_tag(tag: &OverrideTag, processed: &mut ProcessedTags) -> Result<(), RenderError> {
    match tag.name() {
        // Position tags
        "pos" => {
            if let Some((x, y)) = parse_pos_args(tag.args()) {
                processed.position = Some((x, y));
            }
        }
        "move" => {
            if let Some(data) = parse_move_args(tag.args()) {
                processed.movement = Some(data);
            }
        }
        "org" => {
            if let Some((x, y)) = parse_pos_args(tag.args()) {
                processed.origin = Some((x, y));
            }
        }

        // Color tags
        "c" | "1c" => {
            if let Some(color) = parse_color(tag.args()) {
                processed.colors.primary = Some(color);
            }
        }
        "2c" => {
            if let Some(color) = parse_color(tag.args()) {
                processed.colors.secondary = Some(color);
            }
        }
        "3c" => {
            if let Some(color) = parse_color(tag.args()) {
                processed.colors.outline = Some(color);
            }
        }
        "4c" => {
            if let Some(color) = parse_color(tag.args()) {
                processed.colors.shadow = Some(color);
            }
        }
        "alpha" => {
            if let Some(alpha) = parse_alpha(tag.args()) {
                processed.colors.alpha = Some(alpha);
            }
        }
        "1a" => {
            if let Some(alpha) = parse_alpha(tag.args()) {
                processed.colors.alpha1 = Some(alpha);
            }
        }
        "2a" => {
            if let Some(alpha) = parse_alpha(tag.args()) {
                processed.colors.alpha2 = Some(alpha);
            }
        }
        "3a" => {
            if let Some(alpha) = parse_alpha(tag.args()) {
                processed.colors.alpha3 = Some(alpha);
            }
        }
        "4a" => {
            if let Some(alpha) = parse_alpha(tag.args()) {
                processed.colors.alpha4 = Some(alpha);
            }
        }

        // Font tags
        "fn" => {
            processed.font.name = Some(tag.args().trim().to_string());
        }
        "fs" => {
            if let Ok(size) = tag.args().parse::<f32>() {
                processed.font.size = Some(size);
            }
        }
        "fscx" => {
            if let Ok(scale) = tag.args().parse::<f32>() {
                processed.font.scale_x = Some(scale); // Keep as percentage
            }
        }
        "fscy" => {
            if let Ok(scale) = tag.args().parse::<f32>() {
                processed.font.scale_y = Some(scale); // Keep as percentage
            }
        }
        "fsp" => {
            if let Ok(spacing) = tag.args().parse::<f32>() {
                processed.font.spacing = Some(spacing);
            }
        }
        "frz" | "fr" => {
            if let Ok(angle) = tag.args().parse::<f32>() {
                processed.font.angle = Some(angle);
                processed.font.rotation_z = Some(angle);
            }
        }
        "frx" => {
            if let Ok(angle) = tag.args().parse::<f32>() {
                processed.font.rotation_x = Some(angle);
            }
        }
        "fry" => {
            if let Ok(angle) = tag.args().parse::<f32>() {
                processed.font.rotation_y = Some(angle);
            }
        }
        "fax" => {
            if let Ok(shear) = tag.args().parse::<f32>() {
                processed.shear_x = Some(shear);
            }
        }
        "fay" => {
            if let Ok(shear) = tag.args().parse::<f32>() {
                processed.shear_y = Some(shear);
            }
        }
        "fe" => {
            if let Ok(encoding) = tag.args().parse::<u8>() {
                processed.font.encoding = Some(encoding);
            }
        }

        // Formatting tags
        "b" => {
            processed.formatting.bold = Some(tag.args() != "0");
        }
        "i" => {
            processed.formatting.italic = Some(tag.args() != "0");
        }
        "u" => {
            processed.formatting.underline = Some(tag.args() != "0");
        }
        "s" => {
            processed.formatting.strikeout = Some(tag.args() != "0");
        }
        "bord" => {
            if let Ok(width) = tag.args().parse::<f32>() {
                processed.formatting.border = Some(width);
            }
        }
        "xbord" => {
            if let Ok(width) = tag.args().parse::<f32>() {
                processed.formatting.border_x = Some(width);
            }
        }
        "ybord" => {
            if let Ok(width) = tag.args().parse::<f32>() {
                processed.formatting.border_y = Some(width);
            }
        }
        "shad" => {
            if let Ok(depth) = tag.args().parse::<f32>() {
                processed.formatting.shadow = Some(depth);
            }
        }
        "xshad" => {
            if let Ok(depth) = tag.args().parse::<f32>() {
                processed.formatting.shadow_x = Some(depth);
            }
        }
        "yshad" => {
            if let Ok(depth) = tag.args().parse::<f32>() {
                processed.formatting.shadow_y = Some(depth);
            }
        }
        "be" => {
            if let Ok(blur) = tag.args().parse::<f32>() {
                processed.formatting.blur_edges = Some(blur);
            }
        }
        "blur" => {
            if let Ok(blur) = tag.args().parse::<f32>() {
                processed.formatting.blur = Some(blur);
            }
        }
        "a" | "an" => {
            if let Ok(align) = tag.args().parse::<u8>() {
                processed.formatting.alignment = Some(align);
            }
        }
        "q" => {
            if let Ok(wrap) = tag.args().parse::<u8>() {
                processed.formatting.wrap_style = Some(wrap);
            }
        }
        "r" => {
            // Reset to style
            processed.reset = Some(tag.args().trim().to_string());
        }
        "pbo" => {
            // Baseline offset
            if let Ok(offset) = tag.args().parse::<f32>() {
                processed.baseline_offset = Some(offset);
            }
        }

        // Drawing mode
        "p" => {
            if let Ok(level) = tag.args().parse::<u8>() {
                processed.drawing_mode = Some(level);
            }
        }

        // Clipping
        "clip" | "iclip" => {
            if let Some(clip) = parse_clip_args(tag.args()) {
                let mut clip = clip;
                clip.inverse = tag.name() == "iclip";
                processed.clip = Some(clip);
            }
        }

        // Fade effects
        "fad" | "fade" => {
            if let Some(fade) = parse_fade_args(tag.args()) {
                processed.fade = Some(fade);
            }
        }

        // Karaoke (durations are already in centiseconds in ASS format)
        "k" => {
            if let Ok(duration) = tag.args().parse::<u32>() {
                processed.karaoke = Some(KaraokeData {
                    duration, // Already in centiseconds
                    style: KaraokeStyle::Basic,
                    start_time: None,
                });
            }
        }
        "kf" => {
            if let Ok(duration) = tag.args().parse::<u32>() {
                processed.karaoke = Some(KaraokeData {
                    duration, // Already in centiseconds
                    style: KaraokeStyle::Fill,
                    start_time: None,
                });
            }
        }
        "ko" => {
            if let Ok(duration) = tag.args().parse::<u32>() {
                processed.karaoke = Some(KaraokeData {
                    duration, // Already in centiseconds
                    style: KaraokeStyle::Outline,
                    start_time: None,
                });
            }
        }
        "K" => {
            // Capital K is sweep karaoke
            if let Ok(duration) = tag.args().parse::<u32>() {
                processed.karaoke = Some(KaraokeData {
                    duration, // Already in centiseconds
                    style: KaraokeStyle::Sweep,
                    start_time: None,
                });
            }
        }
        "kt" => {
            // Karaoke syllable start time
            if let Ok(start_time) = tag.args().parse::<u32>() {
                if let Some(ref mut karaoke) = processed.karaoke {
                    karaoke.start_time = Some(start_time);
                } else {
                    processed.karaoke = Some(KaraokeData {
                        duration: 0,
                        style: KaraokeStyle::Basic,
                        start_time: Some(start_time),
                    });
                }
            }
        }

        // Transform animation
        "t" => {
            // Parse transform animation
            if let Some(transform) = TransformAnimation::parse(tag.args()) {
                // Add transform to list (can have multiple)
                processed.transforms.push(TransformData {
                    animation: transform,
                });
            }
        }

        _ => {
            // Unknown tag or handled by plugin
        }
    }

    Ok(())
}
