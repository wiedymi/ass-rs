//! The wgpu pipeline, batched render pass and readback for the GPU backend.
//!
//! [`Compositor`] builds its textured-quad pipeline (see [`super::shader`]),
//! sampler and bind-group layouts once, then reuses them every frame.
//! [`Compositor::composite`] draws an ordered list of [`RenderBitmap`] tiles in a
//! single render pass over a transparent offscreen `Rgba8Unorm` target (linear,
//! no gamma — matching the software compositor), then reads the target back to a
//! straight byte layout, stripping wgpu's 256-byte row padding.
//!
//! Per-frame resource churn is avoided: the offscreen target and readback buffer
//! are cached by size, the per-tile quad uniforms share one dynamic-offset buffer,
//! and the tile textures come from a [`super::pool::TilePool`].

use std::num::NonZeroU64;

use crate::backends::coverage::RenderBitmap;
use crate::utils::RenderError;

use super::pipeline::Programs;
use super::pool::{self, TilePool, UNIFORM_SIZE};
use super::target::Target;

/// Reusable per-frame quad-uniform buffer plus the group-0 bind group (sampler
/// and dynamic uniform) that reads it.
struct Uniforms {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    capacity: u32,
}

/// GPU tile compositor: pipeline, layouts, sampler and cached frame resources.
pub(super) struct Compositor {
    pipeline: wgpu::RenderPipeline,
    frame_layout: wgpu::BindGroupLayout,
    tile_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform_stride: u32,
    target: Option<Target>,
    uniforms: Option<Uniforms>,
    pool: TilePool,
}

impl Compositor {
    /// Build the compositor pipeline, layouts and sampler on `device`.
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let Programs {
            pipeline,
            frame_layout,
            tile_layout,
            sampler,
        } = Programs::new(device);

        let align = device.limits().min_uniform_buffer_offset_alignment;
        let uniform_stride = UNIFORM_SIZE.div_ceil(align) * align;

        Self {
            pipeline,
            frame_layout,
            tile_layout,
            sampler,
            uniform_stride,
            target: None,
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

    /// Ensure the shared uniform buffer holds at least `count` tile slots,
    /// growing (and rebuilding the group-0 bind group) only when needed.
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
        let count = u32::try_from(bitmaps.len()).unwrap_or(u32::MAX);
        self.ensure_uniforms(device, count);

        let stride = self.uniform_stride as usize;
        let mut uniform_bytes = vec![0u8; stride * bitmaps.len()];
        let mut tiles = Vec::with_capacity(bitmaps.len());
        for (i, bmp) in bitmaps.iter().enumerate() {
            let desc = pool::describe(bmp, width, height);
            let off = i * stride;
            uniform_bytes[off..off + UNIFORM_SIZE as usize]
                .copy_from_slice(desc.uniform.as_bytes());
            let tile = self.pool.acquire(
                device,
                &self.tile_layout,
                desc.format,
                desc.width,
                desc.height,
            );
            tile.upload(queue, desc.data, desc.bytes_per_row);
            tiles.push(tile);
        }
        if let Some(uniforms) = self.uniforms.as_ref() {
            if !uniform_bytes.is_empty() {
                queue.write_buffer(&uniforms.buffer, 0, &uniform_bytes);
            }
        }

        let target = self.target.as_ref().expect("target set by ensure_target");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ass-gpu-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ass-gpu-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target.view(),
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
            if let Some(uniforms) = self.uniforms.as_ref() {
                pass.set_pipeline(&self.pipeline);
                let mut offset = 0u32;
                for tile in &tiles {
                    pass.set_bind_group(0, &uniforms.bind_group, &[offset]);
                    pass.set_bind_group(1, &tile.bind_group, &[]);
                    pass.draw(0..6, 0..1);
                    offset += self.uniform_stride;
                }
            }
        }

        target.copy_into_readback(&mut encoder);
        queue.submit(Some(encoder.finish()));

        let out = target.read_back(device)?;
        self.pool.release(tiles);
        Ok(out)
    }
}
