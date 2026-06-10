// Analytic drop shadow for rounded rects -- Evan Wallace's approximation
// (https://madebyevan.com/shaders/fast-rounded-rectangle-shadows/, CC0).
// A Gaussian blur of a rounded box is separable: the X axis integrates to a
// closed form via the error function (polynomial approximation below); the
// Y axis is approximated with 4 Gaussian samples. No extra render pass:
// the shadow is plain geometry evaluated per fragment.

struct Projection {
    matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> projection: Projection;

struct VertexInput {
    @location(0) position: vec2<f32>,
    // Pixel offset from the shadow rect center (covers the padded quad).
    @location(1) local: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) params: vec4<f32>, // half_w, half_h, corner_radius, sigma
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) params: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection.matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.local = in.local;
    out.color = in.color;
    out.params = in.params;
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = in.params.xy;
    let corner = in.params.z;
    let sigma = max(in.params.w, 0.0001);
    let point = in.local;

    // The Gaussian is negligible past 3 sigma; restrict the integration
    // range so the 4 samples are spent where the signal lives.
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

    // Premultiplied alpha output
    let a = in.color.a * alpha;
    return vec4(in.color.rgb * a, a);
}
