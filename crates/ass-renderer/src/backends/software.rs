//! Software (CPU) rendering backend using tiny-skia

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, format, sync::Arc, vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{boxed::Box, sync::Arc, vec::Vec};

use crate::backends::{BackendFeature, BackendType, RenderBackend};
use crate::pipeline::{IntermediateLayer, Pipeline, SoftwarePipeline};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};
use tiny_skia::{Pixmap, Transform};

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
        let margin = 2.0 + data.stroke.as_ref().map_or(0.0, |s| s.width);
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
        mask.fill_path(&clip_path, fill_rule, true, Transform::identity());
        Some(mask)
    }

    /// Render a text layer from A8 coverage tiles (shadow, outline, fill) and
    /// return `true`, or `false` when the layer uses an effect this fast path
    /// does not cover (blur, edge blur, karaoke, clip, opaque box, underline,
    /// strikethrough) so the caller falls back to the full vector path.
    ///
    /// `base_transform` already bakes rotation/scale/shear, so the rasterized
    /// coverage depends only on the layer geometry — which is what makes the
    /// per-frame work for unchanged geometry a cheap composite rather than a
    /// re-rasterization.
    /// Composite a text layer from cached coverage tiles WITHOUT shaping or
    /// building glyph paths. Returns `true` on a cache hit; `false` if the layer
    /// is ineligible or its geometry is not cached yet (the caller then builds
    /// the paths and calls [`Self::rasterize_coverage_miss`]). This is the path
    /// that makes geometry-static animated layers nearly free: the per-frame work
    /// is just the composite.
    #[cfg(not(feature = "nostd"))]
    fn coverage_hit(
        &mut self,
        data: &crate::pipeline::TextData,
        base_transform: Transform,
        baseline_y: f32,
    ) -> bool {
        let Some((key, outline, shadow, local, fill_color, karaoke_sweep)) =
            coverage_key(data, base_transform, baseline_y)
        else {
            return false;
        };
        let shadow_paint = shadow.map(|(c, sx, sy)| (c, shadow_delta(local, sx, sy)));
        let anchor_x = data.x.round() as i32;
        let anchor_y = baseline_y.round() as i32;
        let pixmap_w = self.pixmap.width();
        let pixmap_h = self.pixmap.height();
        let dst = self.pixmap.data_mut();
        RUN_COVERAGE.with(|cache| {
            let map = cache.borrow();
            let Some(cached) = map.get(&key) else {
                return false;
            };
            composite_cached(
                dst,
                pixmap_w,
                pixmap_h,
                cached,
                (anchor_x, anchor_y),
                (outline.map(|(c, _, _)| c), shadow_paint, fill_color),
                karaoke_sweep,
            );
            true
        })
    }

    /// Rasterize a text layer's coverage from already-built `paths`, cache it,
    /// and composite it. Returns `true` if the coverage path handled the layer,
    /// or `false` if it is ineligible and must take the full vector path.
    #[cfg(not(feature = "nostd"))]
    fn rasterize_coverage_miss(
        &mut self,
        data: &crate::pipeline::TextData,
        paths: &[tiny_skia::Path],
        base_transform: Transform,
        baseline_y: f32,
    ) -> bool {
        let Some((key, outline, shadow, local, fill_color, karaoke_sweep)) =
            coverage_key(data, base_transform, baseline_y)
        else {
            return false;
        };

        RUN_COVERAGE.with(|cache| {
            if !cache.borrow().contains_key(&key) {
                let cached =
                    rasterize_run_coverage(paths, local, outline.map(|(_, wx, wy)| (wx, wy)));
                let mut map = cache.borrow_mut();
                // Bound memory: continuously geometry-animated layers produce a
                // fresh key every frame, so drop the cache when it grows large
                // rather than leak; geometry-static layers re-cache once after.
                if map.len() >= 256 {
                    map.clear();
                }
                map.insert(key.clone(), cached);
            }
        });

        let shadow_paint = shadow.map(|(c, sx, sy)| (c, shadow_delta(local, sx, sy)));
        let anchor_x = data.x.round() as i32;
        let anchor_y = baseline_y.round() as i32;
        let pixmap_w = self.pixmap.width();
        let pixmap_h = self.pixmap.height();
        let dst = self.pixmap.data_mut();
        RUN_COVERAGE.with(|cache| {
            let map = cache.borrow();
            if let Some(cached) = map.get(&key) {
                composite_cached(
                    dst,
                    pixmap_w,
                    pixmap_h,
                    cached,
                    (anchor_x, anchor_y),
                    (outline.map(|(c, _, _)| c), shadow_paint, fill_color),
                    karaoke_sweep,
                );
            }
        });
        true
    }

    /// Composite a blurred text layer directly from the cached blurred bitmap
    /// (see [`BlurTileKey`]), skipping the font lookup and glyph-path build.
    /// Returns `true` on a hit. Eligible only when the cached bitmap is the
    /// layer's *entire* output: `\blur` is present and every effect is one the
    /// bitmap captures (outline/shadow) or that the blur branch ignores
    /// (bold/italic/rotation/scale/shear). A clip, opaque box, underline,
    /// strikethrough, edge blur or karaoke draws beyond the tile, so those layers
    /// fall through to the full path.
    #[cfg(not(feature = "nostd"))]
    fn blur_tile_hit(
        &mut self,
        data: &crate::pipeline::TextData,
        bold: bool,
        italic: bool,
        baseline_y: f32,
        ascent: f32,
    ) -> bool {
        use crate::pipeline::TextEffect;

        let eligible = data.effects.iter().all(|e| {
            matches!(
                e,
                TextEffect::Blur { .. }
                    | TextEffect::Outline { .. }
                    | TextEffect::Shadow { .. }
                    | TextEffect::Bold
                    | TextEffect::Italic
                    | TextEffect::Rotation { .. }
                    | TextEffect::Scale { .. }
                    | TextEffect::Shear { .. }
            )
        });
        if !eligible {
            return false;
        }

        // Extract blur/outline/shadow with the same first-match semantics the blur
        // branch uses, so the key is identical to the one it stored.
        let Some(radius) = data.effects.iter().find_map(|e| match e {
            TextEffect::Blur { radius } => Some(*radius),
            _ => None,
        }) else {
            return false;
        };
        let outline_info = data.effects.iter().find_map(|e| match e {
            TextEffect::Outline {
                color,
                width_x,
                width_y,
            } => Some((*color, *width_x, *width_y)),
            _ => None,
        });
        let shadow_info = data.effects.iter().find_map(|e| match e {
            TextEffect::Shadow {
                color,
                x_offset,
                y_offset,
            } => Some((*color, *x_offset, *y_offset)),
            _ => None,
        });

        let key = BlurTileKey {
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
        };
        let Some(tile) = BLUR_TILES.with(|c| c.borrow().get(&key).cloned()) else {
            return false;
        };

        let blur_size = (radius * 3.0).ceil();
        let x = data.x - blur_size;
        // The tile's baseline sits at `blur_size + ascent` from its top (see the
        // blur branch's temp_transform), so the tile origin lands here.
        let y = baseline_y - blur_size - ascent;

        // Bitmap-list mode: the cached tile IS this layer's entire output (the
        // eligibility check guarantees nothing else is drawn), so emit it directly
        // as a positioned bitmap — skipping the full-frame scratch render + crop +
        // clear the generic vector path would do. `composite_rgba` places it at
        // integer (x, y); the sharp `\blur` path's nearest-filter `draw_pixmap`
        // lands at the same rounded position, so this stays frame-equivalent.
        let emitted = EMIT_SINK.with(|sink| {
            if let Some(list) = sink.borrow_mut().as_mut() {
                list.push(crate::backends::coverage::RenderBitmap::Rgba {
                    width: tile.width,
                    height: tile.height,
                    pixels: tile.data.clone(),
                    x: x.round() as i32,
                    y: y.round() as i32,
                });
                true
            } else {
                false
            }
        });
        if emitted {
            return true;
        }

        // Composite mode: draw the tile into the frame at the same fractional
        // origin and SourceOver blend the blur branch uses on a hit.
        if let Some(pixref) =
            tiny_skia::PixmapRef::from_bytes(tile.data.as_slice(), tile.width, tile.height)
        {
            let paint = tiny_skia::PixmapPaint {
                blend_mode: tiny_skia::BlendMode::SourceOver,
                ..Default::default()
            };
            self.pixmap
                .draw_pixmap(0, 0, pixref, &paint, Transform::from_translate(x, y), None);
        }
        true
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

                            // Apply blur to the outline
                            apply_gaussian_blur(&mut temp_pixmap, blur_radius);

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

                // Apply simple box blur
                apply_gaussian_blur(&mut temp_pixmap, radius);

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

/// Cache key for a text layer's coverage tiles: everything that determines the
/// rasterized *geometry* (glyph shapes + the position-independent transform +
/// outline width + shadow offset), but NOT colour, alpha or screen position —
/// those are applied when the cached tiles are composited, so a layer whose
/// geometry is unchanged between frames (e.g. `\move`, `\fad`, colour `\t`,
/// karaoke colour) reuses its tiles instead of re-rasterizing.
#[cfg(not(feature = "nostd"))]
#[derive(Clone, PartialEq, Eq, Hash)]
struct RunCoverageKey {
    text: String,
    font: String,
    size: u32,
    spacing: u32,
    bold: bool,
    italic: bool,
    outline: Option<(u32, u32)>,
    shadow: Option<(u32, u32)>,
    transform: [u32; 6],
}

/// Cache key for a blurred text layer's pre-composited bitmap. The `\blur` branch
/// rasterizes shadow + outline + fill into a temp pixmap and box-blurs it; that
/// result depends only on the glyph outlines, the blur radius and the baked
/// colours — NOT screen position (applied at composite). So the recurring letters
/// of blurred credit text reuse one rasterized+blurred bitmap, as libass does.
/// Colours are part of the key because they are baked into the bitmap.
#[cfg(not(feature = "nostd"))]
#[derive(Clone, PartialEq, Eq, Hash)]
struct BlurTileKey {
    text: String,
    font: String,
    size: u32,
    spacing: u32,
    bold: bool,
    italic: bool,
    blur: u32,
    fill: [u8; 4],
    outline: Option<(u32, u32, [u8; 4])>,
    shadow: Option<([u8; 4], u32, u32)>,
}

/// A cached blurred-text bitmap: premultiplied RGBA. The pixels live in an `Arc`
/// so a cache hit can emit them as a [`RenderBitmap::Rgba`] with a cheap clone.
#[cfg(not(feature = "nostd"))]
struct BlurTile {
    data: Arc<Vec<u8>>,
    width: u32,
    height: u32,
}

/// Rasterized coverage tiles for one text layer, in position-independent local
/// space. Each entry is the A8 tile plus its `(x, y)` offset from the layer
/// anchor, so compositing happens at `anchor + offset`. The shadow is not stored
/// separately: it is the fill shape, so it reuses the fill tile composited at an
/// offset (see [`composite_cached`]).
#[cfg(not(feature = "nostd"))]
struct CachedCoverage {
    fill: Option<(crate::backends::coverage::CoverageTile, i32, i32)>,
    outline: Option<(crate::backends::coverage::CoverageTile, i32, i32)>,
}

/// Rasterize a layer's fill and outline coverage in local space.
#[cfg(not(feature = "nostd"))]
fn rasterize_run_coverage(
    paths: &[tiny_skia::Path],
    local: Transform,
    outline_width: Option<(f32, f32)>,
) -> CachedCoverage {
    use crate::backends::coverage::CoverageTile;

    // Merge the per-glyph paths once; both the fill and the outline derive from
    // the same merged outline (previously merged twice, redundantly).
    let merged = merge_transformed(paths, local);
    let fill = merged.as_ref().and_then(CoverageTile::rasterize);
    let outline = outline_width
        .zip(merged.as_ref())
        .and_then(|((wx, wy), merged)| {
            // libass grows the glyph outward by the per-axis border (\xbord/\ybord);
            // stroke_outline produces that (uniform for the symmetric case).
            let outlined = stroke_outline(merged, wx, wy)?;
            CoverageTile::rasterize(&outlined)
        });
    CachedCoverage { fill, outline }
}

/// Screen-space shadow displacement for a local-space `(sx, sy)` offset under the
/// layer transform's linear part — so the fill tile can be reused as the shadow.
#[cfg(not(feature = "nostd"))]
fn shadow_delta(local: Transform, sx: f32, sy: f32) -> (i32, i32) {
    (
        (sx * local.sx + sy * local.kx).round() as i32,
        (sx * local.ky + sy * local.sy).round() as i32,
    )
}

// Per-thread cache of rasterized coverage tiles, shared by the hit and miss
// paths and persistent across frames.
#[cfg(not(feature = "nostd"))]
std::thread_local! {
    static RUN_COVERAGE: std::cell::RefCell<std::collections::HashMap<RunCoverageKey, CachedCoverage>> =
        std::cell::RefCell::new(std::collections::HashMap::new());

    /// Per-thread cache of blurred text bitmaps (see [`BlurTileKey`]), persistent
    /// across frames so recurring blurred glyphs are rasterized+blurred once.
    static BLUR_TILES: std::cell::RefCell<std::collections::HashMap<BlurTileKey, std::sync::Arc<BlurTile>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());

    /// When `Some`, coverage-path layers append their bitmaps here instead of
    /// compositing — this is how `render_to_bitmaps` collects the libass-style
    /// output while reusing the normal layer-rendering code unchanged.
    static EMIT_SINK: std::cell::RefCell<Option<Vec<crate::backends::coverage::RenderBitmap>>> =
        const { std::cell::RefCell::new(None) };

    /// A generous screen-space `(min_x, min_y, max_x, max_y)` of a vector layer's
    /// drawn pixels, set during `render_to_bitmaps` so the scratch crop scans only
    /// that region instead of the whole (4K) frame.
    static DIRTY_BBOX: std::cell::RefCell<Option<(i32, i32, i32, i32)>> =
        const { std::cell::RefCell::new(None) };
}

