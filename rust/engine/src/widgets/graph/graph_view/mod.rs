//! Graph/network canvas: a force-directed node-link view with pan (drag),
//! wheel zoom anchored at the cursor, hover hit-testing and click
//! selection with incident-edge highlight.
//!
//! The app owns the data and the side panels: feed a
//! [`Graph`](crate::widgets::graph::model::Graph) (or a precomputed
//! [`GraphScene`](crate::widgets::graph::geometry::GraphScene) when the layout ran off the UI
//! thread), then read [`GraphView::hovered`]/[`selected`](GraphView::selected)
//! to drive your own tooltip/detail — the widget never owns a Tooltip.
//! Node payloads stay app-side; everything here indexes nodes by ordinal
//! (through `scene.node_to` when the layout BFS-subsampled a large graph).
//!
//! Rendering batches edges into one path per kind per pass (cool/hot) —
//! each edge is its own `end_open` sub-path because lyon panics on a
//! `begin` without `end` (the nestui graph discovered this). Nodes are
//! accent circles, radius clamped per zoom level.
//!
//! Its layout runs once ([`geometry::compute_layout`]); a graph whose
//! positions move with a simulation or a drag is
//! [`LiveGraph`](super::live_graph::LiveGraph).

pub mod geometry;

use std::collections::HashMap;

use self::geometry::{GraphData, GraphScene, ViewTransform, compute_layout};
use crate::drawing::compositor::{Compositor, SceneNode};
use crate::drawing::path::PathBuilder;
use crate::styling::theme::Theme;
use crate::widgets::graph::model::Graph;

use crate::widgets::recipe::rounded_rect_stroke;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder, Value};

/// Fixed world box the layout runs in when the widget computes it.
const WORLD: f32 = 1000.0;
/// World-space node radius; screen radius is clamped per zoom level.
const NODE_R: f32 = 5.0;
/// Click-vs-pan discrimination: below this drag distance a press is a click.
const CLICK_DIST: f32 = 4.0;
/// How far past a node's edge a click still finds it.
const HIT_SLACK: f32 = 3.0;

/// Theme tone for an edge kind (mapped to tokens at render time).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeTone {
    /// Quiet neutral (the soft glass edge) — structural edges.
    Dim,
    /// Theme accent — primary relationship.
    Accent,
    /// Informational color.
    Info,
    /// Constructive color.
    Success,
    /// Destructive color.
    Danger,
}

impl EdgeTone {
    fn color(self, theme: &Theme) -> [f32; 4] {
        let g = &theme.navigation.graph;
        match self {
            EdgeTone::Dim => g.edge_dim.0,
            EdgeTone::Accent => g.accent.0,
            EdgeTone::Info => g.edge_info.0,
            EdgeTone::Success => g.edge_success.0,
            EdgeTone::Danger => g.edge_danger.0,
        }
    }
}

/// Interactive network canvas. See the module docs for the ownership
/// contract.
#[derive(Clone, Debug)]
pub struct GraphView {
    scene: Option<GraphScene>,
    transform: ViewTransform,
    /// Fit the world into the viewport on the next render (after
    /// `set_graph`/`set_scene` or [`fit_view`](Self::fit_view)): the box of
    /// its nodes, not the world it was laid out in.
    needs_fit: bool,
    hovered: Option<usize>,
    selected: Option<usize>,
    /// Last cursor position while panning.
    drag: Option<(f32, f32)>,
    /// Distance dragged since MouseDown (click vs. pan discrimination).
    drag_dist: f32,
    /// kind → tone overrides (defaults: 0 Dim, 1 Accent, 2 Info).
    edge_tones: HashMap<u8, EdgeTone>,
    sem: Option<SemId>,
    /// The node with the keyboard focus, driven by the owning view.
    focused: Option<usize>,
    /// The top-left of the bounds it last drew or took an event in: the
    /// transform is the canvas's own, and this places it in the window.
    origin: (f32, f32),
}

