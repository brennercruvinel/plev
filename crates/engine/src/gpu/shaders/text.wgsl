struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

@group(1) @binding(0)
var atlas_texture: texture_2d<f32>;
@group(1) @binding(1)
var atlas_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    // Glyph atlas coordinates in TEXELS, not normalized: the atlas can grow
    // partway through a frame, and a grow copies the old contents to the
    // same texel origin — so texel coordinates stay valid while normalized
    // ones would not. The fragment shader divides by the bound texture size.
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection.matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(atlas_texture));
    let alpha = textureSample(atlas_texture, atlas_sampler, in.uv / dims).r;
    // Linearize, then premultiplied alpha output.
    let a = in.color.a * alpha;
    let rgb = srgb_to_linear(in.color.rgb);
    return vec4<f32>(rgb * a, a);
}
