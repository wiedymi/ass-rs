//! WebGPU shader definitions

/// Vertex shader for rendering quads
pub const VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Uniforms {
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    return out;
}
"#;

/// Fragment shader for texture rendering
pub const FRAGMENT_SHADER: &str = r#"
@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

struct Uniforms {
    color: vec4<f32>,
    opacity: f32,
}

@group(1) @binding(0)
var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(@location(0) tex_coords: vec2<f32>) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, tex_coords);
    return tex_color * uniforms.color * uniforms.opacity;
}
"#;

/// Compute shader for effects processing
pub const COMPUTE_SHADER: &str = r#"
@group(0) @binding(0)
var input_texture: texture_storage_2d<rgba8unorm, read>;

@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

struct EffectParams {
    blur_radius: f32,
    shadow_offset: vec2<f32>,
    shadow_color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> params: EffectParams;

@compute @workgroup_size(8, 8)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(input_texture);
    if (global_id.x >= dims.x || global_id.y >= dims.y) {
        return;
    }
    
    let coord = vec2<i32>(global_id.xy);
    var color = textureLoad(input_texture, coord);
    
    // Apply blur effect if radius > 0
    if (params.blur_radius > 0.0) {
        var sum = vec4<f32>(0.0);
        var weight_sum = 0.0;
        let radius = i32(params.blur_radius);
        
        for (var x = -radius; x <= radius; x = x + 1) {
            for (var y = -radius; y <= radius; y = y + 1) {
                let sample_coord = coord + vec2<i32>(x, y);
                if (sample_coord.x >= 0 && sample_coord.x < i32(dims.x) &&
                    sample_coord.y >= 0 && sample_coord.y < i32(dims.y)) {
                    let dist = length(vec2<f32>(f32(x), f32(y)));
                    let weight = exp(-dist * dist / (2.0 * params.blur_radius * params.blur_radius));
                    sum = sum + textureLoad(input_texture, sample_coord) * weight;
                    weight_sum = weight_sum + weight;
                }
            }
        }
        
        color = sum / weight_sum;
    }
    
    textureStore(output_texture, coord, color);
}
"#;

/// Text rendering shader with SDF
pub const TEXT_VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(input.position, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    out.color = input.color;
    return out;
}
"#;

/// Text fragment shader with SDF rendering
pub const TEXT_FRAGMENT_SHADER: &str = r#"
@group(0) @binding(1)
var t_glyph: texture_2d<f32>;
@group(0) @binding(2)
var s_glyph: sampler;

struct TextUniforms {
    outline_color: vec4<f32>,
    shadow_color: vec4<f32>,
    outline_width: f32,
    shadow_blur: f32,
    shadow_offset: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> text_uniforms: TextUniforms;

@fragment
fn fs_main(
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>
) -> @location(0) vec4<f32> {
    let distance = textureSample(t_glyph, s_glyph, tex_coords).r;
    
    // SDF threshold for main text
    let alpha = smoothstep(0.5 - 0.05, 0.5 + 0.05, distance);
    
    // Outline rendering
    var final_color = color;
    if (text_uniforms.outline_width > 0.0) {
        let outline_distance = 0.5 - text_uniforms.outline_width;
        let outline_alpha = smoothstep(outline_distance - 0.05, outline_distance + 0.05, distance);
        final_color = mix(text_uniforms.outline_color, color, alpha);
    }
    
    return vec4<f32>(final_color.rgb, final_color.a * alpha);
}
"#;
