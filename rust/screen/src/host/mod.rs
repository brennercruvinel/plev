//! the host of the active surfaces, and the ways a surface reaches the
//! app: its dispatcher (`Dispatch`) and its frames (`Frames`).
//!
//! a surface changes form two ways: a whole new `VerifiedIr`
//! (`transform`, a new proposal), or small operations (`apply`, one;
//! `apply_all`, the ones a person or an agent asked for together,
//! task-70), each verified again under the policy the surface was
//! verified under (kept here since it opened), reconciled the same way,
//! and kept with its inverse and who asked, so the last one is undone
//! (`undo`), or the last ask whole (`undo_asked`).
//!
//! a `promote` may name the template of an item (`silences.cut[{item}]`,
//! task-70 phase 3: a composite runs on each item): the button then goes
//! inside the item's template, one per item; `uir::Op::Promote` finds the
//! action on the items the template stands for (task-70 phase 4).
//!
//! the person's preferences of accessibility (`prefer`, task-70 phase 4:
//! larger text, more contrast, a density) are the last layer of every
//! surface the host shows: the host keeps each surface as it was asked
//! (its form and the changes asked, by anyone) and draws it with the
//! operations of the preferences over it (`uir::Preferences::over`), made
//! again after every change, in the name of the person who turned them
//! on, and kept in the same history (`Change::preference`). a surface
//! opens with them already, before its first frame; turning one off makes
//! the asked surface again without it, so what others asked stays and the
//! rest comes back byte for byte. what a transformation is asked about,
//! and what a model is shown, is the asked surface (`current`): the
//! preferences never leave the host.

mod prefer;

use std::collections::{BTreeMap, BTreeSet};

use engine::widgets::Wake;
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::image::ImageHandle;
use engine::styling::theme::Theme;
use accord::{Actor, Receipt, Request, SemId, Snapshot, SnapshotBuilder};
use plevir::{
    Adapted, Capabilities, Context, GenerationPolicy, Inverse, Op, Preference, Preferences,
    TransformError, VerifiedIr,
};

use crate::expand::expand;
use crate::transition::{Ghost, tick_ghosts};
use crate::surface::Surface;

/// the app, as a surface reaches it: its one dispatcher and its
/// projection. the surface builds the request; the app runs it, the way
/// it runs a click of its own, an agent's act or a screen reader's.
pub trait Dispatch {
    /// runs `req` through the app's dispatcher; the receipt as it stands
    /// after. `None` when the app took no request.
    fn dispatch(&mut self, req: Request) -> Option<Receipt>;

    /// the app projected now, what the surface shows after its request.
    fn project(&self) -> Option<Snapshot>;
}

/// the frames an app holds for the images of a surface. an `image` binds
/// its `source` to a text of the app: a frame ref the app projects, naming
/// a media node and an instant (`silences.video@4.017052`), the app's own
/// form. when it draws, the host asks the app for the picture by that
/// text; `None` draws the shell with the label, and the app decodes on
/// its own time (a worker, a cache, from its wake), so the next frame
/// asks again. the surface never holds bytes, never decodes, and never
/// asks for a decode itself: `Surface::sources` tells the app what it
/// shows.
pub trait Frames {
    fn frame(&self, source: &str) -> Option<ImageHandle>;
}

/// no frames: every image is its shell and label (a surface drawn
/// without an app that holds pictures).
#[derive(Clone, Copy, Debug, Default)]
pub struct NoFrames;

impl Frames for NoFrames {
    fn frame(&self, _source: &str) -> Option<ImageHandle> {
        None
    }
}

/// a surface that cannot open next to the ones already open, or a form it
/// cannot take.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SurfaceError {
    /// a surface with this id is open (changing an open surface is a
    /// transformation: `SurfaceHost::transform`).
    #[error("the surface {0} is already open")]
    AlreadyOpen(SemId),
    /// two open surfaces would project a node with the same id.
    #[error("{0} is already a node of the open surface {1}")]
    Collision(SemId, SemId),
    /// no open surface has the id the IR names, so there is nothing to
    /// transform.
    #[error("no open surface is {0}")]
    NotOpen(SemId),
    /// the operation makes no surface: refused, or what it makes does not
    /// verify. the surface keeps its form.
    #[error("{0}")]
    Transform(TransformError),
    /// the surface has no transformation left to undo.
    #[error("the surface {0} has no transformation to undo")]
    NothingToUndo(SemId),
    /// the preferences of accessibility are the person's: an agent does
    /// not set them.
    #[error("the preferences of accessibility are the person's: {0} does not set them")]
    NotThePerson(Actor),
}

