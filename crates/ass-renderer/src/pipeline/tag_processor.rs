//! Tag processing for events using ass-core's tag parsing

use crate::pipeline::transform::TransformAnimation;
use crate::utils::RenderError;
#[cfg(feature = "nostd")]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use ass_core::analysis::events::TextAnalysis;
#[cfg(not(feature = "nostd"))]
use std::collections::HashMap;
#[cfg(not(feature = "nostd"))]
use std::{
    eprintln,
    string::{String, ToString},
    vec::Vec,
};

/// Processed tag values ready for rendering
#[derive(Debug, Clone)]
pub struct ProcessedTags {
    /// Position override (x, y)
    pub position: Option<(f32, f32)>,
    /// Move command (x1, y1, x2, y2, t1, t2)
    pub movement: Option<(f32, f32, f32, f32, u32, u32)>,
    /// Origin point for rotation (x, y)
    pub origin: Option<(f32, f32)>,
    /// Color overrides (primary, secondary, outline, shadow)
    pub colors: ColorOverrides,
    /// Font overrides
    pub font: FontOverrides,
    /// Text formatting overrides
    pub formatting: FormattingOverrides,
    /// Transform/animation - can have multiple transforms
    pub transforms: Vec<TransformData>,
    /// Drawing mode level
    pub drawing_mode: Option<u8>,
    /// Clipping region
    pub clip: Option<ClipData>,
    /// Fade effects
    pub fade: Option<FadeData>,
    /// Karaoke timing
    pub karaoke: Option<KaraokeData>,
    /// Reset style
    pub reset: Option<String>,
    /// Baseline offset
    pub baseline_offset: Option<f32>,
    /// Perspective (\fax, \fay)
    pub shear_x: Option<f32>,
    pub shear_y: Option<f32>,
    /// Line breaks and special characters
    pub line_breaks: Vec<LineBreakType>,
    /// Non-breaking spaces positions
    pub nbsp_positions: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct ColorOverrides {
    pub primary: Option<[u8; 4]>,
    pub secondary: Option<[u8; 4]>,
    pub outline: Option<[u8; 4]>,
    pub shadow: Option<[u8; 4]>,
    pub alpha: Option<u8>,
    pub alpha1: Option<u8>, // \1a
    pub alpha2: Option<u8>, // \2a
    pub alpha3: Option<u8>, // \3a
    pub alpha4: Option<u8>, // \4a
}

#[derive(Debug, Clone, Default)]
pub struct FontOverrides {
    pub name: Option<String>,
    pub size: Option<f32>,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub spacing: Option<f32>,
    pub angle: Option<f32>,
    pub rotation_x: Option<f32>, // \frx
    pub rotation_y: Option<f32>, // \fry
    pub rotation_z: Option<f32>, // \frz
    pub encoding: Option<u8>,    // \fe
}

#[derive(Debug, Clone, Default)]
pub struct FormattingOverrides {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikeout: Option<bool>,
    pub border: Option<f32>,
    pub border_x: Option<f32>, // \xbord
    pub border_y: Option<f32>, // \ybord
    pub shadow: Option<f32>,
    pub shadow_x: Option<f32>, // \xshad
    pub shadow_y: Option<f32>, // \yshad
    pub blur: Option<f32>,
    pub blur_edges: Option<f32>, // \be (edge blur)
    pub alignment: Option<u8>,
    pub margin_l: Option<f32>, // \q
    pub margin_r: Option<f32>,
    pub margin_v: Option<f32>,
    pub wrap_style: Option<u8>, // \q
}

#[derive(Debug, Clone)]
pub struct TransformData {
    pub animation: TransformAnimation,
}

#[derive(Debug, Clone)]
pub struct ClipData {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub inverse: bool,
}

#[derive(Debug, Clone)]
pub struct FadeData {
    pub alpha_start: u8,
    pub alpha_end: u8,
    pub time_start: u32,
    pub time_end: u32,
    /// For complex fade with 7 parameters
    pub alpha_middle: Option<u8>,
    pub time_fade_in: Option<u32>,
    pub time_fade_out: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct KaraokeData {
    pub duration: u32,
    pub style: KaraokeStyle,
    /// Karaoke syllable start time in centiseconds (for \kt)
    pub start_time: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum KaraokeStyle {
    Basic,
    Fill,
    Outline,
    Sweep, // For \K capital K
}

#[derive(Debug, Clone)]
pub enum LineBreakType {
    Hard(usize), // \N position in text
    Soft(usize), // \n position in text
}

/// Process tags from event text
pub fn process_event_tags<'a>(
    text: &'a str,
    registry: Option<&ExtensionRegistry>,
) -> Result<(ProcessedTags, String), RenderError> {
    // Analyze text to extract tags
    #[cfg(feature = "plugins")]
    let analysis = TextAnalysis::analyze_with_registry(text, registry)
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
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            eprintln!("PARSING MOVE TAG: args = '{}'", tag.args());

            if let Some(data) = parse_move_args(tag.args()) {
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                eprintln!(
                    "  Parsed move: x1={}, y1={}, x2={}, y2={}, t1={}, t2={}",
                    data.0, data.1, data.2, data.3, data.4, data.5
                );
                processed.movement = Some(data);
            } else {
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                eprintln!("  Failed to parse move args!");
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
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                eprintln!("FADE PARSE: Parsed fade tag with args '{}' -> FadeData {{ alpha_start: {}, alpha_end: {}, time_start: {}, time_end: {}, alpha_middle: {:?} }}", 
                    tag.args(), fade.alpha_start, fade.alpha_end, fade.time_start, fade.time_end, fade.alpha_middle);
                processed.fade = Some(fade);
            }
        }

