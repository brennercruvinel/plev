struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) rect_params: vec4<f32>,   // half_w, half_h, corner_radius, border_w
    @location(4) border_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) rect_params: vec4<f32>,
    @location(3) border_color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection.matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    out.rect_params = in.rect_params;
    out.border_color = in.border_color;
    return out;
}

// Signed distance to a rounded box (Inigo Quilez formula)
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2(r);
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = in.rect_params.xy;
    let radius = in.rect_params.z;
    let border_w = in.rect_params.w;

    // UV is [-1,1], scale to pixel space relative to rect center
    let p = in.uv * half_size;
    let d = sd_rounded_box(p, half_size, radius);

    // Anti-aliased outer edge (1px feather)
    let outer_alpha = 1.0 - smoothstep(-0.5, 0.5, d);
    if (outer_alpha <= 0.0) {
        discard;
    }

    var color: vec4<f32>;
    if (border_w > 0.0) {
        let inner_r = max(radius - border_w, 0.0);
        let d_inner = sd_rounded_box(p, half_size - vec2(border_w), inner_r);
        let inner_alpha = 1.0 - smoothstep(-0.5, 0.5, d_inner);
        // Border outside, fill inside
        color = mix(in.border_color, in.color, inner_alpha);
    } else {
        color = in.color;
    }

    // Premultiplied alpha output
    let a = color.a * outer_alpha;
    return vec4(color.rgb * a, a);
}
