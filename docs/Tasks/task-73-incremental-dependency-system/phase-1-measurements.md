---
id: task-73/phase-1-measurements
title: "task-73 phase 1: what each stage of a frame costs"
project: plev
type: section
parent: task-73
status: reference
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [performance, rendering, gpu]
relations: [task-73]
evidence: [4698224, 9522ee5]
source: docs/log/steps/task-73-measurements.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-73 phase 1: what each stage of a frame costs

Records the measurement of phase 1 of task-73: what each stage of a frame costs in nine scenarios (typing, click, hover, modal, frames without interaction), measured without changing behavior, and phase 1b, which repeats the scenarios with GPU timestamp queries. The result is that, on this M4, the frame is limited by the GPU and not by the CPU: the GPU wait is between 450 and 2200 µs per frame, 4 to 15 times the CPU time of the same frame. That is why the orchestrator's decision took phase 2 of task-73 as far as the GPU, and not only to the CPU graph.

phase 1 of task-73 measures, without changing any behavior, the work each
stage of a frame does for a small set of real interactions, so the
incremental dependency system of phase 2 starts from numbers. nothing here
is an optimization or a design decision. the tables at the end are one full
run; the sections before them say how the numbers were taken, how far to
trust them and what they show.

## how to reproduce

- machine: apple m4 (4 performance and 6 efficiency cores), 16 GB, macOS
  26.6.2, metal through wgpu 28, rustc 1.98.1.
- build: the workspace release profile (thin lto, one codegen unit).
- the tables: `cargo run --release -p measure -- --out
  <file.md>` (every scenario, 120 turns each after 10 warm-up turns;
  `--frames`, `--scenario <key>`, `--theme`, `--width`, `--height`,
  `--scale` and `--no-gpu-sync` change the run, `--help` lists them).
- the cost of one probe: `cargo run --release -p measure
  -- --probe-cost`.
- the paths the probes sit on, before and after:
  `cargo bench -p engine --bench scene_build -- probed_paths`; the group
  uses only APIs older than the probes, so the same file runs on 4698224.

the runs shared the machine with other agents (load average 9 to 12, a
file sync client at 70 to 270 percent cpu). three full runs moved up to
about 30 percent from each other; the shape of every table stayed the same.
the tables below are the second run; the summary gives the range of the
three.

## what is measured

`engine::perf::stages` keeps, per thread and per frame, three numbers per
stage: calls, items and time. a probe is one call at the site it measures:
`span(stage)` times the scope it lives in, `count(stage, n)` adds without a
clock. the loop owner calls `take()` once per frame, which is the frame
boundary. stages nest, so a parent's time includes its children.

| stage | where | items | inside |
| --- | --- | --- | --- |
| frame | the runner's loop turn | - | - |
| input | the runner, per event delivered to the view | events | frame |
| semantic projection | `TodoPage::snapshot` (build and finish) | - | input |
| tick | the runner, the view's animation tick | - | frame |
| scene build | the runner, `view.render` | nodes in every layer | frame |
| layout (LayoutEngine) | `LayoutEngine::compute` | layout items | where it is called |
| text measure, cache hit / miss | `TextMeasurer::measure_styled` | - | where it is called |
| text shaping (measurer) | every shaping the measurer runs: misses, hit tests, caret queries | bytes | where it is called |
| semantic bind | `Compositor::bind_to_layer`, first bind of an id | - | scene build |
| hit test | `Compositor::hit` | bindings scanned | where it is called |
| draw (frame tail) | `window::draw_frame`, up to submit | - | frame |
| compositor resolve | `Compositor::resolve` | - | draw |
| layer hash | `Layer::compute_hash` | nodes | resolve |
| geometry rebuild | a dirty layer's `build_geometry` | the layer's nodes | resolve |
| text resolve | text of the dirty layers, in `draw_frame` | glyphs in dirty layers | draw |
| gpu buffer upload | `GpuVec::upload` | bytes | resolve, text resolve |
| encode and submit | layer passes, composite, submit | draw calls | draw |
| gpu wait | after submit, only with the gpu sync on | - | frame |

the runner (`tool/bench/measure/src/`) drives the gallery
headless through `engine::headless::Renderer`, the window's frame path, with
the input the window delivers (`showcase::app` routing: pointer events,
characters, named keys, Escape). a frame is drawn only when an event
handler asked for one or the last tick was still moving, as in the window's
loop; the tick is a fixed 1/60 s.

the gpu wait spins on `device.poll(PollType::Poll)` until the queue is
empty. wgpu's blocking wait on metal checks the command buffer and sleeps a
millisecond between checks, which made every reading 1.5, 3.0 or 4.5 ms in
a first run; the spin reads with microsecond resolution. it is still the
time from submit to seeing the work done, so it holds the gpu's latency as
well as its work: an upper bound of the gpu time, not a timestamp query.

## the probes cost nothing when off

- one probe with stages off (`--probe-cost`, best of 7 rounds of ten
  million): 0.25 to 0.28 ns per loop iteration against 0.46 to 0.48 ns for
  the empty loop, below what the loop can resolve. with stages on, a span
  costs about 35 ns (two clock reads) and a count 0.5 to 3 ns.
- the busiest turn measured makes 90 probe calls (editor page); off, that
  is under 50 ns per frame against 60 to 400 us of cpu per frame.
- the probed paths on criterion (`probed_paths`, stages off), base and head
  run alternately in the same target dir: the first pair moved between -8
  and +7 percent per benchmark while the probe-free control
  (`dirty_tracking`) moved +8 percent; the second pair ran under load and
  moved by -68 to +121 percent in both directions, control included. no
  difference above the noise.
- same pixels: `showcase --render` at 4698224 and at the head of this
  branch give the same sha256 for Todo (core 1x, hoff 2x), Card (core 1x,
  hoff 2x), Editor (hoff 2x), Dialog (core 1x), Typography (hoff 2x) and
  Scroll Area (core 1x). the engine test
  `stages_on_draw_the_same_pixels_as_stages_off` draws one scene with
  stages and the gpu sync on and off and compares the bytes.

## summary

1200x800 logical at scale 2 (2400x1600 physical, the window's launch size
on this display), theme core, times in microseconds, p50 over 120 turns.
ranges are over the three runs.

