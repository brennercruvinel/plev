//! The binding table: where each logical identity (`accord::SemId`) sits in
//! the current frame. It lives beside the scene, never in it: a binding
//! is not a scene node, takes no part in a layer hash and never changes a
//! pixel, so binding an id, or not, can never dirty a layer.
//!
//! A widget with a `SemId` binds the rect it draws and hit-tests with, on
//! the layer it draws into: `bind` targets the default layer like `push`,
//! `bind_to_layer` a given one like `push_to_layer`. The clip is read from
//! that layer's own `PushClip`/`PopClip` nodes as they stand at the bind,
//! so the drawing path is untouched. `begin_frame` clears the table:
//! between two frames it answers for the frame on screen, which is what a
//! person clicking saw.
//!
//! See docs/adr/semantics/0035-semantic-binding-beside-the-scene.md.

use accord::SemId;
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashSet};

use super::clip::{ClipRect, ClipStack, intersect_rects};
use super::{Compositor, LayerId, SceneNode};
use crate::control::input::ViewId;
use crate::runtime::perf::stages::{self, Stage};

/// Where one logical identity sits in the current frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Binding {
    /// `(x, y, w, h)` in logical window pixels: the rect the widget draws
    /// and hit-tests with.
    pub rect: [f32; 4],
    /// The layer the widget draws into.
    pub layer: LayerId,
    /// The clip open in `layer` at the bind (the intersection of every
    /// open `PushClip`); `None` when unclipped.
    pub clip: Option<ClipRect>,
    /// Position among this frame's bindings. Within one layer a higher
    /// order was bound later, so it sits on top.
    pub order: u32,
    /// The temporary render identity, when the widget registered a hit
    /// region under one this frame (`Compositor::set_binding_view`).
    pub view: Option<ViewId>,
}

impl Binding {
    /// The part of the rect inside its clip, `(x, y, w, h)` in logical
    /// window pixels: where the bound id shows in the frame. `None` when
    /// nothing of it shows (clipped away, or an empty rect). The layer's
    /// visibility and opacity are not read, only the geometry.
    pub fn visible_rect(&self) -> Option<[f32; 4]> {
        let [x, y, w, h] = self
            .clip
            .map_or(self.rect, |clip| intersect_rects(self.rect, clip));
        (w > 0.0 && h > 0.0).then_some([x, y, w, h])
    }
}

/// The per-frame table and what it needs to fill it.
#[derive(Default)]
pub(crate) struct BindTable {
    table: IndexMap<SemId, Binding, FxBuildHasher>,
    /// One clip scan per layer bound into this frame.
    scans: Vec<ClipScan>,
    /// Ids already reported as bound twice (each is logged once).
    warned: FxHashSet<SemId>,
}

impl BindTable {
    pub(crate) fn begin_frame(&mut self) {
        self.table.clear();
        self.scans.clear();
    }

    /// The clip open in `layer` after the nodes pushed to it so far.
    fn clip(&mut self, layer: LayerId, nodes: &[SceneNode]) -> Option<ClipRect> {
        let at = match self.scans.iter().position(|s| s.layer == layer) {
            Some(at) => at,
            None => {
                self.scans.push(ClipScan::new(layer));
                self.scans.len() - 1
            }
        };
        self.scans.get_mut(at).and_then(|scan| scan.advance(nodes))
    }
}

/// How far the clip scan of one layer got this frame: the nodes read and
/// the clips still open after them. Each node is read once per frame, so
/// binding costs the nodes pushed since the previous bind, not the layer.
struct ClipScan {
    layer: LayerId,
    read: usize,
    clips: ClipStack,
}

impl ClipScan {
    fn new(layer: LayerId) -> Self {
        Self {
            layer,
            read: 0,
            clips: ClipStack::default(),
        }
    }

