//! Rendering backend trait and implementations

use crate::pipeline::{IntermediateLayer, Pipeline};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, format, sync::Arc, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{boxed::Box, sync::Arc, vec::Vec};

#[cfg(feature = "software-backend")]
pub mod software;

#[cfg(feature = "hardware-backend")]
pub mod hardware;

#[cfg(feature = "web-backend")]
pub mod web;

/// Backend type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Auto-detect best available backend
    Auto,
    /// CPU-based software renderer
    Software,
    /// Vulkan hardware acceleration
    Vulkan,
    /// Metal hardware acceleration (macOS)
    Metal,
    /// WebGPU for web and native
    WebGPU,
    /// WebGL fallback for web
    WebGL,
}

impl BackendType {
    /// Get backend type as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Software => "Software",
            Self::Vulkan => "Vulkan",
            Self::Metal => "Metal",
            Self::WebGPU => "WebGPU",
            Self::WebGL => "WebGL",
        }
    }
}

/// Core rendering backend trait
pub trait RenderBackend: Send + Sync {
    /// Get the backend type
    fn backend_type(&self) -> BackendType;

    /// Create a pipeline for this backend
    fn create_pipeline(&self) -> Result<Box<dyn Pipeline>, RenderError>;

    /// Composite layers into final frame
    fn composite_layers(
        &self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError>;

    /// Composite layers incrementally (dirty regions only)
    fn composite_layers_incremental(
        &self,
        layers: &[IntermediateLayer],
        dirty_regions: &[DirtyRegion],
        previous_frame: &[u8],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        // Default implementation: full re-render
        let _ = (dirty_regions, previous_frame);
        self.composite_layers(layers, context)
    }

    /// Check if backend supports a specific feature
    fn supports_feature(&self, feature: BackendFeature) -> bool {
        match feature {
            BackendFeature::IncrementalRendering => false,
            BackendFeature::HardwareAcceleration => false,
            BackendFeature::ComputeShaders => false,
            BackendFeature::AsyncRendering => false,
        }
    }

    /// Get backend metrics if available
    #[cfg(feature = "backend-metrics")]
    fn metrics(&self) -> Option<BackendMetrics> {
        None
    }
}

/// Backend feature capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendFeature {
    /// Supports incremental rendering of dirty regions
    IncrementalRendering,
    /// Hardware accelerated rendering
    HardwareAcceleration,
    /// Compute shader support
    ComputeShaders,
    /// Async rendering operations
    AsyncRendering,
}

/// Backend performance metrics
#[cfg(feature = "backend-metrics")]
#[derive(Debug, Clone)]
pub struct BackendMetrics {
    /// VRAM usage in bytes
    pub vram_usage: u64,
    /// Number of draw calls per frame
    pub draw_calls: usize,
    /// Batch threshold for events
    pub batch_threshold: usize,
    /// Average frame time in milliseconds
    pub avg_frame_time_ms: f32,
    /// Peak frame time in milliseconds
    pub peak_frame_time_ms: f32,
}

#[cfg(feature = "backend-metrics")]
impl BackendMetrics {
    /// Create new metrics with defaults
    pub fn new() -> Self {
        Self {
            vram_usage: 0,
            draw_calls: 0,
            batch_threshold: 100,
            avg_frame_time_ms: 0.0,
            peak_frame_time_ms: 0.0,
        }
    }
}

#[cfg(feature = "backend-metrics")]
impl Default for BackendMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "nostd"))]
#[cfg(feature = "nostd")]
use alloc::sync::Arc;

/// Create a backend instance for the given type
pub fn create_backend(
    backend_type: BackendType,
    width: u32,
    height: u32,
) -> Result<Arc<dyn RenderBackend>, RenderError> {
    match backend_type {
        BackendType::Auto => {
            // Try backends in order of preference
            #[cfg(feature = "web-backend")]
            if let Ok(backend) = create_backend(BackendType::WebGPU, width, height) {
                return Ok(backend);
            }

            #[cfg(all(feature = "hardware-backend", feature = "vulkan"))]
            if let Ok(backend) = create_backend(BackendType::Vulkan, width, height) {
                return Ok(backend);
            }

            #[cfg(all(feature = "hardware-backend", feature = "metal", target_os = "macos"))]
            if let Ok(backend) = create_backend(BackendType::Metal, width, height) {
                return Ok(backend);
            }

            #[cfg(feature = "software-backend")]
            return create_backend(BackendType::Software, width, height);

            #[allow(unreachable_code)]
            Err(RenderError::BackendError("No backend available".into()))
        }

        #[cfg(feature = "software-backend")]
        BackendType::Software => {
            let context = crate::renderer::RenderContext::new(width, height);
            let backend = software::SoftwareBackend::new(&context)?;
            Ok(Arc::new(backend))
        }

        #[cfg(all(feature = "hardware-backend", feature = "vulkan"))]
        BackendType::Vulkan => {
            // TODO: Implement VulkanBackend
            Err(RenderError::BackendError("Vulkan backend not yet implemented".to_string()))
        }

        #[cfg(all(feature = "hardware-backend", feature = "metal", target_os = "macos"))]
        BackendType::Metal => {
            let backend = hardware::metal::MetalBackend::new(width, height)?;
            Ok(Arc::new(backend))
        }

        #[cfg(feature = "web-backend")]
        BackendType::WebGPU => {
            // TODO: Implement WebGPUBackend
            Err(RenderError::BackendError("WebGPU backend not yet implemented".to_string()))
        }
        BackendType::WebGL => {
            Err(RenderError::BackendError(
                "WebGL backend is not supported. Please use the Software backend instead, \
                 which provides full feature support and works in all environments including web browsers.".into()
            ))
        }

        #[allow(unreachable_patterns)]
        _ => Err(RenderError::BackendError(format!(
            "{} backend not available in this build",
            backend_type.as_str()
        ))),
    }
}
