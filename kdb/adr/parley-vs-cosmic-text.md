---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: text
---

# parley vs cosmic-text - factual comparison for plev

**date:** 2026-03-11
**context:** plev uses cosmic-text 0.18.2. task-32 assesses whether to migrate to parley.

---

## 1. overview

| | **cosmic-text 0.18.2** | **parley 0.7.0** |
|---|---|---|
| maintainer | system76 (cosmic desktop) | linebender (xilem/vello ecosystem) |
| purpose | multi-line text handling | rich text layout engine |
| stars | ~2000 | ~534 |
| dependents | ~7300 crates | ~402 crates |
| license | MIT/apache-2.0 | MIT/apache-2.0 |
| MSRV | varies per release | rust 1.88 |
| self-described stability | stable enough for cosmic de | "alpha-quality software" |

---

## 2. shaping engine (harfrust vs harfbuzz)

**both libraries now use harfrust.** this is a critical finding that simplifies the comparison.

- cosmic-text 0.18.2 depends on `harfrust` (confirmed in cargo.lock)
- parley 0.7.0 depends on `harfrust` (via workspace)
- harfrust is a rust port of harfbuzz, matching harfbuzz v13.0.0
- harfrust is **not** a complete port: no arabic fallback shaping, no `mort` table, no graphite
- performance: "less than 25% slower than harfbuzz on most common fonts"
- pure rust: no c/c++ build dependency, no `cc` crate, no system `libharfbuzz`

**conclusion:** no dependency advantage for either library on shaping. both are pure rust.

---

## 3. cursor positioning API

### cosmic-text

```rust
// Cursor is a simple struct with public fields
pub struct Cursor {
    pub line: usize,   // BufferLine index
    pub index: usize,  // byte index within line
    pub affinity: Affinity,
}

// Hit-testing is on Buffer
buffer.hit(x: f32, y: f32) -> Option<Cursor>

// Cursor motion
buffer.cursor_motion(font_system, cursor, cursor_x_opt, motion) -> Option<(Cursor, Option<i32>)>

// Getting pixel position
buffer.layout_cursor(font_system, cursor) -> Option<LayoutCursor>
```

cursor is **line-local** (line index + byte offset within that line). navigation via `Motion` enum (left, right, up, down, home, end, etc.) with explicit `cursor_motion()` calls.

### parley

```rust
// Cursor has private fields, constructed via factory methods
Cursor::from_byte_index(layout, index, affinity) -> Self  // global byte index
Cursor::from_point(layout, x, y) -> Self                  // hit-testing built-in

// Rich navigation methods on Cursor itself
cursor.previous_visual(layout) / cursor.next_visual(layout)
cursor.previous_visual_word(layout) / cursor.next_visual_word(layout)
cursor.previous_logical_word(layout) / cursor.next_logical_word(layout)

// Geometry
cursor.geometry(width: f32) -> BoundingBox  // cursor rectangle for rendering
cursor.index() -> usize                     // global byte index
cursor.affinity() -> Affinity

// Cluster access for fine-grained positioning
cursor.logical_clusters() -> [Option<Cluster>; 2]  // upstream, downstream
cursor.visual_clusters() -> [Option<Cluster>; 2]    // left, right
```

cursor uses **global byte index** (not line-local). navigation methods are on the cursor itself. `from_point()` combines hit-testing with cursor creation. `geometry()` returns a renderable rectangle directly.

### verdict

**parley is significantly better.** cosmic-text requires manual coordinate juggling (line + offset), separate hit() call, and separate layout_cursor() to get pixel position. parley unifies everything: hit-test returns a cursor, cursor has its own geometry(), navigation is built-in with visual/logical word-level movement. the cluster access API enables precise bidi-aware positioning.

---

## 4. selection geometry API

### cosmic-text

```rust
// Selection is a mode enum, not a range
pub enum Selection {
    None,
    Normal(Cursor),  // anchor cursor, focus = current cursor
    Line(Cursor),
    Word(Cursor),
}

// Editor computes selection bounds
editor.selection_bounds() -> Option<(Cursor, Cursor)>

// Rendering: Editor.draw() handles selection rendering internally
editor.draw(font_system, cache, text_color, cursor_color, selection_color, selected_text_color, callback)
```

