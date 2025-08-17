//! Metal backend for macOS/iOS

#![allow(dead_code)] // Work in progress backend

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, format, vec, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{boxed::Box, vec::Vec};

#[cfg(target_os = "macos")]
use metal::*;

use crate::backends::{BackendFeature, BackendType, RenderBackend};
use crate::pipeline::{IntermediateLayer, Pipeline, SoftwarePipeline};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};

/// Metal rendering backend
pub struct MetalBackend {
    #[cfg(target_os = "macos")]
    device: Option<Device>,
    #[cfg(target_os = "macos")]
    command_queue: Option<CommandQueue>,
    #[cfg(target_os = "macos")]
    pipeline_state: Option<RenderPipelineState>,
    #[cfg(target_os = "macos")]
    depth_stencil_state: Option<DepthStencilState>,
    width: u32,
    height: u32,
}

impl MetalBackend {
    /// Create new Metal backend
    pub fn new(context: &RenderContext) -> Result<Self, RenderError> {
        #[cfg(target_os = "macos")]
        {
            // Get default Metal device
            let device = Device::system_default()
                .ok_or_else(|| RenderError::BackendError("No Metal device found".into()))?;

            // Create command queue
            let command_queue = device.new_command_queue();

            Ok(Self {
                device: Some(device),
                command_queue: Some(command_queue),
                pipeline_state: None,
                depth_stencil_state: None,
                width: context.width(),
                height: context.height(),
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(Self {
                width: context.width(),
                height: context.height(),
            })
        }
    }

    #[cfg(target_os = "macos")]
    /// Initialize the render pipeline
    fn init_pipeline(&mut self) -> Result<(), RenderError> {
        let device = self
            .device
            .as_ref()
            .ok_or_else(|| RenderError::BackendError("Device not initialized".into()))?;

        // Create shader library from source
        let shader_source = include_str!("metal_shaders.metal");
        let library = device
            .new_library_with_source(shader_source, &CompileOptions::new())
            .map_err(|e| RenderError::BackendError(format!("Failed to compile shaders: {e}")))?;

        let vertex_function = library
            .get_function("vertex_main", None)
            .map_err(|_| RenderError::BackendError("Vertex function not found".into()))?;
        let fragment_function = library
            .get_function("fragment_main", None)
            .map_err(|_| RenderError::BackendError("Fragment function not found".into()))?;

        // Create vertex descriptor
        let vertex_descriptor = VertexDescriptor::new();

        // Position attribute
        let pos_attr = vertex_descriptor.attributes().object_at(0).unwrap();
        pos_attr.set_format(MTLVertexFormat::Float2);
        pos_attr.set_offset(0);
        pos_attr.set_buffer_index(0);

        // Texture coordinate attribute
        let tex_attr = vertex_descriptor.attributes().object_at(1).unwrap();
        tex_attr.set_format(MTLVertexFormat::Float2);
        tex_attr.set_offset(8);
        tex_attr.set_buffer_index(0);

        // Vertex layout
        let layout = vertex_descriptor.layouts().object_at(0).unwrap();
        layout.set_stride(16);
        layout.set_step_function(MTLVertexStepFunction::PerVertex);

        // Create pipeline descriptor
        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));
        pipeline_descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        // Configure color attachment
        let color_attachment = pipeline_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        color_attachment.set_blending_enabled(true);
        color_attachment.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
        color_attachment.set_destination_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_source_alpha_blend_factor(MTLBlendFactor::One);
        color_attachment.set_destination_alpha_blend_factor(MTLBlendFactor::Zero);

        // Create pipeline state
        let pipeline_state = device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| {
                RenderError::BackendError(format!("Failed to create pipeline state: {e}"))
            })?;

        self.pipeline_state = Some(pipeline_state);

        // Create depth stencil state
        let depth_descriptor = DepthStencilDescriptor::new();
        depth_descriptor.set_depth_compare_function(MTLCompareFunction::Less);
        depth_descriptor.set_depth_write_enabled(true);

        let depth_stencil_state = device.new_depth_stencil_state(&depth_descriptor);
        self.depth_stencil_state = Some(depth_stencil_state);

        Ok(())
    }

    #[cfg(target_os = "macos")]
    /// Render layers using Metal
    fn render_layers(
        &self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        let device = self
            .device
            .as_ref()
            .ok_or_else(|| RenderError::BackendError("Device not initialized".into()))?;
        let command_queue = self
            .command_queue
            .as_ref()
            .ok_or_else(|| RenderError::BackendError("Command queue not initialized".into()))?;

        // Create texture for rendering
        let texture_descriptor = TextureDescriptor::new();
        texture_descriptor.set_width(context.width() as u64);
        texture_descriptor.set_height(context.height() as u64);
        texture_descriptor.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        texture_descriptor.set_usage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);

        let render_texture = device.new_texture(&texture_descriptor);

        // Create command buffer and encoder
        let command_buffer = command_queue.new_command_buffer();

        let render_pass_descriptor = RenderPassDescriptor::new();
        let color_attachment = render_pass_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_texture(Some(&render_texture));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor::new(0.0, 0.0, 0.0, 0.0));
        color_attachment.set_store_action(MTLStoreAction::Store);

        let encoder = command_buffer.new_render_command_encoder(render_pass_descriptor);

        if let Some(pipeline_state) = &self.pipeline_state {
            encoder.set_render_pipeline_state(pipeline_state);

            // Render each layer
            for layer in layers {
                match layer {
                    IntermediateLayer::Text(_text_data) => {
                        // Render text layer
                        // In production, would create vertex buffer with glyph data
                    }
                    IntermediateLayer::Vector(_vector_data) => {
                        // Render vector layer
                        // In production, would tessellate and render path
                    }
                    _ => {}
                }
            }
        }

        encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();

        // Read back texture data
        let bytes_per_row = context.width() * 4;
        let buffer_size = (bytes_per_row * context.height()) as usize;
        let mut buffer = vec![0u8; buffer_size];

        render_texture.get_bytes(
            buffer.as_mut_ptr() as *mut _,
            bytes_per_row as u64,
            MTLRegion {
                origin: MTLOrigin { x: 0, y: 0, z: 0 },
                size: MTLSize {
                    width: context.width() as u64,
                    height: context.height() as u64,
                    depth: 1,
                },
            },
            0,
        );

        Ok(buffer)
    }
}

impl RenderBackend for MetalBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Metal
    }

    fn create_pipeline(&self) -> Result<Box<dyn Pipeline>, RenderError> {
        // Use software pipeline as Metal-accelerated pipeline would require more setup
        Ok(Box::new(SoftwarePipeline::new()))
    }

    fn composite_layers(
        &self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        #[cfg(target_os = "macos")]
        {
            self.render_layers(layers, context)
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Fallback for non-macOS platforms
            let buffer_size = (context.width() * context.height() * 4) as usize;
            Ok(vec![0u8; buffer_size])
        }
    }

    fn composite_layers_incremental(
        &self,
        layers: &[IntermediateLayer],
        _dirty_regions: &[DirtyRegion],
        _previous_frame: &[u8],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        self.composite_layers(layers, context)
    }

    fn supports_feature(&self, feature: BackendFeature) -> bool {
        matches!(
            feature,
            BackendFeature::HardwareAcceleration | BackendFeature::ComputeShaders
        )
    }
}
