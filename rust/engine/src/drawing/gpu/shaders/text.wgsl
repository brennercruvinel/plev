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

// A color glyph (an emoji bitmap) keeps its RGBA in the same R8 atlas, four
// texels per pixel side by side (straight alpha, sRGB), from an x that is a
// multiple of 4; its quad carries u negated. The quad lands on whole
// physical pixels and the bitmap comes at the physical size, so each
// fragment reads its own pixel, unfiltered. Of the text color only the
// alpha (opacity) applies, never the rgb.
fn color_glyph(uv: vec2<f32>, opacity: f32) -> vec4<f32> {
    let x = i32(floor(-uv.x / 4.0)) * 4;
    let y = i32(floor(uv.y));
    let r = textureLoad(atlas_texture, vec2<i32>(x, y), 0).r;
    let g = textureLoad(atlas_texture, vec2<i32>(x + 1, y), 0).r;
    let b = textureLoad(atlas_texture, vec2<i32>(x + 2, y), 0).r;
    let a = textureLoad(atlas_texture, vec2<i32>(x + 3, y), 0).r * opacity;
    return vec4<f32>(srgb_to_linear(vec3<f32>(r, g, b)) * a, a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(atlas_texture));
    // Sampled before the branch: textureSample needs uniform control flow.
    let alpha = textureSample(atlas_texture, atlas_sampler, in.uv / dims).r;
    if in.uv.x < 0.0 {
        return color_glyph(in.uv, in.color.a);
    }
    // A coverage mask: linearize, then premultiplied alpha output.
    let a = in.color.a * alpha;
    let rgb = srgb_to_linear(in.color.rgb);
    return vec4<f32>(rgb * a, a);
}
