//! WebGPU backend implementation

#![allow(dead_code)] // Work in progress backend

#[cfg(feature = "nostd")]
use alloc::{boxed::Box, format, vec::Vec};
#[cfg(not(feature = "nostd"))]
use std::{boxed::Box, vec::Vec};

use crate::backends::{BackendFeature, BackendType, RenderBackend};
use crate::pipeline::{IntermediateLayer, Pipeline, SoftwarePipeline};
use crate::renderer::RenderContext;
use crate::utils::{DirtyRegion, RenderError};

mod pipeline;
mod render_utils;
mod shader;
mod texture_cache;

pub use self::pipeline::WebGpuPipeline;

/// WebGPU rendering backend
pub struct WebGpuBackend {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    texture_bind_group_layout: Option<wgpu::BindGroupLayout>,
    width: u32,
    height: u32,
}

impl WebGpuBackend {
    /// Create a new WebGPU backend
    pub async fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        // Initialize wgpu
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| {
                RenderError::BackendError("Failed to find suitable GPU adapter".into())
            })?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("ASS Renderer Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| RenderError::BackendError(format!("Failed to create device: {e}")))?;

        Ok(Self {
            device: Some(device),
            queue: Some(queue),
            render_pipeline: None,
            texture_bind_group_layout: None,
            width,
            height,
        })
    }

    /// Create backend with existing wgpu resources
    pub fn from_dimensions(width: u32, height: u32) -> Self {
        Self {
            device: None,
            queue: None,
            render_pipeline: None,
            texture_bind_group_layout: None,
            width,
            height,
        }
    }

    /// Initialize render pipeline
    fn init_pipeline(&mut self) -> Result<(), RenderError> {
        let device = self
            .device
            .as_ref()
            .ok_or_else(|| RenderError::BackendError("Device not initialized".into()))?;

        // Create shader modules
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ASS Renderer Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(shader::TEXT_VERTEX_SHADER.into()),
        });

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ASS Renderer Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(shader::TEXT_FRAGMENT_SHADER.into()),
        });

        // Create texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ASS Renderer Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ASS Renderer Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.render_pipeline = Some(render_pipeline);
        self.texture_bind_group_layout = Some(texture_bind_group_layout);

        Ok(())
    }
}

impl RenderBackend for WebGpuBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::WebGPU
    }

    fn create_pipeline(&self) -> Result<Box<dyn Pipeline>, RenderError> {
        // Use software pipeline as fallback for now
        Ok(Box::new(SoftwarePipeline::new()))
    }

    fn composite_layers(
        &self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        let device = self
            .device
            .as_ref()
            .ok_or_else(|| RenderError::BackendError("Device not initialized".into()))?;
        let queue = self
            .queue
            .as_ref()
            .ok_or_else(|| RenderError::BackendError("Queue not initialized".into()))?;

        // Create output texture
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width: context.width(),
                height: context.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let output_texture = device.create_texture(&texture_desc);
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create staging buffer for readback
        let buffer_size = (context.width() * context.height() * 4) as u64;
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Prepare all buffers first (outside render pass)
        let mut buffers_and_counts: Vec<(wgpu::Buffer, wgpu::Buffer, u32)> = Vec::new();

        for layer in layers {
            match layer {
                IntermediateLayer::Text(text_data) => {
                    let (vertex_buffer, index_buffer, index_count) =
                        render_utils::render_text_layer(
                            device,
                            text_data,
                            context.width() as f32,
                            context.height() as f32,
                        )?;
                    buffers_and_counts.push((vertex_buffer, index_buffer, index_count));
                }
                IntermediateLayer::Vector(vector_data) => {
                    let (vertex_buffer, index_buffer, index_count) =
                        render_utils::render_vector_layer(
                            device,
                            vector_data,
                            context.width() as f32,
                            context.height() as f32,
                        )?;
                    buffers_and_counts.push((vertex_buffer, index_buffer, index_count));
                }
                _ => {}
            }
        }

        // Begin render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Initialize pipeline if needed
            if self.render_pipeline.is_none() {
                // Skip initialization in render pass for now
                // In production, this would be done during backend creation
            }

            // Now render all layers
            if let Some(pipeline) = &self.render_pipeline {
                for (vertex_buffer, index_buffer, index_count) in &buffers_and_counts {
                    render_pass.set_pipeline(pipeline);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..*index_count, 0, 0..1);
                }
            }
        }

        // Copy texture to staging buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(context.width() * 4),
                    rows_per_image: Some(context.height()),
                },
            },
            wgpu::Extent3d {
                width: context.width(),
                height: context.height(),
                depth_or_array_layers: 1,
            },
        );

        // Submit commands
        queue.submit(core::iter::once(encoder.finish()));

        // Read back buffer
        let buffer_slice = staging_buffer.slice(..);

        // Use pollster for async operations in wgpu (simpler than futures)
        pollster::block_on(async {
            buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::Maintain::Wait);
        });

        let data = buffer_slice.get_mapped_range();
        let result = data.to_vec();
        drop(data);
        staging_buffer.unmap();

        Ok(result)
    }

    fn composite_layers_incremental(
        &self,
        layers: &[IntermediateLayer],
        dirty_regions: &[DirtyRegion],
        previous_frame: &[u8],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        if dirty_regions.is_empty() {
            return Ok(previous_frame.to_vec());
        }
        self.composite_layers(layers, context)
    }

    fn supports_feature(&self, feature: BackendFeature) -> bool {
        match feature {
            BackendFeature::HardwareAcceleration => true,
            BackendFeature::ComputeShaders => true,
            BackendFeature::AsyncRendering => true,
            BackendFeature::IncrementalRendering => true,
        }
    }

    #[cfg(feature = "backend-metrics")]
    fn metrics(&self) -> Option<crate::backends::BackendMetrics> {
        Some(crate::backends::BackendMetrics {
            vram_usage: 0,
            draw_calls: 0,
            batch_threshold: 1000,
            avg_frame_time_ms: 0.0,
            peak_frame_time_ms: 0.0,
        })
    }
}
