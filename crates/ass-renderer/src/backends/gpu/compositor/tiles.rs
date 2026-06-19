//! Per-tile preparation and the batched tile render pass.
//!
//! These helpers are shared by [`super::Compositor::composite`] (readback path)
//! and [`super::Compositor::render_layer`] (resident-layer path): both upload the
//! frame's tiles, write their quad uniforms and draw them in one render pass with
//! the premultiplied source-over blend. Kept in a descendant module so they reach
//! the compositor's private pool, uniform buffer and pipeline without exposing
//! them more widely.

use crate::backends::coverage::RenderBitmap;
use crate::backends::gpu::pool::{self, PooledTile, UNIFORM_SIZE};

use super::{Compositor, Uniforms};

impl Compositor {
    /// Acquire and upload one pooled texture per `bitmaps` entry and write their
    /// quad uniforms into the shared buffer, returning the tiles for one draw.
    pub(super) fn prepare_tiles(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bitmaps: &[RenderBitmap],
        width: u32,
        height: u32,
    ) -> Vec<PooledTile> {
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
        tiles
    }

    /// Record one batched render pass that clears `view` to transparent and draws
    /// every prepared tile with the premultiplied source-over blend.
    pub(super) fn draw_tiles_pass(
        pipeline: &wgpu::RenderPipeline,
        uniforms: Option<&Uniforms>,
        uniform_stride: u32,
        tiles: &[PooledTile],
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ass-gpu-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
        if let Some(uniforms) = uniforms {
            pass.set_pipeline(pipeline);
            let mut offset = 0u32;
            for tile in tiles {
                pass.set_bind_group(0, &uniforms.bind_group, &[offset]);
                pass.set_bind_group(1, &tile.bind_group, &[]);
                pass.draw(0..6, 0..1);
                offset += uniform_stride;
            }
        }
    }
}
