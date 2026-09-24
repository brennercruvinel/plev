//! the transitions of a surface that changes form: what the frames show
//! while a node is born, leaves or moves. identity and semantics do not
//! wait for them. a node that is born is projected, bound and hit at
//! once, while it fades in; a node that left is out of the projection at
//! once, and only its picture stays, fading out where it was, never bound
//! or hit (a `Ghost`); a node that moved is where it is drawn, sliding
//! from where it was.
//!
//! no animation engine of its own: the entry and the exit are the
//! overlay's progress spring (`Spring<f32>` on `theme.motion`, 0 hidden to
//! 1 shown, as `OverlayManager::push_animated` and `pop_animated` run it,
//! drawn as the opacity), and the move is an engine `Tween` over
//! `theme.duration.slow`, the sliding transitions' length. a transition
//! starts at the first frame that draws the new form, with that frame's
//! theme and layout, and asks for frames until it settles (`Motion::wake`);
//! at rest nothing is asked. with reduced motion every change is
//! instant: nothing fades, nothing slides, nothing is kept after it left.

use std::collections::BTreeMap;
use std::sync::Arc;

use comps::{Rect, Wake};
use engine::animation::{Easing, Spring, Tween};
use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::theme::Theme;
use sem::{SemId, Snapshot};

use crate::draw;
use crate::expand::Node;
use crate::widget::{Kind, Widget};

/// the smallest step of opacity a frame shows: one level of an 8-bit
/// channel. a fade within it of its end changes no pixel, so it is over.
/// the engine spring's own rest threshold (0.001 on value and velocity)
/// asks for half as many frames again, all of them the same picture (the
/// mixer's change of form: 56 turns to rest, 37 to within a step).
const STEP: f32 = 1.0 / 255.0;

/// the entry and exit progress of the overlay, from `from` to `to`.
fn spring(from: f32, to: f32, theme: &Theme) -> Spring<f32> {
    let mut s = Spring::new(from).with_motion(&theme.motion);
    s.set_target(to);
    s
}

/// advances a fade toward `to` by `dt`; true while a frame would still
/// show it move.
fn fading(s: &mut Spring<f32>, to: f32, dt: f32) -> bool {
    s.tick(dt);
    s.is_animating() && (s.get() - to).abs() >= STEP
}

fn lerp(a: Rect, b: Rect, t: f32) -> Rect {
    let at = |a: f32, b: f32| a + (b - a) * t;
    Rect::new(at(a.x, b.x), at(a.y, b.y), at(a.w, b.w), at(a.h, b.h))
}

/// two rects no pixel apart: no move to show.
fn same(a: Rect, b: Rect) -> bool {
    [a.x - b.x, a.y - b.y, a.w - b.w, a.h - b.h]
        .iter()
        .all(|d| d.abs() < 0.5)
}

/// `node` as it shows at `alpha`: the alpha of every color scaled (a
/// path's vertices too, its hash moved with them so the layer sees the
/// change). an image or a backdrop blur has no color to scale: it shows
/// whole down to half, and not below.
fn faded(node: &SceneNode, alpha: f32) -> Option<SceneNode> {
    let scale = |c: &mut [f32; 4]| c[3] *= alpha;
    let mut node = node.clone();
    match &mut node {
        SceneNode::Rect { color, .. } | SceneNode::Text { color, .. } => scale(color),
        SceneNode::Shadow { color, .. } => scale(color),
        SceneNode::RoundedRect {
            color,
            border_color,
            ..
        } => {
            scale(color);
            scale(border_color);
        }
        SceneNode::GradientRect {
            color,
            color2,
            border_color,
            ..
        } => {
            scale(color);
            scale(color2);
            scale(border_color);
        }
        SceneNode::Path { data } => {
            for v in &mut data.vertices {
                scale(&mut v.color);
            }
            data.hash = data.hash.rotate_left(17) ^ u64::from(alpha.to_bits());
        }
        SceneNode::Image { .. } | SceneNode::BackdropBlur { .. } if alpha < 0.5 => return None,
        SceneNode::Image { .. }
        | SceneNode::BackdropBlur { .. }
        | SceneNode::PushClip { .. }
        | SceneNode::PopClip => {}
    }
    Some(node)
}

