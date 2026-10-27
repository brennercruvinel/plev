//! Graphs. Three questions, kept apart so every kind of graph reuses the
//! answers it shares:
//!
//! - what the graph is: [`model`] ([`Graph`]: nodes, edges, groups), the
//!   data a source format is read into, with no color and no position;
//! - how it looks: [`look`] ([`Look`]: one look per node and per edge,
//!   computed from the data, in theme tones);
//! - where its nodes go: [`layout`] ([`Layout`]: d3's force simulation, or
//!   compound groups solved from the inside), one file per algorithm.
//!
//! The components draw and answer the pointer, one folder each:
//!
//! - [`live_graph`]: [`LiveGraph`], positions as state: the simulation
//!   ticking frame by frame, nodes and groups dragged, pan, zoom, hover and
//!   selection. Built from a `Graph`, a `Look` and a `Layout`.
//! - [`graph_view`]: [`GraphView`], a layout computed once
//!   ([`geometry::compute_layout`], subsampled past a few thousand nodes)
//!   from the same `Graph` (`set_graph`), or from a `GraphData` index an
//!   app fills itself; the urna explorer and the gallery's Graph page draw
//!   it.

pub mod graph_view;
pub mod layout;
pub mod live_graph;
pub mod look;
pub mod model;

// `engine::widgets::graph::geometry` is the path the gallery and the urna
// explorer import the layout by; it stays where it was.
pub use graph_view::geometry;
pub use graph_view::{EdgeTone, GraphView};
pub use layout::{Forces, Layout};
pub use live_graph::LiveGraph;
pub use look::{EdgeLook, LabelAt, Look, NodeLook, Tone};
pub use model::{Edge, Graph, Group, Node};
