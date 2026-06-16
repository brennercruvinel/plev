use super::pipeline::{LineVertex, Vertex3D};

pub struct MeshData { pub verts: Vec<Vertex3D>, pub idxs: Vec<u32> }
pub struct LineData { pub verts: Vec<LineVertex> }

impl MeshData {
    pub fn new() -> Self { Self { verts: Vec::new(), idxs: Vec::new() } }
    pub fn extend(&mut self, o: &MeshData) {
        let b = self.verts.len() as u32;
        self.verts.extend_from_slice(&o.verts);
        self.idxs.extend(o.idxs.iter().map(|i| i + b));
    }
}
impl LineData {
    pub fn new() -> Self { Self { verts: Vec::new() } }
    pub fn extend(&mut self, o: &LineData) { self.verts.extend_from_slice(&o.verts); }
}

/// Floor plane (XZ at y)
pub fn floor(x: f32, z: f32, w: f32, d: f32, y: f32, c: [f32; 4]) -> MeshData {
    let n = [0.0, 1.0, 0.0];
    MeshData {
        verts: vec![
            Vertex3D { position: [x,y,z], normal: n, color: c },
            Vertex3D { position: [x+w,y,z], normal: n, color: c },
            Vertex3D { position: [x+w,y,z+d], normal: n, color: c },
            Vertex3D { position: [x,y,z+d], normal: n, color: c },
        ],
        idxs: vec![0,1,2, 0,2,3],
    }
}

/// Wall along X-axis
pub fn wall_x(x: f32, z: f32, w: f32, h: f32, nz: f32, c: [f32; 4]) -> MeshData {
    let n = [0.0, 0.0, nz];
    MeshData {
        verts: vec![
            Vertex3D { position: [x,0.0,z], normal: n, color: c },
            Vertex3D { position: [x+w,0.0,z], normal: n, color: c },
            Vertex3D { position: [x+w,h,z], normal: n, color: c },
            Vertex3D { position: [x,h,z], normal: n, color: c },
        ],
        idxs: vec![0,1,2, 0,2,3],
    }
}

/// Wall along Z-axis
pub fn wall_z(x: f32, z: f32, d: f32, h: f32, nx: f32, c: [f32; 4]) -> MeshData {
    let n = [nx, 0.0, 0.0];
    MeshData {
        verts: vec![
            Vertex3D { position: [x,0.0,z], normal: n, color: c },
            Vertex3D { position: [x,0.0,z+d], normal: n, color: c },
            Vertex3D { position: [x,h,z+d], normal: n, color: c },
            Vertex3D { position: [x,h,z], normal: n, color: c },
        ],
        idxs: vec![0,1,2, 0,2,3],
    }
}

/// A face defined by four corner positions and a normal vector.
type QuadFace = ([f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3]);

/// Box centered at (cx, cy, cz)
pub fn box_mesh(cx: f32, cy: f32, cz: f32, w: f32, h: f32, d: f32, c: [f32; 4]) -> MeshData {
    let (hw, hh, hd) = (w/2.0, h/2.0, d/2.0);
    let (x0,x1) = (cx-hw, cx+hw);
    let (y0,y1) = (cy-hh, cy+hh);
    let (z0,z1) = (cz-hd, cz+hd);
    let mut m = MeshData::new();
    let faces: &[QuadFace] = &[
        ([x0,y0,z1],[x1,y0,z1],[x1,y1,z1],[x0,y1,z1],[0.0,0.0,1.0]),
        ([x1,y0,z0],[x0,y0,z0],[x0,y1,z0],[x1,y1,z0],[0.0,0.0,-1.0]),
        ([x1,y0,z1],[x1,y0,z0],[x1,y1,z0],[x1,y1,z1],[1.0,0.0,0.0]),
        ([x0,y0,z0],[x0,y0,z1],[x0,y1,z1],[x0,y1,z0],[-1.0,0.0,0.0]),
        ([x0,y1,z1],[x1,y1,z1],[x1,y1,z0],[x0,y1,z0],[0.0,1.0,0.0]),
        ([x0,y0,z0],[x1,y0,z0],[x1,y0,z1],[x0,y0,z1],[0.0,-1.0,0.0]),
    ];
    for &(p0,p1,p2,p3,n) in faces {
        let b = m.verts.len() as u32;
        m.verts.extend_from_slice(&[
            Vertex3D { position: p0, normal: n, color: c },
            Vertex3D { position: p1, normal: n, color: c },
            Vertex3D { position: p2, normal: n, color: c },
            Vertex3D { position: p3, normal: n, color: c },
        ]);
        m.idxs.extend_from_slice(&[b,b+1,b+2, b,b+2,b+3]);
    }
    m
}

/// Room wireframe edges
pub fn wireframe(x: f32, z: f32, w: f32, d: f32, h: f32, c: [f32; 4]) -> LineData {
    let mut m = LineData::new();
    let bot = [[x,0.0,z],[x+w,0.0,z],[x+w,0.0,z+d],[x,0.0,z+d]];
    let top = [[x,h,z],[x+w,h,z],[x+w,h,z+d],[x,h,z+d]];
    for i in 0..4 {
        let j = (i+1) % 4;
        m.verts.push(LineVertex { position: bot[i], color: c });
        m.verts.push(LineVertex { position: bot[j], color: c });
        m.verts.push(LineVertex { position: top[i], color: c });
        m.verts.push(LineVertex { position: top[j], color: c });
        m.verts.push(LineVertex { position: bot[i], color: c });
        m.verts.push(LineVertex { position: top[i], color: c });
    }
    m
}

/// Grid lines on XZ plane
pub fn grid(size: f32, divs: u32, c_major: [f32; 4], c_minor: [f32; 4]) -> LineData {
    let mut m = LineData::new();
    let half = size / 2.0;
    let step = size / divs as f32;
    let maj = if divs >= 10 { divs / 10 } else { 1 };
    for i in 0..=divs {
        let t = -half + i as f32 * step;
        let c = if i % maj == 0 { c_major } else { c_minor };
        m.verts.push(LineVertex { position: [t, 0.0, -half], color: c });
        m.verts.push(LineVertex { position: [t, 0.0, half], color: c });
        m.verts.push(LineVertex { position: [-half, 0.0, t], color: c });
        m.verts.push(LineVertex { position: [half, 0.0, t], color: c });
    }
    m
}