impl GraphView {
    pub fn new() -> Self {
        Self {
            scene: None,
            transform: ViewTransform::default(),
            needs_fit: true,
            hovered: None,
            selected: None,
            drag: None,
            drag_dist: 0.0,
            edge_tones: HashMap::new(),
            sem: None,
            focused: None,
            origin: (0.0, 0.0),
        }
    }

    /// Keyboard focus on scene node `node`, or on none, driven by the
    /// owning view: the focus ring around it while it is drawn.
    pub fn set_focused(&mut self, node: Option<usize>) {
        let n = self.scene.as_ref().map_or(0, |s| s.positions.len());
        self.focused = node.filter(|i| *i < n);
    }

    pub fn focused(&self) -> Option<usize> {
        self.focused
    }

    /// The logical identity the render binds the canvas to; each node
    /// it draws binds as `<id>[<scene ordinal>]`.
    pub fn set_sem(&mut self, id: SemId) {
        self.sem = Some(id);
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Select scene node `node`, or none: what a click on it does. `false`
    /// for a node the scene does not hold, or when nothing changed.
    pub fn select(&mut self, node: Option<usize>) -> bool {
        let n = self.scene.as_ref().map_or(0, |s| s.positions.len());
        if node.is_some_and(|i| i >= n) || node == self.selected {
            return false;
        }
        self.selected = node;
        true
    }

    /// Declares the canvas as `id`: a `Role::Canvas` named `label` whose
    /// `selected` state is the selected node's ordinal (or none), with a
    /// `Role::ListItem` `<id>[<i>]` per scene node, labeled by
    /// `node_label`, that offers `select` (it reads the canvas's
    /// `selected`). Pan and zoom move the view, not the graph, and are
    /// not declared. Only the selected node, or the first, is a stop of
    /// Tab. Returns the canvas's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
        mut node_label: impl FnMut(usize) -> String,
    ) -> NodeBuilder<'b> {
        let selected = format!("{id}:selected");
        let n = self.scene.as_ref().map_or(0, |s| s.positions.len());
        let stop = self.selected.unwrap_or(0);
        for i in 0..n {
            let mut node = b.node(id.key(i.to_string()), Role::ListItem, node_label(i));
            node.parent(id).focusable(i == stop);
            node.action("select").reads(selected.as_str());
        }
        let mut node = b.node(id, Role::Canvas, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        let value = match self.selected {
            Some(i) => Value::Text(i.to_string()),
            None => Value::Text(String::new()),
        };
        node.state("selected", value);
        node
    }

    /// Set the graph and lay it out inline (O(n²) force iterations on this
    /// thread — fine for the widget's demo scale; for thousands of nodes
    /// precompute on a worker and hand over the scene via `set_scene`).
    pub fn set_graph(&mut self, graph: &Graph) {
        self.set_scene(compute_layout(&GraphData::from_graph(graph), WORLD, WORLD));
    }

    /// Set a precomputed scene (the off-thread path for large graphs).
    pub fn set_scene(&mut self, scene: GraphScene) {
        self.scene = Some(scene);
        self.needs_fit = true;
        self.hovered = None;
        self.selected = None;
    }

    /// Clear the graph (empty canvas).
    pub fn clear(&mut self) {
        self.scene = None;
        self.hovered = None;
        self.selected = None;
    }

    /// Re-fit the world into the viewport on the next render.
    pub fn fit_view(&mut self) {
        self.needs_fit = true;
    }

    /// Hovered scene node (ordinal into the scene; map through
    /// `scene.node_to` for the app ordinal).
    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    /// Selected scene node. Selection changes report
    /// [`EventResult::clicked`] from `handle_event`.
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// The current scene (for `node_to`, `subsampled`, positions).
    pub fn scene(&self) -> Option<&GraphScene> {
        self.scene.as_ref()
    }

    /// Screen position of a scene node after the current transform
    /// (app-side tooltips/labels anchor here).
    pub fn node_screen_pos(&self, node: usize) -> Option<(f32, f32)> {
        let scene = self.scene.as_ref()?;
        let (wx, wy) = *scene.positions.get(node)?;
        Some(self.screen(wx, wy))
    }

