//! View trait definition.

use crate::compositor::SceneNode;
use crate::layout::LayoutStyle;

use super::context::ViewContext;

/// Produces SceneNodes without touching the compositor directly.
pub trait View {
    fn layout(&self) -> LayoutStyle {
        LayoutStyle::default()
    }

    fn children(&self) -> &[Box<dyn View>] {
        &[]
    }

    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode>;
}
