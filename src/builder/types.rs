use crate::compositor::{Compositor, SceneNode};
use crate::input::{InputState, ViewId};
use crate::layout::ComputedBounds;
use crate::view::ViewContext;

use super::Element;

// ---------------------------------------------------------------------------
// RenderResult -- SceneNodes + hit regions for interactive elements
// ---------------------------------------------------------------------------

/// Result of rendering an Element tree. Contains the visual SceneNodes
/// and hit region data for elements that have event handlers.
pub struct RenderResult {
    pub nodes: Vec<SceneNode>,
    pub hit_regions: Vec<HitRegionEntry>,
}

/// A hit region generated from an Element with event handlers.
pub struct HitRegionEntry {
    pub view_id: ViewId,
    pub bounds: ComputedBounds,
    pub focusable: bool,
}

/// Render an Element tree and push results to the compositor and input state.
/// This is the app-facing entry point -- app code calls this instead of
/// touching Compositor or SceneNode directly (Mantra-01).
pub fn render_element_to_compositor(
    element: &Element,
    compositor: &mut Compositor,
    input: &mut InputState,
    cx: &mut ViewContext,
) -> RenderResult {
    let result = element.render_interactive(cx);
    for node in &result.nodes {
        compositor.push(node.clone());
    }
    for entry in &result.hit_regions {
        input.register_hit_region(
            entry.view_id,
            entry.bounds.x,
            entry.bounds.y,
            entry.bounds.width,
            entry.bounds.height,
            entry.focusable,
        );
    }
    result
}
