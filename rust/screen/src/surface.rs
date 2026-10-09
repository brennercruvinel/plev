//! one materialized surface: a `VerifiedIr`, the tree it makes over the
//! app state it shows, and the retained widget of every node by `SemId`.

use std::collections::BTreeMap;
use std::sync::Arc;

use engine::widgets::form::EditKey;
use engine::widgets::{EventResult, Rect, Wake, WidgetEvent};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use accord::{Actor, Receipt, Request, Revision, SemId, Snapshot, SnapshotBuilder, Value};
use plevir::{Axis, CapabilityRef, VerifiedIr};

use crate::draw::{self, closed};
use crate::expand::{Node, expand};
use crate::host::{Dispatch, Frames, NoFrames};
use crate::layout::{self, axis_of};
use crate::transition::{self, Ghost, Motion};
use crate::project;
use crate::widget::{Kind, Widget};

/// a surface materialized from a verified IR over an app's state.
///
/// what it shows is read from the app's `Snapshot`: `sync` reads the
/// bindings again when the app's revision moved, and keeps that snapshot
/// as the one shown. an event on one of its widgets becomes a
/// `sem::Request` for the capability the IR bound, observed on that
/// snapshot, by the person (`Actor::Human`), handed to the app's
/// dispatcher; the surface itself never runs anything.
///
/// a new form (`transform`) or a tree the app's state moved (`sync`)
/// changes what the surface is at once, and what the frames show over a
/// few: a node born fades in, a node that moved slides, a node that left
/// fades out where it was, no longer a capability (the `motion` module).
#[derive(Debug)]
pub struct Surface {
    ir: VerifiedIr,
    tree: Vec<Node>,
    widgets: BTreeMap<SemId, Widget>,
    shown: Snapshot,
    revision: Revision,
    /// the rect each node was drawn at in the last frame, by its id: what
    /// the person saw, what a click is tested against. a node not drawn
    /// (not yet, or inside a closed dialog) has none; a node that left the
    /// tree loses its own at once.
    drawn: BTreeMap<SemId, Rect>,
    /// a frame has been drawn: before one, a change has nothing to move
    /// from and shows at once.
    framed: bool,
    motion: Motion,
    inert: bool,
    last: Option<Receipt>,
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
            drawn: BTreeMap::new(),
            framed: false,
            motion: Motion::default(),
            inert: false,
            last: None,
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

