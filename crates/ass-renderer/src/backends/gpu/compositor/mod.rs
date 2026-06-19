//! The wgpu pipeline, batched render pass and readback for the GPU backend.
//!
//! [`Compositor`] builds its textured-quad pipeline (see [`super::shader`]),
//! sampler and bind-group layouts once, then reuses them every frame.
//! [`Compositor::composite`] draws an ordered list of [`RenderBitmap`] tiles in a
//! single render pass over a transparent offscreen `Rgba8Unorm` target (linear,
//! no gamma — matching the software compositor), then reads the target back to a
//! straight byte layout, stripping wgpu's 256-byte row padding.
//!
//! For the video-overlay fast path the same tile draw composites into a resident
//! [`super::layer::Layer`] texture via [`Compositor::render_layer`] (no readback),
//! and [`Compositor::present_over`] blends that cached layer over a background in
//! a single full-screen quad — the steady-state per-frame op when the subtitle is
//! unchanged: no re-rasterize, no upload, no readback. The tile-draw helpers live
//! in [`tiles`] and the present pass in [`present`]; both are descendant modules
//! so they share these private fields without widening their visibility.
//!
//! Per-frame resource churn is avoided: the offscreen target, layer and screen
//! are cached by size, the per-tile quad uniforms share one dynamic-offset buffer,
//! and the tile textures come from a [`super::pool::TilePool`].

mod present;
mod tiles;

use std::collections::HashMap;
use std::num::NonZeroU64;

use crate::backends::coverage::RenderBitmap;
use crate::utils::RenderError;

use super::layer::{Layer, Screen};
use super::pipeline::{build_pipeline, Programs};
use super::pool::{TilePool, UNIFORM_SIZE};
use super::target::Target;

pub use present::{Background, PresentTarget};

/// Reusable per-frame quad-uniform buffer plus the group-0 bind group (sampler
/// and dynamic uniform) that reads it.
struct Uniforms {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: u32,
}

/// GPU tile compositor: pipeline, layouts, sampler and cached frame resources.
///
/// The struct is public so a windowed demo can own its own wgpu device/surface
/// and drive the resident-layer fast path directly via [`Compositor::new`],
/// [`Compositor::render_layer`] and [`Compositor::present_to_view`]; the fields
/// stay private.
pub struct Compositor {
    pipeline: wgpu::RenderPipeline,
    frame_layout: wgpu::BindGroupLayout,
    tile_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    present_pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
    uniform_stride: u32,
    target: Option<Target>,
    layer: Option<Layer>,
    screen: Option<Screen>,
    uniforms: Option<Uniforms>,
    pool: TilePool,
}

impl Compositor {
    /// Build the compositor pipeline, layouts and sampler on `device`.
    pub fn new(device: &wgpu::Device) -> Self {
        let Programs {
            pipeline,
            frame_layout,
            tile_layout,
            sampler,
            shader,
            pipeline_layout,
        } = Programs::new(device);

        let align = device.limits().min_uniform_buffer_offset_alignment;
        let uniform_stride = UNIFORM_SIZE.div_ceil(align) * align;

        Self {
            pipeline,
            frame_layout,
            tile_layout,
            sampler,
            shader,
            pipeline_layout,
            present_pipelines: HashMap::new(),
            uniform_stride,
            target: None,
            layer: None,
            screen: None,
            uniforms: None,
            pool: TilePool::new(),
        }
    }

    /// Ensure a cached offscreen target and readback buffer exist for `width *
    /// height`, recreating them only when the requested size changes.
    fn ensure_target(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self
            .target
            .as_ref()
            .is_some_and(|t| t.matches(width, height))
        {
            return;
        }
        self.target = Some(Target::new(device, width, height));
    }

    /// Ensure the resident subtitle-layer texture matches `width * height`.
    fn ensure_layer(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self
            .layer
            .as_ref()
            .is_some_and(|l| l.matches(width, height))
        {
            return;
        }
        self.layer = Some(Layer::new(device, &self.tile_layout, width, height));
    }

    /// Ensure the present pass's screen target matches `width * height`.
    fn ensure_screen(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self
            .screen
            .as_ref()
            .is_some_and(|s| s.matches(width, height))
        {
            return;
        }
        self.screen = Some(Screen::new(device, width, height));
    }

