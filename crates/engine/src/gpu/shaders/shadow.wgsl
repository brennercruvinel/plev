// Shadow extraction shader — extracts silhouette from source alpha.
// Output: shadow_color with alpha = source.a * shadow_color.a
// Uses full-screen triangle (no vertex buffer).

struct ShadowUniforms {
    color: vec4<f32>,
}

@group(0) @binding(0)
var source_texture: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;

@group(1) @binding(0)
var<uniform> shadow: ShadowUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let x = f32(i32(idx & 1u)) * 4.0 - 1.0;
    let y = f32(i32(idx >> 1u)) * 4.0 - 1.0;
    var out: VertexOutput;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = textureSample(source_texture, source_sampler, in.uv);
    let alpha = src.a * shadow.color.a;
    // Linearize the sRGB shadow tint, then premultiplied output.
    return vec4<f32>(srgb_to_linear(shadow.color.rgb) * alpha, alpha);
}