| scenario | frames drawn | frame | cpu (frame minus gpu wait) | gpu wait | gpu wait above the empty frame |
| --- | --- | --- | --- | --- | --- |
| a. type in the todo input | 120 of 120 | 1709 to 2160 | 191 to 395 | 1518 to 1765 | about 915 |
| b. toggle a todo item | 120 of 120 | 1522 to 2514 | 176 to 324 | 1346 to 2191 | about 745 |
| c. hover a list | 16 of 120 | 0.5 to 0.7 (a turn that draws nothing) | 145 on a drawn turn | 1390 on a drawn turn | about 790 |
| d. modal mid-opening | 120 of 120 | 830 to 986 | 63 to 94 | 768 to 893 | about 165 |
| e. type in a 100k-line document | 120 of 120 | 1663 to 1831 | 307 to 320 | 1343 to 1524 | about 920 |
| e'. type in the Editor page | 120 of 120 | 1777 to 2285 | 263 to 318 | 1497 to 1967 | about 1210 |
| f. idle redraw | 120 of 120 | 905 to 1293 | 61 to 91 | 844 to 1202 | about 465 |
| g. focused field, no typing | 120 of 120 | 908 to 1180 | 62 to 94 | 846 to 1104 | about 500 |
| h. empty frame | 120 of 120 | 481 to 749 | 32 to 85 | 449 to 664 | 0 |

"cpu" is the frame's p50 minus the gpu wait's p50 of the same run, the
input, tick, scene build and frame tail together. "gpu wait above the empty
frame" is the second run's gpu wait minus the empty frame's in the same
run: what the scenario's content adds on the gpu. the hover row is the
second run's 16 drawn turns.

## what the numbers show

these are readings for phase 2, not decisions.

1. the gpu side is larger than the cpu side in every scenario. the cpu of
   a frame is 30 to 400 us; the gpu wait is 450 to 2200 us, and 165 to
   1200 us of it is above the empty frame (the second run). the gpu work
   follows the pixels touched, not the size of the change: a keystroke
   re-renders its whole layer (full surface, msaa 4x) and every frame
   composites every visible layer, the empty ones included (4 composite
   draws on the gallery pages, 2 of those layers hold no node). at scale 1
   the idle gpu wait drops from 0.8 to 1.3 ms to about 0.45 to 0.7 ms and
   the empty frame from 0.45 to 0.65 ms to about 0.26 ms, so pixel count
   drives it.
2. one character rebuilds a whole layer. typing in the todo input rebuilds
   the default layer (35 nodes), re-emits its 243 glyphs and uploads 45 KB;
   in the editor one character re-emits the 1551 glyphs of the visible
   lines and uploads 236 KB. the document size does not show: in a
   100,000-line document the key handler takes 1.6 us and the scene holds
   118 nodes, the lines on screen.
3. shaping is per run, and the caret shapes too. `TextField` places its
   caret with `TextMeasurer::cursor_x_styled`, which the measurer never
   caches, so every frame the caret shows reshapes the whole field text
   (70 bytes at p50, 22 to 37 us across the runs); the editor does the
   same for the caret's line (116 bytes, 27 to 30 us). the text resolve
   then shapes the changed run again on the render side, whose shaping
   cache is keyed by the whole run (inside the 66 to 111 us p50 of text
   resolve; not split out, see limits).
4. semantic projection runs only where a person acts on a capability. a
   toggle projects twice through `TodoPage::snapshot` (9 to 14 us p50 for
   the two, three items) and once more inside the dispatcher (unprobed); the
   whole click is 30 to 49 us of input, most of it that path. typing,
   hover, the caret and the modal project nothing.
5. the engine's layout and hit test are not on these paths: 0 calls to
   `LayoutEngine::compute` and to `Compositor::hit` in every scenario. the
   pages lay out by hand from tokens inside input and scene build, with 35
   to 71 text measures per frame, all cache hits.
6. render on demand holds where nothing is asked: left alone the idle todo
   page drew 0 frames in 120 turns, and a hover drew only on the 16 turns
   its row changed (0.4 to 0.6 us for the others). a focused todo field
   breaks it: the page asks for a frame on every tick while focused, so
   every turn draws with nothing rebuilt (scene 12 to 15 us, tail 42 to 61
   us, gpu about 1 ms), 60 times a second for a caret that changes twice.
7. the modal's animation is a layer opacity: mid-opening nothing is
   rebuilt and nothing uploaded, yet the whole scene is rebuilt on the cpu
   (81 nodes, 12 to 15 us) and all four layers composited.
8. a redraw with nothing changed costs about 65 us of cpu: scene build 14
   us (79 nodes, 35 measure hits, 8 binds), resolve 28 us of which the
   layer hashes are under 1 us, text resolve 3 us, encode 20 us. the
   resolve also writes each layer's opacity uniform every frame
   (`Layer::ensure_texture`), which the upload stage does not count.

## stop and report: the frame is not bound by the cpu here

- problema: the brief asks to stop and report if the measurement shows the
  bottleneck is not cpu. on this machine the gpu wait (450 to 2200 us per
  frame) is 4 to 15 times the cpu of the same frame (30 to 400 us); the
  part above an empty frame (165 to 1200 us) is still 2.5 to 7 times it.
  the gpu work scales with pixels re-rendered and layers composited, not
  with the size of the change. the cpu stages phase 2 was framed around
  are small in every scenario at p50: projection 9 to 14 us per click,
  layout 0, scene build 12 to 60 us.
- opções possíveis: (a) keep phase 2 on the cpu dependency graph as
  planned; (b) extend phase 2 to carry the invalidation down to the gpu:
  damaged regions per layer instead of whole-layer redraws, no composite
  for an empty or unchanged layer, retained text vertices per run or line;
  (c) first measure the gpu with timestamp queries (out of scope of phase
  1, the perf monitor already names it) to split latency from work, then
  decide between (a) and (b).
- impacto: (a) optimizes the smaller half: at most the 30 to 400 us of cpu
  per frame, while a keystroke still redraws a full msaa layer; (b) touches
  the compositor and the frame tail, a larger surface than the files the
  task names for phases 2 and 3; (c) costs a small task (a query set
  threaded through the layer and composite passes) and makes the gpu
  numbers exact.
