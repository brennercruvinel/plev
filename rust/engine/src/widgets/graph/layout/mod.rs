//! Where a graph's nodes go: the layouts, one file each, any graph
//! component may run. A layout reads the [`Graph`](super::model::Graph) and
//! the node sizes of its [`Look`](super::look::Look), never the source format.
//!
//! - [`force`]: d3-force ported, ticking frame by frame (`Layout::Force`).
//! - [`compound`]: groups solved from the inside out, once
//!   (`Layout::Compound`).

pub mod compound;
pub mod force;

/// The forces, as d3 names them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Forces {
    pub link_distance: f64,
    pub charge: f64,
    /// Collision radius of a node, world px.
    pub collide: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Layout {
    /// A live simulation in a fixed `w × h` world (d3's viewBox), centered.
    Force { forces: Forces, world: (f32, f32) },
    /// Groups solved from the inside out, once ([`compound::layout`]).
    Compound { forces: Forces },
}
