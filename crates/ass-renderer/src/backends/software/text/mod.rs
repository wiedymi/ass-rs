//! Text-layer glyph drawing for the software backend.
//!
//! A text layer is rendered in passes: a shared per-run setup
//! ([`SoftwareBackend::prepare_text_run`]) resolves the positioned glyph
//! outlines, transform, colours and effect parameters (and takes the cached
//! coverage / blur-tile fast paths), then the sharp shadow, opaque box, outline,
//! main fill and decorations are drawn in order. The main fill dispatches to one
//! of three paths — blurred text (temp pixmap + blur + composite), swept/binary
//! karaoke, or plain sharp text — kept separate here from the parent module's
//! layer compositing.

use tiny_skia::Transform;

use crate::pipeline::shaping::ShapedText;
use crate::pipeline::TextData;
use crate::utils::RenderError;

mod blur;
mod decorations;
mod fills;
mod prepare;

/// Resolved per-run drawing state shared by the shadow, opaque-box, outline,
/// main-fill (blur / karaoke / sharp) and decoration passes. Built once by
/// [`SoftwareBackend::prepare_text_run`] so every pass reads the same positioned
/// glyph outlines, transform, colours and effect parameters.
struct TextRun {
    /// Positioned glyph outlines (already projected for `\frx`/`\fry`).
    paths: Vec<tiny_skia::Path>,
    /// Base affine transform baking translation, rotation, scale and shear.
    base_transform: Transform,
    /// Baseline Y (`data.y + shaped.baseline`).
    baseline_y: f32,
    /// Shaped run metrics (width/height/ascent/descent/baseline).
    shaped: ShapedText,
    /// Rectangular `\clip`/`\iclip` mask, or `None` when unclipped.
    clip_mask: Option<tiny_skia::Mask>,
    /// `\blur` radius, or `None` when the text is sharp.
    blur_radius: Option<f32>,
    /// Outline colour and per-axis width (`\bord`/`\xbord`/`\ybord`).
    outline_info: Option<([u8; 4], f32, f32)>,
    /// Shadow colour and offset (`\shad`).
    shadow_info: Option<([u8; 4], f32, f32)>,
    /// `\be` edge-blur radius (softens the outline only).
    edge_blur_radius: Option<f32>,
    /// Karaoke `(progress, style, secondary colour)`.
    karaoke_info: Option<(f32, u8, [u8; 4])>,
    /// Glyph outlines merged once under `base_transform` (sharp/non-`\blur` path).
    merged_base: Option<tiny_skia::Path>,
    /// Fill paint in the primary colour.
    text_paint: tiny_skia::Paint<'static>,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl super::SoftwareBackend {
    pub(super) fn draw_text_layer(&mut self, data: &TextData) -> Result<(), RenderError> {
        let Some(run) = self.prepare_text_run(data)? else {
            return Ok(());
        };

        // Apply effects in order: shadow, outline, then main text. The sharp
        // shadow is skipped when \blur is active (it is folded into the blur
        // temp below so it softens together with the outline and fill).
        self.draw_text_shadow(&run);

        // Draw opaque box (BorderStyle 3) behind the text, covering the glyph
        // bounds expanded by the padding, in the outline colour.
        self.draw_opaque_box(data, &run);

        // Draw outline if present
        self.draw_text_outline(data, &run);

        // Apply blur if needed
        if let Some(radius) = run.blur_radius {
            self.draw_blurred_text(data, &run, radius);
        } else if let Some(karaoke) = run.karaoke_info {
            self.draw_karaoke_text(data, &run, karaoke);
        } else {
            self.draw_plain_text(&run);
        }

        self.draw_text_decorations(data, &run);

        Ok(())
    }
}
