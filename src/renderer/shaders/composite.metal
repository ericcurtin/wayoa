// Alpha blending composite shader for Wayoa
// This shader composites multiple surfaces with alpha blending

#include <metal_stdlib>
using namespace metal;

// Vertex output / Fragment input
struct VertexOut {
    float4 position [[position]];
    float2 texCoord;
};

// Uniforms for per-surface rendering
struct SurfaceUniforms {
    float4x4 transform;
    float opacity;
    float3 padding;
};

// Vertex shader for compositing
vertex VertexOut composite_vertex(uint vertexID [[vertex_id]],
                                   constant float2 *positions [[buffer(0)]],
                                   constant SurfaceUniforms &uniforms [[buffer(1)]]) {
    uint index = vertexID * 2;

    VertexOut out;
    float4 pos = float4(positions[index], 0.0, 1.0);
    out.position = uniforms.transform * pos;
    out.texCoord = positions[index + 1];

    return out;
}

// Fragment shader with alpha blending
fragment float4 composite_fragment(VertexOut in [[stage_in]],
                                    texture2d<float> surfaceTexture [[texture(0)]],
                                    constant SurfaceUniforms &uniforms [[buffer(1)]]) {
    constexpr sampler textureSampler(mag_filter::linear,
                                     min_filter::linear,
                                     address::clamp_to_edge);

    float4 color = surfaceTexture.sample(textureSampler, in.texCoord);

    // Apply surface opacity
    color.a *= uniforms.opacity;

    // Premultiply alpha for correct blending
    color.rgb *= color.a;

    return color;
}

// Solid color fragment shader (for backgrounds, damage visualization, etc.)
fragment float4 solid_color_fragment(VertexOut in [[stage_in]],
                                      constant float4 &color [[buffer(0)]]) {
    return color;
}

// Subsurface compositing shader
// Handles child surfaces with offsets
struct SubsurfaceUniforms {
    float2 offset;
    float2 size;
    float opacity;
    float3 padding;
};

vertex VertexOut subsurface_vertex(uint vertexID [[vertex_id]],
                                    constant float2 *positions [[buffer(0)]],
                                    constant SubsurfaceUniforms &uniforms [[buffer(1)]],
                                    constant float2 &viewport [[buffer(2)]]) {
    uint index = vertexID * 2;

    // Apply offset and scale to viewport
    float2 pos = positions[index];
    pos = pos * uniforms.size / viewport;
    pos = pos + uniforms.offset / viewport * 2.0;

    VertexOut out;
    out.position = float4(pos, 0.0, 1.0);
    out.texCoord = positions[index + 1];

    return out;
}
