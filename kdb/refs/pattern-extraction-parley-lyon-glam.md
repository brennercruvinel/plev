---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: research
---

# pattern extraction: parley, lyon, glam-rs

extracted 2026-03-11 by source code study of reference repos.
source repos: `bunker/repos/3d-graphics/rendering/{parley,lyon,glam-rs}/`

---

## pattern 1: byte-index cursor with affinity (parley)

**source:** `parley/parley/src/editing/cursor.rs` lines 14-67

**description:**
parley's `Cursor` uses a **byte index** into the source string (not char index, not grapheme index) combined with an `Affinity` enum (`Upstream` / `Downstream`). affinity disambiguates the visual position when a byte index falls at a line break boundary, the same byte index can appear at the end of line n or the beginning of line n+1. the cursor is always snapped to a cluster (grapheme) boundary via `Cluster::from_byte_index()`.

key design:
```rust
pub struct Cursor {
    index: usize,        // byte offset in source text
    affinity: Affinity,  // Upstream = came from left, Downstream = came from right
}
```

construction from pixel coordinates:
```rust
pub fn from_point<B: Brush>(layout: &Layout<B>, x: f32, y: f32) -> Self {
    let (index, affinity) = if let Some((cluster, side)) = Cluster::from_point(layout, x, y) {
        // snap to cluster start/end depending on which half was hit
        ...
    };
    Self { index, affinity }
}
```

cursor geometry (for rendering the caret):
```rust
pub fn geometry<B: Brush>(&self, layout: &Layout<B>, width: f32) -> BoundingBox {
    // Returns a rectangle positioned at the visual cursor location
}
```

**comparison with plev's textbuffer:**
plev's `TextBuffer` uses a `cursor: usize` byte index but has no affinity concept. this means at soft line breaks, the cursor position is ambiguous. plev also uses a crude `font_size * 0.6` monospace approximation for cursor-to-pixel mapping, while parley queries the actual layout `Cluster` objects for precise positioning.

**applicability to plev:**
when plev migrates to parley (task-32) or improves text editing, the cursor model should gain affinity. even before migration, the `from_point()` / `geometry()` pair demonstrates the correct API surface for cursor positioning. the current monospace approximation (`cursor_to_x`/`x_to_cursor` in `text_input.rs`) can be replaced by querying shaped glyph advances from cosmic-text's `Buffer::line_layout()`.

**decision: adapt**, adopt byte-index + affinity pattern when improving text editing. can be implemented incrementally even on top of cosmic-text before any parley migration.

---

## pattern 2: selection geometry via callback (parley)

**source:** `parley/parley/src/editing/selection.rs` lines 497-609

**description:**
parley's `Selection` computes visual highlight rectangles through a `geometry_with()` callback method:

```rust
pub fn geometry_with<B: Brush>(
    &self,
    layout: &Layout<B>,
    mut f: impl FnMut(BoundingBox, usize),  // rect + line_index
) {
    // Walk cluster-by-cluster through affected lines
    // Emit one rect per contiguous selected region per line
}
```

this avoids allocation (no vec) when the caller just wants to render. a convenience `geometry()` method exists that collects into a `Vec<(BoundingBox, usize)>`.

the algorithm handles:
- multi-line selections (one rect per line)
- RTL text (discontiguous selection rects within a line for bidi text)
- inline boxes (skipped from selection geometry)
- trailing newline whitespace (visual indicator that the newline is selected)

the `Selection` struct itself is `anchor + focus` (two `Cursor` values) plus an `AnchorBase` enum that tracks whether the selection was initiated at cluster/word/line granularity for extending operations.

**applicability to plev:**
plev's `TextBuffer::selection` is a simple `Option<(usize, usize)>`, no visual geometry API at all. when plev adds proper text selection rendering (currently just stores the range), it should adopt the callback pattern. emitting `BoundingBox` per line-segment maps directly to plev's `SceneNode::Rect`, each selection rect becomes a highlight quad pushed before the text layer.

**decision: adopt**, implement `selection_geometry_with()` callback pattern when adding selection rendering to textinput component.

---

