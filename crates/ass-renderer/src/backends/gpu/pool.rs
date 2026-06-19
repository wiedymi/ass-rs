//! Frame-to-frame reuse of per-tile GPU textures and quad uniforms.
//!
//! Tile geometry changes every frame, so contents are re-uploaded, but the GPU
//! objects are not recreated: [`TilePool`] hands out textures (and their group-1
//! bind groups) keyed by `(format, width, height)` and takes them back at the end
//! of the frame. [`describe`] turns a [`RenderBitmap`] into the upload data plus
//! the [`QuadUniform`] that places and tints it, so the compositor never touches
//! `wgpu` resource construction in its hot loop.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};

use crate::backends::coverage::RenderBitmap;

/// Byte size of one [`QuadUniform`] (three `vec4<f32>`), used to size and stride
/// the shared uniform buffer.
pub(super) const UNIFORM_SIZE: u32 = core::mem::size_of::<QuadUniform>() as u32;

/// Per-quad shader input: clip-space rectangle, straight colour and mode flag.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct QuadUniform {
    /// `(x0, y0, x1, y1)` clip-space corners (`y0` top, `y1` bottom).
    rect: [f32; 4],
    /// Straight (non-premultiplied) RGBA colour for coverage tiles, `0..=1`.
    color: [f32; 4],
    /// `mode.x` selects coverage (`0`) or premultiplied RGBA (`1`); rest padding.
    mode: [f32; 4],
}

impl QuadUniform {
    /// A premultiplied-RGBA passthrough quad covering the whole clip-space frame,
    /// used by the present pass to draw the layer (or a background) full-screen.
    pub(super) fn full_frame_rgba() -> Self {
        Self {
            rect: [-1.0, 1.0, 1.0, -1.0],
            color: [0.0; 4],
            mode: [1.0, 0.0, 0.0, 0.0],
        }
    }

    /// View the uniform as the raw bytes uploaded into the shared buffer.
    pub(super) fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

/// Upload data and placement for one tile, derived from a [`RenderBitmap`].
pub(super) struct TileDesc<'a> {
    /// Clip-space rectangle, colour and mode for this tile's quad.
    pub(super) uniform: QuadUniform,
    /// Texture format the pooled tile must use.
    pub(super) format: wgpu::TextureFormat,
    /// Tile width in pixels.
    pub(super) width: u32,
    /// Tile height in pixels.
    pub(super) height: u32,
    /// Pixel bytes to upload (`R8` coverage or `Rgba8` premultiplied).
    pub(super) data: &'a [u8],
    /// Row stride of `data` in bytes.
    pub(super) bytes_per_row: u32,
}

/// Describe `bmp`'s upload data and quad uniform for a `frame_w * frame_h` target.
pub(super) fn describe<'a>(bmp: &'a RenderBitmap, frame_w: u32, frame_h: u32) -> TileDesc<'a> {
    match bmp {
        RenderBitmap::Coverage {
            width,
            height,
            coverage,
            x,
            y,
            color,
        } => TileDesc {
            uniform: QuadUniform {
                rect: tile_rect(*x, *y, *width, *height, frame_w, frame_h),
                color: [
                    f32::from(color[0]) / 255.0,
                    f32::from(color[1]) / 255.0,
                    f32::from(color[2]) / 255.0,
                    f32::from(color[3]) / 255.0,
                ],
                mode: [0.0, 0.0, 0.0, 0.0],
            },
            format: wgpu::TextureFormat::R8Unorm,
            width: *width,
            height: *height,
            data: coverage.as_slice(),
            bytes_per_row: *width,
        },
        RenderBitmap::Rgba {
            width,
            height,
            pixels,
            x,
            y,
        } => TileDesc {
            uniform: QuadUniform {
                rect: tile_rect(*x, *y, *width, *height, frame_w, frame_h),
                color: [0.0; 4],
                mode: [1.0, 0.0, 0.0, 0.0],
            },
            format: wgpu::TextureFormat::Rgba8Unorm,
            width: *width,
            height: *height,
            data: pixels.as_slice(),
            bytes_per_row: *width * 4,
        },
    }
}

/// A pooled tile texture and the group-1 bind group that samples it.
pub(super) struct PooledTile {
    /// Group-1 bind group exposing [`PooledTile::texture`] to the shader.
    pub(super) bind_group: wgpu::BindGroup,
    /// Pool key `(format, width, height)` this tile is returned under.
    key: (wgpu::TextureFormat, u32, u32),
    /// The backing texture, re-uploaded each time the tile is reused.
    texture: wgpu::Texture,
}

impl PooledTile {
    /// Overwrite the tile's texels with `data` (`bytes_per_row` per row).
    pub(super) fn upload(&self, queue: &wgpu::Queue, data: &[u8], bytes_per_row: u32) {
        let (_, width, height) = self.key;
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// Size-keyed cache of idle tile textures and their bind groups.
pub(super) struct TilePool {
    free: HashMap<(wgpu::TextureFormat, u32, u32), Vec<PooledTile>>,
}

impl TilePool {
    /// Create an empty pool.
    pub(super) fn new() -> Self {
        Self {
            free: HashMap::new(),
        }
    }

    /// Take a tile matching `(format, width, height)`, reusing an idle one when
    /// possible and otherwise creating a fresh texture and bind group.
    pub(super) fn acquire(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> PooledTile {
        let key = (format, width, height);
        if let Some(tile) = self.free.get_mut(&key).and_then(Vec::pop) {
            return tile;
        }
        create_tile(device, layout, key)
    }

    /// Return the frame's tiles to the pool for reuse next frame.
    pub(super) fn release(&mut self, tiles: Vec<PooledTile>) {
        for tile in tiles {
            self.free.entry(tile.key).or_default().push(tile);
        }
    }
}

/// Build a fresh tile texture and its group-1 bind group.
fn create_tile(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    key: (wgpu::TextureFormat, u32, u32),
) -> PooledTile {
    let (format, width, height) = key;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ass-gpu-tile"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ass-gpu-tile-bind"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&view),
        }],
    });
    PooledTile {
        bind_group,
        key,
        texture,
    }
}

/// Map a tile's integer pixel rectangle to clip-space corners, flipping `y` so
/// the tile's top row lands at the top of the output (wgpu texture origin).
fn tile_rect(x: i32, y: i32, width: u32, height: u32, frame_w: u32, frame_h: u32) -> [f32; 4] {
    let frame_w = frame_w as f32;
    let frame_h = frame_h as f32;
    let left = x as f32;
    let right = (x + width as i32) as f32;
    let top = y as f32;
    let bottom = (y + height as i32) as f32;
    [
        left / frame_w * 2.0 - 1.0,
        1.0 - top / frame_h * 2.0,
        right / frame_w * 2.0 - 1.0,
        1.0 - bottom / frame_h * 2.0,
    ]
}