/// what `draw` draws, into `c` at `alpha`: first into a compositor of its
/// own, then each scene node it pushed, faded, into `c`. `bind` carries
/// over the bindings it made: a node fading in is a capability already, a
/// ghost is not.
pub(crate) fn draw_faded(
    c: &mut Compositor,
    alpha: f32,
    bind: bool,
    draw: impl FnOnce(&mut Compositor),
) {
    let mut own = Compositor::new();
    draw(&mut own);
    let pushed = own.layer(LayerId::DEFAULT).map_or(&[][..], |l| l.nodes());
    for n in pushed.iter().filter_map(|n| faded(n, alpha)) {
        c.push(n);
    }
    if bind {
        for (id, b) in own.bindings() {
            c.bind(id, b.rect);
        }
    }
}

/// the picture of a node that left: its node, its widget and the state it
/// showed, drawn where it was until its exit settles. never projected,
/// bound, hit or routed to.
#[derive(Debug)]
pub(crate) struct Ghost {
    node: Node,
    widget: Widget,
    rect: Rect,
    shown: Arc<Snapshot>,
    /// the opacity it leaves from (it may have been fading in).
    from: f32,
    /// the exit, from the first frame that draws it.
    fade: Option<Spring<f32>>,
}

impl Ghost {
    /// `node` leaves from `rect`, shown over `shown` at the opacity
    /// `from`. a field it held gives the keyboard up: a ghost takes no
    /// input.
    pub fn new(
        node: Node,
        mut widget: Widget,
        rect: Rect,
        shown: Arc<Snapshot>,
        from: f32,
    ) -> Ghost {
        if let Some(f) = widget.field() {
            f.unfocus();
        }
        Ghost {
            node,
            widget,
            rect,
            shown,
            from,
            fade: None,
        }
    }

    fn opacity(&self) -> f32 {
        self.fade
            .as_ref()
            .map_or(self.from, |s| s.get().clamp(0.0, 1.0))
    }

    /// true while it still shows: not started, or its exit in flight.
    fn tick(&mut self, dt: f32) -> bool {
        match &mut self.fade {
            Some(s) => fading(s, 0.0, dt),
            None => true,
        }
    }

    /// draws it at its opacity, starting its exit on its first frame.
    pub fn draw(&mut self, c: &mut Compositor, theme: &Theme) {
        let from = self.from;
        self.fade.get_or_insert_with(|| spring(from, 0.0, theme));
        let alpha = self.opacity();
        let Ghost {
            node,
            widget,
            rect,
            shown,
            ..
        } = self;
        draw_faded(c, alpha, false, |own| {
            draw::draw(own, node, widget, *rect, shown, theme);
        });
    }
}

/// the ghosts of `leaving`, advanced by `dt`: the ones whose exit settled
/// are dropped, and that frame is a visible change; while any fades, the
/// next frame is wanted.
pub(crate) fn tick_ghosts(leaving: &mut Vec<Ghost>, dt: f32) -> Wake {
    let before = leaving.len();
    leaving.retain_mut(|g| g.tick(dt));
    let mut wake = if leaving.len() < before {
        Wake::now()
    } else {
        Wake::idle()
    };
    if !leaving.is_empty() {
        wake = wake.merge(Wake::continuous());
    }
    wake
}

/// a node moving from where it was drawn to where the layout puts it.
#[derive(Debug)]
struct Slide {
    from: Rect,
    t: Tween<f32>,
}

/// the transitions of one surface.
#[derive(Debug, Default)]
pub(crate) struct Motion {
    reduced: bool,
    /// a change of form the next frame has not drawn yet: the ids that
    /// get a new widget, each with the opacity it enters from (0, or what
    /// a ghost it came back from still showed).
    pending: Option<BTreeMap<SemId, f32>>,
    entering: BTreeMap<SemId, Spring<f32>>,
    sliding: BTreeMap<SemId, Slide>,
    leaving: Vec<Ghost>,
}

