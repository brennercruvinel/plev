---
type: reference
tags: [architecture, modules, contracts, data-flow]
date: 2026-06-12
status: living
---

# plev architecture

single human reference. machine map: arc.yaml. visual map: arc.mmd.
update all three together when structure changes.

## layers, bottom up

| layer | path | role |
|---|---|---|
| gpu | crates/engine/src/gpu/ | wgpu device/surface/pipelines; srgb view formats; image atlas; GpuVec buffers (vec); texture_pool; wgsl shaders (crates/engine/src/gpu/shaders) |
| compositor | crates/engine/src/compositor/ | scene nodes, layers with dirty hashes, resolve to gpu buffers |
| text | crates/engine/src/text/ | cosmic-text shaping, gpu glyph atlas, TextMeasurer (the only width source) |
| path | crates/engine/src/path/ | lyon tessellation behind PathBuilder (auto-finishes open subpaths) |
| layout | crates/engine/src/layout/ | taffy wrapper: flex, percent, text measure functions |
| input | crates/engine/src/input/ | pointer state, hit regions, touch tracker + gesture recognizer, touch-to-pointer synth, scroll, dispatch |
| animation | crates/engine/src/animation/ | Tween, FrameClock, easing (dt-based; web_time) |
| perf | crates/engine/src/perf/ | PerfMonitor: rolling windows over AnimationTick + RenderStats (fps, dt p50/p95/p99, cpu micros, memory incl. native rss); PerfHud overlay layer; opt-in via RenderConfig (perf_log, perf_hud); gpu timestamp queries pending (gpu_micros stays None) |
| signal | crates/engine/src/signal/ | reactive primitives (create_signal, runtime) |
| window | crates/engine/src/window/ | App runner: event loop, render-on-demand, render passes, wasm canvas |
| theme | crates/engine/src/theme/ | measured hoff tokens, oklch tooling, intents |
| ui | crates/engine/src/ui/ | retained widgets (~15) + immediate builder + lucide icons |
| builder | crates/engine/src/builder/ | declarative element tree (div/text/button), layout pipeline, emit |
| component/view | crates/engine/src/component/, crates/engine/src/view/ | Lifecycle trait, View trait, ViewContext |
| platform | crates/engine/src/platform/ | safe areas (mod), ime, app lifecycle |

apps consume the engine; they never reimplement engine capabilities (ADR
content-driven-layout-not-fixed-constants and the engine manual record why).

## workspace tiers

three tiers (ADR workspace-engine-at-root-libs-in-crates-demos-in-examples,
updated: the engine moved from the root crate to crates/engine, the root is
now a virtual workspace):

| tier | where | members |
|---|---|---|
| engine | crate `engine` (crates/engine) | the layers above, plus the examples/ demos it ships |
| libraries and apps | crates/ | git, ide, lot, macros, monster, narrate, narrate-macro, parser, prime, rope, showcase, svg |
| demos | crates/engine/examples/ | 16 windowed (counter, editor, charts, snake, scene3d, monster_player...) + 2 clis (lot2monsters, svg2monster) |

crate roles: `engine` the compositing engine itself (every app builds on it);
`monster` binary animation codec (.monster, ADR
binary-animation-format-with-discovered-deltas); `lot` lottie importer that
converts to .monster and never embeds a foreign runtime (ADR
import-foreign-formats-by-conversion-not-embedding); `svg` still-image svg
importer on the same exit door (usvg normalizes, lyon tessellates into one
keyframe, .monster out; gradients approximate to a solid, filters/masks/
clips/text are skipped visibly); `parser` ui transpiler poc; `rope`
text-editing core; `git` git ops; `ide` git client app; `showcase`
design-system gallery; `prime` an emergent particle swarm driven by prime
coherence (port of the codepen demo); `narrate`/`narrate-macro` experimental
dsl; `macros` the #[component] proc-macro. cargo hygiene: workspace.package,
workspace.dependencies (single version source), workspace.lints, tuned
profiles; every crate publish = false.

tooling: crates/parser is a parse-resolve-emit transpiler poc (tree-sitter)
that turns one react component (hoff research card) and one gpui widget
(separator) into plev builder source, mapping colors to hoff theme tokens
and reporting every unrepresentable construct on a droplist with
file:line; golden byte tests plus compile-and-run tests over the emitted
code live in crates/parser/tests.

## frame flow

event -> input state / app state mutation -> invalidate (request_redraw)
-> build scene (per-frame regeneration, no retained diffing) -> compositor
resolve (dirty layers only) -> text resolve (atlas) -> encode layer passes
(quads, sdf, shadows, images, text, backdrop blur, in push order) ->
composite pass to the srgb surface view -> present. unchanged layers cost
zero gpu work.

## binding contracts (violations are defects even if pixels look right)

1. one TextStyle per text run, shared by measurement and drawing
2. srgb decode exactly once entering gpu work; encode exactly once at the
   surface write (surface_render_view; to_linear_array)
3. container geometry derives from available space, never from constants
4. visible state change implies invalidation (render on demand)
5. clip rects are logical; scale by clip_scale for physical scissors

## targets

macos metal (shipping), browser webgpu via trunk/wasm32 (shipping, same
pixel as desktop), android vulkan and ios metal (both run the showcase: an
apk via a GameActivity host, and the ios simulator via a thin objc shell that
calls `showcase_ios_main`; see `android/` and `ios/showcase/`), linux/windows
(untested, wgpu primary backends).

## errors and versioning

pre-1.0: no api stability promised. library code must not panic on user
input paths; tessellation and parsers degrade gracefully (log + empty
output). binary/format versioning follows the anim-format rules
implemented in crates/monster (docs/adr/monster-format-v0.md): explicit version,
frozen golden fixtures, per-section checksums. the full delta op set is
decodable: modify becomes tracks, place/replace/remove become timeline
op lists that act inside their keyframe segment. both encoder modes
exist: mode a lowers an authored timeline (write), mode b discovers
deltas from a sampled frame sequence (discover): slot diffing emits the
structural ops, per-prop runs merge into linear segments, snapshots are
inserted on discontinuity and on the random access cadence. an
encoder-side optimizer (optimize) runs idempotent passes over any
timeline before encoding: static track collapse, rdp keyframe
reduction and collinear segment fusion, tolerances expressed in wire
quantization steps with lossless-on-the-wire defaults. crates/monster
also ships the player: AnimationTick-driven, deterministic f32
timeline, windowed
segment evaluation plus segment-local structural op replay (seek stays
O(1) in frames), play/pause/scrub via signals, lowering its ir scene
to SceneNodes that the app pushes per frame.
