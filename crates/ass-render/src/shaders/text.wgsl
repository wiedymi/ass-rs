// Vertex shader for hardware text rendering

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct RenderUniforms {
    view_proj: mat4x4<f32>,
    viewport_size: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: RenderUniforms;

@group(1) @binding(0)
var font_texture: texture_2d<f32>;

@group(1) @binding(1)
var font_sampler: sampler;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position to clip space
    out.clip_position = uniforms.view_proj * vec4<f32>(vertex.position, 1.0);
    out.tex_coords = vertex.tex_coords;
    out.color = vertex.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the font atlas texture (single channel)
    let alpha = textureSample(font_texture, font_sampler, in.tex_coords).r;
    
    // Apply text color with sampled alpha
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}