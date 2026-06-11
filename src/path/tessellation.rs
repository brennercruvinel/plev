//! Fill/stroke tessellation via Lyon.

use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, FillVertexConstructor, LineCap,
    LineJoin, StrokeOptions, StrokeTessellator, StrokeVertex, StrokeVertexConstructor,
    VertexBuffers,
};

use crate::compositor::QuadVertex;

use super::builder::PathBuilder;
use super::types::{TessellatedPath, compute_commands_hash};

// ---------------------------------------------------------------------------
// FillVertexConstructor bridge -- Lyon -> QuadVertex
// ---------------------------------------------------------------------------

struct WithColor([f32; 4]);

impl FillVertexConstructor<QuadVertex> for WithColor {
    fn new_vertex(&mut self, vertex: FillVertex) -> QuadVertex {
        QuadVertex {
            position: vertex.position().to_array(),
            color: self.0,
        }
    }
}

// ---------------------------------------------------------------------------
// StrokeVertexConstructor bridge -- Lyon -> QuadVertex
// ---------------------------------------------------------------------------

struct StrokeWithColor([f32; 4]);

impl StrokeVertexConstructor<QuadVertex> for StrokeWithColor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> QuadVertex {
        QuadVertex {
            position: vertex.position().to_array(),
            color: self.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Fill tessellation
// ---------------------------------------------------------------------------

pub(super) fn fill(mut pb: PathBuilder, color: [f32; 4], tolerance: f32) -> TessellatedPath {
    let hash = compute_commands_hash(&pb.commands);
    pb.finish_open();
    let path = pb.builder.build();

    let mut buffers: VertexBuffers<QuadVertex, u32> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();

    let options = FillOptions::tolerance(tolerance);
    let result = tessellator.tessellate_path(
        &path,
        &options,
        &mut BuffersBuilder::new(&mut buffers, WithColor(color)),
    );

    if let Err(e) = result {
        log::warn!("Path fill tessellation failed: {:?}", e);
        return TessellatedPath {
            vertices: Vec::new(),
            indices: Vec::new(),
            hash,
        };
    }

    TessellatedPath {
        vertices: buffers.vertices,
        indices: buffers.indices,
        hash,
    }
}

// ---------------------------------------------------------------------------
// Stroke tessellation
// ---------------------------------------------------------------------------

pub(super) fn stroke(
    pb: PathBuilder,
    color: [f32; 4],
    line_width: f32,
    tolerance: f32,
) -> TessellatedPath {
    let options = StrokeOptions::tolerance(tolerance).with_line_width(line_width);
    stroke_with_options(pb, color, options)
}

/// Stroke with round caps and joins (the Lucide icon look).
pub(super) fn stroke_round(
    pb: PathBuilder,
    color: [f32; 4],
    line_width: f32,
    tolerance: f32,
) -> TessellatedPath {
    let options = StrokeOptions::tolerance(tolerance)
        .with_line_width(line_width)
        .with_start_cap(LineCap::Round)
        .with_end_cap(LineCap::Round)
        .with_line_join(LineJoin::Round);
    stroke_with_options(pb, color, options)
}

fn stroke_with_options(
    mut pb: PathBuilder,
    color: [f32; 4],
    options: StrokeOptions,
) -> TessellatedPath {
    let hash = compute_commands_hash(&pb.commands);
    pb.finish_open();
    let path = pb.builder.build();

    let mut buffers: VertexBuffers<QuadVertex, u32> = VertexBuffers::new();
    let mut tessellator = StrokeTessellator::new();

    let result = tessellator.tessellate_path(
        &path,
        &options,
        &mut BuffersBuilder::new(&mut buffers, StrokeWithColor(color)),
    );

    if let Err(e) = result {
        log::warn!("Path stroke tessellation failed: {:?}", e);
        return TessellatedPath {
            vertices: Vec::new(),
            indices: Vec::new(),
            hash,
        };
    }

    TessellatedPath {
        vertices: buffers.vertices,
        indices: buffers.indices,
        hash,
    }
}
