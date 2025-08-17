//! Core renderer implementation

use crate::backends::RenderBackend;
use crate::pipeline::Pipeline;
use crate::utils::RenderError;
#[cfg(feature = "analysis-integration")]
use ass_core::analysis::ScriptAnalysis;
use ass_core::parser::Script;

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, sync::Arc};
#[cfg(not(feature = "nostd"))]
use std::{boxed::Box, sync::Arc, time::Duration};

mod context;
mod event_selector;
mod frame;
mod probing;

pub use context::RenderContext;
pub use event_selector::{ActiveEvents, DirtyRegion, EventSelector};
pub use frame::Frame;
pub use probing::BackendProber;

/// Main renderer that coordinates rendering pipeline
pub struct Renderer {
    context: RenderContext,
    backend: Arc<dyn RenderBackend>,
    pipeline: Box<dyn Pipeline>,
    event_selector: event_selector::EventSelector,
}

impl Renderer {
    /// Create a new renderer with the given backend type and context
    pub fn new(
        backend_type: crate::backends::BackendType,
        context: RenderContext,
    ) -> Result<Self, RenderError> {
        let backend =
            crate::backends::create_backend(backend_type, context.width(), context.height())?;
        let pipeline = backend.create_pipeline()?;

        Ok(Self {
            context,
            backend,
            pipeline,
            event_selector: event_selector::EventSelector::new(),
        })
    }

    /// Create a new renderer with a specific backend instance
    pub fn with_backend(
        context: RenderContext,
        backend: Arc<dyn RenderBackend>,
    ) -> Result<Self, RenderError> {
        let pipeline = backend.create_pipeline()?;

        Ok(Self {
            context,
            backend,
            pipeline,
            event_selector: event_selector::EventSelector::new(),
        })
    }

    /// Create renderer with automatic backend detection
    #[cfg(feature = "backend-probing")]
    pub fn with_auto_backend(context: RenderContext) -> Result<Self, RenderError> {
        let prober = BackendProber::new();
        let backend = prober.probe_best_backend(&context)?;
        Self::with_backend(context, backend)
    }

    /// Render a frame for the given script at the specified time
    pub fn render_frame(&mut self, script: &Script, time_cs: u32) -> Result<Frame, RenderError> {
        #[cfg(feature = "analysis-integration")]
        let analysis = ScriptAnalysis::analyze(script).ok();
        #[cfg(not(feature = "analysis-integration"))]
        let analysis = None;

        // Extract script resolution and update context
        for section in script.sections() {
            if let ass_core::parser::Section::ScriptInfo(info) = section {
                if let Some((play_x, play_y)) = info.play_resolution() {
                    self.context.set_playback_resolution(play_x, play_y);
                }
                if let Some((layout_x, layout_y)) = info.layout_resolution() {
                    self.context.set_storage_resolution(layout_x, layout_y);
                }
                break; // Only need first ScriptInfo section
            }
        }

        let active = self.event_selector.select_active(script, time_cs)?;
        let events = active.events;

        if events.is_empty() {
            return Ok(Frame::empty(
                self.context.width(),
                self.context.height(),
                time_cs,
            ));
        }

        self.pipeline.prepare_script(script, analysis.as_ref())?;
        let layers = self
            .pipeline
            .process_events(&events, time_cs, &self.context)?;

        // Debug: Check what layers we have
        // eprintln!("RENDERER: Got {} layers", layers.len());
        for (i, layer) in layers.iter().enumerate() {
            match layer {
                crate::pipeline::IntermediateLayer::Vector(_) => {
                    // eprintln!("RENDERER: Layer {} is VectorData", i);
                }
                crate::pipeline::IntermediateLayer::Text(_) => {
                    // eprintln!("RENDERER: Layer {} is TextData", i);
                }
                crate::pipeline::IntermediateLayer::Raster(_) => {
                    // eprintln!("RENDERER: Layer {} is RasterData", i);
                }
            }
        }

        let frame_data = self.backend.composite_layers(&layers, &self.context)?;

        Ok(Frame::new(
            frame_data,
            self.context.width(),
            self.context.height(),
            time_cs,
        ))
    }

    /// Render frame incrementally (dirty regions only)
    pub fn render_frame_incremental(
        &mut self,
        script: &Script,
        time_cs: u32,
        previous_frame: &Frame,
    ) -> Result<Frame, RenderError> {
        let active = self.event_selector.select_active(script, time_cs)?;
        let events = active.events;
        let dirty_regions =
            self.pipeline
                .compute_dirty_regions(&events, time_cs, previous_frame.timestamp())?;

        if dirty_regions.is_empty() {
            return Ok(previous_frame.clone());
        }

        #[cfg(feature = "analysis-integration")]
        let analysis = ScriptAnalysis::analyze(script).ok();
        #[cfg(not(feature = "analysis-integration"))]
        let analysis = None;

        self.pipeline.prepare_script(script, analysis.as_ref())?;
        let layers = self
            .pipeline
            .process_events(&events, time_cs, &self.context)?;
        let frame_data = self.backend.composite_layers_incremental(
            &layers,
            &dirty_regions,
            previous_frame.data(),
            &self.context,
        )?;

        Ok(Frame::new(
            frame_data,
            self.context.width(),
            self.context.height(),
            time_cs,
        ))
    }

    /// Get current backend type
    pub fn backend_type(&self) -> crate::backends::BackendType {
        self.backend.backend_type()
    }

    /// Get backend metrics if available
    #[cfg(feature = "backend-metrics")]
    pub fn backend_metrics(&self) -> Option<crate::backends::BackendMetrics> {
        self.backend.metrics()
    }

    /// Update render context
    pub fn set_context(&mut self, context: RenderContext) {
        self.context = context;
    }

    /// Get render context
    pub fn context(&self) -> &RenderContext {
        &self.context
    }

    /// Get mutable render context
    pub fn context_mut(&mut self) -> &mut RenderContext {
        &mut self.context
    }

    /// Set collision resolver for subtitle positioning
    pub fn set_collision_resolver(
        &mut self,
        _resolver: Box<dyn crate::collision::CollisionDetector>,
    ) {
        // TODO: Implement collision resolver integration
    }

    /// Get performance metrics if available
    pub fn metrics(&self) -> Option<PerformanceMetrics> {
        // TODO: Implement metrics collection
        None
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStatistics {
        CacheStatistics {
            glyph_hits: 0,
            font_entries: 0,
        }
    }
}

/// Performance metrics for rendering
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    #[cfg(not(feature = "nostd"))]
    pub parse_time: Duration,
    #[cfg(not(feature = "nostd"))]
    pub shape_time: Duration,
    #[cfg(not(feature = "nostd"))]
    pub render_time: Duration,
    #[cfg(not(feature = "nostd"))]
    pub total_time: Duration,
    #[cfg(feature = "nostd")]
    pub parse_time: u64, // milliseconds
    #[cfg(feature = "nostd")]
    pub shape_time: u64, // milliseconds
    #[cfg(feature = "nostd")]
    pub render_time: u64, // milliseconds
    #[cfg(feature = "nostd")]
    pub total_time: u64, // milliseconds
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub glyph_hits: usize,
    pub font_entries: usize,
}
