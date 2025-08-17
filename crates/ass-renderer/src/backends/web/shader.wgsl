// Vertex shader
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct Uniforms {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = vec4<f32>(input.position, 0.0, 1.0);
    output.position = uniforms.projection * uniforms.view * world_pos;
    output.tex_coord = input.tex_coord;
    output.color = input.color;
    return output;
}

// Fragment shader
@group(0) @binding(1)
var texture_sampler: sampler;

@group(0) @binding(2)
var texture_2d: texture_2d<f32>;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture_2d, texture_sampler, input.tex_coord);
    return tex_color * input.color;
}

// Text rendering shader
@fragment
fn fs_text(input: VertexOutput) -> @location(0) vec4<f32> {
    let alpha = textureSample(texture_2d, texture_sampler, input.tex_coord).r;
    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}

// Solid color shader (for vector graphics)
@fragment 
fn fs_solid(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}