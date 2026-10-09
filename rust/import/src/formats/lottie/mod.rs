//! lot: lottie (bodymovin) importer for the plev engine. Reads the
//! json once and either renders it directly ([`rnd`]) or converts it
//! to our .monster format ([`cnv`]); after conversion playback runs on
//! `monster::MonsterPlayer` and no lottie code executes.
//!
//! Supported subset: shape layers (ty 4), null layers (ty 3) with
//! parenting, precomps (ty 0), static and keyframed transforms with
//! bezier easing, shapes gr/sh/el/rc, fills (fl), strokes (st),
//! gradient fill/stroke approximated as solid color. Unsupported
//! features (masks, mattes, trim paths, text, images, expressions)
//! are skipped with a one-time log, never a panic.

pub mod convert;
pub mod geometry;
pub mod keyframe;
pub mod render;
pub mod schema;

pub use convert::{CnvError, Stats, convert};
pub use geometry::Mat;
pub use render::Player;
pub use schema::Animation;
