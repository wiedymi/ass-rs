#include <metal_stdlib>
using namespace metal;

struct VertexInput {
    float2 position [[attribute(0)]];
    float2 tex_coords [[attribute(1)]];
};

struct VertexOutput {
    float4 position [[position]];
    float2 tex_coords;
};

struct Uniforms {
    float4x4 transform;
    float4 color;
    float opacity;
};

vertex VertexOutput vertex_main(VertexInput in [[stage_in]],
                                constant Uniforms& uniforms [[buffer(1)]]) {
    VertexOutput out;
    out.position = uniforms.transform * float4(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    return out;
}

fragment float4 fragment_main(VertexOutput in [[stage_in]],
                             texture2d<float> texture [[texture(0)]],
                             sampler textureSampler [[sampler(0)]],
                             constant Uniforms& uniforms [[buffer(1)]]) {
    float4 tex_color = texture.sample(textureSampler, in.tex_coords);
    return tex_color * uniforms.color * uniforms.opacity;
}

// Text rendering with SDF
struct TextVertexInput {
    float2 position [[attribute(0)]];
    float2 tex_coords [[attribute(1)]];
    float4 color [[attribute(2)]];
};

struct TextVertexOutput {
    float4 position [[position]];
    float2 tex_coords;
    float4 color;
};

vertex TextVertexOutput text_vertex_main(TextVertexInput in [[stage_in]],
                                         constant float4x4& projection [[buffer(1)]]) {
    TextVertexOutput out;
    out.position = projection * float4(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    out.color = in.color;
    return out;
}

fragment float4 text_fragment_main(TextVertexOutput in [[stage_in]],
                                  texture2d<float> glyphTexture [[texture(0)]],
                                  sampler glyphSampler [[sampler(0)]]) {
    float distance = glyphTexture.sample(glyphSampler, in.tex_coords).r;
    float alpha = smoothstep(0.45, 0.55, distance);
    return float4(in.color.rgb, in.color.a * alpha);
}