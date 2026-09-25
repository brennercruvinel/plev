//! the host of the active surfaces, and the ways a surface reaches the
//! app: its dispatcher (`Dispatch`) and its frames (`Frames`).
//!
//! a surface changes form two ways: a whole new `VerifiedIr`
//! (`transform`, a new proposal), or one small operation at a time
//! (`apply`, task-70), each verified again, reconciled the same way, and
//! kept with its inverse and who asked, so the last one is undone
//! (`undo`).

use std::collections::{BTreeMap, BTreeSet};

use comps::Wake;
use engine::compositor::Compositor;
use engine::gpu::image::ImageHandle;
use engine::theme::Theme;
use sem::{Actor, Receipt, Request, SemId, Snapshot, SnapshotBuilder};
use uir::{
    Adapted, Capabilities, Context, GenerationPolicy, Inverse, Op, TransformError, VerifiedIr,
};

use crate::expand::expand;
use crate::motion::{Ghost, tick_ghosts};
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
}

/// one transformation the host applied to a surface: the operation, the
/// way back, and who asked for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub op: Op,
    pub inverse: Inverse,
    pub by: Actor,
}

/// the transformations of one open surface: the form they left it in,
/// with the nodes set aside, and every change applied, the last one last.
#[derive(Debug)]
struct Adapting {
    now: Adapted,
    changes: Vec<Change>,
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
}

impl SurfaceHost {
    pub fn new() -> SurfaceHost {
        SurfaceHost::default()
    }

    /// materializes `ir` over `app` and keeps it open. refused when a
    /// surface with its id is open, or when one of its nodes would take
    /// the id of a node another open surface holds.
    pub fn open(&mut self, ir: VerifiedIr, app: &Snapshot) -> Result<&mut Surface, SurfaceError> {
        let mut surface = Surface::materialize(ir, app);
        surface.set_reduced_motion(self.reduced);
        if self.get(surface.id()).is_some() {
            return Err(SurfaceError::AlreadyOpen(surface.id().clone()));
        }
        let ids = std::iter::once(surface.id()).chain(surface.nodes());
        self.collision(ids.collect(), None)?;
        let at = self.surfaces.len();
        self.surfaces.push(surface);
        Ok(&mut self.surfaces[at])
    }

    /// the open surface the IR names takes the form `ir` describes over
    /// `app`, reconciled by `SemId` (`Surface::transform`). refused when
    /// no surface with that id is open, or when one of the new form's
    /// nodes would take the id of a node another open surface holds; the
    /// surface keeps its form then.
    /// a whole new form ends the history of the one before: what was set
    /// aside and every change are let go.
    pub fn transform(
        &mut self,
        ir: VerifiedIr,
        app: &Snapshot,
    ) -> Result<&mut Surface, SurfaceError> {
        let id = ir.ir().surface.id.clone();
        let at = self.at(&id)?;
        self.take_form(at, ir, app)?;
        self.adapting.remove(&id);
        Ok(&mut self.surfaces[at])
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
            .or_insert_with(|| Adapting {
                now: Adapted::new(surface.verified().clone()),
                changes: Vec::new(),
            });
        if entry.now.verified() != surface.verified() {
            entry.now = Adapted::new(surface.verified().clone());
            entry.changes.clear();
        }
        entry
    }

    /// `op`, asked by `by`, on the open surface `id` over `app`: checked
    /// and verified again against the app's registry under `policy` (the
    /// policy the surface was verified under), the surface takes the form
    /// it makes, reconciled by `SemId` as `transform` does (a widget whose
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
        policy: &GenerationPolicy,
    ) -> Result<&mut Surface, SurfaceError> {
        let at = self.at(id)?;
        let caps = Capabilities::from_snapshot(app);
        let cx = Context {
            caps: &caps,
            policy,
        };
        let (next, inverse) = self
            .adapting(at)
            .now
            .apply(&op, &by, cx)
            .map_err(SurfaceError::Transform)?;
        self.take_form(at, next.verified().clone(), app)?;
        if let Some(adapting) = self.adapting.get_mut(id) {
            adapting.now = next;
            adapting.changes.push(Change { op, inverse, by });
        }
        Ok(&mut self.surfaces[at])
    }

    /// undoes the last transformation of the open surface `id`: its
    /// inverse, verified again, reconciled as `apply` does. the change
    /// undone; refused, the surface as it was, when there is none or the
    /// form before no longer verifies against the app.
    pub fn undo(
        &mut self,
        id: &SemId,
        app: &Snapshot,
        policy: &GenerationPolicy,
    ) -> Result<Change, SurfaceError> {
        let at = self.at(id)?;
        let caps = Capabilities::from_snapshot(app);
        let cx = Context {
            caps: &caps,
            policy,
        };
        let adapting = self.adapting(at);
        let Some(last) = adapting.changes.last() else {
            return Err(SurfaceError::NothingToUndo(id.clone()));
        };
        let before = adapting
            .now
            .undo(&last.inverse, cx)
            .map_err(SurfaceError::Transform)?;
        self.take_form(at, before.verified().clone(), app)?;
        let adapting = self.adapting.get_mut(id);
        let undone = adapting.and_then(|a| {
            a.now = before;
            a.changes.pop()
        });
        undone.ok_or_else(|| SurfaceError::NothingToUndo(id.clone()))
    }

    /// the transformations applied to the open surface `id` since it took
    /// its form, the last one last.
    pub fn changes(&self, id: &SemId) -> &[Change] {
        self.adapting.get(id).map_or(&[], |a| a.changes.as_slice())
    }

    /// the open surface `id` as its transformations left it, with the
    /// nodes set aside; `None` before its first.
    pub fn adapted(&self, id: &SemId) -> Option<&Adapted> {
        self.adapting.get(id).map(|a| &a.now)
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
