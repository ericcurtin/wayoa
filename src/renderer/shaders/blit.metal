// Basic texture blit shader for Wayoa
// This shader renders textured quads for Wayland surface content

#include <metal_stdlib>
using namespace metal;

// Vertex input
struct VertexIn {
    float2 position [[attribute(0)]];
    float2 texCoord [[attribute(1)]];
};

// Vertex output / Fragment input
struct VertexOut {
    float4 position [[position]];
    float2 texCoord;
};

// Vertex shader
vertex VertexOut vertex_main(uint vertexID [[vertex_id]],
                              constant float2 *positions [[buffer(0)]]) {
    // Each vertex has position (2 floats) and texCoord (2 floats)
    // Packed as: [pos.x, pos.y, tex.x, tex.y] per vertex
    uint index = vertexID * 2;

    VertexOut out;
    out.position = float4(positions[index], 0.0, 1.0);
    out.texCoord = positions[index + 1];

    return out;
}

// Fragment shader
fragment float4 fragment_main(VertexOut in [[stage_in]],
                               texture2d<float> surfaceTexture [[texture(0)]]) {
    constexpr sampler textureSampler(mag_filter::linear,
                                     min_filter::linear,
                                     address::clamp_to_edge);

    float4 color = surfaceTexture.sample(textureSampler, in.texCoord);

    return color;
}
