//! Fixed software pipeline implementation with proper style resolution

// Debug macro that only works with std
#[cfg(not(feature = "nostd"))]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        eprintln!($($arg)*)
    };
}

#[cfg(feature = "nostd")]
macro_rules! debug_println {
    ($($arg:tt)*) => {};
}

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

use crate::pipeline::{
    animation::{calculate_fade_progress, calculate_move_progress},
    drawing::process_drawing_commands,
    shaping::{shape_text_with_style, GlyphRenderer},
    tag_processor::{KaraokeStyle, ProcessedTags},
    text_segmenter::{segment_text_with_tags, TextSegment},
    transform::{interpolate_alpha, interpolate_color, interpolate_f32, AnimatableTag},
    IntermediateLayer, Pipeline, TextData, TextEffect, VectorData,
};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};
use ahash::AHashMap;
#[cfg(feature = "analysis-integration")]
use ass_core::analysis::ScriptAnalysis;
use ass_core::parser::{Event, Script, Style};
use fontdb::Database as FontDatabase;
use smallvec::SmallVec;
use tiny_skia::Transform;

/// Owned style for storing in pipeline
#[derive(Clone)]
struct OwnedStyle {
    #[allow(dead_code)] // Style identifier - stored for completeness
    name: String,
    fontname: String,
    fontsize: String,
    primary_colour: String,
    secondary_colour: String,
    outline_colour: String,
    back_colour: String,
    bold: String,
    italic: String,
    underline: String,
    strikeout: String,
    scale_x: String,
    scale_y: String,
    spacing: String,
    #[allow(dead_code)] // Text rotation angle - stored for completeness
    angle: String,
    #[allow(dead_code)] // Border rendering style - stored for completeness
    border_style: String,
    outline: String,
    shadow: String,
    alignment: String,
    margin_l: String,
    margin_r: String,
    margin_v: String,
    #[allow(dead_code)] // Text encoding specification - stored for completeness
    encoding: String,
}

impl OwnedStyle {
    fn from_style(style: &Style) -> Self {
        Self {
            name: style.name.to_string(),
            fontname: style.fontname.to_string(),
            fontsize: style.fontsize.to_string(),
            primary_colour: style.primary_colour.to_string(),
            secondary_colour: style.secondary_colour.to_string(),
            outline_colour: style.outline_colour.to_string(),
            back_colour: style.back_colour.to_string(),
            bold: style.bold.to_string(),
            italic: style.italic.to_string(),
            underline: style.underline.to_string(),
            strikeout: style.strikeout.to_string(),
            scale_x: style.scale_x.to_string(),
            scale_y: style.scale_y.to_string(),
            spacing: style.spacing.to_string(),
            angle: style.angle.to_string(),
            border_style: style.border_style.to_string(),
            outline: style.outline.to_string(),
            shadow: style.shadow.to_string(),
            alignment: style.alignment.to_string(),
            margin_l: style.margin_l.to_string(),
            margin_r: style.margin_r.to_string(),
            margin_v: style.margin_v.to_string(),
            encoding: style.encoding.to_string(),
        }
    }
}

/// Software rendering pipeline with proper style inheritance
pub struct SoftwarePipeline {
    /// Font database for text rendering
    font_database: FontDatabase,
    /// Glyph renderer
    #[allow(dead_code)] // Glyph rendering component - used in future rendering features
    glyph_renderer: GlyphRenderer,
    /// Collision resolver for subtitle positioning
    collision_resolver: crate::collision::CollisionResolver,
    /// Render cache for performance
    cache: crate::cache::RenderCache,
    /// Current script styles map for quick lookup
    styles_map: AHashMap<String, OwnedStyle>,
    /// Default style for fallback
    default_style: Option<OwnedStyle>,
    /// Script playback resolution from PlayResX/PlayResY
    play_res_x: f32,
    play_res_y: f32,
    /// Script layout resolution from LayoutResX/LayoutResY (if present)
    layout_res_x: Option<f32>,
    layout_res_y: Option<f32>,
    /// Whether to scale border and shadow with video resolution
    scaled_border_and_shadow: bool,
    /// DPI scale factor for font rendering (default: 0.9)
    /// libass uses 72 DPI, some systems use 96 DPI
    /// Empirically adjusted to 0.9 for better libass visual match
    dpi_scale: f32,
}

impl Default for SoftwarePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl SoftwarePipeline {
    /// Create a new fixed software pipeline
    pub fn new() -> Self {
        let mut font_database = FontDatabase::new();
        font_database.load_system_fonts();

        Self {
            font_database,
            glyph_renderer: GlyphRenderer::new(),
            collision_resolver: crate::collision::CollisionResolver::new(1920.0, 1080.0),
            cache: crate::cache::RenderCache::with_limits(5000, 2000),
            styles_map: AHashMap::new(),
            default_style: None,
            play_res_x: 1920.0, // Default resolution
            play_res_y: 1080.0, // Default resolution
            layout_res_x: None,
            layout_res_y: None,
            scaled_border_and_shadow: true, // Default to true per ASS spec
            dpi_scale: 0.9,                 // Adjusted for better libass compatibility (was 0.75)
        }
    }

