//! The design sidebar's projection, and its text fields (the
//! artboard's width and height) for the page that routes the keys.

use accord::{Role, SemId, SnapshotBuilder};

use crate::styling::theme::Theme;
use crate::widgets::form::TextField;
use crate::widgets::widget::Rect;

use super::{Body, DesignSidebar, End};

/// Where the artboard's width and height are typed: each field's well,
/// after its prefix, under the select (`select_h` tall).
pub(super) fn size_rects(body: Rect, select_h: f32, theme: &Theme) -> [Rect; 2] {
    let d = &theme.navigation.sidebars.canvas.design;
    let y = body.y + select_h + d.gap;
    let fw = (body.w - d.gap - d.unit_w - d.stack_gap) / 2.0;
    [0.0, 1.0].map(|i| {
        let x = body.x + (fw + d.stack_gap) * i;
        Rect::new(
            x + d.field_text_x,
            y,
            (fw - d.field_text_x).max(0.0),
            d.field_h,
        )
    })
}

impl DesignSidebar {
    /// The text fields of the tab shown: the artboard's width and height.
    pub fn text_fields(&mut self) -> Vec<&mut TextField> {
        let tab = self.current_tab();
        self.blocks
            .iter_mut()
            .filter(|b| b.tab == tab)
            .flat_map(|b| match &mut b.body {
                Body::Artboard(_, fields) => fields.iter_mut().collect(),
                Body::Picker(p) if p.focused().is_some() => vec![p.text_mut()],
                _ => Vec::new(),
            })
            .collect()
    }

    /// Which of them has the keys, in the same order.
    pub fn focused_fields(&self) -> Vec<bool> {
        let tab = self.current_tab();
        self.blocks
            .iter()
            .filter(|b| b.tab == tab)
            .flat_map(|b| match &b.body {
                Body::Artboard(_, fields) => fields.iter().map(|f| f.is_focused()).collect(),
                Body::Picker(p) if p.focused().is_some() => vec![true],
                _ => Vec::new(),
            })
            .collect()
    }

    /// Their rects, in the same order.
    pub fn field_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        self.layout(bounds, theme)
            .into_iter()
            .flat_map(|l| match &self.blocks[l.index].body {
                Body::Artboard(s, _) => size_rects(l.body, s.height_in(theme), theme).to_vec(),
                Body::Picker(p) if p.focused().is_some() => {
                    p.ime_area(l.body, theme).into_iter().collect()
                }
                _ => Vec::new(),
            })
            .collect()
    }

    /// A `Role::Region` named `label`: the Share button, the tabs, and
    /// per group of the current tab a `Role::Group` with its switch,
    /// selects, tabs and tiles.
    #[track_caller]
    pub fn project(
        &self,
        b: &mut SnapshotBuilder,
        id: &SemId,
        label: &str,
        parent: Option<&SemId>,
    ) {
        let mut node = b.node(id, Role::Region, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        if let Some(at) = self.share.sem_id() {
            let mut node = b.node(at, Role::Button, "Share");
            node.parent(id).action("activate");
        }
        if let Some(at) = self.tabs.sem_id() {
            self.tabs.project(b, at, "Design or animation", Some(id));
        }
        let tab = self.current_tab();
        for (bi, block) in self.blocks.iter().enumerate().filter(|(_, b)| b.tab == tab) {
            let Some(at) = id.child("block").ok().map(|f| f.key(bi.to_string())) else {
                continue;
            };
            b.node(&at, Role::Group, block.title.as_str()).parent(id);
            if let End::Switch(s) = &block.end
                && let Some(sid) = s.sem_id()
            {
                s.project(b, sid, block.title.as_str(), Some(&at));
            }
            match &block.body {
                Body::Artboard(s, fields) => {
                    if let Some(sid) = s.sem_id() {
                        s.project(b, sid, "Artboard size", Some(&at));
                    }
                    for (f, name) in fields.iter().zip(["Width", "Height"]) {
                        if let Some(fid) = f.sem_id() {
                            f.project(b, fid, name, Some(&at)).description("px");
                        }
                    }
                }
                Body::Picker(p) => {
                    if let Some(pid) = p.sem_id() {
                        p.project(b, pid, block.title.as_str(), Some(&at));
                    }
                }
                Body::Lens(list) => {
                    for s in list {
                        if let Some(sid) = s.sem_id() {
                            s.project(b, sid, s.selected_label().unwrap_or("Lens"), Some(&at));
                        }
                    }
                }
                Body::Camera(t, _) | Body::Loop(t, _) => {
                    if let Some(tid) = t.sem_id() {
                        t.project(b, tid, block.title.as_str(), Some(&at));
                    }
                }
                Body::Tiles { n, picked, .. } => {
                    for i in 0..*n {
                        let Some(tid) = self.tile_id(bi, i) else {
                            continue;
                        };
                        let name = format!("{} {}", block.title, i + 1);
                        let mut node = b.node(&tid, Role::Button, name);
                        node.parent(&at).state("selected", *picked == Some(i));
                        node.action("activate");
                    }
                }
                _ => {}
            }
        }
    }
}
