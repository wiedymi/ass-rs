//! Decoration passes for the software text layer: sharp shadow, opaque box,
//! outline (with optional `\be` edge blur) and underline/strikethrough.

use tiny_skia::{Pixmap, Transform};

use crate::backends::blur::apply_gaussian_blur;
use crate::backends::geometry::{merge_transformed, stroke_outline};
use crate::pipeline::TextData;

use super::TextRun;

impl super::super::SoftwareBackend {
    /// Sharp shadow pass: fill the shadow silhouette (offset glyphs, plus the
    /// stroked border so it matches libass's final-glyph silhouette). Skipped
    /// when `\blur` is active — the shadow is folded into the blur temp instead.
    pub(super) fn draw_text_shadow(&mut self, run: &TextRun) {
        if run.blur_radius.is_none() {
            if let Some((color, x_offset, y_offset)) = run.shadow_info {
                let mut shadow_paint = tiny_skia::Paint::default();
                shadow_paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
                shadow_paint.anti_alias = true;
                shadow_paint.blend_mode = tiny_skia::BlendMode::SourceOver;

                let shadow_transform = run.base_transform.pre_translate(x_offset, y_offset);

                if let Some(merged) = merge_transformed(&run.paths, shadow_transform) {
                    // libass's shadow is the silhouette of the FINAL glyph (fill +
                    // border), so when there is an outline, stroke it into the shadow
                    // too. Without this the shadow is thinner than libass's by the
                    // border width (and inconsistent with the \blur branch, which
                    // already includes it).
                    if let Some((_, owx, owy)) = run.outline_info {
                        if let Some(stroked) = stroke_outline(&merged, owx, owy) {
                            self.pixmap.fill_path(
                                &stroked,
                                &shadow_paint,
                                tiny_skia::FillRule::Winding,
                                Transform::identity(),
                                run.clip_mask.as_ref(),
                            );
                        }
                    }
                    self.pixmap.fill_path(
                        &merged,
                        &shadow_paint,
                        tiny_skia::FillRule::Winding,
                        Transform::identity(),
                        run.clip_mask.as_ref(),
                    );
                }
            }
        }
    }

    /// Opaque-box pass (`BorderStyle: 3`): fill the glyph bounds expanded by the
    /// per-axis padding, in the outline colour, behind the text.
    pub(super) fn draw_opaque_box(&mut self, data: &TextData, run: &TextRun) {
        for effect in &data.effects {
            if let crate::pipeline::TextEffect::OpaqueBox {
                color,
                padding_x,
                padding_y,
            } = effect
            {
                let mut bounds: Option<tiny_skia::Rect> = None;
                for path in &run.paths {
                    if let Some(t) = path.clone().transform(run.base_transform) {
                        let b = t.bounds();
                        bounds = Some(match bounds {
                            None => b,
                            Some(acc) => tiny_skia::Rect::from_ltrb(
                                acc.left().min(b.left()),
                                acc.top().min(b.top()),
                                acc.right().max(b.right()),
                                acc.bottom().max(b.bottom()),
                            )
                            .unwrap_or(acc),
                        });
                    }
                }
                if let Some(b) = bounds {
                    if let Some(rect) = tiny_skia::Rect::from_ltrb(
                        b.left() - *padding_x,
                        b.top() - *padding_y,
                        b.right() + *padding_x,
                        b.bottom() + *padding_y,
                    ) {
                        let mut box_paint = tiny_skia::Paint::default();
                        box_paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
                        box_paint.anti_alias = true;
                        box_paint.blend_mode = tiny_skia::BlendMode::SourceOver;
                        self.pixmap.fill_rect(
                            rect,
                            &box_paint,
                            Transform::identity(),
                            run.clip_mask.as_ref(),
                        );
                    }
                }
            }
        }
    }

