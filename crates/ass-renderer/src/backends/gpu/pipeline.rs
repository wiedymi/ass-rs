//! One-time construction of the compositor's render pipeline, bind-group layouts
//! and sampler.
//!
//! Split out so [`super::compositor`] holds only per-frame logic. The two layouts
//! mirror the shader's groups: group 0 carries the shared sampler plus the
//! dynamic-offset quad uniform, group 1 carries each tile's texture.

use std::num::NonZeroU64;

use super::pool::UNIFORM_SIZE;

/// Offscreen target format. `Unorm` (not `Srgb`) so blending happens directly on
/// the stored bytes, reproducing the software backend's gamma-free premultiplied
/// source-over.
pub(super) const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// The compositor's reusable pipeline, bind-group layouts and sampler.
pub(super) struct Programs {
    /// Textured-quad render pipeline targeting [`TARGET_FORMAT`], shared by every
    /// tile draw and the internal-screen present pass.
    pub(super) pipeline: wgpu::RenderPipeline,
    /// Group-0 layout: sampler + dynamic-offset quad uniform.
    pub(super) frame_layout: wgpu::BindGroupLayout,
    /// Group-1 layout: a single sampled tile texture.
    pub(super) tile_layout: wgpu::BindGroupLayout,
    /// Nearest-neighbour sampler shared by all tiles.
    pub(super) sampler: wgpu::Sampler,
    /// The compiled WGSL module, retained so present pipelines for other target
    /// formats (e.g. a window surface) can be built later without recompiling.
    pub(super) shader: wgpu::ShaderModule,
    /// The shared pipeline layout, retained for the same reason as [`Self::shader`].
    pub(super) pipeline_layout: wgpu::PipelineLayout,
}

impl Programs {
    /// Build every reusable GPU object the compositor needs on `device`.
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ass-gpu-shader"),
            source: wgpu::ShaderSource::Wgsl(super::shader::SHADER.into()),
        });

        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ass-gpu-frame-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: NonZeroU64::new(u64::from(UNIFORM_SIZE)),
                    },
                    count: None,
                },
            ],
        });

        let tile_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ass-gpu-tile-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ass-gpu-pipeline-layout"),
            bind_group_layouts: &[&frame_layout, &tile_layout],
            push_constant_ranges: &[],
        });

        let pipeline = build_pipeline(device, &shader, &pipeline_layout, TARGET_FORMAT);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ass-gpu-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipeline,
            frame_layout,
            tile_layout,
            sampler,
            shader,
            pipeline_layout,
        }
    }
}

/// Build a textured-quad render pipeline targeting `format`, reusing the shared
/// `shader` and `layout`.
///
/// Every tile draw and every present pass uses the same WGSL and bindings; only
/// the colour-attachment format differs (the offscreen target/layer use
/// [`TARGET_FORMAT`], a window surface uses its own format). The blend stays the
/// gamma-free premultiplied source-over (`One` / `OneMinusSrcAlpha`).
pub(super) fn build_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let blend = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ass-gpu-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: blend,
                    alpha: blend,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
