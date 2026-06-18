//! Fixed software pipeline implementation with proper style resolution

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

use crate::collision::PositionedEvent;
use crate::pipeline::{
    shaping::{shape_text_cached, GlyphRenderer},
    tag_processor::KaraokeStyle,
    text_segmenter::{segment_text_with_tags, TextSegment},
    IntermediateLayer, Pipeline, TextData, TextEffect,
};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};
use ahash::AHashMap;
#[cfg(feature = "analysis-integration")]
use ass_core::analysis::ScriptAnalysis;
use ass_core::parser::{Event, Script};
use fontdb::Database as FontDatabase;
use smallvec::SmallVec;

mod animation;
mod drawing;
mod position;
mod style;
mod wrap;
use style::OwnedStyle;

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
    /// Script `WrapStyle` header (0 smart, 1 greedy, 2 none, 3 smart); the `\q`
    /// override takes precedence per event. Defaults to 0.
    wrap_style: u8,
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
            wrap_style: 0,
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
            wrap_style: 0,
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

    fn process_event(
        &mut self,
        event: &Event,
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        // Get text segments with their individual tags
        let segments = segment_text_with_tags(event.text, None)?;

        if segments.is_empty() {
            return Ok(Vec::new());
        }

        // Check if this is a drawing command
        if let Some(draw_level) = segments[0].tags.drawing_mode {
            if draw_level > 0 {
                // Clone the style to avoid borrow issues
                let style_cloned = self
                    .styles_map
                    .get(event.style)
                    .or(self.default_style.as_ref())
                    .cloned();

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

        // Process text segments with proper style inheritance
        self.process_text_segments(segments, event, style_cloned.as_ref(), time_cs, context)
    }

    fn process_text_segments(
        &mut self,
        segments: Vec<TextSegment>,
        event: &Event,
        style: Option<&OwnedStyle>,
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        let mut all_layers = Vec::new();

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

        // Effective wrap style: a `\q` override (clamped to 0..=3, like libass which
        // falls back to the track style on an invalid value) takes precedence over
        // the script's WrapStyle header.
        let event_wrap_style = segments
            .iter()
            .find_map(|s| s.tags.formatting.wrap_style)
            .filter(|&q| q <= 3)
            .unwrap_or(self.wrap_style);

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

        // Width-based auto-wrap. WrapStyle 2 disables width wrapping entirely (only
        // explicit \N breaks, already split into logical_lines); positioned events
        // (\pos/\move) keep their explicit layout. WrapStyle 1 wraps greedily with
        // no balancing; 0/3 balance the lines (libass smart wrapping).
        if !Self::event_is_positioned(event) && event_wrap_style != 2 {
            let margin_l =
                Self::margin_or_style(event.margin_l, style.map(|s| s.margin_l.as_str()));
            let margin_r =
                Self::margin_or_style(event.margin_r, style.map(|s| s.margin_r.as_str()));
            let available = (context.width() as f32 - (margin_l + margin_r) * scale_x).max(1.0);
            let balance = event_wrap_style != 1;
            let mut wrapped: Vec<Vec<TextSegment>> = Vec::with_capacity(logical_lines.len());
            for line in logical_lines {
                wrapped.extend(self.wrap_segments(
                    &line,
                    default_font_name,
                    default_font_size_base,
                    default_scale_y,
                    default_bold,
                    default_italic,
                    default_spacing,
                    default_scale_x,
                    scale_y,
                    available,
                    balance,
                ));
            }
            logical_lines = wrapped;
        }

        // Calculate total height for multi-line text
        let num_lines = logical_lines.len();
        // For line height in libass:
        // - Line spacing uses the font size in script resolution scaled to render resolution
        // - DPI scaling affects glyph rendering but NOT line spacing
        // - ScaleY affects glyph size but NOT line spacing
        let line_spacing_multiplier = 1.0; // Match libass line spacing

        // Line height for spacing between lines (not affected by the glyph DPI
        // scale in libass). libass advances by the line's font size, so use the
        // largest font size across the event (covering inline `\fs` overrides)
        // rather than only the style default, which packed large-`\fs` lines too
        // tightly.
        let max_font_size = logical_lines
            .iter()
            .flatten()
            .filter_map(|seg| seg.tags.font.size)
            .fold(default_font_size_base, f32::max);
        let estimated_line_height = max_font_size * scale_y;

        let _total_text_height = estimated_line_height * num_lines as f32 * line_spacing_multiplier;

        // Process each logical line
        let mut line_y_offset = 0.0;

        for (line_index, line_segments) in logical_lines.into_iter().enumerate() {
            // For a multi-segment line, total rendered width so it is aligned as
            // one unit rather than each segment re-centering on its own width. A
            // single-segment line uses its per-segment shaped width below (which
            // already reflects \t animations), so skip the pre-pass there.
            let is_multi_segment = line_segments.len() > 1;
            let line_total_width: f32 = if is_multi_segment {
                line_segments
                    .iter()
                    .map(|seg| {
                        let size = seg.tags.font.size.unwrap_or(default_font_size_base)
                            * scale_y
                            * (seg.tags.font.scale_y.unwrap_or(default_scale_y) / 100.0)
                            * self.dpi_scale;
                        let fsx = seg.tags.font.scale_x.unwrap_or(default_scale_x) / 100.0;
                        let font = seg.tags.font.name.as_deref().unwrap_or(default_font_name);
                        let bold = seg.tags.formatting.bold.unwrap_or(default_bold);
                        let italic = seg.tags.formatting.italic.unwrap_or(default_italic);
                        shape_text_cached(&seg.text, font, size, bold, italic, &self.font_database)
                            .map_or(0.0, |sh| sh.width * fsx)
                    })
                    .sum()
            } else {
                0.0
            };
            let mut current_x = 0.0;
            let mut needs_initial_position = true;
            let mut karaoke_accumulated_time = 0u32; // Track cumulative karaoke time for this line

            for segment in line_segments {
                let mut tags = segment.tags.clone();
                let line_text = &segment.text;

                // Check if this segment is in drawing mode
                if let Some(drawing_mode) = tags.drawing_mode {
                    if drawing_mode > 0 {
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
                let default_colors = (
                    default_primary_color,
                    default_secondary_color,
                    default_outline_color,
                    default_back_color,
                );
                self.apply_transform_animations(
                    &mut tags,
                    event_start_cs,
                    event.end_time_cs().unwrap_or(u32::MAX),
                    time_cs,
                    default_colors,
                    default_font_size_base,
                );

                // Shape the text first to get dimensions for proper alignment
                // Get base font size and scale factors
                let base_font_size = tags.font.size.unwrap_or(default_font_size_base); // Use unscaled base size
                let font_scale_x = tags.font.scale_x.unwrap_or(default_scale_x) / 100.0;
                let font_scale_y = tags.font.scale_y.unwrap_or(default_scale_y) / 100.0;
                // Apply both resolution scaling, percentage scaling, and DPI scaling
                let actual_font_size = base_font_size * scale_y * font_scale_y * self.dpi_scale;

                let font_to_use = tags.font.name.as_deref().unwrap_or(default_font_name);

                let shaped = shape_text_cached(
                    line_text,
                    font_to_use,
                    actual_font_size,
                    tags.formatting.bold.unwrap_or(default_bold),
                    tags.formatting.italic.unwrap_or(default_italic),
                    &self.font_database,
                )?;

                // Calculate position for this segment
                let (segment_x, segment_y) = if tags.position.is_some() || tags.movement.is_some() {
                    // Explicit \pos/\move override (inherited by every segment of the
                    // line). The first segment anchors the whole line — centred on the
                    // full line width for multi-segment lines — and seeds current_x;
                    // later segments continue left-to-right from current_x instead of
                    // each re-centering on the same point (which stacked karaoke and
                    // colour-split segments on top of one another).
                    let (anchor_x, anchor_y) = self.calculate_position_from_tags(
                        &tags,
                        event,
                        context,
                        time_cs,
                        default_alignment,
                    );
                    let alignment = tags.formatting.alignment.unwrap_or(default_alignment);
                    if needs_initial_position {
                        let line_width = if is_multi_segment {
                            line_total_width
                        } else {
                            shaped.width
                        };
                        let (x, y) = self.apply_alignment_offset(
                            anchor_x,
                            anchor_y,
                            line_width,
                            shaped.height,
                            alignment,
                        );
                        current_x = x;
                        needs_initial_position = false;
                        (x, y)
                    } else {
                        let (_, y) = self.apply_alignment_offset(
                            anchor_x,
                            anchor_y,
                            shaped.width,
                            shaped.height,
                            alignment,
                        );
                        (current_x, y)
                    }
                } else if needs_initial_position {
                    // First segment of the line: position the whole line. Later
                    // segments continue from current_x (below) so they lay out
                    // left-to-right instead of each re-centering.
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

                    // Multi-segment lines centre on the whole-line width; single
                    // segments use their own shaped width (reflects \t animations).
                    let line_width = if is_multi_segment {
                        line_total_width
                    } else {
                        shaped.width
                    };
                    let (x, y) = self.apply_alignment_offset(
                        anchor_x,
                        adjusted_y,
                        line_width,
                        height_for_positioning,
                        alignment,
                    );

                    // Start the pen at the line's left edge; the post-draw advance
                    // below moves current_x past each segment (centering uses the
                    // whole line's width so multi-segment lines align as a unit).
                    current_x = x;
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

                // `\c`/`\3c`/`\4c` override only RGB; their 6-digit form carries no
                // alpha (parse_bgr_color yields 0). Alpha is inherited from the style
                // and may be overridden by `\alpha`/`\1a`-family tags below, so restore
                // the inherited alpha here rather than letting the override zero it out.
                color[3] = default_primary_color[3];
                outline_color[3] = default_outline_color[3];
                shadow_color[3] = default_back_color[3];

                // Apply individual alpha overrides (ASS alpha is inverted: 00=opaque, FF=transparent)
                if let Some(alpha) = tags.colors.alpha1.or(tags.colors.alpha) {
                    // Alpha is already inverted in parse_alpha (255 = opaque, 0 = transparent)
                    color[3] = alpha;
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

                    let fade_alpha = if let Some(alpha_mid) = fade.alpha_middle {
                        // Complex \fade(a1,a2,a3,t1,t2,t3,t4): a 5-segment piecewise
                        // alpha over event-relative times (a1 before t1, ramp a1->a2
                        // over t1..t2, hold a2 over t2..t3, ramp a2->a3 over t3..t4,
                        // a3 after t4). ASS alphas are inverted (00=opaque).
                        let (a1, a2, a3) = (
                            fade.alpha_start as f32,
                            alpha_mid as f32,
                            fade.alpha_end as f32,
                        );
                        let t1 = event_start + fade.time_start;
                        let t2 = t1 + fade.time_fade_in.unwrap_or(0);
                        let t4 = event_start + fade.time_end;
                        let t3 = t4.saturating_sub(fade.time_fade_out.unwrap_or(0));
                        let ass_alpha = if time_cs <= t1 {
                            a1
                        } else if time_cs < t2 {
                            a1 + (a2 - a1) * (time_cs - t1) as f32 / (t2 - t1).max(1) as f32
                        } else if time_cs <= t3 {
                            a2
                        } else if time_cs < t4 {
                            a2 + (a3 - a2) * (time_cs - t3) as f32 / (t4 - t3).max(1) as f32
                        } else {
                            a3
                        };
                        // Convert ASS alpha (00=opaque, FF=transparent) to opacity.
                        255.0 - ass_alpha
                    } else {
                        // Simple fade - times are durations
                        let fade_in_end = event_start + fade.time_start;
                        let fade_out_start = event_end.saturating_sub(fade.time_end);

                        if time_cs < fade_in_end {
                            // During fade in
                            let progress = (time_cs.saturating_sub(event_start)) as f32
                                / fade.time_start.max(1) as f32;
                            255.0 * progress.min(1.0)
                        } else if time_cs >= fade_out_start && fade_out_start < event_end {
                            // During fade out
                            let progress = (event_end.saturating_sub(time_cs)) as f32
                                / fade.time_end.max(1) as f32;
                            255.0 * progress.min(1.0)
                        } else {
                            // Fully visible
                            255.0
                        }
                    };

                    // Apply fade to all color components (primary, outline, shadow)
                    let fade_factor = fade_alpha / 255.0;

                    color[3] = (color[3] as f32 * fade_factor) as u8;
                    outline_color[3] = (outline_color[3] as f32 * fade_factor) as u8;
                    shadow_color[3] = (shadow_color[3] as f32 * fade_factor) as u8;
                }

                // Get spacing value (from tags or default style). `\fsp`/style
                // spacing is in script units; scale it to screen like positions so
                // the rendered run width (and thus wrapping) matches libass, which
                // applies fsp * scale. Unscaled, spaced text drew ~2% too wide.
                let spacing = tags.font.spacing.unwrap_or(default_spacing) * scale_x;

                // Create text layer
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
                let border_style = style
                    .and_then(|s| s.border_style.trim().parse::<u8>().ok())
                    .unwrap_or(1);
                if border_style == 3 {
                    // BorderStyle 3: opaque box behind the text in the outline colour,
                    // padded per-axis (\xbord/\ybord).
                    layer.effects.push(TextEffect::OpaqueBox {
                        color: outline_color,
                        padding_x: outline_width_x,
                        padding_y: outline_width_y,
                    });
                } else if outline_width_x > 0.0 || outline_width_y > 0.0 {
                    layer.effects.push(TextEffect::Outline {
                        color: outline_color,
                        width_x: outline_width_x,
                        width_y: outline_width_y,
                    });
                }

                // Add shadow effect with per-axis shadow support
                // Scale tag values only if ScaledBorderAndShadow is enabled
                let shadow_x = if self.scaled_border_and_shadow {
                    tags.formatting
                        .shadow_x
                        .map(|s| s * scale_x)
                        .or(tags.formatting.shadow.map(|s| s * scale_x))
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
                    // libass offsets the shadow by the full (scaled) \shad distance.
                    layer.effects.push(TextEffect::Shadow {
                        color: shadow_color,
                        x_offset: shadow_x,
                        y_offset: shadow_y,
                    });
                }

                // Add blur effect - handle both blur and edge blur
                let blur_radius = tags.formatting.blur.unwrap_or(0.0);
                let edge_blur = tags.formatting.blur_edges.unwrap_or(0.0);
                if blur_radius > 0.0 {
                    // libass converts \blur to screen pixels via blur_scale =
                    // frame/PlayRes (a resolution conversion applied
                    // unconditionally, independent of ScaledBorderAndShadow);
                    // apply_gaussian_blur then maps that screen radius to a
                    // Gaussian std-dev with blur_radius_scale = 2/sqrt(ln 256).
                    layer.effects.push(TextEffect::Blur {
                        radius: blur_radius * scale_y,
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
                    // `\org` sets the rotation centre in script coordinates; scale
                    // it to screen space for the backend.
                    let origin = tags.origin.map(|(ox, oy)| (ox * scale_x, oy * scale_y));
                    layer.effects.push(TextEffect::Rotation {
                        x: rotation_x,
                        y: rotation_y,
                        z: rotation_z,
                        origin,
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

                // Add a scale effect when either axis is non-100%. \fscy is folded
                // into the shaped font size (uniform), so the backend uses the
                // x/y ratio to correct the horizontal axis; \fscy alone (x=100,
                // y!=100) still needs the effect so that correction runs.
                let font_scale_x_val = tags.font.scale_x.unwrap_or(default_scale_x);
                let font_scale_y_val = tags.font.scale_y.unwrap_or(default_scale_y);
                if (font_scale_x_val - 100.0).abs() > 0.01
                    || (font_scale_y_val - 100.0).abs() > 0.01
                {
                    layer.effects.push(TextEffect::Scale {
                        x: font_scale_x_val,
                        y: font_scale_y_val,
                    });
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

                    // Unsung syllables use the secondary colour. `\2c` overrides are
                    // 6-digit (no alpha), so inherit alpha from the style default.
                    let mut karaoke_secondary =
                        tags.colors.secondary.unwrap_or(default_secondary_color);
                    karaoke_secondary[3] = default_secondary_color[3];

                    // Add karaoke effect with correct progress
                    layer.effects.push(TextEffect::Karaoke {
                        progress,
                        style: match karaoke.style {
                            KaraokeStyle::Basic => 0,
                            KaraokeStyle::Fill => 1,
                            KaraokeStyle::Outline => 2,
                            KaraokeStyle::Sweep => 3,
                        },
                        secondary: karaoke_secondary,
                    });

                    // Accumulate time for next syllable
                    karaoke_accumulated_time += karaoke.duration;
                }

                // Advance the pen to the end of this segment so the next run on the
                // line continues right after it. Always use the real shaped advance:
                // the old character-count estimate (used whenever alignment was set,
                // including inherited `\an`) over-advanced and left gaps between the
                // runs of a multi-segment line.
                if let Some(advance) = shaped.total_advance() {
                    current_x += advance * font_scale_x;
                }

                all_layers.push(IntermediateLayer::Text(layer));
            }

            // Move to next line. Advance by the nominal line height (font size in
            // render resolution), matching libass's baseline-to-baseline spacing.
            // `line_height` carries the 0.9 glyph dpi_scale, which libass does not
            // apply to line advance, so it must not be used here.
            line_y_offset += estimated_line_height * line_spacing_multiplier;
        }

        Ok(all_layers)
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

                    // Extract WrapStyle (0 smart / 1 greedy / 2 none / 3 smart)
                    self.wrap_style = info.wrap_style();

                    // Extract LayoutResX and LayoutResY if present
                    if let Some((layout_x, layout_y)) = info.layout_resolution() {
                        self.layout_res_x = Some(layout_x as f32);
                        self.layout_res_y = Some(layout_y as f32);

                        // If LayoutRes differs from PlayRes, we need to scale styles
                        // This is done later when processing styles
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

        let scale_y = context.height() as f32 / self.play_res_y;

        // Process each event, applying collision resolution so simultaneous
        // non-positioned events stack instead of overlapping (libass "Normal"
        // collisions). Positioned events (\pos/\move) are exempt and do not
        // participate in stacking.
        for event in sorted_events {
            let mut event_layers = self.process_event(event, time_cs, context)?;

            if !Self::event_is_positioned(event) {
                if let Some(bbox) = self.event_bounding_box(&event_layers) {
                    let positioned = PositionedEvent {
                        bbox,
                        layer: event.layer.parse::<i32>().unwrap_or(0),
                        margin_v: self.event_margin_v(event, scale_y) as i32,
                        margin_l: 0,
                        margin_r: 0,
                        alignment: self.effective_alignment(event),
                        priority: 0,
                    };
                    let resolved = self.collision_resolver.find_position(positioned);
                    let dy = resolved.y - bbox.y;
                    if dy.abs() > 0.5 {
                        Self::offset_layers_y(&mut event_layers, dy);
                    }
                }
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
