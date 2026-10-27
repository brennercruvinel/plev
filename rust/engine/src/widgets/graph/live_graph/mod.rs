//! A graph whose node positions are state: the layout writes them and so
//! does the pointer. A force graph runs its simulation frame by frame (d3's
//! timer) and a drag pins one node while the rest follow, as d3-drag does
//! (`alphaTarget(0.3)` while held). A compound graph is laid out once, group
//! by group (`compound`); a drag moves a node, or a group with all it holds,
//! and the group boxes follow. Pan (drag on empty space), wheel zoom at the
//! cursor, hover and click selection work on both, over a hit grid of what
//! was drawn (`hit`). Drawing is `draw`.

mod draw;
pub mod hit;
#[cfg(test)]
mod tests;

use web_time::Instant;

use self::hit::{Hit, HitGrid};
use super::graph_view::geometry::ViewTransform;
use super::layout::force::{Link, Params, Sim};
use super::layout::{Layout, compound};
use super::look::Look;
use super::model::Graph;
use crate::drawing::compositor::LayerId;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Below this pointer travel (screen px) a press is a click, not a drag.
const CLICK_DIST: f32 = 4.0;
/// Hit tolerance in screen px, whatever the zoom.
const HIT_TOL: f32 = 4.0;
/// Frames kept for the timing percentiles.
const WINDOW: usize = 240;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Gesture {
    Idle,
    Pan {
        last: (f32, f32),
        dist: f32,
    },
    /// `offset` is the target's anchor minus the pointer (world), so it does not jump.
    Drag {
        hit: Hit,
        offset: (f32, f32),
        last: (f32, f32),
        dist: f32,
    },
}

/// What one frame cost on the CPU, in ms: the simulation step and the scene build.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameStats {
    pub frames: u64,
    pub tick_ms: Vec<f32>,
    pub build_ms: Vec<f32>,
}

impl FrameStats {
    fn push(list: &mut Vec<f32>, ms: f32) {
        if list.len() == WINDOW {
            list.remove(0);
        }
        list.push(ms);
    }

    /// `(p50, p95, max)` of a list.
    pub fn summary(list: &[f32]) -> (f32, f32, f32) {
        if list.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let mut v = list.to_vec();
        v.sort_by(f32::total_cmp);
        let at = |q: f32| v[((v.len() - 1) as f32 * q).round() as usize];
        (at(0.5), at(0.95), v[v.len() - 1])
    }
}

pub struct LiveGraph {
    pub(super) graph: Graph,
    pub(super) look: Look,
    pub(super) pos: Vec<(f32, f32)>,
    sim: Option<Sim>,
    pub(super) boxes: Vec<Option<[f32; 4]>>,
    /// Groups outer first: the order they paint and stack for hits.
    pub(super) group_order: Vec<usize>,
    /// The world box fitted into the view: `x, y, w, h`.
    world: [f32; 4],
    pub(super) transform: ViewTransform,
    fitted: Option<Rect>,
    pub(super) hovered: Option<Hit>,
    pub(super) selected: Option<Hit>,
    gesture: Gesture,
    /// What was drawn, in world px; rebuilt when positions or boxes move,
    /// never for a pan or a zoom.
    pub(super) grid: HitGrid,
    pub(super) grid_dirty: bool,
    pub(super) overlay: Option<LayerId>,
    pub stats: FrameStats,
}

impl LiveGraph {
    /// `look` holds one look per node and per edge of `graph` ([`Look::of`]).
    ///
    /// # Panics
    ///
    /// When `look` does not fit `graph`, or when an edge or a group names
    /// a node or a group the graph does not have ([`Graph::holds`]): a
    /// reader of a source format refuses those before a graph is built.
    pub fn new(graph: Graph, look: Look, layout: Layout) -> Self {
        assert!(
            look.fits(&graph),
            "a LiveGraph's look has one NodeLook per node and one EdgeLook per edge"
        );
        assert!(
            graph.holds(),
            "a LiveGraph's edges and groups name only the graph's own nodes and groups"
        );
        let (_, group_parent) = graph.parents();
        let depth = |mut g: usize| {
            let mut d = 0;
            while let Some(p) = group_parent[g] {
                g = p;
                d += 1;
                if d > group_parent.len() {
                    break;
                }
            }
            d
        };
        let mut group_order: Vec<usize> = (0..graph.groups.len()).collect();
        group_order.sort_by_key(|&g| depth(g));
        let (pos, sim, world) = match layout {
            Layout::Force { forces, world } => {
                let links: Vec<Link> = graph
                    .edges
                    .iter()
                    .map(|e| Link {
                        source: e.from,
                        target: e.to,
                    })
                    .collect();
                let center = (world.0 as f64 / 2.0, world.1 as f64 / 2.0);
                let p = Params {
                    link_distance: forces.link_distance,
                    charge: forces.charge,
                    center,
                };
                let sim = Sim::new(&vec![forces.collide; graph.nodes.len()], &links, p);
                let pos = sim
                    .bodies
                    .iter()
                    .map(|b| (b.x as f32, b.y as f32))
                    .collect();
                (pos, Some(sim), [0.0, 0.0, world.0, world.1])
            }
            Layout::Compound { forces } => {
                let pos = compound::layout(&graph, &look, forces);
                (pos, None, [0.0; 4])
            }
        };
        let mut live = Self {
            boxes: vec![None; graph.groups.len()],
            graph,
            look,
            pos,
            sim,
            group_order,
            world,
            transform: ViewTransform::default(),
            fitted: None,
            hovered: None,
            selected: None,
            gesture: Gesture::Idle,
            grid: HitGrid::default(),
            grid_dirty: true,
            overlay: None,
            stats: FrameStats::default(),
        };
        live.refresh_boxes();
        if live.sim.is_none() {
            live.world = live.extent();
        }
        live
    }

