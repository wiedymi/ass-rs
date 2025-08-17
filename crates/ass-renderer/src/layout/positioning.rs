//! Advanced positioning and alignment calculations based on libass
//!
//! This module provides positioning calculations that match libass behavior,
//! including proper anchor point handling, rotation origins, and alignment.

use crate::pipeline::tag_processor::ProcessedTags;

/// Calculated position with anchor and origin points
#[derive(Debug, Clone)]
pub struct PositionInfo {
    /// Actual render position (top-left of bounding box)
    pub render_x: f32,
    pub render_y: f32,
    /// Anchor point for alignment (based on alignment value)
    pub anchor_x: f32,
    pub anchor_y: f32,
    /// Rotation origin point
    pub origin_x: f32,
    pub origin_y: f32,
    /// Whether position was explicitly set
    pub explicit_position: bool,
}

/// Bounding box for text
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Configuration for position calculation
pub struct PositionConfig {
    pub screen_width: f32,
    pub screen_height: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub margin_vertical: f32,
    pub default_alignment: u8,
}

impl PositionInfo {
    /// Calculate position based on alignment and tags
    pub fn calculate(tags: &ProcessedTags, bbox: &BoundingBox, config: &PositionConfig) -> Self {
        // Get effective alignment
        let alignment = tags
            .formatting
            .alignment
            .unwrap_or(config.default_alignment);

        // Calculate anchor point based on alignment
        let (anchor_x, anchor_y) = Self::get_anchor_point(
            alignment,
            bbox,
            config.screen_width,
            config.screen_height,
            config.margin_left,
            config.margin_right,
            config.margin_vertical,
        );

        // Check for explicit positioning
        let (render_x, render_y, explicit) = if let Some((pos_x, pos_y)) = tags.position {
            // Explicit \pos tag
            let render_x = pos_x - Self::get_alignment_offset_x(alignment, bbox);
            let render_y = pos_y - Self::get_alignment_offset_y(alignment, bbox);
            (render_x, render_y, true)
        } else if let Some((x1, y1, _x2, _y2, _t1, _t2)) = tags.movement {
            // \move tag (simplified - should interpolate based on time)
            // For now just use start position
            let render_x = x1 - Self::get_alignment_offset_x(alignment, bbox);
            let render_y = y1 - Self::get_alignment_offset_y(alignment, bbox);
            (render_x, render_y, true)
        } else {
            // Use alignment-based positioning
            (anchor_x, anchor_y, false)
        };

        // Calculate rotation origin
        let (origin_x, origin_y) = if let Some((org_x, org_y)) = tags.origin {
            // Explicit \org tag
            (org_x, org_y)
        } else {
            // Default to anchor point (libass behavior)
            (anchor_x + bbox.width / 2.0, anchor_y + bbox.height / 2.0)
        };

        Self {
            render_x,
            render_y,
            anchor_x,
            anchor_y,
            origin_x,
            origin_y,
            explicit_position: explicit,
        }
    }

    /// Get anchor point based on alignment value
    fn get_anchor_point(
        alignment: u8,
        bbox: &BoundingBox,
        screen_width: f32,
        screen_height: f32,
        margin_left: f32,
        margin_right: f32,
        margin_vertical: f32,
    ) -> (f32, f32) {
        // Horizontal position based on alignment
        let x = match alignment % 3 {
            1 => {
                // Left alignment (1, 4, 7)
                margin_left
            }
            2 | 0 => {
                // Center alignment (2, 5, 8)
                (screen_width - bbox.width) / 2.0
            }
            _ => {
                // Right alignment (3, 6, 9)
                screen_width - margin_right - bbox.width
            }
        };

        // Vertical position based on alignment
        let y = match alignment {
            1..=3 => {
                // Bottom alignment
                screen_height - margin_vertical - bbox.height
            }
            4..=6 => {
                // Middle alignment
                (screen_height - bbox.height) / 2.0
            }
            7..=9 => {
                // Top alignment
                margin_vertical
            }
            _ => {
                // Default to bottom
                screen_height - margin_vertical - bbox.height
            }
        };

        (x, y)
    }

    /// Get horizontal offset for alignment anchor
    fn get_alignment_offset_x(alignment: u8, bbox: &BoundingBox) -> f32 {
        match alignment % 3 {
            1 => 0.0,                  // Left
            2 | 0 => bbox.width / 2.0, // Center
            _ => bbox.width,           // Right
        }
    }

    /// Get vertical offset for alignment anchor
    fn get_alignment_offset_y(alignment: u8, bbox: &BoundingBox) -> f32 {
        match alignment {
            1..=3 => bbox.height,       // Bottom
            4..=6 => bbox.height / 2.0, // Middle
            7..=9 => 0.0,               // Top
            _ => bbox.height,           // Default to bottom
        }
    }

