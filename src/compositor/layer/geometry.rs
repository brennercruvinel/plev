use super::{INITIAL_IB_SIZE, INITIAL_VB_SIZE, Layer};
use crate::compositor::scene::SceneNode;
use crate::compositor::vertex::{QuadVertex, RectSdfVertex};
use crate::gpu_vec::GpuVec;

impl Layer {
    pub(crate) fn rebuild_quad_geometry(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.quad_vertices.clear();
        self.quad_indices.clear();

        for node in &self.nodes {
            match node {
                SceneNode::Rect { x, y, w, h, color } => {
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
                    let base = self.quad_vertices.len() as u32;
                    self.quad_vertices.extend_from_slice(&data.vertices);
                    self.quad_indices
                        .extend(data.indices.iter().map(|i| i + base));
                }
                SceneNode::RoundedRect { .. } | SceneNode::Text { .. } => {}
            }
        }

        self.quad_index_count = self.quad_indices.len() as u32;

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

    pub(crate) fn rebuild_sdf_geometry(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.sdf_vertices.clear();
        self.sdf_indices.clear();

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
