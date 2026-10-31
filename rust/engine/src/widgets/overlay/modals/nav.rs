//! The side menu Export and Settings share (`Export/Menu`, the Settings
//! menu): a column of items, each a glyph in a box and a label, the
//! active one lit with its box raised on paper. It is a tab list: its
//! items are pressables ([`ItemRole::Tab`]) under `<id>.section`, and the
//! modal shows the section the active one names.
//!
//! Beside its section at the kit's width; in a narrower room it goes
//! above it, its items as wide as the sheet ([`Nav::stacked`]).

use accord::{Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::Compositor;
use crate::styling::theme::Theme;
use crate::widgets::recipe::focus_ring_inside;
use crate::widgets::widget::Rect;

use super::tokens::{ModalInk, NavTokens};
use super::{Controls, Item, ItemRole, Kind, boxed, child, glyph, middle, raise, text};

/// The menu's items (glyph, label) and which is active.
pub struct Nav {
    pub items: Vec<(&'static str, &'static str)>,
    pub active: usize,
    pub id: Option<SemId>,
}

impl Nav {
    /// The items as pressables under `<id>.section`, the first active.
    pub fn new(items: Vec<(&'static str, &'static str)>, id: Option<&SemId>) -> (Self, Controls) {
        let family = child(id, "section");
        let mut controls = Controls::default();
        for (i, (_, label)) in items.iter().enumerate() {
            let item = Item::new(*label, ItemRole::Tab);
            controls.push(
                Kind::Item(item),
                family.as_ref().map(|f| f.key(i.to_string())),
            );
        }
        controls.choose(0..items.len(), Some(0));
        (
            Self {
                items,
                active: 0,
                id: family,
            },
            controls,
        )
    }

    /// Whether the menu goes above its section: the room is narrower than
    /// the kit's sheet.
    pub fn stacked(rect: Rect, kit_w: f32) -> bool {
        rect.w + 0.5 < kit_w
    }

    pub fn item_h(n: &NavTokens) -> f32 {
        n.box_size + n.item_pad * 2.0
    }

    /// The items down `column` (its padding inside), and the bottom of the
    /// last.
    pub fn rects(&self, column: Rect, n: &NavTokens) -> (Vec<Rect>, f32) {
        let h = Self::item_h(n);
        let w = (column.w - n.pad * 2.0).max(0.0);
        let mut y = column.y + n.pad;
        let mut out = Vec::with_capacity(self.items.len());
        for i in 0..self.items.len() {
            if i > 0 {
                y += n.item_gap;
            }
            out.push(Rect::new(column.x + n.pad, y, w, h));
            y += h;
        }
        (out, y)
    }

    /// `i` active: `true` when it changed.
    pub fn activate(&mut self, controls: &mut Controls, i: usize) -> bool {
        let changed = self.active != i;
        self.active = i.min(self.items.len().saturating_sub(1));
        controls.choose(0..self.items.len(), Some(self.active));
        changed
    }

    pub fn draw(
        &self,
        c: &mut Compositor,
        rects: &[Rect],
        controls: &Controls,
        n: &NavTokens,
        ink: &ModalInk,
        theme: &Theme,
    ) {
        for (i, ((icon, label), r)) in self.items.iter().zip(rects).enumerate() {
            let active = i == self.active;
            let hovered = controls.item(i).is_some_and(|it| it.press.hovered);
            if active || hovered {
                let edge = active.then_some((ink.edge, ink.rule_w));
                boxed(c, *r, n.item_radius, ink.hover, edge);
            }
            let b = Rect::new(r.x + n.item_pad, r.y + n.item_pad, n.box_size, n.box_size);
            if active || hovered {
                if active {
                    raise(c, b, n.box_radius, &ink.raised);
                }
                boxed(c, b, n.box_radius, ink.paper, None);
            } else {
                boxed(c, b, n.box_radius, n.box_rest, None);
            }
            let tint = if active { ink.ink } else { ink.icon };
            glyph(c, icon, n.icon, tint, b);
            let tx = b.right() + n.gap;
            let end = r.right() - n.item_pad_end - if n.chevron > 0.0 { n.chevron } else { 0.0 };
            text(
                c,
                label,
                &n.label,
                tx,
                middle(&n.label, r.y, r.h),
                end - tx,
                ink.ink,
            );
            if active && n.chevron > 0.0 {
                let chev = Rect::new(r.right() - n.item_pad_end - n.chevron, r.y, n.chevron, r.h);
                glyph(c, "chevron-right", n.chevron, ink.icon, chev);
            }
            if controls.0.get(i).is_some_and(|c| c.is_focused()) {
                c.push(focus_ring_inside(*r, n.item_radius, theme));
            }
        }
    }

    /// The tab list `<id>.section`, named `label`, its `selected` the
    /// active key; the items project as its tabs.
    pub fn project(&self, b: &mut SnapshotBuilder, parent: &SemId, label: &str) {
        if let Some(id) = &self.id {
            b.node(id, Role::TabList, label)
                .parent(parent)
                .state("selected", self.active.to_string());
        }
    }
}
