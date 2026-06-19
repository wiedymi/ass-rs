//! Size-keyed offscreen render target and its padded readback buffer.
//!
//! The compositor draws into [`Target::view`], queues a copy via
//! [`Target::copy_into_readback`] and pulls the bytes out with
//! [`Target::read_back`]. The same texture and buffer are reused across frames
//! until the requested size changes, so steady-state frames allocate nothing here.

use crate::utils::RenderError;

use super::pipeline::TARGET_FORMAT;
use super::readback::Readback;

/// Cached offscreen render target and its padded readback buffer for one size.
pub(super) struct Target {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    readback: Readback,
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
        let readback = Readback::new(device, width, height);
        Self {
            width,
            height,
            texture,
            view,
            readback,
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
        self.readback.copy_from(encoder, &self.texture);
    }

    /// Map the readback buffer, copy out the frame (stripping wgpu's 256-byte row
    /// padding) and unmap it for reuse next frame.
    pub(super) fn read_back(&self, device: &wgpu::Device) -> Result<Vec<u8>, RenderError> {
        self.readback.read(device)
    }
}
