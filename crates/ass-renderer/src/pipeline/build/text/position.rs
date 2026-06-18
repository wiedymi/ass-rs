//! Per-segment positioning for the software pipeline: anchoring lines, applying
//! alignment offsets, and advancing the line pen.

use super::{LineLayout, Pen, RunCtx};
use crate::pipeline::shaping::ShapedText;
use crate::pipeline::tag_processor::ProcessedTags;

impl super::super::SoftwarePipeline {
    /// Compute the top-left render position of a segment, advancing the line's
    /// pen and seeding it on the first segment of the line.
    pub(super) fn position_segment(
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
}