    /// the receipt of the last request the surface's own widgets made.
    pub fn last_receipt(&self) -> Option<&Receipt> {
        self.last.as_ref()
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

    /// while an overlay holds the page: the surface projects inert and
    /// takes no input.
    pub fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    /// the person's preference for reduced motion: every change of form
    /// shows at once, the transitions in flight included. the engine does
    /// not know the system's preference yet (task-75), so whoever does
    /// passes it here.
    pub fn set_reduced_motion(&mut self, reduced: bool) {
        self.motion.set_reduced(reduced);
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

    /// the surface takes the form `ir` describes, over the app state
    /// `app`, and reconciles by `SemId`: a node whose id stays and whose
    /// component stays drawn by the same comps widget keeps its retained
    /// widget, and with it what lives there (focus, caret, selection, a
    /// drag in flight); a node whose id stays under another widget gets a
    /// new one, and nothing of the old widget passes to it (what it shows
    /// is the app's, read again through its bindings); a new id gets a new
    /// widget; an id that is gone drops its own. the new form is
    /// projected, routed and hit-tested at once. only a `VerifiedIr`
    /// transforms a surface, as only one materializes it.
    pub fn transform(&mut self, ir: VerifiedIr, app: &Snapshot) {
        self.ir = ir;
        self.read(app);
    }

    fn read(&mut self, app: &Snapshot) {
        let tree = expand(&self.ir.ir().surface.root, app);
        let changed = tree != self.tree;
        let mut widgets = BTreeMap::new();
        let mut born = Vec::new();
        let mut gone = Vec::new();
        for node in &tree {
            let kind = Kind::of(node);
            let mut widget = match self.widgets.remove(&node.id) {
                Some(kept) if kept.kind() == kind => kept,
                old => {
                    // the id stays under another widget: the old one's
                    // picture leaves as the new one enters
                    gone.extend(old.map(|w| (node.id.clone(), w)));
                    match self.motion.reclaim(&node.id, kind) {
                        // back before its picture was gone: the same
                        // instance, from where it showed
                        Some((w, opacity, rect)) => {
                            self.drawn.insert(node.id.clone(), rect);
                            born.push((node.id.clone(), opacity));
                            w
                        }
                        None => {
                            born.push((node.id.clone(), 0.0));
                            Widget::new(node)
                        }
                    }
                }
            };
            widget.sync(node, app);
            widgets.insert(node.id.clone(), widget);
        }
        gone.extend(std::mem::take(&mut self.widgets));
        self.leave(gone);
        // a node that left is no longer where it was drawn; one that stays
        // is, until the next frame draws it again.
        self.drawn.retain(|id, _| widgets.contains_key(id));
        if changed && self.framed {
            self.motion.changed(born);
        }
        self.widgets = widgets;
        self.tree = tree;
        self.shown = app.clone();
        self.revision = app.revision();
    }

    /// the widgets of the nodes that left (or changed widget), each with
    /// its node in the tree they left: the ones drawn in the last frame
    /// fade out where they were, over the state they showed.
    fn leave(&mut self, gone: Vec<(SemId, Widget)>) {
        let mut shown: Option<Arc<Snapshot>> = None;
        for (id, widget) in gone {
            let node = self.tree.iter().find(|n| n.id == id);
            let (Some(node), Some(&rect)) = (node, self.drawn.get(&id)) else {
                continue;
            };
            let shown = shown.get_or_insert_with(|| Arc::new(self.shown.clone()));
            self.motion.leave(node.clone(), widget, rect, shown);
        }
    }

    /// the surface dismissed: the picture of every node it drew leaves
    /// where it was (none with reduced motion, or before a first frame).
    /// the surface is no capability any more.
    pub(crate) fn dismiss(mut self) -> Vec<Ghost> {
        let gone = std::mem::take(&mut self.widgets).into_iter().collect();
        self.leave(gone);
        self.motion.take_leaving()
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

    /// the frame refs the surface's images show now: the text each one's
    /// `source` is bound to, in tree order, once each. what the app
    /// decodes for the surface (`Frames`).
    pub fn sources(&self) -> Vec<String> {
        let mut out = Vec::new();
        for node in &self.tree {
            if let Some(source) = draw::source(node, &self.shown)
                && !out.contains(&source)
            {
                out.push(source);
            }
        }
        out
    }

    /// draws the surface at the top of `rect`, as wide as it and as tall
    /// as its content: every node on its widget, every projected node
    /// bound to the rect it is drawn and hit-tested with, the pictures of
    /// the nodes that left over them (bound to nothing), an open dropdown
    /// on top. the first frame after a change of form starts its
    /// transitions: each node born fades in from here, each node drawn
    /// elsewhere before slides from there. every image is its shell and
    /// label (`render_with` for the pictures).
    pub fn render(&mut self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        self.render_with(c, rect, theme, &NoFrames);
    }

    /// `render`, with the picture of each image asked of `frames`, the
    /// app's, by the frame ref its `source` is bound to.
    pub fn render_with(
        &mut self,
        c: &mut Compositor,
        rect: Rect,
        theme: &Theme,
        frames: &dyn Frames,
    ) {
        for node in &self.tree {
            if let Some(w) = self.widgets.get_mut(&node.id) {
                w.prepare(node, &self.shown, theme);
            }
        }
        let rects = self.lay_out(rect, theme);
        let change = self.motion.take_change();
        let was = std::mem::take(&mut self.drawn);
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
            let (Some(to), Some(w)) = (rects.get(at).copied(), self.widgets.get_mut(&node.id))
            else {
                continue;
            };
            if let Some(born) = &change {
                if let Some(&from) = born.get(&node.id) {
                    self.motion.enter(&node.id, from, theme);
                }
                if let Some(&from) = was.get(&node.id) {
                    self.motion.slide(&node.id, from, to, theme);
                }
            }
            let r = self.motion.rect(&node.id, to);
            let alpha = self.motion.opacity(&node.id);
            let picture = draw::picture(node, &self.shown, frames);
            if alpha < 1.0 {
                let shown = &self.shown;
                transition::draw_faded(c, alpha, true, |own| {
                    draw::draw(own, node, w, r, shown, theme, picture);
                });
            } else {
                draw::draw(c, node, w, r, &self.shown, theme, picture);
            }
            if w.is_open() {
                open.push((at, r));
            }
            self.drawn.insert(node.id.clone(), r);
        }
        for ghost in self.motion.leaving_mut() {
            ghost.draw(c, theme, frames);
        }
        for (at, r) in open {
            let node = self.tree.get(at);
            if let Some(Widget::Select(s)) = node.and_then(|n| self.widgets.get(&n.id)) {
                s.render_dropdown(c, LayerId::DEFAULT, r, theme);
            }
        }
        self.framed = true;
    }

    /// a pointer event over the last frame drawn. each widget sees it
    /// with its own rect (an open dropdown sees it alone); an activation
    /// completed on one becomes a request through `app`.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
        app: &mut dyn Dispatch,
    ) -> EventResult {
        if self.inert {
            return EventResult::IGNORED;
        }
        let open = self
            .tree
            .iter()
            .position(|n| self.widgets.get(&n.id).is_some_and(Widget::is_open));
        let mut result = EventResult::IGNORED;
        let mut asks = Vec::new();
        for (at, node) in self.tree.iter().enumerate() {
            if open.is_some_and(|o| o != at) {
                continue;
            }
            let (Some(rect), Some(w)) = (
                self.drawn.get(&node.id).copied(),
                self.widgets.get_mut(&node.id),
            ) else {
                continue;
            };
            let (r, fire) = w.handle(node, event, rect, theme);
            result = result.merge(r);
            if let Some(fire) = fire
                && let Some(cap) = node.bind.actions.get(fire.event)
            {
                asks.push((cap.clone(), fire.param));
            }
        }
        for (cap, param) in asks {
            self.ask(&cap, param, app);
            result = result.merge(EventResult::changed());
        }
        result
    }

    /// the person asks `cap` with `param`: observed on the snapshot the
    /// surface shows, through the app's dispatcher; then the surface
    /// shows the state that came out.
    fn ask(
        &mut self,
        cap: &CapabilityRef,
        param: Option<Value>,
        app: &mut dyn Dispatch,
    ) -> Option<Receipt> {
        let req = Request::observe(&self.shown, Actor::Human, &cap.target, &cap.action, param);
        let req = match req {
            Ok(req) => req,
            Err(e) => {
                log::warn!("surface {}: {e}", self.id());
                return None;
            }
        };
        let receipt = app.dispatch(req);
        if let Some(now) = app.project() {
            self.sync(&now);
        }
        self.last.clone_from(&receipt);
        receipt
    }

    /// the text field that has the keyboard, by its place in the tree.
    fn focused_at(&self) -> Option<usize> {
        self.tree.iter().position(
            |n| matches!(self.widgets.get(&n.id), Some(Widget::TextField(f)) if f.is_focused()),
        )
    }

    /// the node that has the keyboard.
    pub fn focused(&self) -> Option<&SemId> {
        self.focused_at()
            .and_then(|at| self.tree.get(at))
            .map(|n| &n.id)
    }

    /// edits the focused field with `edit`; a text that moved is the
    /// field's `set_value`, with the new text. false when no field of the
    /// surface has the keyboard.
    fn edit(&mut self, app: &mut dyn Dispatch, edit: impl FnOnce(&mut Widget) -> bool) -> bool {
        if self.inert {
            return false;
        }
        let Some(node) = self.focused_at().and_then(|at| self.tree.get(at)) else {
            return false;
        };
        let (id, cap) = (node.id.clone(), node.bind.actions.get("set_value").cloned());
        let Some(w) = self.widgets.get_mut(&id) else {
            return false;
        };
        let before = w.field().map(|f| f.text().to_owned());
        if !edit(w) {
            return false;
        }
        let after = w.field().map(|f| f.text().to_owned());
        if let (Some(cap), Some(text)) = (cap, after)
            && before.as_ref() != Some(&text)
        {
            self.ask(&cap, Some(Value::Text(text)), app);
        }
        true
    }

    /// a printable character typed while a field of the surface has the
    /// keyboard.
    pub fn handle_text(&mut self, s: &str, app: &mut dyn Dispatch) -> bool {
        let mut chars = s.chars();
        let (Some(c), None) = (chars.next(), chars.next()) else {
            return false;
        };
        if c.is_control() {
            return false;
        }
        self.edit(app, |w| w.field().is_some_and(|f| f.insert(&c.to_string())))
    }

    /// an editing key (backspace, arrows, home, end, select all).
    pub fn handle_edit_key(&mut self, key: EditKey, app: &mut dyn Dispatch) -> bool {
        self.edit(app, |w| w.field().is_some_and(|f| f.edit(key)))
    }

    /// Enter in a focused field is its `submit`, when the IR bound one;
    /// consumed while a field of the surface has the keyboard.
    pub fn handle_enter(&mut self, app: &mut dyn Dispatch) -> bool {
        if self.inert {
            return false;
        }
        let Some(node) = self.focused_at().and_then(|at| self.tree.get(at)) else {
            return false;
        };
        if let Some(cap) = node.bind.actions.get("submit").cloned() {
            self.ask(&cap, None, app);
        }
        true
    }

    /// Escape blurs the focused field.
    pub fn handle_escape(&mut self) -> bool {
        let Some(id) = self.focused().cloned() else {
            return false;
        };
        if let Some(f) = self.widgets.get_mut(&id).and_then(Widget::field) {
            f.unfocus();
        }
        true
    }

    /// keyboard focus from outside (a screen reader, a Tab in semantic
    /// order): a field of the surface takes it, another node of the
    /// surface takes it from the fields. false for an id the surface does
    /// not hold, and while inert.
    pub fn focus(&mut self, id: &SemId) -> bool {
        if self.inert || !self.holds(id) {
            return false;
        }
        for (node_id, w) in &mut self.widgets {
            if let Some(f) = w.field() {
                f.set_focused(node_id == id);
            }
        }
        true
    }

    /// the motion the widgets run on their own (a caret, a knob), and the
    /// surface's transitions: a frame when a change of form waits to be
    /// drawn, every frame while a node fades or slides, and nothing once
    /// they settle.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let widgets = self
            .widgets
            .values_mut()
            .fold(Wake::idle(), |wake, w| wake.merge(w.wake(dt)));
        widgets.merge(self.motion.wake(dt))
    }