    /// Override the tone of an edge kind.
    pub fn set_edge_tone(&mut self, kind: u8, tone: EdgeTone) {
        self.edge_tones.insert(kind, tone);
    }

    fn edge_tone(&self, kind: u8) -> EdgeTone {
        self.edge_tones.get(&kind).copied().unwrap_or(match kind {
            0 => EdgeTone::Dim,
            1 => EdgeTone::Accent,
            _ => EdgeTone::Info,
        })
    }

    /// Binds the canvas and each node it draws inside `bounds`, as the
    /// square its click finds it in.
    fn bind_nodes(&self, c: &mut Compositor, id: &SemId, scene: &GraphScene, bounds: Rect) {
        c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        let r = self.node_r() + HIT_SLACK;
        for (i, &(wx, wy)) in scene.positions.iter().enumerate() {
            let (sx, sy) = self.screen(wx, wy);
            if bounds.contains(sx, sy) {
                c.bind(&id.key(i.to_string()), [sx - r, sy - r, r * 2.0, r * 2.0]);
            }
        }
    }

    /// A world point in window px: the canvas's transform, then its
    /// place in the window.
    fn screen(&self, wx: f32, wy: f32) -> (f32, f32) {
        let (x, y) = self.transform.world_to_screen(wx, wy);
        (x + self.origin.0, y + self.origin.1)
    }

    /// Node radius in screen px at the current zoom (clamped so nodes
    /// stay visible zoomed out and stop growing zoomed in).
    fn node_r(&self) -> f32 {
        (NODE_R * self.transform.scale).clamp(2.0, 9.0)
    }

    /// Nearest node to a screen point within its click radius.
    fn node_at(&self, scene: &GraphScene, x: f32, y: f32) -> Option<usize> {
        let r = self.node_r() + HIT_SLACK;
        let mut best: Option<(usize, f32)> = None;
        for (i, &(wx, wy)) in scene.positions.iter().enumerate() {
            let (sx, sy) = self.screen(wx, wy);
            let d = ((sx - x).powi(2) + (sy - y).powi(2)).sqrt();
            if d <= r && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        best.map(|(i, _)| i)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        if !bounds.contains(event.pos().0, event.pos().1) && self.drag.is_none() {
            return EventResult::IGNORED;
        }
        self.origin = (bounds.x, bounds.y);
        let Some(scene) = &self.scene else {
            return EventResult::IGNORED;
        };
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                if let Some((lx, ly)) = self.drag {
                    // Pan follows the cursor 1:1 (screen px).
                    self.transform.pan_by(x - lx, y - ly);
                    self.drag = Some((x, y));
                    self.drag_dist += (x - lx).abs() + (y - ly).abs();
                    return EventResult::changed();
                }
                let hit = self.node_at(scene, x, y);
                if hit != self.hovered {
                    self.hovered = hit;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                self.drag = Some((x, y));
                self.drag_dist = 0.0;
                EventResult {
                    handled: true,
                    ..EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                if self.drag.take().is_none() {
                    return EventResult::IGNORED;
                }
                if self.drag_dist < CLICK_DIST {
                    // A click, not a pan: toggle the node selection.
                    let hit = self.node_at(scene, x, y);
                    if hit != self.selected {
                        self.selected = hit;
                        return EventResult::clicked();
                    }
                    return EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    };
                }
                EventResult::changed()
            }
            WidgetEvent::Scroll { x, y, delta } => {
                // Wheel zoom, anchored at the cursor. Trackpad pixel deltas
                // are small, line deltas large; exp() normalizes both.
                let at = (x - self.origin.0, y - self.origin.1);
                self.transform.zoom_at(at, (-delta * 0.002).exp());
                EventResult::changed()
            }
        }
    }

    /// Nothing animates (the layout is computed, not live).
    pub fn tick(&mut self, _dt: f32) -> bool {
        false
    }

