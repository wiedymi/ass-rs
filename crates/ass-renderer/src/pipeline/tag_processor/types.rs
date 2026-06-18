//! Processed tag data structures used during rendering

use crate::pipeline::transform::TransformAnimation;
#[cfg(feature = "nostd")]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

/// Processed tag values ready for rendering
#[derive(Debug, Clone, Default)]
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
/// Color override settings for rendering
pub struct ColorOverrides {
    /// Primary color override
    pub primary: Option<[u8; 4]>,
    /// Secondary color override
    pub secondary: Option<[u8; 4]>,
    /// Outline color override
    pub outline: Option<[u8; 4]>,
    /// Shadow color override
    pub shadow: Option<[u8; 4]>,
    /// General alpha override
    pub alpha: Option<u8>,
    /// Primary color alpha override (\1a)
    pub alpha1: Option<u8>,
    /// Secondary color alpha override (\2a)
    pub alpha2: Option<u8>,
    /// Outline color alpha override (\3a)
    pub alpha3: Option<u8>,
    /// Shadow color alpha override (\4a)
    pub alpha4: Option<u8>,
}

#[derive(Debug, Clone, Default)]
/// Font override settings
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
/// Text formatting override settings
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
/// Karaoke timing and style data
pub struct KaraokeData {
    /// Duration of karaoke effect in centiseconds
    pub duration: u32,
    /// Style of karaoke effect
    pub style: KaraokeStyle,
    /// Karaoke syllable start time in centiseconds (for \kt)
    pub start_time: Option<u32>,
}

#[derive(Debug, Clone)]
/// Karaoke effect styles
pub enum KaraokeStyle {
    /// Basic karaoke effect
    Basic,
    /// Fill-based karaoke effect
    Fill,
    /// Outline-based karaoke effect
    Outline,
    /// Sweep karaoke effect (\K capital K)
    Sweep,
}

#[derive(Debug, Clone)]
/// Line break types in ASS text
pub enum LineBreakType {
    /// Hard line break (\N) at position in text
    Hard(usize),
    /// Soft line break (\n) at position in text
    Soft(usize),
}