/// one transformation the host applied to a surface: the operation, the
/// way back, who asked for it, and, for an operation made for one of the
/// person's preferences of accessibility, which one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub op: Op,
    pub inverse: Inverse,
    pub by: Actor,
    /// the preference the operation is made for; `None` when someone
    /// asked for it.
    pub preference: Option<Preference>,
}

/// the transformations of one open surface: the surface as it was asked
/// (its form and the changes asked, no preference), the one drawn (the
/// asked one with the preferences over it), both with the nodes set
/// aside; every change, the ones asked in order and then the
/// preferences' (the last `preferred`); and how many changes each ask
/// made, in order.
#[derive(Clone, Debug)]
struct Adapting {
    asked: Adapted,
    now: Adapted,
    changes: Vec<Change>,
    preferred: usize,
    asks: Vec<usize>,
}

/// the active surfaces of an app, in the order they opened. each is a
/// `VerifiedIr` and the retained widgets of its nodes by `SemId`.
#[derive(Debug, Default)]
pub struct SurfaceHost {
    surfaces: Vec<Surface>,
    /// the pictures of the surfaces dismissed, fading out where they
    /// were: drawn, never projected, routed to or hit.
    closing: Vec<Ghost>,
    reduced: bool,
    /// the transformations of each surface since it took its form.
    adapting: BTreeMap<SemId, Adapting>,
    /// the policy each open surface was verified under: its operations
    /// are verified under it again.
    policies: BTreeMap<SemId, GenerationPolicy>,
    /// the person's preferences of accessibility, over every surface.
    preferences: Preferences,
    /// who turned them on: the operations are made in their name.
    person: Option<Actor>,
}

impl SurfaceHost {
    pub fn new() -> SurfaceHost {
        SurfaceHost::default()
    }

    /// materializes `ir` over `app` and keeps it open, as verified under
    /// the default policy (`open_under`). refused when a surface with its
    /// id is open, or when one of its nodes would take the id of a node
    /// another open surface holds.
    pub fn open(&mut self, ir: VerifiedIr, app: &Snapshot) -> Result<&mut Surface, SurfaceError> {
        self.open_under(ir, app, GenerationPolicy::default())
    }

    /// `open`, with the policy `ir` was verified under: every operation
    /// on the surface is verified under it again.
    pub fn open_under(
        &mut self,
        ir: VerifiedIr,
        app: &Snapshot,
        policy: GenerationPolicy,
    ) -> Result<&mut Surface, SurfaceError> {
        let preferred = self.preferred(ir, app, &policy);
        let mut surface = Surface::materialize(preferred.now.verified().clone(), app);
        surface.set_reduced_motion(self.reduced);
        if self.get(surface.id()).is_some() {
            return Err(SurfaceError::AlreadyOpen(surface.id().clone()));
        }
        let ids = std::iter::once(surface.id()).chain(surface.nodes());
        self.collision(ids.collect(), None)?;
        let at = self.surfaces.len();
        self.policies.insert(surface.id().clone(), policy);
        if !self.preferences.is_empty() {
            self.adapting.insert(surface.id().clone(), preferred);
        }
        self.surfaces.push(surface);
        Ok(&mut self.surfaces[at])
    }

    /// the open surface the IR names takes the form `ir` describes over
    /// `app`, reconciled by `SemId` (`Surface::transform`). refused when
    /// no surface with that id is open, or when one of the new form's
    /// nodes would take the id of a node another open surface holds; the
    /// surface keeps its form then.
    /// a whole new form ends the history of the one before: what was set
    /// aside and every change are let go. the form is taken as verified
    /// under the default policy (`transform_under`).
    pub fn transform(
        &mut self,
        ir: VerifiedIr,
        app: &Snapshot,
    ) -> Result<&mut Surface, SurfaceError> {
        self.transform_under(ir, app, GenerationPolicy::default())
    }

