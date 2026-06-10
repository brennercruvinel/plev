// Sprite pipeline: sample the RGBA8 image atlas, optionally masked by a
// rounded-rect SDF (corner_radius). Atlas coordinates are passed in PIXELS
// and normalized against textureDimensions, so geometry stays valid when
// the atlas grows (allocations never move).

struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

@group(1) @binding(0)
var atlas: texture_2d<f32>;
@group(1) @binding(1)
var atlas_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) atlas_px: vec2<f32>,   // sample position in atlas pixels
    @location(2) local: vec2<f32>,      // pixel offset from the rect center
    @location(3) params: vec4<f32>,     // half_w, half_h, corner_radius, unused
    @location(4) uv_bounds: vec4<f32>,  // min_x, min_y, max_x, max_y (atlas px)
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) atlas_px: vec2<f32>,
    @location(1) local: vec2<f32>,
    @location(2) params: vec4<f32>,
    @location(3) uv_bounds: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection.matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.atlas_px = in.atlas_px;
    out.local = in.local;
    out.params = in.params;
    out.uv_bounds = in.uv_bounds;
    return out;
}

// Signed distance to a rounded box (Inigo Quilez formula)
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2(r);
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(atlas));
    // Clamp half a texel inside the image rect so linear filtering never
    // bleeds neighboring atlas entries.
    let uv_px = clamp(in.atlas_px, in.uv_bounds.xy, in.uv_bounds.zw);
    let color = textureSample(atlas, atlas_sampler, uv_px / dims);

    let half_size = in.params.xy;
    let radius = in.params.z;
    let d = sd_rounded_box(in.local, half_size, radius);
    let mask = 1.0 - smoothstep(-0.5, 0.5, d);
    if (mask <= 0.0) {
        discard;
    }

    // Premultiplied alpha output
    let a = color.a * mask;
    return vec4(color.rgb * a, a);
}
