//! Transform (`\t`) animation application for the software pipeline.

use crate::pipeline::{
    tag_processor::ProcessedTags,
    transform::{interpolate_alpha, interpolate_color, interpolate_f32, AnimatableTag},
};

impl super::SoftwarePipeline {
    /// Apply transform animations to tags based on current time
    pub(super) fn apply_transform_animations(
        &self,
        tags: &mut ProcessedTags,
        event_start_cs: u32,
        event_end_cs: u32,
        current_time_cs: u32,
        default_colors: ([u8; 4], [u8; 4], [u8; 4], [u8; 4]), // primary, secondary, outline, shadow
        default_font_size: f32,
    ) {
        // Event duration in milliseconds: a \t with no explicit end animates over it.
        let full_duration_ms = event_end_cs
            .saturating_sub(event_start_cs)
            .saturating_mul(10);

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
            let progress = animation.calculate_progress(relative_time_ms, full_duration_ms);

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
                                // No explicit \fs before \t: animate from the style's
                                // base size (libass), not from zero.
                                tags.font.size = Some(interpolate_f32(
                                    default_font_size,
                                    *target_size,
                                    progress,
                                ));
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
}