    /// Apply transform animations to tags based on current time
    fn apply_transform_animations(
        &self,
        tags: &mut ProcessedTags,
        event_start_cs: u32,
        current_time_cs: u32,
        default_colors: ([u8; 4], [u8; 4], [u8; 4], [u8; 4]), // primary, secondary, outline, shadow
    ) {
        // Process all transforms (can have multiple)
        for transform_data in &tags.transforms {
            let animation = &transform_data.animation;

            // Calculate time relative to event start (convert to milliseconds for animation)
            let relative_time_ms = if current_time_cs >= event_start_cs {
                (current_time_cs - event_start_cs) * 10 // Convert centiseconds to milliseconds
            } else {
                0
            };

            // Calculate animation progress (expects milliseconds)
            let progress = animation.calculate_progress(relative_time_ms);

            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            debug_println!("TRANSFORM: event_start_cs={}, current_time_cs={}, relative_time_ms={}, progress={}", 
                event_start_cs, current_time_cs, relative_time_ms, progress);

            // If animation hasn't started or has finished, we might not need to interpolate
            if progress <= 0.0 || progress >= 1.0 {
                if progress >= 1.0 {
                    // Apply final values
                    for target in &animation.target_tags {
                        match target {
                            AnimatableTag::FontSize(size) => tags.font.size = Some(*size),
                            AnimatableTag::FontScaleX(scale) => tags.font.scale_x = Some(*scale),
                            AnimatableTag::FontScaleY(scale) => tags.font.scale_y = Some(*scale),
                            AnimatableTag::FontSpacing(spacing) => {
                                tags.font.spacing = Some(*spacing)
                            }
                            AnimatableTag::FontRotationZ(angle) => {
                                tags.font.rotation_z = Some(*angle)
                            }
                            AnimatableTag::FontRotationX(angle) => {
                                tags.font.rotation_x = Some(*angle)
                            }
                            AnimatableTag::FontRotationY(angle) => {
                                tags.font.rotation_y = Some(*angle)
                            }
                            AnimatableTag::PrimaryColor(color) => {
                                tags.colors.primary = Some(*color)
                            }
                            AnimatableTag::SecondaryColor(color) => {
                                tags.colors.secondary = Some(*color)
                            }
                            AnimatableTag::OutlineColor(color) => {
                                tags.colors.outline = Some(*color)
                            }
                            AnimatableTag::ShadowColor(color) => tags.colors.shadow = Some(*color),
                            AnimatableTag::Alpha(alpha) => tags.colors.alpha = Some(*alpha),
                            AnimatableTag::BorderWidth(width) => {
                                tags.formatting.border = Some(*width)
                            }
                            AnimatableTag::ShadowDepth(depth) => {
                                tags.formatting.shadow = Some(*depth)
                            }
                            AnimatableTag::Blur(blur) => tags.formatting.blur = Some(*blur),
                        }
                    }
                }
                // Don't return early - we still need to process the rest of the code
                // to apply effects like rotation
            } else if progress > 0.0 {
                // Interpolate values
                for target in &animation.target_tags {
                    match target {
                        AnimatableTag::FontSize(target_size) => {
                            if let Some(current) = tags.font.size {
                                tags.font.size =
                                    Some(interpolate_f32(current, *target_size, progress));
                            } else {
                                tags.font.size = Some(*target_size * progress);
                            }
                        }
                        AnimatableTag::FontScaleX(target_scale) => {
                            let current = tags.font.scale_x.unwrap_or(100.0);
                            tags.font.scale_x =
                                Some(interpolate_f32(current, *target_scale, progress));
                        }
                        AnimatableTag::FontScaleY(target_scale) => {
                            let current = tags.font.scale_y.unwrap_or(100.0);
                            tags.font.scale_y =
                                Some(interpolate_f32(current, *target_scale, progress));
                        }
                        AnimatableTag::FontSpacing(target_spacing) => {
                            let current = tags.font.spacing.unwrap_or(0.0);
                            tags.font.spacing =
                                Some(interpolate_f32(current, *target_spacing, progress));
                        }
                        AnimatableTag::FontRotationZ(target_angle) => {
                            let current = tags.font.rotation_z.unwrap_or(0.0);
                            tags.font.rotation_z =
                                Some(interpolate_f32(current, *target_angle, progress));
                        }
                        AnimatableTag::FontRotationX(target_angle) => {
                            let current = tags.font.rotation_x.unwrap_or(0.0);
                            tags.font.rotation_x =
                                Some(interpolate_f32(current, *target_angle, progress));
                        }
                        AnimatableTag::FontRotationY(target_angle) => {
                            let current = tags.font.rotation_y.unwrap_or(0.0);
                            tags.font.rotation_y =
                                Some(interpolate_f32(current, *target_angle, progress));
                        }
                        AnimatableTag::PrimaryColor(target_color) => {
                            let current = tags.colors.primary.unwrap_or(default_colors.0);
                            tags.colors.primary =
                                Some(interpolate_color(current, *target_color, progress));
                        }
                        AnimatableTag::SecondaryColor(target_color) => {
                            let current = tags.colors.secondary.unwrap_or(default_colors.1);
                            tags.colors.secondary =
                                Some(interpolate_color(current, *target_color, progress));
                        }
                        AnimatableTag::OutlineColor(target_color) => {
                            let current = tags.colors.outline.unwrap_or(default_colors.2);
                            tags.colors.outline =
                                Some(interpolate_color(current, *target_color, progress));
                        }
                        AnimatableTag::ShadowColor(target_color) => {
                            let current = tags.colors.shadow.unwrap_or(default_colors.3);
                            tags.colors.shadow =
                                Some(interpolate_color(current, *target_color, progress));
                        }
                        AnimatableTag::Alpha(target_alpha) => {
                            let current = tags.colors.alpha.unwrap_or(0);
                            tags.colors.alpha =
                                Some(interpolate_alpha(current, *target_alpha, progress));
                        }
                        AnimatableTag::BorderWidth(target_width) => {
                            let current = tags.formatting.border.unwrap_or(0.0);
                            tags.formatting.border =
                                Some(interpolate_f32(current, *target_width, progress));
                        }
                        AnimatableTag::ShadowDepth(target_depth) => {
                            let current = tags.formatting.shadow.unwrap_or(0.0);
                            tags.formatting.shadow =
                                Some(interpolate_f32(current, *target_depth, progress));
                        }
                        AnimatableTag::Blur(target_blur) => {
                            let current = tags.formatting.blur.unwrap_or(0.0);
                            tags.formatting.blur =
                                Some(interpolate_f32(current, *target_blur, progress));
                        }
                    }
                }
            }
        }
    }

    /// Create with specific dimensions
    pub fn with_dimensions(width: f32, height: f32) -> Self {
        let mut font_database = FontDatabase::new();
        font_database.load_system_fonts();

        Self {
            font_database,
            glyph_renderer: GlyphRenderer::new(),
            collision_resolver: crate::collision::CollisionResolver::new(width, height),
            cache: crate::cache::RenderCache::with_limits(5000, 2000),
            styles_map: AHashMap::new(),
            default_style: None,
            play_res_x: width,  // Use provided dimensions as default
            play_res_y: height, // Use provided dimensions as default
            layout_res_x: None,
            layout_res_y: None,
            scaled_border_and_shadow: true, // Default to true per ASS spec
            dpi_scale: 0.9,                 // Adjusted for better libass compatibility (was 0.75)
        }
    }

    /// Set DPI scale factor (default is 0.9 for libass compatibility)
    /// Use 1.0 for 96 DPI, 0.9 for empirically matched libass rendering
    pub fn set_dpi_scale(&mut self, scale: f32) {
        self.dpi_scale = scale;
    }

    /// Get current DPI scale factor
    pub fn dpi_scale(&self) -> f32 {
        self.dpi_scale
    }

    /// Get style by name from the stored styles
    #[allow(dead_code)] // Utility method for style lookup
    fn get_style(&self, style_name: &str) -> Option<&OwnedStyle> {
        self.styles_map
            .get(style_name)
            .or(self.default_style.as_ref())
    }

    /// Parse color from ASS format
    fn parse_ass_color(color: &str) -> [u8; 4] {
        // ASS colors are in &HAABBGGRR format where:
        // AA = Alpha (00 = opaque, FF = transparent)
        // BB = Blue, GG = Green, RR = Red
        let color_trimmed = color.trim_end_matches('&');
        if let Some(hex) = color_trimmed.strip_prefix("&H") {
            if let Ok(value) = u32::from_str_radix(hex, 16) {
                // Check if this is AABBGGRR (8 hex digits) or BBGGRR (6 hex digits)
                let (alpha, bgr_value) = if hex.len() >= 8 {
                    // Full AABBGGRR format
                    let alpha = ((value >> 24) & 0xFF) as u8;
                    // Convert ASS alpha (00=opaque, FF=transparent) to RGBA (00=transparent, FF=opaque)
                    let rgba_alpha = 255 - alpha;
                    (rgba_alpha, value & 0xFFFFFF)
                } else {
                    // Legacy BBGGRR format without alpha, assume opaque
                    (255u8, value)
                };

                // Extract colors in BGR order
                let r = (bgr_value & 0xFF) as u8; // Last 2 hex digits = Red
                let g = ((bgr_value >> 8) & 0xFF) as u8; // Middle 2 hex digits = Green
                let b = ((bgr_value >> 16) & 0xFF) as u8; // First 2 hex digits = Blue

                // Return in RGBA order for rendering
                return [r, g, b, alpha];
            }
        }
        [255, 255, 255, 255] // Default white, opaque
    }

