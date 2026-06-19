//! The no-readback present pass: blend the resident layer over a background.
//!
//! [`run`] draws an optional background (an opaque clear colour or a provided
//! premultiplied texture) and then the cached subtitle layer, each as one
//! full-screen quad through the compositor's existing pipeline, into the
//! size-cached screen target. It submits and waits but never reads back — this is
//! the steady-state per-frame cost the resident layer exists to collapse. Lives in
//! a descendant module so it can reach the compositor's private fields directly.

use crate::backends::gpu::pool::{QuadUniform, UNIFORM_SIZE};
use crate::utils::RenderError;

use super::Compositor;

/// Background to establish before the present pass draws the subtitle layer.
#[derive(Clone, Copy)]
pub(in crate::backends::gpu) enum Background<'a> {
    /// Clear the screen target to an opaque colour, then draw the layer over it.
    Clear(wgpu::Color),
    /// Draw a premultiplied-RGBA texture full-screen, then the layer over it.
    Texture(&'a wgpu::TextureView),
}

/// Draw `background` then the resident layer over it into the screen target.
///
/// The caller has already ensured the screen target and a uniform buffer holding
/// at least `quad_count` slots exist; this writes the full-frame quad uniforms,
/// records the single present pass and waits for it to finish.
pub(super) fn run(
    comp: &Compositor,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    background: Background,
    quad_count: u32,
) -> Result<(), RenderError> {
    let stride = comp.uniform_stride as usize;
    let mut bytes = vec![0u8; stride * quad_count as usize];
    let full = QuadUniform::full_frame_rgba();
    for slot in 0..quad_count as usize {
        let off = slot * stride;
        bytes[off..off + UNIFORM_SIZE as usize].copy_from_slice(full.as_bytes());
    }
    let Some(uniforms) = comp.uniforms.as_ref() else {
        return Err(RenderError::BackendError(
            "present pass missing uniform buffer".into(),
        ));
    };
    queue.write_buffer(&uniforms.buffer, 0, &bytes);

    let bg_bind = match background {
        Background::Texture(view) => Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ass-gpu-bg-bind"),
            layout: &comp.tile_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            }],
        })),
        Background::Clear(_) => None,
    };
    let load = match background {
        Background::Clear(color) => wgpu::LoadOp::Clear(color),
        Background::Texture(_) => wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
    };

    let screen = comp.screen.as_ref().expect("screen set by caller");
    let layer = comp.layer.as_ref().expect("layer presence checked by caller");
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ass-gpu-present-encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ass-gpu-present-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: screen.view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&comp.pipeline);
        let mut offset = 0u32;
        if let Some(bind) = bg_bind.as_ref() {
            pass.set_bind_group(0, &uniforms.bind_group, &[offset]);
            pass.set_bind_group(1, bind, &[]);
            pass.draw(0..6, 0..1);
            offset += comp.uniform_stride;
        }
        pass.set_bind_group(0, &uniforms.bind_group, &[offset]);
        pass.set_bind_group(1, layer.bind_group(), &[]);
        pass.draw(0..6, 0..1);
    }
    queue.submit(Some(encoder.finish()));
    device.poll(wgpu::Maintain::Wait);
    Ok(())
}