/// A generous screen-space bbox covering a text layer's vector-path output
/// (glyphs plus an outline/shadow/blur margin), or `None` if it has no geometry.
#[cfg(not(feature = "nostd"))]
fn text_vector_dirty_bbox(
    data: &crate::pipeline::TextData,
    paths: &[tiny_skia::Path],
    base_transform: Transform,
) -> Option<(i32, i32, i32, i32)> {
    use crate::pipeline::TextEffect;
    let bounds = merge_transformed(paths, base_transform)?.bounds();
    let (mut outline, mut shadow, mut blur) = (0.0_f32, 0.0_f32, 0.0_f32);
    for effect in &data.effects {
        match effect {
            TextEffect::Outline {
                width_x, width_y, ..
            } => outline = outline.max(width_x.max(*width_y)),
            TextEffect::Shadow {
                x_offset, y_offset, ..
            } => shadow = shadow.max(x_offset.abs()).max(y_offset.abs()),
            TextEffect::Blur { radius } | TextEffect::EdgeBlur { radius } => {
                blur = blur.max(*radius)
            }
            _ => {}
        }
    }
    // Generous: box blur of radius r spreads ~r each side; ×4 leaves head-room.
    let margin = 4.0 + outline * 2.0 + shadow + blur * 4.0;
    Some((
        (bounds.left() - margin).floor() as i32,
        (bounds.top() - margin).floor() as i32,
        (bounds.right() + margin).ceil() as i32,
        (bounds.bottom() + margin).ceil() as i32,
    ))
}

