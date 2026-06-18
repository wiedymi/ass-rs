//! Software (CPU) rendering backend using tiny-skia

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{boxed::Box, sync::Arc, vec::Vec};

use crate::backends::{BackendFeature, BackendType, RenderBackend};
use crate::pipeline::{IntermediateLayer, Pipeline, SoftwarePipeline};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};
use tiny_skia::Pixmap;

mod cache;
mod dirty;
mod text;
mod vector;
#[cfg(not(feature = "nostd"))]
use cache::{DIRTY_BBOX, EMIT_SINK};
#[cfg(not(feature = "nostd"))]
use dirty::{clear_region, crop_pixmap};

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