- recomendação do agente: (c) then (b) folded into the phase 2 adr: the
  dependency graph keeps its keys (state, semantics, hover, animation,
  theme, viewport) and gains the gpu end of the chain (which layer regions
  and which composites depend on which keys). the cpu graph still pays off
  where it grows with content (projection per item, text resolve per
  glyph of a dirty layer), but it is not what bounds the frame today.
- decisão necessária: whether phase 2's scope includes gpu-side
  invalidation, and whether gpu timestamp queries come before the adr.

## limits of this measurement

- the gpu wait is an upper bound: submit latency is in it (the empty frame
  alone waits 0.45 to 0.66 ms). no timestamp queries.
- the loop runs turns back to back, with no vsync pacing; a window at 60
  Hz idles between frames and its gpu may clock lower.
- the render-side shaping cache (`TextSystem::resolve_for_layer`,
  `text/system.rs`) has no probe: the file is outside this task's list.
  text resolve times shaping, rasterization, emission and the text upload
  together. one `count` at its miss branch would split it.
- the upload stage counts `GpuVec` writes (vertices, indices, text). the
  glyph and image atlas texture writes and the per-layer opacity uniform
  are not counted.
- the dispatcher's own projection inside `sem` is not probed; a toggle
  projects three times, two of them measured.
- "resolução de estilo (tokens lidos)" from the task file is not measured:
  tokens are field reads inside comps widgets, which this task may not
  touch. hit tests done by comps widgets (`Rect::contains`) are invisible
  to the engine.
- e runs `comps::editor::EditorView` on its own (no gallery chrome) with a
  generated 100,000-line document, since the Editor page opens a fixed
  200-line sample; e' is that page inside the chrome.
- a toggle or a key every frame is a steady-state load, not a human
  cadence; what a first keystroke costs (new glyphs rasterized, buffers
  grown) went into the warm-up turns.
- the machine was shared during the runs; see the ranges above.

## candidates for phase 2 (not implemented)

- composite only layers with content, and skip a layer whose texture and
  opacity did not change (every gallery frame composites 4 full-screen
  layers, 2 of them empty).
- redraw the damaged region of a dirty layer instead of the whole msaa
  texture; retain text vertices per run or per line so one changed line
  re-emits and re-uploads one line (today 1551 glyphs and 236 KB per key in
  the editor).
- one shaping per changed run: the caret query and the render side shape
  the same text; the measurer's hit tests and caret queries are never
  cached.
- the todo field's blink asks for every frame while focused; asking for a
  frame at the next blink would leave the other 58 frames of a second
  undrawn.
- an opacity-only animation needs no scene rebuild.
- `TextMeasurer::measure_styled` builds an owned key (a `String` copy)
  for every lookup, hits included (35 to 71 per frame here).
- the resolve writes every layer's opacity uniform each frame, dirty or
  not.

## the tables

one probe (the `--probe-cost` run used above):

| loop body | stages off (ns) | stages on (ns) |
| --- | ---: | ---: |
| nothing (the floor) | 0.46 | 0.46 |
| `span(stage).items(n)` and its drop | 0.25 | 35.86 |
| `count(stage, n)` | 0.25 | 0.54 |

the second full run, as `measure --out` wrote it:

120 measured turns per scenario after 10 warm-up turns, fixed tick of 16.67 ms, viewport 1200x800 logical at scale 2 (2400x1600 physical), theme core, release build, stages on with the gpu wait. time is microseconds; p50 and p95 are nearest-rank over the turns.

| scenario | frames drawn | frame p50 | frame p95 | input p50 | scene build p50 | draw p50 | gpu wait p50 | layers rebuilt p50 | nodes rebuilt p50 | upload bytes p50 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| type | 120 of 120 | 1708.9 | 2732.3 | 0.12 | 36.0 | 137.4 | 1518.2 | 1 | 35 | 45504 |
| toggle | 120 of 120 | 1521.6 | 1817.9 | 29.8 | 12.3 | 118.5 | 1346.1 | 1 | 34 | 40080 |
| hover | 16 of 120 | 0.46 | 1556.9 | 0.42 | 0 | 0 | 0 | 0 | 0 | 0 |
| overlay | 120 of 120 | 830.4 | 1061.2 | 0 | 12.5 | 45.5 | 767.8 | 0 | 0 | 0 |
| editor | 120 of 120 | 1830.6 | 3440.7 | 1.58 | 50.8 | 233.3 | 1524.0 | 1 | 118 | 236232 |
| editor-page | 120 of 120 | 2076.0 | 3265.0 | 1.67 | 54.2 | 232.8 | 1812.8 | 1 | 91 | 166312 |
| idle | 120 of 120 | 1134.1 | 1882.8 | 0 | 14.4 | 51.1 | 1065.5 | 0 | 0 | 0 |
| caret | 120 of 120 | 1180.3 | 2755.5 | 0 | 14.5 | 54.4 | 1104.2 | 0 | 0 | 0 |
| floor | 120 of 120 | 658.2 | 885.7 | 0 | 0.04 | 42.7 | 602.8 | 0 | 0 | 0 |

#### a. type one character in the todo input

Todo page; a click on the center of `todo.input` focuses it. each turn is one key of "plev measures what one keystroke costs " (a space is the named key), so the field grows by one character per frame.

frames drawn: 120 of 120. the last turn's scene: 4 layers (z 0 35 nodes, z 10 0 nodes, z 1000 46 nodes, z 1200 0 nodes), 8 ids bound. probe calls per turn: p50 64, p95 64.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 1708.9 | 2732.3 |
| input | events | 1 | 1 | 1 | 1 | 0.12 | 0.38 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 1 | 1 | - | - | 0.17 | 0.33 |
| scene build | nodes | 1 | 1 | 81 | 81 | 36.0 | 50.8 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 35 | 35 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 1 | 1 | 70 | 124 | 22.5 | 36.7 |
| semantic bind | - | 8 | 8 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 137.4 | 227.6 |
| compositor resolve | - | 1 | 1 | - | - | 33.8 | 46.1 |
| layer hash | nodes | 4 | 4 | 81 | 81 | 0.62 | 0.92 |
| geometry rebuild | nodes | 1 | 1 | 35 | 35 | 1.38 | 2.21 |
| text resolve | glyphs | 1 | 1 | 243 | 288 | 65.9 | 135.9 |
| gpu buffer upload | bytes | 6 | 6 | 45504 | 52344 | 23.7 | 32.4 |
| encode and submit | draw calls | 1 | 1 | 26 | 26 | 37.7 | 56.0 |
| gpu wait | - | 1 | 1 | - | - | 1518.2 | 2558.5 |