/// Record a generous dirty bbox for the current vector layer (used to bound the
/// scratch crop). No-op outside `render_to_bitmaps`.
#[cfg(not(feature = "nostd"))]
fn note_dirty_bbox(bbox: (i32, i32, i32, i32)) {
    DIRTY_BBOX.with(|b| {
        let mut slot = b.borrow_mut();
        *slot = Some(match *slot {
            None => bbox,
            Some((x0, y0, x1, y1)) => (
                x0.min(bbox.0),
                y0.min(bbox.1),
                x1.max(bbox.2),
                y1.max(bbox.3),
            ),
        });
    });
}

/// Build the coverage cache key (and extract the outline/shadow paints and the
/// position-independent transform) for a text layer, or `None` if the layer uses
/// an effect the coverage path does not handle (blur, edge blur, karaoke, clip,
/// opaque box, underline, strikethrough) and must take the full vector path.
#[cfg(not(feature = "nostd"))]
#[allow(clippy::type_complexity)]
fn coverage_key(
    data: &crate::pipeline::TextData,
    base_transform: Transform,
    baseline_y: f32,
) -> Option<(
    RunCoverageKey,
    Option<([u8; 4], f32, f32)>,
    Option<([u8; 4], f32, f32)>,
    Transform,
    [u8; 4],
    Option<(f32, [u8; 4])>,
)> {
    use crate::pipeline::TextEffect;

    let mut outline: Option<([u8; 4], f32, f32)> = None;
    let mut shadow: Option<([u8; 4], f32, f32)> = None;
    let mut bold = false;
    let mut italic = false;
    // The fill colour is normally the primary colour. Karaoke leaves the glyph
    // GEOMETRY unchanged, so it stays cacheable: binary `\k` just flips the whole
    // syllable's fill colour (primary once sung, else secondary), and swept
    // `\K`/`\kf` is `karaoke_sweep = (progress, secondary)` — applied at composite
    // as a secondary base plus a primary fill cropped to the sweep boundary.
    let mut fill_color = data.color;
    let mut karaoke_sweep: Option<(f32, [u8; 4])> = None;
    for effect in &data.effects {
        match effect {
            TextEffect::Outline {
                color,
                width_x,
                width_y,
            } => outline = Some((*color, *width_x, *width_y)),
            TextEffect::Shadow {
                color,
                x_offset,
                y_offset,
            } => shadow = Some((*color, *x_offset, *y_offset)),
            TextEffect::Bold => bold = true,
            TextEffect::Italic => italic = true,
            TextEffect::Rotation { .. } | TextEffect::Scale { .. } | TextEffect::Shear { .. } => {}
            TextEffect::Karaoke {
                progress,
                style,
                secondary,
            } => {
                if *style == 0 {
                    fill_color = if *progress > 0.0 {
                        data.color
                    } else {
                        *secondary
                    };
                } else {
                    karaoke_sweep = Some((*progress, *secondary));
                }
            }
            _ => return None,
        }
    }

    // Strip the screen translation so the coverage depends only on geometry and
    // can be reused at any screen position / colour.
    let local = base_transform.post_translate(-data.x, -baseline_y);
    let key = RunCoverageKey {
        text: data.text.clone(),
        font: data.font_family.clone(),
        size: data.font_size.to_bits(),
        spacing: data.spacing.to_bits(),
        bold,
        italic,
        outline: outline.map(|(_, wx, wy)| (wx.to_bits(), wy.to_bits())),
        shadow: shadow.map(|(_, x, y)| (x.to_bits(), y.to_bits())),
        transform: [
            local.sx.to_bits(),
            local.kx.to_bits(),
            local.ky.to_bits(),
            local.sy.to_bits(),
            local.tx.to_bits(),
            local.ty.to_bits(),
        ],
    };
    Some((key, outline, shadow, local, fill_color, karaoke_sweep))
}

