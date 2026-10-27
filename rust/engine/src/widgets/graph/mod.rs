//! Graphs: node-link canvases with pan (drag on empty space), wheel zoom at
//! the cursor, hover and click selection. Two components, one folder each:
//!
//! - [`graph_view`]: [`GraphView`], a layout computed once
//!   ([`geometry::compute_layout`], deterministic, subsampled past a few
//!   thousand nodes) and drawn as accent circles with cool and hot edges.
//!   The urna explorer and the gallery's Graph page draw this one.
//! - [`live_graph`]: [`LiveGraph`], positions as state: d3's force
//!   simulation ticking frame by frame, or a compound graph laid out group
//!   by group; nodes and groups follow the pointer, with labels, edge
//!   widths and heads ([`live_graph::net::NetSpec`]).
//!
//! The app owns the data and reads the hover and the selection back; a
//! source format (d3, cytoscape, a graph language) is read into the
//! component's spec outside the engine.

pub mod graph_view;
pub mod live_graph;

// `engine::widgets::graph::geometry` is the path the gallery and the urna
// explorer import the layout by; it stays where it was.
pub use graph_view::geometry;
pub use graph_view::{EdgeTone, GraphView};
pub use live_graph::LiveGraph;
