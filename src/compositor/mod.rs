pub(crate) mod drawing;
mod layer;
mod layer_ops;
mod scene;
mod vertex;

#[cfg(test)]
mod tests;

pub use drawing::RoundedRectParams;
pub use layer::{Layer, LayerEffect, LayerId};
pub use scene::{SceneNode, TextNodeKey};
pub use vertex::{QuadVertex, RectSdfVertex};

/// GPU resources needed for layer texture resolution and compositing.
pub struct ResolveResources<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
    pub composite_bgl: &'a wgpu::BindGroupLayout,
    pub opacity_bgl: &'a wgpu::BindGroupLayout,
    pub sampler: &'a wgpu::Sampler,
}

pub struct Compositor {
    layers: Vec<Layer>,
    next_layer_id: u32,
    sorted: bool,
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
        };
        comp.layers.push(Layer::new(LayerId::DEFAULT, 0));
        comp
    }

    pub fn begin_frame(&mut self) {
        for layer in &mut self.layers {
            layer.begin_frame();
        }
    }

    pub fn resolve(&mut self, res: &ResolveResources<'_>) {
        if !self.sorted {
            self.layers.sort_by_key(|l| l.z_order);
            self.sorted = true;
        }

        let width = res.width.max(1);
        let height = res.height.max(1);

        for layer in &mut self.layers {
            layer.resolve_dirty();
            layer.ensure_texture(res, width, height);

            if layer.dirty {
                layer.rebuild_quad_geometry(res.device, res.queue);
                layer.rebuild_sdf_geometry(res.device, res.queue);
                log::debug!(
                    "Layer {:?} dirty: {} quads, {} sdf_rects, {} text nodes",
                    layer.id,
                    layer.quad_index_count / 6,
                    layer.sdf_index_count / 6,
                    layer.text_nodes().len()
                );
            }
        }
    }
}