impl Motion {
    /// reduced motion makes every change instant, the ones in flight
    /// included.
    pub fn set_reduced(&mut self, reduced: bool) {
        self.reduced = reduced;
        if reduced {
            self.entering.clear();
            self.sliding.clear();
            self.leaving.clear();
            if let Some(born) = &mut self.pending {
                born.clear();
            }
        }
    }

    /// the form changed: the next frame draws it and starts what it
    /// shows. `born` are the ids with a new widget.
    pub fn changed(&mut self, born: impl IntoIterator<Item = (SemId, f32)>) {
        let pending = self.pending.get_or_insert_with(BTreeMap::new);
        if !self.reduced {
            pending.extend(born);
        }
    }

    /// `node` left (or its widget did): its picture fades out from
    /// `rect`, unless motion is reduced.
    pub fn leave(&mut self, node: Node, widget: Widget, rect: Rect, shown: &Arc<Snapshot>) {
        if self.reduced {
            return;
        }
        let from = self.opacity(&node.id);
        self.entering.remove(&node.id);
        self.sliding.remove(&node.id);
        self.leaving
            .push(Ghost::new(node, widget, rect, Arc::clone(shown), from));
    }

    /// the widget of a ghost `id` left as, when it comes back under the
    /// same widget: the instance, the opacity it still showed and where
    /// it showed.
    pub fn reclaim(&mut self, id: &SemId, kind: Kind) -> Option<(Widget, f32, Rect)> {
        let at = self
            .leaving
            .iter()
            .position(|g| g.node.id == *id && g.widget.kind() == kind)?;
        let g = self.leaving.remove(at);
        let opacity = g.opacity();
        Some((g.widget, opacity, g.rect))
    }

    /// takes the change the frame being drawn shows: `Some` once after a
    /// change of form.
    pub fn take_change(&mut self) -> Option<BTreeMap<SemId, f32>> {
        self.pending.take()
    }

    /// `id` is born in this frame, from the opacity `from`.
    pub fn enter(&mut self, id: &SemId, from: f32, theme: &Theme) {
        if !self.reduced && from < 1.0 {
            self.entering.insert(id.clone(), spring(from, 1.0, theme));
        }
    }

    /// `id`, drawn at `was` before the change, is laid out at `to` in
    /// this frame: it slides there.
    pub fn slide(&mut self, id: &SemId, was: Rect, to: Rect, theme: &Theme) {
        if self.reduced || same(was, to) {
            self.sliding.remove(id);
            return;
        }
        let mut t = Tween::new(0.0_f32, theme.duration.slow, Easing::EaseInOutCubic);
        t.set_target(1.0);
        self.sliding.insert(id.clone(), Slide { from: was, t });
    }

    /// where `id` is drawn when the layout puts it at `to`.
    pub fn rect(&self, id: &SemId, to: Rect) -> Rect {
        match self.sliding.get(id) {
            Some(s) => lerp(s.from, to, s.t.get()),
            None => to,
        }
    }

    /// the opacity `id` is drawn at: below 1 while it fades in.
    pub fn opacity(&self, id: &SemId) -> f32 {
        self.entering
            .get(id)
            .map_or(1.0, |s| s.get().clamp(0.0, 1.0))
    }

    pub fn leaving_mut(&mut self) -> &mut [Ghost] {
        &mut self.leaving
    }

    /// everything that shows the surface's nodes, taken for its
    /// dismissal: the ghosts in flight.
    pub fn take_leaving(&mut self) -> Vec<Ghost> {
        std::mem::take(&mut self.leaving)
    }

    /// advances what is in flight by `dt`: a frame now when a change of
    /// form waits to be drawn or a transition settled, and every frame
    /// while one runs.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let mut wake = if self.pending.is_some() {
            Wake::now()
        } else {
            Wake::idle()
        };
        let before = self.entering.len() + self.sliding.len();
        self.entering.retain(|_, s| fading(s, 1.0, dt));
        self.sliding.retain(|_, s| {
            s.t.tick(dt);
            s.t.is_animating()
        });
        let after = self.entering.len() + self.sliding.len();
        if after < before {
            wake = wake.merge(Wake::now());
        }
        if after > 0 {
            wake = wake.merge(Wake::continuous());
        }
        wake.merge(tick_ghosts(&mut self.leaving, dt))
    }
}
