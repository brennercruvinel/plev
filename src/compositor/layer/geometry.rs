use super::{INITIAL_IB_SIZE, INITIAL_VB_SIZE, Layer};
use crate::compositor::scene::SceneNode;
use crate::compositor::vertex::{QuadVertex, RectSdfVertex};
use crate::gpu_vec::GpuVec;

/// Whether an axis-aligned box is entirely outside the `[0,0]..viewport` rect.
fn outside_viewport(viewport: (f32, f32), x: f32, y: f32, w: f32, h: f32) -> bool {
    x + w <= 0.0 || y + h <= 0.0 || x >= viewport.0 || y >= viewport.1
}

/// Bounding box of tessellated path vertices, or `None` for empty paths.
fn path_bounds(vertices: &[QuadVertex]) -> Option<(f32, f32, f32, f32)> {
    let first = vertices.first()?;
    let (mut min_x, mut min_y) = (first.position[0], first.position[1]);
    let (mut max_x, mut max_y) = (min_x, min_y);
    for v in &vertices[1..] {
        min_x = min_x.min(v.position[0]);
        min_y = min_y.min(v.position[1]);
        max_x = max_x.max(v.position[0]);
        max_y = max_y.max(v.position[1]);
    }
    Some((min_x, min_y, max_x - min_x, max_y - min_y))
}

impl Layer {
    /// Rebuild CPU-side quad geometry (rects + paths), skipping nodes whose
    /// bounds are entirely outside the viewport. Returns the number of nodes
    /// culled.
    pub(crate) fn build_quad_geometry(&mut self, viewport: (f32, f32)) -> u32 {
        self.quad_vertices.clear();
        self.quad_indices.clear();
        let mut culled = 0u32;

        for node in &self.nodes {
            match node {
                SceneNode::Rect { x, y, w, h, color } => {
                    if outside_viewport(viewport, *x, *y, *w, *h) {
                        culled += 1;
                        continue;
                    }
                    let base = self.quad_vertices.len() as u32;
                    self.quad_vertices.extend_from_slice(&[
                        QuadVertex {
                            position: [*x, *y],
                            color: *color,
                        },
                        QuadVertex {
                            position: [x + w, *y],
                            color: *color,
                        },
                        QuadVertex {
                            position: [x + w, y + h],
                            color: *color,
                        },
                        QuadVertex {
                            position: [*x, y + h],
                            color: *color,
                        },
                    ]);
                    self.quad_indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base + 2,
                        base + 3,
                        base,
                    ]);
                }
                SceneNode::Path { data } => {
                    if let Some((bx, by, bw, bh)) = path_bounds(&data.vertices)
                        && outside_viewport(viewport, bx, by, bw, bh)
                    {
                        culled += 1;
                        continue;
                    }
                    let base = self.quad_vertices.len() as u32;
                    self.quad_vertices.extend_from_slice(&data.vertices);
                    self.quad_indices
                        .extend(data.indices.iter().map(|i| i + base));
                }
                SceneNode::RoundedRect { .. } | SceneNode::Text { .. } => {}
            }
        }

        self.quad_index_count = self.quad_indices.len() as u32;
        culled
    }

    pub(crate) fn upload_quad_geometry(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.quad_vertices.is_empty() {
            let vb = self.quad_vb.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_quad_vb",
                    wgpu::BufferUsages::VERTEX,
                    INITIAL_VB_SIZE,
                )
            });
            vb.upload(device, queue, &self.quad_vertices);

            let ib = self.quad_ib.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_quad_ib",
                    wgpu::BufferUsages::INDEX,
                    INITIAL_IB_SIZE,
                )
            });
            ib.upload(device, queue, &self.quad_indices);
        }
    }

    /// Rebuild CPU-side SDF geometry (rounded rects), skipping nodes entirely
    /// outside the viewport. Returns the number of nodes culled.
    pub(crate) fn build_sdf_geometry(&mut self, viewport: (f32, f32)) -> u32 {
        self.sdf_vertices.clear();
        self.sdf_indices.clear();
        let mut culled = 0u32;

        for node in &self.nodes {
            if let SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                color,
                corner_radius,
                border_width,
                border_color,
            } = node
            {
                if outside_viewport(viewport, *x, *y, *w, *h) {
                    culled += 1;
                    continue;
                }
                let hw = w / 2.0;
                let hh = h / 2.0;
                let params = [hw, hh, *corner_radius, *border_width];

                let base = self.sdf_vertices.len() as u32;
                self.sdf_vertices.extend_from_slice(&[
                    RectSdfVertex {
                        position: [*x, *y],
                        uv: [-1.0, -1.0],
                        color: *color,
                        rect_params: params,
                        border_color: *border_color,
                    },
                    RectSdfVertex {
                        position: [x + w, *y],
                        uv: [1.0, -1.0],
                        color: *color,
                        rect_params: params,
                        border_color: *border_color,
                    },
                    RectSdfVertex {
                        position: [x + w, y + h],
                        uv: [1.0, 1.0],
                        color: *color,
                        rect_params: params,
                        border_color: *border_color,
                    },
                    RectSdfVertex {
                        position: [*x, y + h],
                        uv: [-1.0, 1.0],
                        color: *color,
                        rect_params: params,
                        border_color: *border_color,
                    },
                ]);
                self.sdf_indices.extend_from_slice(&[
                    base,
                    base + 1,
                    base + 2,
                    base + 2,
                    base + 3,
                    base,
                ]);
            }
        }

        self.sdf_index_count = self.sdf_indices.len() as u32;
        culled
    }

    pub(crate) fn upload_sdf_geometry(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.sdf_vertices.is_empty() {
            let vb = self.sdf_vb.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_sdf_vb",
                    wgpu::BufferUsages::VERTEX,
                    INITIAL_VB_SIZE,
                )
            });
            vb.upload(device, queue, &self.sdf_vertices);

            let ib = self.sdf_ib.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_sdf_ib",
                    wgpu::BufferUsages::INDEX,
                    INITIAL_IB_SIZE,
                )
            });
            ib.upload(device, queue, &self.sdf_indices);
        }
    }
}
