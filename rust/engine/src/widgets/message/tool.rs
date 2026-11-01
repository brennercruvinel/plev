//! A tool of the kit's comment boxes: a square icon button drawn bare,
//! the icon in the muted tone, a wash under the pointer
//! (`NewComment/index.tsx:38`, `QuickComment/Comment/index.tsx:90`). Its
//! size, corner, icon and colors are the owner's tokens
//! ([`ToolTokens`]); it keeps only the press and its name.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::tokens::ToolTokens;
use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

#[derive(Clone, Debug)]
pub struct Tool {
    pub icon: &'static str,
    /// What a reader hears.
    pub label: &'static str,
    press: Press,
    sem: Option<SemId>,
}

impl Tool {
    pub fn new(icon: &'static str, label: &'static str) -> Self {
        Self {
            icon,
            label,
            press: Press::default(),
            sem: None,
        }
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn set_focused(&mut self, on: bool) {
        self.press.focused = on;
    }

    pub fn is_focused(&self) -> bool {
        self.press.focused
    }

    /// A press and a release inside it: `clicked`.
    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.press.handle(event, bounds, false)
    }

    pub fn render(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        r: Rect,
        t: &ToolTokens,
        theme: &Theme,
    ) {
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [r.x, r.y, r.w, r.h]);
        }
        if self.press.hovered || self.press.pressed {
            c.push_to_layer(layer, rounded_rect(r.x, r.y, r.w, r.h, t.radius, t.hover.0));
        }
        if self.press.focused {
            c.push_to_layer(layer, focus_ring(r, t.radius, theme));
        }
        let (cx, cy) = r.center();
        let at = (cx - t.icon / 2.0, cy - t.icon / 2.0);
        if let Some(node) = icons::icon_at(self.icon, t.icon, t.color.0, at.0, at.1) {
            c.push_to_layer(layer, node);
        }
    }

    /// Declares the tool as `id`: a `Role::Button` named by its label,
    /// with `activate`. What it does is the owner's.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Button, self.label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.action("activate");
        node
    }
}
