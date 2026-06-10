mod geometry;
mod texture;

use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

use crate::compositor::clip::{ClipRect, ClipStack, DrawRange};
use crate::compositor::scene::SceneNode;
use crate::compositor::vertex::{ImageVertex, QuadVertex, RectSdfVertex, ShadowVertex};
use crate::gpu_vec::GpuVec;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(pub u32);

impl LayerId {
    pub const DEFAULT: LayerId = LayerId(0);
}

#[derive(Clone, Debug)]
pub enum LayerEffect {
    Blur { sigma: f32 },
    Shadow { sigma: f32, color: [f32; 4] },
}

pub struct Layer {
    pub id: LayerId,
    pub z_order: i32,
    pub opacity: f32,
    pub visible: bool,
    pub effects: Vec<LayerEffect>,
    pub clip_rect: Option<(u32, u32, u32, u32)>,
    pub(crate) msaa_texture: Option<wgpu::Texture>,
    pub(crate) msaa_view: Option<wgpu::TextureView>,
    pub(crate) texture: Option<wgpu::Texture>,
    pub(crate) texture_view: Option<wgpu::TextureView>,
    pub(crate) composite_bind_group: Option<wgpu::BindGroup>,
    pub(crate) opacity_buffer: Option<wgpu::Buffer>,
    pub(crate) opacity_bind_group: Option<wgpu::BindGroup>,
    pub(crate) nodes: Vec<SceneNode>,
    pub(crate) prev_hash: u64,
    pub(crate) dirty: bool,
    pub(crate) quad_vertices: Vec<QuadVertex>,
    pub(crate) quad_indices: Vec<u32>,
    pub(crate) quad_ranges: Vec<DrawRange>,
    pub(crate) quad_vb: Option<GpuVec>,
    pub(crate) quad_ib: Option<GpuVec>,
    pub(crate) quad_index_count: u32,
    pub(crate) sdf_vertices: Vec<RectSdfVertex>,
    pub(crate) sdf_indices: Vec<u32>,
    pub(crate) sdf_ranges: Vec<DrawRange>,
    pub(crate) sdf_vb: Option<GpuVec>,
    pub(crate) sdf_ib: Option<GpuVec>,
    pub(crate) sdf_index_count: u32,
    pub(crate) shadow_vertices: Vec<ShadowVertex>,
    pub(crate) shadow_indices: Vec<u32>,
    pub(crate) shadow_ranges: Vec<DrawRange>,
    pub(crate) shadow_vb: Option<GpuVec>,
    pub(crate) shadow_ib: Option<GpuVec>,
    pub(crate) shadow_index_count: u32,
    pub(crate) image_vertices: Vec<ImageVertex>,
    pub(crate) image_indices: Vec<u32>,
    pub(crate) image_ranges: Vec<DrawRange>,
    pub(crate) image_vb: Option<GpuVec>,
    pub(crate) image_ib: Option<GpuVec>,
    pub(crate) image_index_count: u32,
    pub(crate) text_vertices: Vec<crate::text::TextVertex>,
    pub(crate) text_indices: Vec<u32>,
    pub(crate) text_ranges: Vec<DrawRange>,
    pub(crate) text_vb: Option<GpuVec>,
    pub(crate) text_ib: Option<GpuVec>,
    pub(crate) text_index_count: u32,
    pub(crate) tex_width: u32,
    pub(crate) tex_height: u32,
}

pub(crate) const INITIAL_VB_SIZE: u64 = 4096;
pub(crate) const INITIAL_IB_SIZE: u64 = 2048;

