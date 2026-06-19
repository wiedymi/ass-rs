//! Padded texture-to-CPU readback buffer shared by offscreen GPU resources.
//!
//! wgpu requires `copy_texture_to_buffer` rows to be padded to a 256-byte
//! alignment, so the destination buffer is sized to the padded stride and
//! [`Readback::read`] strips that padding back to a tight `width * 4` layout.
//! The buffer is size-keyed by its owner and reused across frames, so a parity
//! readback allocates nothing once warmed up.

use crate::utils::RenderError;

/// A reusable, row-padded buffer that copies a texture back to straight bytes.
pub(super) struct Readback {
    buffer: wgpu::Buffer,
    padded_row: u32,
    width: u32,
    height: u32,
}

impl Readback {
    /// Allocate a readback buffer for a `width * height` `Rgba8` texture, padded
    /// to wgpu's 256-byte row alignment.
    pub(super) fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let padded_row = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ass-gpu-readback"),
            size: u64::from(padded_row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            padded_row,
            width,
            height,
        }
    }

    /// Queue a copy of `texture` into the padded readback buffer.
    pub(super) fn copy_from(&self, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::Texture) {
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.buffer,
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

    /// Map the buffer, copy out the frame (stripping the row padding) and unmap it
    /// for reuse next time.
    pub(super) fn read(&self, device: &wgpu::Device) -> Result<Vec<u8>, RenderError> {
        let slice = self.buffer.slice(..);
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
        self.buffer.unmap();
        Ok(out)
    }
}