/// Per-layer composite colours: `(outline, shadow (colour + screen displacement),
/// fill)`. Outline and shadow are `None` when absent.
#[cfg(not(feature = "nostd"))]
type LayerColors = (Option<[u8; 4]>, Option<([u8; 4], (i32, i32))>, [u8; 4]);

/// Emit a layer's cached coverage as positioned [`RenderBitmap`]s (shadow, then
/// outline, then fill), applying the current colours at the rounded screen
/// anchor. Producing each is an `Arc` clone of the cached tile, so a
/// geometry-static layer costs almost nothing — this is the libass-style output.
#[cfg(not(feature = "nostd"))]
fn emit_cached(
    cached: &CachedCoverage,
    anchor: (i32, i32),
    colors: LayerColors,
    karaoke_sweep: Option<(f32, [u8; 4])>,
) -> Vec<crate::backends::coverage::RenderBitmap> {
    use crate::backends::coverage::RenderBitmap;
    let (anchor_x, anchor_y) = anchor;
    let (outline_color, shadow, fill_color) = colors;
    let mut out = Vec::new();
    let bitmap =
        |tile: &crate::backends::coverage::CoverageTile, x: i32, y: i32, color: [u8; 4]| {
            RenderBitmap::Coverage {
                width: tile.width,
                height: tile.height,
                coverage: tile.data.clone(),
                x,
                y,
                color,
            }
        };
    // Shadow: the fill shape in the shadow colour, displaced. Reuses the fill
    // tile rather than a separately rasterized one.
    if let (Some((color, (dx, dy))), Some((tile, ox, oy))) = (shadow, &cached.fill) {
        out.push(bitmap(tile, anchor_x + ox + dx, anchor_y + oy + dy, color));
    }
    if let (Some(color), Some((tile, ox, oy))) = (outline_color, &cached.outline) {
        out.push(bitmap(tile, anchor_x + ox, anchor_y + oy, color));
    }
    if let Some((tile, ox, oy)) = &cached.fill {
        let (x, y) = (anchor_x + ox, anchor_y + oy);
        match karaoke_sweep {
            // Swept `\K`/`\kf`: not-yet-sung syllables are wholly secondary,
            // fully-sung wholly primary (both reuse the shared tile, no copy);
            // only the one syllable mid-sweep needs a secondary base plus a
            // primary fill cropped to the advancing boundary.
            Some((progress, secondary)) if progress <= 0.0 => {
                out.push(bitmap(tile, x, y, secondary));
            }
            Some((progress, _)) if progress >= 1.0 => {
                out.push(bitmap(tile, x, y, fill_color));
            }
            Some((progress, secondary)) => {
                out.push(bitmap(tile, x, y, secondary));
                let cols = (1.0 + progress * (tile.width as f32 - 2.0))
                    .round()
                    .clamp(0.0, tile.width as f32) as u32;
                if cols > 0 {
                    out.push(RenderBitmap::Coverage {
                        width: cols,
                        height: tile.height,
                        coverage: crop_coverage_columns(tile, cols),
                        x,
                        y,
                        color: fill_color,
                    });
                }
            }
            None => out.push(bitmap(tile, x, y, fill_color)),
        }
    }
    out
}