    pub fn render(&mut self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let Some(scene) = &self.scene else {
            return;
        };
        self.origin = (bounds.x, bounds.y);
        if self.needs_fit {
            let margin = theme.spacing.xl;
            self.transform =
                ViewTransform::fit_points(&scene.positions, bounds.w, bounds.h, margin);
            self.needs_fit = false;
        }

        if let Some(id) = &self.sem {
            self.bind_nodes(c, id, scene, bounds);
        }
        c.push(SceneNode::PushClip {
            x: bounds.x,
            y: bounds.y,
            w: bounds.w,
            h: bounds.h,
        });
        let look = &theme.navigation.graph;
        let r = self.node_r();
        let edge_w = (self.transform.scale * 1.2).clamp(0.5, 2.5);

        // Edges: one batched path per kind, cool pass (all edges) then a
        // hot pass (the selected node's incident edges at full alpha).
        let selected = self.selected;
        let hot = |i: usize, j: u32| selected.is_some_and(|s| s == i || s as u32 == j);
        for pass_hot in [false, true] {
            let mut builders: HashMap<u8, PathBuilder> = HashMap::new();
            for i in 0..scene.graph.n_nodes {
                let (x1, y1) = self.screen(scene.positions[i].0, scene.positions[i].1);
                for (k, &j) in scene.graph.neighbors(i).iter().enumerate() {
                    let j = j as usize;
                    if j >= scene.graph.n_nodes || j < i {
                        continue; // draw each pair once
                    }
                    if hot(i, j as u32) != pass_hot {
                        continue;
                    }
                    let kind = scene.graph.kind(i, k).unwrap_or(0);
                    let (x2, y2) = self.screen(scene.positions[j].0, scene.positions[j].1);
                    let b = builders.entry(kind).or_default();
                    // Lyon requires end() between sub-paths; each edge is
                    // its own open sub-path in the batched stroke.
                    *b = std::mem::take(b).move_to(x1, y1).line_to(x2, y2).end_open();
                }
            }
            // Push in sorted kind order: HashMap iteration order is
            // nondeterministic and the scene hash would differ per frame
            // (caught by the showcase idle-rerender probe).
            let mut kinds: Vec<u8> = builders.keys().copied().collect();
            kinds.sort_unstable();
            for kind in kinds {
                let builder = builders.remove(&kind).unwrap();
                let base = self.edge_tone(kind).color(theme);
                // Cool pass at low alpha so dense clusters stay readable;
                // the selected node's incident edges go nearly opaque.
                let alpha = if pass_hot {
                    look.edge_hot_alpha
                } else {
                    look.edge_cool_alpha
                };
                c.draw_path(builder.stroke([base[0], base[1], base[2], alpha], edge_w));
            }
        }

        // Nodes: accent dots; hovered/selected get a ring and full alpha.
        let node_color = look.accent.0;
        for (i, &(wx, wy)) in scene.positions.iter().enumerate() {
            let (sx, sy) = self.screen(wx, wy);
            let is_sel = self.selected == Some(i);
            let is_hov = self.hovered == Some(i);
            let alpha = if is_sel || is_hov {
                1.0
            } else {
                look.node_rest_alpha
            };
            let color = [node_color[0], node_color[1], node_color[2], alpha];
            c.draw_path(PathBuilder::circle(sx, sy, r).fill(color));
            if is_sel || is_hov {
                c.draw_path(
                    PathBuilder::circle(sx, sy, r + theme.spacing.xs)
                        .stroke(look.ring.0, theme.control.edge_width),
                );
            }
        }
        if let Some(&(wx, wy)) = self.focused.and_then(|i| scene.positions.get(i)) {
            let (sx, sy) = self.screen(wx, wy);
            let ring = r + theme.spacing.xs + theme.control.focus_ring_offset;
            c.draw_path(
                PathBuilder::circle(sx, sy, ring)
                    .stroke(theme.colors.accent.0, theme.control.focus_ring_width),
            );
        }
        c.push(SceneNode::PopClip);

        // Selection frame cue: a soft stroke around the canvas while a
        // node is selected (the detail panel belongs to the app).
        if self.selected.is_some() {
            c.push(rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                look.frame_radius,
                look.frame.0,
                theme.control.edge_width,
            ));
        }
    }
}

impl Default for GraphView {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Headless tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::graph::model::{Edge, Node};

