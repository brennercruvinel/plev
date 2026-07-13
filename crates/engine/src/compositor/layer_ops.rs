use super::{Compositor, Layer, LayerEffect, LayerId};

impl Compositor {
    pub fn create_layer(&mut self, z_order: i32) -> LayerId {
        let id = LayerId(self.next_layer_id);
        self.next_layer_id += 1;
        self.layers.push(Layer::new(id, z_order));
        self.sorted = false;
        id
    }

    pub fn remove_layer(&mut self, id: LayerId) {
        if id == LayerId::DEFAULT {
            log::warn!("Cannot remove the default layer");
            return;
        }
        self.layers.retain(|l| l.id != id);
    }

    /// Change a layer's z-order. Marks the layer order dirty only when the
    /// value actually changes, so `resolve` skips re-sorting otherwise.
    pub fn set_layer_z_order(&mut self, id: LayerId, z_order: i32) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id)
            && layer.z_order != z_order
        {
            layer.z_order = z_order;
            self.sorted = false;
        }
    }

    pub fn set_layer_opacity(&mut self, id: LayerId, opacity: f32) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.opacity = opacity.clamp(0.0, 1.0);
        }
    }

    pub fn set_layer_visible(&mut self, id: LayerId, visible: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.visible = visible;
        }
    }

    pub fn set_layer_effects(&mut self, id: LayerId, effects: Vec<LayerEffect>) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.effects = effects;
        }
    }

    pub fn set_layer_clip_rect(&mut self, id: LayerId, rect: Option<(u32, u32, u32, u32)>) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.clip_rect = rect;
        }
    }

    pub fn layer_has_effects(&self, id: LayerId) -> bool {
        self.layers
            .iter()
            .find(|l| l.id == id)
            .is_some_and(|l| !l.effects.is_empty())
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn mark_layer_clean(&mut self, id: LayerId) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == id) {
            layer.dirty = false;
        }
    }
}
