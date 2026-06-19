//! Parity tests for the GPU backend against the software compositor.
//!
//! Each test skips gracefully when no usable wgpu adapter is present so CI on
//! headless machines stays green. They assert the GPU readback (and the resident
//! layer) reproduce the software `composite_bitmap` reference within blend/format
//! rounding (the layer path must match the readback path exactly).

use super::readback::Readback;
use super::{Background, Compositor, GpuBackend, PresentTarget};
use crate::backends::coverage::{composite_bitmap, RenderBitmap};
use std::sync::Arc;

/// Composite one opaque-white coverage tile on the GPU and confirm the covered
/// region is white, the outside is transparent, and the readback matches the
/// software compositor byte-for-byte (within float rounding).
#[test]
fn gpu_smoke_composites_white_tile() {
    let mut backend = match GpuBackend::new(64, 64) {
        Ok(backend) => backend,
        Err(e) => {
            eprintln!("skipping GPU smoke test (no usable adapter): {e}");
            return;
        }
    };

    let tile = RenderBitmap::Coverage {
        width: 10,
        height: 10,
        coverage: Arc::new(vec![255u8; 10 * 10]),
        x: 5,
        y: 5,
        color: [255, 255, 255, 255],
    };

    let out = backend
        .composite_bitmaps(std::slice::from_ref(&tile), 64, 64)
        .expect("gpu composite");
    assert_eq!(out.len(), 64 * 64 * 4);

    let at = |x: u32, y: u32| ((y * 64 + x) * 4) as usize;

    let center = at(10, 10);
    assert!(
        out[center] > 250
            && out[center + 1] > 250
            && out[center + 2] > 250
            && out[center + 3] > 250,
        "covered pixel should be opaque white, got {:?}",
        &out[center..center + 4]
    );

    let outside = at(0, 0);
    assert_eq!(
        &out[outside..outside + 4],
        &[0, 0, 0, 0],
        "pixel outside the tile must stay transparent"
    );

    // Ground truth: composite the same tile the software way.
    let mut reference = vec![0u8; 64 * 64 * 4];
    composite_bitmap(&mut reference, 64, 64, &tile);
    let max_diff = out
        .iter()
        .zip(reference.iter())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert!(
        max_diff <= 2,
        "GPU readback should match software compositor (max channel diff {max_diff})"
    );
}

/// Build the overlapping-tile fixture shared by the blend and layer parity tests.
fn blended_tiles() -> Vec<RenderBitmap> {
    let gradient = |w: u32, h: u32| -> Arc<Vec<u8>> {
        let mut data = vec![0u8; (w * h) as usize];
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = ((i * 7 + 11) % 256) as u8;
        }
        Arc::new(data)
    };
    // Straight (255, 0, 0, 160) premultiplied -> (160, 0, 0, 160).
    let rgba = {
        let mut px = vec![0u8; 6 * 6 * 4];
        for chunk in px.chunks_exact_mut(4) {
            chunk.copy_from_slice(&[160, 0, 0, 160]);
        }
        Arc::new(px)
    };
    vec![
        RenderBitmap::Coverage {
            width: 20,
            height: 20,
            coverage: gradient(20, 20),
            x: 4,
            y: 4,
            color: [200, 100, 50, 128],
        },
        RenderBitmap::Coverage {
            width: 20,
            height: 20,
            coverage: gradient(20, 20),
            x: 12,
            y: 8,
            color: [0, 180, 255, 200],
        },
        RenderBitmap::Rgba {
            width: 6,
            height: 6,
            pixels: rgba,
            x: 30,
            y: 10,
        },
    ]
}

/// Composite overlapping semi-transparent coverage tiles and a premultiplied RGBA
/// tile, then compare against the software `composite_bitmap` reference. Reports
/// MAE and max channel diff; both must be near zero (float blend vs the integer
/// software blend rounds within one or two least-significant bits).
#[test]
fn gpu_matches_software_on_blended_tiles() {
    let mut backend = match GpuBackend::new(48, 32) {
        Ok(backend) => backend,
        Err(e) => {
            eprintln!("skipping GPU parity test (no usable adapter): {e}");
            return;
        }
    };

    let tiles = blended_tiles();
    let out = backend
        .composite_bitmaps(&tiles, 48, 32)
        .expect("gpu composite");

    let mut reference = vec![0u8; 48 * 32 * 4];
    for tile in &tiles {
        composite_bitmap(&mut reference, 48, 32, tile);
    }

    let mut max_diff = 0u32;
    let mut sum_diff = 0u64;
    for (a, b) in out.iter().zip(reference.iter()) {
        let d = u32::from(a.abs_diff(*b));
        max_diff = max_diff.max(d);
        sum_diff += u64::from(d);
    }
    let mae = sum_diff as f64 / out.len() as f64;
    eprintln!("GPU vs software: MAE={mae:.4}  max_channel_diff={max_diff}");
    assert!(
        max_diff <= 2,
        "GPU blend should match software within rounding (max diff {max_diff}, MAE {mae:.4})"
    );
}

