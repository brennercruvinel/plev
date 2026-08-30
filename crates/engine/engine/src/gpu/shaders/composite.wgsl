// Composite shader — full-screen triangle, samples layer texture, applies opacity
// Uses vertex_index trick: 3 vertices covering the full screen, no vertex buffer needed

@group(0) @binding(0)
var layer_texture: texture_2d<f32>;
@group(0) @binding(1)
var layer_sampler: sampler;

@group(1) @binding(0)
var<uniform> opacity: f32;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen triangle: 3 vertices that cover [-1,1] clip space
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index >> 1u) * 4 - 1);
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    // Map clip [-1,1] to UV [0,1], flip Y for texture coords
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(layer_texture, layer_sampler, in.uv);
    // color is already premultiplied — just scale by opacity
    return color * opacity;
}
