//! Text-segment layout for the software pipeline: resolving style defaults,
//! per-run colours/alpha/fade, effects, and the positioned `TextData` layers.

#[cfg(feature = "nostd")]
use alloc::{string::ToString, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::ToString, vec::Vec};

use ass_core::parser::Event;
use smallvec::SmallVec;

use super::OwnedStyle;
use crate::pipeline::{
    shaping::shape_text_cached, tag_processor::KaraokeStyle, text_segmenter::TextSegment,
    IntermediateLayer, TextData, TextEffect,
};
use crate::renderer::RenderContext;
use crate::utils::RenderError;

mod colors;
mod defaults;
mod effects;
mod position;
mod types;

use colors::{resolve_run_colors, split_into_logical_lines};
use types::{EffectColors, LineLayout, Pen, RunCtx};

impl super::SoftwarePipeline {
    pub(super) fn process_text_segments(
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

        // Resolve the style-derived defaults (font/size/colours/formatting) once.
        let defaults = self.resolve_text_defaults(style, scale_y);

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
        let mut logical_lines = split_into_logical_lines(segments);

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
                    defaults.font_name,
                    defaults.font_size_base,
                    defaults.scale_y,
                    defaults.bold,
                    defaults.italic,
                    defaults.spacing,
                    defaults.scale_x,
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
            .fold(defaults.font_size_base, f32::max);
        let estimated_line_height = max_font_size * scale_y;

        let _total_text_height = estimated_line_height * num_lines as f32 * line_spacing_multiplier;

        // Process each logical line
        let mut line_y_offset = 0.0;

        let run_ctx = RunCtx {
            event,
            context,
            time_cs,
            scale_x,
            scale_y,
        };

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
                        let size = seg.tags.font.size.unwrap_or(defaults.font_size_base)
                            * scale_y
                            * (seg.tags.font.scale_y.unwrap_or(defaults.scale_y) / 100.0)
                            * self.dpi_scale;
                        let fsx = seg.tags.font.scale_x.unwrap_or(defaults.scale_x) / 100.0;
                        let font = seg.tags.font.name.as_deref().unwrap_or(defaults.font_name);
                        let bold = seg.tags.formatting.bold.unwrap_or(defaults.bold);
                        let italic = seg.tags.formatting.italic.unwrap_or(defaults.italic);
                        shape_text_cached(&seg.text, font, size, bold, italic, &self.font_database)
                            .map_or(0.0, |sh| sh.width * fsx)
                    })
                    .sum()
            } else {
                0.0
            };
            let mut pen = Pen {
                current_x: 0.0,
                needs_initial_position: true,
            };
            let mut karaoke_accumulated_time = 0u32; // Track cumulative karaoke time for this line

            let layout = LineLayout {
                is_multi_segment,
                line_total_width,
                num_lines,
                line_index,
                estimated_line_height,
                line_spacing_multiplier,
                line_y_offset,
            };

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
                    defaults.primary_color,
                    defaults.secondary_color,
                    defaults.outline_color,
                    defaults.back_color,
                );
                self.apply_transform_animations(
                    &mut tags,
                    event_start_cs,
                    event.end_time_cs().unwrap_or(u32::MAX),
                    time_cs,
                    default_colors,
                    defaults.font_size_base,
                );

                // Shape the text first to get dimensions for proper alignment
                // Get base font size and scale factors
                let base_font_size = tags.font.size.unwrap_or(defaults.font_size_base); // Use unscaled base size
                let font_scale_x = tags.font.scale_x.unwrap_or(defaults.scale_x) / 100.0;
                let font_scale_y = tags.font.scale_y.unwrap_or(defaults.scale_y) / 100.0;
                // Apply both resolution scaling, percentage scaling, and DPI scaling
                let actual_font_size = base_font_size * scale_y * font_scale_y * self.dpi_scale;

                let font_to_use = tags.font.name.as_deref().unwrap_or(defaults.font_name);

                let shaped = shape_text_cached(
                    line_text,
                    font_to_use,
                    actual_font_size,
                    tags.formatting.bold.unwrap_or(defaults.bold),
                    tags.formatting.italic.unwrap_or(defaults.italic),
                    &self.font_database,
                )?;

                // Calculate position for this segment
                let (segment_x, segment_y) = self.position_segment(
                    &tags,
                    &shaped,
                    &layout,
                    &mut pen,
                    &run_ctx,
                    defaults.alignment,
                );

                // Get font information with proper inheritance
                let font_family = tags.font.name.as_deref().unwrap_or(defaults.font_name);
                // Font size was already calculated above for shaping
                // Use the same value here for consistency
                let font_size = actual_font_size;

                // Note: 'shaped' was already created above for alignment calculation

                // Resolve colours, alpha overrides, and fade for this run.
                let (color, outline_color, shadow_color) =
                    resolve_run_colors(&tags, &defaults, &run_ctx);

                // Get spacing value (from tags or default style). `\fsp`/style
                // spacing is in script units; scale it to screen like positions so
                // the rendered run width (and thus wrapping) matches libass, which
                // applies fsp * scale. Unscaled, spaced text drew ~2% too wide.
                let spacing = tags.font.spacing.unwrap_or(defaults.spacing) * scale_x;

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

                // Add effects (formatting, outline/shadow/blur/opaque-box, rotation,
                // shear, scale, clip, baseline offset).
                self.push_text_effects(
                    &mut layer,
                    &tags,
                    EffectColors {
                        outline_color,
                        shadow_color,
                    },
                    &run_ctx,
                    &defaults,
                    style,
                );

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
                        tags.colors.secondary.unwrap_or(defaults.secondary_color);
                    karaoke_secondary[3] = defaults.secondary_color[3];

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
                    pen.current_x += advance * font_scale_x;
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