#### b. toggle one todo item

Todo page with its three seed items; each turn is a click (press and release) on the center of the first bound `todo.item[..]` row, which lies on its checkbox: the item toggles through the dispatcher every frame.

frames drawn: 120 of 120. the last turn's scene: 4 layers (z 0 34 nodes, z 10 0 nodes, z 1000 46 nodes, z 1200 0 nodes), 8 ids bound. probe calls per turn: p50 75, p95 75.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 1521.6 | 1817.9 |
| input | events | 2 | 2 | 2 | 2 | 29.8 | 33.9 |
| semantic projection | - | 2 | 2 | - | - | 9.29 | 10.8 |
| tick | - | 1 | 1 | - | - | 0.17 | 0.29 |
| scene build | nodes | 1 | 1 | 80 | 81 | 12.3 | 15.9 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 44 | 44 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic bind | - | 8 | 8 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 118.5 | 156.7 |
| compositor resolve | - | 1 | 1 | - | - | 36.1 | 47.6 |
| layer hash | nodes | 4 | 4 | 80 | 81 | 0.62 | 0.88 |
| geometry rebuild | nodes | 1 | 1 | 34 | 35 | 1.38 | 2.25 |
| text resolve | glyphs | 1 | 1 | 210 | 210 | 43.2 | 55.6 |
| gpu buffer upload | bytes | 6 | 6 | 40080 | 40560 | 23.7 | 29.5 |
| encode and submit | draw calls | 1 | 1 | 25 | 26 | 38.8 | 54.8 |
| gpu wait | - | 1 | 1 | - | - | 1346.1 | 1566.3 |

#### c. hover across the rows of a list

Scroll Area page (the 10,000-row virtualized list, its rows on their own layer); each turn moves the pointer 5 px down the middle of the list and back, no button pressed.

frames drawn: 16 of 120. the last turn's scene: 4 layers (z 0 6 nodes, z 10 61 nodes, z 1000 46 nodes, z 1200 0 nodes), 0 ids bound. probe calls per turn: p50 2, p95 65.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 0.46 | 1556.9 |
| input | events | 1 | 1 | 1 | 1 | 0.42 | 0.54 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 0 | 1 | - | - | 0 | 0.17 |
| scene build | nodes | 0 | 1 | 0 | 113 | 0 | 17.0 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 0 | 47 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic bind | - | 0 | 0 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 0 | 1 | - | - | 0 | 127.4 |
| compositor resolve | - | 0 | 1 | - | - | 0 | 40.0 |
| layer hash | nodes | 0 | 4 | 0 | 113 | 0 | 0.75 |
| geometry rebuild | nodes | 0 | 1 | 0 | 61 | 0 | 4.29 |
| text resolve | glyphs | 0 | 1 | 0 | 210 | 0 | 45.1 |
| gpu buffer upload | bytes | 0 | 4 | 0 | 72912 | 0 | 26.9 |
| encode and submit | draw calls | 0 | 1 | 0 | 44 | 0 | 43.2 |
| gpu wait | - | 0 | 1 | - | - | 0 | 1413.5 |

the 16 drawn turns only:

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 1535.9 | 2909.0 |
| input | events | 1 | 1 | 1 | 1 | 0.42 | 0.46 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 1 | 1 | - | - | 0.12 | 0.25 |
| scene build | nodes | 1 | 1 | 113 | 113 | 16.9 | 18.9 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 47 | 47 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic bind | - | 0 | 0 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 126.9 | 135.5 |
| compositor resolve | - | 1 | 1 | - | - | 39.5 | 42.8 |
| layer hash | nodes | 4 | 4 | 113 | 113 | 0.75 | 0.83 |
| geometry rebuild | nodes | 1 | 1 | 61 | 61 | 4.29 | 4.62 |
| text resolve | glyphs | 1 | 1 | 210 | 210 | 43.6 | 50.0 |
| gpu buffer upload | bytes | 4 | 4 | 72912 | 72912 | 26.7 | 33.5 |
| encode and submit | draw calls | 1 | 1 | 44 | 44 | 41.9 | 46.8 |
| gpu wait | - | 1 | 1 | - | - | 1390.4 | 2759.6 |

#### d. one frame in the middle of a modal opening

Dialog page; a click on the 'Open dialog' button opens the modal, whose spring takes 54 turns to rest. each sample is turn 27 after the click, with no input; between samples the modal settles, Escape closes it and the page settles again.

frames drawn: 120 of 120. the last turn's scene: 4 layers (z 0 13 nodes, z 10 0 nodes, z 1000 68 nodes, z 1200 0 nodes), 0 ids bound. probe calls per turn: p50 50, p95 50.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 830.4 | 1061.2 |
| input | events | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 1 | 1 | - | - | 0.17 | 0.25 |
| scene build | nodes | 1 | 1 | 81 | 81 | 12.5 | 15.1 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 38 | 38 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic bind | - | 0 | 0 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 45.5 | 70.8 |
| compositor resolve | - | 1 | 1 | - | - | 24.8 | 37.2 |
| layer hash | nodes | 4 | 4 | 81 | 81 | 0.67 | 0.96 |
| geometry rebuild | nodes | 0 | 0 | 0 | 0 | 0 | 0 |
| text resolve | glyphs | 1 | 1 | 0 | 0 | 2.33 | 2.83 |
| gpu buffer upload | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| encode and submit | draw calls | 1 | 1 | 4 | 4 | 18.7 | 27.4 |
| gpu wait | - | 1 | 1 | - | - | 767.8 | 983.4 |

#### e. insert one character in a large document

comps::editor::EditorView, the widget the Editor page embeds, filling the viewport on its own (no gallery chrome) with a 100000-line document (2.8 MB); the wheel scrolls to the middle and a click puts the caret near line 49972. each turn is one key of "plev measures what one keystroke costs ".