    fn advance(&mut self, nodes: &[SceneNode]) -> Option<ClipRect> {
        if self.read > nodes.len() {
            // The layer holds fewer nodes than were read: it was cleared,
            // so read it again from the start.
            *self = Self::new(self.layer);
        }
        for node in nodes.iter().skip(self.read) {
            match *node {
                SceneNode::PushClip { x, y, w, h } => self.clips.push([x, y, w, h]),
                SceneNode::PopClip => self.clips.pop(),
                _ => {}
            }
        }
        self.read = nodes.len();
        self.clips.current()
    }
}

/// Inclusive on every edge, like the widgets' own `Rect::contains` and the
/// input hit test.
fn contains(r: [f32; 4], x: f32, y: f32) -> bool {
    x >= r[0] && x <= r[0] + r[2] && y >= r[1] && y <= r[1] + r[3]
}

impl Compositor {
    /// Binds `id` to `rect` on the layer `push` draws on (the default
    /// one, but inside [`Compositor::on_layer`]). `rect` is `(x, y, w, h)` in logical window pixels, the same
    /// rect the widget draws and hit-tests with.
    pub fn bind(&mut self, id: &SemId, rect: [f32; 4]) {
        self.bind_to_layer(self.target, id, rect);
    }

    /// Binds `id` to `rect` on `layer`, with the clip open in that layer
    /// at this point of the frame. An id binds once per frame: a second
    /// bind is logged (once per id) and ignored, and the first binding
    /// holds. A layer that does not exist binds nothing, like a push to
    /// it draws nothing.
    pub fn bind_to_layer(&mut self, layer: LayerId, id: &SemId, rect: [f32; 4]) {
        let Some(target) = self.layers.iter().find(|l| l.id == layer) else {
            log::warn!("bind: layer {layer:?} not found, {id} is not bound");
            return;
        };
        let binds = &mut self.bind_table;
        if binds.table.contains_key(id) {
            if binds.warned.insert(id.clone()) {
                log::warn!("bind: {id} is bound twice in one frame; the first binding holds");
            }
            return;
        }
        let clip = binds.clip(layer, &target.nodes);
        let order = binds.table.len() as u32;
        binds.table.insert(
            id.clone(),
            Binding {
                rect,
                layer,
                clip,
                order,
                view: None,
            },
        );
        stages::count(Stage::Bind, 0);
    }

    /// Records the `ViewId` under which the widget bound as `id`
    /// registered its hit region this frame. `false` when `id` is not
    /// bound.
    pub fn set_binding_view(&mut self, id: &SemId, view: ViewId) -> bool {
        match self.bind_table.table.get_mut(id) {
            Some(binding) => {
                binding.view = Some(view);
                true
            }
            None => false,
        }
    }