        // Karaoke (durations are already in centiseconds in ASS format)
        "k" => {
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            eprintln!("TAG PROCESSOR: Found \\k tag with args: '{}'", tag.args());

            if let Ok(duration) = tag.args().parse::<u32>() {
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                eprintln!("TAG PROCESSOR: Parsed karaoke duration: {}", duration);

                processed.karaoke = Some(KaraokeData {
                    duration, // Already in centiseconds
                    style: KaraokeStyle::Basic,
                    start_time: None,
                });
            } else {
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                eprintln!(
                    "TAG PROCESSOR: Failed to parse karaoke duration from '{}'",
                    tag.args()
                );
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

pub fn parse_pos_args(args: &str) -> Option<(f32, f32)> {
    let args = args.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = args.split(',').collect();
    if parts.len() == 2 {
        if let (Ok(x), Ok(y)) = (
            parts[0].trim().parse::<f32>(),
            parts[1].trim().parse::<f32>(),
        ) {
            return Some((x, y));
        }
    }
    None
}

pub fn parse_move_args(args: &str) -> Option<(f32, f32, f32, f32, u32, u32)> {
    let args = args.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = args.split(',').collect();

    if parts.len() >= 4 {
        let x1 = parts[0].trim().parse::<f32>().ok()?;
        let y1 = parts[1].trim().parse::<f32>().ok()?;
        let x2 = parts[2].trim().parse::<f32>().ok()?;
        let y2 = parts[3].trim().parse::<f32>().ok()?;

        // Times in \move are in milliseconds, need to convert to centiseconds
        let t1_ms = if parts.len() > 4 {
            parts[4].trim().parse::<u32>().unwrap_or(0)
        } else {
            0
        };

        let t2_ms = if parts.len() > 5 {
            parts[5].trim().parse::<u32>().unwrap_or(0)
        } else {
            0
        };

        // Convert milliseconds to centiseconds
        let t1 = t1_ms / 10;
        let t2 = t2_ms / 10;

        return Some((x1, y1, x2, y2, t1, t2));
    }

    None
}

pub fn parse_color(args: &str) -> Option<[u8; 4]> {
    // Use ass-core's color parsing
    ass_core::utils::parse_bgr_color(args).ok()
}

pub fn parse_alpha(args: &str) -> Option<u8> {
    let hex = args
        .trim_start_matches("&H")
        .trim_start_matches("&h")
        .trim_end_matches('&');
    // ASS uses inverted alpha: 0 = opaque, 255 = transparent
    // We need to invert it to match standard RGBA: 255 = opaque, 0 = transparent
    u8::from_str_radix(hex, 16)
        .ok()
        .map(|ass_alpha| 255 - ass_alpha)
}

fn parse_clip_args(args: &str) -> Option<ClipData> {
    let args = args.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = args.split(',').collect();

    if parts.len() == 4 {
        let x1 = parts[0].trim().parse::<f32>().ok()?;
        let y1 = parts[1].trim().parse::<f32>().ok()?;
        let x2 = parts[2].trim().parse::<f32>().ok()?;
        let y2 = parts[3].trim().parse::<f32>().ok()?;

        return Some(ClipData {
            x1,
            y1,
            x2,
            y2,
            inverse: false,
        });
    }

    None
}

pub fn parse_fade_args(args: &str) -> Option<FadeData> {
    let args = args.trim_start_matches('(').trim_end_matches(')');
    let parts: Vec<&str> = args.split(',').collect();

    if parts.len() >= 2 {
        // Simple fade: fade_in_time, fade_out_time (in milliseconds)
        if parts.len() == 2 {
            let fade_in_ms = parts[0].trim().parse::<u32>().ok()?;
            let fade_out_ms = parts[1].trim().parse::<u32>().ok()?;

            // Convert milliseconds to centiseconds
            let time_start = fade_in_ms / 10;
            let time_end = fade_out_ms / 10;

            // For simple fade, we store durations not alpha values
            // The actual alpha calculation happens during rendering
            return Some(FadeData {
                alpha_start: 0, // Not used for simple fade
                alpha_end: 0,   // Not used for simple fade
                time_start,     // Fade-in duration in centiseconds
                time_end,       // Fade-out duration in centiseconds
                alpha_middle: None,
                time_fade_in: None,
                time_fade_out: None,
            });
        }

        // Complex fade: alpha1, alpha2, alpha3, t1, t2, t3, t4
        // In ASS format, alpha values are INVERTED: 00=opaque, FF=transparent
        // alpha1: alpha before fade in (00-FF in hex, where 00=opaque, FF=transparent)
        // alpha2: alpha during main display
        // alpha3: alpha after fade out
        // t1-t2: fade in period (in milliseconds)
        // t2-t3: fully visible period (in milliseconds)
        // t3-t4: fade out period (in milliseconds)
        if parts.len() >= 7 {
            let alpha1 = parts[0].trim().parse::<u8>().ok()?;
            let alpha2 = parts[1].trim().parse::<u8>().ok()?;
            let alpha3 = parts[2].trim().parse::<u8>().ok()?;
            let t1_ms = parts[3].trim().parse::<u32>().ok()?;
            let t2_ms = parts[4].trim().parse::<u32>().ok()?;
            let t3_ms = parts[5].trim().parse::<u32>().ok()?;
            let t4_ms = parts[6].trim().parse::<u32>().ok()?;

            // Convert milliseconds to centiseconds
            let t1 = t1_ms / 10;
            let t2 = t2_ms / 10;
            let t3 = t3_ms / 10;
            let t4 = t4_ms / 10;

            // Store the ASS alpha values as-is (00=opaque, FF=transparent)
            // They'll be inverted when applied
            return Some(FadeData {
                alpha_start: alpha1,
                alpha_end: alpha3,
                time_start: t1,
                time_end: t4,
                alpha_middle: Some(alpha2),
                time_fade_in: Some(t2 - t1),
                time_fade_out: Some(t4 - t3),
            });
        }
    }

    None
}

impl Default for ProcessedTags {
    fn default() -> Self {
        Self {
            position: None,
            movement: None,
            origin: None,
            colors: ColorOverrides::default(),
            font: FontOverrides::default(),
            formatting: FormattingOverrides::default(),
            transforms: Vec::new(),
            drawing_mode: None,
            clip: None,
            fade: None,
            karaoke: None,
            reset: None,
            baseline_offset: None,
            shear_x: None,
            shear_y: None,
            line_breaks: Vec::new(),
            nbsp_positions: Vec::new(),
        }
    }
}
