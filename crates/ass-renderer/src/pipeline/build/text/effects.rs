//! Per-run text effects for the software pipeline: formatting, outline/shadow/
//! blur/opaque-box, rotation, shear, scale, clip, and baseline offset.

use super::super::OwnedStyle;
use super::{EffectColors, RunCtx, TextDefaults};
use crate::pipeline::{tag_processor::ProcessedTags, TextData, TextEffect};

impl super::super::SoftwarePipeline {
    /// Push the per-run effects (formatting, outline/shadow/blur/opaque-box,
    /// rotation, shear, scale, clip) onto `layer` and apply any baseline offset.
    pub(super) fn push_text_effects(
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
