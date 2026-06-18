//! Blurred-text fill pass for the software text layer: rasterize shadow,
//! outline and fill into a padded temp pixmap (or reuse a cached one), Gaussian
//! blur it, and composite at the baseline.

use tiny_skia::{Pixmap, Transform};

use crate::backends::blur::apply_gaussian_blur;
use crate::backends::geometry::stroke_outline;
use crate::pipeline::TextData;

#[cfg(not(feature = "nostd"))]
use super::super::cache::{BlurTile, BlurTileKey, BLUR_TILES};
use super::TextRun;

impl super::super::SoftwareBackend {
    /// Blurred-text pass: rasterize shadow + outline + fill into a padded temp
    /// pixmap (or reuse a cached one), box-blur it, and composite at `baseline_y`.
    pub(super) fn draw_blurred_text(&mut self, data: &TextData, run: &TextRun, radius: f32) {
        let shaped = &run.shaped;
        let text_paint = &run.text_paint;
        let clip_mask = run.clip_mask.as_ref();
        let outline_info = run.outline_info;
        let shadow_info = run.shadow_info;
        let paths = &run.paths;
        let baseline_y = run.baseline_y;

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
            bold: run.bold,
            italic: run.italic,
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
                    for path in paths {
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
                for path in paths {
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
                for path in paths {
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
            for path in paths {
                if let Some(transformed) = path.clone().transform(temp_transform) {
                    temp_pixmap.fill_path(
                        &transformed,
                        text_paint,
                        tiny_skia::FillRule::Winding,
                        Transform::identity(),
                        clip_mask,
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
    }
}