    /// Calculate position with movement interpolation
    pub fn calculate_with_movement(
        tags: &ProcessedTags,
        bbox: &BoundingBox,
        config: &PositionConfig,
        current_time_ms: u32,
        event_start_ms: u32,
    ) -> Self {
        // Get base position
        let mut pos = Self::calculate(tags, bbox, config);

        // Apply movement if present
        if let Some((x1, y1, x2, y2, t1, t2)) = tags.movement {
            let event_time = current_time_ms.saturating_sub(event_start_ms);

            // Calculate interpolation factor
            let factor = if t2 > t1 {
                let progress = (event_time.saturating_sub(t1)) as f32 / (t2 - t1) as f32;
                progress.clamp(0.0, 1.0)
            } else {
                1.0 // Instant movement
            };

            // Interpolate position
            let current_x = x1 + (x2 - x1) * factor;
            let current_y = y1 + (y2 - y1) * factor;

            // Apply alignment offset
            let alignment = tags
                .formatting
                .alignment
                .unwrap_or(config.default_alignment);
            pos.render_x = current_x - Self::get_alignment_offset_x(alignment, bbox);
            pos.render_y = current_y - Self::get_alignment_offset_y(alignment, bbox);
            pos.explicit_position = true;
        }

        pos
    }
}

/// Handle PlayResX/PlayResY coordinate scaling
pub fn scale_coordinates(
    x: f32,
    y: f32,
    play_res_x: f32,
    play_res_y: f32,
    screen_width: f32,
    screen_height: f32,
) -> (f32, f32) {
    let scale_x = screen_width / play_res_x;
    let scale_y = screen_height / play_res_y;
    (x * scale_x, y * scale_y)
}

/// Convert SSA legacy alignment to ASS alignment
pub fn convert_ssa_alignment(ssa_align: u8) -> u8 {
    // SSA: 1=left, 2=center, 3=right
    // +0=sub, +4=title, +8=top, +128=mid-title
    let h_align = ssa_align & 3;
    let v_align = (ssa_align >> 2) & 3;

    let h_part = match h_align {
        1 => 1, // Left
        2 => 2, // Center
        3 => 3, // Right
        _ => 2, // Default center
    };

    let v_part = match v_align {
        0 => 0,     // Bottom
        1 | 3 => 3, // Middle (title or mid-title)
        2 => 6,     // Top
        _ => 0,     // Default bottom
    };

    h_part + v_part
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alignment_positions() {
        let bbox = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };

        let tags = ProcessedTags::default();

        // Test bottom-center alignment (default)
        let config = PositionConfig {
            screen_width: 1920.0,
            screen_height: 1080.0,
            margin_left: 0.0,
            margin_right: 0.0,
            margin_vertical: 0.0,
            default_alignment: 2,
        };
        let pos = PositionInfo::calculate(&tags, &bbox, &config);

        assert_eq!(pos.anchor_x, 910.0); // (1920 - 100) / 2
        assert_eq!(pos.anchor_y, 1030.0); // 1080 - 0 - 50
    }

    #[test]
    fn test_explicit_position() {
        let bbox = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };

        let tags = ProcessedTags {
            position: Some((500.0, 300.0)),
            ..ProcessedTags::default()
        };

        let config = PositionConfig {
            screen_width: 1920.0,
            screen_height: 1080.0,
            margin_left: 0.0,
            margin_right: 0.0,
            margin_vertical: 0.0,
            default_alignment: 2,
        };
        let pos = PositionInfo::calculate(&tags, &bbox, &config);

        assert!(pos.explicit_position);
        assert_eq!(pos.render_x, 450.0); // 500 - 50 (center offset)
        assert_eq!(pos.render_y, 250.0); // 300 - 50 (bottom offset)
    }

    #[test]
    fn test_movement_interpolation() {
        let bbox = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };

        let tags = ProcessedTags {
            movement: Some((100.0, 100.0, 500.0, 300.0, 0, 1000)),
            ..ProcessedTags::default()
        };

        // Test at halfway point (500ms)
        let config = PositionConfig {
            screen_width: 1920.0,
            screen_height: 1080.0,
            margin_left: 0.0,
            margin_right: 0.0,
            margin_vertical: 0.0,
            default_alignment: 2,
        };
        let pos = PositionInfo::calculate_with_movement(&tags, &bbox, &config, 500, 0);

        // Position should be interpolated halfway
        assert_eq!(pos.render_x, 250.0); // 300 - 50 (center offset)
        assert_eq!(pos.render_y, 150.0); // 200 - 50 (bottom offset)
    }
}
