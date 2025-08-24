//! Vulkan backend

// Minimal Vulkan backend wrapper that currently delegates compositing to the
// software renderer for full feature coverage. This ensures Vulkan backend is
// available and feature-complete while a true GPU path is developed.
#[cfg(all(feature = "vulkan", not(feature = "nostd")))]
mod vulkan_impl {

    use crate::backends::{BackendFeature, BackendType, RenderBackend};
    use crate::pipeline::{IntermediateLayer, Pipeline, SoftwarePipeline};
    use crate::renderer::RenderContext;
    use crate::utils::{DirtyRegion, RenderError};

    /// Vulkan rendering backend (software-composited for now)
    pub struct VulkanBackend;

    impl VulkanBackend {
        /// Create a new Vulkan backend instance
        pub fn new() -> Result<Self, RenderError> {
            Ok(Self)
        }
    }

    impl RenderBackend for VulkanBackend {
        fn backend_type(&self) -> BackendType {
            BackendType::Vulkan
        }

        fn create_pipeline(&self) -> Result<Box<dyn Pipeline>, RenderError> {
            // Use existing software pipeline until GPU pipeline is implemented
            Ok(Box::new(SoftwarePipeline::new()))
        }

        fn composite_layers(
            &self,
            layers: &[IntermediateLayer],
            context: &RenderContext,
        ) -> Result<Vec<u8>, RenderError> {
            // Delegate to software backend for pixel-correct results
            let backend = crate::backends::software::SoftwareBackend::new(context)?;
            backend.composite_layers(layers, context)
        }

        fn composite_layers_incremental(
            &self,
            layers: &[IntermediateLayer],
            dirty_regions: &[DirtyRegion],
            previous_frame: &[u8],
            context: &RenderContext,
        ) -> Result<Vec<u8>, RenderError> {
            let backend = crate::backends::software::SoftwareBackend::new(context)?;
            backend.composite_layers_incremental(layers, dirty_regions, previous_frame, context)
        }

        fn supports_feature(&self, feature: BackendFeature) -> bool {
            match feature {
                // We currently render via CPU but maintain incremental capability
                BackendFeature::IncrementalRendering => true,
                BackendFeature::HardwareAcceleration => false,
                BackendFeature::ComputeShaders => false,
                BackendFeature::AsyncRendering => false,
            }
        }
    }
}

#[cfg(all(feature = "vulkan", not(feature = "nostd")))]
pub use vulkan_impl::VulkanBackend;
