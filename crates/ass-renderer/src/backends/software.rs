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

        Ok(Self {
            pixmap,
            font_database,
            glyph_renderer: crate::pipeline::shaping::GlyphRenderer::new(),
            #[cfg(feature = "backend-metrics")]
            metrics: super::BackendMetrics::new(),
        })
    }

    /// Resize the backend pixmap
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        self.pixmap = Pixmap::new(width, height).ok_or(RenderError::InvalidDimensions)?;
        Ok(())
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
        let mut paint = tiny_skia::Paint::default();
        // Ensure we're setting color with proper alpha handling
        // tiny-skia expects premultiplied alpha internally
        paint.set_color_rgba8(data.color[0], data.color[1], data.color[2], data.color[3]);
        paint.anti_alias = true;
        paint.blend_mode = tiny_skia::BlendMode::SourceOver;

        if let Some(path) = &data.path {
            self.pixmap.fill_path(
                path,
                &paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
        }

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

            if let Some(path) = &data.path {
                self.pixmap
                    .stroke_path(path, &paint, &sk_stroke, Transform::identity(), None);
            }
        }

        Ok(())
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
        let Some((key, outline, shadow, _local)) = coverage_key(data, base_transform, baseline_y)
        else {
            return false;
        };
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
                (outline.map(|(c, _)| c), shadow.map(|(c, ..)| c), data.color),
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
        let Some((key, outline, shadow, local)) = coverage_key(data, base_transform, baseline_y)
        else {
            return false;
        };

        RUN_COVERAGE.with(|cache| {
            if !cache.borrow().contains_key(&key) {
                let cached = rasterize_run_coverage(
                    paths,
                    local,
                    outline.map(|(_, w)| w),
                    shadow.map(|(_, x, y)| (x, y)),
                );
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
                    (outline.map(|(c, _)| c), shadow.map(|(c, ..)| c), data.color),
                );
            }
        });
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

                    // X rotation -> vertical skew (perspective approximation).
                    if *x != 0.0 {
                        let skew_y = (x * core::f32::consts::PI / 180.0).sin() * 0.5;
                        base_transform = base_transform
                            .pre_translate(text_center_x, text_center_y)
                            .pre_concat(Transform::from_skew(0.0, skew_y))
                            .pre_translate(-text_center_x, -text_center_y);
                    }

                    // Y rotation -> horizontal skew (perspective approximation).
                    if *y != 0.0 {
                        let skew_x = (y * core::f32::consts::PI / 180.0).sin() * 0.5;
                        base_transform = base_transform
                            .pre_translate(text_center_x, text_center_y)
                            .pre_concat(Transform::from_skew(skew_x, 0.0))
                            .pre_translate(-text_center_x, -text_center_y);
                    }
                }
                crate::pipeline::TextEffect::Scale { x, y } => {
                    // Font Y-scale is already applied to the font size during shaping
                    // But X-scale needs to be applied as a transform since fonts don't support
                    // asymmetric scaling through size alone
                    // Apply X-scale transform if it's different from Y-scale
                    let x_scale = *x / 100.0;
                    let y_scale = *y / 100.0;

                    // Apply scale transform
                    // Note: Y-scale is already partially applied to font size during shaping,
                    // but we still need to apply the transform for proper scaling
                    if (x_scale - 1.0).abs() > 0.01 || (y_scale - 1.0).abs() > 0.01 {
                        // Get the center of the text for scaling
                        let text_center_x = shaped.width / 2.0;
                        let text_center_y = shaped.height / 2.0 - shaped.baseline;

                        base_transform = base_transform
                            .pre_translate(text_center_x, text_center_y)
                            .pre_scale(x_scale, 1.0) // X-scale, Y is in font size already
                            .pre_translate(-text_center_x, -text_center_y);
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
        if self.coverage_hit(data, base_transform, baseline_y) {
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

        // Rasterize, cache and composite the coverage. Returns false only for
        // effects the coverage path does not handle, which fall through to the
        // full vector path below.
        #[cfg(not(feature = "nostd"))]
        if self.rasterize_coverage_miss(data, &paths, base_transform, baseline_y) {
            return Ok(());
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
            if let crate::pipeline::TextEffect::Outline { color, width } = e {
                Some((*color, *width))
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
            if let crate::pipeline::TextEffect::OpaqueBox { color, padding } = effect {
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
                        b.left() - *padding,
                        b.top() - *padding,
                        b.right() + *padding,
                        b.bottom() + *padding,
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
            if let crate::pipeline::TextEffect::Outline { color, width } = effect {
                let mut outline_paint = tiny_skia::Paint::default();
                outline_paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
                outline_paint.anti_alias = true;
                outline_paint.blend_mode = tiny_skia::BlendMode::SourceOver;

                // Create stroke configuration for path expansion
                let stroke = tiny_skia::Stroke {
                    width: *width * 0.6, // Further reduce width to match libass
                    line_cap: tiny_skia::LineCap::Square,
                    line_join: tiny_skia::LineJoin::Miter,
                    ..Default::default()
                };

                // If edge blur is needed, render outline to temporary pixmap first
                if let Some(blur_radius) = edge_blur_radius {
                    if blur_radius > 0.0 {
                        let blur_size = (blur_radius * 2.0).ceil() as u32;
                        let outline_width =
                            (shaped.width + blur_size as f32 * 2.0 + *width * 2.0).ceil() as u32;
                        let outline_height =
                            (shaped.height + blur_size as f32 * 2.0 + *width * 2.0).ceil() as u32;

                        if let Some(mut temp_pixmap) = Pixmap::new(outline_width, outline_height) {
                            temp_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                            // Draw outline to temporary pixmap
                            let temp_transform = Transform::from_translate(
                                blur_size as f32 + *width,
                                blur_size as f32 + *width,
                            );

                            let mut stroker = tiny_skia::PathStroker::new();
                            for path in &paths {
                                if let Some(transformed) = path.clone().transform(temp_transform) {
                                    // Expand the path to create an outline shape
                                    if let Some(outlined_path) =
                                        stroker.stroke(&transformed, &stroke, 1.0)
                                    {
                                        // Fill the expanded outline path
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
                            apply_box_blur(&mut temp_pixmap, blur_radius);

                            // Draw blurred outline to main pixmap
                            let blend_transform = base_transform.pre_translate(
                                -(blur_size as f32) - *width,
                                -(blur_size as f32) - *width,
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
                        let mut stroker = tiny_skia::PathStroker::new();
                        if let Some(outlined_path) = stroker.stroke(merged, &stroke, 1.0) {
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
            // Create a temporary pixmap for blurred text
            let blur_size = (radius * 2.0).ceil() as u32;
            let text_width = (shaped.width + blur_size as f32 * 2.0).ceil() as u32;
            let text_height = (shaped.height + blur_size as f32 * 2.0).ceil() as u32;

            if let Some(mut temp_pixmap) = Pixmap::new(text_width, text_height) {
                temp_pixmap.fill(tiny_skia::Color::TRANSPARENT);

                // Draw shadow (if any) then outline then text into the temp
                // pixmap, so the box blur below softens shadow, outline and fill
                // together. The shadow goes down first as it sits behind the rest.
                let temp_transform = Transform::from_translate(blur_size as f32, blur_size as f32);
                if let Some((scolor, sx, sy)) = shadow_info {
                    let mut shadow_paint = tiny_skia::Paint {
                        anti_alias: true,
                        blend_mode: tiny_skia::BlendMode::SourceOver,
                        ..Default::default()
                    };
                    shadow_paint.set_color_rgba8(scolor[0], scolor[1], scolor[2], scolor[3]);
                    let shadow_transform = temp_transform.pre_translate(sx, sy);
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
                if let Some((ocolor, owidth)) = outline_info {
                    let mut outline_paint = tiny_skia::Paint {
                        anti_alias: true,
                        blend_mode: tiny_skia::BlendMode::SourceOver,
                        ..Default::default()
                    };
                    outline_paint.set_color_rgba8(ocolor[0], ocolor[1], ocolor[2], ocolor[3]);
                    let stroke = tiny_skia::Stroke {
                        width: owidth * 0.6,
                        line_cap: tiny_skia::LineCap::Square,
                        line_join: tiny_skia::LineJoin::Miter,
                        ..Default::default()
                    };
                    let mut stroker = tiny_skia::PathStroker::new();
                    for path in &paths {
                        if let Some(transformed) = path.clone().transform(temp_transform) {
                            if let Some(outlined) = stroker.stroke(&transformed, &stroke, 1.0) {
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
                apply_box_blur(&mut temp_pixmap, radius);

                // Draw blurred result to main pixmap. Use baseline_y (the same
                // vertical origin as the sharp path) so the blurred glyphs land on
                // the text rather than floating above it as a halo.
                let blend_transform = Transform::from_translate(
                    data.x - blur_size as f32,
                    baseline_y - blur_size as f32,
                );
                let paint = tiny_skia::PixmapPaint {
                    blend_mode: tiny_skia::BlendMode::SourceOver,
                    ..Default::default()
                };

                self.pixmap
                    .draw_pixmap(0, 0, temp_pixmap.as_ref(), &paint, blend_transform, None);
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
    outline: Option<u32>,
    shadow: Option<(u32, u32)>,
    transform: [u32; 6],
}

/// Rasterized coverage tiles for one text layer, in position-independent local
/// space. Each entry is the A8 tile plus its `(x, y)` offset from the layer
/// anchor, so compositing happens at `anchor + offset`.
#[cfg(not(feature = "nostd"))]
struct CachedCoverage {
    fill: Option<(crate::backends::coverage::CoverageTile, i32, i32)>,
    outline: Option<(crate::backends::coverage::CoverageTile, i32, i32)>,
    shadow: Option<(crate::backends::coverage::CoverageTile, i32, i32)>,
}

/// Rasterize a layer's fill, outline and shadow coverage in local space.
#[cfg(not(feature = "nostd"))]
fn rasterize_run_coverage(
    paths: &[tiny_skia::Path],
    local: Transform,
    outline_width: Option<f32>,
    shadow_offset: Option<(f32, f32)>,
) -> CachedCoverage {
    use crate::backends::coverage::CoverageTile;

    let fill = merge_transformed(paths, local).and_then(|p| CoverageTile::rasterize(&p));
    let outline = outline_width.and_then(|width| {
        let merged = merge_transformed(paths, local)?;
        let stroke = tiny_skia::Stroke {
            width: width * 0.6,
            line_cap: tiny_skia::LineCap::Square,
            line_join: tiny_skia::LineJoin::Miter,
            ..Default::default()
        };
        let outlined = tiny_skia::PathStroker::new().stroke(&merged, &stroke, 1.0)?;
        CoverageTile::rasterize(&outlined)
    });
    let shadow = shadow_offset.and_then(|(sx, sy)| {
        let p = merge_transformed(paths, local.pre_translate(sx, sy))?;
        CoverageTile::rasterize(&p)
    });
    CachedCoverage {
        fill,
        outline,
        shadow,
    }
}

// Per-thread cache of rasterized coverage tiles, shared by the hit and miss
// paths and persistent across frames.
#[cfg(not(feature = "nostd"))]
std::thread_local! {
    static RUN_COVERAGE: std::cell::RefCell<std::collections::HashMap<RunCoverageKey, CachedCoverage>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
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
    Option<([u8; 4], f32)>,
    Option<([u8; 4], f32, f32)>,
    Transform,
)> {
    use crate::pipeline::TextEffect;

    let mut outline: Option<([u8; 4], f32)> = None;
    let mut shadow: Option<([u8; 4], f32, f32)> = None;
    let mut bold = false;
    let mut italic = false;
    for effect in &data.effects {
        match effect {
            TextEffect::Outline { color, width } => outline = Some((*color, *width)),
            TextEffect::Shadow {
                color,
                x_offset,
                y_offset,
            } => shadow = Some((*color, *x_offset, *y_offset)),
            TextEffect::Bold => bold = true,
            TextEffect::Italic => italic = true,
            TextEffect::Rotation { .. } | TextEffect::Scale { .. } | TextEffect::Shear { .. } => {}
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
        outline: outline.map(|(_, w)| w.to_bits()),
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
    Some((key, outline, shadow, local))
}

/// Composite cached coverage tiles (shadow, then outline, then fill) onto the
/// premultiplied buffer at the rounded screen anchor, applying the current
/// colours.
#[cfg(not(feature = "nostd"))]
fn composite_cached(
    dst: &mut [u8],
    pixmap_w: u32,
    pixmap_h: u32,
    cached: &CachedCoverage,
    anchor: (i32, i32),
    colors: (Option<[u8; 4]>, Option<[u8; 4]>, [u8; 4]),
) {
    use crate::backends::coverage::composite;
    let (anchor_x, anchor_y) = anchor;
    let (outline_color, shadow_color, fill_color) = colors;
    if let (Some(color), Some((tile, ox, oy))) = (shadow_color, &cached.shadow) {
        composite(
            dst,
            pixmap_w,
            pixmap_h,
            tile,
            anchor_x + ox,
            anchor_y + oy,
            color,
        );
    }
    if let (Some(color), Some((tile, ox, oy))) = (outline_color, &cached.outline) {
        composite(
            dst,
            pixmap_w,
            pixmap_h,
            tile,
            anchor_x + ox,
            anchor_y + oy,
            color,
        );
    }
    if let Some((tile, ox, oy)) = &cached.fill {
        composite(
            dst,
            pixmap_w,
            pixmap_h,
            tile,
            anchor_x + ox,
            anchor_y + oy,
            fill_color,
        );
    }
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

/// Apply a simple box blur to a pixmap
fn apply_box_blur(pixmap: &mut Pixmap, radius: f32) {
    if radius <= 0.0 {
        return;
    }

    // A single separable box blur. The oracle confirms this already tracks libass's
    // Gaussian to within the AA noise floor (<=0.54% even at radius 20); a 3-pass
    // Gaussian approximation gained ~0.02% for 3x the cost, so it is not used.
    let radius = radius.round() as i32;
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let data = pixmap.data_mut();

    // Create temporary buffer for horizontal pass
    let mut temp = vec![0u8; data.len()];

    // Horizontal blur pass
    for y in 0..height {
        for x in 0..width {
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            let mut a = 0u32;
            let mut count = 0u32;

            for dx in -radius..=radius {
                let sx = (x + dx).clamp(0, width - 1);
                let idx = ((y * width + sx) * 4) as usize;
                r += data[idx] as u32;
                g += data[idx + 1] as u32;
                b += data[idx + 2] as u32;
                a += data[idx + 3] as u32;
                count += 1;
            }

            let out_idx = ((y * width + x) * 4) as usize;
            temp[out_idx] = (r / count) as u8;
            temp[out_idx + 1] = (g / count) as u8;
            temp[out_idx + 2] = (b / count) as u8;
            temp[out_idx + 3] = (a / count) as u8;
        }
    }

    // Vertical blur pass
    for y in 0..height {
        for x in 0..width {
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            let mut a = 0u32;
            let mut count = 0u32;

            for dy in -radius..=radius {
                let sy = (y + dy).clamp(0, height - 1);
                let idx = ((sy * width + x) * 4) as usize;
                r += temp[idx] as u32;
                g += temp[idx + 1] as u32;
                b += temp[idx + 2] as u32;
                a += temp[idx + 3] as u32;
                count += 1;
            }

            let out_idx = ((y * width + x) * 4) as usize;
            data[out_idx] = (r / count) as u8;
            data[out_idx + 1] = (g / count) as u8;
            data[out_idx + 2] = (b / count) as u8;
            data[out_idx + 3] = (a / count) as u8;
        }
    }
}

/// Apply optimized box blur using SIMD when available
#[cfg(feature = "simd")]
#[allow(dead_code)] // Placeholder for future SIMD optimization
fn apply_box_blur_simd(pixmap: &mut Pixmap, radius: f32) {
    if radius <= 0.0 {
        return;
    }

    // Use SIMD instructions for faster blur
    // This is a placeholder - real SIMD implementation would use intrinsics
    apply_box_blur(pixmap, radius);
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