/// Copy the leftmost `cols` columns of a coverage tile into a new buffer — the
/// "sung" portion of a swept karaoke syllable.
#[cfg(not(feature = "nostd"))]
fn crop_coverage_columns(
    tile: &crate::backends::coverage::CoverageTile,
    cols: u32,
) -> std::sync::Arc<Vec<u8>> {
    let cols = cols.min(tile.width) as usize;
    let width = tile.width as usize;
    let mut data = vec![0u8; cols * tile.height as usize];
    for y in 0..tile.height as usize {
        let src = y * width;
        let dst = y * cols;
        data[dst..dst + cols].copy_from_slice(&tile.data[src..src + cols]);
    }
    std::sync::Arc::new(data)
}

/// Composite cached coverage tiles (shadow, then outline, then fill) onto the
/// premultiplied buffer at the rounded screen anchor — emits the layer's bitmaps
/// and blends them in order.
#[cfg(not(feature = "nostd"))]
fn composite_cached(
    dst: &mut [u8],
    pixmap_w: u32,
    pixmap_h: u32,
    cached: &CachedCoverage,
    anchor: (i32, i32),
    colors: LayerColors,
    karaoke_sweep: Option<(f32, [u8; 4])>,
) {
    use crate::backends::coverage::composite_bitmap;
    let bitmaps = emit_cached(cached, anchor, colors, karaoke_sweep);
    EMIT_SINK.with(|sink| {
        let mut sink = sink.borrow_mut();
        if let Some(sink) = sink.as_mut() {
            // Collecting the bitmap list: hand the layer's bitmaps over instead of
            // blending them into a frame buffer.
            sink.extend(bitmaps);
        } else {
            for bitmap in &bitmaps {
                composite_bitmap(dst, pixmap_w, pixmap_h, bitmap);
            }
        }
    });
}