    /// `transform`, with the policy the new form was verified under.
    pub fn transform_under(
        &mut self,
        ir: VerifiedIr,
        app: &Snapshot,
        policy: GenerationPolicy,
    ) -> Result<&mut Surface, SurfaceError> {
        let id = ir.ir().surface.id.clone();
        let at = self.at(&id)?;
        let preferred = self.preferred(ir, app, &policy);
        self.take_form(at, preferred.now.verified().clone(), app)?;
        self.adapting.remove(&id);
        if !self.preferences.is_empty() {
            self.adapting.insert(id.clone(), preferred);
        }
        self.policies.insert(id, policy);
        Ok(&mut self.surfaces[at])
    }

    /// the policy the open surface `id` was verified under.
    pub fn policy(&self, id: &SemId) -> Option<&GenerationPolicy> {
        self.get(id).and(self.policies.get(id))
    }

    /// where the open surface `id` is, or `NotOpen`.
    fn at(&self, id: &SemId) -> Result<usize, SurfaceError> {
        self.surfaces
            .iter()
            .position(|s| s.id() == id)
            .ok_or_else(|| SurfaceError::NotOpen(id.clone()))
    }

    /// the surface at `at` takes the form `ir` over `app`, reconciled by
    /// `SemId`, unless a node of it would take an id another open surface
    /// holds.
    fn take_form(&mut self, at: usize, ir: VerifiedIr, app: &Snapshot) -> Result<(), SurfaceError> {
        let tree = expand(&ir.ir().surface.root, app);
        let id = &ir.ir().surface.id;
        let ids = std::iter::once(id).chain(tree.iter().map(|n| &n.id));
        self.collision(ids.collect(), Some(at))?;
        self.surfaces[at].transform(ir, app);
        Ok(())
    }

    /// the transformations of the open surface at `at`, begun afresh when
    /// its form is not the one they left (it took a form some other way).
    fn adapting(&mut self, at: usize) -> &mut Adapting {
        let surface = &self.surfaces[at];
        let entry = self
            .adapting
            .entry(surface.id().clone())
            .or_insert_with(|| Adapting::fresh(Adapted::new(surface.verified().clone())));
        if entry.now.verified() != surface.verified() {
            *entry = Adapting::fresh(Adapted::new(surface.verified().clone()));
        }
        entry
    }

    /// `op`, asked by `by`, on the open surface `id` over `app`: checked
    /// and verified again against the app's registry under the policy
    /// the surface was verified under, the surface takes the form it
    /// makes, reconciled by `SemId` as `transform` does (a widget whose
    /// node stays keeps what lives in it), and the change is kept for
    /// `undo`. refused, the surface as it was, when it is not open, when
    /// the operation makes no verified surface, or when the form would
    /// take an id another open surface holds.
    pub fn apply(
        &mut self,
        id: &SemId,
        op: Op,
        by: Actor,
        app: &Snapshot,
    ) -> Result<&mut Surface, SurfaceError> {
        self.apply_all(id, vec![op], by, app)
    }

    /// `ops`, asked together by `by`, as `apply` does each, in order,
    /// each over what the ones before made: all or none. they apply to the
    /// surface as it was asked, and the preferences are made again over
    /// what they make. the surface takes the form once, and the changes
    /// are kept as one ask (`undo_asked`). an empty list changes nothing.
    pub fn apply_all(
        &mut self,
        id: &SemId,
        ops: Vec<Op>,
        by: Actor,
        app: &Snapshot,
    ) -> Result<&mut Surface, SurfaceError> {
        let at = self.at(id)?;
        let caps = Capabilities::from_snapshot(app);
        let policy = self.policies.get(id).cloned().unwrap_or_default();
        let cx = Context {
            caps: &caps,
            policy: &policy,
        };
        let mut next = self.adapting(at).clone();
        let mut changes = Vec::with_capacity(ops.len());
        for op in ops {
            let (made, inverse) = next
                .asked
                .apply(&op, &by, cx)
                .map_err(SurfaceError::Transform)?;
            next.asked = made;
            changes.push(Change {
                op,
                inverse,
                by: by.clone(),
                preference: None,
            });
        }
        if changes.is_empty() {
            return Ok(&mut self.surfaces[at]);
        }
        let asked = next.asked_len();
        next.changes.truncate(asked);
        next.asks.push(changes.len());
        next.changes.extend(changes);
        next.preferred = 0;
        self.settle(at, next, cx, app)?;
        Ok(&mut self.surfaces[at])
    }