## pattern 3: plaineditordriver, borrowed context wrapper (parley)

**source:** `parley/parley/src/editing/editor.rs` lines 154-166

**description:**
parley separates the editor state (`PlainEditor`) from the operation context (`PlainEditorDriver`). the driver borrows the editor plus the font/layout contexts needed for operations:

```rust
pub struct PlainEditorDriver<'a, T: Brush> {
    pub editor: &'a mut PlainEditor<T>,
    pub font_cx: &'a mut FontContext,
    pub layout_cx: &'a mut LayoutContext<T>,
}
```

operations that modify text (insert, delete, backspace) live on the driver, not the editor. this pattern solves the rust borrow-checker challenge where text mutation needs both the editor and the layout system simultaneously. the editor stores a `layout_dirty` flag and the driver calls `update_layout()` after mutations.

generation tracking (`Generation(u32)`) provides cheap dirty detection, consumers compare generations to know if they need to redraw, avoiding redundant redraws.

**applicability to plev:**
plev's `TextInput` component currently works around borrow issues by cloning or taking ownership. the driver pattern would cleanly separate "what data the editor holds" from "what contexts are needed to perform operations." this is particularly relevant when plev integrates cosmic-text's `FontSystem` (which must be mutably borrowed during shaping) alongside the text buffer. the generation counter would integrate well with plev's fxhash dirty tracking.

**decision: adapt**, adopt the driver/context separation and generation counter when refactoring textinput for proper layout integration. the exact struct shape will differ (plev uses cosmic-text's fontsystem, not parley's fontcontext), but the pattern applies directly.

---

## pattern 4: geometrybuilder trait, decoupled tessellation output (lyon)

**source:** `lyon/crates/tessellation/src/geometry_builder.rs` lines 210-255, 393-433

**description:**
lyon's tessellators produce geometry through a `GeometryBuilder` trait, fully decoupling the tessellation algorithm from the output format:

```rust
pub trait GeometryBuilder {
    fn begin_geometry(&mut self) {}
    fn end_geometry(&mut self) {}
    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId);
    fn abort_geometry(&mut self) {}
}

pub trait FillGeometryBuilder: GeometryBuilder {
    fn add_fill_vertex(&mut self, vertex: FillVertex) -> Result<VertexId, GeometryBuilderError>;
}
```

the tessellator only ever calls `add_fill_vertex()` and `add_triangle()`. a `FillVertexConstructor` trait converts lyon's `FillVertex` (position + metadata) to whatever the user wants:

```rust
pub trait FillVertexConstructor<OutputVertex> {
    fn new_vertex(&mut self, vertex: FillVertex) -> OutputVertex;
}
```

the default `BuffersBuilder` writes into `VertexBuffers<V, I>` (pair of vecs), but custom implementations can write directly to mapped GPU memory, de-interleaved streams, or anything else.

**applicability to plev:**
plev currently generates quads procedurally (4 vertices + 6 indices per rect in compositor.rs). when lyon is integrated (task-31) for vector paths, plev needs to convert lyon's tessellated triangles into its existing `QuadVertex` format (`position: [f32; 2], color: [f32; 4]`). a custom `FillVertexConstructor` implementation is the natural integration point:

```rust
struct plevPathVertex([f32; 4]); // color
impl FillVertexConstructor<QuadVertex> for plevPathVertex {
    fn new_vertex(&mut self, vertex: FillVertex) -> QuadVertex {
        QuadVertex {
            position: vertex.position().to_array(),
            color: self.0,
        }
    }
}
```

the `VertexBuffers` output can feed directly into plev's `GpuVec` with `bytemuck::cast_slice()`. see the lyon wgpu example (`examples/wgpu/src/main.rs`) where this exact pattern is demonstrated with a `#[repr(C)] #[derive(Pod, Zeroable)]` vertex struct.

**decision: adopt**, this is the exact integration path for task-31. implement `FillVertexConstructor<QuadVertex>` and `StrokeVertexConstructor<QuadVertex>` to produce vertices compatible with plev's existing quad pipeline. no new shader needed, tessellated paths render through the same quad pipeline as rects.

