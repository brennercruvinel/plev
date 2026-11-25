mod binding;
mod clip;
pub(crate) mod drawing;
mod layer;
mod memory;
mod scene;
mod sequence;
mod stats;
mod vertex;

mod layer_operations;
#[cfg(test)]
mod tests;

use crate::runtime::perf::stages::{self, Stage};

pub use binding::Binding;
pub use clip::{
    ClipRect, DrawRange, clip_to_scissor, intersect_rects, intersect_scissors, merge_text_groups,
};
pub use drawing::{GradientRectParams, RoundedRectParams, ShadowParams};
pub use layer::{Layer, LayerEffect, LayerId};
pub use scene::{SceneNode, TextNodeKey};
pub use sequence::{DrawCommand, DrawKind};
pub use stats::RenderStats;
pub use vertex::{
    BackdropVertex, ImageVertex, QuadVertex, RectSdfVertex, ShadowVertex, gradient_direction,
    shadow_padding, shadow_sigma,
};

/// GPU resources needed for layer texture resolution and compositing.
pub struct ResolveResources<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
    /// MSAA sample count for layer textures (1 = render directly, no resolve).
    pub msaa_samples: u32,
    pub composite_bgl: &'a wgpu::BindGroupLayout,
    pub opacity_bgl: &'a wgpu::BindGroupLayout,
    pub sampler: &'a wgpu::Sampler,
}

pub struct Compositor {
    layers: Vec<Layer>,
    next_layer_id: u32,
    sorted: bool,
    invalidated: bool,
    stats: RenderStats,
    /// Where each bound `SemId` sits this frame, beside the scene (see
    /// `binding`): never part of a layer hash.
    bind_table: binding::BindTable,
    /// The layer `push` and `bind` draw on: the default one, but inside
    /// [`Compositor::on_layer`].
    target: LayerId,
    /// The index of the frame being drawn, as the frame's owner counts
    /// them (see [`Compositor::set_frame_index`]).
    frame_index: u64,
    /// The layers the last `window::draw_frame` encoded, in order, with
    /// the draw calls of their passes; reused frame to frame.
    pub(crate) drawn: Vec<(LayerId, u32)>,
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new()
    }
}

impl Compositor {
    pub fn new() -> Self {
        let mut comp = Self {
            layers: Vec::new(),
            next_layer_id: 1,
            sorted: true,
            invalidated: false,
            stats: RenderStats::default(),
            bind_table: binding::BindTable::default(),
            target: LayerId::DEFAULT,
            frame_index: 0,
            drawn: Vec::new(),
        };
        comp.layers.push(Layer::new(LayerId::DEFAULT, 0));
        comp
    }

    pub fn begin_frame(&mut self) {
        for layer in &mut self.layers {
            layer.begin_frame();
        }
        self.bind_table.begin_frame();
        self.target = LayerId::DEFAULT;
    }

    /// Runs `draw` with `push` and `bind` aimed at `layer`: a widget that
    /// draws through `push` (and the widgets it composes) lands on the
    /// layer its owner chose, as a sidebar sliding over a page that uses
    /// layers of its own. The target before is restored after.
    pub fn on_layer<R>(&mut self, layer: LayerId, draw: impl FnOnce(&mut Self) -> R) -> R {
        let before = std::mem::replace(&mut self.target, layer);
        let out = draw(self);
        self.target = before;
        out
    }

    /// The layer `push` and `bind` aim at now: the default one, or the
    /// one [`Compositor::on_layer`] chose. A widget that draws to a layer
    /// by its id passes this to land where its owner drew it.
    pub fn target(&self) -> LayerId {
        self.target
    }

    /// Request a render regardless of scene changes. Called on input, resize,
    /// and animation ticks so the render loop wakes up.
    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    /// Whether a new frame should be rendered: an external invalidation is
    /// pending, layer order changed, or some layer's scene differs from what
    /// was last rendered.
    pub fn needs_render(&self) -> bool {
        if self.invalidated || !self.sorted {
            return true;
        }
        self.layers
            .iter()
            .any(|l| l.dirty || l.compute_hash() != l.prev_hash)
    }