    /// projects the surface into `b`, which holds the app's own
    /// projection too (the actions read the app's state).
    pub fn project(&self, b: &mut SnapshotBuilder) {
        let surface = &self.ir.ir().surface;
        project::project(
            b,
            (&surface.id, surface.title.as_str()),
            &self.tree,
            &self.shown,
            self.inert,
            self.focused(),
        );
    }

    /// an actor's request aimed at the surface: an action of one of its
    /// nodes is the capability the IR bound it to, asked by the same
    /// actor with the same parameter. its observation is the same values,
    /// each named again by the app's state the surface showed it for
    /// (the action reads `todo_panel.input:text` where the capability
    /// reads `todo.input:text`). any other request, and a node of the
    /// app's own, goes as it is.
    pub fn route(&self, req: Request) -> Request {
        let Some(node) = self.tree.iter().find(|n| n.id == req.target) else {
            return req;
        };
        if self.shown.get(&node.id).is_some() {
            return req;
        }
        let Some(cap) = node.bind.actions.get(req.action.as_str()) else {
            return req;
        };
        let mut observed = req.observed;
        for (at, _) in &mut observed.values {
            let app = self
                .tree
                .iter()
                .find(|n| n.id == at.node && self.shown.get(&n.id).is_none())
                .and_then(|n| n.bind.state.get(at.key.as_str()));
            if let Some(app) = app {
                *at = app.clone();
            }
        }
        Request {
            target: cap.target.clone(),
            action: cap.action.clone(),
            observed,
            ..req
        }
    }
}
