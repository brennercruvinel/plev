use crate::layout::{Align, Justify};

use super::builder::Ui;
use super::node::Visual;
use super::theme::{Accent, UiTheme};

// ---------------------------------------------------------------------------
// NodeRef -- chaining modifiers on the last pushed node
// ---------------------------------------------------------------------------

/// Handle returned by builder methods. Pass back to `Ui::modify()` for chaining.
#[derive(Clone, Copy)]
pub struct NodeRef {
    pub idx: usize,
}

impl Ui {
    pub fn modify(&mut self, nr: NodeRef) -> NodeMod<'_> {
        NodeMod {
            ui: self,
            idx: nr.idx,
        }
    }
}

/// Mutable modifier handle -- borrows Ui safely.
pub struct NodeMod<'a> {
    pub(super) ui: &'a mut Ui,
    idx: usize,
}

impl<'a> NodeMod<'a> {
    fn node(&mut self) -> &mut super::node::UiNode {
        &mut self.ui.nodes[self.idx]
    }

    fn theme(&self) -> &UiTheme {
        &self.ui.theme
    }

    pub fn pad(mut self, y: f32, x: f32) -> Self {
        self.node().layout.padding = [y, x, y, x];
        self
    }

    pub fn gap(mut self, g: f32) -> Self {
        self.node().layout.gap = g;
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.node().layout.width = Some(w);
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.node().layout.height = Some(h);
        self
    }

    pub fn flex(mut self, grow: f32) -> Self {
        self.node().layout.flex_grow = grow;
        self
    }

    pub fn align_center(mut self) -> Self {
        self.node().layout.align = Align::Center;
        self
    }

    pub fn justify_between(mut self) -> Self {
        self.node().layout.justify = Justify::SpaceBetween;
        self
    }

    pub fn bg(mut self, color: [f32; 4]) -> Self {
        match &mut self.node().visual {
            Visual::Box { bg, .. } => *bg = color,
            Visual::None => {
                self.node().visual = Visual::Box {
                    bg: color,
                    border_color: [0.0; 4],
                    border_width: 0.0,
                    corner_radius: 0.0,
                };
            }
            _ => {}
        }
        self
    }

    pub fn corner(mut self, r: f32) -> Self {
        match &mut self.node().visual {
            Visual::Box { corner_radius, .. } => *corner_radius = r,
            Visual::None => {
                self.node().visual = Visual::Box {
                    bg: [0.0; 4],
                    border_color: [0.0; 4],
                    border_width: 0.0,
                    corner_radius: r,
                };
            }
            _ => {}
        }
        self
    }

    pub fn border(mut self, w: f32, color: [f32; 4]) -> Self {
        if let Visual::Box {
            border_width,
            border_color,
            ..
        } = &mut self.node().visual
        {
            *border_width = w;
            *border_color = color;
        }
        self
    }

    pub fn on_click(mut self, id: u64) -> Self {
        self.node().click_id = Some(id);
        self
    }

    // Text-specific modifiers
    pub fn size(mut self, s: f32) -> Self {
        if let Visual::Text {
            size, line_height, ..
        } = &mut self.node().visual
        {
            *size = s;
            *line_height = s * 1.4;
        }
        let lh = s * 1.4;
        self.node().layout.height = Some(lh);
        self
    }

    pub fn bold(mut self) -> Self {
        if let Visual::Text { weight, .. } = &mut self.node().visual {
            *weight = 700;
        }
        self
    }

    pub fn semibold(mut self) -> Self {
        if let Visual::Text { weight, .. } = &mut self.node().visual {
            *weight = 600;
        }
        self
    }

    pub fn color(mut self, c: [f32; 4]) -> Self {
        if let Visual::Text { color, .. } = &mut self.node().visual {
            *color = c;
        }
        self
    }

    // Accent modifiers (for badge/button)
    pub fn accent(mut self, a: Accent) -> Self {
        let idx = UiTheme::accent_idx(a);
        let theme = self.theme();
        let bg_color = theme.accent_bg[idx];
        let fg_color = theme.accent_fg[idx];
        if let Visual::Box { bg, .. } = &mut self.node().visual {
            *bg = bg_color;
        }
        // Also update text children
        let children = self.node().children.clone();
        for ci in children {
            if let Visual::Text { color, .. } = &mut self.ui.nodes[ci].visual {
                *color = fg_color;
            }
        }
        self
    }

    pub fn soft(mut self, a: Accent) -> Self {
        let idx = UiTheme::accent_idx(a);
        let theme = self.theme();
        let bg_color = theme.accent_soft_bg[idx];
        let fg_color = theme.accent_soft_fg[idx];
        if let Visual::Box { bg, .. } = &mut self.node().visual {
            *bg = bg_color;
        }
        let children = self.node().children.clone();
        for ci in children {
            if let Visual::Text { color, .. } = &mut self.ui.nodes[ci].visual {
                *color = fg_color;
            }
        }
        self
    }

    pub fn outline(mut self, a: Accent) -> Self {
        let idx = UiTheme::accent_idx(a);
        let theme = self.theme();
        let border_c = theme.accent_bg[idx];
        let fg_color = theme.accent_bg[idx];
        if let Visual::Box {
            bg,
            border_color,
            border_width,
            ..
        } = &mut self.node().visual
        {
            *bg = [0.0, 0.0, 0.0, 0.0];
            *border_color = [border_c[0], border_c[1], border_c[2], 0.5];
            *border_width = 1.0;
        }
        let children = self.node().children.clone();
        for ci in children {
            if let Visual::Text { color, .. } = &mut self.ui.nodes[ci].visual {
                *color = fg_color;
            }
        }
        self
    }

    pub fn ghost(mut self, a: Accent) -> Self {
        let idx = UiTheme::accent_idx(a);
        let theme = self.theme();
        let fg_color = theme.accent_bg[idx];
        if let Visual::Box { bg, .. } = &mut self.node().visual {
            *bg = [0.0, 0.0, 0.0, 0.0];
        }
        let children = self.node().children.clone();
        for ci in children {
            if let Visual::Text { color, .. } = &mut self.ui.nodes[ci].visual {
                *color = fg_color;
            }
        }
        self
    }
}