/// The no-readback layer path must hold exactly the bytes the readback composite
/// path produces: composite a tile set with `composite_bitmaps`, then render the
/// same tiles into the resident layer and read it back. Both use the identical
/// pipeline, blend and transparent clear, so the bytes must match exactly (no
/// rounding slack). Also exercises the present pass end to end.
#[test]
fn gpu_layer_matches_readback_composite() {
    let mut backend = match GpuBackend::new(48, 32) {
        Ok(backend) => backend,
        Err(e) => {
            eprintln!("skipping GPU layer parity test (no usable adapter): {e}");
            return;
        }
    };

    let tiles = blended_tiles();
    let readback = backend
        .composite_bitmaps(&tiles, 48, 32)
        .expect("readback composite");
    backend
        .render_subtitle_layer(&tiles, 48, 32)
        .expect("render layer");
    let layer = backend.layer_to_bytes().expect("layer readback");

    assert_eq!(readback.len(), layer.len());
    let max_diff = readback
        .iter()
        .zip(layer.iter())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    assert_eq!(
        max_diff, 0,
        "resident layer must hold the same bytes as the readback composite"
    );

    // The present pass must run end to end against the cached layer.
    backend.present_frame(48, 32).expect("present frame");
}

/// Acquire a headless wgpu device/queue, or `None` when no usable adapter exists
/// (so the test skips gracefully on headless CI).
fn headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))?;
    pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("ass-gpu-test-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        },
        None,
    ))
    .ok()
}

/// The format-parameterized present path must render into an external view of a
/// non-`Rgba8Unorm` surface format. Build a `Compositor` on a headless device,
/// composite an opaque-white tile into its resident layer, then `present_to_view`
/// over an opaque-black clear into a freshly-created `Bgra8Unorm` target (the kind
/// of format a window surface reports), read that target back and assert the tile
/// region is white over a black background. Proves the format-matched present
/// pipeline and external-view path work with no window.
#[test]
fn present_to_view_targets_external_bgra_surface() {
    let Some((device, queue)) = headless_device() else {
        eprintln!("skipping present_to_view test (no usable adapter)");
        return;
    };

    let (width, height) = (64u32, 64u32);
    let mut compositor = Compositor::new(&device);

    let tile = RenderBitmap::Coverage {
        width: 10,
        height: 10,
        coverage: Arc::new(vec![255u8; 10 * 10]),
        x: 5,
        y: 5,
        color: [255, 255, 255, 255],
    };
    compositor
        .render_layer(&device, &queue, std::slice::from_ref(&tile), width, height)
        .expect("render layer");

    let format = wgpu::TextureFormat::Bgra8Unorm;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ass-gpu-test-surface"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    compositor
        .present_to_view(
            &device,
            &queue,
            PresentTarget {
                view: &view,
                format,
            },
            Background::Clear(wgpu::Color::BLACK),
            width,
            height,
        )
        .expect("present to view");

    let readback = Readback::new(&device, width, height);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ass-gpu-test-readback-encoder"),
    });
    readback.copy_from(&mut encoder, &target);
    queue.submit(Some(encoder.finish()));
    let out = readback.read(&device).expect("read external surface back");
    assert_eq!(out.len(), (width * height * 4) as usize);

    // Opaque white tile (255,255,255,255) is channel-order agnostic, so the
    // Bgra8Unorm byte layout reads identically; the covered region is white.
    let at = |x: u32, y: u32| ((y * width + x) * 4) as usize;
    let center = at(10, 10);
    assert!(
        out[center] > 250
            && out[center + 1] > 250
            && out[center + 2] > 250
            && out[center + 3] > 250,
        "covered pixel should be opaque white over the layer, got {:?}",
        &out[center..center + 4]
    );

    // Outside the tile the opaque-black background shows through.
    let outside = at(40, 40);
    assert!(
        out[outside] < 5 && out[outside + 1] < 5 && out[outside + 2] < 5 && out[outside + 3] > 250,
        "pixel outside the tile should be the opaque black background, got {:?}",
        &out[outside..outside + 4]
    );
}
