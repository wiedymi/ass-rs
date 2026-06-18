//! Per-run setup for the software text layer: shape the text, build the base
//! transform, take the cached coverage / blur-tile fast paths, build and project
//! the glyph paths, and resolve the clip mask, blur/outline/shadow/karaoke info,
//! merged outline and fill paint into a [`TextRun`].

use tiny_skia::Transform;

use crate::backends::geometry::{merge_transformed, project_path_3d};
use crate::pipeline::TextData;
use crate::utils::RenderError;

#[cfg(not(feature = "nostd"))]
use super::super::dirty::{note_dirty_bbox, text_vector_dirty_bbox};
use super::TextRun;

impl super::super::SoftwareBackend {
    /// Resolve the shared per-run drawing state from `data`: shape the text,
    /// build the base transform (rotation/scale/shear), take the cached coverage
    /// and blur-tile fast paths (returning `None` when a fast path already
    /// composited the layer), build the glyph paths (projecting them for
    /// `\frx`/`\fry`), and extract the clip mask, blur/outline/shadow info, merged
    /// outline, fill paint and karaoke parameters.
    pub(super) fn prepare_text_run(
        &mut self,
        data: &TextData,
    ) -> Result<Option<TextRun>, RenderError> {
        use crate::pipeline::shaping::{find_font_for_text, shape_text_cached};

        // Extract bold/italic from effects
        let bold = data
            .effects
            .iter()
            .any(|e| matches!(e, crate::pipeline::TextEffect::Bold));
        let italic = data
            .effects
            .iter()
            .any(|e| matches!(e, crate::pipeline::TextEffect::Italic));
        let underline = data
            .effects
            .iter()
            .any(|e| matches!(e, crate::pipeline::TextEffect::Underline));
        let strikethrough = data
            .effects
            .iter()
            .any(|e| matches!(e, crate::pipeline::TextEffect::Strikethrough));

        // Shape the text via the shared per-thread cache (persists across frames
        // and reuses the run the pipeline already shaped for layout).
        let shaped = shape_text_cached(
            &data.text,
            &data.font_family,
            data.font_size,
            bold,
            italic,
            &self.font_database,
        )?;

        // Build base transform with rotation and scaling
        // The data.x and data.y are the top-left corner of the text box
        // But glyphs are positioned from their baseline, so we need to adjust y by adding the baseline offset
        let baseline_y = data.y + shaped.baseline;

        let mut base_transform = Transform::from_translate(data.x, baseline_y);

        // \frx/\fry need a true projective transform; record (frx_rad, fry_rad,
        // local rotation centre) here and project the glyph paths per-point below.
        let mut rot3d: Option<(f32, f32, f32, f32)> = None;

        // Check for rotation, scaling, and shear effects
        for effect in &data.effects {
            match effect {
                crate::pipeline::TextEffect::Rotation { x, y, z, origin } => {
                    // Rotations are applied around a centre in local space. By
                    // default that is the text's own centre; `\org` overrides it with
                    // an explicit screen-space point (converted to local coords).
                    // Doing this in local space keeps the glyphs in place; the skews
                    // used to approximate \frx/\fry previously sheared around the
                    // screen origin, which flung the text off-frame.
                    // Local origin sits on the baseline (see base_transform), so the
                    // text's vertical centre is `height/2 - baseline` above it (matching
                    // the Scale effect); using `height/2` rotated about a point ~one
                    // ascent too low.
                    let (text_center_x, text_center_y) = match origin {
                        Some((ox, oy)) => (ox - data.x, oy - baseline_y),
                        None => (shaped.width / 2.0, shaped.height / 2.0 - shaped.baseline),
                    };

                    // Z rotation (true 2D rotation). tiny-skia's pre_rotate takes
                    // DEGREES and turns clockwise in screen space, but ASS `\frz` is
                    // counter-clockwise for positive angles, so negate to match libass.
                    if *z != 0.0 {
                        base_transform = base_transform
                            .pre_translate(text_center_x, text_center_y)
                            .pre_rotate(-*z)
                            .pre_translate(-text_center_x, -text_center_y);
                    }

                    // \frx/\fry are a true perspective projection (libass divides by
                    // a camera distance), which tiny-skia's affine Transform cannot
                    // express. Record the angles + the local rotation centre; the
                    // glyph paths are projected per-point below. \frz stays affine and
                    // is applied first, matching libass's RZ->RX->RY order.
                    if *x != 0.0 || *y != 0.0 {
                        rot3d = Some((
                            x * core::f32::consts::PI / 180.0,
                            y * core::f32::consts::PI / 180.0,
                            text_center_x,
                            text_center_y,
                        ));
                    }
                }
                crate::pipeline::TextEffect::Scale { x, y } => {
                    // `\fscy` is baked into the font SIZE during shaping, so the glyph
                    // arrives scaled by the y-factor on BOTH axes. libass scales the
                    // axes independently, so here we only correct the HORIZONTAL one:
                    // dividing `\fscx` by `\fscy` makes the net horizontal scale equal
                    // `\fscx` while the vertical stays `\fscy`. (For `\fscx` alone, y is
                    // 100 so this reduces to the plain x-scale; for uniform scaling the
                    // ratio is 1 and this is a no-op.)
                    let x_scale = if *y != 0.0 { *x / *y } else { *x / 100.0 };
                    if (x_scale - 1.0).abs() > 0.01 {
                        let text_center_x = shaped.width / 2.0;
                        base_transform = base_transform
                            .pre_translate(text_center_x, 0.0)
                            .pre_scale(x_scale, 1.0)
                            .pre_translate(-text_center_x, 0.0);
                    }
                }
                crate::pipeline::TextEffect::Shear { x, y } => {
                    // Apply shear (\fax/\fay) around the text centre. Shearing around
                    // the screen origin displaced the text by skew*position, shoving
                    // it far across the frame.
                    let text_center_x = shaped.width / 2.0;
                    let text_center_y = shaped.height / 2.0;
                    base_transform = base_transform
                        .pre_translate(text_center_x, text_center_y)
                        .pre_concat(Transform::from_skew(*x, *y))
                        .pre_translate(-text_center_x, -text_center_y);
                }
                _ => {}
            }
        }

        // Fast path: composite the layer from cached coverage tiles. On a cache
        // hit this skips font lookup, glyph-path building and rasterization
        // entirely, so a geometry-static animated layer (\move/\fad/colour \t)
        // costs only the composite — the lever for animation-heavy content.
        #[cfg(not(feature = "nostd"))]
        if rot3d.is_none() && self.coverage_hit(data, base_transform, baseline_y) {
            return Ok(None);
        }

        // Fast path for blurred text whose bitmap is already cached: composite it
        // and skip the font lookup + glyph-path build entirely. On a blur-cache hit
        // those paths are unused (the blur branch just composites the cached tile),
        // so this is bit-identical — it just avoids the per-call font parse and
        // outline build, the dominant cost of the recurring blurred credit glyphs.
        #[cfg(not(feature = "nostd"))]
        if rot3d.is_none() && self.blur_tile_hit(data, bold, italic, baseline_y, shaped.ascent) {
            return Ok(None);
        }

        // Cache miss (or an effect the coverage path does not handle): resolve the
        // font and build the glyph paths now.
        let font_id = find_font_for_text(
            &self.font_database,
            &data.font_family,
            bold,
            italic,
            &data.text,
        )?;
        let paths = self.glyph_renderer.render_shaped_text(
            &shaped,
            font_id,
            &self.font_database,
            data.spacing,
        )?;

        // For \frx/\fry, project the positioned glyph paths through the perspective
        // transform once and switch to an identity base transform, so the vector
        // fills below operate on the already-projected screen-space outlines. The
        // perspective camera distance is libass's 20000 in 1/64-px units.
        #[cfg(not(feature = "nostd"))]
        let (paths, base_transform) = if let Some((frx, fry, lcx, lcy)) = rot3d {
            let mut center = [tiny_skia::Point::from_xy(lcx, lcy)];
            base_transform.map_points(&mut center);
            let dist = 20000.0 / 64.0;
            let projected: Vec<tiny_skia::Path> = paths
                .iter()
                .filter_map(|p| {
                    let screen = p.clone().transform(base_transform)?;
                    project_path_3d(&screen, frx, fry, center[0].x, center[0].y, dist)
                })
                .collect();
            (projected, Transform::identity())
        } else {
            (paths, base_transform)
        };

        // Rasterize, cache and composite the coverage. Returns false only for
        // effects the coverage path does not handle, which fall through to the
        // full vector path below. Skipped for 3D (the projected paths are not
        // representable by the affine coverage cache).
        #[cfg(not(feature = "nostd"))]
        if rot3d.is_none() && self.rasterize_coverage_miss(data, &paths, base_transform, baseline_y)
        {
            return Ok(None);
        }

        // Vector path (blur / swept karaoke / clip / opaque box / under-strike).
        // Record a generous dirty bbox so a bitmap-list crop scans only this region.
        #[cfg(not(feature = "nostd"))]
        if let Some(bbox) = text_vector_dirty_bbox(data, &paths, base_transform) {
            note_dirty_bbox(bbox);
        }

        // Create clip mask if needed
        let clip_mask = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::Clip {
                x1,
                y1,
                x2,
                y2,
                inverse,
            } = e
            {
                // Create a mask for clipping
                let width = self.pixmap.width();
                let height = self.pixmap.height();

                if let Some(mut mask) = tiny_skia::Mask::new(width, height) {
                    let mut builder = tiny_skia::PathBuilder::new();
                    // The clip rectangle itself.
                    builder.move_to(*x1, *y1);
                    builder.line_to(*x2, *y1);
                    builder.line_to(*x2, *y2);
                    builder.line_to(*x1, *y2);
                    builder.close();

                    // For \iclip, also add a full-canvas rectangle and use the
                    // even-odd rule so coverage is left *outside* the clip rect
                    // (the rect ends up with winding count 2 => uncovered).
                    let fill_rule = if *inverse {
                        builder.move_to(0.0, 0.0);
                        builder.line_to(width as f32, 0.0);
                        builder.line_to(width as f32, height as f32);
                        builder.line_to(0.0, height as f32);
                        builder.close();
                        tiny_skia::FillRule::EvenOdd
                    } else {
                        tiny_skia::FillRule::Winding
                    };

                    if let Some(clip_path) = builder.finish() {
                        mask.fill_path(&clip_path, fill_rule, true, Transform::identity());
                        return Some(mask);
                    }
                }
                None
            } else {
                None
            }
        });

        // \blur radius, outline and shadow, detected up front. When \blur is
        // active the outline and shadow are rendered into the blur temp (below)
        // and softened together with the fill, rather than drawn sharp here.
        let blur_radius = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::Blur { radius } = e {
                Some(*radius)
            } else {
                None
            }
        });
        let outline_info = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::Outline {
                color,
                width_x,
                width_y,
            } = e
            {
                Some((*color, *width_x, *width_y))
            } else {
                None
            }
        });
        let shadow_info = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::Shadow {
                color,
                x_offset,
                y_offset,
            } = e
            {
                Some((*color, *x_offset, *y_offset))
            } else {
                None
            }
        });

        // Merge the layer's positioned glyph outlines once (sharp/non-\blur path):
        // the sharp shadow, outline and main fill each rasterize this single path
        // in one pass instead of looping per glyph — the dominant per-frame cost on
        // animated scenes. The \blur branch keeps its own per-glyph temp pixmap.
        let merged_base = if blur_radius.is_none() {
            merge_transformed(&paths, base_transform)
        } else {
            None
        };

        // Check for edge blur effect (applies to outline only)
        let edge_blur_radius = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::EdgeBlur { radius } = e {
                Some(*radius)
            } else {
                None
            }
        });

        // Draw main text
        let mut text_paint = tiny_skia::Paint::default();
        text_paint.set_color_rgba8(data.color[0], data.color[1], data.color[2], data.color[3]);
        text_paint.anti_alias = true;
        text_paint.blend_mode = tiny_skia::BlendMode::SourceOver;

        // Check for karaoke effect
        let karaoke_info = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::Karaoke {
                progress,
                style,
                secondary,
            } = e
            {
                Some((*progress, *style, *secondary))
            } else {
                None
            }
        });

        Ok(Some(TextRun {
            paths,
            base_transform,
            baseline_y,
            shaped,
            clip_mask,
            blur_radius,
            outline_info,
            shadow_info,
            edge_blur_radius,
            karaoke_info,
            merged_base,
            text_paint,
            bold,
            italic,
            underline,
            strikethrough,
        }))
    }
}
