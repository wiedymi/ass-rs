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
//! `One`/`OneMinusSrcAlpha` blend. The `compositor` submodule holds the pipeline.

mod compositor;
mod layer;
mod pipeline;
mod pool;
mod readback;
mod shader;
mod target;

#[cfg(test)]
mod tests;

use compositor::{Background, Compositor};

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

    /// Composite `bitmaps` into the resident GPU subtitle-layer texture, leaving
    /// it on the GPU with no readback. Call this only when the active subtitle
    /// changes; thereafter [`GpuBackend::present_frame`] reuses the cached layer.
    pub fn render_subtitle_layer(
        &mut self,
        bitmaps: &[RenderBitmap],
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        self.compositor
            .render_layer(&self.device, &self.queue, bitmaps, width, height)
    }

    /// Present the cached subtitle layer over an opaque black background into the
    /// screen target. The steady-state per-frame op: no re-rasterize, no upload,
    /// no readback. Requires a prior [`GpuBackend::render_subtitle_layer`] at the
    /// same size.
    pub fn present_frame(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        self.compositor.present_over(
            &self.device,
            &self.queue,
            Background::Clear(wgpu::Color::BLACK),
            width,
            height,
        )
    }

    /// Read the resident subtitle layer back to straight premultiplied-RGBA bytes.
    /// Used to verify the no-readback layer holds the same bytes the readback
    /// composite path produces; not part of the steady-state present path.
    pub fn layer_to_bytes(&self) -> Result<Vec<u8>, RenderError> {
        self.compositor.layer_to_bytes(&self.device, &self.queue)
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