    /// Ensure the shared uniform buffer holds at least `count` quad slots, growing
    /// (and rebuilding the group-0 bind group) only when needed.
    fn ensure_uniforms(&mut self, device: &wgpu::Device, count: u32) {
        if count == 0 || self.uniforms.as_ref().is_some_and(|u| u.capacity >= count) {
            return;
        }
        let capacity = count.next_power_of_two();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ass-gpu-uniforms"),
            size: u64::from(self.uniform_stride) * u64::from(capacity),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ass-gpu-frame-bind"),
            layout: &self.frame_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: NonZeroU64::new(u64::from(UNIFORM_SIZE)),
                    }),
                },
            ],
        });
        self.uniforms = Some(Uniforms {
            buffer,
            bind_group,
            capacity,
        });
    }

    /// Composite `bitmaps` (painter order) over a transparent `width * height`
    /// target and return straight, premultiplied-RGBA bytes (the same layout and
    /// semantics as the software backend's frame buffer).
    pub(super) fn composite(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bitmaps: &[RenderBitmap],
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions);
        }
        self.ensure_target(device, width, height);
        let tiles = self.prepare_tiles(device, queue, bitmaps, width, height);

        let target = self.target.as_ref().expect("target set by ensure_target");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ass-gpu-encoder"),
        });
        Self::draw_tiles_pass(
            &self.pipeline,
            self.uniforms.as_ref(),
            self.uniform_stride,
            &tiles,
            target.view(),
            &mut encoder,
        );
        target.copy_into_readback(&mut encoder);
        queue.submit(Some(encoder.finish()));

        let out = target.read_back(device)?;
        self.pool.release(tiles);
        Ok(out)
    }

    /// Composite `bitmaps` into the resident subtitle-layer texture and leave it on
    /// the GPU. The "rare" path, run only when the subtitle changes; no readback is
    /// performed. The layer is then presented every frame into the internal screen
    /// target or, via [`Compositor::present_to_view`], an external surface view.
    pub fn render_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bitmaps: &[RenderBitmap],
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions);
        }
        self.ensure_layer(device, width, height);
        let tiles = self.prepare_tiles(device, queue, bitmaps, width, height);

        let layer = self.layer.as_ref().expect("layer set by ensure_layer");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ass-gpu-layer-encoder"),
        });
        Self::draw_tiles_pass(
            &self.pipeline,
            self.uniforms.as_ref(),
            self.uniform_stride,
            &tiles,
            layer.view(),
            &mut encoder,
        );
        queue.submit(Some(encoder.finish()));
        self.pool.release(tiles);
        Ok(())
    }

    /// Draw `background`, then the resident subtitle layer over it as one
    /// full-screen quad, into the size-cached screen target. The steady-state
    /// per-frame op: no re-rasterize, no upload, no readback. Requires a prior
    /// [`Compositor::render_layer`] at the same size.
    pub(super) fn present_over(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        background: Background,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions);
        }
        if !self
            .layer
            .as_ref()
            .is_some_and(|l| l.matches(width, height))
        {
            return Err(RenderError::BackendError(
                "present_over requires a matching render_layer first".into(),
            ));
        }
        self.ensure_screen(device, width, height);
        let quad_count = if matches!(background, Background::Texture(_)) {
            2
        } else {
            1
        };
        self.ensure_uniforms(device, quad_count);
        present::run(self, device, queue, background, quad_count)
    }

    /// Build (once per format) and cache a present pipeline whose colour target is
    /// `format`, so an external surface of an arbitrary format can be presented to.
    fn ensure_present_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.present_pipelines.contains_key(&format) {
            return;
        }
        let pipeline = build_pipeline(device, &self.shader, &self.pipeline_layout, format);
        self.present_pipelines.insert(format, pipeline);
    }

    /// Blend the resident subtitle layer over `background` into the externally owned
    /// `target` (e.g. a window surface texture view and its format), then submit.
    /// Unlike the internal-screen present this targets the caller's view with a
    /// pipeline matched to `target.format` and never waits or reads back — the
    /// surface's own present call paces it. Requires a prior
    /// [`Compositor::render_layer`] at the same size.
    pub fn present_to_view(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: PresentTarget<'_>,
        background: Background,
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        if width == 0 || height == 0 {
            return Err(RenderError::InvalidDimensions);
        }
        if !self
            .layer
            .as_ref()
            .is_some_and(|l| l.matches(width, height))
        {
            return Err(RenderError::BackendError(
                "present_to_view requires a matching render_layer first".into(),
            ));
        }
        let quad_count = if matches!(background, Background::Texture(_)) {
            2
        } else {
            1
        };
        self.ensure_uniforms(device, quad_count);
        self.ensure_present_pipeline(device, target.format);
        present::run_to_view(
            self,
            device,
            queue,
            target.view,
            target.format,
            background,
            quad_count,
        )
    }

    /// Read the resident subtitle layer back to straight premultiplied-RGBA bytes.
    /// Used to verify the layer holds the same bytes as the readback composite
    /// path; the steady-state present pass never reads back.
    pub(super) fn layer_to_bytes(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u8>, RenderError> {
        let layer = self.layer.as_ref().ok_or_else(|| {
            RenderError::BackendError("layer_to_bytes requires a prior render_layer".into())
        })?;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ass-gpu-layer-readback-encoder"),
        });
        layer.copy_into_readback(&mut encoder);
        queue.submit(Some(encoder.finish()));
        layer.read_back(device)
    }
}
