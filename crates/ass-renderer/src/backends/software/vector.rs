//! Raster- and vector-path layer drawing for the software backend.
//!
//! `\p` vector drawings (with optional `\bord` stroke, `\blur` and rectangular
//! `\clip`/`\iclip`) and raw raster layers are blended straight onto the frame
//! pixmap here, separate from the glyph/text path in the parent module.

use tiny_skia::{Pixmap, Transform};

use crate::backends::blur::apply_gaussian_blur;
use crate::utils::RenderError;

#[cfg(not(feature = "nostd"))]
use super::dirty::note_dirty_bbox;

impl super::SoftwareBackend {
    pub(super) fn draw_raster_layer(
        &mut self,
        data: &crate::pipeline::RasterData,
    ) -> Result<(), RenderError> {
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

    pub(super) fn draw_vector_layer(
        &mut self,
        data: &crate::pipeline::VectorData,
    ) -> Result<(), RenderError> {
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
}
