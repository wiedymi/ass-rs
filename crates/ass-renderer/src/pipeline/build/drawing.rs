//! Drawing (`\p` vector) command processing for the software pipeline.

#[cfg(feature = "nostd")]
use alloc::{vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::vec::Vec;

use ass_core::parser::Event;
use tiny_skia::Transform;

use super::OwnedStyle;
use crate::pipeline::{
    animation::calculate_move_progress, drawing::process_drawing_commands,
    text_segmenter::TextSegment, IntermediateLayer, StrokeInfo, VectorData,
};
use crate::renderer::RenderContext;
use crate::utils::RenderError;

impl super::SoftwarePipeline {
    pub(super) fn process_drawing_command(
        &mut self,
        segment: &TextSegment,
        _event: &Event,
        style: Option<&OwnedStyle>,
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError> {
        let plain_text = &segment.text;
        let tags = &segment.tags;

        // Try to get drawing path from cache
        let draw_cache_key = crate::cache::DrawingCacheKey {
            commands: plain_text.clone(),
        };

        let path_opt = if let Some(cached) = self.cache.get_drawing_path(&draw_cache_key) {
            cached
        } else {
            let path = process_drawing_commands(plain_text)?;
            self.cache.store_drawing_path(draw_cache_key, path.clone());
            path
        };

        if let Some(path) = path_opt {
            // Get color from tags or style. `\c` overrides only RGB, so inherit the
            // alpha from the style (parse_bgr_color leaves alpha at 0 for 6-digit tags).
            let mut color = if let Some(mut c) = tags.colors.primary {
                c[3] = style.map_or(255, |s| Self::parse_ass_color(&s.primary_colour)[3]);
                c
            } else if let Some(s) = style {
                Self::parse_ass_color(&s.primary_colour)
            } else {
                [255, 255, 255, 255]
            };
            // Apply the `\alpha` / `\1a` override (drawings are filled with the
            // primary colour). parse_alpha already inverted ASS alpha to RGBA
            // (255 = opaque). Without this, layered glow drawings (each with a
            // decreasing `\alpha`) all draw opaque and accumulate far too much ink.
            if let Some(alpha) = tags.colors.alpha1.or(tags.colors.alpha) {
                color[3] = alpha;
            }

            // Calculate scaling factors
            let scale_x = context.width() as f32 / self.play_res_x;
            let scale_y = context.height() as f32 / self.play_res_y;

            // Drawing geometry is in script (PlayRes) units, exactly like `\pos`,
            // so it must be scaled to the render resolution. Previously only the
            // position was scaled and the shape was left at script size, rendering
            // ~1.5x too large whenever the output differed from PlayRes (the ED
            // sparkle particles, PlayRes 1920 rendered at 1280, were the visible
            // case: oversized shapes ~3x the ink libass produced).
            let path = path
                .clone()
                .transform(Transform::from_scale(scale_x, scale_y))
                .unwrap_or(path);

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

            // `\clip` / `\iclip` coordinates are in script space (like `\pos`),
            // so scale them into render space the same way the text path does.
            let clip = tags.clip.as_ref().map(|c| {
                (
                    c.x1 * scale_x,
                    c.y1 * scale_y,
                    c.x2 * scale_x,
                    c.y2 * scale_y,
                    c.inverse,
                )
            });

            // `\blur` on a drawing softens the filled shape exactly like text:
            // scale the script value to screen pixels by blur_scale = frame/PlayRes
            // (apply_gaussian_blur maps it to a std-dev). Sparkle/dust particles and
            // gradient glows rely on this; without it they render as hard, bright
            // shapes instead of soft, dim ones.
            let blur = tags.formatting.blur.unwrap_or(0.0) * scale_y;

            // `\bord` on a drawing strokes its outline in the `\3c` colour. Scale the
            // width only when ScaledBorderAndShadow is set, mirroring text borders.
            let border_w = tags
                .formatting
                .border_x
                .or(tags.formatting.border)
                .map(|w| {
                    if self.scaled_border_and_shadow {
                        w * scale_y
                    } else {
                        w
                    }
                })
                .unwrap_or(0.0);
            let stroke = (border_w > 0.0).then(|| {
                let mut oc = tags.colors.outline.unwrap_or_else(|| {
                    style
                        .map(|s| Self::parse_ass_color(&s.outline_colour))
                        .unwrap_or([0, 0, 0, 255])
                });
                oc[3] = style.map_or(255, |s| Self::parse_ass_color(&s.outline_colour)[3]);
                if let Some(a) = tags.colors.alpha3.or(tags.colors.alpha) {
                    oc[3] = a;
                }
                StrokeInfo {
                    color: oc,
                    width: border_w,
                }
            });

            return Ok(vec![IntermediateLayer::Vector(VectorData {
                path: transformed_path,
                color,
                stroke,
                bounds: None,
                clip,
                blur,
            })]);
        }

        Ok(Vec::new())
    }
}
