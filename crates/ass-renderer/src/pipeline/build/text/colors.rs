//! Per-run colour resolution and logical-line splitting for the software
//! pipeline's text builder.

#[cfg(feature = "nostd")]
use alloc::{string::ToString, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::ToString, vec::Vec};

use crate::pipeline::{tag_processor::ProcessedTags, text_segmenter::TextSegment};

use super::types::{RunCtx, TextDefaults};

/// Split the event's tagged segments into logical lines on explicit `\N`
/// newlines, preserving per-segment tags.
pub(super) fn split_into_logical_lines(segments: Vec<TextSegment>) -> Vec<Vec<TextSegment>> {
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
pub(super) fn resolve_run_colors(
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