impl Layer {
    pub(crate) fn new(id: LayerId, z_order: i32) -> Self {
        Self {
            id,
            z_order,
            opacity: 1.0,
            visible: true,
            effects: Vec::new(),
            clip_rect: None,
            msaa_texture: None,
            msaa_view: None,
            texture: None,
            texture_view: None,
            composite_bind_group: None,
            opacity_buffer: None,
            opacity_bind_group: None,
            nodes: Vec::new(),
            prev_hash: 0,
            dirty: true,
            quad_vertices: Vec::new(),
            quad_indices: Vec::new(),
            quad_ranges: Vec::new(),
            quad_vb: None,
            quad_ib: None,
            quad_index_count: 0,
            sdf_vertices: Vec::new(),
            sdf_indices: Vec::new(),
            sdf_ranges: Vec::new(),
            sdf_vb: None,
            sdf_ib: None,
            sdf_index_count: 0,
            shadow_vertices: Vec::new(),
            shadow_indices: Vec::new(),
            shadow_ranges: Vec::new(),
            shadow_vb: None,
            shadow_ib: None,
            shadow_index_count: 0,
            image_vertices: Vec::new(),
            image_indices: Vec::new(),
            image_ranges: Vec::new(),
            image_vb: None,
            image_ib: None,
            image_index_count: 0,
            text_vertices: Vec::new(),
            text_indices: Vec::new(),
            text_ranges: Vec::new(),
            text_vb: None,
            text_ib: None,
            text_index_count: 0,
            tex_width: 0,
            tex_height: 0,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn nodes(&self) -> &[SceneNode] {
        &self.nodes
    }
    pub fn effects(&self) -> &[LayerEffect] {
        &self.effects
    }
    pub fn has_effects(&self) -> bool {
        !self.effects.is_empty()
    }

    pub fn text_nodes(&self) -> Vec<SceneNode> {
        self.nodes
            .iter()
            .filter(|n| matches!(n, SceneNode::Text { .. }))
            .cloned()
            .collect()
    }

    /// Text nodes grouped by the clip rect active where they appear, in
    /// paint order. Consecutive nodes sharing a clip form one group, so the
    /// render loop can resolve each group and scissor it independently.
    pub fn text_node_groups(&self) -> Vec<(Vec<SceneNode>, Option<ClipRect>)> {
        let mut groups: Vec<(Vec<SceneNode>, Option<ClipRect>)> = Vec::new();
        let mut clips = ClipStack::default();

        for node in &self.nodes {
            match node {
                SceneNode::PushClip { x, y, w, h } => clips.push([*x, *y, *w, *h]),
                SceneNode::PopClip => clips.pop(),
                SceneNode::Text { .. } => {
                    let clip = clips.current();
                    match groups.last_mut() {
                        Some((nodes, group_clip)) if *group_clip == clip => {
                            nodes.push(node.clone());
                        }
                        _ => groups.push((vec![node.clone()], clip)),
                    }
                }
                _ => {}
            }
        }
        groups
    }

    pub fn has_quads(&self) -> bool {
        self.quad_index_count > 0
    }

    pub fn quad_draw_ranges(&self) -> &[DrawRange] {
        &self.quad_ranges
    }
    pub fn sdf_draw_ranges(&self) -> &[DrawRange] {
        &self.sdf_ranges
    }
    pub fn shadow_draw_ranges(&self) -> &[DrawRange] {
        &self.shadow_ranges
    }
    pub fn image_draw_ranges(&self) -> &[DrawRange] {
        &self.image_ranges
    }
    pub fn text_draw_ranges(&self) -> &[DrawRange] {
        &self.text_ranges
    }

    pub fn quad_buffers(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, u32)> {
        let vb = self.quad_vb.as_ref()?;
        let ib = self.quad_ib.as_ref()?;
        if self.quad_index_count == 0 {
            return None;
        }
        Some((vb.buffer(), ib.buffer(), self.quad_index_count))
    }

    pub fn has_sdf_rects(&self) -> bool {
        self.sdf_index_count > 0
    }

    pub fn sdf_rect_buffers(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, u32)> {
        let vb = self.sdf_vb.as_ref()?;
        let ib = self.sdf_ib.as_ref()?;
        if self.sdf_index_count == 0 {
            return None;
        }
        Some((vb.buffer(), ib.buffer(), self.sdf_index_count))
    }

    pub fn has_shadows(&self) -> bool {
        self.shadow_index_count > 0
    }

    pub fn shadow_buffers(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, u32)> {
        let vb = self.shadow_vb.as_ref()?;
        let ib = self.shadow_ib.as_ref()?;
        if self.shadow_index_count == 0 {
            return None;
        }
        Some((vb.buffer(), ib.buffer(), self.shadow_index_count))
    }

    pub fn has_images(&self) -> bool {
        self.image_index_count > 0
    }

    pub fn image_buffers(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, u32)> {
        let vb = self.image_vb.as_ref()?;
        let ib = self.image_ib.as_ref()?;
        if self.image_index_count == 0 {
            return None;
        }
        Some((vb.buffer(), ib.buffer(), self.image_index_count))
    }

    pub fn has_text(&self) -> bool {
        self.text_index_count > 0
    }

    pub fn text_buffers(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, u32)> {
        let vb = self.text_vb.as_ref()?;
        let ib = self.text_ib.as_ref()?;
        if self.text_index_count == 0 {
            return None;
        }
        Some((vb.buffer(), ib.buffer(), self.text_index_count))
    }

    pub fn msaa_view(&self) -> Option<&wgpu::TextureView> {
        self.msaa_view.as_ref()
    }
    pub fn texture_view(&self) -> Option<&wgpu::TextureView> {
        self.texture_view.as_ref()
    }

    /// Color attachment for this layer's render pass: with MSAA the pass
    /// renders into the MSAA texture and resolves into the layer texture;
    /// with a single sample it renders directly (no resolve target).
    pub fn render_attachment(&self) -> Option<(&wgpu::TextureView, Option<&wgpu::TextureView>)> {
        match (self.msaa_view.as_ref(), self.texture_view.as_ref()) {
            (Some(msaa), target) => Some((msaa, target)),
            (None, Some(target)) => Some((target, None)),
            (None, None) => None,
        }
    }

    /// Number of glyph quads currently uploaded (6 indices per glyph).
    pub fn glyph_count(&self) -> u32 {
        self.text_index_count / 6
    }
    pub fn composite_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.composite_bind_group.as_ref()
    }
    pub fn opacity_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.opacity_bind_group.as_ref()
    }