frames drawn: 120 of 120. the last turn's scene: 1 layers (z 0 118 nodes), 0 ids bound. probe calls per turn: p50 75, p95 75.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 1830.6 | 3440.7 |
| input | events | 1 | 1 | 1 | 1 | 1.58 | 5.38 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 1 | 1 | - | - | 0.04 | 0.04 |
| scene build | nodes | 1 | 1 | 118 | 118 | 50.8 | 94.7 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 59 | 59 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 1 | 1 | 116 | 170 | 27.2 | 50.6 |
| semantic bind | - | 0 | 0 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 233.3 | 420.8 |
| compositor resolve | - | 1 | 1 | - | - | 30.6 | 50.5 |
| layer hash | nodes | 1 | 1 | 118 | 118 | 1.08 | 1.21 |
| geometry rebuild | nodes | 1 | 1 | 118 | 118 | 4.25 | 5.88 |
| text resolve | glyphs | 1 | 1 | 1551 | 1596 | 171.6 | 258.4 |
| gpu buffer upload | bytes | 4 | 4 | 236232 | 243072 | 51.5 | 77.7 |
| encode and submit | draw calls | 1 | 1 | 4 | 4 | 31.4 | 72.3 |
| gpu wait | - | 1 | 1 | - | - | 1524.0 | 3183.0 |

#### e'. insert one character in the showcase Editor page

the gallery's Editor page with its own sample (about 200 lines), inside the chrome; each turn is one key of "plev measures what one keystroke costs " at the caret (the start of the document).

frames drawn: 120 of 120. the last turn's scene: 4 layers (z 0 91 nodes, z 10 0 nodes, z 1000 46 nodes, z 1200 0 nodes), 0 ids bound. probe calls per turn: p50 90, p95 90.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 2076.0 | 3265.0 |
| input | events | 1 | 1 | 1 | 1 | 1.67 | 3.25 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 1 | 1 | - | - | 0.21 | 0.46 |
| scene build | nodes | 1 | 1 | 137 | 137 | 54.2 | 76.4 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 71 | 71 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 1 | 1 | 84 | 138 | 22.5 | 37.0 |
| semantic bind | - | 0 | 0 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 232.8 | 296.7 |
| compositor resolve | - | 1 | 1 | - | - | 41.2 | 57.0 |
| layer hash | nodes | 4 | 4 | 137 | 137 | 1.38 | 1.67 |
| geometry rebuild | nodes | 1 | 1 | 91 | 91 | 4.08 | 5.67 |
| text resolve | glyphs | 1 | 1 | 1091 | 1136 | 150.0 | 200.1 |
| gpu buffer upload | bytes | 4 | 4 | 166312 | 173152 | 46.3 | 62.7 |
| encode and submit | draw calls | 1 | 1 | 8 | 8 | 39.3 | 60.2 |
| gpu wait | - | 1 | 1 | - | - | 1812.8 | 2926.0 |

#### f. an idle redraw with nothing changed

Todo page at rest, nothing focused; each turn the platform asks for a frame with nothing changed (an expose), so the whole frame runs on an unchanged scene.

frames drawn: 120 of 120. the last turn's scene: 4 layers (z 0 33 nodes, z 10 0 nodes, z 1000 46 nodes, z 1200 0 nodes), 8 ids bound. probe calls per turn: p50 55, p95 55.

left alone for 120 turns, the loop drew 0 frames: with nothing asked, render on demand runs no stage at all.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 1134.1 | 1882.8 |
| input | events | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 1 | 1 | - | - | 0.21 | 0.38 |
| scene build | nodes | 1 | 1 | 79 | 79 | 14.4 | 18.6 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 35 | 35 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic bind | - | 8 | 8 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 51.1 | 67.5 |
| compositor resolve | - | 1 | 1 | - | - | 27.7 | 36.7 |
| layer hash | nodes | 4 | 4 | 79 | 79 | 0.71 | 0.83 |
| geometry rebuild | nodes | 0 | 0 | 0 | 0 | 0 | 0 |
| text resolve | glyphs | 1 | 1 | 0 | 0 | 2.71 | 3.29 |
| gpu buffer upload | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| encode and submit | draw calls | 1 | 1 | 4 | 4 | 20.4 | 30.8 |
| gpu wait | - | 1 | 1 | - | - | 1065.5 | 1818.8 |

#### g. a focused todo input, no typing

Todo page with `todo.input` focused by a click; no input afterwards. the todo page asks for frames while its field is focused (the caret blinks), so every turn draws.

frames drawn: 120 of 120. the last turn's scene: 4 layers (z 0 35 nodes, z 10 0 nodes, z 1000 46 nodes, z 1200 0 nodes), 8 ids bound. probe calls per turn: p50 55, p95 55.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 1180.3 | 2755.5 |
| input | events | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 1 | 1 | - | - | 0.21 | 0.29 |
| scene build | nodes | 1 | 1 | 80 | 81 | 14.5 | 18.2 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 35 | 35 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic bind | - | 8 | 8 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 54.4 | 79.4 |
| compositor resolve | - | 1 | 1 | - | - | 29.2 | 41.0 |
| layer hash | nodes | 4 | 4 | 80 | 81 | 0.71 | 0.92 |
| geometry rebuild | nodes | 0 | 0 | 0 | 0 | 0 | 0 |
| text resolve | glyphs | 1 | 1 | 0 | 0 | 2.67 | 4.00 |
| gpu buffer upload | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| encode and submit | draw calls | 1 | 1 | 4 | 4 | 22.9 | 32.9 |
| gpu wait | - | 1 | 1 | - | - | 1104.2 | 2542.1 |

#### h. the floor: an empty scene redrawn

no page: a compositor with only its empty default layer, redrawn every turn. what is left is the frame tail's fixed cost, the clear and the composite of one layer, and on the gpu the latency of a submit.

frames drawn: 120 of 120. the last turn's scene: 1 layers (z 0 0 nodes), 0 ids bound. probe calls per turn: p50 9, p95 9.

