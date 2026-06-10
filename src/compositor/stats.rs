//! Per-frame renderer instrumentation.

/// Counters collected over a single frame.
///
/// Resolve-phase fields are filled by `Compositor::resolve`; encode-phase
/// fields (`draw_calls`, `glyphs`, `encode_micros`) are recorded by the render
/// loop via `Compositor::record_encode_stats`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    /// Draw calls issued in the layer and composite passes.
    pub draw_calls: u32,
    /// Total CPU-side quad vertices across all layers (rects + paths).
    pub quad_vertices: u32,
    /// Total CPU-side SDF vertices across all layers (rounded rects).
    pub sdf_vertices: u32,
    /// Glyph quads uploaded across all layers.
    pub glyphs: u32,
    /// Number of layers managed by the compositor.
    pub layers_total: u32,
    /// Layers whose geometry was rebuilt this frame.
    pub layers_redrawn: u32,
    /// Scene nodes skipped by viewport culling this frame.
    pub nodes_culled: u32,
    /// Time spent in `Compositor::resolve` (microseconds).
    pub resolve_micros: u64,
    /// Time spent encoding and submitting render passes (microseconds).
    pub encode_micros: u64,
}