    /// Outline pass (`\bord`): stroke the merged glyph path once and fill the
    /// expansion, or — for `\be` edge blur — render the outline into a padded temp
    /// pixmap, soften it and composite. Skipped when `\blur` is active (the outline
    /// goes into the blur temp below so it blurs together with the fill).
    pub(super) fn draw_text_outline(&mut self, data: &TextData, run: &TextRun) {
        for effect in &data.effects {
            if let crate::pipeline::TextEffect::Outline {
                color,
                width_x,
                width_y,
            } = effect
            {
                let mut outline_paint = tiny_skia::Paint::default();
                outline_paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
                outline_paint.anti_alias = true;
                outline_paint.blend_mode = tiny_skia::BlendMode::SourceOver;

                // `width` (the larger axis) sizes the temp pixmap and offsets; the
                // stroke itself grows per-axis via stroke_outline.
                let width = width_x.max(*width_y);

                // If edge blur is needed, render outline to temporary pixmap first
                if let Some(blur_radius) = run.edge_blur_radius {
                    if blur_radius > 0.0 {
                        let blur_size = (blur_radius * 3.0).ceil() as u32;
                        let outline_width =
                            (run.shaped.width + blur_size as f32 * 2.0 + width * 2.0).ceil() as u32;
                        let outline_height =
                            (run.shaped.height + blur_size as f32 * 2.0 + width * 2.0).ceil()
                                as u32;

                        if let Some(mut temp_pixmap) = Pixmap::new(outline_width, outline_height) {
                            temp_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                            // Draw outline to temporary pixmap
                            let temp_transform = Transform::from_translate(
                                blur_size as f32 + width,
                                blur_size as f32 + width,
                            );

                            for path in &run.paths {
                                if let Some(transformed) = path.clone().transform(temp_transform) {
                                    if let Some(outlined_path) =
                                        stroke_outline(&transformed, *width_x, *width_y)
                                    {
                                        temp_pixmap.fill_path(
                                            &outlined_path,
                                            &outline_paint,
                                            tiny_skia::FillRule::Winding,
                                            Transform::identity(),
                                            None,
                                        );
                                    }
                                }
                            }

                            // Edge softening (\be): a gentler 1/sqrt(ln 256)
                            // std-dev keeps it a thin outline blur, not a halo.
                            apply_gaussian_blur(
                                &mut temp_pixmap,
                                blur_radius * (1.0 / 256.0_f32.ln().sqrt()),
                            );

                            // Draw blurred outline to main pixmap
                            let blend_transform = run.base_transform.pre_translate(
                                -(blur_size as f32) - width,
                                -(blur_size as f32) - width,
                            );

                            let paint = tiny_skia::PixmapPaint {
                                blend_mode: tiny_skia::BlendMode::SourceOver,
                                ..Default::default()
                            };

                            self.pixmap.draw_pixmap(
                                0,
                                0,
                                temp_pixmap.as_ref(),
                                &paint,
                                blend_transform,
                                run.clip_mask.as_ref(),
                            );
                        }
                    }
                } else if run.blur_radius.is_none() {
                    // Draw outline using path expansion (like libass): stroke the
                    // merged glyph path once and fill the expansion, rather than
                    // stroking each glyph separately. (When \blur is active this is
                    // skipped — the outline goes into the blur temp below so it
                    // blurs together with the fill.)
                    if let Some(ref merged) = run.merged_base {
                        if let Some(outlined_path) = stroke_outline(merged, *width_x, *width_y) {
                            self.pixmap.fill_path(
                                &outlined_path,
                                &outline_paint,
                                tiny_skia::FillRule::Winding,
                                Transform::identity(),
                                run.clip_mask.as_ref(),
                            );
                        }
                    }
                }
            }
        }
    }

    /// Underline / strikethrough decoration pass, positioned from the baseline
    /// using libass's offsets and stroked in the primary colour.
    pub(super) fn draw_text_decorations(&mut self, data: &TextData, run: &TextRun) {
        let shaped = &run.shaped;
        let baseline_y = run.baseline_y;
        let text_paint = &run.text_paint;
        let clip_mask = run.clip_mask.as_ref();

        // Draw underline if present
        if run.underline {
            // Position underline according to libass formula: baseline + descent/2
            // baseline_y is already calculated as data.y + (*shaped).baseline
            // descent is negative, so we need to subtract half of its absolute value
            let underline_y = baseline_y - shaped.descent / 2.0;
            let mut builder = tiny_skia::PathBuilder::new();
            builder.move_to(data.x, underline_y);
            builder.line_to(data.x + shaped.width, underline_y);

            if let Some(underline_path) = builder.finish() {
                let stroke = tiny_skia::Stroke {
                    width: data.font_size * 0.08,
                    line_cap: tiny_skia::LineCap::Round,
                    ..Default::default()
                };
                self.pixmap.stroke_path(
                    &underline_path,
                    text_paint,
                    &stroke,
                    Transform::identity(),
                    clip_mask,
                );
            }
        }

        // Draw strikethrough if present
        if run.strikethrough {
            // Position strikethrough according to libass formula: baseline - ascent/3
            // baseline_y is already calculated as data.y + (*shaped).baseline
            let strike_y = baseline_y - shaped.ascent / 3.0;
            let mut builder = tiny_skia::PathBuilder::new();
            builder.move_to(data.x, strike_y);
            builder.line_to(data.x + shaped.width, strike_y);

            if let Some(strike_path) = builder.finish() {
                let stroke = tiny_skia::Stroke {
                    width: data.font_size * 0.06,
                    line_cap: tiny_skia::LineCap::Round,
                    ..Default::default()
                };
                self.pixmap.stroke_path(
                    &strike_path,
                    text_paint,
                    &stroke,
                    Transform::identity(),
                    clip_mask,
                );
            }
        }
    }
}