selection geometry is **not exposed** as a separate API. the `Editor.draw()` method handles selection highlight rendering internally through its callback, mixing layout and rendering concerns. if you want custom rendering (like plev's GPU pipeline), you must reverse-engineer selection rectangles from cursor positions.

### parley

```rust
// Selection is a range of two Cursors
Selection::new(anchor: Cursor, focus: Cursor) -> Self

// Rich selection construction
Selection::from_point(layout, x, y) -> Self
Selection::word_from_point(layout, x, y) -> Self
Selection::line_from_point(layout, x, y) -> Self

// Geometry: callback-based (zero-alloc) or collected
selection.geometry_with(layout, |bounding_box, line_index| { ... })  // zero-alloc
selection.geometry(layout) -> Vec<(BoundingBox, usize)>              // collected

// Navigation with extend flag
selection.next_visual(layout, extend: bool) -> Self
selection.previous_visual(layout, extend: bool) -> Self
selection.next_line(layout, extend: bool) -> Self
// ... same for word, line_start, line_end, hard_line_start, hard_line_end

// Direct extension
selection.extend_to_point(layout, x, y) -> Self
selection.shift_click_extension(layout, x, y) -> Self
```

### verdict

**parley is dramatically better.** the `geometry_with` callback provides zero-allocation selection rectangle computation, per-line. this maps directly to plev's `SceneNode::Rect` emission for selection highlights. cosmic-text's approach forces you either to use its opaque `Editor.draw()` or to manually compute rectangles. parley also handles multi-line selection correctly with per-line boundingbox results.

---

## 5. inlinebox support

### cosmic-text

**not supported.** no concept of inline boxes. text is text; non-text elements cannot participate in the text flow.

### parley

```rust
pub struct InlineBox {
    pub id: u64,       // user identifier
    pub index: usize,  // byte offset in text
    pub width: f32,
    pub height: f32,
}

// Layout iteration distinguishes text from inline boxes
for item in line.items() {
    match item {
        PositionedLayoutItem::GlyphRun(glyph_run) => { /* text */ }
        PositionedLayoutItem::InlineBox(inline_box) => { /* embedded element */ }
    }
}
```

### verdict

**parley wins by having it at all.** inlinebox enables: inline images/icons, embedded UI elements within text flow, mention chips, link previews, custom decorations. this is important for any rich text scenario and cannot be retrofitted onto cosmic-text without forking.

---

## 6. WASM compatibility

### cosmic-text

- has `wasm-web` feature flag for locale detection
- uses `fontdb` which supports embedded fonts via `Database::load_font_data(bytes)`
- plev already uses cosmic-text on WASM successfully (include_bytes + fontdb)
- battle-tested in cosmic desktop ecosystem (though cosmic itself doesn't target WASM)

### parley

- **issue #70 is still open** ("building for webassembly")
- `system` feature (default) doesn't compile for wasm32
- workaround: disable `system` feature, load fonts manually via `fontique::Collection::register_fonts()`
- confirmed working by maintainers: "if you want to build for WASM, you have to disable the system feature [...] but it does work"
- fontique uses `Blob<u8>` for font data (vs fontdb's `Vec<u8>`) - slightly different loading pattern
- no official WASM example or CI target

### verdict

**cosmic-text is more mature on WASM.** both work, but cosmic-text's WASM path is well-trodden (plev already uses it). parley's WASM support is confirmed functional but requires manual feature flag management and has no official documentation/examples. the open issue suggests the story isn't finalized.

---

## 7. stability and breaking changes

### cosmic-text

- **version:** 0.18.2 (february 2026)
- **cadence:** minor versions roughly every 2-4 months
- **breaking changes:** moderate. the 0.12->0.18 path had several (shaping enum changes, attrs by reference, added alignment param to set_text, harfbuzz->harfrust transition)
- **adoption:** 7300+ dependents. powers the cosmic desktop environment (system76's production OS)
- **risk:** system76 is invested in cosmic; unlikely to abandon cosmic-text

### parley

- **version:** 0.7.0 (november 2024 - over 4 months ago, no newer release)
- **cadence:** roughly every 2-4 months, with significant API churn each release
- **breaking changes per release:**
  - 0.7.0: module reorganization (`parley::editor` -> `parley::editing`)
  - 0.6.0: renamed alignment variants, replaced kurbo::rect with boundingbox, **swash -> harfrust migration**
  - 0.5.0: redesigned line height handling, builders now consume self
  - 0.4.0: selection geometry methods return line indices, collection API changed
- **self-assessment:** "alpha-quality software"
- **risk:** linebender is google-adjacent (raph levien). xilem/vello ecosystem is ambitious but pre-1.0 across the board. API will keep changing.

### verdict

**cosmic-text is more stable.** parley explicitly labels itself alpha. every minor version brings breaking changes that require migration work. for a GPU engine that wants stability in its text pipeline, this matters. however, parley's API design is clearly superior - the churn is the cost of getting the design right.

---

## 8. performance characteristics

### cosmic-text

- shaping: harfrust (same as parley)
- font discovery: fontdb (mature, widely used)
- caching: shape-run-cache feature flag, user manages buffer reuse
- rasterization: swash (rust glyph rasterizer with hinting)
- memory: buffer owns shaped data, single allocation pattern

### parley

- shaping: harfrust (same as cosmic-text)
- font discovery: fontique (newer, designed for the linebender stack)
- caching: layoutcontext provides scratch space reuse across layouts
- rasterization: **not included.** parley is layout-only. you need skrifa for outlines or an external rasterizer for bitmaps.
- memory: layout is a separate object from context, enabling better arena patterns

### key difference: rasterization

plev currently uses `SwashCache::get_image_uncached()` to produce alpha masks for the glyph atlas. **parley does not include a rasterizer.** migration would require:
- using `skrifa` to get glyph outlines, then rasterizing them yourself (CPU-side)
- or keeping `swash` as a direct dependency alongside parley
- or using a different rasterizer

this is the single biggest migration cost beyond API changes.

### verdict

**roughly equivalent for shaping/layout.** cosmic-text's bundled swash rasterizer is a significant convenience. parley's separation of concerns is cleaner architecturally but creates more integration work for GPU engines that need bitmap atlas patterns (like plev).

---

## 9. accesskit integration

### cosmic-text

no built-in accesskit support. plev would need to build the accessibility bridge manually.

### parley

built-in `accesskit` feature flag with:
- `LayoutAccessibility::build_nodes()` generates accesskit tree nodes
- `Cursor::from_access_position()` converts accesskit positions to parley cursors
- `Selection::from_access_selection()` / `to_access_selection()` for bidirectional conversion

### verdict

**parley wins.** since plev already has an `accessibility` feature with accesskit (task-30), parley's built-in bridge would eliminate manual mapping code.

---

## 10. migration cost assessment

### what currently uses cosmic-text in plev

| file | usage | migration difficulty |
|---|---|---|
| `src/text.rs` | fontsystem, buffer, metrics, attrs, shaping, swashcache, swashcontent, cachekey, cachekeyflags, fontdb::id, layout_runs(), layoutrun, glyph.physical() | **high** - core rendering pipeline |
| `src/text_input.rs` | no direct cosmic-text deps (uses approximate cursor_to_x) | **low** - would benefit from parley's cursor API |
| `Cargo.toml` | `cosmic-text = "0.18"` | trivial |

### specific API mappings required

| cosmic-text | parley equivalent | notes |
|---|---|---|
| `FontSystem::new()` | `FontContext::new()` | 1:1 but fontique vs fontdb underneath |
| `FontSystem::new_with_locale_and_db()` | `FontContext::default()` + `collection.register_fonts()` | different font loading pattern for WASM/ios |
| `Buffer::new(font_system, metrics)` | `LayoutContext::new()` + builder pattern | fundamentally different: parley separates context from layout |
| `buffer.set_text(font_system, text, attrs, shaping, align)` | `layout_cx.ranged_builder(font_cx, text, scale, quantize)` + push styles + `build()` | more verbose but more powerful |
| `buffer.set_size(font_system, width, height)` | `layout.break_all_lines(max_advance)` + `layout.align(...)` | explicit line-breaking step |
| `buffer.shape_until_scroll(font_system, prune)` | implicit in `build()` | parley shapes during build |
| `buffer.layout_runs()` | `layout.lines()` -> `line.items()` -> `GlyphRun` | similar pattern, extra nesting |
| `run.glyphs.iter()` | `glyph_run.glyphs()` | similar iterator |
| `glyph.physical((x, y), scale)` | `glyph.x`, `glyph.y` (already positioned) | parley glyphs have absolute positions |
| `SwashCache::get_image_uncached(font_system, key)` | **no equivalent** - need skrifa or keep swash | biggest gap |
| `cosmic_text::CacheKey` (font_id + glyph_id + size + flags) | `glyph.id` (u32) + font from run | different caching key structure |
| `fontdb::ID` | fontique font identifier | different type |

### estimated LOC changes

- `text.rs`: ~200 lines rewritten (out of 559)
- `text_input.rs`: ~30 lines improved (cursor_to_x/x_to_cursor can be replaced with parley cursor)
- new code: ~50 lines for skrifa-based rasterization or keeping swash as standalone dep
- test updates: ~20 lines
- **total: ~300 LOC, touching the most critical rendering path**

---

## 11. plaineditor comparison

### cosmic-text editor

- wraps a buffer, provides edit operations via `Edit` trait
- `draw()` method renders text + cursor + selection internally
- opaque rendering: you get a callback with (x, y, w, h, color) but can't separate concerns
- `action()` method for all editing operations

### parley plaineditor

- uses `PlainEditorDriver` pattern (borrows fontcontext + layoutcontext)
- `layout()` returns the layout for separate rendering
- clean separation: editor manages state, you render however you want
- designed for integration with custom renderers

### verdict

**parley's plaineditor is better for GPU engines.** cosmic-text's editor wants to own rendering. parley's editor gives you the layout and lets you render it - exactly what plev needs.

---

## 12. summary comparison table

| criterion | cosmic-text 0.18 | parley 0.7 | winner |
|---|---|---|---|
| cursor API | line-local, manual hit-test | global byte index, built-in hit-test + geometry | **parley** |
| selection geometry | opaque (inside editor.draw) | `geometry_with` callback, zero-alloc | **parley** |
| inlinebox | not supported | full support | **parley** |
| shaping engine | harfrust | harfrust | **tie** |
| WASM compatibility | mature, tested in plev | works but needs manual setup, open issue | **cosmic-text** |
| stability | 0.18, production in cosmic de | 0.7, self-described "alpha" | **cosmic-text** |
| breaking change frequency | moderate | high (every release) | **cosmic-text** |
| rasterization | swashcache included | not included (layout only) | **cosmic-text** |
| accesskit integration | none | built-in feature | **parley** |
| editor for GPU engines | opaque draw callback | separated layout + render | **parley** |
| font loading (WASM/ios) | fontdb load_font_data | fontique register_fonts | **tie** |
| ecosystem momentum | system76/cosmic | linebender/xilem/vello | **parley** (long-term) |
| migration cost from current | n/a (current) | ~300 LOC, touches critical path | **cosmic-text** |
| performance | harfrust + swash | harfrust + skrifa (outlines only) | **tie** |

---

## 13. recommendation

### wait. do not migrate now. re-evaluate at parley 1.0 or 0.9.

**rationale:**

1. **parley is alpha.** every minor version breaks the API. migrating now means migrating again in 3-4 months when 0.8 ships. and again at 0.9. the API churn cost compounds.

2. **the rasterization gap is real.** plev's text pipeline depends on swashcache for bitmap glyph rasterization into the atlas. parley doesn't provide this. adding a skrifa-based rasterizer or keeping swash alongside parley adds complexity with no clear benefit today.

3. **plev's text_input.rs doesn't use cosmic-text's cursor API anyway.** the current `cursor_to_x`/`x_to_cursor` functions are approximate and font-system-independent. the benefit of parley's cursor API only materializes when plev needs true font-aware cursor positioning (bidi text, variable-width fonts, multi-line editing). that's a future need, not a current one.

4. **WASM is a first-class target for plev.** cosmic-text's WASM path is proven. parley's WASM story has an open issue and no official examples.

5. **the shaping engine is the same.** both use harfrust. there's no performance or compatibility benefit on the shaping side.

### what to do instead

- **now:** keep cosmic-text 0.18. improve `cursor_to_x`/`x_to_cursor` using cosmic-text's `Buffer::hit()` and `Buffer::layout_cursor()` for font-aware positioning. this captures 80% of parley's cursor benefit with zero migration risk.

- **track:** monitor parley releases. key milestones to watch:
  - WASM documentation and official examples
  - API stabilization (fewer breaking changes per release)
  - 0.8 or 0.9 release (last pre-1.0 churn)
  - skrifa rasterization capabilities (currently outline-only)

- **prepare:** structure `text.rs` to isolate cosmic-text types behind a thin abstraction layer. when migration time comes, only the abstraction internals change.

- **migrate when:** parley reaches 0.9+, WASM issue #70 is closed, and skrifa (or a parley-ecosystem rasterizer) can produce bitmap masks for atlas rendering. or when plev needs inlinebox / rich text / accesskit text bridge - those features don't exist in cosmic-text and can't be added without forking.

### decision matrix trigger

| if plev needs... | action |
|---|---|
| better cursor positioning (bidi, multi-line) | use cosmic-text buffer::hit() first |
| inlinebox / embedded elements in text | migrate to parley (no alternative) |
| accesskit text bridge | migrate to parley (saves ~200 LOC of manual bridging) |
| rich text (mixed styles in one paragraph) | both support it; cosmic-text via set_rich_text, parley via rangedbuilder |
| production stability on WASM | stay with cosmic-text |