    /// undoes the last transformation of the open surface `id`: its
    /// inverse, verified again, reconciled as `apply` does. the change
    /// undone; refused, the surface as it was, when there is none or the
    /// form before no longer verifies against the app.
    pub fn undo(&mut self, id: &SemId, app: &Snapshot) -> Result<Change, SurfaceError> {
        let mut undone = self.take_back(id, app, |_| 1)?;
        undone
            .pop()
            .ok_or_else(|| SurfaceError::NothingToUndo(id.clone()))
    }

    /// undoes the last ask on the open surface `id`, every change it made
    /// (the last first), as `undo` does one: the form before them taken
    /// once. the changes undone, in the order they were made.
    pub fn undo_asked(&mut self, id: &SemId, app: &Snapshot) -> Result<Vec<Change>, SurfaceError> {
        self.take_back(id, app, |a| a.asks.last().copied().unwrap_or(0))
    }

    /// the last `count(adapting)` changes of the open surface `id`
    /// undone, verified again, the form they leave taken once.
    fn take_back(
        &mut self,
        id: &SemId,
        app: &Snapshot,
        count: impl Fn(&Adapting) -> usize,
    ) -> Result<Vec<Change>, SurfaceError> {
        let at = self.at(id)?;
        let caps = Capabilities::from_snapshot(app);
        let policy = self.policies.get(id).cloned().unwrap_or_default();
        let cx = Context {
            caps: &caps,
            policy: &policy,
        };
        let mut next = self.adapting(at).clone();
        let made = next.asked_len();
        let n = count(&next).min(made);
        if n == 0 {
            return Err(SurfaceError::NothingToUndo(id.clone()));
        }
        for change in next.changes[made - n..made].iter().rev() {
            next.asked = next
                .asked
                .undo(&change.inverse, cx)
                .map_err(SurfaceError::Transform)?;
        }
        next.changes.truncate(made);
        let undone = next.changes.split_off(made - n);
        next.preferred = 0;
        let mut left = n;
        while left > 0 {
            match next.asks.last_mut() {
                Some(k) if *k > left => {
                    *k -= left;
                    left = 0;
                }
                Some(k) => {
                    left -= *k;
                    next.asks.pop();
                }
                None => left = 0,
            }
        }
        self.settle(at, next, cx, app)?;
        Ok(undone)
    }

    /// the transformations applied to the open surface `id` since it took
    /// its form: the ones asked, the last one last, then the ones made for
    /// the person's preferences (`Change::preference`).
    pub fn changes(&self, id: &SemId) -> &[Change] {
        self.adapting.get(id).map_or(&[], |a| a.changes.as_slice())
    }

    /// the changes of the last ask on the open surface `id`, in the order
    /// they were made: what `undo_asked` takes back.
    pub fn last_ask(&self, id: &SemId) -> &[Change] {
        self.adapting.get(id).map_or(&[], |a| {
            let asked = a.asked_len();
            let n = a.asks.last().copied().unwrap_or(0).min(asked);
            &a.changes[asked - n..asked]
        })
    }

    /// the open surface `id` as its transformations left it, the
    /// preferences' included, with the nodes set aside: what is drawn and
    /// projected. `None` before its first.
    pub fn adapted(&self, id: &SemId) -> Option<&Adapted> {
        self.adapting.get(id).map(|a| &a.now)
    }

    /// the open surface `id` as it was asked, with the nodes set aside:
    /// what a transformation is asked about and what a model is shown,
    /// never the person's preferences (they are made over it). its form
    /// alone, nothing aside, before its first transformation or after a
    /// whole new form.
    pub fn current(&self, id: &SemId) -> Option<Adapted> {
        let surface = self.get(id)?;
        Some(match self.adapting.get(id) {
            Some(a) if a.now.verified() == surface.verified() => a.asked.clone(),
            _ => Adapted::new(surface.verified().clone()),
        })
    }