---

## pattern 5: lyon + wgpu integration, vertex layout and instancing (lyon)

**source:** `lyon/examples/wgpu/src/main.rs` lines 47-92, 150-183, 320-387

**description:**
the lyon wgpu example demonstrates a complete integration pattern:

1. **vertex format** uses `#[repr(C)] #[derive(Pod, Zeroable)]` for GPU-safe layout:
```rust
struct GpuVertex {
    position: [f32; 2],  // Float32x2 at offset 0
    normal: [f32; 2],    // Float32x2 at offset 8
    prim_id: u32,        // Uint32 at offset 16
}
```

2. **vertex constructor** bridges lyon to the GPU vertex format, adding a primitive id for instancing:
```rust
impl FillVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: FillVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position().to_array(),
            normal: [0.0, 0.0],
            prim_id: self.0,
        }
    }
}
```

3. **one tessellation, multiple draws**: tessellation happens once at startup. different shape ranges (fill, stroke, arrows) are stored as `Range<u32>` into the same index buffer. draw calls use `draw_indexed(range, base_vertex, instances)`.

4. **primitive buffer for instancing**: per-instance data (color, transform, z-index) is stored in a uniform buffer array, indexed by `prim_id` from the vertex.

**applicability to plev:**
this directly maps to plev's architecture. plev already has:
- `QuadVertex { position: [f32; 2], color: [f32; 4] }` with `#[repr(C)] Pod/Zeroable`
- per-layer dirty tracking and persistent `GpuVec` buffers
- the existing quad pipeline

for task-31 integration, plev can:
- add a `SceneNode::Path { path: lyon::Path, color, stroke_opts }` variant
- tessellate in the compositor's `resolve()` phase (not per-frame if path hasn't changed)
- append tessellated vertices/indices to the existing quad vertex/index buffers
- use the same quad shader, tessellated paths are just colored triangles

the example also validates that tessellation can be cached: tessellate once, store index ranges, draw many frames. this fits plev's fxhash dirty tracking model.

**decision: adopt**, the integration architecture from this example is the template for task-31.

---

## pattern 6: glam vec2 with bytemuck pod, zero-cost GPU attributes (glam)

**source:** `glam-rs/src/f32/vec2.rs` lines 20-32, `glam-rs/Cargo.toml` line 58

**description:**
glam's `Vec2` is `#[repr(C)]` with named fields `x, y` and conditionally derives `bytemuck::Pod + Zeroable`:

```rust
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
```

when `bytemuck` feature is enabled, `Vec2` can be directly cast to `&[u8]` for GPU upload. glam also provides SIMD backends transparently:
- **sse2** on x86/x86_64
- **NEON** on aarch64 (arm, apple silicon)
- **simd128** on wasm32

