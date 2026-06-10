// Analytic shadow for rounded rects -- Evan Wallace's approximation
// (https://madebyevan.com/shaders/fast-rounded-rectangle-shadows/, CC0).
// A Gaussian blur of a rounded box is separable: the X axis integrates to a
// closed form via the error function (polynomial approximation below); the
// Y axis is approximated with 4 Gaussian samples. No extra render pass:
// the shadow is plain geometry evaluated per fragment.
//
// Two modes, selected per vertex by params2.x:
//   drop  (0): the classic outer shadow; the quad is padded by the blur
//              and pre-shifted by the offset on the CPU.
//   inset (1): CSS `box-shadow: inset` -- the blurred mask is INVERTED
//              (dark/light grows inward from the border), evaluated at
//              `local - offset` and hard-clipped to the rect's rounded
//              SDF so nothing leaks outside the surface.

struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

struct VertexInput {
    @location(0) position: vec2<f32>,
    // Pixel offset from the shadow rect center (covers the padded quad for
    // drop shadows; exactly the rect for inset shadows).
    @location(1) local: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) params: vec4<f32>, // half_w, half_h, corner_radius, sigma
    @location(4) params2: vec4<f32>, // inset, offset_x, offset_y, unused
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) params: vec4<f32>,
    @location(3) params2: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection.matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.local = in.local;
    out.color = in.color;
    out.params = in.params;
    out.params2 = in.params2;
    return out;
}

fn gaussian(x: f32, sigma: f32) -> f32 {
    let pi = 3.141592653589793;
    return exp(-(x * x) / (2.0 * sigma * sigma)) / (sqrt(2.0 * pi) * sigma);
}

// Error function, Abramowitz-Stegun polynomial approximation.
fn erf2(x: vec2<f32>) -> vec2<f32> {
    let s = sign(x);
    let a = abs(x);
    var r = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
    r = r * r;
    return s - s / (r * r);
}

// Blurred mask along the X axis for one scanline of a rounded box.
fn rounded_box_shadow_x(x: f32, y: f32, sigma: f32, corner: f32, half_size: vec2<f32>) -> f32 {
    let delta = min(half_size.y - corner - abs(y), 0.0);
    let curved = half_size.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
    let integral = 0.5 + 0.5 * erf2((vec2(x, x) + vec2(-curved, curved)) * (sqrt(0.5) / sigma));
    return integral.y - integral.x;
}

// Gaussian-blurred coverage of the rounded box at `point`: 4-sample Y
// integration restricted to the +/-3 sigma window where the signal lives.
fn rounded_box_shadow(point: vec2<f32>, sigma: f32, corner: f32, half_size: vec2<f32>) -> f32 {
    let low = point.y - half_size.y;
    let high = point.y + half_size.y;
    let start = clamp(-3.0 * sigma, low, high);
    let end = clamp(3.0 * sigma, low, high);

    let step = (end - start) / 4.0;
    var y = start + step * 0.5;
    var alpha = 0.0;
    for (var i = 0; i < 4; i++) {
        alpha += rounded_box_shadow_x(point.x, point.y - y, sigma, corner, half_size)
            * gaussian(y, sigma) * step;
        y += step;
    }
    return alpha;
}

// Signed distance to a rounded box (Inigo Quilez formula).
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2(r);
    return length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = in.params.xy;
    let corner = in.params.z;
    let sigma = max(in.params.w, 0.0001);

    var alpha: f32;
    if (in.params2.x > 0.5) {
        // Inset: invert the blurred coverage of the rect shifted by the
        // offset (positive offsets pool the shadow at the top/left inside
        // edges, like CSS), clipped to the rect's rounded silhouette.
        let blurred = rounded_box_shadow(in.local - in.params2.yz, sigma, corner, half_size);
        let d = sd_rounded_box(in.local, half_size, corner);
        let rect_mask = 1.0 - smoothstep(-0.5, 0.5, d);
        alpha = (1.0 - blurred) * rect_mask;
    } else {
        alpha = rounded_box_shadow(in.local, sigma, corner, half_size);
    }

    // Premultiplied alpha output
    let a = in.color.a * alpha;
    return vec4(in.color.rgb * a, a);
}