    /// Parse alpha from ASS format
    #[allow(dead_code)] // Utility for parsing ASS alpha values
    fn parse_ass_alpha(alpha: &str) -> u8 {
        if let Some(hex) = alpha.strip_prefix("&H") {
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                // ASS alpha is inverted (00 = opaque, FF = transparent)
                return 255 - value;
            }
        }
        255 // Default opaque
    }

    fn process_event(
        &mut self,
        event: &Event,
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        debug_println!(
            "PROCESS_EVENT: Processing event with text: '{}'",
            event.text
        );
        // Get text segments with their individual tags
        let segments = segment_text_with_tags(event.text, None)?;

        let segment_count = segments.len();
        debug_println!("PROCESS_EVENT: Got {segment_count} segments");
        for (i, seg) in segments.iter().enumerate() {
            debug_println!(
                "  Segment {}: text='{}', drawing_mode={:?}",
                i,
                seg.text,
                seg.tags.drawing_mode
            );
        }

        if segments.is_empty() {
            return Ok(Vec::new());
        }

        // Check if this is a drawing command
        if let Some(draw_level) = segments[0].tags.drawing_mode {
            debug_println!(
                "DRAWING: Found drawing mode level {} for text: '{}'",
                draw_level,
                segments[0].text.chars().take(50).collect::<String>()
            );

            if draw_level > 0 {
                // Clone the style to avoid borrow issues
                let style_cloned = self
                    .styles_map
                    .get(event.style)
                    .or(self.default_style.as_ref())
                    .cloned();

                debug_println!("DRAWING: Processing drawing command");

                return self.process_drawing_command(
                    &segments[0],
                    event,
                    style_cloned.as_ref(),
                    time_cs,
                    context,
                );
            }
        }

        // Clone the style to avoid borrow issues
        let style_cloned = self
            .styles_map
            .get(event.style)
            .or(self.default_style.as_ref())
            .cloned();

        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        if event.text.contains("Чысценькая") {
            debug_println!(
                "  Style '{}' found: {}",
                event.style,
                style_cloned.is_some()
            );
            if let Some(ref style) = style_cloned {
                debug_println!("    Font: {}, Size: {}", style.fontname, style.fontsize);
            }
        }

        // Process text segments with proper style inheritance
        self.process_text_segments(segments, event, style_cloned.as_ref(), time_cs, context)
    }

    fn process_drawing_command(
        &mut self,
        segment: &TextSegment,
        _event: &Event,
        style: Option<&OwnedStyle>,
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        let plain_text = &segment.text;
        let tags = &segment.tags;

        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        debug_println!(
            "DRAWING: process_drawing_command called with text: '{}'",
            plain_text
        );

        // Try to get drawing path from cache
        let draw_cache_key = crate::cache::DrawingCacheKey {
            commands: plain_text.clone(),
        };

        let path_opt = if let Some(cached) = self.cache.get_drawing_path(&draw_cache_key) {
            debug_println!("DRAWING: Got cached path");
            cached
        } else {
            let path = process_drawing_commands(plain_text)?;
            debug_println!(
                "DRAWING: Parsed drawing commands, got path: {}",
                path.is_some()
            );
            self.cache.store_drawing_path(draw_cache_key, path.clone());
            path
        };

        if let Some(path) = path_opt {
            debug_println!("DRAWING: Creating VectorData layer");
            // Get color from tags or style
            let color = if let Some(c) = tags.colors.primary {
                c
            } else if let Some(s) = style {
                Self::parse_ass_color(&s.primary_colour)
            } else {
                [255, 255, 255, 255]
            };

            // Calculate scaling factors
            let scale_x = context.width() as f32 / self.play_res_x;
            let scale_y = context.height() as f32 / self.play_res_y;

            // Calculate position with proper scaling
            let (x, y) = if let Some((px, py)) = tags.position {
                // Scale from script coordinates to render coordinates
                (px * scale_x, py * scale_y)
            } else if let Some((x1, y1, x2, y2, t1, t2)) = tags.movement {
                // Movement times are relative to event start
                let event_start_cs = _event.start_time_cs().unwrap_or(0);
                let event_end_cs = _event.end_time_cs().unwrap_or(u32::MAX);

                // If t1 and t2 are both 0, movement spans the entire event duration
                let (move_start_cs, move_end_cs) = if t1 == 0 && t2 == 0 {
                    (event_start_cs, event_end_cs)
                } else {
                    (event_start_cs + t1, event_start_cs + t2)
                };

                let progress = calculate_move_progress(time_cs, move_start_cs, move_end_cs);
                let x = x1 + (x2 - x1) * progress;
                let y = y1 + (y2 - y1) * progress;
                // Scale from script coordinates to render coordinates
                (x * scale_x, y * scale_y)
            } else {
                // Default center (in render coordinates)
                (context.width() as f32 / 2.0, context.height() as f32 / 2.0)
            };

            // Get path bounds to calculate proper alignment offset
            let bounds = path.bounds();

            // Get alignment from tags or style (default to 5 = center)
            let alignment = tags
                .formatting
                .alignment
                .map(|a| a as i32)
                .or(style.and_then(|s| {
                    // Parse alignment from style - it's stored as a string like "5"
                    s.alignment.parse::<i32>().ok()
                }))
                .unwrap_or(5);

            // Calculate alignment offset based on path bounds
            // Alignment uses numpad layout: 1-3 bottom, 4-6 middle, 7-9 top
            let (align_x_offset, align_y_offset) = {
                // For alignment, we need to position the bounding box
                // The \pos coordinate should be where the alignment point ends up

                // Horizontal alignment: 1,4,7 = left, 2,5,8 = center, 3,6,9 = right
                let x_offset = match alignment % 3 {
                    1 => -bounds.left(), // Left align: move left edge to pos
                    2 => -(bounds.left() + bounds.right()) / 2.0, // Center align: move center to pos
                    0 => -bounds.right(), // Right align: move right edge to pos
                    _ => -(bounds.left() + bounds.right()) / 2.0, // Default center
                };

                // Vertical alignment: 1,2,3 = bottom, 4,5,6 = middle, 7,8,9 = top
                let y_offset = match alignment {
                    1..=3 => -bounds.bottom(), // Bottom align: move bottom edge to pos
                    4..=6 => -(bounds.top() + bounds.bottom()) / 2.0, // Middle align: move center to pos
                    7..=9 => -bounds.top(), // Top align: move top edge to pos
                    _ => -(bounds.top() + bounds.bottom()) / 2.0, // Default middle
                };

                (x_offset, y_offset)
            };

            // Apply transform to path with alignment offset
            let transformed_path = path.transform(Transform::from_translate(
                x + align_x_offset,
                y + align_y_offset,
            ));

            debug_println!(
                "DRAWING: Alignment={}, offset=({:.2}, {:.2}), final pos=({:.2}, {:.2})",
                alignment,
                align_x_offset,
                align_y_offset,
                x + align_x_offset,
                y + align_y_offset
            );
            debug_println!(
                "DRAWING: Created transformed path, returning VectorData with color {:?}",
                color
            );

            return Ok(vec![IntermediateLayer::Vector(VectorData {
                path: transformed_path,
                color,
                stroke: None,
                bounds: None,
            })]);
        }

        Ok(Vec::new())
    }

    fn process_text_segments(
        &mut self,
        segments: Vec<TextSegment>,
        event: &Event,
        style: Option<&OwnedStyle>,
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        if event.text.contains("Чысценькая") {
            let segment_count = segments.len();
            debug_println!("  process_text_segments: {segment_count} segments");
            for (i, seg) in segments.iter().enumerate() {
                debug_println!(
                    "    Segment {}: text='{}', tags={:?}",
                    i,
                    seg.text,
                    seg.tags.position
                );
            }
        }

        let mut all_layers = Vec::new();
        let mut line_height = 48.0; // Default from styles

        // Calculate scaling factors for sizes
        let scale_x = context.width() as f32 / self.play_res_x;
        let scale_y = context.height() as f32 / self.play_res_y;

        // Get base style properties
        let default_font_name = style.map(|s| s.fontname.as_ref()).unwrap_or("Arial");
        let default_font_size_base = style
            .map(|s| s.fontsize.parse::<f32>().unwrap_or(48.0))
            .unwrap_or(48.0);
        // Font sizes in ASS are already in the script resolution coordinate system
        // They need to be scaled according to the PlayResY to output resolution ratio
        // This matches libass behavior
        // Also apply DPI scale to match libass (72 DPI vs 96 DPI)
        let _default_font_size = default_font_size_base * scale_y * self.dpi_scale;
        // In ASS format: -1 = true (bold/italic), 0 = false
        let default_bold = style.map(|s| s.bold == "-1").unwrap_or(false);
        let default_italic = style.map(|s| s.italic == "-1").unwrap_or(false);
        let default_underline = style.map(|s| s.underline == "-1").unwrap_or(false);
        let default_strikeout = style.map(|s| s.strikeout == "-1").unwrap_or(false);

        // Parse style colors
        let default_primary_color = style
            .map(|s| Self::parse_ass_color(&s.primary_colour))
            .unwrap_or([255, 255, 255, 255]);
        let default_secondary_color = style
            .map(|s| Self::parse_ass_color(&s.secondary_colour))
            .unwrap_or([255, 0, 0, 255]);
        let default_outline_color = style
            .map(|s| Self::parse_ass_color(&s.outline_colour))
            .unwrap_or([0, 0, 0, 255]);
        let default_back_color = style
            .map(|s| Self::parse_ass_color(&s.back_colour))
            .unwrap_or([0, 0, 0, 128]);

        // Parse style formatting values and scale them only if ScaledBorderAndShadow is enabled
        let default_outline_base = style
            .map(|s| s.outline.parse::<f32>().unwrap_or(2.0))
            .unwrap_or(2.0);
        let default_shadow_base = style
            .map(|s| s.shadow.parse::<f32>().unwrap_or(2.0))
            .unwrap_or(2.0);
        // Scale outline and shadow from script coordinates only if ScaledBorderAndShadow is true
        let default_outline = if self.scaled_border_and_shadow {
            default_outline_base * scale_y
        } else {
            default_outline_base
        };
        let default_shadow = if self.scaled_border_and_shadow {
            default_shadow_base * scale_y
        } else {
            default_shadow_base
        };
        let default_scale_x = style
            .map(|s| s.scale_x.parse::<f32>().unwrap_or(100.0))
            .unwrap_or(100.0);
        let default_scale_y = style
            .map(|s| s.scale_y.parse::<f32>().unwrap_or(100.0))
            .unwrap_or(100.0);
        let default_spacing = style
            .map(|s| s.spacing.parse::<f32>().unwrap_or(0.0))
            .unwrap_or(0.0);
        let default_alignment = style
            .map(|s| s.alignment.parse::<u8>().unwrap_or(2))
            .unwrap_or(2);

        // Get base position from first segment - we'll adjust per-segment as needed
        let _base_tags = &segments[0].tags;

        // Restructure segments into logical lines
        let mut logical_lines: Vec<Vec<TextSegment>> = Vec::new();
        let mut current_line_segments: Vec<TextSegment> = Vec::new();

        for segment in segments {
            if segment.text.is_empty() {
                continue;
            }

            // Check for newlines
            if segment.text.contains('\n') {
                let parts: Vec<&str> = segment.text.split('\n').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 && !current_line_segments.is_empty() {
                        logical_lines.push(current_line_segments.clone());
                        current_line_segments.clear();
                    }

                    if !part.is_empty() {
                        let sub_segment = TextSegment {
                            text: part.to_string(),
                            start: segment.start,
                            end: segment.end,
                            tags: segment.tags.clone(),
                        };
                        current_line_segments.push(sub_segment);
                    }
                }
            } else {
                current_line_segments.push(segment);
            }
        }

        // Add the last line
        if !current_line_segments.is_empty() {
            logical_lines.push(current_line_segments);
        }

        // Calculate total height for multi-line text
        let num_lines = logical_lines.len();
        // For line height in libass:
        // - Line spacing uses the font size in script resolution scaled to render resolution
        // - DPI scaling affects glyph rendering but NOT line spacing
        // - ScaleY affects glyph size but NOT line spacing
        let line_spacing_multiplier = 1.0; // Match libass line spacing

        // Line height for spacing between lines (not affected by DPI scale in libass)
        let estimated_line_height = default_font_size_base * scale_y;

        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        debug_println!("Line height calculation: font_size_base={}, scale_y={}, dpi_scale={}, estimated_line_height={}", 
            default_font_size_base, scale_y, self.dpi_scale, estimated_line_height);
        let _total_text_height = estimated_line_height * num_lines as f32 * line_spacing_multiplier;

        // Process each logical line
        let mut line_y_offset = 0.0;

        for (line_index, line_segments) in logical_lines.into_iter().enumerate() {
            let mut current_x = 0.0;
            let mut needs_initial_position = true;
            let mut karaoke_accumulated_time = 0u32; // Track cumulative karaoke time for this line

            for segment in line_segments {
                let mut tags = segment.tags.clone();
                let line_text = &segment.text;

                // Debug: Check what's in tags
                if line_text.contains("m ") || line_text.contains("l ") {
                    debug_println!(
                        "DRAWING DEBUG: Found potential drawing text: '{}', drawing_mode: {:?}",
                        line_text,
                        tags.drawing_mode
                    );
                }

                // Check if this segment is in drawing mode
                if let Some(drawing_mode) = tags.drawing_mode {
                    if drawing_mode > 0 {
                        debug_println!(
                            "DRAWING: Processing drawing segment with mode {} and commands: {}",
                            drawing_mode,
                            line_text
                        );

                        // Process drawing commands
                        if let Ok(drawing_layers) =
                            self.process_drawing_command(&segment, event, style, time_cs, context)
                        {
                            for layer in drawing_layers {
                                all_layers.push(layer);
                            }
                        }
                        continue; // Skip text processing for drawing segments
                    }
                }

                // Apply transform animations if present
                let event_start_cs = event.start_time_cs().unwrap_or(0);
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                if !tags.transforms.is_empty() {
                    debug_println!(
                        "TRANSFORM: Found {} transform(s) for text segment at time {}cs",
                        tags.transforms.len(),
                        time_cs
                    );
                }
                let default_colors = (
                    default_primary_color,
                    default_secondary_color,
                    default_outline_color,
                    default_back_color,
                );
                self.apply_transform_animations(&mut tags, event_start_cs, time_cs, default_colors);

                // Shape the text first to get dimensions for proper alignment
                // Get base font size and scale factors
                let base_font_size = tags.font.size.unwrap_or(default_font_size_base); // Use unscaled base size
                let font_scale_x = tags.font.scale_x.unwrap_or(default_scale_x) / 100.0;
                let font_scale_y = tags.font.scale_y.unwrap_or(default_scale_y) / 100.0;
                // Apply both resolution scaling, percentage scaling, and DPI scaling
                let actual_font_size = base_font_size * scale_y * font_scale_y * self.dpi_scale;

                let font_to_use = tags.font.name.as_deref().unwrap_or(default_font_name);

                let shaped = shape_text_with_style(
                    line_text,
                    font_to_use,
                    actual_font_size,
                    tags.formatting.bold.unwrap_or(default_bold),
                    tags.formatting.italic.unwrap_or(default_italic),
                    &self.font_database,
                )?;

                // Calculate position for this segment
                let (segment_x, segment_y) = if tags.position.is_some() || tags.movement.is_some() {
                    // Explicit position override - use it directly
                    let (anchor_x, anchor_y) = self.calculate_position_from_tags(
                        &tags,
                        event,
                        context,
                        time_cs,
                        default_alignment,
                    );

                    // Apply alignment-based offset to get top-left corner
                    let alignment = tags.formatting.alignment.unwrap_or(default_alignment);
                    needs_initial_position = false;
                    let (x, y) = self.apply_alignment_offset(
                        anchor_x,
                        anchor_y,
                        shaped.width,
                        shaped.height,
                        alignment,
                    );

                    (x, y)
                } else if needs_initial_position || tags.formatting.alignment.is_some() {
                    // Need to calculate initial position or alignment changed
                    let alignment = tags.formatting.alignment.unwrap_or(default_alignment);
                    let (anchor_x, anchor_y) =
                        self.calculate_position_from_alignment(alignment, event, context);

                    // For multi-line text, adjust positioning based on alignment
                    let adjusted_y = if num_lines > 1 {
                        match alignment {
                            1..=3 => {
                                // Bottom alignment: stack lines upward from bottom
                                // First line is at the bottom, subsequent lines above it
                                let line_offset = (num_lines - 1 - line_index) as f32
                                    * estimated_line_height
                                    * line_spacing_multiplier;
                                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                                debug_println!("Multi-line bottom align: line {} of {}, anchor_y={}, line_offset={}, estimated_line_height={}", 
                                    line_index + 1, num_lines, anchor_y, line_offset, estimated_line_height);
                                anchor_y - line_offset
                            }
                            7..=9 => {
                                // Top alignment: stack lines downward from top
                                anchor_y + line_y_offset
                            }
                            _ => {
                                // Middle alignment: center the block vertically
                                let block_offset = -((num_lines as f32 - 1.0)
                                    * estimated_line_height
                                    * line_spacing_multiplier
                                    / 2.0);
                                anchor_y + block_offset + line_y_offset
                            }
                        }
                    } else {
                        anchor_y
                    };

                    // For multi-line text, use the line height for vertical positioning
                    // to ensure proper spacing between lines
                    let height_for_positioning = if num_lines > 1 {
                        estimated_line_height
                    } else {
                        shaped.height
                    };

                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!("Calling apply_alignment_offset: adjusted_y={}, height_for_positioning={} (shaped.height={})", 
                        adjusted_y, height_for_positioning, shaped.height);

                    let (x, y) = self.apply_alignment_offset(
                        anchor_x,
                        adjusted_y,
                        shaped.width,
                        height_for_positioning,
                        alignment,
                    );

                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!("Result from apply_alignment_offset: y={}", y);
                    current_x = x + shaped.width; // Set position for next segment
                    needs_initial_position = false;
                    (x, y)
                } else {
                    // Continue from previous segment position
                    // Need to maintain the absolute Y position from the previous segment
                    // For now, calculate it based on alignment and line offset
                    let alignment = tags.formatting.alignment.unwrap_or(default_alignment);
                    let (anchor_x, anchor_y) =
                        self.calculate_position_from_alignment(alignment, event, context);

                    let adjusted_y = if num_lines > 1 {
                        match alignment {
                            1..=3 => {
                                // Bottom alignment: stack lines upward from bottom
                                let line_offset = (num_lines - 1 - line_index) as f32
                                    * estimated_line_height
                                    * line_spacing_multiplier;
                                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                                debug_println!("Multi-line bottom align (else branch): line {} of {}, anchor_y={}, line_offset={}, estimated_line_height={}", 
                                    line_index + 1, num_lines, anchor_y, line_offset, estimated_line_height);
                                anchor_y - line_offset
                            }
                            7..=9 => {
                                // Top alignment: stack lines downward from top
                                anchor_y + line_y_offset
                            }
                            _ => {
                                // Middle alignment: center the block vertically
                                let block_offset = -((num_lines as f32 - 1.0)
                                    * estimated_line_height
                                    * line_spacing_multiplier
                                    / 2.0);
                                anchor_y + block_offset + line_y_offset
                            }
                        }
                    } else {
                        anchor_y
                    };

                    // For multi-line text, use the line height for vertical positioning
                    let height_for_positioning = if num_lines > 1 {
                        estimated_line_height
                    } else {
                        shaped.height
                    };

                    let (_, y) = self.apply_alignment_offset(
                        anchor_x,
                        adjusted_y,
                        shaped.width,
                        height_for_positioning,
                        alignment,
                    );
                    (current_x, y)
                };

                // Get font information with proper inheritance
                let font_family = tags.font.name.as_deref().unwrap_or(default_font_name);
                // Font size was already calculated above for shaping
                // Use the same value here for consistency
                let font_size = actual_font_size;
                line_height = font_size;

                // Get formatting with inheritance from style
                let bold = tags.formatting.bold.unwrap_or(default_bold);
                let italic = tags.formatting.italic.unwrap_or(default_italic);
                let underline = tags.formatting.underline.unwrap_or(default_underline);
                let strikeout = tags.formatting.strikeout.unwrap_or(default_strikeout);

                // Note: 'shaped' was already created above for alignment calculation

                // Get colors with proper inheritance
                let mut color = tags.colors.primary.unwrap_or(default_primary_color);
                let mut outline_color = tags.colors.outline.unwrap_or(default_outline_color);
                let mut shadow_color = tags.colors.shadow.unwrap_or(default_back_color);

                // Debug: Print color being used for text
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                if !line_text.is_empty() {
                    debug_println!(
                        "Text '{}' using color: R={}, G={}, B={}, A={}",
                        line_text,
                        color[0],
                        color[1],
                        color[2],
                        color[3]
                    );
                    if let Some(s) = style {
                        let primary_colour = &s.primary_colour;
                        debug_println!("  Style primary_colour: {primary_colour}");
                    }
                }

                // Apply individual alpha overrides (ASS alpha is inverted: 00=opaque, FF=transparent)
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                debug_println!(
                    "Alpha values: alpha={:?}, alpha1={:?}, alpha3={:?}, alpha4={:?}",
                    tags.colors.alpha,
                    tags.colors.alpha1,
                    tags.colors.alpha3,
                    tags.colors.alpha4
                );

                if let Some(alpha) = tags.colors.alpha1.or(tags.colors.alpha) {
                    // Alpha is already inverted in parse_alpha (255 = opaque, 0 = transparent)
                    color[3] = alpha;
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!("Applied alpha to primary color: {alpha}");
                }
                if let Some(alpha) = tags.colors.alpha3 {
                    // Alpha is already inverted in parse_alpha
                    outline_color[3] = alpha;
                }
                if let Some(alpha) = tags.colors.alpha4 {
                    // Alpha is already inverted in parse_alpha
                    shadow_color[3] = alpha;
                }

                // Apply fade effect
                if let Some(fade) = &tags.fade {
                    // For \fad(t1,t2), t1 is fade-in duration, t2 is fade-out duration
                    // Calculate actual fade times relative to event
                    let event_start = event.start_time_cs().unwrap_or(0);
                    let event_end = event.end_time_cs().unwrap_or(u32::MAX);

                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!(
                        "FADE DEBUG: Found fade tag for '{}' at time_cs={}",
                        line_text.chars().take(30).collect::<String>(),
                        time_cs
                    );
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!("  Event times: start={}, end={}", event_start, event_end);
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!("  Fade params: time_start={}, time_end={}, alpha_start={}, alpha_end={}, alpha_middle={:?}", 
                        fade.time_start, fade.time_end, fade.alpha_start, fade.alpha_end, fade.alpha_middle);

                    let fade_alpha = if fade.alpha_middle.is_some() {
                        // Complex fade with 7 parameters - times are absolute
                        let fade_progress =
                            calculate_fade_progress(time_cs, fade.time_start, fade.time_end);
                        // Interpolate between ASS alpha values (00=opaque, FF=transparent)
                        let ass_alpha = fade.alpha_start as f32
                            + (fade.alpha_end as f32 - fade.alpha_start as f32) * fade_progress;
                        // Convert to RGBA alpha (00=transparent, FF=opaque)
                        let result = 255.0 - ass_alpha;
                        #[cfg(all(debug_assertions, not(feature = "nostd")))]
                        debug_println!(
                            "  Complex fade: progress={:.2}, ass_alpha={:.1}, rgba_alpha={:.1}",
                            fade_progress,
                            ass_alpha,
                            result
                        );
                        result
                    } else {
                        // Simple fade - times are durations
                        let fade_in_end = event_start + fade.time_start;
                        let fade_out_start = event_end.saturating_sub(fade.time_end);

                        #[cfg(all(debug_assertions, not(feature = "nostd")))]
                        debug_println!(
                            "  Simple fade: fade_in_end={}, fade_out_start={}",
                            fade_in_end,
                            fade_out_start
                        );

                        if time_cs < fade_in_end {
                            // During fade in
                            let progress = (time_cs.saturating_sub(event_start)) as f32
                                / fade.time_start.max(1) as f32;
                            let alpha = 255.0 * progress.min(1.0);
                            #[cfg(all(debug_assertions, not(feature = "nostd")))]
                            debug_println!(
                                "  FADE IN: progress={:.2}, alpha={:.1}",
                                progress,
                                alpha
                            );
                            alpha
                        } else if time_cs >= fade_out_start && fade_out_start < event_end {
                            // During fade out
                            let progress = (event_end.saturating_sub(time_cs)) as f32
                                / fade.time_end.max(1) as f32;
                            let alpha = 255.0 * progress.min(1.0);
                            #[cfg(all(debug_assertions, not(feature = "nostd")))]
                            debug_println!(
                                "  FADE OUT: progress={:.2}, alpha={:.1}",
                                progress,
                                alpha
                            );
                            alpha
                        } else {
                            // Fully visible
                            #[cfg(all(debug_assertions, not(feature = "nostd")))]
                            debug_println!("  FULLY VISIBLE: alpha=255.0");
                            255.0
                        }
                    };

                    // Apply fade to all color components (primary, outline, shadow)
                    let fade_factor = fade_alpha / 255.0;

                    let _old_alpha = color[3];
                    color[3] = (color[3] as f32 * fade_factor) as u8;
                    outline_color[3] = (outline_color[3] as f32 * fade_factor) as u8;
                    shadow_color[3] = (shadow_color[3] as f32 * fade_factor) as u8;

                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!("  FADE APPLIED: fade_alpha={:.1}, primary: {}→{}, outline: {}→{}, shadow: {}→{}",
                        fade_alpha, _old_alpha, color[3],
                        (outline_color[3] as f32 / fade_factor) as u8, outline_color[3],
                        (shadow_color[3] as f32 / fade_factor) as u8, shadow_color[3]);
                }

                // Get spacing value (from tags or default style)
                let spacing = tags.font.spacing.unwrap_or(default_spacing);
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                if spacing != 0.0 || tags.font.spacing.is_some() {
                    debug_println!("DEBUG: tags.font.spacing={:?}, default_spacing={}, final spacing={} for text '{}'", 
                        tags.font.spacing, default_spacing, spacing, line_text);
                }

                // Create text layer
                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                if !(-100.0..=1080.0).contains(&segment_y) {
                    debug_println!(
                        "WARNING: Text positioned off-screen at Y={} for text '{}'",
                        segment_y,
                        line_text
                    );
                    debug_println!(
                        "  Font size: {}, ScaleY: {}",
                        font_size,
                        tags.font.scale_y.unwrap_or(default_scale_y)
                    );
                }

                let mut layer = TextData {
                    text: line_text.to_string(),
                    font_family: font_family.to_string(),
                    font_size,
                    color,
                    x: segment_x,
                    y: segment_y,
                    effects: SmallVec::new(),
                    spacing,
                };

                #[cfg(all(debug_assertions, not(feature = "nostd")))]
                if event.text.contains("Чысценькая") {
                    debug_println!("    Layer created: pos=({:.1}, {:.1}), font_size={:.1}, color={:?}, text='{}'", 
                        segment_x, segment_y, font_size, color, line_text);
                }

                // Add effects
                if bold {
                    layer.effects.push(TextEffect::Bold);
                }
                if italic {
                    layer.effects.push(TextEffect::Italic);
                }
                if underline {
                    layer.effects.push(TextEffect::Underline);
                }
                if strikeout {
                    layer.effects.push(TextEffect::Strikethrough);
                }

                // Add outline effect with per-axis border support
                // Scale tag values only if ScaledBorderAndShadow is enabled
                let outline_width_x = if self.scaled_border_and_shadow {
                    tags.formatting
                        .border_x
                        .map(|w| w * scale_x)
                        .or(tags.formatting.border.map(|w| w * scale_y))
                        .unwrap_or(default_outline)
                } else {
                    tags.formatting
                        .border_x
                        .or(tags.formatting.border)
                        .unwrap_or(default_outline)
                };
                let outline_width_y = if self.scaled_border_and_shadow {
                    tags.formatting
                        .border_y
                        .map(|w| w * scale_y)
                        .or(tags.formatting.border.map(|w| w * scale_y))
                        .unwrap_or(default_outline)
                } else {
                    tags.formatting
                        .border_y
                        .or(tags.formatting.border)
                        .unwrap_or(default_outline)
                };
                if outline_width_x > 0.0 || outline_width_y > 0.0 {
                    layer.effects.push(TextEffect::Outline {
                        color: outline_color,
                        width: outline_width_x.max(outline_width_y), // Use max for now
                    });
                }

                // Add shadow effect with per-axis shadow support
                // Scale tag values only if ScaledBorderAndShadow is enabled
                let shadow_x = if self.scaled_border_and_shadow {
                    tags.formatting
                        .shadow_x
                        .map(|s| s * scale_x)
                        .or(tags.formatting.shadow.map(|s| s * scale_y))
                        .unwrap_or(default_shadow)
                } else {
                    tags.formatting
                        .shadow_x
                        .or(tags.formatting.shadow)
                        .unwrap_or(default_shadow)
                };
                let shadow_y = if self.scaled_border_and_shadow {
                    tags.formatting
                        .shadow_y
                        .map(|s| s * scale_y)
                        .or(tags.formatting.shadow.map(|s| s * scale_y))
                        .unwrap_or(default_shadow)
                } else {
                    tags.formatting
                        .shadow_y
                        .or(tags.formatting.shadow)
                        .unwrap_or(default_shadow)
                };
                if shadow_x != 0.0 || shadow_y != 0.0 {
                    layer.effects.push(TextEffect::Shadow {
                        color: shadow_color,
                        x_offset: shadow_x * 0.5, // Further reduce shadow to match libass
                        y_offset: shadow_y * 0.5,
                    });
                }

                // Add blur effect - handle both blur and edge blur
                let blur_radius = tags.formatting.blur.unwrap_or(0.0);
                let edge_blur = tags.formatting.blur_edges.unwrap_or(0.0);
                if blur_radius > 0.0 {
                    layer.effects.push(TextEffect::Blur {
                        radius: blur_radius,
                    });
                }
                if edge_blur > 0.0 {
                    // Edge blur only affects the outline
                    layer
                        .effects
                        .push(TextEffect::EdgeBlur { radius: edge_blur });
                }

                // Add rotation effects if present
                let rotation_x = tags.font.rotation_x.unwrap_or(0.0);
                let rotation_y = tags.font.rotation_y.unwrap_or(0.0);
                let rotation_z = tags.font.rotation_z.or(tags.font.angle).unwrap_or(0.0);
                if rotation_x != 0.0 || rotation_y != 0.0 || rotation_z != 0.0 {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!(
                        "ROTATION: Adding rotation effect - x={}, y={}, z={} for text '{}'",
                        rotation_x,
                        rotation_y,
                        rotation_z,
                        line_text
                    );
                    layer.effects.push(TextEffect::Rotation {
                        x: rotation_x,
                        y: rotation_y,
                        z: rotation_z,
                    });
                }

                // Add shear effects if present
                if let Some(shear_x) = tags.shear_x {
                    if shear_x != 0.0 || tags.shear_y.unwrap_or(0.0) != 0.0 {
                        layer.effects.push(TextEffect::Shear {
                            x: shear_x,
                            y: tags.shear_y.unwrap_or(0.0),
                        });
                    }
                }

                // Add scale effect if X-scale is not 100%
                // Y-scale is already applied to font size during shaping,
                // but X-scale needs to be applied as a transform
                let font_scale_x_val = tags.font.scale_x.unwrap_or(default_scale_x);
                let font_scale_y_val = tags.font.scale_y.unwrap_or(default_scale_y);
                if (font_scale_x_val - 100.0).abs() > 0.01 {
                    // Add scale effect to handle X-scaling
                    layer.effects.push(TextEffect::Scale {
                        x: font_scale_x_val,
                        y: font_scale_y_val,
                    });
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!(
                        "SCALE EFFECT ADDED: x={}, y={} for text '{}'",
                        font_scale_x_val,
                        font_scale_y_val,
                        line_text
                    );
                }

                // Add clip region if present (scale from script coordinates)
                if let Some(clip) = &tags.clip {
                    layer.effects.push(TextEffect::Clip {
                        x1: clip.x1 * scale_x,
                        y1: clip.y1 * scale_y,
                        x2: clip.x2 * scale_x,
                        y2: clip.y2 * scale_y,
                        inverse: clip.inverse,
                    });
                }

                // Handle baseline offset
                if let Some(baseline_offset) = tags.baseline_offset {
                    layer.y += baseline_offset;
                }

                // Handle karaoke - track per-syllable timing
                if let Some(karaoke) = &tags.karaoke {
                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!(
                        "KARAOKE FOUND: duration={}, style={:?}, time_cs={}, event_start={}",
                        karaoke.duration,
                        karaoke.style,
                        time_cs,
                        event.start_time_cs().unwrap_or(0)
                    );

                    // Calculate progress for THIS syllable based on cumulative timing
                    let syllable_start =
                        event.start_time_cs().unwrap_or(0) + karaoke_accumulated_time;
                    let syllable_end = syllable_start + karaoke.duration;

                    let progress = if time_cs < syllable_start {
                        0.0 // Not yet started
                    } else if time_cs >= syllable_end {
                        1.0 // Fully highlighted
                    } else {
                        // In progress
                        (time_cs - syllable_start) as f32 / karaoke.duration as f32
                    };

                    #[cfg(all(debug_assertions, not(feature = "nostd")))]
                    debug_println!("KARAOKE TIMING: syllable_start={}, syllable_end={}, time_cs={}, progress={}", 
                        syllable_start, syllable_end, time_cs, progress);

                    // Add karaoke effect with correct progress
                    layer.effects.push(TextEffect::Karaoke {
                        progress,
                        style: match karaoke.style {
                            KaraokeStyle::Basic => 0,
                            KaraokeStyle::Fill => 1,
                            KaraokeStyle::Outline => 2,
                            KaraokeStyle::Sweep => 3,
                        },
                    });

                    // Accumulate time for next syllable
                    karaoke_accumulated_time += karaoke.duration;
                }

                // Update x position for next segment (only if no explicit positioning was used)
                if tags.position.is_none()
                    && tags.movement.is_none()
                    && tags.formatting.alignment.is_none()
                {
                    if let Some(advance) = shaped.total_advance() {
                        current_x += advance * font_scale_x;
                    }
                } else {
                    current_x += line_text.len() as f32 * font_size * 0.6 * font_scale_x;
                }

                all_layers.push(IntermediateLayer::Text(layer));
            }

            // Move to next line (using same multiplier as above)
            line_y_offset += line_height * line_spacing_multiplier;
        }

        Ok(all_layers)
    }

    fn calculate_position_from_tags(
        &self,
        tags: &ProcessedTags,
        event: &Event,
        context: &RenderContext,
        time_cs: u32,
        default_alignment: u8,
    ) -> (f32, f32) {
        // Calculate scaling factors
        let scale_x = context.width() as f32 / self.play_res_x;
        let scale_y = context.height() as f32 / self.play_res_y;

        // Check for explicit position
        if let Some((mut px, mut py)) = tags.position {
            // If LayoutRes is present, positions are in LayoutRes coordinates
            // and need to be scaled to PlayRes first
            if let (Some(layout_x), Some(layout_y)) = (self.layout_res_x, self.layout_res_y) {
                px *= self.play_res_x / layout_x;
                py *= self.play_res_y / layout_y;
            }
            // Then scale from script (PlayRes) coordinates to render coordinates
            return (px * scale_x, py * scale_y);
        }

        // Check for movement
        if let Some((mut x1, mut y1, mut x2, mut y2, t1, t2)) = tags.movement {
            // If LayoutRes is present, movement coordinates are in LayoutRes coordinates
            if let (Some(layout_x), Some(layout_y)) = (self.layout_res_x, self.layout_res_y) {
                x1 *= self.play_res_x / layout_x;
                y1 *= self.play_res_y / layout_y;
                x2 *= self.play_res_x / layout_x;
                y2 *= self.play_res_y / layout_y;
            }

            // Movement times are relative to event start
            let event_start_cs = event.start_time_cs().unwrap_or(0);
            let event_end_cs = event.end_time_cs().unwrap_or(u32::MAX);

            // t1 and t2 are in milliseconds, need to convert to centiseconds
            let t1_cs = t1 / 10;
            let t2_cs = t2 / 10;

            // If t1 and t2 are both 0, movement spans the entire event duration
            let (move_start_cs, move_end_cs) = if t1 == 0 && t2 == 0 {
                (event_start_cs, event_end_cs)
            } else {
                (event_start_cs + t1_cs, event_start_cs + t2_cs)
            };

            #[cfg(debug_assertions)]
            {
                debug_println!(
                    "MOVE DEBUG: x1={}, y1={}, x2={}, y2={}, t1={}, t2={}",
                    x1,
                    y1,
                    x2,
                    y2,
                    t1,
                    t2
                );
                debug_println!(
                    "  event_start_cs={}, move_start_cs={}, move_end_cs={}, time_cs={}",
                    event_start_cs,
                    move_start_cs,
                    move_end_cs,
                    time_cs
                );
            }

            let progress = calculate_move_progress(time_cs, move_start_cs, move_end_cs);
            let x = x1 + (x2 - x1) * progress;
            let y = y1 + (y2 - y1) * progress;

            #[cfg(debug_assertions)]
            debug_println!(
                "  progress={}, calculated x={}, y={}, scale_x={}, scale_y={}",
                progress,
                x,
                y,
                scale_x,
                scale_y
            );

            // Scale from script (PlayRes) coordinates to render coordinates
            return (x * scale_x, y * scale_y);
        }

        // Calculate based on alignment
        let alignment = tags.formatting.alignment.unwrap_or(default_alignment);
        self.calculate_position_from_alignment(alignment, event, context)
    }

    fn calculate_position_from_alignment(
        &self,
        alignment: u8,
        event: &Event,
        context: &RenderContext,
    ) -> (f32, f32) {
        let width = context.width() as f32;
        let height = context.height() as f32;

        // Calculate scaling factors for margins
        let scale_x = width / self.play_res_x;
        let scale_y = height / self.play_res_y;

        // Parse margins - use style margins if event margins are 0 or empty
        // Get margins in script coordinates first
        let style_margin_l = self
            .styles_map
            .get(event.style)
            .and_then(|s| s.margin_l.parse::<f32>().ok())
            .unwrap_or(0.0);
        let style_margin_r = self
            .styles_map
            .get(event.style)
            .and_then(|s| s.margin_r.parse::<f32>().ok())
            .unwrap_or(0.0);
        let style_margin_v = self
            .styles_map
            .get(event.style)
            .and_then(|s| s.margin_v.parse::<f32>().ok())
            .unwrap_or(0.0);

        // Get margins in script coordinates
        let margin_l_script = if event.margin_l.is_empty() || event.margin_l == "0" {
            style_margin_l
        } else {
            event.margin_l.parse::<f32>().unwrap_or(style_margin_l)
        };
        let margin_r_script = if event.margin_r.is_empty() || event.margin_r == "0" {
            style_margin_r
        } else {
            event.margin_r.parse::<f32>().unwrap_or(style_margin_r)
        };
        let margin_v_script = if event.margin_v.is_empty() || event.margin_v == "0" {
            style_margin_v
        } else {
            event.margin_v.parse::<f32>().unwrap_or(style_margin_v)
        };

        // Scale margins to screen coordinates
        let margin_l = margin_l_script * scale_x;
        let margin_r = margin_r_script * scale_x;
        let _margin_v = margin_v_script * scale_y;

        // ASS alignment uses numpad layout
        // SubStation numpad-style alignment:
        // 7 8 9  (top-left, top-center, top-right)
        // 4 5 6  (middle-left, middle-center, middle-right)
        // 1 2 3  (bottom-left, bottom-center, bottom-right)

        // For legacy alignment (\a tag), map to numpad:
        // 1-3: bottom, 4-6: unused, 7-9: top, 10-12: middle
        let mapped_alignment = if alignment > 9 {
            // Legacy alignment mapping
            match alignment {
                10 => 4, // Left middle
                11 => 5, // Center middle
                12 => 6, // Right middle
                _ => alignment,
            }
        } else {
            alignment
        };

        // Return the anchor point based on alignment and margins
        // This is where the aligned point of the text box should be placed
        // ASS alignment uses numpad layout: 1,4,7 = left; 2,5,8 = center; 3,6,9 = right
        let x = match mapped_alignment {
            1 | 4 | 7 => margin_l,         // Left column
            2 | 5 | 8 => width / 2.0,      // Center column
            3 | 6 | 9 => width - margin_r, // Right column
            _ => width / 2.0,              // Default center
        };

        // Position calculation following libass approach
        // libass uses: (PlayResY - MarginV) * scale_y for bottom alignment
        // Calculate position in script coordinates first, then scale to screen
        let y_script = match mapped_alignment {
            1..=3 => self.play_res_y - margin_v_script, // Bottom row: PlayResY - MarginV
            4..=6 => self.play_res_y / 2.0,             // Middle row: PlayResY / 2
            7..=9 => margin_v_script,                   // Top row: MarginV
            _ => self.play_res_y - margin_v_script,     // Default bottom
        };

        // Transform from script coordinates to screen coordinates
        let y = y_script * scale_y;

        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        debug_println!("calculate_position_from_alignment: alignment={}, margins=(L:{:.1}, R:{:.1}, V:{:.1}), screen={}x{} -> anchor=({:.1}, {:.1})",
            mapped_alignment, margin_l, margin_r, _margin_v, width as u32, height as u32, x, y);

        (x, y)
    }

    /// Apply alignment offset to convert from anchor point to top-left corner
    /// Takes the anchor point and text dimensions, returns top-left corner for rendering
    fn apply_alignment_offset(
        &self,
        anchor_x: f32,
        anchor_y: f32,
        text_width: f32,
        text_height: f32,
        alignment: u8,
    ) -> (f32, f32) {
        // Map legacy alignment if needed
        let mapped_alignment = if alignment > 9 {
            match alignment {
                10 => 4,
                11 => 5,
                12 => 6,
                _ => alignment,
            }
        } else {
            alignment
        };

        // Calculate horizontal offset based on alignment
        // ASS alignment uses numpad layout: 1,4,7 = left; 2,5,8 = center; 3,6,9 = right
        let x = match mapped_alignment {
            1 | 4 | 7 => anchor_x,                    // Left-aligned: anchor is left edge
            2 | 5 | 8 => anchor_x - text_width / 2.0, // Center-aligned: anchor is center
            3 | 6 | 9 => anchor_x - text_width,       // Right-aligned: anchor is right edge
            _ => anchor_x - text_width / 2.0,         // Default center
        };

        // Calculate vertical offset based on alignment
        // For ASS/SSA subtitles, the anchor point represents:
        // - Bottom alignment: where the bottom of the text block should be
        // - Middle alignment: center of text box
        // - Top alignment: top of text box
        // We return the top-left corner position for rendering
        let y = match mapped_alignment {
            1..=3 => {
                // Bottom: anchor is where bottom of text should be
                // Subtract text_height to get top of text box
                // libass uses font metrics for exact positioning, not hardcoded factors
                anchor_y - text_height
            }
            4..=6 => anchor_y - text_height / 2.0, // Middle: anchor is center
            7..=9 => anchor_y,                     // Top: anchor is top edge
            _ => anchor_y - text_height,           // Default bottom
        };

        #[cfg(all(debug_assertions, not(feature = "nostd")))]
        debug_println!("apply_alignment_offset: alignment={}, anchor=({:.1}, {:.1}), size=({:.1}x{:.1}) -> pos=({:.1}, {:.1})",
            mapped_alignment, anchor_x, anchor_y, text_width, text_height, x, y);

        (x, y)
    }
}

