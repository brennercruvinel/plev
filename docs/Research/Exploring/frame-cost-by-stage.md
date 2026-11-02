---
id: experiment-frame-cost-by-stage
title: "what each stage of a frame costs in nine scenarios"
project: plev
type: experiment
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-10-31
topics: [performance, gpu, rendering]
relations: [task-73, task-73/phase-1-measurements]
evidence: [ed814da, 91fdb33]
source: reconstruction
source_revision: c2e3f8cd
revised: 2026-10-31
review: reviewed
---

# what each stage of a frame costs in nine scenarios

The experiment measured, without changing behavior, what each stage of a frame costs in nine scenarios (typing, clicking, hovering, a modal opening, and frames with nothing happening) on the showcase pages and on the editor widget. Its purpose was to give the phase 2 design of task-73 numbers to start from. The result: on this M4 the frame is bound by the GPU, not the CPU. The GPU wait is 450 to 2200 µs per frame, 4 to 15 times the CPU time of the same frame (30 to 400 µs). Phase 1b, with GPU timestamp queries, put the GPU's own work at 215 µs (the empty frame) to about 1.2 ms (a keystroke in the Editor page), and the orchestrator took phase 2 as far as the GPU.

## Question

Quotes from the Portuguese sources are translated here.

- Phase 1 ([phase-1 page](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md), opening): measure, without changing behavior, "the work each stage of a frame does for a small set of real interactions", so that the incremental dependency system of phase 2 "starts from numbers". The page adds: "nothing here is an optimization or a design decision."
- Stop rule ([task-73 README](../../Tasks/task-73-incremental-dependency-system/task-73-incremental-dependency-system.md), "pare e reporte se"): stop and report if the measurement shows that the bottleneck is not the CPU, because the plan changes.
- Phase 1b ([phase-1 page](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md), "fase 1b"): separate the GPU's own work from submit latency, so that "the gpu reports its own clock around each pass".

## Setup

