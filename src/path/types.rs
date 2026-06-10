//! Core types for the path module.

use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

use crate::compositor::QuadVertex;

// ---------------------------------------------------------------------------
// TessellatedPath -- pre-tessellated geometry ready for the GPU
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct TessellatedPath {
    pub vertices: Vec<QuadVertex>,
    pub indices: Vec<u32>,
    pub hash: u64,
}

// ---------------------------------------------------------------------------
// PathCommand -- recorded commands for hashing
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) enum PathCommand {
    MoveTo(u32, u32),
    LineTo(u32, u32),
    QuadTo(u32, u32, u32, u32),
    CubicTo(u32, u32, u32, u32, u32, u32),
    Close,
}

impl PathCommand {
    pub(crate) fn hash_into(&self, h: &mut FxHasher) {
        match self {
            PathCommand::MoveTo(x, y) => {
                0u8.hash(h);
                x.hash(h);
                y.hash(h);
            }
            PathCommand::LineTo(x, y) => {
                1u8.hash(h);
                x.hash(h);
                y.hash(h);
            }
            PathCommand::QuadTo(cx, cy, x, y) => {
                2u8.hash(h);
                cx.hash(h);
                cy.hash(h);
                x.hash(h);
                y.hash(h);
            }
            PathCommand::CubicTo(c1x, c1y, c2x, c2y, x, y) => {
                3u8.hash(h);
                c1x.hash(h);
                c1y.hash(h);
                c2x.hash(h);
                c2y.hash(h);
                x.hash(h);
                y.hash(h);
            }
            PathCommand::Close => {
                4u8.hash(h);
            }
        }
    }
}

/// Compute a stable hash from a sequence of path commands.
pub(crate) fn compute_commands_hash(commands: &[PathCommand]) -> u64 {
    let mut h = FxHasher::default();
    for cmd in commands {
        cmd.hash_into(&mut h);
    }
    h.finish()
}
