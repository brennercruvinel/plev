//! one materialized surface: a `VerifiedIr`, the tree it makes over the
//! app state it shows, and the retained widget of every node by `SemId`.

use std::collections::BTreeMap;

use comps::{Rect, Wake};
use engine::compositor::{Compositor, LayerId};
use engine::theme::Theme;
use sem::{Revision, SemId, Snapshot};
use uir::{Axis, VerifiedIr};

use crate::draw::{self, closed};
use crate::expand::{Node, expand};
use crate::layout::{self, axis_of};
use crate::widget::Widget;

/// a surface materialized from a verified IR over an app's state.
///
/// what it shows is read from the app's `Snapshot`: `sync` reads the
/// bindings again when the app's revision moved, and keeps that snapshot
/// as the one shown.
#[derive(Debug)]
pub struct Surface {
    ir: VerifiedIr,
    tree: Vec<Node>,
    widgets: BTreeMap<SemId, Widget>,
    shown: Snapshot,
    revision: Revision,
    /// the rect of each node in the last frame drawn (what the person
    /// saw, what a click is tested against); empty until one is drawn and
    /// after the tree changed.
    rects: Vec<Rect>,
    /// the nodes drawn in the last frame (a closed dialog's subtree is
    /// not).
    drawn: Vec<bool>,
}

impl Surface {
    /// the surface `ir` describes, over the app state `app`. only a
    /// `VerifiedIr` materializes: the verifier is the one way to one.
    pub fn materialize(ir: VerifiedIr, app: &Snapshot) -> Surface {
        let mut surface = Surface {
            ir,
            tree: Vec::new(),
            widgets: BTreeMap::new(),
            shown: Snapshot::default(),
            revision: Snapshot::default().revision(),
            rects: Vec::new(),
            drawn: Vec::new(),
        };
        surface.read(app);
        surface
    }

    /// the surface's id, the `Region` it projects as.
    pub fn id(&self) -> &SemId {
        &self.ir.ir().surface.id
    }

    pub fn verified(&self) -> &VerifiedIr {
        &self.ir
    }

    /// the app state the surface shows: what the last `sync` read.
    pub fn shown(&self) -> &Snapshot {
        &self.shown
    }

    /// the ids of the nodes the surface holds now, repeats unrolled, in
    /// tree order.
    pub fn nodes(&self) -> impl Iterator<Item = &SemId> {
        self.tree.iter().map(|n| &n.id)
    }

    /// true when `id` is one of the surface's nodes (or the surface).
    pub fn holds(&self, id: &SemId) -> bool {
        id == self.id() || self.tree.iter().any(|n| n.id == *id)
    }

    /// reads the app's state again when its revision moved; true when it
    /// did. repeats unroll over the new values, a node that is new gets
    /// its widget, a node that stays keeps its own (hover, focus, caret),
    /// a node that left drops it.
    pub fn sync(&mut self, app: &Snapshot) -> bool {
        if app.revision() == self.revision {
            return false;
        }
        self.read(app);
        true
    }

    fn read(&mut self, app: &Snapshot) {
        let tree = expand(&self.ir.ir().surface.root, app);
        let mut widgets = BTreeMap::new();
        for node in &tree {
            let mut widget = self
                .widgets
                .remove(&node.id)
                .unwrap_or_else(|| Widget::new(node));
            widget.sync(node, app);
            widgets.insert(node.id.clone(), widget);
        }
        if tree != self.tree {
            // the geometry of the last frame no longer names these
            // nodes: nothing is hit until the next frame is drawn.
            self.rects.clear();
            self.drawn.clear();
        }
        self.widgets = widgets;
        self.tree = tree;
        self.shown = app.clone();
        self.revision = app.revision();
    }

    /// the rect of every node laid out in `rect`, the tokens read from
    /// `theme`.
    fn lay_out(&self, rect: Rect, theme: &Theme) -> Vec<Rect> {
        let (tree, widgets, shown) = (&self.tree, &self.widgets, &self.shown);
        let leaf = |at: usize| match tree.get(at) {
            Some(node) => {
                let axis = node
                    .parent
                    .and_then(|p| tree.get(p))
                    .map_or(Axis::Vertical, axis_of);
                draw::leaf(node, widgets.get(&node.id), shown, axis, theme)
            }
            None => layout::Leaf::Hidden,
        };
        layout::compute(tree, rect, theme, &leaf)
    }

    /// the height the surface takes at `width`.
    pub fn height(&self, width: f32, theme: &Theme) -> f32 {
        let rects = self.lay_out(Rect::new(0.0, 0.0, width, f32::MAX / 4.0), theme);
        rects.first().map_or(0.0, |r| r.h)
    }

    /// draws the surface at the top of `rect`, as wide as it and as tall
    /// as its content: every node on its widget, every projected node
    /// bound to the rect it is drawn and hit-tested with, an open
    /// dropdown on top.
    pub fn render(&mut self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        for node in &self.tree {
            if let Some(w) = self.widgets.get_mut(&node.id) {
                w.prepare(node, &self.shown, theme);
            }
        }
        self.rects = self.lay_out(rect, theme);
        self.drawn = vec![false; self.tree.len()];
        let mut open = Vec::new();
        let mut hidden = vec![false; self.tree.len()];
        for (at, node) in self.tree.iter().enumerate() {
            let skip = hidden.get(at).copied().unwrap_or(true) || closed(node, &self.shown);
            if skip {
                for &child in &node.children {
                    if let Some(h) = hidden.get_mut(child) {
                        *h = true;
                    }
                }
                continue;
            }
            let (Some(r), Some(w)) = (self.rects.get(at).copied(), self.widgets.get_mut(&node.id))
            else {
                continue;
            };
            draw::draw(c, node, w, r, &self.shown, theme);
            if w.is_open() {
                open.push(at);
            }
            if let Some(d) = self.drawn.get_mut(at) {
                *d = true;
            }
        }
        for at in open {
            let node = self.tree.get(at);
            let r = self.rects.get(at).copied().unwrap_or_default();
            if let Some(Widget::Select(s)) = node.and_then(|n| self.widgets.get(&n.id)) {
                s.render_dropdown(c, LayerId::DEFAULT, r, theme);
            }
        }
    }

    /// the motion the widgets run on their own (a caret, a knob).
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.widgets
            .values_mut()
            .fold(Wake::idle(), |wake, w| wake.merge(w.wake(dt)))
    }
}
