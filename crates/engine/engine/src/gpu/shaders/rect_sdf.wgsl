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
    @location(5) color2: vec4<f32>,        // second gradient stop (== color when solid)
    @location(6) gradient: vec4<f32>,      // dir_x, dir_y, enabled, unused
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) rect_params: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) color2: vec4<f32>,
    @location(5) gradient: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection.matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    out.rect_params = in.rect_params;
    out.border_color = in.border_color;
    out.color2 = in.color2;
    out.gradient = in.gradient;
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

    // Fill brush: solid color, or 2-stop linear gradient interpolated by the
    // projection of the point onto the gradient direction across the rect.
    var fill = in.color;
    if (in.gradient.z > 0.0) {
        let dir = in.gradient.xy;
        // Extent of the rect along the gradient axis (from center).
        let extent = max(abs(dir.x) * half_size.x + abs(dir.y) * half_size.y, 1e-4);
        let t = clamp(dot(p, dir) / extent * 0.5 + 0.5, 0.0, 1.0);
        fill = mix(in.color, in.color2, t);
    }

    var color: vec4<f32>;
    if (border_w > 0.0) {
        let inner_r = max(radius - border_w, 0.0);
        let d_inner = sd_rounded_box(p, half_size - vec2(border_w), inner_r);
        let inner_alpha = 1.0 - smoothstep(-0.5, 0.5, d_inner);
        // Border outside, fill inside
        color = mix(in.border_color, fill, inner_alpha);
    } else {
        color = fill;
    }

    // Linearize (sRGB theme color → linear; the sRGB surface re-encodes on
    // write), then premultiplied alpha output.
    let a = color.a * outer_alpha;
    let rgb = srgb_to_linear(color.rgb);
    return vec4(rgb * a, a);
}

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}
