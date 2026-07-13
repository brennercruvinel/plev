---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2022-05-24
domain: paper
---

# plev: a GPU-first compositing engine for cross-platform UI rendering
## arxiv paper outline

### abstract (~150 words)
we present plev, an open-source GPU-first compositing engine written in rust that targets six platforms (macos, ios, linux, android, windows, web) from a single codebase via wgpu. unlike existing approaches that either sacrifice performance for portability (react native, flutter) or sacrifice portability for performance (GPUI), plev achieves both through: (1) a two-pass scene graph with per-layer fxhash dirty tracking that eliminates redundant GPU work, (2) premultiplied-alpha compositing with zero platform branches in the render path, (3) a verbal declarative DSL (plev_narrate!) that compiles to the same scene graph, and (4) feature-gated accessibility via accesskit. benchmarks on apple m4 show scene construction at 159-222m rects/s, dirty tracking overhead of 3.3us/1000 nodes, and lyon vector tessellation at 1.5-3.7us/shape. the engine comprises ~14,000 LOC with 370+ tests.

### 1. introduction
- problem: UI rendering fragmentation across platforms
- existing solutions: web-based (electron), platform-native (swiftui/jetpack), cross-platform frameworks (flutter, react native)
- gap: no GPU-first compositing engine focused on the rendering layer (not widgets)
- contribution: plev fills this gap with a single rust codebase, zero platform branches in rendering, and a novel verbal DSL

### 2. related work
| system | language | GPU | platforms | approach |
|--------|----------|-----|-----------|----------|
| egui | rust | immediate | 5 | immediate mode, no scene graph |
| slint | rust | retained | 5 | widget-level, mcu focus |
| GPUI | rust | metal | 1 | macos-only, zed editor |
| iced | rust | wgpu | 5 | elm architecture |
| xilem | rust | vello | 4 | linebender, research stage |
| flutter | dart | skia/impeller | 4 | widget framework, large runtime |
| plev | rust | wgpu | 6 | scene graph + compositing, not widgets |

### 3. architecture
- 3.1 frame lifecycle (begin_frame -> build_scene -> resolve -> render -> present)
- 3.2 scene graph: scenenode enum (rect, text, path), per-layer dirty tracking via fxhasher
- 3.3 compositor: layer system with z-ordering, offscreen textures, opacity
- 3.4 two pipelines: quad (colored rectangles + tessellated paths) + text (glyph atlas sampling)
- 3.5 text system: cosmic-text shaping, etagere atlas packing, LRU eviction, borrow-split pattern
- 3.6 gpuvec: grow-only persistent GPU buffers (prevent fragmentation)
- 3.7 effects: 13-tap separable gaussian blur, shadow silhouette, texturepool

### 4. cross-platform strategy
- 4.1 wgpu abstraction: metal/vulkan/dx12/webgpu - zero platform branches in shaders/render loop
- 4.2 platform differences confined to: init, font loading, mobile lifecycle, IME
- 4.3 android: cargo-ndk + gameactivity + GPU host mode
- 4.4 ios: aarch64-apple-ios-sim, no with_inner_size, metal surface persistence
- 4.5 WASM: eventloopproxy async GPU init, web_time::instant, trunk build (2.4mb)

### 5. verbal DSL: plev_narrate!
- 5.1 motivation: HTML/JSX ergonomics without xml syntax in rust
- 5.2 grammar: element-first, english-like modifiers, format interpolation
- 5.3 implementation: proc-macro with recursive descent parser, levenshtein suggestions
- 5.4 example: `plev_narrate! { div padded(20) color("blue") { text font_size(24) show("Hello") } }`

### 6. accessibility
- feature-gated accesskit integration
- lazy activation (zero overhead without screen reader)
- per-frame treeupdate from hit regions
- focusgraph: sequential + directional navigation

### 7. vector paths
- lyon tessellation reusing quad pipeline (zero new shaders)
- fillvertexconstructor<quadvertex> bridge
- pre-tessellated shapes with hash-based dirty tracking

### 8. reactive primitives
- thread-local signal system via slotmap
- RAII observer guard for panic safety
- fxindexset for o(1) subscriber management
- peek() for non-tracking reads

### 9. evaluation
- 9.1 benchmark methodology (criterion, m4 mac)
- 9.2 scene construction: 159-222m rects/s
- 9.3 dirty tracking: 3.3us/1000 nodes (static scene = zero GPU after frame 1)
- 9.4 tessellation: 1.5-3.7us/shape
- 9.5 signal throughput: 67ns/cycle
- 9.6 test coverage: 370+ tests across 14 subsystems
- 9.7 LOC: ~14,000 (engine) + ~2,000 (proc-macros)

### 10. limitations and future work
- no widget library (intentional - compositing layer only)
- text shaping depends on cosmic-text (c dependency via harfbuzz) - parley evaluation pending
- WASM visual validation incomplete
- no GPU-side benchmarks (requires headless rendering)
- plugin system deferred (WASM-in-WASM problematic)

### 11. conclusion
plev demonstrates that a single-codebase GPU compositing engine can achieve high performance across six platforms without sacrificing correctness or accessibility. the key insight is that platform differences belong in initialization, not rendering - enabling a render path with zero platform branches.