| stage | items are | calls p50 | calls p95 | items p50 | items p95 | time p50 | time p95 |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame | - | 1 | 1 | - | - | 658.2 | 885.7 |
| input | events | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic projection | - | 0 | 0 | - | - | 0 | 0 |
| tick | - | 1 | 1 | - | - | 0.04 | 0.04 |
| scene build | nodes | 1 | 1 | 0 | 0 | 0.04 | 0.08 |
| layout (LayoutEngine) | layout items | 0 | 0 | 0 | 0 | 0 | 0 |
| text measure, cache hit | - | 0 | 0 | - | - | 0 | 0 |
| text measure, cache miss | - | 0 | 0 | - | - | 0 | 0 |
| text shaping (measurer) | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| semantic bind | - | 0 | 0 | - | - | 0 | 0 |
| hit test (Compositor::hit) | bindings | 0 | 0 | 0 | 0 | 0 | 0 |
| draw (frame tail) | - | 1 | 1 | - | - | 42.7 | 71.0 |
| compositor resolve | - | 1 | 1 | - | - | 20.2 | 37.8 |
| layer hash | nodes | 1 | 1 | 0 | 0 | 0.04 | 0.04 |
| geometry rebuild | nodes | 0 | 0 | 0 | 0 | 0 | 0 |
| text resolve | glyphs | 1 | 1 | 0 | 0 | 0.04 | 0.12 |
| gpu buffer upload | bytes | 0 | 0 | 0 | 0 | 0 | 0 |
| encode and submit | draw calls | 1 | 1 | 1 | 1 | 22.1 | 32.5 |
| gpu wait | - | 1 | 1 | - | - | 602.8 | 817.3 |

## fase 1b: what the gpu itself spends

phase 1 read the gpu side as a wait: the time from submit to seeing the
queue empty, an upper bound that holds the latency of a submit as well as
the work. phase 1b adds gpu timestamp queries, so the gpu reports its own
clock around each pass, and runs the nine scenarios again. nothing here is
an optimization or a design decision, and phase 1 above stays as it was
measured.

### how to reproduce

- machine: the same apple m4 (10-core gpu, metal 4), 16 GB, macOS 26.6.2,
  metal through wgpu 28, rustc 1.98.1.
- build: the workspace release profile.
- the tables: `cargo run --release -p measure -- --out
  <file.md>`. timestamps are on by default now (the runner asks the
  renderer for `RenderConfig::gpu_timestamps`); `--no-gpu-timestamps`
  runs without them. the other flags are as in phase 1.
- the runs: ten with timestamps and five without (the last nine
  interleaved on and off), 120 turns each after 10 warm-up turns, 1200x800
  logical at scale 2, theme core; two more with timestamps at `--scale 1`.
  the machine was shared with other agents (load average 4 to 12 during
  the runs), and so was its gpu.
- the engine side: `cargo test -p engine perf::`; the timestamp tests skip
  where no adapter has `TIMESTAMP_QUERY`.

### what is measured

- `RenderConfig::gpu_timestamps` (off by default) makes the device ask for
  `TIMESTAMP_QUERY`, and only when the adapter has it. off, or on an
  adapter without it, the device asks for no feature and the frame is what
  it was. this m4 has it. it does not have `TIMESTAMP_QUERY_INSIDE_PASSES`
  (wgpu offers that on metal for amd and intel gpus only), so the
  timestamps are per pass: each layer pass and the composite pass write
  one where they start and one where they end
  (`RenderPassDescriptor::timestamp_writes`). a frame writes them only
  while `perf::stages` is on.
- a frame's queries are resolved once the gpu has finished that frame, by
  the next frame's encoder, and read back from a buffer mapped on submit;
  nothing waits for them (`engine::perf::gpu_time`). the first version
  resolved in the same command buffer and read the composite's end as zero
  in some frames: metal writes an end-of-pass sample when the pass's
  fragment work completes, and nothing orders the resolve's blit after it.
  so with the gpu sync on, the gpu stages a turn records belong to the
  frame drawn before it (without the sync, a few frames before). in every
  scenario here consecutive drawn frames do the same work, so the
  per-scenario percentiles hold; one turn's gpu row does not describe that
  turn.
- three stages sit next to the gpu wait: `gpu span (timestamps)`, the
  first pass's start to the composite's end; `gpu layer passes`, the first
  layer pass's start to the last one's end (items: passes with both
  timestamps); `gpu composite pass`, from the later of its own start and
  the layer passes' end, to its end. the cut follows the gpu's timeline
  rather than each pass's own start and end: in the first runs a
  keystroke's composite read 3033 us start to end against a layer pass of
  1908 and a span of 3150, because on this tile-based gpu the composite's
  vertex stage starts alongside the layer pass and its fragments wait for
  the layer's texture. with the cut, layers and composite add up to the
  span.
- submit latency is the gpu wait's p50 minus the span's p50: what the
  wait holds beyond the gpu's own work (scheduling the command buffer,
  seeing it complete through the poll and, with timestamps on, the resolve
  of the previous frame's queries).
- `StageFrame::gpu_micros` (the span) is what a frame loop feeds
  `PerfMonitor::record_gpu_micros`. the runner feeds its monitor every
  turn and prints what it holds after the last one
  (`PerfSnapshot::gpu_micros`: 220 us on the floor to 1156 us in the
  editor in the run below).

### timestamps change no pixel and cost little

- same pixels: `showcase --render` at 9522ee5 and at the head of this
  branch give the same sha256 for Todo (hoff 2x, core 1x) and Editor (core
  1x, hoff 2x). the engine test
  `timestamps_draw_the_same_pixels_as_a_plain_context` draws one scene on
  a plain context and on one with timestamps and stages on, and compares
  the bytes.
- off, the frame path runs two `Option` checks more: no feature, no query
  set, no timestamp write. only `window::draw_frame` writes timestamps; the
  public `encode_layer_passes` and `encode_composite_pass` keep their
  signatures and write none.
- on, the gpu wait's median over the ten runs is 0 to 153 us longer than
  over the five runs without (a resolve blit and the pass samples per
  frame), and the cpu 5 to 17 us longer (the resolve encoded, the readback
  read). runs of the same kind spread as much.

### summary

1200x800 logical at scale 2 (2400x1600 physical), theme core, times in
microseconds, p50 over the drawn turns of each run. each cell is the
median over the ten runs with timestamps, the lowest and highest run in
parentheses. cpu is the frame's p50 minus the gpu wait's p50, as in phase
1.

