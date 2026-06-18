//! Software (CPU) rendering backend using tiny-skia

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{boxed::Box, sync::Arc, vec::Vec};

use crate::backends::blur::apply_gaussian_blur;
use crate::backends::geometry::{merge_transformed, project_path_3d, stroke_outline};
use crate::backends::{BackendFeature, BackendType, RenderBackend};
use crate::pipeline::{IntermediateLayer, Pipeline, SoftwarePipeline};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};
use tiny_skia::{Pixmap, Transform};

mod cache;
mod dirty;
#[cfg(not(feature = "nostd"))]
use cache::{BlurTile, BlurTileKey, BLUR_TILES, DIRTY_BBOX, EMIT_SINK};
#[cfg(not(feature = "nostd"))]
use dirty::{clear_region, crop_pixmap, note_dirty_bbox, text_vector_dirty_bbox};

/// Software rendering backend using tiny-skia
pub struct SoftwareBackend {
    pixmap: Pixmap,
    font_database: Arc<fontdb::Database>,
    glyph_renderer: crate::pipeline::shaping::GlyphRenderer,
    /// Reused scratch pixmap into which a vector-path layer is rendered when
    /// collecting a bitmap list (`render_to_bitmaps`), then cropped to a tile.
    #[cfg(not(feature = "nostd"))]
    scratch: Pixmap,
    #[cfg(feature = "backend-metrics")]
    metrics: super::BackendMetrics,
}

impl SoftwareBackend {
    /// Create a new software backend
    pub fn new(context: &RenderContext) -> Result<Self, RenderError> {
        let pixmap =
            Pixmap::new(context.width(), context.height()).ok_or(RenderError::InvalidDimensions)?;

        // Share the process-wide, lazily-loaded system font database. A fresh
        // backend is built every frame, so re-scanning system fonts here (the old
        // behaviour) dominated frame time; cloning the shared Arc is ~free.
        #[cfg(not(feature = "nostd"))]
        let font_database = crate::pipeline::font_loader::shared_system_fonts();
        #[cfg(feature = "nostd")]
        let font_database = Arc::new(fontdb::Database::new());

        #[cfg(not(feature = "nostd"))]
        let scratch =
            Pixmap::new(context.width(), context.height()).ok_or(RenderError::InvalidDimensions)?;

        Ok(Self {
            pixmap,
            font_database,
            glyph_renderer: crate::pipeline::shaping::GlyphRenderer::new(),
            #[cfg(not(feature = "nostd"))]
            scratch,
            #[cfg(feature = "backend-metrics")]
            metrics: super::BackendMetrics::new(),
        })
    }

