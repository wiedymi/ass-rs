//! Resident subtitle-layer texture and the screen target the present pass draws.
//!
//! [`Layer`] is the keystone of the no-readback fast path: the subtitle tiles are
//! composited into it once (when the subtitle changes) and it then lives on the
//! GPU as a sampled texture. Every steady-state frame the present pass blends it
//! over a background into a [`Screen`] target, with no re-rasterize, no upload and
//! no readback. Both resources are size-keyed and recreated only when the frame
//! size changes; [`Layer`] additionally keeps a [`Readback`] so the layer bytes
//! can be pulled back to verify they match the readback composite path.

use super::pipeline::TARGET_FORMAT;
use super::readback::Readback;

use crate::utils::RenderError;

/// Persistent, size-keyed subtitle-layer texture holding premultiplied RGBA.
pub(super) struct Layer {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    readback: Readback,
}

impl Layer {
    /// Allocate a `width * height` layer texture usable as both a render target
    /// and a sampled texture (and copyable for the parity readback). `tile_layout`
    /// is the compositor's group-1 layout so the layer can be bound like a tile.
    pub(super) fn new(
        device: &wgpu::Device,
        tile_layout: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ass-gpu-layer"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ass-gpu-layer-bind"),
            layout: tile_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            }],
        });
        let readback = Readback::new(device, width, height);
        Self {
            width,
            height,
            texture,
            view,
            bind_group,
            readback,
        }
    }

    /// Whether this layer already matches `width * height`.
    pub(super) fn matches(&self, width: u32, height: u32) -> bool {
        self.width == width && self.height == height
    }

    /// The colour-attachment view the layer is composited into.
    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// The group-1 bind group that samples the layer in the present pass.
    pub(super) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Queue a copy of the layer into its readback buffer (parity check only).
    pub(super) fn copy_into_readback(&self, encoder: &mut wgpu::CommandEncoder) {
        self.readback.copy_from(encoder, &self.texture);
    }

    /// Read the layer back to straight premultiplied-RGBA bytes (parity check).
    pub(super) fn read_back(&self, device: &wgpu::Device) -> Result<Vec<u8>, RenderError> {
        self.readback.read(device)
    }
}

/// Persistent, size-keyed screen target the present pass renders the final frame
/// into. Only the view is retained; in wgpu the view keeps the texture alive, and
/// the present path never reads it back.
pub(super) struct Screen {
    width: u32,
    height: u32,
    view: wgpu::TextureView,
}

impl Screen {
    /// Allocate a `width * height` opaque screen target for the present pass.
    pub(super) fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ass-gpu-screen"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            width,
            height,
            view,
        }
    }

    /// Whether this screen target already matches `width * height`.
    pub(super) fn matches(&self, width: u32, height: u32) -> bool {
        self.width == width && self.height == height
    }

    /// The colour-attachment view the present pass draws into.
    pub(super) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
