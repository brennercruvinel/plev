use super::geometry::*;
use super::pipeline::*;
use phi::wgpu;

fn room_color(ty: &str, alpha: f32) -> [f32; 4] {
    let (r, g, b) = match ty {
        "clinical"    => (0.533, 0.733, 1.0),
        "admin"       => (0.533, 1.0, 0.733),
        "service"     => (1.0, 0.733, 0.533),
        "commercial"  => (1.0, 0.533, 0.867),
        "circulation" => (0.667, 0.667, 0.667),
        "external"    => (0.4, 0.8, 0.6),
        _             => (0.5, 0.5, 0.5),
    };
    [r, g, b, alpha]
}

struct Room {
    x: f32, z: f32, w: f32, d: f32, h: f32, ty: &'static str,
    furniture: &'static [&'static str],
}

fn clinic_rooms() -> Vec<Room> {
    vec![
        Room { x:0.5, z:5.0, w:3.2, d:3.8, h:2.8, ty:"clinical", furniture:&["maca","mesa","cadeira","armario"] },
        Room { x:4.0, z:5.0, w:3.2, d:3.5, h:2.8, ty:"clinical", furniture:&["maca","mesa","cadeira"] },
        Room { x:0.5, z:7.5, w:3.5, d:4.5, h:2.8, ty:"clinical", furniture:&["maca","mesa","cadeira","armario"] },
        Room { x:4.0, z:7.5, w:3.8, d:3.2, h:2.8, ty:"clinical", furniture:&["maca","mesa"] },
        Room { x:0.5, z:5.0, w:2.0, d:3.2, h:2.8, ty:"clinical", furniture:&["poltrona"] },
        Room { x:2.5, z:5.0, w:2.0, d:3.2, h:2.8, ty:"clinical", furniture:&["poltrona"] },
        Room { x:4.5, z:5.0, w:2.0, d:3.2, h:2.8, ty:"clinical", furniture:&["poltrona"] },
        Room { x:0.5, z:7.5, w:3.95, d:4.4, h:2.8, ty:"clinical", furniture:&["bancada"] },
        Room { x:4.5, z:9.0, w:2.2, d:2.8, h:2.8, ty:"service", furniture:&["bancada"] },
        Room { x:6.8, z:9.0, w:2.2, d:2.5, h:2.8, ty:"service", furniture:&["poltrona"] },
        Room { x:1.0, z:3.5, w:15.0, d:1.3, h:2.8, ty:"circulation", furniture:&[] },
        Room { x:6.0, z:0.5, w:2.5, d:1.8, h:2.8, ty:"external", furniture:&[] },
        Room { x:8.5, z:0.5, w:4.0, d:3.2, h:2.8, ty:"admin", furniture:&["bancada","mesa"] },
        Room { x:12.5, z:0.5, w:3.5, d:3.0, h:2.8, ty:"admin", furniture:&["mesa"] },
        Room { x:13.5, z:3.8, w:4.3, d:5.0, h:2.8, ty:"commercial", furniture:&["bancada"] },
        Room { x:8.0, z:4.5, w:3.6, d:3.5, h:2.8, ty:"admin", furniture:&["mesa","armario"] },
        Room { x:8.0, z:8.0, w:3.0, d:3.5, h:2.8, ty:"admin", furniture:&["mesa"] },
        Room { x:11.5, z:4.5, w:2.8, d:3.1, h:2.8, ty:"admin", furniture:&["mesa"] },
        Room { x:11.5, z:8.0, w:2.8, d:2.5, h:2.8, ty:"service", furniture:&["mesa"] },
        Room { x:14.5, z:8.0, w:1.5, d:1.5, h:2.8, ty:"service", furniture:&[] },
        Room { x:14.5, z:9.5, w:1.8, d:2.5, h:2.8, ty:"service", furniture:&[] },
        Room { x:16.0, z:8.0, w:2.0, d:1.8, h:2.8, ty:"service", furniture:&[] },
        Room { x:0.5, z:12.5, w:5.0, d:4.0, h:2.8, ty:"external", furniture:&[] },
        Room { x:8.0, z:11.0, w:3.5, d:2.8, h:2.8, ty:"external", furniture:&[] },
        Room { x:5.5, z:12.0, w:6.0, d:1.3, h:2.8, ty:"external", furniture:&[] },
    ]
}

fn make_buf(device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: data.len() as u64,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buf, 0, data);
    buf
}