/// Crop a premultiplied-RGBA pixmap to the bounding box of its non-transparent
/// pixels and return it as an `Rgba` [`RenderBitmap`], or `None` if fully empty.
#[cfg(not(feature = "nostd"))]
/// Zero a rectangular region `(x, y, width, height)` of `pixmap` (clamped to its
/// bounds). Used to restore the scratch pixmap to transparent after a vector
/// layer is cropped, clearing only the touched rectangle rather than the frame.
#[cfg(not(feature = "nostd"))]
fn clear_region(pixmap: &mut Pixmap, region: (i32, i32, u32, u32)) {
    let (rx, ry, rw, rh) = region;
    let w = pixmap.width() as i32;
    let h = pixmap.height() as i32;
    let x0 = rx.max(0);
    let y0 = ry.max(0);
    let x1 = (rx + rw as i32).min(w);
    let y1 = (ry + rh as i32).min(h);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let row_bytes = (x1 - x0) as usize * 4;
    let data = pixmap.data_mut();
    for y in y0..y1 {
        let start = (y * w + x0) as usize * 4;
        data[start..start + row_bytes].fill(0);
    }
}

#[cfg(not(feature = "nostd"))]
fn crop_pixmap(
    pixmap: &Pixmap,
    hint: Option<(i32, i32, i32, i32)>,
) -> Option<crate::backends::coverage::RenderBitmap> {
    use crate::backends::coverage::RenderBitmap;
    let w = pixmap.width() as i32;
    let h = pixmap.height() as i32;
    let data = pixmap.data();
    // Only scan the layer's (generous) dirty region — scanning the whole 4K frame
    // per vector layer is memory-bound and dominates otherwise.
    let (scan_x0, scan_y0, scan_x1, scan_y1) = match hint {
        Some((x0, y0, x1, y1)) => (x0.max(0), y0.max(0), (x1 + 1).min(w), (y1 + 1).min(h)),
        None => (0, 0, w, h),
    };
    if scan_x1 <= scan_x0 || scan_y1 <= scan_y0 {
        return None;
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (scan_x1, scan_y1, -1_i32, -1_i32);
    for y in scan_y0..scan_y1 {
        let row = (y * w) as usize * 4;
        for x in scan_x0..scan_x1 {
            if data[row + x as usize * 4 + 3] != 0 {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }
    if max_x < min_x {
        return None;
    }
    let bw = (max_x - min_x + 1) as u32;
    let bh = (max_y - min_y + 1) as u32;
    let row_bytes = bw as usize * 4;
    let mut pixels = vec![0u8; row_bytes * bh as usize];
    for ty in 0..bh as i32 {
        let src = (((min_y + ty) * w + min_x) as usize) * 4;
        let dst = ty as usize * row_bytes;
        pixels[dst..dst + row_bytes].copy_from_slice(&data[src..src + row_bytes]);
    }
    Some(RenderBitmap::Rgba {
        width: bw,
        height: bh,
        pixels: Arc::new(pixels),
        x: min_x,
        y: min_y,
    })
}

/// Merge positioned glyph outlines into a single path under one transform.
///
/// Lets a layer's glyphs be stroked and filled in one rasterizer pass instead of
/// one per glyph — the per-call setup of `fill_path`/`PathStroker` and the path
/// clones dominate per-frame cost on glyph-dense animated scenes. For
/// non-overlapping glyphs (normal text) the merged Winding fill/stroke is
/// pixel-identical to filling each glyph separately. Returns `None` if no glyph
/// produced geometry.
fn merge_transformed(paths: &[tiny_skia::Path], transform: Transform) -> Option<tiny_skia::Path> {
    let mut builder = tiny_skia::PathBuilder::new();
    for path in paths {
        if let Some(transformed) = path.clone().transform(transform) {
            builder.push_path(&transformed);
        }
    }
    builder.finish()
}

/// Stroke a glyph path to its outline, growing it outward by `wx` horizontally
/// and `wy` vertically (`\xbord`/`\ybord`). tiny-skia strokes uniformly, so an
/// asymmetric border is produced by stroking in a vertically-scaled space and
/// scaling back. The symmetric case (`wx == wy`, almost all content) is a plain
/// uniform stroke. The stroke width is doubled to match libass's outward grow.
#[cfg(not(feature = "nostd"))]
fn stroke_outline(path: &tiny_skia::Path, wx: f32, wy: f32) -> Option<tiny_skia::Path> {
    let mk = |w: f32| tiny_skia::Stroke {
        width: w * 2.0,
        line_cap: tiny_skia::LineCap::Square,
        line_join: tiny_skia::LineJoin::Miter,
        ..Default::default()
    };
    let mut stroker = tiny_skia::PathStroker::new();
    if (wx - wy).abs() < 0.05 || wx <= 0.0 || wy <= 0.0 {
        return stroker.stroke(path, &mk(wx.max(wy)), 1.0);
    }
    // Stroke uniformly with radius wx in a space scaled by (1, wx/wy), then undo
    // the scale: the vertical extent becomes wy while the horizontal stays wx.
    let sy = wx / wy;
    let scaled = path.clone().transform(Transform::from_scale(1.0, sy))?;
    let stroked = stroker.stroke(&scaled, &mk(wx), 1.0)?;
    stroked.transform(Transform::from_scale(1.0, 1.0 / sy))
}

/// Project a screen-space path through a 3D rotation (`\frx`/`\fry`) and a pinhole
/// perspective division about `(cx, cy)`, mirroring libass's transform matrix
/// (RX then RY about a camera at distance `dist`). `\frz` is applied beforehand as
/// a 2D rotation, matching libass's RZ->RX->RY order. Returns `None` if the
/// projected outline is empty.
#[cfg(not(feature = "nostd"))]
fn project_path_3d(
    path: &tiny_skia::Path,
    frx_rad: f32,
    fry_rad: f32,
    cx: f32,
    cy: f32,
    dist: f32,
) -> Option<tiny_skia::Path> {
    use tiny_skia::{PathSegment, Point};
    // libass: sx = -sin(frx), cx = cos(frx); sy = sin(fry), cy = cos(fry).
    let (sfx, cfx) = (-frx_rad.sin(), frx_rad.cos());
    let (sfy, cfy) = (fry_rad.sin(), fry_rad.cos());
    let project = |p: Point| -> Point {
        let dx = p.x - cx;
        let dy = p.y - cy;
        // The glyph starts in the z=0 plane; rotate about X then Y.
        let z3 = dy * sfx; // depth after \frx
        let y3 = dy * cfx; // y after \frx
        let x4 = dx * cfy - z3 * sfy;
        let z4 = dx * sfy + z3 * cfy;
        // Perspective divide by the camera distance (libass adds dist to z).
        let zf = (z4 + dist).max(0.1);
        Point::from_xy(cx + x4 * dist / zf, cy + y3 * dist / zf)
    };
    let mut pb = tiny_skia::PathBuilder::new();
    for seg in path.segments() {
        match seg {
            PathSegment::MoveTo(p) => {
                let q = project(p);
                pb.move_to(q.x, q.y);
            }
            PathSegment::LineTo(p) => {
                let q = project(p);
                pb.line_to(q.x, q.y);
            }
            PathSegment::QuadTo(a, b) => {
                let (qa, qb) = (project(a), project(b));
                pb.quad_to(qa.x, qa.y, qb.x, qb.y);
            }
            PathSegment::CubicTo(a, b, c) => {
                let (qa, qb, qc) = (project(a), project(b), project(c));
                pb.cubic_to(qa.x, qa.y, qb.x, qb.y, qc.x, qc.y);
            }
            PathSegment::Close => pb.close(),
        }
    }
    pb.finish()
}

/// Apply a simple box blur to a pixmap
fn apply_gaussian_blur(pixmap: &mut Pixmap, blur: f32) {
    if blur <= 0.0 {
        return;
    }

    // A true separable Gaussian, matching libass. libass maps the `\blur` value to a
    // Gaussian std-dev via blur_radius_scale = 2/sqrt(ln 256) times the
    // storage->display blur_scale (ass_render.c:2539). Calibrated against the FFI
    // oracle, the effective factor at a 1:1 render is 1/sqrt(ln 256) ~= 0.425 (the
    // blur_scale contributes the remaining ~0.5), so `\blur8` is sigma ~= 3.4px. A
    // flat box blur lowers the peak and washes the glyph centre out at larger radii;
    // the Gaussian keeps a bright centre with a soft falloff. Applied to glyph-sized
    // temp pixmaps, so the per-tap cost stays small.
    let sigma = blur * (1.0 / 256.0_f32.ln().sqrt());
    if sigma <= 0.0 {
        return;
    }
    let radius = (sigma * 3.0).ceil() as i32;
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    if radius < 1 || width == 0 || height == 0 {
        return;
    }

    // Normalised 1D Gaussian kernel.
    let inv_two_sigma_sq = 1.0 / (2.0 * sigma * sigma);
    let mut kernel = vec![0f32; (2 * radius + 1) as usize];
    let mut sum = 0f32;
    for (i, k) in kernel.iter_mut().enumerate() {
        let x = i as i32 - radius;
        let v = (-((x * x) as f32) * inv_two_sigma_sq).exp();
        *k = v;
        sum += v;
    }
    for k in &mut kernel {
        *k /= sum;
    }

    let stride = width as usize * 4;
    let data = pixmap.data_mut();
    let mut temp = vec![0u8; data.len()];

    // Horizontal pass (data -> temp).
    for y in 0..height {
        let row = y as usize * stride;
        for x in 0..width {
            let mut acc = [0f32; 4];
            for (ki, &kw) in kernel.iter().enumerate() {
                let sx = (x + ki as i32 - radius).clamp(0, width - 1) as usize;
                let i = row + sx * 4;
                for (a, &v) in acc.iter_mut().zip(&data[i..i + 4]) {
                    *a += kw * f32::from(v);
                }
            }
            let o = row + x as usize * 4;
            for (dst, &a) in temp[o..o + 4].iter_mut().zip(&acc) {
                *dst = a.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    // Vertical pass (temp -> data).
    for x in 0..width {
        let col = x as usize * 4;
        for y in 0..height {
            let mut acc = [0f32; 4];
            for (ki, &kw) in kernel.iter().enumerate() {
                let sy = (y + ki as i32 - radius).clamp(0, height - 1) as usize;
                let i = sy * stride + col;
                for (a, &v) in acc.iter_mut().zip(&temp[i..i + 4]) {
                    *a += kw * f32::from(v);
                }
            }
            let o = y as usize * stride + col;
            for (dst, &a) in data[o..o + 4].iter_mut().zip(&acc) {
                *dst = a.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

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
