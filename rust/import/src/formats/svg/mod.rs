//! svg: static SVG importer for the plev engine. Normalizes an svg
//! document with usvg (groups, transforms, gradients, use/defs and the
//! shape-to-path conversion all resolved), tessellates the resulting flat
//! path tree through `engine::path::PathBuilder`, and converts it to a
//! single-frame .monster file ([`cnv`]); afterwards playback runs on
//! `monster::MonsterPlayer` and no svg code executes (import by
//! conversion, not embedding).
//!
//! Supported subset: filled and stroked paths (usvg lowers rect, circle,
//! ellipse, line, polyline and polygon to paths for us), solid colors,
//! and the whole transform/group hierarchy usvg flattens. Approximated:
//! gradient and pattern fills collapse to a representative solid color,
//! as lot does. Skipped with a one-time log, never a panic: filters
//! (blur, shadow), clip paths, masks, images and text.

pub use convert::{Stats, SvgError, convert};

pub mod convert;
pub mod tessellate;
#[cfg(test)]
mod tests;