    /// This frame's bindings, in bind order.
    pub fn bindings(&self) -> impl Iterator<Item = (&SemId, &Binding)> + '_ {
        self.bind_table.table.iter()
    }

    /// This frame's bindings that show, each with its visible rect
    /// ([`Binding::visible_rect`]), in bind order: where an agent or a
    /// screen reader finds each id on screen.
    pub fn visible_bindings(&self) -> impl Iterator<Item = (&SemId, [f32; 4])> + '_ {
        self.bindings()
            .filter_map(|(id, b)| b.visible_rect().map(|rect| (id, rect)))
    }

    /// Where `id` sits in this frame, if it was bound.
    pub fn binding(&self, id: &SemId) -> Option<&Binding> {
        self.bind_table.table.get(id)
    }

    /// The topmost bound id at `(x, y)` in logical window pixels: the one
    /// drawn last among those whose rect and clip hold the point, on a
    /// visible layer (`visible` and an opacity above zero). Layers stack by
    /// z-order, then by creation, as the frame composites them; within a
    /// layer, the later bind is on top. Only bound ids take part, so an
    /// unbound draw on top does not hide what is bound beneath it.
    pub fn hit(&self, x: f32, y: f32) -> Option<&SemId> {
        let _probe = stages::span(Stage::HitTest).items(self.bind_table.table.len() as u64);
        let mut top: Option<((i32, usize, u32), &SemId)> = None;
        for (id, b) in &self.bind_table.table {
            if !contains(b.rect, x, y) || b.clip.is_some_and(|c| !contains(c, x, y)) {
                continue;
            }
            let Some((rank, layer)) = self
                .layers
                .iter()
                .enumerate()
                .find(|(_, l)| l.id == b.layer)
            else {
                continue;
            };
            if !layer.visible || layer.opacity <= 0.0 {
                continue;
            }
            let key = (layer.z_order, rank, b.order);
            if top.is_none_or(|(best, _)| key > best) {
                top = Some((key, id));
            }
        }
        top.map(|(_, id)| id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> SceneNode {
        SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    #[test]
    fn bind_records_rect_layer_and_order_and_the_frame_clears_it() {
        let mut c = Compositor::new();
        c.begin_frame();
        c.bind(&id("a"), [0.0, 0.0, 10.0, 10.0]);
        c.bind(&id("b"), [20.0, 0.0, 10.0, 10.0]);
        let b = c.binding(&id("b")).unwrap();
        assert_eq!(b.rect, [20.0, 0.0, 10.0, 10.0]);
        assert_eq!(b.layer, LayerId::DEFAULT);
        assert_eq!(b.clip, None);
        assert_eq!(b.order, 1);
        assert_eq!(b.view, None);
        let ids: Vec<&str> = c.bindings().map(|(i, _)| i.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);

        c.begin_frame();
        assert_eq!(c.bindings().count(), 0);
        assert_eq!(c.binding(&id("a")), None);
        assert_eq!(c.hit(5.0, 5.0), None);
    }

    #[test]
    fn bind_takes_the_clip_open_in_its_layer_at_that_point() {
        let mut c = Compositor::new();
        let over = c.create_layer(5);
        c.begin_frame();
        c.push_clip(0.0, 0.0, 100.0, 100.0);
        c.bind(&id("outer"), [10.0, 10.0, 10.0, 10.0]);
        c.push_clip(50.0, 50.0, 100.0, 100.0);
        c.push(rect(60.0, 60.0, 5.0, 5.0));
        c.bind(&id("inner"), [60.0, 60.0, 5.0, 5.0]);
        // Another layer's clip never leaks into this one.
        c.bind_to_layer(over, &id("other"), [0.0, 0.0, 5.0, 5.0]);
        c.pop_clip();
        c.bind(&id("after_pop"), [1.0, 1.0, 1.0, 1.0]);
        c.pop_clip();
        c.bind(&id("unclipped"), [1.0, 1.0, 1.0, 1.0]);

        let clip = |s| c.binding(&id(s)).unwrap().clip;
        assert_eq!(clip("outer"), Some([0.0, 0.0, 100.0, 100.0]));
        assert_eq!(clip("inner"), Some([50.0, 50.0, 50.0, 50.0]));
        assert_eq!(clip("other"), None);
        assert_eq!(c.binding(&id("other")).unwrap().layer, over);
        assert_eq!(clip("after_pop"), Some([0.0, 0.0, 100.0, 100.0]));
        assert_eq!(clip("unclipped"), None);
    }

    #[test]
    fn hit_returns_the_topmost_bound_id_in_draw_order() {
        let mut c = Compositor::new();
        let high = c.create_layer(10);
        let twin = c.create_layer(10);
        c.begin_frame();
        // Bound first on a higher layer: still on top of the default layer.
        c.bind_to_layer(high, &id("popup"), [40.0, 40.0, 20.0, 20.0]);
        c.bind(&id("panel"), [0.0, 0.0, 100.0, 100.0]);
        c.bind(&id("row"), [0.0, 0.0, 100.0, 20.0]);

        assert_eq!(c.hit(50.0, 10.0).map(SemId::as_str), Some("row"));
        assert_eq!(c.hit(50.0, 30.0).map(SemId::as_str), Some("panel"));
        assert_eq!(c.hit(50.0, 50.0).map(SemId::as_str), Some("popup"));
        assert_eq!(c.hit(150.0, 50.0), None);
        // Edges are inclusive, like Rect::contains.
        assert_eq!(c.hit(100.0, 100.0).map(SemId::as_str), Some("panel"));

        // Equal z-order: the layer created later composites on top.
        c.bind_to_layer(twin, &id("twin"), [40.0, 40.0, 20.0, 20.0]);
        assert_eq!(c.hit(50.0, 50.0).map(SemId::as_str), Some("twin"));
        // Raising a layer after the bind still counts at hit time.
        c.set_layer_z_order(high, 20);
        assert_eq!(c.hit(50.0, 50.0).map(SemId::as_str), Some("popup"));
    }

    #[test]
    fn hit_respects_the_clip() {
        let mut c = Compositor::new();
        c.begin_frame();
        c.bind(&id("panel"), [0.0, 0.0, 200.0, 200.0]);
        c.push_clip(0.0, 0.0, 100.0, 50.0);
        // A row scrolled half out of its list viewport.
        c.bind(&id("row"), [0.0, 30.0, 100.0, 40.0]);
        c.pop_clip();

        assert_eq!(c.hit(50.0, 40.0).map(SemId::as_str), Some("row"));
        assert_eq!(
            c.hit(50.0, 60.0).map(SemId::as_str),
            Some("panel"),
            "the clipped-away half of the row is not hit"
        );

        // An empty clip intersection hides whatever is bound inside it.
        c.push_clip(0.0, 0.0, 10.0, 10.0);
        c.push_clip(20.0, 20.0, 10.0, 10.0);
        c.bind(&id("hidden"), [0.0, 0.0, 200.0, 200.0]);
        c.pop_clip();
        c.pop_clip();
        assert_eq!(c.hit(25.0, 25.0).map(SemId::as_str), Some("panel"));
    }

    #[test]
    fn hit_skips_invisible_layers() {
        let mut c = Compositor::new();
        let hidden = c.create_layer(10);
        let faded = c.create_layer(20);
        c.begin_frame();
        c.bind(&id("base"), [0.0, 0.0, 100.0, 100.0]);
        c.bind_to_layer(hidden, &id("hidden"), [0.0, 0.0, 100.0, 100.0]);
        c.bind_to_layer(faded, &id("faded"), [0.0, 0.0, 100.0, 100.0]);
        c.set_layer_visible(hidden, false);
        c.set_layer_opacity(faded, 0.0);
        assert_eq!(c.hit(50.0, 50.0).map(SemId::as_str), Some("base"));

        c.set_layer_opacity(faded, 0.5);
        assert_eq!(c.hit(50.0, 50.0).map(SemId::as_str), Some("faded"));
    }

    #[test]
    fn a_second_bind_in_one_frame_is_ignored_and_the_first_holds() {
        let mut c = Compositor::new();
        c.begin_frame();
        c.bind(&id("x"), [0.0, 0.0, 10.0, 10.0]);
        c.bind(&id("x"), [50.0, 50.0, 10.0, 10.0]);
        c.bind(&id("x"), [90.0, 90.0, 10.0, 10.0]);
        assert_eq!(c.bindings().count(), 1);
        assert_eq!(c.binding(&id("x")).unwrap().rect, [0.0, 0.0, 10.0, 10.0]);
        assert_eq!(c.hit(55.0, 55.0), None);

        // The next frame binds it afresh.
        c.begin_frame();
        c.bind(&id("x"), [50.0, 50.0, 10.0, 10.0]);
        assert_eq!(c.binding(&id("x")).unwrap().rect, [50.0, 50.0, 10.0, 10.0]);
    }

    #[test]
    fn binding_to_a_missing_layer_binds_nothing() {
        let mut c = Compositor::new();
        c.begin_frame();
        c.bind_to_layer(LayerId(99), &id("ghost"), [0.0, 0.0, 10.0, 10.0]);
        assert_eq!(c.binding(&id("ghost")), None);
        assert_eq!(c.hit(5.0, 5.0), None);
    }

    #[test]
    fn set_binding_view_records_the_render_identity() {
        let mut c = Compositor::new();
        c.begin_frame();
        c.bind(&id("x"), [0.0, 0.0, 10.0, 10.0]);
        assert!(c.set_binding_view(&id("x"), ViewId(7)));
        assert_eq!(c.binding(&id("x")).unwrap().view, Some(ViewId(7)));
        assert!(!c.set_binding_view(&id("y"), ViewId(8)));
    }

    #[test]
    fn the_visible_rect_is_the_part_inside_the_clip() {
        let mut c = Compositor::new();
        c.begin_frame();
        c.bind(&id("free"), [5.0, 5.0, 10.0, 10.0]);
        c.push_clip(0.0, 0.0, 100.0, 50.0);
        // A row scrolled half out of its list viewport.
        c.bind(&id("half"), [0.0, 30.0, 100.0, 40.0]);
        c.bind(&id("inside"), [10.0, 10.0, 20.0, 20.0]);
        // Below the viewport: nothing of it shows.
        c.bind(&id("below"), [0.0, 60.0, 100.0, 40.0]);
        // Touching the clip's edge only: a line, no area.
        c.bind(&id("edge"), [0.0, 50.0, 100.0, 40.0]);
        c.pop_clip();
        c.bind(&id("empty"), [0.0, 0.0, 0.0, 10.0]);

        let seen = |s| c.binding(&id(s)).unwrap().visible_rect();
        assert_eq!(seen("free"), Some([5.0, 5.0, 10.0, 10.0]), "unclipped");
        assert_eq!(seen("half"), Some([0.0, 30.0, 100.0, 20.0]));
        assert_eq!(seen("inside"), Some([10.0, 10.0, 20.0, 20.0]));
        assert_eq!(seen("below"), None);
        assert_eq!(seen("edge"), None);
        assert_eq!(seen("empty"), None);

        // The frame's visible bindings: those that show, in bind order.
        let shown: Vec<(&str, [f32; 4])> = c
            .visible_bindings()
            .map(|(id, rect)| (id.as_str(), rect))
            .collect();
        assert_eq!(
            shown,
            [
                ("free", [5.0, 5.0, 10.0, 10.0]),
                ("half", [0.0, 30.0, 100.0, 20.0]),
                ("inside", [10.0, 10.0, 20.0, 20.0]),
            ]
        );
        c.begin_frame();
        assert_eq!(c.visible_bindings().count(), 0);
    }

    #[test]
    fn binding_ids_over_identical_pixels_dirties_no_layer() {
        let scene = |c: &mut Compositor, bind: bool| {
            c.begin_frame();
            c.push_clip(0.0, 0.0, 200.0, 200.0);
            c.push(rect(0.0, 0.0, 100.0, 40.0));
            if bind {
                c.bind(&id("todo.input"), [0.0, 0.0, 100.0, 40.0]);
            }
            c.push(rect(0.0, 50.0, 100.0, 40.0));
            if bind {
                c.bind(&id("todo.item[1]"), [0.0, 50.0, 100.0, 40.0]);
            }
            c.pop_clip();
        };
        let mut c = Compositor::new();
        scene(&mut c, false);
        let unbound = c.layer(LayerId::DEFAULT).unwrap().compute_hash();
        c.resolve_scene((800.0, 600.0));
        c.mark_layer_clean(LayerId::DEFAULT);

        scene(&mut c, true);
        assert_eq!(c.bindings().count(), 2);
        assert_eq!(
            c.layer(LayerId::DEFAULT).unwrap().compute_hash(),
            unbound,
            "a binding is not a scene node"
        );
        assert!(!c.needs_render(), "binding never dirties a layer");

        // And dropping the bindings again changes nothing either.
        scene(&mut c, false);
        assert!(!c.needs_render());
    }
}