    /// Resize the backend pixmap
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        self.pixmap = Pixmap::new(width, height).ok_or(RenderError::InvalidDimensions)?;
        #[cfg(not(feature = "nostd"))]
        {
            self.scratch = Pixmap::new(width, height).ok_or(RenderError::InvalidDimensions)?;
        }
        Ok(())
    }

    /// Render layers into a positioned bitmap list (libass `ASS_Image` style)
    /// instead of compositing into a frame buffer.
    ///
    /// Coverage-path layers emit cheap A8 [`RenderBitmap::Coverage`] tiles (an
    /// `Arc` clone of the cached coverage); vector-path layers (blur, swept
    /// karaoke, clip, drawings) are rendered into a scratch pixmap and cropped to
    /// an [`RenderBitmap::Rgba`] tile. This skips the full-frame clear and the
    /// final copy entirely — the caller (or a GPU) composites the list.
    #[cfg(not(feature = "nostd"))]
    fn render_to_bitmaps(
        &mut self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<crate::backends::coverage::RenderBitmap>, RenderError> {
        if self.pixmap.width() != context.width() || self.pixmap.height() != context.height() {
            self.resize(context.width(), context.height())?;
        }

        // The scratch starts (and stays) clear; only vector-path layers draw into
        // it, after which it is cropped and cleared again. Coverage-path layers
        // emit into the sink and never touch it — so we avoid a per-layer clear
        // and full-frame scan, which would dwarf the bitmap emit.
        self.scratch.fill(tiny_skia::Color::TRANSPARENT);
        let mut out = Vec::new();
        for layer in layers {
            EMIT_SINK.with(|sink| *sink.borrow_mut() = Some(Vec::new()));
            DIRTY_BBOX.with(|b| *b.borrow_mut() = None);
            std::mem::swap(&mut self.pixmap, &mut self.scratch);
            let result = self.composite_layer(layer, context);
            std::mem::swap(&mut self.pixmap, &mut self.scratch);
            result?;

            let coverage = EMIT_SINK.with(|sink| sink.borrow_mut().take().unwrap_or_default());
            if coverage.is_empty() {
                // Vector / raster / drawing layer: it rendered into the scratch.
                let hint = DIRTY_BBOX.with(|b| *b.borrow());
                if let Some(bitmap) = crop_pixmap(&self.scratch, hint) {
                    // Clear only the cropped extent (all non-zero pixels lie within
                    // it) to restore a transparent scratch for the next layer,
                    // rather than memset-ing the whole frame per drawing.
                    if let crate::backends::coverage::RenderBitmap::Rgba {
                        x,
                        y,
                        width,
                        height,
                        ..
                    } = &bitmap
                    {
                        clear_region(&mut self.scratch, (*x, *y, *width, *height));
                    }
                    out.push(bitmap);
                }
            } else {
                out.extend(coverage);
            }
        }
        EMIT_SINK.with(|sink| *sink.borrow_mut() = None);
        Ok(out)
    }

    fn composite_layer(
        &mut self,
        layer: &IntermediateLayer,
        _context: &RenderContext,
    ) -> Result<(), RenderError> {
        match layer {
            IntermediateLayer::Raster(raster_data) => {
                self.draw_raster_layer(raster_data)?;
            }
            IntermediateLayer::Vector(path_data) => {
                self.draw_vector_layer(path_data)?;
            }
            IntermediateLayer::Text(text_data) => {
                self.draw_text_layer(text_data)?;
            }
        }
        Ok(())
    }

    fn draw_raster_layer(&mut self, data: &crate::pipeline::RasterData) -> Result<(), RenderError> {
        if data.pixels.len() != (data.width * data.height * 4) as usize {
            return Err(RenderError::InvalidBufferSize {
                expected: (data.width * data.height * 4) as usize,
                actual: data.pixels.len(),
            });
        }

        let src_pixmap = Pixmap::from_vec(
            data.pixels.clone(),
            tiny_skia::IntSize::from_wh(data.width, data.height)
                .ok_or(RenderError::InvalidDimensions)?,
        )
        .ok_or(RenderError::InvalidPixmap)?;

        let transform = Transform::from_translate(data.x as f32, data.y as f32);

        // Use SourceOver blend mode for proper alpha compositing
        let paint = tiny_skia::PixmapPaint {
            blend_mode: tiny_skia::BlendMode::SourceOver,
            ..Default::default()
        };

        self.pixmap
            .draw_pixmap(0, 0, src_pixmap.as_ref(), &paint, transform, None);

        Ok(())
    }

    fn draw_vector_layer(&mut self, data: &crate::pipeline::VectorData) -> Result<(), RenderError> {
        let Some(path) = &data.path else {
            return Ok(());
        };

        // Record the drawn region so `render_to_bitmaps` crops and clears only
        // this shape's bounds instead of scanning/clearing the whole frame per
        // drawing — the dominant cost on sparkle-heavy frames (dozens-to-hundreds
        // of `\p` drawings, each previously a full-frame scan + clear).
        let b = path.bounds();
        let margin = 2.0 + data.stroke.as_ref().map_or(0.0, |s| s.width) + data.blur * 3.0;
        note_dirty_bbox((
            (b.left() - margin).floor() as i32,
            (b.top() - margin).floor() as i32,
            (b.right() + margin).ceil() as i32,
            (b.bottom() + margin).ceil() as i32,
        ));

        let clip_mask = self.vector_clip_mask(data.clip);

        let mut paint = tiny_skia::Paint::default();
        // Ensure we're setting color with proper alpha handling
        // tiny-skia expects premultiplied alpha internally
        paint.set_color_rgba8(data.color[0], data.color[1], data.color[2], data.color[3]);
        paint.anti_alias = true;
        paint.blend_mode = tiny_skia::BlendMode::SourceOver;

        // Sharp drawing (no `\blur`): fill, then stroke, straight onto the frame.
        if data.blur <= 0.0 {
            self.pixmap.fill_path(
                path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                clip_mask.as_ref(),
            );
            if let Some(stroke) = &data.stroke {
                paint.set_color_rgba8(
                    stroke.color[0],
                    stroke.color[1],
                    stroke.color[2],
                    stroke.color[3],
                );
                let sk_stroke = tiny_skia::Stroke {
                    width: stroke.width,
                    ..Default::default()
                };
                self.pixmap.stroke_path(
                    path,
                    &paint,
                    &sk_stroke,
                    Transform::identity(),
                    clip_mask.as_ref(),
                );
            }
            return Ok(());
        }

        // Blurred drawing: render fill+stroke into a padded temp pixmap, blur the
        // whole silhouette, then composite (and clip). libass blurs the drawing
        // bitmap and applies `\clip` afterwards, so clipped gradient strips tile
        // seamlessly and dust/sparkle shapes spread into soft, dim glows instead
        // of hard bright dots.
        let pad = (data.blur * 3.0).ceil().max(1.0);
        let tw = ((b.width() + pad * 2.0).ceil() as u32).max(1);
        let th = ((b.height() + pad * 2.0).ceil() as u32).max(1);
        let Some(mut temp) = Pixmap::new(tw, th) else {
            return Ok(());
        };
        let off = Transform::from_translate(pad - b.left(), pad - b.top());
        temp.fill_path(path, &paint, tiny_skia::FillRule::Winding, off, None);
        if let Some(stroke) = &data.stroke {
            let mut sp = tiny_skia::Paint::default();
            sp.set_color_rgba8(
                stroke.color[0],
                stroke.color[1],
                stroke.color[2],
                stroke.color[3],
            );
            sp.anti_alias = true;
            let sk_stroke = tiny_skia::Stroke {
                width: stroke.width,
                ..Default::default()
            };
            temp.stroke_path(path, &sp, &sk_stroke, off, None);
        }
        apply_gaussian_blur(&mut temp, data.blur * (2.0 / 256.0_f32.ln().sqrt()));
        let blend = Transform::from_translate(b.left() - pad, b.top() - pad);
        self.pixmap.draw_pixmap(
            0,
            0,
            temp.as_ref(),
            &tiny_skia::PixmapPaint {
                blend_mode: tiny_skia::BlendMode::SourceOver,
                ..Default::default()
            },
            blend,
            clip_mask.as_ref(),
        );

        Ok(())
    }

    /// Build a full-canvas clip mask for a drawing's rectangular `\clip` /
    /// `\iclip` (coordinates already in render space). Mirrors the text clip in
    /// [`Self::composite_layer`]; `None` leaves the drawing unclipped.
    fn vector_clip_mask(
        &self,
        clip: Option<(f32, f32, f32, f32, bool)>,
    ) -> Option<tiny_skia::Mask> {
        let (x1, y1, x2, y2, inverse) = clip?;
        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let mut mask = tiny_skia::Mask::new(width, height)?;
        let mut builder = tiny_skia::PathBuilder::new();
        builder.move_to(x1, y1);
        builder.line_to(x2, y1);
        builder.line_to(x2, y2);
        builder.line_to(x1, y2);
        builder.close();
        let fill_rule = if inverse {
            builder.move_to(0.0, 0.0);
            builder.line_to(width as f32, 0.0);
            builder.line_to(width as f32, height as f32);
            builder.line_to(0.0, height as f32);
            builder.close();
            tiny_skia::FillRule::EvenOdd
        } else {
            tiny_skia::FillRule::Winding
        };
        let clip_path = builder.finish()?;
        // Rasterize the clip rect WITHOUT anti-aliasing. Gradient/banner effects
        // tile many abutting 2px-wide `\clip` strips of the same shape; an
        // anti-aliased clip gives each strip partial coverage at the shared
        // boundary, and SourceOver compositing under-fills there (0.33 over 0.67 =
        // 0.78, not 1.0) — leaving a dark seam at every strip edge. A binary mask
        // assigns each pixel to exactly one strip (pixel-center test), so abutting
        // strips tile into a solid fill like libass.
        mask.fill_path(&clip_path, fill_rule, false, Transform::identity());
        Some(mask)
    }

    fn draw_text_layer(&mut self, data: &crate::pipeline::TextData) -> Result<(), RenderError> {
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
            return Ok(());
        }

        // Fast path for blurred text whose bitmap is already cached: composite it
        // and skip the font lookup + glyph-path build entirely. On a blur-cache hit
        // those paths are unused (the blur branch just composites the cached tile),
        // so this is bit-identical — it just avoids the per-call font parse and
        // outline build, the dominant cost of the recurring blurred credit glyphs.
        #[cfg(not(feature = "nostd"))]
        if rot3d.is_none() && self.blur_tile_hit(data, bold, italic, baseline_y, shaped.ascent) {
            return Ok(());
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
            return Ok(());
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

        // Apply effects in order: shadow, outline, then main text. The sharp
        // shadow is skipped when \blur is active (it is folded into the blur
        // temp below so it softens together with the outline and fill).
        if blur_radius.is_none() {
            if let Some((color, x_offset, y_offset)) = shadow_info {
                let mut shadow_paint = tiny_skia::Paint::default();
                shadow_paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
                shadow_paint.anti_alias = true;
                shadow_paint.blend_mode = tiny_skia::BlendMode::SourceOver;

                let shadow_transform = base_transform.pre_translate(x_offset, y_offset);

                if let Some(merged) = merge_transformed(&paths, shadow_transform) {
                    // libass's shadow is the silhouette of the FINAL glyph (fill +
                    // border), so when there is an outline, stroke it into the shadow
                    // too. Without this the shadow is thinner than libass's by the
                    // border width (and inconsistent with the \blur branch, which
                    // already includes it).
                    if let Some((_, owx, owy)) = outline_info {
                        if let Some(stroked) = stroke_outline(&merged, owx, owy) {
                            self.pixmap.fill_path(
                                &stroked,
                                &shadow_paint,
                                tiny_skia::FillRule::Winding,
                                Transform::identity(),
                                clip_mask.as_ref(),
                            );
                        }
                    }
                    self.pixmap.fill_path(
                        &merged,
                        &shadow_paint,
                        tiny_skia::FillRule::Winding,
                        Transform::identity(),
                        clip_mask.as_ref(),
                    );
                }
            }
        }

        // Draw opaque box (BorderStyle 3) behind the text, covering the glyph
        // bounds expanded by the padding, in the outline colour.
        for effect in &data.effects {
            if let crate::pipeline::TextEffect::OpaqueBox {
                color,
                padding_x,
                padding_y,
            } = effect
            {
                let mut bounds: Option<tiny_skia::Rect> = None;
                for path in &paths {
                    if let Some(t) = path.clone().transform(base_transform) {
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
                            clip_mask.as_ref(),
                        );
                    }
                }
            }
        }

        // Check for edge blur effect (applies to outline only)
        let edge_blur_radius = data.effects.iter().find_map(|e| {
            if let crate::pipeline::TextEffect::EdgeBlur { radius } = e {
                Some(*radius)
            } else {
                None
            }
        });

        // Draw outline if present
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
                if let Some(blur_radius) = edge_blur_radius {
                    if blur_radius > 0.0 {
                        let blur_size = (blur_radius * 3.0).ceil() as u32;
                        let outline_width =
                            (shaped.width + blur_size as f32 * 2.0 + width * 2.0).ceil() as u32;
                        let outline_height =
                            (shaped.height + blur_size as f32 * 2.0 + width * 2.0).ceil() as u32;

                        if let Some(mut temp_pixmap) = Pixmap::new(outline_width, outline_height) {
                            temp_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                            // Draw outline to temporary pixmap
                            let temp_transform = Transform::from_translate(
                                blur_size as f32 + width,
                                blur_size as f32 + width,
                            );

                            for path in &paths {
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
                            let blend_transform = base_transform.pre_translate(
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
                                clip_mask.as_ref(),
                            );
                        }
                    }
                } else if blur_radius.is_none() {
                    // Draw outline using path expansion (like libass): stroke the
                    // merged glyph path once and fill the expansion, rather than
                    // stroking each glyph separately. (When \blur is active this is
                    // skipped — the outline goes into the blur temp below so it
                    // blurs together with the fill.)
                    if let Some(ref merged) = merged_base {
                        if let Some(outlined_path) = stroke_outline(merged, *width_x, *width_y) {
                            self.pixmap.fill_path(
                                &outlined_path,
                                &outline_paint,
                                tiny_skia::FillRule::Winding,
                                Transform::identity(),
                                clip_mask.as_ref(),
                            );
                        }
                    }
                }
            }
        }

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

        // Use base_transform built above with rotation/scaling
        let text_transform = base_transform;

        // Apply blur if needed
        if let Some(radius) = blur_radius {
            // Create a temporary pixmap for blurred text. The padding must contain
            // the full Gaussian kernel (~3*sigma) or the soft tail is clipped and
            // the blurred glyph loses mass at larger radii.
            let blur_size = (radius * 3.0).ceil() as u32;
            let text_width = (shaped.width + blur_size as f32 * 2.0).ceil() as u32;
            let text_height = (shaped.height + blur_size as f32 * 2.0).ceil() as u32;

            // The blurred bitmap is a pure function of the glyph outlines, blur
            // radius and baked colours (screen position is applied at composite),
            // so identical blurred glyphs reuse one bitmap. A positional `\clip`
            // makes the result position-dependent, so it is not cached.
            let cache_key = clip_mask.is_none().then(|| BlurTileKey {
                text: data.text.clone(),
                font: data.font_family.clone(),
                size: data.font_size.to_bits(),
                spacing: data.spacing.to_bits(),
                bold,
                italic,
                blur: radius.to_bits(),
                fill: data.color,
                outline: outline_info.map(|(c, wx, wy)| (wx.to_bits(), wy.to_bits(), c)),
                shadow: shadow_info.map(|(c, x, y)| (c, x.to_bits(), y.to_bits())),
            });

            let cached = cache_key
                .as_ref()
                .and_then(|k| BLUR_TILES.with(|c| c.borrow().get(k).cloned()));

            let tile = if cached.is_some() {
                cached
            } else if let Some(mut temp_pixmap) = Pixmap::new(text_width, text_height) {
                temp_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                // Draw shadow (if any) then outline then text into the temp
                // pixmap, so the box blur below softens shadow, outline and fill
                // together. The shadow goes down first as it sits behind the rest.
                //
                // The glyph paths have their origin on the baseline and rise by
                // `ascent` above it, so the baseline must sit `ascent` below the
                // temp's top (plus the blur margin) — otherwise tall glyphs are
                // clipped at the temp's top edge (only the lower part survives,
                // the bug on large blurred text like the OP/ED song).
                let temp_transform =
                    Transform::from_translate(blur_size as f32, blur_size as f32 + shaped.ascent);
                if let Some((scolor, sx, sy)) = shadow_info {
                    let mut shadow_paint = tiny_skia::Paint {
                        anti_alias: true,
                        blend_mode: tiny_skia::BlendMode::SourceOver,
                        ..Default::default()
                    };
                    shadow_paint.set_color_rgba8(scolor[0], scolor[1], scolor[2], scolor[3]);
                    let shadow_transform = temp_transform.pre_translate(sx, sy);
                    // The shadow is the silhouette of the FINAL glyph (fill +
                    // border), so when there is a border, stroke it into the
                    // shadow too. Without this a heavy `\bord` is absent from the
                    // shadow — e.g. the "Declassified" body box is a row of `b`s
                    // (BSOD block font) drawn shadow-only with `\bord12`; the 12px
                    // border is what merges them into a solid box, so a fill-only
                    // shadow collapsed it to bare glyph blobs.
                    if let Some((_, owx, owy)) = outline_info {
                        for path in &paths {
                            if let Some(t) = path.clone().transform(shadow_transform) {
                                if let Some(outlined) = stroke_outline(&t, owx, owy) {
                                    temp_pixmap.fill_path(
                                        &outlined,
                                        &shadow_paint,
                                        tiny_skia::FillRule::Winding,
                                        Transform::identity(),
                                        None,
                                    );
                                }
                            }
                        }
                    }
                    for path in &paths {
                        if let Some(transformed) = path.clone().transform(shadow_transform) {
                            temp_pixmap.fill_path(
                                &transformed,
                                &shadow_paint,
                                tiny_skia::FillRule::Winding,
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
                if let Some((ocolor, owx, owy)) = outline_info {
                    let mut outline_paint = tiny_skia::Paint {
                        anti_alias: true,
                        blend_mode: tiny_skia::BlendMode::SourceOver,
                        ..Default::default()
                    };
                    outline_paint.set_color_rgba8(ocolor[0], ocolor[1], ocolor[2], ocolor[3]);
                    for path in &paths {
                        if let Some(transformed) = path.clone().transform(temp_transform) {
                            if let Some(outlined) = stroke_outline(&transformed, owx, owy) {
                                temp_pixmap.fill_path(
                                    &outlined,
                                    &outline_paint,
                                    tiny_skia::FillRule::Winding,
                                    Transform::identity(),
                                    None,
                                );
                            }
                        }
                    }
                }
                for path in &paths {
                    if let Some(transformed) = path.clone().transform(temp_transform) {
                        temp_pixmap.fill_path(
                            &transformed,
                            &text_paint,
                            tiny_skia::FillRule::Winding,
                            Transform::identity(),
                            clip_mask.as_ref(),
                        );
                    }
                }

                // `radius` is the screen-pixel \blur (scaled by blur_scale in
                // the pipeline); map it to a Gaussian std-dev via libass's
                // blur_radius_scale = 2/sqrt(ln 256).
                apply_gaussian_blur(&mut temp_pixmap, radius * (2.0 / 256.0_f32.ln().sqrt()));

                let tile = std::sync::Arc::new(BlurTile {
                    data: std::sync::Arc::new(temp_pixmap.data().to_vec()),
                    width: text_width,
                    height: text_height,
                });
                if let Some(key) = cache_key {
                    BLUR_TILES.with(|c| {
                        let mut map = c.borrow_mut();
                        // Bound memory: drop the cache wholesale if it grows large
                        // (a varied blurred scene) rather than leak.
                        if map.len() >= 512 {
                            map.clear();
                        }
                        map.insert(key, tile.clone());
                    });
                }
                Some(tile)
            } else {
                None
            };

            // Draw the (cached or freshly rendered) blurred bitmap. Use baseline_y
            // (the same vertical origin as the sharp path) so the blurred glyphs
            // land on the text rather than floating above it as a halo.
            if let Some(tile) = tile {
                if let Some(pixref) =
                    tiny_skia::PixmapRef::from_bytes(tile.data.as_slice(), tile.width, tile.height)
                {
                    // The baseline sits at `blur_size + ascent` inside the tile
                    // (see temp_transform), so offset the composite to land that
                    // baseline back on `baseline_y`.
                    let blend_transform = Transform::from_translate(
                        data.x - blur_size as f32,
                        baseline_y - blur_size as f32 - shaped.ascent,
                    );
                    let paint = tiny_skia::PixmapPaint {
                        blend_mode: tiny_skia::BlendMode::SourceOver,
                        ..Default::default()
                    };
                    self.pixmap
                        .draw_pixmap(0, 0, pixref, &paint, blend_transform, None);
                }
            }
        } else if let Some((progress, karaoke_style, karaoke_secondary)) = karaoke_info {
            // ASS karaoke colours: a syllable is the secondary colour until it is
            // "sung", then the primary colour (the layer's `data.color`).
            let primary = data.color;
            let secondary = karaoke_secondary;

            let mut paint = tiny_skia::Paint {
                anti_alias: true,
                blend_mode: tiny_skia::BlendMode::SourceOver,
                ..Default::default()
            };

            // For \kf/\K mid-syllable, compute the left-to-right sweep boundary
            // from the glyph bounds. (Skipped when a \clip is active — combining
            // the sweep with an arbitrary clip mask is left to the colour blend.)
            let sweeping = karaoke_style != 0 && progress > 0.0 && progress < 1.0;
            let sweep_bounds = if sweeping && clip_mask.is_none() {
                let mut b: Option<tiny_skia::Rect> = None;
                for path in &paths {
                    if let Some(t) = path.clone().transform(text_transform) {
                        let pb = t.bounds();
                        b = Some(match b {
                            None => pb,
                            Some(acc) => tiny_skia::Rect::from_ltrb(
                                acc.left().min(pb.left()),
                                acc.top().min(pb.top()),
                                acc.right().max(pb.right()),
                                acc.bottom().max(pb.bottom()),
                            )
                            .unwrap_or(acc),
                        });
                    }
                }
                b
            } else {
                None
            };

            if let Some(b) = sweep_bounds {
                // Secondary base across the whole syllable.
                paint.set_color_rgba8(secondary[0], secondary[1], secondary[2], secondary[3]);
                for path in &paths {
                    if let Some(t) = path.clone().transform(text_transform) {
                        self.pixmap.fill_path(
                            &t,
                            &paint,
                            tiny_skia::FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                    }
                }
                // Primary on the already-sung left portion, clipped to the sweep rect.
                let sweep_x = b.left() + progress * (b.right() - b.left());
                if let (Some(rect), Some(mut mask)) = (
                    tiny_skia::Rect::from_ltrb(b.left(), b.top(), sweep_x, b.bottom()),
                    tiny_skia::Mask::new(self.pixmap.width(), self.pixmap.height()),
                ) {
                    let mut pb = tiny_skia::PathBuilder::new();
                    pb.push_rect(rect);
                    if let Some(rect_path) = pb.finish() {
                        mask.fill_path(
                            &rect_path,
                            tiny_skia::FillRule::Winding,
                            true,
                            Transform::identity(),
                        );
                        paint.set_color_rgba8(primary[0], primary[1], primary[2], primary[3]);
                        for path in &paths {
                            if let Some(t) = path.clone().transform(text_transform) {
                                self.pixmap.fill_path(
                                    &t,
                                    &paint,
                                    tiny_skia::FillRule::Winding,
                                    Transform::identity(),
                                    Some(&mask),
                                );
                            }
                        }
                    }
                }
            } else {
                // Single-colour fill: binary \k, a finished/not-started sweep, or a
                // sweep under an active \clip (approximated by a secondary->primary
                // colour blend).
                let c = if karaoke_style == 0 {
                    if progress > 0.0 {
                        primary
                    } else {
                        secondary
                    }
                } else if progress >= 1.0 {
                    primary
                } else if progress <= 0.0 {
                    secondary
                } else {
                    let lerp =
                        |s: u8, e: u8| (s as f32 * (1.0 - progress) + e as f32 * progress) as u8;
                    [
                        lerp(secondary[0], primary[0]),
                        lerp(secondary[1], primary[1]),
                        lerp(secondary[2], primary[2]),
                        primary[3],
                    ]
                };
                paint.set_color_rgba8(c[0], c[1], c[2], c[3]);
                for path in &paths {
                    if let Some(t) = path.clone().transform(text_transform) {
                        self.pixmap.fill_path(
                            &t,
                            &paint,
                            tiny_skia::FillRule::Winding,
                            Transform::identity(),
                            clip_mask.as_ref(),
                        );
                    }
                }
            }
        } else {
            // Draw without blur or karaoke: fill the merged glyph path in one pass.
            // (text_transform == base_transform, so merged_base applies directly.)
            if let Some(ref merged) = merged_base {
                self.pixmap.fill_path(
                    merged,
                    &text_paint,
                    tiny_skia::FillRule::Winding,
                    Transform::identity(),
                    clip_mask.as_ref(),
                );
            }
        }

        // Draw underline if present
        if underline {
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
                    &text_paint,
                    &stroke,
                    Transform::identity(),
                    clip_mask.as_ref(),
                );
            }
        }

        // Draw strikethrough if present
        if strikethrough {
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
                    &text_paint,
                    &stroke,
                    Transform::identity(),
                    clip_mask.as_ref(),
                );
            }
        }

        Ok(())
    }
}

