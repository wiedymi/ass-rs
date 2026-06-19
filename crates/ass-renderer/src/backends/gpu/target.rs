//! Size-keyed offscreen render target and its padded readback buffer.
//!
//! The compositor draws into [`Target::view`], queues a copy via
//! [`Target::copy_into_readback`] and pulls the bytes out with
//! [`Target::read_back`]. The same texture and buffer are reused across frames
//! until the requested size changes, so steady-state frames allocate nothing here.

use crate::utils::RenderError;

use super::pipeline::TARGET_FORMAT;

/// Cached offscreen render target and its padded readback buffer for one size.
pub(super) struct Target {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: wgpu::Buffer,
    padded_row: u32,
}

impl Target {
    /// Allocate a transparent `width * height` target plus a readback buffer
    /// padded to wgpu's 256-byte row alignment.
    pub(super) fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ass-gpu-target"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let padded_row = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ass-gpu-readback"),
            size: u64::from(padded_row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            width,
            height,
            texture,
            view,
            readback,
            padded_row,
        }
    }

    /// Whether this target already matches `width * height`.
    pub(super) fn matches(&self, width: u32, height: u32) -> bool {
        self.width == width && self.height == height
    }

    /// The colour-attachment view the render pass draws into.
    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Queue a copy of the rendered target into the padded readback buffer.
    pub(super) fn copy_into_readback(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.readback,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Map the readback buffer, copy out the frame (stripping wgpu's 256-byte row
    /// padding) and unmap it for reuse next frame.
    pub(super) fn read_back(&self, device: &wgpu::Device) -> Result<Vec<u8>, RenderError> {
        let slice = self.readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        device.poll(wgpu::Maintain::Wait);
        match rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(RenderError::BackendError(format!("buffer map failed: {e}"))),
            Err(e) => {
                return Err(RenderError::BackendError(format!(
                    "map channel closed: {e}"
                )))
            }
        }

        let row = (self.width * 4) as usize;
        let padded_row = self.padded_row as usize;
        let mapped = slice.get_mapped_range();
        let mut out = vec![0u8; row * self.height as usize];
        for (y, dst_row) in out.chunks_exact_mut(row).enumerate() {
            let src = y * padded_row;
            dst_row.copy_from_slice(&mapped[src..src + row]);
        }
        drop(mapped);
        self.readback.unmap();
        Ok(out)
    }
}
