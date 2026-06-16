use std::f32::consts::PI;

pub struct OrbitCamera {
    pub target: [f32; 3],
    pub distance: f32,
    pub azimuth: f32,
    pub elevation: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    az_target: f32,
    el_target: f32,
    dist_target: f32,
    tgt_target: [f32; 3],
}

impl OrbitCamera {
    pub fn new(target: [f32; 3]) -> Self {
        let dist = 32.0;
        let az = PI / 4.0;
        let el = 0.6;
        Self {
            target, distance: dist, azimuth: az, elevation: el,
            fov: 50.0, near: 0.1, far: 300.0,
            az_target: az, el_target: el, dist_target: dist, tgt_target: target,
        }
    }

    pub fn rotate(&mut self, daz: f32, del: f32) {
        self.az_target += daz;
        self.el_target = (self.el_target + del).clamp(0.05, PI / 2.1);
    }

    pub fn zoom(&mut self, d: f32) {
        self.dist_target = (self.dist_target + d).clamp(5.0, 50.0);
    }

    pub fn pan(&mut self, dx: f32, dz: f32) {
        let (sa, ca) = (self.azimuth.sin(), self.azimuth.cos());
        self.tgt_target[0] += ca * dx - sa * dz;
        self.tgt_target[2] += -sa * dx - ca * dz;
    }

    pub fn update(&mut self) {
        let t = 0.15;
        self.azimuth += (self.az_target - self.azimuth) * t;
        self.elevation += (self.el_target - self.elevation) * t;
        self.distance += (self.dist_target - self.distance) * t;
        for i in 0..3 { self.target[i] += (self.tgt_target[i] - self.target[i]) * t; }
    }

    pub fn position(&self) -> [f32; 3] {
        let (se, ce) = (self.elevation.sin(), self.elevation.cos());
        let (sa, ca) = (self.azimuth.sin(), self.azimuth.cos());
        [
            self.target[0] + self.distance * ce * sa,
            self.target[1] + self.distance * se,
            self.target[2] + self.distance * ce * ca,
        ]
    }

    /// Returns column-major [f32; 16] for wgpu/WGSL (same convention as phi engine's ortho_projection)
    pub fn view_proj_flat(&self, aspect: f32) -> [f32; 16] {
        let view = look_at_col_major(self.position(), self.target);
        let proj = perspective_col_major(self.fov.to_radians(), aspect, self.near, self.far);
        mat4_mul_col_major(&proj, &view)
    }
}

/// Column-major look_at: output[0..4] = column 0, [4..8] = column 1, etc.
fn look_at_col_major(eye: [f32; 3], tgt: [f32; 3]) -> [f32; 16] {
    let f = norm(sub(tgt, eye));
    let r = norm(cross(f, [0.0, 1.0, 0.0]));
    let u = cross(r, f);
    // Column-major: m[col*4 + row]
    [
        // col 0
        r[0], u[0], -f[0], 0.0,
        // col 1
        r[1], u[1], -f[1], 0.0,
        // col 2
        r[2], u[2], -f[2], 0.0,
        // col 3
        -dot(r, eye), -dot(u, eye), dot(f, eye), 1.0,
    ]
}

/// Column-major perspective for wgpu (Z: [0,1])
fn perspective_col_major(fov: f32, asp: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fov / 2.0).tan();
    let nf = 1.0 / (near - far);
    [
        // col 0
        f / asp, 0.0, 0.0, 0.0,
        // col 1
        0.0, f, 0.0, 0.0,
        // col 2
        0.0, 0.0, far * nf, -1.0,
        // col 3
        0.0, 0.0, near * far * nf, 0.0,
    ]
}

/// Column-major 4x4 matrix multiply: C = A * B
/// m[col*4+row]
fn mat4_mul_col_major(a: &[f32; 16], b: &[f32; 16]) -> [f32; 16] {
    let mut o = [0.0_f32; 16];
    for j in 0..4 { // output column
        for i in 0..4 { // output row
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k * 4 + i] * b[j * 4 + k];
            }
            o[j * 4 + i] = sum;
        }
    }
    o
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn norm(v: [f32; 3]) -> [f32; 3] { let l = dot(v,v).sqrt(); if l < 1e-10 { [0.0,0.0,1.0] } else { [v[0]/l,v[1]/l,v[2]/l] } }
