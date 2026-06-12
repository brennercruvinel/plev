struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    light_dir: vec4<f32>,
    ambient: vec4<f32>,
    fog_color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> u: Uniforms;

// --- Mesh (triangles with lighting) ---

struct VIn {
    @location(0) pos: vec3<f32>,
    @location(1) norm: vec3<f32>,
    @location(2) col: vec4<f32>,
}
struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) col: vec4<f32>,
    @location(1) norm: vec3<f32>,
    @location(2) wpos: vec3<f32>,
}

@vertex fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    o.clip = u.view_proj * vec4(v.pos, 1.0);
    o.col = v.col;
    o.norm = v.norm;
    o.wpos = v.pos;
    return o;
}

@fragment fn fs_main(f: VOut) -> @location(0) vec4<f32> {
    let n = normalize(f.norm);
    let l = normalize(u.light_dir.xyz);
    let ndotl = abs(dot(n, l));
    let light = u.ambient.rgb * u.ambient.a + vec3(1.0) * ndotl * 0.6;
    var c = vec4(f.col.rgb * light, f.col.a);
    let d = distance(u.camera_pos.xyz, f.wpos);
    let fog = exp(-u.fog_color.a * d);
    c = vec4(mix(u.fog_color.rgb, c.rgb, fog), c.a);
    return vec4(c.rgb * c.a, c.a);
}

// --- Lines (no lighting) ---

struct LIn {
    @location(0) pos: vec3<f32>,
    @location(1) col: vec4<f32>,
}
struct LOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) col: vec4<f32>,
}

@vertex fn vs_line(v: LIn) -> LOut {
    var o: LOut;
    o.clip = u.view_proj * vec4(v.pos, 1.0);
    o.col = v.col;
    return o;
}

@fragment fn fs_line(f: LOut) -> @location(0) vec4<f32> {
    return vec4(f.col.rgb * f.col.a, f.col.a);
}
