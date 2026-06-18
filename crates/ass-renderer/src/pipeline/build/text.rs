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
    shaping::{shape_text_cached, ShapedText},
    tag_processor::{KaraokeStyle, ProcessedTags},
    text_segmenter::TextSegment,
    IntermediateLayer, TextData, TextEffect,
};
use crate::renderer::RenderContext;
use crate::utils::RenderError;

/// Style-derived defaults (font, size, colours, formatting) resolved once per
/// event. Borrows the run's resolved style for its zero-copy font name.
struct TextDefaults<'a> {
    font_name: &'a str,
    font_size_base: f32,
    bold: bool,
    italic: bool,
    underline: bool,
    strikeout: bool,
    primary_color: [u8; 4],
    secondary_color: [u8; 4],
    outline_color: [u8; 4],
    back_color: [u8; 4],
    outline: f32,
    shadow: f32,
    scale_x: f32,
    scale_y: f32,
    spacing: f32,
    alignment: u8,
}

/// Per-call rendering context shared by the per-segment helpers (constant
/// across every line and segment of the event).
struct RunCtx<'a, 'b> {
    event: &'a Event<'b>,
    context: &'a RenderContext,
    time_cs: u32,
    scale_x: f32,
    scale_y: f32,
}

/// Per-line layout constants consumed when positioning each segment.
struct LineLayout {
    is_multi_segment: bool,
    line_total_width: f32,
    num_lines: usize,
    line_index: usize,
    estimated_line_height: f32,
    line_spacing_multiplier: f32,
    line_y_offset: f32,
}

/// Pen state carried across the segments of a single line.
struct Pen {
    current_x: f32,
    needs_initial_position: bool,
}

/// Resolved outline and shadow colours handed to the effects builder.
struct EffectColors {
    outline_color: [u8; 4],
    shadow_color: [u8; 4],
}

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

    /// Resolve the style-derived defaults (font, size, colours, and formatting)
    /// shared by every segment of the event.
    fn resolve_text_defaults<'a>(
        &self,
        style: Option<&'a OwnedStyle>,
        scale_y: f32,
    ) -> TextDefaults<'a> {
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

        TextDefaults {
            font_name: default_font_name,
            font_size_base: default_font_size_base,
            bold: default_bold,
            italic: default_italic,
            underline: default_underline,
            strikeout: default_strikeout,
            primary_color: default_primary_color,
            secondary_color: default_secondary_color,
            outline_color: default_outline_color,
            back_color: default_back_color,
            outline: default_outline,
            shadow: default_shadow,
            scale_x: default_scale_x,
            scale_y: default_scale_y,
            spacing: default_spacing,
            alignment: default_alignment,
        }
    }

    /// Compute the top-left render position of a segment, advancing the line's
    /// pen and seeding it on the first segment of the line.
    fn position_segment(
        &self,
        tags: &ProcessedTags,
        shaped: &ShapedText,
        layout: &LineLayout,
        pen: &mut Pen,
        ctx: &RunCtx,
        default_alignment: u8,
    ) -> (f32, f32) {
        let event = ctx.event;
        let context = ctx.context;
        let time_cs = ctx.time_cs;
        let is_multi_segment = layout.is_multi_segment;
        let line_total_width = layout.line_total_width;
        let num_lines = layout.num_lines;
        let line_index = layout.line_index;
        let estimated_line_height = layout.estimated_line_height;
        let line_spacing_multiplier = layout.line_spacing_multiplier;
        let line_y_offset = layout.line_y_offset;

        if tags.position.is_some() || tags.movement.is_some() {
            // Explicit \pos/\move override (inherited by every segment of the
            // line). The first segment anchors the whole line — centred on the
            // full line width for multi-segment lines — and seeds current_x;
            // later segments continue left-to-right from current_x instead of
            // each re-centering on the same point (which stacked karaoke and
            // colour-split segments on top of one another).
            let (anchor_x, anchor_y) =
                self.calculate_position_from_tags(tags, event, context, time_cs, default_alignment);
            let alignment = tags.formatting.alignment.unwrap_or(default_alignment);
            if pen.needs_initial_position {
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
                pen.current_x = x;
                pen.needs_initial_position = false;
                (x, y)
            } else {
                let (_, y) = self.apply_alignment_offset(
                    anchor_x,
                    anchor_y,
                    shaped.width,
                    shaped.height,
                    alignment,
                );
                (pen.current_x, y)
            }
        } else if pen.needs_initial_position {
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
            pen.current_x = x;
            pen.needs_initial_position = false;
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
            (pen.current_x, y)
        }
    }

    /// Push the per-run effects (formatting, outline/shadow/blur/opaque-box,
    /// rotation, shear, scale, clip) onto `layer` and apply any baseline offset.
    fn push_text_effects(
        &self,
        layer: &mut TextData,
        tags: &ProcessedTags,
        colors: EffectColors,
        ctx: &RunCtx,
        defaults: &TextDefaults,
        style: Option<&OwnedStyle>,
    ) {
        let scale_x = ctx.scale_x;
        let scale_y = ctx.scale_y;
        let default_outline = defaults.outline;
        let default_shadow = defaults.shadow;
        let default_scale_x = defaults.scale_x;
        let default_scale_y = defaults.scale_y;
        let outline_color = colors.outline_color;
        let shadow_color = colors.shadow_color;

        // Get formatting with inheritance from style
        let bold = tags.formatting.bold.unwrap_or(defaults.bold);
        let italic = tags.formatting.italic.unwrap_or(defaults.italic);
        let underline = tags.formatting.underline.unwrap_or(defaults.underline);
        let strikeout = tags.formatting.strikeout.unwrap_or(defaults.strikeout);

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
        if (font_scale_x_val - 100.0).abs() > 0.01 || (font_scale_y_val - 100.0).abs() > 0.01 {
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
    }
}

/// Split the event's tagged segments into logical lines on explicit `\N`
/// newlines, preserving per-segment tags.
fn split_into_logical_lines(segments: Vec<TextSegment>) -> Vec<Vec<TextSegment>> {
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

    logical_lines
}

/// Resolve the primary, outline, and shadow colours for a run, applying the
/// `\1a`/`\3a`/`\4a` alpha overrides and any `\fad`/`\fade` effect.
fn resolve_run_colors(
    tags: &ProcessedTags,
    defaults: &TextDefaults,
    ctx: &RunCtx,
) -> ([u8; 4], [u8; 4], [u8; 4]) {
    let default_primary_color = defaults.primary_color;
    let default_outline_color = defaults.outline_color;
    let default_back_color = defaults.back_color;
    let event = ctx.event;
    let time_cs = ctx.time_cs;

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
                let progress =
                    (time_cs.saturating_sub(event_start)) as f32 / fade.time_start.max(1) as f32;
                255.0 * progress.min(1.0)
            } else if time_cs >= fade_out_start && fade_out_start < event_end {
                // During fade out
                let progress =
                    (event_end.saturating_sub(time_cs)) as f32 / fade.time_end.max(1) as f32;
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

    (color, outline_color, shadow_color)
}
