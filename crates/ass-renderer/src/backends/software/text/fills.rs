//! Main-fill passes for the software text layer: the swept/binary karaoke fill
//! and the plain sharp fill.

use tiny_skia::Transform;

use crate::pipeline::TextData;

use super::TextRun;

impl super::super::SoftwareBackend {
    /// Karaoke-fill pass: a left-to-right sweep (`\kf`/`\K`) draws the secondary
    /// colour across the syllable with the already-sung left portion in primary;
    /// a binary `\k` (or a sweep under a `\clip`) fills a single blended colour.
    pub(super) fn draw_karaoke_text(
        &mut self,
        data: &TextData,
        run: &TextRun,
        karaoke: (f32, u8, [u8; 4]),
    ) {
        let (progress, karaoke_style, karaoke_secondary) = karaoke;
        let paths = &run.paths;
        let clip_mask = run.clip_mask.as_ref();

        // ASS karaoke colours: a syllable is the secondary colour until it is
        // "sung", then the primary colour (the layer's `data.color`).
        let primary = data.color;
        let secondary = karaoke_secondary;

        let mut paint = tiny_skia::Paint {
            anti_alias: true,
            blend_mode: tiny_skia::BlendMode::SourceOver,
            ..Default::default()
        };

        // Use base_transform built above with rotation/scaling
        let text_transform = run.base_transform;

        // For \kf/\K mid-syllable, compute the left-to-right sweep boundary
        // from the glyph bounds. (Skipped when a \clip is active — combining
        // the sweep with an arbitrary clip mask is left to the colour blend.)
        let sweeping = karaoke_style != 0 && progress > 0.0 && progress < 1.0;
        let sweep_bounds = if sweeping && clip_mask.is_none() {
            let mut b: Option<tiny_skia::Rect> = None;
            for path in paths {
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
            for path in paths {
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
                    for path in paths {
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
                let lerp = |s: u8, e: u8| (s as f32 * (1.0 - progress) + e as f32 * progress) as u8;
                [
                    lerp(secondary[0], primary[0]),
                    lerp(secondary[1], primary[1]),
                    lerp(secondary[2], primary[2]),
                    primary[3],
                ]
            };
            paint.set_color_rgba8(c[0], c[1], c[2], c[3]);
            for path in paths {
                if let Some(t) = path.clone().transform(text_transform) {
                    self.pixmap.fill_path(
                        &t,
                        &paint,
                        tiny_skia::FillRule::Winding,
                        Transform::identity(),
                        clip_mask,
                    );
                }
            }
        }
    }

    /// Plain/sharp text pass: fill the merged glyph path in one pass.
    pub(super) fn draw_plain_text(&mut self, run: &TextRun) {
        // Draw without blur or karaoke: fill the merged glyph path in one pass.
        // (text_transform == base_transform, so merged_base applies directly.)
        if let Some(ref merged) = run.merged_base {
            self.pixmap.fill_path(
                merged,
                &run.text_paint,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                run.clip_mask.as_ref(),
            );
        }
    }
}
