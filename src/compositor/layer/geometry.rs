use super::{INITIAL_IB_SIZE, INITIAL_VB_SIZE, Layer};
use crate::compositor::clip::{ClipStack, record_range};
use crate::compositor::scene::SceneNode;
use crate::compositor::vertex::{
    ImageVertex, QuadVertex, RectSdfVertex, ShadowVertex, gradient_direction, shadow_padding,
    shadow_sigma,
};
use crate::gpu_vec::GpuVec;

/// Resolved per-rect parameters shared by the solid and gradient SDF paths.
struct SdfRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: [f32; 4],
    color2: [f32; 4],
    gradient: [f32; 4],
    corner_radius: f32,
    border_width: f32,
    border_color: [f32; 4],
}

/// Whether an axis-aligned box is entirely outside the `[0,0]..viewport` rect.
fn outside_viewport(viewport: (f32, f32), x: f32, y: f32, w: f32, h: f32) -> bool {
    x + w <= 0.0 || y + h <= 0.0 || x >= viewport.0 || y >= viewport.1
}

fn emit_sdf_rect(vertices: &mut Vec<RectSdfVertex>, indices: &mut Vec<u32>, r: &SdfRect) {
    let params = [r.w / 2.0, r.h / 2.0, r.corner_radius, r.border_width];
    let base = vertices.len() as u32;
    let corners = [
        ([r.x, r.y], [-1.0, -1.0]),
        ([r.x + r.w, r.y], [1.0, -1.0]),
        ([r.x + r.w, r.y + r.h], [1.0, 1.0]),
        ([r.x, r.y + r.h], [-1.0, 1.0]),
    ];
    for (position, uv) in corners {
        vertices.push(RectSdfVertex {
            position,
            uv,
            color: r.color,
            rect_params: params,
            border_color: r.border_color,
            color2: r.color2,
            gradient: r.gradient,
        });
    }
    indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
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
    /// bounds are entirely outside the viewport or inside an empty clip.
    /// Returns the number of nodes culled.
    pub(crate) fn build_quad_geometry(&mut self, viewport: (f32, f32)) -> u32 {
        self.quad_vertices.clear();
        self.quad_indices.clear();
        self.quad_ranges.clear();
        let mut clips = ClipStack::default();
        let mut culled = 0u32;

        for node in &self.nodes {
            match node {
                SceneNode::PushClip { x, y, w, h } => clips.push([*x, *y, *w, *h]),
                SceneNode::PopClip => clips.pop(),
                SceneNode::Rect { x, y, w, h, color } => {
                    if outside_viewport(viewport, *x, *y, *w, *h) || clips.is_empty_clip() {
                        culled += 1;
                        continue;
                    }
                    let first_index = self.quad_indices.len() as u32;
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
                    record_range(&mut self.quad_ranges, first_index, 6, clips.current());
                }
                SceneNode::Path { data } => {
                    if clips.is_empty_clip() {
                        culled += 1;
                        continue;
                    }
                    if let Some((bx, by, bw, bh)) = path_bounds(&data.vertices)
                        && outside_viewport(viewport, bx, by, bw, bh)
                    {
                        culled += 1;
                        continue;
                    }
                    let first_index = self.quad_indices.len() as u32;
                    let base = self.quad_vertices.len() as u32;
                    self.quad_vertices.extend_from_slice(&data.vertices);
                    self.quad_indices
                        .extend(data.indices.iter().map(|i| i + base));
                    record_range(
                        &mut self.quad_ranges,
                        first_index,
                        data.indices.len() as u32,
                        clips.current(),
                    );
                }
                _ => {}
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

    /// Rebuild CPU-side SDF geometry (rounded + gradient rects), skipping
    /// nodes entirely outside the viewport. Returns the number of nodes
    /// culled.
    pub(crate) fn build_sdf_geometry(&mut self, viewport: (f32, f32)) -> u32 {
        self.sdf_vertices.clear();
        self.sdf_indices.clear();
        self.sdf_ranges.clear();
        let mut clips = ClipStack::default();
        let mut culled = 0u32;

        for node in &self.nodes {
            let rect = match node {
                SceneNode::PushClip { x, y, w, h } => {
                    clips.push([*x, *y, *w, *h]);
                    continue;
                }
                SceneNode::PopClip => {
                    clips.pop();
                    continue;
                }
                SceneNode::RoundedRect {
                    x,
                    y,
                    w,
                    h,
                    color,
                    corner_radius,
                    border_width,
                    border_color,
                } => SdfRect {
                    x: *x,
                    y: *y,
                    w: *w,
                    h: *h,
                    color: *color,
                    color2: *color,
                    gradient: [0.0; 4],
                    corner_radius: *corner_radius,
                    border_width: *border_width,
                    border_color: *border_color,
                },
                SceneNode::GradientRect {
                    x,
                    y,
                    w,
                    h,
                    color,
                    color2,
                    angle_deg,
                    corner_radius,
                    border_width,
                    border_color,
                } => {
                    let dir = gradient_direction(*angle_deg);
                    SdfRect {
                        x: *x,
                        y: *y,
                        w: *w,
                        h: *h,
                        color: *color,
                        color2: *color2,
                        gradient: [dir[0], dir[1], 1.0, 0.0],
                        corner_radius: *corner_radius,
                        border_width: *border_width,
                        border_color: *border_color,
                    }
                }
                _ => continue,
            };

            if outside_viewport(viewport, rect.x, rect.y, rect.w, rect.h) || clips.is_empty_clip()
            {
                culled += 1;
                continue;
            }
            let first_index = self.sdf_indices.len() as u32;
            emit_sdf_rect(&mut self.sdf_vertices, &mut self.sdf_indices, &rect);
            record_range(&mut self.sdf_ranges, first_index, 6, clips.current());
        }

        self.sdf_index_count = self.sdf_indices.len() as u32;
        culled
    }

    /// Rebuild CPU-side image sprite geometry. Atlas coordinates are stored
    /// in pixels (normalized in the shader), so geometry stays valid across
    /// atlas growth. Returns the number of nodes culled.
    pub(crate) fn build_image_geometry(&mut self, viewport: (f32, f32)) -> u32 {
        self.image_vertices.clear();
        self.image_indices.clear();
        self.image_ranges.clear();
        let mut clips = ClipStack::default();
        let mut culled = 0u32;

        for node in &self.nodes {
            let SceneNode::Image {
                x,
                y,
                w,
                h,
                image,
                corner_radius,
            } = node
            else {
                match node {
                    SceneNode::PushClip { x, y, w, h } => clips.push([*x, *y, *w, *h]),
                    SceneNode::PopClip => clips.pop(),
                    _ => {}
                }
                continue;
            };

            if outside_viewport(viewport, *x, *y, *w, *h) || clips.is_empty_clip() {
                culled += 1;
                continue;
            }

            let params = [w / 2.0, h / 2.0, *corner_radius, 0.0];
            let (ax, ay) = (image.atlas_x as f32, image.atlas_y as f32);
            let (aw, ah) = (image.width as f32, image.height as f32);
            // Clamp sampling half a texel inside the image rect (the shader
            // applies it) so linear filtering never bleeds neighbors.
            let uv_bounds = [ax + 0.5, ay + 0.5, ax + aw - 0.5, ay + ah - 0.5];
            let corners = [
                ([*x, *y], [ax, ay], [-w / 2.0, -h / 2.0]),
                ([x + w, *y], [ax + aw, ay], [w / 2.0, -h / 2.0]),
                ([x + w, y + h], [ax + aw, ay + ah], [w / 2.0, h / 2.0]),
                ([*x, y + h], [ax, ay + ah], [-w / 2.0, h / 2.0]),
            ];

            let first_index = self.image_indices.len() as u32;
            let base = self.image_vertices.len() as u32;
            for (position, atlas_px, local) in corners {
                self.image_vertices.push(ImageVertex {
                    position,
                    atlas_px,
                    local,
                    params,
                    uv_bounds,
                });
            }
            self.image_indices
                .extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
            record_range(&mut self.image_ranges, first_index, 6, clips.current());
        }

        self.image_index_count = self.image_indices.len() as u32;
        culled
    }

    pub(crate) fn upload_image_geometry(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.image_vertices.is_empty() {
            let vb = self.image_vb.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_image_vb",
                    wgpu::BufferUsages::VERTEX,
                    INITIAL_VB_SIZE,
                )
            });
            vb.upload(device, queue, &self.image_vertices);

            let ib = self.image_ib.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_image_ib",
                    wgpu::BufferUsages::INDEX,
                    INITIAL_IB_SIZE,
                )
            });
            ib.upload(device, queue, &self.image_indices);
        }
    }

    /// Rebuild CPU-side analytic shadow geometry. Each shadow is a single
    /// quad expanded by the blur padding and shifted by the offset; the
    /// fragment shader evaluates the blurred rounded-box mask analytically.
    /// Returns the number of nodes culled.
    pub(crate) fn build_shadow_geometry(&mut self, viewport: (f32, f32)) -> u32 {
        self.shadow_vertices.clear();
        self.shadow_indices.clear();
        self.shadow_ranges.clear();
        let mut clips = ClipStack::default();
        let mut culled = 0u32;

        for node in &self.nodes {
            let SceneNode::Shadow {
                x,
                y,
                w,
                h,
                corner_radius,
                blur_radius,
                offset,
                color,
            } = node
            else {
                match node {
                    SceneNode::PushClip { x, y, w, h } => clips.push([*x, *y, *w, *h]),
                    SceneNode::PopClip => clips.pop(),
                    _ => {}
                }
                continue;
            };

            let pad = shadow_padding(*blur_radius);
            let (qx, qy) = (x - pad + offset[0], y - pad + offset[1]);
            let (qw, qh) = (w + 2.0 * pad, h + 2.0 * pad);
            if outside_viewport(viewport, qx, qy, qw, qh) || clips.is_empty_clip() {
                culled += 1;
                continue;
            }

            let params = [
                w / 2.0,
                h / 2.0,
                *corner_radius,
                shadow_sigma(*blur_radius),
            ];
            // Quad corners relative to the (offset) rect center.
            let half = [w / 2.0 + pad, h / 2.0 + pad];
            let corners = [
                ([qx, qy], [-half[0], -half[1]]),
                ([qx + qw, qy], [half[0], -half[1]]),
                ([qx + qw, qy + qh], [half[0], half[1]]),
                ([qx, qy + qh], [-half[0], half[1]]),
            ];

            let first_index = self.shadow_indices.len() as u32;
            let base = self.shadow_vertices.len() as u32;
            for (position, local) in corners {
                self.shadow_vertices.push(ShadowVertex {
                    position,
                    local,
                    color: *color,
                    params,
                });
            }
            self.shadow_indices
                .extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
            record_range(&mut self.shadow_ranges, first_index, 6, clips.current());
        }

        self.shadow_index_count = self.shadow_indices.len() as u32;
        culled
    }

    pub(crate) fn upload_shadow_geometry(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.shadow_vertices.is_empty() {
            let vb = self.shadow_vb.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_shadow_vb",
                    wgpu::BufferUsages::VERTEX,
                    INITIAL_VB_SIZE,
                )
            });
            vb.upload(device, queue, &self.shadow_vertices);

            let ib = self.shadow_ib.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_shadow_ib",
                    wgpu::BufferUsages::INDEX,
                    INITIAL_IB_SIZE,
                )
            });
            ib.upload(device, queue, &self.shadow_indices);
        }
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