#[allow(dead_code)] // Utility for karaoke effects - used in future features
fn calculate_karaoke_progress(time_cs: u32, start_time_cs: u32, duration_cs: u32) -> f32 {
    if time_cs < start_time_cs {
        return 0.0;
    }
    let elapsed = time_cs - start_time_cs;
    if elapsed >= duration_cs {
        return 1.0;
    }
    elapsed as f32 / duration_cs as f32
}

impl Pipeline for SoftwarePipeline {
    fn prepare_script(
        &mut self,
        script: &Script,
        #[cfg(feature = "analysis-integration")] analysis: Option<&ScriptAnalysis>,
        #[cfg(not(feature = "analysis-integration"))] _analysis: Option<()>,
    ) -> Result<(), RenderError> {
        // Load embedded and referenced fonts from the script
        super::font_loader::load_script_fonts(script, &mut self.font_database);

        // Clear and rebuild styles map
        self.styles_map.clear();
        self.default_style = None;

        // If we have analysis with resolved styles (which handle LayoutRes->PlayRes scaling),
        // we should use those instead of raw styles
        #[cfg(feature = "analysis-integration")]
        let _use_resolved_styles = analysis.is_some();
        #[cfg(not(feature = "analysis-integration"))]
        let _use_resolved_styles = false;

        // Extract script info and styles from the script
        for section in script.sections() {
            match section {
                ass_core::parser::Section::ScriptInfo(info) => {
                    // Extract PlayResX and PlayResY from script info
                    if let Some((res_x, res_y)) = info.play_resolution() {
                        self.play_res_x = res_x as f32;
                        self.play_res_y = res_y as f32;
                    }

                    // Extract LayoutResX and LayoutResY if present
                    if let Some((layout_x, layout_y)) = info.layout_resolution() {
                        self.layout_res_x = Some(layout_x as f32);
                        self.layout_res_y = Some(layout_y as f32);

                        // If LayoutRes differs from PlayRes, we need to scale styles
                        // This is done later when processing styles
                        #[cfg(all(debug_assertions, not(feature = "nostd")))]
                        debug_println!(
                            "LayoutRes detected: {}x{}, PlayRes: {}x{}",
                            layout_x,
                            layout_y,
                            self.play_res_x,
                            self.play_res_y
                        );
                    }

                    // Extract ScaledBorderAndShadow setting
                    // Default is "yes" per ASS spec, but can be "no" to disable scaling
                    if let Some(scaled_value) = info.get_field("ScaledBorderAndShadow") {
                        self.scaled_border_and_shadow = scaled_value.to_lowercase() != "no";
                    }
                }
                ass_core::parser::Section::Styles(styles) => {
                    // Calculate LayoutRes->PlayRes scaling factors if LayoutRes is present
                    let layout_to_play_scale_x = if let Some(layout_x) = self.layout_res_x {
                        if layout_x != self.play_res_x {
                            self.play_res_x / layout_x
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };

                    let layout_to_play_scale_y = if let Some(layout_y) = self.layout_res_y {
                        if layout_y != self.play_res_y {
                            self.play_res_y / layout_y
                        } else {
                            1.0
                        }
                    } else {
                        1.0
                    };

                    let needs_layout_scaling =
                        layout_to_play_scale_x != 1.0 || layout_to_play_scale_y != 1.0;

                    for style in styles {
                        let style_name = style.name.to_string();
                        let mut owned_style = OwnedStyle::from_style(style);

                        // Apply LayoutRes->PlayRes scaling if needed
                        if needs_layout_scaling {
                            // Scale font size (using Y scale as per libass)
                            if let Ok(font_size) = owned_style.fontsize.parse::<f32>() {
                                owned_style.fontsize =
                                    (font_size * layout_to_play_scale_y).to_string();
                            }

                            // Scale margins
                            if let Ok(margin_l) = owned_style.margin_l.parse::<f32>() {
                                owned_style.margin_l =
                                    (margin_l * layout_to_play_scale_x).to_string();
                            }
                            if let Ok(margin_r) = owned_style.margin_r.parse::<f32>() {
                                owned_style.margin_r =
                                    (margin_r * layout_to_play_scale_x).to_string();
                            }
                            if let Ok(margin_v) = owned_style.margin_v.parse::<f32>() {
                                owned_style.margin_v =
                                    (margin_v * layout_to_play_scale_y).to_string();
                            }

                            // Scale outline and shadow if ScaledBorderAndShadow is enabled
                            if self.scaled_border_and_shadow {
                                if let Ok(outline) = owned_style.outline.parse::<f32>() {
                                    owned_style.outline =
                                        (outline * layout_to_play_scale_y).to_string();
                                }
                                if let Ok(shadow) = owned_style.shadow.parse::<f32>() {
                                    owned_style.shadow =
                                        (shadow * layout_to_play_scale_y).to_string();
                                }
                            }

                            // Scale spacing
                            if let Ok(spacing) = owned_style.spacing.parse::<f32>() {
                                owned_style.spacing =
                                    (spacing * layout_to_play_scale_x).to_string();
                            }

                            #[cfg(all(debug_assertions, not(feature = "nostd")))]
                            debug_println!(
                                "Scaled style '{}' from LayoutRes to PlayRes",
                                style_name
                            );
                        }

                        if style_name == "Default" || style_name == "*Default" {
                            self.default_style = Some(owned_style.clone());
                        }

                        self.styles_map.insert(style_name, owned_style);
                    }
                }
                _ => {}
            }
        }

        // If no default style found, use the first one
        if self.default_style.is_none() && !self.styles_map.is_empty() {
            self.default_style = self.styles_map.values().next().cloned();
        }

        Ok(())
    }

    fn script(&self) -> Option<&Script<'_>> {
        None // We don't store the script reference directly
    }

