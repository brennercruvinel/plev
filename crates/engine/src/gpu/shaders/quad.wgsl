struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection.matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

// sRGB (the space theme/hex colors live in) → linear. The surface is an sRGB
// format, so the GPU re-encodes linear→sRGB on write; without this an #303030
// fill would be treated as linear and shown ~2.5× too light.
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Linearize, then premultiplied alpha output.
    let rgb = srgb_to_linear(in.color.rgb);
    return vec4<f32>(rgb * in.color.a, in.color.a);
}