/// Per-layer composite colours: `(outline, shadow (colour + screen displacement),
/// fill)`. Outline and shadow are `None` when absent.
#[cfg(not(feature = "nostd"))]
type LayerColors = (Option<[u8; 4]>, Option<([u8; 4], (i32, i32))>, [u8; 4]);

impl RenderBackend for SoftwareBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Software
    }

    fn create_pipeline(&self) -> Result<Box<dyn Pipeline>, RenderError> {
        Ok(Box::new(SoftwarePipeline::new()))
    }

    fn composite_layers(
        &mut self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        // The backend persists across frames, so the per-glyph outline cache and
        // font-data cache in `glyph_renderer` (and the pixmap allocation) survive
        // instead of being rebuilt each frame. Match the pixmap to the current
        // context size, then clear and redraw.
        if self.pixmap.width() != context.width() || self.pixmap.height() != context.height() {
            self.resize(context.width(), context.height())?;
        }

        self.pixmap.fill(tiny_skia::Color::TRANSPARENT);

        for layer in layers {
            self.composite_layer(layer, context)?;
        }

        Ok(self.pixmap.data().to_vec())
    }

    fn render_layers_to_bitmaps(
        &mut self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<crate::backends::coverage::RenderBitmap>, RenderError> {
        self.render_to_bitmaps(layers, context)
    }

    fn composite_layers_incremental(
        &mut self,
        layers: &[IntermediateLayer],
        dirty_regions: &[DirtyRegion],
        previous_frame: &[u8],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        if self.pixmap.width() != context.width() || self.pixmap.height() != context.height() {
            self.resize(context.width(), context.height())?;
        }

        // Seed from the previous frame, then redraw only the dirty regions.
        if previous_frame.len() == self.pixmap.data().len() {
            self.pixmap.data_mut().copy_from_slice(previous_frame);
        } else {
            self.pixmap.fill(tiny_skia::Color::TRANSPARENT);
        }

        // Only redraw dirty regions
        for region in dirty_regions {
            // TODO: Create clip mask for dirty region
            // tiny_skia doesn't expose ClipMask publicly
            let _ = region; // TODO: Apply clipping

            // Composite layers within this region
            for layer in layers {
                if layer.intersects_region(region) {
                    self.composite_layer(layer, context)?;
                }
            }
        }

        Ok(self.pixmap.data().to_vec())
    }

    fn supports_feature(&self, feature: BackendFeature) -> bool {
        match feature {
            BackendFeature::IncrementalRendering => true,
            BackendFeature::HardwareAcceleration => false,
            BackendFeature::ComputeShaders => false,
            BackendFeature::AsyncRendering => false,
        }
    }

    #[cfg(feature = "backend-metrics")]
    fn metrics(&self) -> Option<super::BackendMetrics> {
        Some(self.metrics.clone())
    }
}
