//! Hybrid GPU compositing backend.
//!
//! Glyph and shape rasterization stays on the CPU: this backend owns a
//! [`SoftwareBackend`] and reuses its cached, parity-perfect coverage/RGBA tiles
//! ([`RenderBitmap`]). Only the per-frame compositing moves to the GPU — each
//! tile is uploaded to a texture and drawn as a positioned quad over an
//! offscreen target, which is read back to a straight premultiplied-RGBA buffer
//! byte-compatible with the software backend's frame output.
//!
//! Compositing is gamma-free integer-equivalent premultiplied source-over: the
//! target is `Rgba8Unorm` (linear), cleared to transparent, with a
//! `One`/`OneMinusSrcAlpha` blend. See [`compositor`] for the pipeline.

mod compositor;
mod pipeline;
mod pool;
mod shader;
mod target;

use compositor::Compositor;

use crate::backends::coverage::RenderBitmap;
use crate::backends::software::SoftwareBackend;
use crate::backends::{BackendFeature, BackendType, RenderBackend};
use crate::pipeline::{IntermediateLayer, Pipeline, SoftwarePipeline};
use crate::renderer::RenderContext;
use crate::utils::RenderError;

/// GPU backend that composites software-produced tiles via wgpu.
pub struct GpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    software: SoftwareBackend,
    compositor: Compositor,
}

impl GpuBackend {
    /// Initialise wgpu (high-performance adapter, default features/limits) and a
    /// software backend sized to `width * height` for tile production.
    pub fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| RenderError::BackendError("no wgpu adapter available".into()))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("ass-gpu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .map_err(|e| RenderError::BackendError(format!("wgpu request_device failed: {e}")))?;

        let context = RenderContext::new(width, height);
        let software = SoftwareBackend::new(&context)?;
        let compositor = Compositor::new(&device);

        Ok(Self {
            device,
            queue,
            software,
            compositor,
        })
    }

    /// Composite a pre-rasterized tile list directly on the GPU, returning a
    /// straight premultiplied-RGBA `width * height * 4` buffer.
    ///
    /// This is the entry point integrations (and the `gpu_compare` benchmark) use
    /// when they already hold [`RenderBitmap`] tiles and only want the GPU
    /// compositing step, skipping the full-frame pipeline.
    pub fn composite_bitmaps(
        &mut self,
        bitmaps: &[RenderBitmap],
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, RenderError> {
        self.compositor
            .composite(&self.device, &self.queue, bitmaps, width, height)
    }
}

impl RenderBackend for GpuBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Gpu
    }

    fn create_pipeline(&self) -> Result<Box<dyn Pipeline>, RenderError> {
        Ok(Box::new(SoftwarePipeline::new()))
    }

    fn composite_layers(
        &mut self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<u8>, RenderError> {
        let bitmaps = self.software.render_layers_to_bitmaps(layers, context)?;
        self.compositor.composite(
            &self.device,
            &self.queue,
            &bitmaps,
            context.width(),
            context.height(),
        )
    }

    fn render_layers_to_bitmaps(
        &mut self,
        layers: &[IntermediateLayer],
        context: &RenderContext,
    ) -> Result<Vec<RenderBitmap>, RenderError> {
        self.software.render_layers_to_bitmaps(layers, context)
    }

    fn supports_feature(&self, feature: BackendFeature) -> bool {
        matches!(feature, BackendFeature::HardwareAcceleration)
    }
}

#[cfg(test)]
mod tests {
    use super::GpuBackend;
    use crate::backends::coverage::{composite_bitmap, RenderBitmap};
    use std::sync::Arc;

    /// Composite one opaque-white coverage tile on the GPU and confirm the
    /// covered region is white, the outside is transparent, and the readback
    /// matches the software compositor byte-for-byte (within float rounding).
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

    /// Composite overlapping semi-transparent coverage tiles and a premultiplied
    /// RGBA tile, then compare against the software `composite_bitmap` reference.
    /// Reports MAE and max channel diff; both must be near zero (float blend vs
    /// the integer software blend rounds within one or two least-significant bits).
    #[test]
    fn gpu_matches_software_on_blended_tiles() {
        let mut backend = match GpuBackend::new(48, 32) {
            Ok(backend) => backend,
            Err(e) => {
                eprintln!("skipping GPU parity test (no usable adapter): {e}");
                return;
            }
        };

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

        let tiles = vec![
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
        ];

        let out = backend.composite_bitmaps(&tiles, 48, 32).expect("gpu composite");

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
}
