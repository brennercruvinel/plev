// Backdrop blur blit: draws a rounded-rect quad sampling the pre-blurred
// backdrop texture (everything composited below this point, resolved and
// blurred by the effect processor) at the fragment's own framebuffer
// position. The rounded-rect SDF mask clips the frosted region; content
// drawn after composites on top.

struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

@group(1) @binding(0)
var backdrop_tex: texture_2d<f32>;
@group(1) @binding(1)
var backdrop_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) local: vec2<f32>,  // pixel offset from the rect center
    @location(2) params: vec4<f32>, // half_w, half_h, corner_radius, unused
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) params: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection.matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.local = in.local;
    out.params = in.params;
    return out;
}

// Signed distance to a rounded box (Inigo Quilez formula)
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2(r);
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // The blurred texture covers the whole surface: sample it at this
    // fragment's framebuffer position (physical pixels), which stays
    // correct under logical-coordinate projections (HiDPI).
    let dims = vec2<f32>(textureDimensions(backdrop_tex));
    let color = textureSample(backdrop_tex, backdrop_sampler, in.clip_position.xy / dims);

    let half_size = in.params.xy;
    let radius = in.params.z;
    let d = sd_rounded_box(in.local, half_size, radius);
    let mask = 1.0 - smoothstep(-0.5, 0.5, d);
    if (mask <= 0.0) {
        discard;
    }

    // The composed backdrop is premultiplied; scaling by the mask keeps it
    // premultiplied through the blend.
    return color * mask;
}
