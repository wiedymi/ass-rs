//! Per-tag-name application that updates `ProcessedTags`

use crate::pipeline::tag_processor::{
    parse_alpha, parse_clip_args, parse_color, parse_fade_args, parse_move_args, parse_pos_args,
    KaraokeData, KaraokeStyle, ProcessedTags, TransformData,
};
use crate::pipeline::transform::TransformAnimation;
use crate::utils::RenderError;

use super::split::{extract_tag_name_and_args, split_tags_carefully};

#[cfg(feature = "nostd")]
use alloc::string::ToString;
#[cfg(not(feature = "nostd"))]
use std::string::ToString;

/// Process tags within a tag block and update current tags
pub(super) fn process_tag_block(
    content: &str,
    current: &mut ProcessedTags,
) -> Result<(), RenderError> {
    // Parse tags more carefully to handle nested parentheses in \t tags
    let parts = split_tags_carefully(content);

    for part in &parts {
        if part.is_empty() {
            continue;
        }

        let (name, args) = extract_tag_name_and_args(part);

        // Process based on tag name
        match name {
            // Color tags
            "c" | "1c" => {
                if let Some(color) = parse_color(args) {
                    current.colors.primary = Some(color);
                }
            }
            "2c" => {
                if let Some(color) = parse_color(args) {
                    current.colors.secondary = Some(color);
                }
            }
            "3c" => {
                if let Some(color) = parse_color(args) {
                    current.colors.outline = Some(color);
                }
            }
            "4c" => {
                if let Some(color) = parse_color(args) {
                    current.colors.shadow = Some(color);
                }
            }
            "alpha" => {
                if let Some(alpha) = parse_alpha(args) {
                    current.colors.alpha = Some(alpha);
                }
            }
            "1a" => {
                if let Some(alpha) = parse_alpha(args) {
                    current.colors.alpha1 = Some(alpha);
                }
            }
            "2a" => {
                if let Some(alpha) = parse_alpha(args) {
                    current.colors.alpha2 = Some(alpha);
                }
            }
            "3a" => {
                if let Some(alpha) = parse_alpha(args) {
                    current.colors.alpha3 = Some(alpha);
                }
            }
            "4a" => {
                if let Some(alpha) = parse_alpha(args) {
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
                }
            }
            "blur" => {
                if let Ok(radius) = args.parse::<f32>() {
                    current.formatting.blur = Some(radius);
                }
            }
            "be" => {
                if let Ok(radius) = args.parse::<f32>() {
                    current.formatting.blur_edges = Some(radius);
                }
            }
            "clip" | "iclip" => {
                if let Some(mut clip) = parse_clip_args(args) {
                    clip.inverse = name == "iclip";
                    current.clip = Some(clip);
                }
            }
            "fax" => {
                if let Ok(shear) = args.parse::<f32>() {
                    current.shear_x = Some(shear);
                }
            }
            "fay" => {
                if let Ok(shear) = args.parse::<f32>() {
                    current.shear_y = Some(shear);
                }
            }
            "org" => {
                if let Some((x, y)) = parse_pos_args(args) {
                    current.origin = Some((x, y));
                }
            }

            // Rotation tags
            "frz" | "fr" => {
                if let Ok(angle) = args.parse::<f32>() {
                    current.font.rotation_z = Some(angle);
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

            // Border / outline width
            "bord" => {
                if let Ok(w) = args.parse::<f32>() {
                    current.formatting.border = Some(w);
                }
            }
            "xbord" => {
                if let Ok(w) = args.parse::<f32>() {
                    current.formatting.border_x = Some(w);
                }
            }
            "ybord" => {
                if let Ok(w) = args.parse::<f32>() {
                    current.formatting.border_y = Some(w);
                }
            }
            // Shadow depth
            "shad" => {
                if let Ok(d) = args.parse::<f32>() {
                    current.formatting.shadow = Some(d);
                }
            }
            "xshad" => {
                if let Ok(d) = args.parse::<f32>() {
                    current.formatting.shadow_x = Some(d);
                }
            }
            "yshad" => {
                if let Ok(d) = args.parse::<f32>() {
                    current.formatting.shadow_y = Some(d);
                }
            }
            // Wrap style
            "q" => {
                if let Ok(wrap) = args.parse::<u8>() {
                    current.formatting.wrap_style = Some(wrap);
                }
            }
            // \r resets all inline overrides back to the line's style.
            "r" => {
                *current = ProcessedTags::default();
            }

            // Alignment tags
            "a" | "an" => {
                if let Ok(align) = args.parse::<u8>() {
                    current.formatting.alignment = Some(align);
                }
            }

            // Position tags - these apply to the whole event, not segments
            "pos" => {
                if let Some((x, y)) = parse_pos_args(args) {
                    current.position = Some((x, y));
                }
            }
            "move" => {
                if let Some(data) = parse_move_args(args) {
                    current.movement = Some(data);
                }
            }

            // Fade effects
            "fad" | "fade" => {
                // Import the parse_fade_args function
                if let Some(fade) = parse_fade_args(args) {
                    current.fade = Some(fade);
                }
            }

            // Drawing mode
            "p" => {
                if let Ok(level) = args.parse::<u8>() {
                    current.drawing_mode = Some(level);
                }
            }

            // Transform animation
            "t" => {
                // Transform tags need special handling for nested tags
                if let Some(animation) = TransformAnimation::parse(args) {
                    current.transforms.push(TransformData { animation });
                }
            }

            // Karaoke tags
            "k" => {
                if let Ok(duration) = args.parse::<u32>() {
                    current.karaoke = Some(KaraokeData {
                        duration,
                        style: KaraokeStyle::Basic,
                        start_time: None,
                    });
                }
            }
            "kf" => {
                if let Ok(duration) = args.parse::<u32>() {
                    current.karaoke = Some(KaraokeData {
                        duration,
                        style: KaraokeStyle::Fill,
                        start_time: None,
                    });
                }
            }
            "ko" => {
                if let Ok(duration) = args.parse::<u32>() {
                    current.karaoke = Some(KaraokeData {
                        duration,
                        style: KaraokeStyle::Outline,
                        start_time: None,
                    });
                }
            }
            "K" => {
                if let Ok(duration) = args.parse::<u32>() {
                    current.karaoke = Some(KaraokeData {
                        duration,
                        style: KaraokeStyle::Sweep,
                        start_time: None,
                    });
                }
            }

            _ => {
                // Other tags can be added as needed
            }
        }
    }

    Ok(())
}
