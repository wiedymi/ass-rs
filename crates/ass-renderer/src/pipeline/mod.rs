//! Rendering pipeline for processing events into layers

use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};
#[cfg(feature = "analysis-integration")]
use ass_core::analysis::ScriptAnalysis;
use ass_core::parser::{Event, Script};
use smallvec::SmallVec;

#[cfg(feature = "nostd")]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{string::String, vec::Vec};

pub mod animation;
pub mod compositing;
pub mod drawing;
pub mod effects;
pub mod font_loader;
pub mod shaping;
pub mod tag_processor;
pub mod text_segmenter;
pub mod transform;
pub mod validation;

mod software_pipeline;
// mod software_pipeline_new;  // Not currently used - fixes are already in main pipeline
pub use software_pipeline::SoftwarePipeline;

/// Pipeline trait for processing events
pub trait Pipeline: Send + Sync {
    /// Prepare the pipeline with a script
    fn prepare_script(
        &mut self,
        script: &Script,
        #[cfg(feature = "analysis-integration")] analysis: Option<&ScriptAnalysis>,
        #[cfg(not(feature = "analysis-integration"))] _analysis: Option<()>,
    ) -> Result<(), RenderError>;

    /// Get the current script
    fn script(&self) -> Option<&Script>;

    /// Process events into intermediate layers
    fn process_events(
        &mut self,
        events: &[&Event],
        time_cs: u32,
        context: &RenderContext,
    ) -> Result<Vec<IntermediateLayer>, RenderError>;

    /// Compute dirty regions for incremental rendering
    fn compute_dirty_regions(
        &self,
        events: &[&Event],
        time_cs: u32,
        prev_time_cs: u32,
    ) -> Result<Vec<DirtyRegion>, RenderError>;
}

/// Pipeline stage for processing
#[derive(Debug, Clone, Copy)]
pub enum PipelineStage {
    /// Text shaping stage
    Shaping,
    /// Drawing command processing
    Drawing,
    /// Effect application
    Effects,
    /// Layer compositing
    Compositing,
}

/// Intermediate layer representation
pub enum IntermediateLayer {
    /// Rasterized bitmap layer
    Raster(RasterData),
    /// Vector graphics layer
    Vector(VectorData),
    /// Text layer
    Text(TextData),
}

impl IntermediateLayer {
    /// Check if layer intersects with a dirty region
    pub fn intersects_region(&self, region: &DirtyRegion) -> bool {
        match self {
            Self::Raster(data) => {
                let layer_bounds = (data.x, data.y, data.x + data.width, data.y + data.height);
                region.intersects(layer_bounds)
            }
            Self::Vector(data) => {
                if let Some(bounds) = &data.bounds {
                    region.intersects(*bounds)
                } else {
                    true
                }
            }
            Self::Text(data) => {
                let approx_bounds = (
                    data.x as u32,
                    data.y as u32,
                    (data.x + 200.0) as u32,
                    (data.y + data.font_size * 1.5) as u32,
                );
                region.intersects(approx_bounds)
            }
        }
    }
}

/// Raster layer data
pub struct RasterData {
    /// Pixel data (RGBA)
    pub pixels: Vec<u8>,
    /// Layer X position
    pub x: u32,
    /// Layer Y position
    pub y: u32,
    /// Layer width
    pub width: u32,
    /// Layer height
    pub height: u32,
    /// Opacity (0-255)
    pub opacity: u8,
}

/// Vector graphics layer data
pub struct VectorData {
    /// Path to draw
    pub path: Option<tiny_skia::Path>,
    /// Fill color (RGBA)
    pub color: [u8; 4],
    /// Stroke information
    pub stroke: Option<StrokeInfo>,
    /// Bounding box
    pub bounds: Option<(u32, u32, u32, u32)>,
}

/// Stroke information for vector graphics
pub struct StrokeInfo {
    /// Stroke color (RGBA)
    pub color: [u8; 4],
    /// Stroke width
    pub width: f32,
}

/// Text layer data
pub struct TextData {
    /// Text content
    pub text: String,
    /// Font family
    pub font_family: String,
    /// Font size in pixels
    pub font_size: f32,
    /// Text color (RGBA)
    pub color: [u8; 4],
    /// X position
    pub x: f32,
    /// Y position
    pub y: f32,
    /// Text effects
    pub effects: SmallVec<[TextEffect; 4]>,
    /// Letter spacing in pixels
    pub spacing: f32,
}

/// Text effect enumeration
#[derive(Clone, Debug)]
pub enum TextEffect {
    /// Bold text
    Bold,
    /// Italic text
    Italic,
    /// Underline
    Underline,
    /// Strikethrough
    Strikethrough,
    /// Outline with color and width
    Outline { color: [u8; 4], width: f32 },
    /// Shadow with color and offset
    Shadow {
        color: [u8; 4],
        x_offset: f32,
        y_offset: f32,
    },
    /// Blur effect
    Blur { radius: f32 },
    /// Edge blur effect (only blurs outline/edges)
    EdgeBlur { radius: f32 },
    /// Karaoke effect
    Karaoke { progress: f32, style: u8 },
    /// 3D rotation (in degrees)
    Rotation { x: f32, y: f32, z: f32 },
    /// Shear/skew transformation
    Shear { x: f32, y: f32 },
    /// Scale transformation
    Scale { x: f32, y: f32 },
    /// Clip region
    Clip {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        inverse: bool,
    },
}