    fn process_events(
        &mut self,
        events: &[&Event],
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        // Clear collision resolver for this frame (but keep dimensions)
        self.collision_resolver.clear();

        // Pre-allocate with estimated capacity to reduce allocations
        let mut all_layers = Vec::with_capacity(events.len() * 3);

        // Sort events first by layer, then by start time to ensure proper ordering
        let mut sorted_events = events.to_vec();
        sorted_events.sort_by(|a, b| {
            let layer_a = a.layer.parse::<i32>().unwrap_or(0);
            let layer_b = b.layer.parse::<i32>().unwrap_or(0);
            let start_a = a.start_time_cs().unwrap_or(0);
            let start_b = b.start_time_cs().unwrap_or(0);

            // Sort by layer first, then by start time
            layer_a.cmp(&layer_b).then(start_a.cmp(&start_b))
        });

        // Process each event
        for event in sorted_events {
            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            if event.text.contains("Чысценькая") {
                debug_println!(
                    "Processing target event: style={}, text_len={}",
                    event.style,
                    event.text.len()
                );
            }

            let event_layers = self.process_event(event, time_cs, context)?;

            #[cfg(all(debug_assertions, not(feature = "nostd")))]
            if event.text.contains("Чысценькая") {
                debug_println!("  Generated {} layers for target event", event_layers.len());
            }

            all_layers.extend(event_layers);
        }

        Ok(all_layers)
    }

    fn compute_dirty_regions(
        &self,
        events: &[&Event],
        time_cs: u32,
        prev_time_cs: u32,
    ) -> Result<Vec<DirtyRegion>, RenderError> {
        let mut regions = Vec::new();

        for event in events {
            let was_active = event.start_time_cs().unwrap_or(0) <= prev_time_cs
                && event.end_time_cs().unwrap_or(u32::MAX) > prev_time_cs;
            let is_active = event.start_time_cs().unwrap_or(0) <= time_cs
                && event.end_time_cs().unwrap_or(u32::MAX) > time_cs;

            if was_active != is_active {
                // Event visibility changed, mark entire screen as dirty for now
                regions.push(DirtyRegion::full_screen());
                break;
            }
        }

        Ok(regions)
    }
}