pub struct ClinicScene {
    pub opaque_vb: wgpu::Buffer,
    pub opaque_ib: wgpu::Buffer,
    pub opaque_count: u32,
    pub transparent_vb: wgpu::Buffer,
    pub transparent_ib: wgpu::Buffer,
    pub transparent_count: u32,
    pub line_vb: wgpu::Buffer,
    pub line_count: u32,
    pub center: [f32; 3],
}

impl ClinicScene {
    pub fn build(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let mut opaque = MeshData::new();
        let mut trans = MeshData::new();
        let mut lines = LineData::new();

        lines.extend(&grid(50.0, 100, [0.188,0.188,0.188,1.0], [0.102,0.102,0.102,1.0]));

        // DEBUG: giant bright triangle at scene center to verify pipeline
        let bright = [1.0, 0.0, 0.0, 1.0]; // solid red
        let n_up = [0.0, 1.0, 0.0];
        opaque.verts.push(Vertex3D { position: [5.0, 0.1, 5.0], normal: n_up, color: bright });
        opaque.verts.push(Vertex3D { position: [15.0, 0.1, 5.0], normal: n_up, color: bright });
        opaque.verts.push(Vertex3D { position: [10.0, 0.1, 15.0], normal: n_up, color: bright });
        let b = (opaque.verts.len() - 3) as u32;
        opaque.idxs.extend_from_slice(&[b, b+1, b+2]);

        let rooms = clinic_rooms();
        let (mut mnx, mut mxx, mut mnz, mut mxz) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);

        for r in &rooms {
            mnx = mnx.min(r.x); mxx = mxx.max(r.x+r.w);
            mnz = mnz.min(r.z); mxz = mxz.max(r.z+r.d);

            let fc = room_color(r.ty, 0.4);   // floor — visible
            let wc = room_color(r.ty, 0.25);  // walls — visible
            let lc = room_color(r.ty, 0.6);   // wireframe — bright

            opaque.extend(&floor(r.x, r.z, r.w, r.d, 0.01, fc));
            trans.extend(&wall_x(r.x, r.z, r.w, r.h, -1.0, wc));
            trans.extend(&wall_x(r.x, r.z+r.d, r.w, r.h, 1.0, wc));
            trans.extend(&wall_z(r.x, r.z, r.d, r.h, -1.0, wc));
            trans.extend(&wall_z(r.x+r.w, r.z, r.d, r.h, 1.0, wc));
            lines.extend(&wireframe(r.x, r.z, r.w, r.d, r.h, lc));

            let fc2 = [1.0, 1.0, 1.0, 0.3];
            let (cx, cz) = (r.x + r.w/2.0, r.z + r.d/2.0);
            for (i, _) in r.furniture.iter().enumerate() {
                let ox = ((i%3) as f32 - 1.0) * (r.w*0.25);
                let oz = ((i/3) as f32 - 0.5) * (r.d*0.25);
                trans.extend(&box_mesh(cx+ox, 0.4, cz+oz, 0.8, 0.6, 0.5, fc2));
            }
        }

        let center = [(mnx+mxx)/2.0, 0.0, (mnz+mxz)/2.0];

        let opaque_vb = make_buf(device, queue, bytemuck::cast_slice(&opaque.verts), wgpu::BufferUsages::VERTEX);
        let opaque_ib = make_buf(device, queue, bytemuck::cast_slice(&opaque.idxs), wgpu::BufferUsages::INDEX);
        let transparent_vb = make_buf(device, queue, bytemuck::cast_slice(&trans.verts), wgpu::BufferUsages::VERTEX);
        let transparent_ib = make_buf(device, queue, bytemuck::cast_slice(&trans.idxs), wgpu::BufferUsages::INDEX);
        let line_vb = make_buf(device, queue, bytemuck::cast_slice(&lines.verts), wgpu::BufferUsages::VERTEX);

        eprintln!("[scene3d] opaque: {} verts, {} idx | trans: {} verts, {} idx | lines: {} verts | center: {:?}",
            opaque.verts.len(), opaque.idxs.len(),
            trans.verts.len(), trans.idxs.len(),
            lines.verts.len(), center);

        Self {
            opaque_vb, opaque_ib, opaque_count: opaque.idxs.len() as u32,
            transparent_vb, transparent_ib, transparent_count: trans.idxs.len() as u32,
            line_vb, line_count: lines.verts.len() as u32,
            center,
        }
    }
}
