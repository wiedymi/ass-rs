//! Backend auto-detection and probing

use crate::backends::{BackendType, RenderBackend};
use crate::renderer::RenderContext;
use crate::utils::RenderError;

#[cfg(feature = "nostd")]
use alloc::{sync::Arc, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{sync::Arc, vec::Vec};

/// Backend prober for automatic backend selection
pub struct BackendProber {
    preferred_order: Vec<BackendType>,
}

impl BackendProber {
    /// Create a new backend prober with default preferences
    pub fn new() -> Self {
        let mut preferred_order = Vec::new();

        #[cfg(all(feature = "web-backend", target_arch = "wasm32"))]
        {
            preferred_order.push(BackendType::WebGPU);
            preferred_order.push(BackendType::WebGL);
        }

        #[cfg(all(feature = "vulkan", not(target_arch = "wasm32")))]
        preferred_order.push(BackendType::Vulkan);

        #[cfg(all(feature = "metal", target_os = "macos"))]
        preferred_order.push(BackendType::Metal);

        #[cfg(feature = "software-backend")]
        preferred_order.push(BackendType::Software);

        Self { preferred_order }
    }

    /// Create prober with custom backend preference order
    pub fn with_preference(preferred_order: Vec<BackendType>) -> Self {
        Self { preferred_order }
    }

    /// Probe for the best available backend
    pub fn probe_best_backend(
        &self,
        context: &RenderContext,
    ) -> Result<Arc<dyn RenderBackend>, RenderError> {
        for backend_type in &self.preferred_order {
            match self.try_create_backend(*backend_type, context) {
                Ok(backend) => {
                    #[cfg(not(feature = "nostd"))]
                    eprintln!("Selected {} backend for rendering", backend_type.as_str());
                    return Ok(backend);
                }
                Err(_e) => {
                    #[cfg(not(feature = "nostd"))]
                    eprintln!(
                        "Failed to initialize {} backend: {}",
                        backend_type.as_str(),
                        _e
                    );
                }
            }
        }

        Err(RenderError::NoBackendAvailable)
    }

    /// Try to create a specific backend
    fn try_create_backend(
        &self,
        backend_type: BackendType,
        context: &RenderContext,
    ) -> Result<Arc<dyn RenderBackend>, RenderError> {
        match backend_type {
            #[cfg(feature = "software-backend")]
            BackendType::Software => {
                use crate::backends::software::SoftwareBackend;
                Ok(Arc::new(SoftwareBackend::new(context)?))
            }

            #[cfg(all(feature = "vulkan", not(feature = "nostd")))]
            BackendType::Vulkan => {
                use crate::backends::hardware::VulkanBackend;
                Ok(Arc::new(VulkanBackend::new()?))
            }

            #[cfg(feature = "metal")]
            BackendType::Metal => {
                use crate::backends::hardware::metal::MetalBackend;
                Ok(Arc::new(MetalBackend::new(context)?))
            }

            #[cfg(feature = "web-backend")]
            BackendType::WebGPU => {
                use crate::backends::web::WebGpuBackend;
                Ok(Arc::new(WebGpuBackend::from_dimensions(
                    context.width(),
                    context.height(),
                )))
            }

            #[cfg(feature = "web-backend")]
            BackendType::WebGL => {
                // WebGL backend not implemented yet
                Err(RenderError::UnsupportedBackend(
                    "WebGL backend not implemented",
                ))
            }

            _ => Err(RenderError::UnsupportedBackend(backend_type.as_str())),
        }
    }

    /// Check if a backend is available
    pub fn is_backend_available(&self, backend_type: BackendType) -> bool {
        match backend_type {
            #[cfg(feature = "software-backend")]
            BackendType::Software => true,

            #[cfg(feature = "vulkan")]
            BackendType::Vulkan => {
                // Could add runtime Vulkan availability check here
                true
            }

            #[cfg(feature = "metal")]
            BackendType::Metal => {
                #[cfg(target_os = "macos")]
                {
                    true
                }
                #[cfg(not(target_os = "macos"))]
                {
                    false
                }
            }

            #[cfg(feature = "web-backend")]
            BackendType::WebGPU | BackendType::WebGL => {
                #[cfg(target_arch = "wasm32")]
                {
                    true
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    false
                }
            }

            _ => false,
        }
    }

    /// Get list of available backends
    pub fn available_backends(&self) -> Vec<BackendType> {
        self.preferred_order
            .iter()
            .filter(|&&backend| self.is_backend_available(backend))
            .copied()
            .collect()
    }
}

impl Default for BackendProber {
    fn default() -> Self {
        Self::new()
    }
}