the scalar fallback maintains identical layout/alignment. for `Vec2` specifically, there is no SIMD benefit (it's 8 bytes, below 16-byte SIMD width), but `Vec4` and `Mat4` get significant acceleration. the `bytemuck` dependency in glam uses `features = ["aarch64_simd", "wasm_simd"]` to handle SIMD type casting on those platforms.

**comparison with plev's current approach:**
plev uses raw `[f32; 2]` and `[f32; 4]` everywhere in vertex structs. this works perfectly but loses named field access (`pos[0]` vs `pos.x`) and any future SIMD optimization. the conversion cost is zero, glam `Vec2` has the same memory layout as `[f32; 2]`.

**applicability to plev:**
adopting glam would:
1. replace `position: [f32; 2]` with `position: Vec2` in vertex structs
2. replace `color: [f32; 4]` with `color: Vec4` in vertex structs
3. enable operator overloading for vector math (`a + b` instead of manual component ops)
4. get SIMD-accelerated matrix operations for the orthographic projection in `gpu.rs`
5. lyon already uses `euclid::Point2D` internally but outputs `[f32; 2]` via `.to_array()`

the risk is low: `Vec2` is `#[repr(C)]` with `Pod`, so it's a drop-in replacement in vertex structs. glam has no required dependencies (bytemuck is optional) and the `no_std` build works for WASM.

however, plev does very little vector math currently, it's primarily pushing rectangles with known coordinates. the benefit would grow with lyon integration (path construction, transforms) and if plev adds scene graph transforms. for now, the raw arrays are fine.

**decision: ignore (for now, evaluate for task-31)**, adopt when lyon integration creates enough vector math to justify the dependency. adding glam just for named fields in vertex structs is not worth the dependency in phase 2. re-evaluate when implementing path tessellation.

---

## pattern 7: inlinebox, non-text elements in text flow (parley)

**source:** `parley/parley/src/inline_box.rs` (entire file), `parley/parley/src/layout/line.rs` lines 161-176

**description:**
parley supports embedding non-text elements (images, widgets, custom drawn content) within text flow through `InlineBox`:

```rust
pub struct InlineBox {
    pub id: u64,       // user-specified, for matching output to input
    pub index: usize,  // byte offset in text where box is placed
    pub width: f32,
    pub height: f32,
}
```

the layout engine treats inline boxes as opaque rectangles that participate in line breaking and alignment. after layout, `PositionedInlineBox` provides the computed `(x, y)` coordinates:

```rust
pub struct PositionedInlineBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub id: u64,
}
```

lines iterate over `PositionedLayoutItem` which is either a `GlyphRun` or an `InlineBox`. the selection geometry algorithm correctly handles inline boxes (skips their extent from selection rectangles).

**applicability to plev:**
this pattern is essential for rich text editing, embedding emoji rendered as images, interactive chips/tags, or any non-glyph content inline with text. plev's current `SceneNode::Text` has no concept of inline content. when plev moves toward rich text support, the inline box model provides the right abstraction: the text layout engine determines positioning, the compositor renders whatever the `id` maps to.

cosmic-text (plev's current text engine) does not have inline boxes. this is one of parley's significant advantages and a reason to consider eventual migration.

**decision: adapt**, adopt the inline box concept when plev adds rich text support. the `id: u64` + positioned output pattern integrates well with plev's existing layer/node system. not needed for phase 2 but important for phase 3+.

---

## summary table

| # | pattern | source | decision | phase |
|---|---------|--------|----------|-------|
| 1 | byte-index cursor + affinity | parley cursor.rs | adapt | task-32 or textinput improvement |
| 2 | selection geometry callback | parley selection.rs | adopt | textinput selection rendering |
| 3 | editordriver (borrowed context) | parley editor.rs | adapt | textinput refactor |
| 4 | geometrybuilder trait | lyon geometry_builder.rs | adopt | task-31 (lyon integration) |
| 5 | lyon+wgpu vertex layout + instancing | lyon examples/wgpu | adopt | task-31 (lyon integration) |
| 6 | glam vec2/vec4 with bytemuck pod | glam-rs vec2.rs | ignore (evaluate later) | task-31 re-evaluation |
| 7 | inlinebox for text-embedded content | parley inline_box.rs | adapt | phase 3+ rich text |

## key takeaway for plev architecture

the three repos confirm plev's integration roadmap:

1. **lyon is a drop-in for vector paths.** the `FillVertexConstructor` trait produces vertices that map 1:1 to plev's `QuadVertex`. tessellate once, cache, render through the existing quad pipeline. no new shader needed.

2. **parley's editing model is strictly better than plev's textbuffer.** byte-index + affinity + cluster-aware operations vs. plev's char-boundary-only cursor. the driver pattern solves the same borrow challenges plev faces with cosmic-text's `FontSystem`. migration path is clear but should wait until task-32 assessment confirms parley stability.

3. **glam adds convenience but not value yet.** plev's `[f32; 2/4]` arrays work fine through bytemuck. glam's SIMD benefits (NEON on apple silicon) would only matter for matrix operations (projection, transforms) which plev does minimally. worth adding alongside lyon to avoid `euclid`<->`[f32;N]` conversions.

4. **parley is WASM-compatible** (no target restrictions in cargo.toml, `no_std` with `libm` feature). lyon is `#![no_std]`. glam supports `no_std` + wasm simd128. all three align with plev's 6-platform target.
