//! the host of the active surfaces, and the way a surface reaches the
//! app's dispatcher.

use std::collections::BTreeSet;

use sem::{Receipt, Request, SemId, Snapshot, SnapshotBuilder};
use uir::VerifiedIr;

use crate::expand::expand;
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
}

/// the active surfaces of an app, in the order they opened. each is a
/// `VerifiedIr` and the retained widgets of its nodes by `SemId`.
#[derive(Debug, Default)]
pub struct SurfaceHost {
    surfaces: Vec<Surface>,
}

impl SurfaceHost {
    pub fn new() -> SurfaceHost {
        SurfaceHost::default()
    }

    /// materializes `ir` over `app` and keeps it open. refused when a
    /// surface with its id is open, or when one of its nodes would take
    /// the id of a node another open surface holds.
    pub fn open(&mut self, ir: VerifiedIr, app: &Snapshot) -> Result<&mut Surface, SurfaceError> {
        let surface = Surface::materialize(ir, app);
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
    pub fn transform(
        &mut self,
        ir: VerifiedIr,
        app: &Snapshot,
    ) -> Result<&mut Surface, SurfaceError> {
        let id = &ir.ir().surface.id;
        let Some(at) = self.surfaces.iter().position(|s| s.id() == id) else {
            return Err(SurfaceError::NotOpen(id.clone()));
        };
        let tree = expand(&ir.ir().surface.root, app);
        let ids = std::iter::once(id).chain(tree.iter().map(|n| &n.id));
        self.collision(ids.collect(), Some(at))?;
        let surface = &mut self.surfaces[at];
        surface.transform(ir, app);
        Ok(surface)
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

    /// takes the surface `id` out: it is no longer drawn, projected or
    /// routed to.
    pub fn close(&mut self, id: &SemId) -> Option<Surface> {
        let at = self.surfaces.iter().position(|s| s.id() == id)?;
        Some(self.surfaces.remove(at))
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