    pub fn set_text_data(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: Vec<crate::text::TextVertex>,
        indices: Vec<u32>,
    ) {
        // Single unclipped range covering everything (callers that resolve
        // text per clip group use `set_text_data_with_ranges`).
        let ranges = if indices.is_empty() {
            Vec::new()
        } else {
            vec![DrawRange {
                first_index: 0,
                index_count: indices.len() as u32,
                clip: None,
            }]
        };
        self.set_text_data_with_ranges(device, queue, vertices, indices, ranges);
    }

    pub fn set_text_data_with_ranges(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: Vec<crate::text::TextVertex>,
        indices: Vec<u32>,
        ranges: Vec<DrawRange>,
    ) {
        self.text_index_count = indices.len() as u32;
        self.text_vertices = vertices;
        self.text_indices = indices;
        self.text_ranges = ranges;

        if !self.text_vertices.is_empty() {
            let vb = self.text_vb.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_text_vb",
                    wgpu::BufferUsages::VERTEX,
                    INITIAL_VB_SIZE,
                )
            });
            vb.upload(device, queue, &self.text_vertices);

            let ib = self.text_ib.get_or_insert_with(|| {
                GpuVec::new(
                    device,
                    "layer_text_ib",
                    wgpu::BufferUsages::INDEX,
                    INITIAL_IB_SIZE,
                )
            });
            ib.upload(device, queue, &self.text_indices);
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.nodes.clear();
    }

    pub(crate) fn compute_hash(&self) -> u64 {
        let mut scene_hasher = FxHasher::default();
        for node in &self.nodes {
            node.hash_u64().hash(&mut scene_hasher);
        }
        scene_hasher.finish()
    }

    pub(crate) fn resolve_dirty(&mut self) {
        let hash = self.compute_hash();
        if hash != self.prev_hash {
            self.dirty = true;
            self.prev_hash = hash;
        }
    }
}
