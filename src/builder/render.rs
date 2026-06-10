use crate::compositor::SceneNode;
use crate::input::ViewId;
use crate::layout::LayoutEngine;
use crate::view::{View, ViewContext};

use super::element::Element;
use super::emit::emit_scene_nodes;
use super::layout_pipeline::collect_layout_items;
use super::types::{HitRegionEntry, RenderResult};

// ---------------------------------------------------------------------------
// View implementation -- Taffy-powered layout
// ---------------------------------------------------------------------------

impl Element {
    /// Render the Element tree producing SceneNodes and hit regions.
    /// App code should prefer `render_element_to_compositor()` over calling this directly.
    pub fn render_interactive(&self, cx: &mut ViewContext) -> RenderResult {
        let mut items = Vec::new();
        let mut elements = Vec::new();
        collect_layout_items(self, &mut items, &mut elements);

        let mut engine = LayoutEngine::new();
        let bounds = engine.compute(&items, cx.width, cx.height);

        let mut nodes = Vec::new();
        emit_scene_nodes(&elements, &bounds, cx.theme.as_ref(), &mut nodes);

        // Collect hit regions for elements with event handlers
        let mut hit_regions = Vec::new();
        let mut next_view_id = 0u64;
        for (i, &element) in elements.iter().enumerate() {
            let has_click = element.events.on_click.is_some();
            let has_hover = element.events.on_hover.is_some();
            let has_focus = element.events.on_focus.is_some();
            if has_click || has_hover {
                let vid = ViewId(next_view_id);
                next_view_id += 1;
                hit_regions.push(HitRegionEntry {
                    view_id: vid,
                    bounds: bounds[i],
                    focusable: has_focus || has_click,
                });
            }
        }

        RenderResult { nodes, hit_regions }
    }
}

impl View for Element {
    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode> {
        self.render_interactive(cx).nodes
    }
}