    /// the first of `ids` that an open surface other than the one at
    /// `except` holds, as the error.
    fn collision(&self, ids: BTreeSet<&SemId>, except: Option<usize>) -> Result<(), SurfaceError> {
        let others = self
            .surfaces
            .iter()
            .enumerate()
            .filter(|&(at, _)| Some(at) != except);
        for (_, open) in others {
            let held = std::iter::once(open.id()).chain(open.nodes());
            if let Some(id) = held.into_iter().find(|id| ids.contains(id)) {
                return Err(SurfaceError::Collision(id.clone(), open.id().clone()));
            }
        }
        Ok(())
    }

    pub fn get(&self, id: &SemId) -> Option<&Surface> {
        self.surfaces.iter().find(|s| s.id() == id)
    }

    pub fn get_mut(&mut self, id: &SemId) -> Option<&mut Surface> {
        self.surfaces.iter_mut().find(|s| s.id() == id)
    }

    /// dismisses the surface `id`: at once it is no longer projected,
    /// routed to or hit, and an act on one of its nodes is refused as on
    /// any node that does not exist; its picture fades out where it was
    /// drawn (`draw_closing`) and then everything it held is let go. true
    /// when it was open.
    pub fn close(&mut self, id: &SemId) -> bool {
        let Some(at) = self.surfaces.iter().position(|s| s.id() == id) else {
            return false;
        };
        let surface = self.surfaces.remove(at);
        self.adapting.remove(surface.id());
        self.policies.remove(surface.id());
        self.closing.extend(surface.dismiss());
        true
    }

    /// draws what is left of the surfaces dismissed, fading out where
    /// they were; nothing once they are gone. their images are their
    /// shells (`draw_closing_with` for the pictures).
    pub fn draw_closing(&mut self, c: &mut Compositor, theme: &Theme) {
        self.draw_closing_with(c, theme, &NoFrames);
    }

    /// `draw_closing`, the pictures of the images asked of `frames`.
    pub fn draw_closing_with(&mut self, c: &mut Compositor, theme: &Theme, frames: &dyn Frames) {
        for ghost in &mut self.closing {
            ghost.draw(c, theme, frames);
        }
    }

    /// the frame refs the images of every open surface show now
    /// (`Surface::sources`), once each: what the app decodes for them.
    pub fn sources(&self) -> Vec<String> {
        let mut out = Vec::new();
        for surface in &self.surfaces {
            for source in surface.sources() {
                if !out.contains(&source) {
                    out.push(source);
                }
            }
        }
        out
    }

    /// the person's preference for reduced motion, for every surface open
    /// and to open: a change of form and a dismissal show at once.
    pub fn set_reduced_motion(&mut self, reduced: bool) {
        self.reduced = reduced;
        for surface in &mut self.surfaces {
            surface.set_reduced_motion(reduced);
        }
        if reduced {
            self.closing.clear();
        }
    }

    /// what the surfaces ask of the loop (`Surface::wake`), and the
    /// dismissed ones while they fade.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let open = self
            .surfaces
            .iter_mut()
            .fold(Wake::idle(), |w, s| w.merge(s.wake(dt)));
        open.merge(tick_ghosts(&mut self.closing, dt))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Surface> {
        self.surfaces.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Surface> {
        self.surfaces.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.surfaces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }

    /// every surface reads the app's state again when it moved; true
    /// when one did.
    pub fn sync(&mut self, app: &Snapshot) -> bool {
        let mut moved = false;
        for surface in &mut self.surfaces {
            moved |= surface.sync(app);
        }
        moved
    }

    /// every surface projected into `b`, after the app's own projection.
    pub fn project(&self, b: &mut SnapshotBuilder) {
        for surface in &self.surfaces {
            surface.project(b);
        }
    }

    /// `req` as the capability it asks for when it aims at a node of an
    /// open surface; as it is otherwise.
    pub fn route(&self, req: Request) -> Request {
        match self.surfaces.iter().find(|s| s.holds(&req.target)) {
            Some(surface) => surface.route(req),
            None => req,
        }
    }
}
