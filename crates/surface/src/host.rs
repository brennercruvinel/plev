//! the host of the active surfaces.

use std::collections::BTreeSet;

use sem::{SemId, Snapshot};
use uir::VerifiedIr;

use crate::surface::Surface;

/// a surface that cannot open next to the ones already open.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SurfaceError {
    /// a surface with this id is open (changing an open surface is a
    /// transformation, the next phase of the task).
    #[error("the surface {0} is already open")]
    AlreadyOpen(SemId),
    /// two open surfaces would project a node with the same id.
    #[error("{0} is already a node of the open surface {1}")]
    Collision(SemId, SemId),
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
        let ids: BTreeSet<&SemId> = std::iter::once(surface.id())
            .chain(surface.nodes())
            .collect();
        for open in &self.surfaces {
            let held = std::iter::once(open.id()).chain(open.nodes());
            if let Some(id) = held.into_iter().find(|id| ids.contains(id)) {
                return Err(SurfaceError::Collision(id.clone(), open.id().clone()));
            }
        }
        let at = self.surfaces.len();
        self.surfaces.push(surface);
        Ok(&mut self.surfaces[at])
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
}
