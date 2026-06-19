//! Per-tile GPU resource construction.
//!
//! Each [`RenderBitmap`] is uploaded to its own texture (an `R8Unorm` coverage
//! mask or an `Rgba8Unorm` premultiplied tile) and paired with a uniform that
//! places the tile in clip space and selects the shader mode. The returned bind
//! group retains its texture, view and uniform buffer through wgpu's internal
//! reference counting, so the caller only needs to keep the bind group alive.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::backends::coverage::RenderBitmap;

/// Per-quad shader input: clip-space rectangle, straight colour and mode flag.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadUniform {
    /// `(x0, y0, x1, y1)` clip-space corners (`y0` top, `y1` bottom).
    rect: [f32; 4],
    /// Straight (non-premultiplied) RGBA colour for coverage tiles, `0..=1`.
    color: [f32; 4],
    /// `mode.x` selects coverage (`0`) or premultiplied RGBA (`1`); rest padding.
    mode: [f32; 4],
}

/// Build the bind group that draws `bmp` over a `frame_w * frame_h` target.
pub(super) fn upload_tile(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    bmp: &RenderBitmap,
    frame_w: u32,
    frame_h: u32,
) -> wgpu::BindGroup {
    let (format, data, width, height, x, y, color, mode) = match bmp {
        RenderBitmap::Coverage {
            width,
            height,
            coverage,
            x,
            y,
            color,
        } => (
            wgpu::TextureFormat::R8Unorm,
            coverage.as_slice(),
            *width,
            *height,
            *x,
            *y,
            [
                f32::from(color[0]) / 255.0,
                f32::from(color[1]) / 255.0,
                f32::from(color[2]) / 255.0,
                f32::from(color[3]) / 255.0,
            ],
            0.0_f32,
        ),
        RenderBitmap::Rgba {
            width,
            height,
            pixels,
            x,
            y,
        } => (
            wgpu::TextureFormat::Rgba8Unorm,
            pixels.as_slice(),
            *width,
            *height,
            *x,
            *y,
            [0.0_f32; 4],
            1.0_f32,
        ),
    };

    let bytes_per_pixel = if format == wgpu::TextureFormat::R8Unorm {
        1
    } else {
        4
    };
    let extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ass-gpu-tile"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(width * bytes_per_pixel),
            rows_per_image: Some(height),
        },
        extent,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let uniform = QuadUniform {
        rect: tile_rect(x, y, width, height, frame_w, frame_h),
        color,
        mode: [mode, 0.0, 0.0, 0.0],
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ass-gpu-quad-uniform"),
        contents: bytemuck::bytes_of(&uniform),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ass-gpu-tile-bind"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    })
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
