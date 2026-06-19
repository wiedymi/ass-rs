//! WGSL source for the hybrid tile compositor.
//!
//! A single pipeline draws every tile as a unit quad whose clip-space rectangle,
//! source colour and mode are supplied per draw through a dynamic-offset uniform
//! in group 0 (shared with the sampler), while the tile texture lives in its own
//! group 1 so it can be swapped per draw without rebuilding the uniform binding.
//! Two modes share one shader:
//!
//! * mode `0` — coverage: the bound texture is `R8Unorm`; its red channel is the
//!   A8 coverage and the uniform carries the straight RGBA colour. The fragment
//!   emits premultiplied `(rgb * a * cov, a * cov)`.
//! * mode `1` — RGBA: the bound texture is `Rgba8Unorm` holding an already
//!   premultiplied tile, emitted unchanged.
//!
//! Both rely on a `One` / `OneMinusSrcAlpha` blend so the result is integer-free
//! premultiplied source-over, matching the software compositor.

/// The complete WGSL module compiled once into the compositor pipeline.
pub(super) const SHADER: &str = r"
struct Quad {
    rect: vec4<f32>,
    color: vec4<f32>,
    mode: vec4<f32>,
};

@group(0) @binding(0) var tile_samp: sampler;
@group(0) @binding(1) var<uniform> quad: Quad;
@group(1) @binding(0) var tile_tex: texture_2d<f32>;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let uv = corners[vi];
    let x = mix(quad.rect.x, quad.rect.z, uv.x);
    let y = mix(quad.rect.y, quad.rect.w, uv.y);
    var out: VsOut;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(tile_tex, tile_samp, in.uv);
    if (quad.mode.x < 0.5) {
        let a = quad.color.a * texel.r;
        return vec4<f32>(quad.color.rgb * a, a);
    }
    return texel;
}
";