    /// Runs the simulation to rest, for a still drawing (a test, an export).
    pub fn settle(&mut self) {
        if let Some(sim) = &mut self.sim {
            sim.settle();
        }
        self.sync();
    }

    pub fn hovered(&self) -> Option<Hit> {
        self.hovered
    }

    pub fn selected(&self) -> Option<Hit> {
        self.selected
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn look(&self) -> &Look {
        &self.look
    }

    /// Screen px per world px under the current view.
    pub fn scale(&self) -> f32 {
        self.transform.scale
    }

    /// World positions of the nodes, in the graph's order.
    pub fn positions(&self) -> &[(f32, f32)] {
        &self.pos
    }

    pub fn group_box(&self, g: usize) -> Option<[f32; 4]> {
        self.boxes.get(g).copied().flatten()
    }

    /// Screen position of a world point under the current view.
    pub fn to_screen(&self, p: (f32, f32)) -> (f32, f32) {
        self.transform.world_to_screen(p.0, p.1)
    }

    /// Whether the next frame moves anything: a warm simulation or a held drag.
    pub fn moving(&self) -> bool {
        self.sim.as_ref().is_some_and(Sim::hot) || matches!(self.gesture, Gesture::Drag { .. })
    }

    /// One frame of motion: a simulation step while warm. `true` while it
    /// still moves on its own. A drag on a force graph keeps the simulation
    /// warm (`alpha_target`), so it counts here; a drag on a compound graph
    /// moves only with the pointer, whose events already ask for the frame,
    /// so a press held still asks for none.
    pub fn tick(&mut self, _dt: f32) -> bool {
        let t0 = Instant::now();
        let hot = match &mut self.sim {
            Some(sim) if sim.hot() => {
                sim.tick();
                true
            }
            _ => false,
        };
        if hot {
            self.sync();
            FrameStats::push(&mut self.stats.tick_ms, t0.elapsed().as_secs_f32() * 1000.0);
        }
        hot
    }

    fn sync(&mut self) {
        if let Some(sim) = &self.sim {
            for (p, b) in self.pos.iter_mut().zip(&sim.bodies) {
                *p = (b.x as f32, b.y as f32);
            }
        }
        self.refresh_boxes();
    }

    fn refresh_boxes(&mut self) {
        if !self.graph.groups.is_empty() {
            self.boxes = compound::group_boxes(&self.graph, &self.look, &self.pos);
        }
        self.grid_dirty = true;
    }

    /// The box everything drawn fills, labels included: what a still graph fits.
    fn extent(&self) -> [f32; 4] {
        let mut b = [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ];
        for (i, &(x, y)) in self.pos.iter().enumerate() {
            let r = self.look.nodes[i].radius;
            b = [
                b[0].min(x - r),
                b[1].min(y - r),
                b[2].max(x + r),
                b[3].max(y + r),
            ];
        }
        for [x, y, w, h] in self.boxes.iter().flatten() {
            b = [
                b[0].min(*x),
                b[1].min(y - compound::LABEL),
                b[2].max(x + w),
                b[3].max(y + h),
            ];
        }
        if !b[0].is_finite() {
            return [0.0, 0.0, 1.0, 1.0];
        }
        [b[0], b[1], b[2] - b[0], b[3] - b[1]]
    }

    /// Fits the world into `bounds` when the bounds change (first frame, resize).
    pub(super) fn fit(&mut self, bounds: Rect, margin: f32) {
        if self.fitted == Some(bounds) {
            return;
        }
        let [x, y, w, h] = self.world;
        let mut t = ViewTransform::fit(w.max(1.0), h.max(1.0), bounds.w, bounds.h, margin);
        t.offset.0 += bounds.x - x * t.scale;
        t.offset.1 += bounds.y - y * t.scale;
        self.transform = t;
        self.fitted = Some(bounds);
    }

    pub(super) fn record_build(&mut self, ms: f32) {
        self.stats.frames += 1;
        FrameStats::push(&mut self.stats.build_ms, ms);
    }

    fn hit_at(&self, screen: (f32, f32)) -> Option<Hit> {
        let w = self.transform.screen_to_world(screen.0, screen.1);
        self.grid.hit(w, HIT_TOL / self.transform.scale)
    }

    /// The world anchor a drag holds: a node's center, or the pointer for a group.
    fn anchor(&self, hit: Hit, world: (f32, f32)) -> Option<(f32, f32)> {
        match hit {
            Hit::Node(i) => Some(self.pos[i]),
            Hit::Group(_) if self.sim.is_none() => Some(world),
            _ => None,
        }
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        let (x, y) = event.pos();
        if !bounds.contains(x, y) && self.gesture == Gesture::Idle {
            return EventResult::IGNORED;
        }
        let world = self.transform.screen_to_world(x, y);
        match *event {
            WidgetEvent::MouseDown { .. } => {
                let hit = self.hit_at((x, y));
                let anchor = hit.and_then(|h| self.anchor(h, world).map(|a| (h, a)));
                self.gesture = match anchor {
                    Some((hit, a)) => {
                        if let (Hit::Node(i), Some(sim)) = (hit, &mut self.sim) {
                            sim.bodies[i].fx = Some(a.0 as f64);
                            sim.bodies[i].fy = Some(a.1 as f64);
                            sim.alpha_target = 0.3;
                        }
                        let offset = (a.0 - world.0, a.1 - world.1);
                        Gesture::Drag {
                            hit,
                            offset,
                            last: world,
                            dist: 0.0,
                        }
                    }
                    None => Gesture::Pan {
                        last: (x, y),
                        dist: 0.0,
                    },
                };
                EventResult {
                    handled: true,
                    ..EventResult::changed()
                }
            }
            WidgetEvent::MouseMove { .. } => match self.gesture {
                Gesture::Pan { last, dist } => {
                    self.transform.pan_by(x - last.0, y - last.1);
                    let dist = dist + (x - last.0).abs() + (y - last.1).abs();
                    self.gesture = Gesture::Pan { last: (x, y), dist };
                    EventResult::changed()
                }
                Gesture::Drag {
                    hit,
                    offset,
                    last,
                    dist,
                } => {
                    let scale = self.transform.scale;
                    let moved = ((world.0 - last.0).abs() + (world.1 - last.1).abs()) * scale;
                    self.drag_to(hit, offset, last, world);
                    self.gesture = Gesture::Drag {
                        hit,
                        offset,
                        last: world,
                        dist: dist + moved,
                    };
                    EventResult::changed()
                }
                Gesture::Idle => {
                    let hit = self.hit_at((x, y));
                    if hit == self.hovered {
                        return EventResult::IGNORED;
                    }
                    self.hovered = hit;
                    EventResult::changed()
                }
            },
            WidgetEvent::MouseUp { .. } => {
                let gesture = std::mem::replace(&mut self.gesture, Gesture::Idle);
                let dist = match gesture {
                    Gesture::Idle => return EventResult::IGNORED,
                    Gesture::Pan { dist, .. } => dist,
                    Gesture::Drag { hit, dist, .. } => {
                        if let (Hit::Node(i), Some(sim)) = (hit, &mut self.sim) {
                            sim.bodies[i].fx = None;
                            sim.bodies[i].fy = None;
                            sim.alpha_target = 0.0;
                        }
                        dist
                    }
                };
                if dist < CLICK_DIST {
                    let hit = self.hit_at((x, y));
                    self.selected = if self.selected == hit { None } else { hit };
                    return EventResult::clicked();
                }
                EventResult::changed()
            }
            WidgetEvent::Scroll { delta, .. } => {
                // the hit grid is in world px: a new view leaves it valid
                self.transform.zoom_at((x, y), (-delta * 0.002).exp());
                EventResult::changed()
            }
        }
    }

    fn drag_to(&mut self, hit: Hit, offset: (f32, f32), last: (f32, f32), world: (f32, f32)) {
        let at = (world.0 + offset.0, world.1 + offset.1);
        match (hit, &mut self.sim) {
            (Hit::Node(i), Some(sim)) => {
                sim.bodies[i].fx = Some(at.0 as f64);
                sim.bodies[i].fy = Some(at.1 as f64);
            }
            (Hit::Node(i), None) => {
                self.pos[i] = at;
                self.refresh_boxes();
            }
            (Hit::Group(g), None) => {
                let (dx, dy) = (world.0 - last.0, world.1 - last.1);
                for i in self.graph.descendants(g) {
                    self.pos[i].0 += dx;
                    self.pos[i].1 += dy;
                }
                self.refresh_boxes();
            }
            _ => {}
        }
    }
}