| scenario | frame | cpu | gpu wait | gpu span | submit latency | gpu layer passes | gpu composite |
| --- | --- | --- | --- | --- | --- | --- | --- |
| a. type in the todo input | 1618 (1550 to 3215) | 149 (133 to 234) | 1467 (1414 to 2980) | 1108 (1032 to 2084) | 392 (332 to 896) | 658 (615 to 1280) | 446 (417 to 686) |
| b. toggle a todo item | 1509 (1281 to 3636) | 131 (124 to 838) | 1379 (1157 to 2798) | 1017 (816 to 1704) | 386 (336 to 1094) | 597 (472 to 1019) | 420 (344 to 684) |
| c. hover a list (16 drawn turns) | 1401 (1224 to 4312) | 130 (122 to 810) | 1277 (1091 to 3502) | 892 (724 to 2700) | 388 (351 to 802) | 469 (370 to 1461) | 422 (354 to 1239) |
| d. modal mid-opening | 784 (764 to 1215) | 47 (44 to 184) | 733 (719 to 1031) | 440 (439 to 499) | 293 (279 to 532) | no pass | 440 (439 to 499) |
| e. type in a 100k-line document | 1444 (1164 to 4059) | 210 (194 to 1131) | 1217 (952 to 2928) | 862 (606 to 1888) | 373 (323 to 1040) | 688 (484 to 1510) | 174 (122 to 378) |
| e'. type in the Editor page | 1781 (1694 to 4727) | 190 (182 to 1226) | 1586 (1505 to 3501) | 1181 (1174 to 2728) | 395 (331 to 773) | 634 (633 to 1489) | 542 (540 to 1238) |
| f. idle redraw | 983 (856 to 2590) | 48 (45 to 471) | 931 (811 to 2119) | 574 (496 to 1326) | 337 (283 to 792) | no pass | 574 (496 to 1326) |
| g. focused field, no typing | 987 (924 to 2476) | 50 (46 to 423) | 937 (868 to 2053) | 576 (572 to 1326) | 345 (295 to 727) | no pass | 575 (572 to 1325) |
| h. empty frame | 524 (458 to 1145) | 31 (27 to 258) | 492 (426 to 888) | 215 (170 to 394) | 283 (252 to 494) | no pass | 215 (170 to 394) |

without timestamps (five runs), the medians of the gpu wait: a 1466, b
1231, c 1124, d 681, e 1084, e' 1487, f 848, g 847, h 420; of the cpu: a
141, b 123, c 121, d 42, e 193, e' 178, f 41, g 41, h 22.

### what the numbers show

these are readings for phase 2, not decisions.

1. the gpu's own work per frame is 215 us (the empty frame: a composite
   of one empty layer) to about 1.2 ms (a keystroke in the Editor page),
   medians. the wait of phase 1 held about 280 to 400 us more in every
   scenario, whatever the content: that is submit latency, not work.
   taken against the waits without timestamps it reads 205 to 360 us.
2. with the latency out, the frame is still bound by the gpu: the span is
   4 to 12 times the cpu of the same frame (the editor 862 against 210,
   the idle redraw 574 against 48, a todo keystroke 1108 against 149).
   phase 1 read 4 to 15 times with the wait.
3. a dirty layer's pass costs 470 to 690 us whatever it holds: 35 nodes
   and 22 draws for the todo keystroke (658), 118 nodes and 3 draws in
   the 100,000-line editor (688), 91 nodes in the Editor page (634). it
   follows pixels: every layer's texture is the whole surface, and its
   pass clears, draws into and stores the msaa 4x texture and resolves
   it. the list's rows sit on a layer clipped to the list, and its pass
   still reads 469.
4. the composite runs on every drawn frame and costs about what a layer
   pass does: 420 to 575 us for the four layers of a gallery page (two of
   them empty), 174 to 215 us for one layer. a frame that rebuilds no
   layer (the idle redraw, the focused caret, the modal mid-opening)
   spends its whole gpu time there: 440 to 576 us.
5. a keystroke splits into a layer pass and a composite of about the same
   size (todo 658 and 446, Editor page 634 and 542); in the editor alone,
   whose scene is one layer, the composite is a fifth of the span (688
   and 174).
6. the gpu runs at clock levels that move between runs: the editor's
   layer pass read 484, 550 or 825 us depending on the run and its
   composite 122, 140 or 209 in the same runs, the same ratios (1.14 and
   1.7). within a run the p50 is steady; the medians above sit between the
   levels. at scale 1 (a quarter of the pixels) the levels spread further:
   two runs read the empty frame at 57 and 72 us and the todo keystroke's
   layer pass at 186 and 507, and one run read the same four-layer
   composite at 127 us in one scenario and 348 in another. at that size
   the times do not follow the pixel count alone.

### limits of this measurement

- a turn's gpu stages are the previous drawn frame's: the resolve waits
  for the gpu to finish. a scenario whose drawn frames differ from one to
  the next would need its readings attributed back.
- a pass is the finest grain: the draws inside a layer pass are not split.
  a backdrop blur's compose and blur passes and the layer effects are not
  timed (they would fall inside the span); none of these scenarios has
  one (one pass per dirty layer in every row).
- on metal a pass that draws nothing (a dirty layer with no node, only
  its clear) samples its start and never its end, so its time is in the
  span and not in `gpu layer passes`. none of these scenarios has one.
- the gpu was shared with other processes, and a timestamp pair holds
  whatever else the gpu ran between its two samples. the clock levels
  (reading 6) move the absolute numbers by up to 1.7 times between runs.
- the loop runs turns back to back with the gpu sync; a window at 60 Hz
  leaves the gpu idle between frames, where it may clock lower still.
- timestamps are written by `window::draw_frame` only (the frame of every
  app that presents through it, and the headless renderer). the engine
  `App`'s own tail (`window/render.rs`) writes none and feeds no gpu time
  to its monitor; that file is outside this task's list.
- webgpu offers `TIMESTAMP_QUERY` too; the code builds for wasm but was
  not run in a browser.

### candidates for phase 2 (not implemented)

added to phase 1's list, with the numbers above:

- the composite is close to half the gpu work of a keystroke and all of
  it in a frame that rebuilds no layer: composing only the layers with
  content, and nothing when no layer and no opacity changed, is where 420
  to 575 us per frame go.
- a layer pass costs the same for 3 draws as for 22: redrawing the
  damaged region of a dirty layer rather than its whole surface is where
  its 470 to 690 us go. a hypothesis to test there, not measured: the pass
  stores its multisampled attachment (`StoreOp::Store`, which the backdrop
  resume needs) as well as the resolve; a pass with no backdrop could
  discard the samples.
- the 200 to 400 us of submit latency is not work. in a window the cpu
  does not wait for it, so it adds to the time until a frame is seen, not
  to the frame budget; the span is the gpu's share of the budget.

### the tables

one run with timestamps (the eighth, a quiet one), the summary and gpu
tables as `measure --out` wrote them:

120 measured turns per scenario after 10 warm-up turns, fixed tick of 16.67 ms, viewport 1200x800 logical at scale 2 (2400x1600 physical), theme core, release build, stages on with the gpu wait, gpu timestamps on. time is microseconds; p50 and p95 are nearest-rank over the turns.

| scenario | frames drawn | frame p50 | frame p95 | input p50 | scene build p50 | draw p50 | gpu wait p50 | layers rebuilt p50 | nodes rebuilt p50 | upload bytes p50 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| type | 120 of 120 | 1577.3 | 2277.8 | 0.08 | 26.5 | 116.0 | 1436.3 | 1 | 35 | 45504 |
| toggle | 120 of 120 | 1439.8 | 1646.2 | 23.5 | 8.42 | 97.3 | 1310.8 | 1 | 34 | 40080 |
| hover | 16 of 120 | 0.38 | 1404.3 | 0.33 | 0 | 0 | 0 | 0 | 0 | 0 |
| overlay | 120 of 120 | 775.0 | 906.9 | 0 | 8.17 | 37.4 | 728.4 | 0 | 0 | 0 |
| editor | 120 of 120 | 1604.7 | 2677.0 | 0.96 | 32.8 | 169.2 | 1406.0 | 1 | 118 | 236232 |
| editor-page | 120 of 120 | 1752.8 | 2535.0 | 0.83 | 30.0 | 149.8 | 1571.2 | 1 | 91 | 166312 |
| idle | 120 of 120 | 855.8 | 1025.7 | 0 | 8.25 | 37.1 | 810.8 | 0 | 0 | 0 |
| caret | 120 of 120 | 986.3 | 1846.1 | 0 | 8.29 | 38.0 | 937.5 | 0 | 0 | 0 |
| floor | 120 of 120 | 511.6 | 779.0 | 0 | 0 | 28.8 | 481.3 | 0 | 0 | 0 |

gpu time per scenario, p50 over the drawn turns. the timestamps of a frame are read one drawn frame later (engine::perf::gpu_time), so a turn's gpu stages are the frame drawn before it. the span runs from the first pass's start to the composite's end; the layer passes from the first one's start to the last one's end; the composite from the end of the layer passes (or its own start) to its end. submit latency is the wait's p50 minus the span's p50.

| scenario | turns read | gpu wait p50 | gpu span p50 | submit latency | layer passes p50 | gpu layer passes p50 | gpu composite p50 | gpu span p95 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| type | 120 of 120 | 1436.3 | 1033.6 | 402.7 | 1 | 615.1 | 418.8 | 1731.8 |
| toggle | 120 of 120 | 1310.8 | 924.7 | 386.1 | 1 | 539.4 | 384.9 | 988.4 |
| hover | 16 of 16 | 1278.5 | 889.2 | 389.3 | 1 | 468.2 | 419.8 | 1631.7 |
| overlay | 120 of 120 | 728.4 | 439.6 | 288.8 | 0 | 0 | 439.6 | 495.6 |
| editor | 120 of 120 | 1406.0 | 1034.0 | 371.9 | 1 | 825.2 | 208.8 | 1889.3 |
| editor-page | 120 of 120 | 1571.2 | 1175.6 | 395.6 | 1 | 633.4 | 541.5 | 1929.3 |
| idle | 120 of 120 | 810.8 | 495.6 | 315.2 | 0 | 0 | 495.6 | 573.6 |
| caret | 120 of 120 | 937.5 | 572.6 | 364.9 | 0 | 0 | 572.2 | 1351.5 |
| floor | 120 of 120 | 481.3 | 219.1 | 262.2 | 0 | 0 | 219.1 | 392.4 |

the gpu rows of each scenario's stage table in the same run (the drawn
turns only for hover; `encode and submit` is the cpu that encoded the same
passes, for scale):

| scenario | gpu wait p50 | gpu wait p95 | gpu span p50 | gpu span p95 | layer passes p50 | gpu layer passes p50 | gpu layer passes p95 | gpu composite p50 | gpu composite p95 | encode and submit p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| type | 1436.3 | 2102.6 | 1033.6 | 1731.8 | 1 | 615.1 | 1008.9 | 418.8 | 609.5 | 37.0 |
| toggle | 1310.8 | 1516.0 | 924.7 | 988.4 | 1 | 539.4 | 588.8 | 384.9 | 415.6 | 36.3 |
| hover | 1278.5 | 2094.0 | 889.2 | 1631.7 | 1 | 468.2 | 1205.7 | 419.8 | 468.2 | 42.2 |
| overlay | 728.4 | 856.3 | 439.6 | 495.6 | 0 | 0 | 0 | 439.6 | 495.6 | 18.4 |
| editor | 1406.0 | 2498.3 | 1034.0 | 1889.3 | 1 | 825.2 | 1390.1 | 208.8 | 376.2 | 29.5 |
| editor-page | 1571.2 | 2360.1 | 1175.6 | 1929.3 | 1 | 633.4 | 1298.8 | 541.5 | 687.4 | 30.9 |
| idle | 810.8 | 979.9 | 495.6 | 573.6 | 0 | 0 | 0 | 495.6 | 573.6 | 17.8 |
| caret | 937.5 | 1707.1 | 572.6 | 1351.5 | 0 | 0 | 0 | 572.2 | 730.5 | 18.3 |
| floor | 481.3 | 750.9 | 219.1 | 392.4 | 0 | 0 | 0 | 219.1 | 392.4 | 16.6 |