    /// Tag the frames drawn from now on with `index` (a window host gives
    /// its frame counter before each frame): the GPU timestamps of a frame
    /// carry it, so a reading that comes back later finds its frame.
    pub fn set_frame_index(&mut self, index: u64) {
        self.frame_index = index;
    }

    /// The index given by [`Compositor::set_frame_index`]; 0 until then.
    pub fn frame_index(&self) -> u64 {
        self.frame_index
    }

    /// The layers the last `window::draw_frame` redrew, in encode order,
    /// each with the draw calls of its layer passes (a backdrop blur's
    /// included). Empty before the first frame, and for a frame that
    /// redrew no layer.
    pub fn drawn_layers(&self) -> &[(LayerId, u32)] {
        &self.drawn
    }

    /// Stats collected for the most recent frame.
    pub fn stats(&self) -> RenderStats {
        self.stats
    }

    /// Record encode-phase stats for the current frame (called by the render
    /// loop after submitting command buffers).
    pub fn record_encode_stats(&mut self, draw_calls: u32, glyphs: u32, encode_micros: u64) {
        self.stats.draw_calls = draw_calls;
        self.stats.glyphs = glyphs;
        self.stats.encode_micros = encode_micros;
        log::debug!("frame stats: {:?}", self.stats);
    }

    pub fn resolve(&mut self, res: &ResolveResources<'_>) {
        let _probe = stages::span(Stage::Resolve);
        let start = web_time::Instant::now();

        let width = res.width.max(1);
        let height = res.height.max(1);

        // Texture (re)creation first: a resize marks layers dirty so their
        // geometry is rebuilt against the new viewport below.
        for layer in &mut self.layers {
            layer.ensure_texture(res, width, height);
        }

        self.resolve_scene((width as f32, height as f32));

        for layer in &mut self.layers {
            if layer.dirty {
                layer.upload_quad_geometry(res.device, res.queue);
                layer.upload_sdf_geometry(res.device, res.queue);
                layer.upload_shadow_geometry(res.device, res.queue);
                layer.upload_image_geometry(res.device, res.queue);
                layer.upload_backdrop_geometry(res.device, res.queue);
                log::debug!(
                    "Layer {:?} dirty: {} quads, {} sdf_rects, {} text nodes",
                    layer.id,
                    layer.quad_index_count / 6,
                    layer.sdf_index_count / 6,
                    layer.text_nodes().len()
                );
            }
        }

        self.stats.resolve_micros = start.elapsed().as_micros() as u64;
    }

    /// CPU-side part of `resolve`: sorts layers when order changed, detects
    /// dirty layers, rebuilds their geometry with viewport culling, and
    /// collects frame stats. Separated from GPU uploads so it is testable
    /// without a device (headless tests drive the same dirty-tracking the
    /// render loop uses).
    pub fn resolve_scene(&mut self, viewport: (f32, f32)) {
        let start = web_time::Instant::now();

        if !self.sorted {
            self.layers.sort_by_key(|l| l.z_order);
            self.sorted = true;
        }

        self.stats = RenderStats {
            layers_total: self.layers.len() as u32,
            ..RenderStats::default()
        };

        for layer in &mut self.layers {
            layer.resolve_dirty();
            if layer.dirty {
                let _probe = stages::span(Stage::Geometry).items(layer.nodes.len() as u64);
                let culled = layer.build_geometry(viewport);
                self.stats.layers_redrawn += 1;
                self.stats.nodes_culled += culled;
            }
            self.stats.quad_vertices += layer.quad_vertices.len() as u32;
            self.stats.sdf_vertices += layer.sdf_vertices.len() as u32;
            self.stats.shadow_vertices += layer.shadow_vertices.len() as u32;
            self.stats.image_vertices += layer.image_vertices.len() as u32;
        }

        // Backdrop blurs sample what is composited below them, so a layer
        // holding one must re-encode whenever any lower layer was redrawn
        // this frame, even if its own scene is unchanged (its geometry is
        // already built and identical -- only the encode repeats).
        let mut below_redrawn = false;
        for layer in &mut self.layers {
            if below_redrawn && !layer.dirty && layer.has_backdrop_nodes() {
                layer.dirty = true;
            }
            below_redrawn |= layer.dirty;
        }

        self.invalidated = false;
        self.stats.resolve_micros = start.elapsed().as_micros() as u64;
    }
}