    fn bounds() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    /// 0→1→2→3 (kind 0), 0→4 (kind 1), 4→5 (kind 2).
    fn demo_graph() -> Graph {
        let edge = |from, to, kind| Edge {
            kind,
            ..Edge::new(from, to)
        };
        Graph {
            nodes: (0..6).map(|i| Node::new(i.to_string())).collect(),
            edges: vec![
                edge(0, 1, 0),
                edge(1, 2, 0),
                edge(2, 3, 0),
                edge(0, 4, 1),
                edge(4, 5, 2),
            ],
            groups: Vec::new(),
        }
    }

    /// Render once to settle the fit transform, then return node 0's
    /// screen position.
    fn node0_screen(view: &mut GraphView, theme: &Theme) -> (f32, f32) {
        let mut c = Compositor::new();
        view.render(&mut c, bounds(), theme);
        view.node_screen_pos(0).unwrap()
    }

    #[test]
    fn hover_click_pan_and_zoom_drive_the_state() {
        let theme = Theme::hoff();
        let mut view = GraphView::new();
        view.set_graph(&demo_graph());
        let (sx, sy) = node0_screen(&mut view, &theme);

        // Hover.
        let r = view.handle_event(&WidgetEvent::MouseMove { x: sx, y: sy }, bounds());
        assert!(r.changed);
        assert_eq!(view.hovered(), Some(0));

        // Click selects (down + up without drag).
        view.handle_event(&WidgetEvent::MouseDown { x: sx, y: sy }, bounds());
        let r = view.handle_event(&WidgetEvent::MouseUp { x: sx, y: sy }, bounds());
        assert!(r.clicked, "selection change reports clicked");
        assert_eq!(view.selected(), Some(0));

        // Click empty space clears the selection.
        view.handle_event(&WidgetEvent::MouseDown { x: 20.0, y: 20.0 }, bounds());
        let r = view.handle_event(&WidgetEvent::MouseUp { x: 20.0, y: 20.0 }, bounds());
        assert!(r.clicked);
        assert_eq!(view.selected(), None);

        // Pan: drag from empty space moves the transform 1:1.
        let before = view.transform.offset;
        view.handle_event(&WidgetEvent::MouseDown { x: 60.0, y: 200.0 }, bounds());
        view.handle_event(&WidgetEvent::MouseMove { x: 110.0, y: 240.0 }, bounds());
        view.handle_event(&WidgetEvent::MouseUp { x: 110.0, y: 240.0 }, bounds());
        assert_eq!(view.transform.offset, (before.0 + 50.0, before.1 + 40.0));

        // Wheel zoom grows the scale, anchored.
        let scale = view.transform.scale;
        let r = view.handle_event(
            &WidgetEvent::Scroll {
                x: 400.0,
                y: 300.0,
                delta: -100.0,
            },
            bounds(),
        );
        assert!(r.changed);
        assert!(view.transform.scale > scale);
    }

    #[test]
    fn events_outside_the_bounds_are_ignored_and_empty_canvas_is_inert() {
        let theme = Theme::hoff();
        let mut view = GraphView::new();
        // No graph: everything ignored.
        let r = view.handle_event(&WidgetEvent::MouseDown { x: 10.0, y: 10.0 }, bounds());
        assert_eq!(r, EventResult::IGNORED);
        let mut c = Compositor::new();
        view.render(&mut c, bounds(), &theme);

        view.set_graph(&demo_graph());
        let r = view.handle_event(&WidgetEvent::MouseMove { x: 900.0, y: 900.0 }, bounds());
        assert_eq!(r, EventResult::IGNORED);
        assert_eq!(view.hovered(), None);
    }

    #[test]
    fn renders_narrow_and_wide_without_gpu() {
        let theme = Theme::hoff();
        let mut view = GraphView::new();
        view.set_graph(&demo_graph());
        for (w, h) in [(300.0, 200.0), (1600.0, 1000.0)] {
            let mut c = Compositor::new();
            view.render(&mut c, Rect::new(0.0, 0.0, w, h), &theme);
        }
    }
}
