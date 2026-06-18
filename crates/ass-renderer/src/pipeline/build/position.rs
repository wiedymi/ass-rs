//! Position, alignment, and margin resolution for the software pipeline.

use crate::collision::BoundingBox;
use crate::pipeline::{
    animation::calculate_move_progress, tag_processor::ProcessedTags, IntermediateLayer,
};
use crate::renderer::RenderContext;
use ass_core::parser::Event;

impl super::SoftwarePipeline {
    /// Whether an event pins its own position (`\pos`/`\move`) and is therefore
    /// exempt from collision stacking.
    pub(super) fn event_is_positioned(event: &Event) -> bool {
        event.text.contains("\\pos") || event.text.contains("\\move")
    }

    /// Effective vertical-alignment row for collision: an inline `\an` override
    /// wins over the style alignment (default 2 = bottom-centre).
    pub(super) fn effective_alignment(&self, event: &Event) -> u8 {
        if let Some(idx) = event.text.find("\\an") {
            if let Some(d) = event.text[idx + 3..]
                .chars()
                .next()
                .and_then(|c| c.to_digit(10))
            {
                if (1..=9).contains(&d) {
                    return d as u8;
                }
            }
        }
        self.styles_map
            .get(event.style)
            .and_then(|s| s.alignment.parse::<u8>().ok())
            .filter(|a| (1..=9).contains(a))
            .unwrap_or(2)
    }

    /// Event vertical margin in render-pixel space (event margin overrides the
    /// style margin; `0`/empty means "use the style").
    pub(super) fn event_margin_v(&self, event: &Event, scale_y: f32) -> f32 {
        let style_mv = self
            .styles_map
            .get(event.style)
            .and_then(|s| s.margin_v.parse::<f32>().ok())
            .unwrap_or(0.0);
        let mv = if event.margin_v.is_empty() || event.margin_v == "0" {
            style_mv
        } else {
            event.margin_v.parse::<f32>().unwrap_or(style_mv)
        };
        mv * scale_y
    }

    /// Approximate the rendered bounding box of an event from its text layers.
    /// Per-line height uses the nominal font size (`font_size / dpi_scale`) so it
    /// matches libass's baseline-to-baseline line box used for stacking.
    pub(super) fn event_bounding_box(&self, layers: &[IntermediateLayer]) -> Option<BoundingBox> {
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        let mut found = false;
        for layer in layers {
            if let IntermediateLayer::Text(text) = layer {
                let line_height = text.font_size / self.dpi_scale.max(0.01);
                let width = text.text.chars().count() as f32 * text.font_size * 0.5;
                min_x = min_x.min(text.x);
                min_y = min_y.min(text.y);
                max_x = max_x.max(text.x + width);
                max_y = max_y.max(text.y + line_height);
                found = true;
            }
        }
        found.then(|| BoundingBox::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }

    /// Shift every layer of an event vertically by `dy` (used after collision
    /// resolution moves the event to a free slot).
    pub(super) fn offset_layers_y(layers: &mut [IntermediateLayer], dy: f32) {
        for layer in layers {
            match layer {
                IntermediateLayer::Text(text) => text.y += dy,
                IntermediateLayer::Raster(raster) => {
                    raster.y = (raster.y as f32 + dy).max(0.0) as u32;
                }
                IntermediateLayer::Vector(_) => {}
            }
        }
    }

    /// Resolve a margin (script coordinates): the event margin overrides the style
    /// margin, but `0`/empty means "use the style".
    pub(super) fn margin_or_style(event_margin: &str, style_margin: Option<&str>) -> f32 {
        let style = style_margin
            .and_then(|m| m.parse::<f32>().ok())
            .unwrap_or(0.0);
        if event_margin.is_empty() || event_margin == "0" {
            style
        } else {
            event_margin.parse::<f32>().unwrap_or(style)
        }
    }

    pub(super) fn calculate_position_from_tags(
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

            // t1/t2 already arrive in centiseconds: the \move parser converted the
            // tag's milliseconds once. With both zero the movement spans the whole
            // event duration (libass default).
            let (move_start_cs, move_end_cs) = if t1 == 0 && t2 == 0 {
                (event_start_cs, event_end_cs)
            } else {
                (event_start_cs + t1, event_start_cs + t2)
            };

            let progress = calculate_move_progress(time_cs, move_start_cs, move_end_cs);
            let x = x1 + (x2 - x1) * progress;
            let y = y1 + (y2 - y1) * progress;

            // Scale from script (PlayRes) coordinates to render coordinates
            return (x * scale_x, y * scale_y);
        }

        // Calculate based on alignment
        let alignment = tags.formatting.alignment.unwrap_or(default_alignment);
        self.calculate_position_from_alignment(alignment, event, context)
    }

    pub(super) fn calculate_position_from_alignment(
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

        (x, y)
    }

    /// Apply alignment offset to convert from anchor point to top-left corner
    /// Takes the anchor point and text dimensions, returns top-left corner for rendering
    pub(super) fn apply_alignment_offset(
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

        (x, y)
    }
}
