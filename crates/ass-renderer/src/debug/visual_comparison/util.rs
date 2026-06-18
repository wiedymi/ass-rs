//! Helpers for composing visual comparison images

use crate::utils::RenderError;
use tiny_skia::{Pixmap, Transform};

/// Create side-by-side comparison image
pub fn create_comparison_image(
    our_output: &Pixmap,
    libass_output: &Pixmap,
    diff_map: Option<&Pixmap>,
) -> Result<Pixmap, RenderError> {
    let width = our_output.width() + libass_output.width() + diff_map.map_or(0, |d| d.width());
    let height = our_output.height().max(libass_output.height());

    let mut comparison = Pixmap::new(width, height).ok_or(RenderError::InvalidPixmap)?;

    // Copy our output to left side
    comparison.draw_pixmap(
        0,
        0,
        our_output.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    // Copy libass output to middle
    comparison.draw_pixmap(
        our_output.width() as i32,
        0,
        libass_output.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    // Copy diff map to right side if provided
    if let Some(diff) = diff_map {
        comparison.draw_pixmap(
            (our_output.width() + libass_output.width()) as i32,
            0,
            diff.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }

    Ok(comparison)
}
