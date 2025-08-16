//! WebGPU rendering utilities

#[cfg(feature = "nostd")]
use alloc::{vec, vec::Vec};

use crate::pipeline::{IntermediateLayer, TextData, VectorData};
use crate::utils::RenderError;
use wgpu::util::DeviceExt;
use wgpu::{BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState};
use wgpu::{DepthStencilState, FragmentState, MultisampleState, PrimitiveState, VertexState};
use wgpu::{Device, PipelineLayout, Queue, RenderPipeline, ShaderModule, TextureFormat};

/// Vertex structure for rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coord: [f32; 2],
    pub color: [f32; 4],
}

/// Create a render pipeline with given shaders
pub fn create_render_pipeline(
    device: &Device,
    layout: &PipelineLayout,
    shader: &ShaderModule,
    vs_entry: &str,
    fs_entry: &str,
    format: TextureFormat,
) -> RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ASS Render Pipeline"),
        layout: Some(layout),
        vertex: VertexState {
            module: shader,
            entry_point: vs_entry,
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: core::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    // Position
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    // TexCoord
                    wgpu::VertexAttribute {
                        offset: 8,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    // Color
                    wgpu::VertexAttribute {
                        offset: 16,
                        shader_location: 2,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                ],
            }],
        },
        fragment: Some(FragmentState {
            module: shader,
            entry_point: fs_entry,
            targets: &[Some(ColorTargetState {
                format,
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

impl Vertex {
    pub fn new(x: f32, y: f32, u: f32, v: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y],
            tex_coord: [u, v],
            color,
        }
    }
}

/// Create a quad for rendering
pub fn create_quad_vertices(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
) -> Vec<Vertex> {
    vec![
        Vertex::new(x, y, 0.0, 0.0, color),
        Vertex::new(x + width, y, 1.0, 0.0, color),
        Vertex::new(x + width, y + height, 1.0, 1.0, color),
        Vertex::new(x, y, 0.0, 0.0, color),
        Vertex::new(x + width, y + height, 1.0, 1.0, color),
        Vertex::new(x, y + height, 0.0, 1.0, color),
    ]
}

/// Render text layer to vertex and index buffers
pub fn render_text_layer(
    device: &Device,
    text_data: &TextData,
    screen_width: f32,
    screen_height: f32,
) -> Result<(wgpu::Buffer, wgpu::Buffer, u32), RenderError> {
    // Convert text position to NDC coordinates
    let x = (text_data.position.0 / screen_width) * 2.0 - 1.0;
    let y = 1.0 - (text_data.position.1 / screen_height) * 2.0;

    // Estimate text size (simplified)
    let text_width = (text_data.text.len() as f32 * 10.0) / screen_width * 2.0;
    let text_height = 30.0 / screen_height * 2.0;

    // Extract color from tags
    let color = extract_color_from_tags(text_data);

    // Create vertices for text quad
    let vertices = create_quad_vertices(x, y, text_width, text_height, color);

    // Create indices
    let indices: Vec<u16> = vec![0, 1, 2, 3, 4, 5];

    // Create vertex buffer
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Text Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // Create index buffer
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Text Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Ok((vertex_buffer, index_buffer, indices.len() as u32))
}

/// Render vector layer to vertex and index buffers
pub fn render_vector_layer(
    device: &Device,
    vector_data: &VectorData,
    screen_width: f32,
    screen_height: f32,
) -> Result<(wgpu::Buffer, wgpu::Buffer, u32), RenderError> {
    // Convert color
    let color = [
        vector_data.fill_color.0 as f32 / 255.0,
        vector_data.fill_color.1 as f32 / 255.0,
        vector_data.fill_color.2 as f32 / 255.0,
        vector_data.fill_color.3 as f32 / 255.0,
    ];

    // Create simple triangle for testing
    let vertices = vec![
        Vertex::new(-0.5, -0.5, 0.0, 0.0, color),
        Vertex::new(0.5, -0.5, 1.0, 0.0, color),
        Vertex::new(0.0, 0.5, 0.5, 1.0, color),
    ];

    let indices: Vec<u16> = vec![0, 1, 2];

    // Create vertex buffer
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vector Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // Create index buffer
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vector Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Ok((vertex_buffer, index_buffer, indices.len() as u32))
}

/// Extract color from text tags
fn extract_color_from_tags(text_data: &TextData) -> [f32; 4] {
    let mut color = [1.0, 1.0, 1.0, 1.0]; // Default white

    if let Some(primary) = text_data.tags.colors.primary {
        // ASS colors are in BGR format
        color[2] = ((primary >> 16) & 0xFF) as f32 / 255.0; // B
        color[1] = ((primary >> 8) & 0xFF) as f32 / 255.0; // G
        color[0] = (primary & 0xFF) as f32 / 255.0; // R
    }

    if let Some(alpha) = text_data.tags.colors.alpha {
        // ASS alpha is inverted (0 = opaque, 255 = transparent)
        color[3] = (255 - alpha) as f32 / 255.0;
    }

    color
}
