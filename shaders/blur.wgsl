// Separable Gaussian blur — 13-tap fragment shader.
// Same shader for horizontal and vertical passes, controlled by `direction` uniform.
// Uses full-screen triangle (no vertex buffer).

struct BlurUniforms {
    direction: vec2<f32>,   // (1,0) for H, (0,1) for V
    texel_size: vec2<f32>,  // 1/width, 1/height
    weights: array<vec4<f32>, 4>, // 16 floats: 13 weights + 3 padding
}

@group(0) @binding(0)
var source_texture: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;

@group(1) @binding(0)
var<uniform> blur: BlurUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    // Full-screen triangle: 3 vertices cover clip space
    let x = f32(i32(idx & 1u)) * 4.0 - 1.0;
    let y = f32(i32(idx >> 1u)) * 4.0 - 1.0;
    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    // UV: clip [-1,1] -> [0,1], flip Y
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// Access weight by index from packed vec4 array
fn get_weight(i: i32) -> f32 {
    let vec_idx = i / 4;
    let comp_idx = i % 4;
    let v = blur.weights[vec_idx];
    if comp_idx == 0 { return v.x; }
    if comp_idx == 1 { return v.y; }
    if comp_idx == 2 { return v.z; }
    return v.w;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let step = blur.direction * blur.texel_size;

    // Center sample (weight index 0)
    var color = textureSample(source_texture, source_sampler, in.uv) * get_weight(0);

    // Symmetric taps: indices 1..6 correspond to offsets +/-1..+/-6
    for (var i: i32 = 1; i < 7; i = i + 1) {
        let offset = step * f32(i);
        let w = get_weight(i);
        color += textureSample(source_texture, source_sampler, in.uv + offset) * w;
        color += textureSample(source_texture, source_sampler, in.uv - offset) * w;
    }

    return color;
}