- **Machine**: Apple M4 (4 performance and 6 efficiency cores; phase 1b: "10-core gpu, metal 4"), 16 GB, macOS 26.6.2, Metal through wgpu 28, rustc 1.98.1. Source: phase-1 page, "how to reproduce", both phases.
- **Build**: the workspace release profile (thin LTO, one codegen unit, as phase 1 states it).
- **Tool**: `cargo run --release -p measure -- --out <file.md>`. The page names these options: `--frames`, `--scenario <key>`, `--theme`, `--width`, `--height`, `--scale`, `--no-gpu-sync`, and `--probe-cost` for the cost of one probe. Phase 1b: timestamps are on by default; `--no-gpu-timestamps` runs without them.
- **Turns**: 120 measured turns per scenario after 10 warm-up turns, with a fixed tick of 16.67 ms (1/60 s). A frame is drawn only when an event handler asked for one, or when the last tick was still moving, as in the window's loop. Percentiles are nearest-rank over the turns.
- **Viewport and theme**: 1200x800 logical at scale 2 (2400x1600 physical), theme core.
- **Runs**: phase 1, three full runs (the summary gives ranges over them; the detailed tables are the second run). Phase 1b, ten runs with timestamps and five without (the last nine interleaved on and off), plus two at scale 1.
- **Load**: phase 1, load average 9 to 12 and a file sync client at 70 to 270 percent CPU; phase 1b, load average 4 to 12, and the GPU was shared with other agents too.
- **Scenarios** (letter in the page, key in the run's file):
  - a. type in the todo input (`type`); a click focuses `todo.input`, one key per turn.
  - b. toggle a todo item (`toggle`); a click on the first row, three seed items.
  - c. hover a list (`hover`); the Scroll Area page, pointer 5 px down the middle of the list and back.
  - d. modal mid-opening (`overlay`); the Dialog page, sampled at turn 27 after the click.
  - e. type in a 100,000-line document (`editor`); `comps::editor::EditorView` alone, 2.8 MB.
  - e'. type in the Editor page (`editor-page`); the page's own sample of about 200 lines.
  - f. idle redraw (`idle`); the Todo page at rest, with an expose each turn.
  - g. focused todo input, no typing (`caret`).
  - h. the floor (`floor`); an empty scene.
- **Stages**: `engine::perf::stages` keeps, per thread and per frame, calls, items and time for each stage. Probes sit at the sites in the page's "what is measured" table.
- **Not recorded**: the commit of the measured branch head. The page names base commits, 4698224 for phase 1 and 9522ee5 for phase 1b, which it uses for the same-pixels checks. The run date is not recorded; the page is dated with the closing of phase 1, 2026-09-23 ([task-73 README](../../Tasks/task-73-incremental-dependency-system/task-73-incremental-dependency-system.md)).

## Results

### Method checks

- Probes cost nothing when off: 0.25 to 0.28 ns per loop iteration with stages off, against 0.46 to 0.48 ns for the empty loop. With stages on, a span costs about 35 ns and a count 0.5 to 3 ns. (Phase 1, "the probes cost nothing when off".)
- Same pixels: `showcase --render` at 4698224 and at the head of the phase 1 branch give the same sha256 for Todo, Card, Editor, Dialog, Typography and Scroll Area. Phase 1b repeats it for Todo and Editor at 9522ee5. (Both phases, "same pixels".)
- The shared machine moved three full runs "up to about 30 percent from each other; the shape of every table stayed the same." (Phase 1, "how to reproduce".)

### Phase 1: cost per scenario, three runs

Source: [phase-1 page, "summary"](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md). Values in µs, p50 over 120 turns; ranges are over the three runs. `cpu` is the frame's p50 minus the gpu wait's p50 of the same run. The last column comes from the second run only.

| scenario | frames drawn | frame | cpu (frame minus gpu wait) | gpu wait | gpu wait above the empty frame (second run) |
|---|---|---|---|---|---|
| a. type in the todo input | 120 of 120 | 1709 to 2160 | 191 to 395 | 1518 to 1765 | about 915 |
| b. toggle a todo item | 120 of 120 | 1522 to 2514 | 176 to 324 | 1346 to 2191 | about 745 |
| c. hover a list | 16 of 120 | 0.5 to 0.7 (a turn that draws nothing) | 145 on a drawn turn | 1390 on a drawn turn | about 790 |
| d. modal mid-opening | 120 of 120 | 830 to 986 | 63 to 94 | 768 to 893 | about 165 |
| e. type in a 100k-line document | 120 of 120 | 1663 to 1831 | 307 to 320 | 1343 to 1524 | about 920 |
| e'. type in the Editor page | 120 of 120 | 1777 to 2285 | 263 to 318 | 1497 to 1967 | about 1210 |
| f. idle redraw | 120 of 120 | 905 to 1293 | 61 to 91 | 844 to 1202 | about 465 |
| g. focused field, no typing | 120 of 120 | 908 to 1180 | 62 to 94 | 846 to 1104 | about 500 |
| h. empty frame | 120 of 120 | 481 to 749 | 32 to 85 | 449 to 664 | 0 |

### Phase 1: second run, by stage

Source: [phase-1 page, "the tables"](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md). Columns are a subset of the run's table; values in µs, bytes and nodes as the page gives them, p50.

| scenario (key) | frames drawn | frame | scene build | draw | gpu wait | layers rebuilt | nodes rebuilt | upload bytes |
|---|---|---|---|---|---|---|---|---|
| a. type (`type`) | 120 of 120 | 1708.9 | 36.0 | 137.4 | 1518.2 | 1 | 35 | 45504 |
| b. toggle (`toggle`) | 120 of 120 | 1521.6 | 12.3 | 118.5 | 1346.1 | 1 | 34 | 40080 |
| c. hover (`hover`) | 16 of 120 | 0.46 | 0 | 0 | 0 | 0 | 0 | 0 |
| d. modal (`overlay`) | 120 of 120 | 830.4 | 12.5 | 45.5 | 767.8 | 0 | 0 | 0 |
| e. 100k-line doc (`editor`) | 120 of 120 | 1830.6 | 50.8 | 233.3 | 1524.0 | 1 | 118 | 236232 |
| e'. Editor page (`editor-page`) | 120 of 120 | 2076.0 | 54.2 | 232.8 | 1812.8 | 1 | 91 | 166312 |
| f. idle (`idle`) | 120 of 120 | 1134.1 | 14.4 | 51.1 | 1065.5 | 0 | 0 | 0 |
| g. focused field (`caret`) | 120 of 120 | 1180.3 | 14.5 | 54.4 | 1104.2 | 0 | 0 | 0 |
| h. floor (`floor`) | 120 of 120 | 658.2 | 0.04 | 42.7 | 602.8 | 0 | 0 | 0 |

For hover, the page gives the 16 drawn turns separately: frame p50 1535.9 and gpu wait p50 1390.4 µs.

### Phase 1: what the second run shows

Source: [phase-1 page, "what the numbers show"](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md), items 2 to 8 unless noted. The page calls these "readings for phase 2, not decisions."

- Typing in the todo input rebuilds the default layer (35 nodes), re-emits 243 glyphs and uploads 45 KB. In the editor, one character re-emits 1551 glyphs and uploads 236 KB. In the 100,000-line document the key handler takes 1.6 µs and the scene holds 118 nodes, the lines on screen.
- Caret shaping: `TextField` reshapes its whole text every frame the caret shows (70 bytes at p50, 22 to 37 µs across the runs). The editor reshapes the caret's line (116 bytes, 27 to 30 µs).
- Semantic projection runs only when a person acts. A toggle projects twice through `TodoPage::snapshot` (9 to 14 µs p50 for the two). The whole click is 30 to 49 µs of input.
- `LayoutEngine::compute` and `Compositor::hit` had 0 calls in every scenario. The pages lay out by hand, with 35 to 71 text measures per frame, all cache hits.
- Render on demand: left alone, the idle todo page drew 0 frames in 120 turns. A hover drew only on the 16 turns its row changed. A focused todo field drew every turn with nothing rebuilt (scene 12 to 15 µs, tail 42 to 61 µs, GPU about 1 ms), "60 times a second for a caret that changes twice."
- Modal mid-opening: nothing is rebuilt or uploaded, yet the whole scene is rebuilt on the CPU (81 nodes, 12 to 15 µs) and all four layers are composited.
- A redraw with nothing changed costs about 65 µs of CPU: scene build 14 µs (79 nodes, 35 measure hits, 8 binds), resolve 28 µs, text resolve 3 µs, encode 20 µs.

### Phase 1: stop and report

Source: [phase-1 page, "stop and report: the frame is not bound by the cpu here"](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md), and the [task-73 README](../../Tasks/task-73-incremental-dependency-system/task-73-incremental-dependency-system.md), "decisão do orquestrador".

- The stop rule was met. The GPU wait is 4 to 15 times the CPU of the same frame (30 to 400 µs against 450 to 2200 µs), and the CPU stages phase 2 was framed around are small (projection 9 to 14 µs per click, layout 0 calls, scene build 12 to 60 µs).
- Options the page lists: (a) keep phase 2 on the CPU dependency graph; (b) carry invalidation down to the GPU; (c) first measure the GPU with timestamp queries. The page recommends (c), then (b), folded into the phase 2 ADR.
- The orchestrator's decision (task-73 README): phase 2 extends the dependency chain to the GPU (damaged regions per layer, no composite for an empty or unchanged layer, text vertices kept per line or per run). A small phase 1b with GPU timestamp queries comes before the ADR.

### Phase 1: a conflict between two values in the same page

- The "what the numbers show" item 1 and the stop-and-report section give the part of the GPU wait above the empty frame as **165 to 1200 µs**.
- The summary table gives **about 1210** µs for e' (second run).

The page does not reconcile the two values; both are shown as written.

### Phase 1b: GPU timestamps, median of ten runs

Source: [phase-1 page, "fase 1b", "summary"](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md). Each cell is the median over the ten runs with timestamps, with the lowest and highest run in parentheses. Values in µs, p50 over the drawn turns of each run. `cpu` is as in phase 1.

| scenario | frame | cpu | gpu wait | gpu span | submit latency | gpu layer passes | gpu composite |
|---|---|---|---|---|---|---|---|
| a. type in the todo input | 1618 (1550 to 3215) | 149 (133 to 234) | 1467 (1414 to 2980) | 1108 (1032 to 2084) | 392 (332 to 896) | 658 (615 to 1280) | 446 (417 to 686) |
| b. toggle a todo item | 1509 (1281 to 3636) | 131 (124 to 838) | 1379 (1157 to 2798) | 1017 (816 to 1704) | 386 (336 to 1094) | 597 (472 to 1019) | 420 (344 to 684) |
| c. hover a list (16 drawn turns) | 1401 (1224 to 4312) | 130 (122 to 810) | 1277 (1091 to 3502) | 892 (724 to 2700) | 388 (351 to 802) | 469 (370 to 1461) | 422 (354 to 1239) |
| d. modal mid-opening | 784 (764 to 1215) | 47 (44 to 184) | 733 (719 to 1031) | 440 (439 to 499) | 293 (279 to 532) | no pass | 440 (439 to 499) |
| e. type in a 100k-line document | 1444 (1164 to 4059) | 210 (194 to 1131) | 1217 (952 to 2928) | 862 (606 to 1888) | 373 (323 to 1040) | 688 (484 to 1510) | 174 (122 to 378) |
| e'. type in the Editor page | 1781 (1694 to 4727) | 190 (182 to 1226) | 1586 (1505 to 3501) | 1181 (1174 to 2728) | 395 (331 to 773) | 634 (633 to 1489) | 542 (540 to 1238) |
| f. idle redraw | 983 (856 to 2590) | 48 (45 to 471) | 931 (811 to 2119) | 574 (496 to 1326) | 337 (283 to 792) | no pass | 574 (496 to 1326) |
| g. focused field, no typing | 987 (924 to 2476) | 50 (46 to 423) | 937 (868 to 2053) | 576 (572 to 1326) | 345 (295 to 727) | no pass | 575 (572 to 1325) |
| h. empty frame | 524 (458 to 1145) | 31 (27 to 258) | 492 (426 to 888) | 215 (170 to 394) | 283 (252 to 494) | no pass | 215 (170 to 394) |

Without timestamps (five runs), the medians of the gpu wait are: a 1466, b 1231, c 1124, d 681, e 1084, e' 1487, f 848, g 847, h 420; the medians of the cpu are: a 141, b 123, c 121, d 42, e 193, e' 178, f 41, g 41, h 22 (same page, same section).

### Phase 1b: what the GPU spends

Source: [phase-1 page, "fase 1b", "what the numbers show"](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md).

- The GPU's own work per frame is 215 µs (the empty frame, one empty layer composited) to about 1.2 ms (a keystroke in the Editor page), medians. The phase 1 wait held about 280 to 400 µs more in every scenario, whatever the content; the page calls that submit latency. Against the waits without timestamps, it reads 205 to 360 µs.
- With the latency out, the frame is still bound by the GPU: the span is 4 to 12 times the CPU of the same frame (editor 862 against 210, idle redraw 574 against 48, todo keystroke 1108 against 149).
- A dirty layer's pass costs 470 to 690 µs whatever it holds: 35 nodes and 22 draws for the todo keystroke (658), 118 nodes and 3 draws in the 100,000-line editor (688), 91 nodes in the Editor page (634). The list's rows, on a layer clipped to the list, read 469.
- The composite runs on every drawn frame: 420 to 575 µs for the four layers of a gallery page (two of them empty), 174 to 215 µs for one layer. A frame that rebuilds no layer (the idle redraw, the focused caret, the modal mid-opening) spends all its GPU time there: 440 to 576 µs.
- A keystroke splits into a layer pass and a composite of about the same size: todo 658 and 446, Editor page 634 and 542.
- The GPU's clock levels move between runs. The editor's layer pass read 484, 550 or 825 µs, and its composite 122, 140 or 209 µs, in the same runs. At scale 1 the levels spread further: two runs read the empty frame at 57 and 72 µs.

Scenario g here is the "before" of the [caret experiment](caret-render-on-demand.md).

## Limits

Stated by the sources:

- The GPU wait is an upper bound: submit latency is inside it (the empty frame alone waits 0.45 to 0.66 ms). Phase 1 had no timestamp queries; phase 1b added them.
- The loop runs turns back to back with no vsync pacing. A 60 Hz window idles between frames, and its GPU may clock lower (both phases).
- The render-side shaping cache (`TextSystem::resolve_for_layer`, `text/system.rs`) has no probe. The text resolve stage is not split: shaping, rasterization, emission and upload are timed together.
- The upload stage counts `GpuVec` writes only. Glyph and image atlas texture writes and the per-layer opacity uniform are not counted.
- The dispatcher's own projection inside `sem` is not probed: a toggle projects three times, two of them measured.
- Tokens read by comps widgets ("resolução de estilo") are not measured. Hit tests done by comps widgets (`Rect::contains`) are invisible to the engine.
- Scenario e runs `EditorView` alone with a generated 100,000-line document; e' is the Editor page inside the chrome with its 200-line sample.
- Holding a key or a toggle every frame is a steady-state load, not a human cadence. The first keystroke's cost (new glyphs, buffers grown) fell into the warm-up turns.
- The machine was shared during the runs.
- Phase 1b: a turn's GPU stages belong to the previous drawn frame, because the resolve waits for the GPU. The pass is the finest grain; the draws inside a pass are not split. On Metal, a pass that draws nothing samples its start and never its end. The GPU was shared with other processes, and a timestamp pair holds whatever else the GPU ran between its two samples. The clock levels move the absolute numbers by up to 1.7 times between runs.
- Phase 1b: timestamps are written by `window::draw_frame` only; the engine `App`'s own tail writes none. WebGPU offers `TIMESTAMP_QUERY` too; the code builds for wasm but was not run in a browser.
- Hypothesis, not measured (phase 1b): the layer pass stores its multisampled attachment (`StoreOp::Store`) as well as the resolve, and a pass with no backdrop could discard the samples. The page says: "a hypothesis to test there, not measured."
- Phase 1 and phase 1b are separate runs, so their absolute numbers are not the same run's numbers.

## Records

- [task-73 README](../../Tasks/task-73-incremental-dependency-system/task-73-incremental-dependency-system.md): the goal, the closings of phases 1 and 1b (`ed814da`, `91fdb33`), the stop rule and the orchestrator's decision.
- [task-73 phase 1 measurements](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md): the method, the nine scenarios, the phase 1 tables and summary, the phase 1b section with its tables, and the stop-and-report.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-73: incremental dependency system](../../Tasks/task-73-incremental-dependency-system/task-73-incremental-dependency-system.md)
- [task-73 phase 1: what each stage of a frame costs](../../Tasks/task-73-incremental-dependency-system/phase-1-measurements.md)
<!-- /related -->
