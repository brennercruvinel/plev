---
id: experiment-visual-determinism-captures
title: "desktop gallery pages: same pixels across scales, themes and runs"
project: plev
type: experiment
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-10-31
topics: [determinism, rendering, effects, platform]
relations: [task-77, task-77/phase-1a-probe, task-77/phase-1b-blur, task-77/closing, adr-0064]
evidence: [8af26af5, 35a47b26, 7127f65a, f6c688b5, adbf948b]
source: reconstruction
source_revision: c2e3f8cd
revised: 2026-10-31
review: reviewed
---

# desktop gallery pages: same pixels across scales, themes and runs

Task-77 asked whether the same gallery page draws the same pixels when it is drawn again, by another GPU context or process, at another scale or in another theme. On an M4, the 82 gallery pages in 4 themes and 4 scales (1312 frames) came out bit-identical with themselves, across two contexts and two processes, drawn headless. Pixels are not compared across scales. The probe found that the blur did not follow the scale; after the fix in phase 1b, scale 1 stayed byte-for-byte equal to the baseline, and the 60 frames the fix changed were all at scales above 1.

## Question

Does the same page draw the same pixels across scales, themes and runs? The phase-1 comparator sets the criterion. A frame is `equal` when every byte matches. Where pixels differ, `requested` means that every differing pixel falls in an area that a product decision declares; `silent` means that it does not, and then a cause is needed ([phase 1a](../../Tasks/task-77-visual-determinism/phase-1a-probe.md), "the comparator and the classes"). Across scales the test is not pixel equality: the ADR line that phase 1 wrote says that across scales one does not compare by pixel. The test for scales is the one phase 1b set: glass and the layer effects should reach the same distance in logical pixels at every scale. The criterion per class is [ADR 0064](../../adr/Workflows/0064-what-identical-means/0064-what-identical-means.md).

## Setup

- **Machine.** Apple M4 (metal 4), macOS 26.6.2, with two displays: the built-in liquid retina XDR at backing scale 2, and a DELL AW2518HF at backing scale 1 ([phase 1a](../../Tasks/task-77-visual-determinism/phase-1a-probe.md)). One GPU on this machine. The GPU driver version is not recorded. The gallery pages were captured headless, with no window.
- **Libraries.** From `Cargo.lock`: wgpu 28.0.0 and winit 0.30.13 (phase 1a); wgpu-hal 28.0.1 and naga 28.0.0 (phase 1b).
- **Build.** The `regress` example in the debug build, with dependencies at opt-level 2 (phase 1a). Phase 1b does not record its build profile.
- **Commits.** Phase 1a on branch `task/77a-determinism`, base `5207f9b8`. Phase 1b on branch `task/77b-blur-scale`, base `3a30113e`, the merge of phase 1.
- **Method, phase 1a.** `regress capture` draws each of the 82 pages in the four themes that the T key cycles (core, hoff, dark and light), at scales 1, 2, 1.25 and 1.5, at 1280x800 logical, twice, with two GPU contexts in one process. It classifies the second frame against the first. `--against <dir>` classifies against the PNG of another process, and `--reverse` draws in reverse order, which gives a different history of atlas and resize. The output goes to `target/determinism/`, outside git. No new baseline was written.
- **Method, phase 1b.** The probe measured three options for the blur at each scale. The chosen one takes one sample per physical texel, with area weights. At scale 1 the pipeline and the uniform bytes are the same as before. The capture made before the fix is kept as `target/determinism/desktop-before`, and the recapture compares against it.
- **Scale-1 fixture.** A 160x96 frame at scale 1, with glass at sigma 3, 8 and 50 over a fractional edge, with stripes, an image, a circle and text. It is compared with `rust/engine/tests/fixtures/blur_at_scale_1.png` (10 KB) and only on the adapter that recorded it, apple M4 with metal ([phase 1b](../../Tasks/task-77-visual-determinism/phase-1b-blur.md)).
- **Cost measure, phase 1b.** GPU span from the timestamps, from the start of the first pass to the end of the composite; p50 over 240 frames after 30 of warm-up; the median of 5 runs per case. The option table was measured in three rounds, and the before-and-after table in six interleaved rounds.

## Results

### Same page, same scale and theme, drawn again

Phase 1a, the desktop against itself (82 pages x 4 themes x 4 scales = 1312 frames):

| run | frames | contexts | processes | time |
|---|---|---|---|---|
| `capture` (writes the PNGs) | 1312 | 1312 equal | not requested | 68,5 s |
| `capture --out target/determinism/desktop-b --against target/determinism/desktop --reverse --no-png` | 1312 | 1312 equal | 1312 equal | 43,2 s |

Zero requested, zero silent, zero of a different size. The 1312 PNGs take 183 MB. In `rust/engine/tests/headless_determinism.rs` (phase 1a), two GPU contexts in one process give the same bytes at scales 1, 1.25, 1.5 and 2, and a renderer resized by 2, 1.5, 1.25, 1, 1.5 and 2 gives, at each scale, the bytes of a new renderer. The `regress check` of the 163 baselines reports them unchanged, with no frame and no GPU, in phase 1a and again in phase 1b.

### Across themes

- The 1312 frames include the four themes at every scale. In both runs of the self-comparison they are all equal ([phase 1a](../../Tasks/task-77-visual-determinism/phase-1a-probe.md)).
- The blur comes from a theme token, `blur_sigma`: 8 in dark and light, 8 in core, 50 in hoff. The shadow sigma feeds the analytic shadow, which was already in logical pixels: 4 in `core.rs` and `hoff_core.rs`, 12 in `hoff.rs` ([phase 1b](../../Tasks/task-77-visual-determinism/phase-1b-blur.md), "where the sigma is declared").
- The recapture after the fix changed 60 frames, all in pages with glass, in the four themes: Effects, the backdrop-blur panel (at 2x up to 66208 px, with a largest channel difference of 76 to 157 per theme); and Bubble, Message, Message Scroller and Dock, the glass of the message dock over a nearly flat background (up to 7371 px, with a largest channel difference of 1 to 5).

### Across scales: the blur reaches the same distance

The phase 1a probe measured a backdrop blur of sigma 4 over a white and black edge. The 10% to 90% ramp was 9 physical pixels at every scale, which is 9 logical pixels at 1x, 6 at 1.5x and 4,5 at 2x. The sigma was applied in physical texels, so the spread in logical pixels shrank as the scale grew ([phase 1a](../../Tasks/task-77-visual-determinism/phase-1a-probe.md)).

Phase 1b measured three options against the 1x result. The table gives the ramp in logical pixels and the largest channel difference against 1x, from 0 to 255, as "before / option 1 / option 2". Option 1b gives the same numbers as option 1 in this table.

| sigma | scale | ramp, logical px: before / option 1 / option 2 | largest channel difference vs 1x: before / option 1 / option 2 |
|---|---|---|---|
| 4 | 1.25 | 7,2 / 9,6 / 9,6 | 63,7 / 0,8 / 0,4 |
| 4 | 1.5 | 6,0 / 8,7 / 8,7 | 88,2 / 0,5 / 0,5 |
| 4 | 2 | 4,5 / 9,0 / 9,0 | 116,6 / 0,6 / 0,6 |
| 8 | 1.25 | 7,2 / 9,6 / 9,6 | 82,8 / 0,6 / 0,5 |
| 8 | 1.5 | 6,0 / 10,0 / 10,0 | 107,3 / 0,6 / 0,5 |
| 8 | 2 | 4,5 / 10,0 / 10,0 | 131,7 / 0,9 / 0,5 |
| 50 | 1.25 | 8,0 / 10,4 / 10,4 | 88,7 / 0,6 / 0,5 |
| 50 | 1.5 | 6,7 / 10,7 / 10,7 | 113,2 / 0,6 / 0,5 |
| 50 | 2 | 5,0 / 10,0 / 10,0 | 136,8 / 0,6 / 0,5 |

Phase 1b chose option 1, one sample per physical texel with area weights, for these reasons as the record gives them:

- Its precision against 1x is the same as the others': a largest difference below 1 on the edge, and a mean of 0,9 to 1,5 on detail under the glass, against 5,4 to 14,0 before.
- It keeps the light of a thin detail in every phase of the grid. Option 2 (a blur at logical resolution, then resampled) does not: a one-physical-pixel line at 3x has light 0,00 / 3,01 / 0,00 in the three phases, against 1,00 / 1,00 / 1,00 before and 0,99 in all three phases with option 1.
- Option 1b, the same weights with two texels per bilinear sample, costs less at 2x in the measurements (1,15 to 1,39x, against 1,42 to 1,49x for option 1) and needs 21 samples per pass at 3x instead of 39, but puts a sub-texel bilinear fraction into every blur outside scale 1, which the ADR lists as one source of difference between GPUs. It was left for when the cost at 3x asks for it.

At scale 1 the change is invisible. The 328 frames at 1x (82 pages in 4 themes) are equal in bit to the capture made before the fix, including the 5 pages with glass, and the scale-1 fixture is equal too ([phase 1b](../../Tasks/task-77-visual-determinism/phase-1b-blur.md), "a escala 1, byte a byte").

### After the fix: the recapture

| run | frames | contexts | processes | time |
|---|---|---|---|---|
| `capture` (writes the PNGs) | 1312 | 1312 equal | not requested | 64,4 s |
| `capture --out target/determinism/desktop-b --against target/determinism/desktop --reverse --no-png` | 1312 | 1312 equal | 1312 equal | 38,8 s |
| `capture --out target/determinism/desktop-vs-before --against target/determinism/desktop-before --no-png` | 1312 | 1312 equal | 1252 equal, 60 silent | not recorded |

The desktop is still deterministic with itself: zero requested, zero silent and zero of a different size, between contexts and between processes. The 60 silent frames of the last row are what the fix changed against the PNGs from before it: 5 pages, 4 themes and 3 scales (1.25, 1.5 and 2), none at 1x.

The glass page costs more after the fix. In six interleaved rounds the after/before ratio was 0,99 (0,94 a 1,04) at 1x, 1,18 (1,10 a 1,25) at 1.25, 1,32 (1,24 a 1,40) at 1.5 and 1,55 (1,31 a 1,71) at 2x, where the range in brackets is the spread of the ratio from round to round. At 2x this is below the 2x stop threshold. At 2x the blur itself doubles, to 27 samples per pass instead of 13. At 1x nothing changes ([phase 1b](../../Tasks/task-77-visual-determinism/phase-1b-blur.md), "the cost before and after").

### Across targets, in the same task

The closure of task-77 compared the pages of other targets with the desktop ([wrap-up](../../Tasks/task-77-visual-determinism/closing.md)). These are not results about scales, themes or runs; they are recorded here because the same captures produced them.

- **Web**, 1280x800 at 1x, WebGPU over Metal: 79 pages identical in every byte, 3 silenced.
- **iOS**, iPhone 17 Pro, 402x874 at 3x: 78 pages with a difference only in the status bar, which is requested, and 4 silenced.
- **Android** stayed at most 2/255 from the desktop, on the emulator's software GPU. Its status-bar line stays `to_confirm`.
- **Causes of the silenced pages.** Surface and Silences, on web and iOS: the proposer selector shows "claude" on the desktop, which has the program, and "rules" where there is no Claude; an app decision. Git, on web and iOS: the page says that the git workspace asks for the CLI and a thread, and only on desktop; an app decision. Editor, on iOS: the cursor blinks, and the capture fell in another phase of the blink; time, not the renderer.
- **A web defect, fixed.** The table id in hex was computed in `usize`, which has 32 bits on wasm32, and it overflowed. It is `u64` now (`view/components/table.rs`), and both table pages are identical in every byte.
- **The table of requested differences** (`requested.json`) marked the web, iOS and android lines `to_confirm` in phase 1a. The closing says the file now confirms the measured lines: web `size`; iOS `status-bar`, `home-indicator` and `size`; android `navigation-bar` and `size`.

## Limits

- **The question holds on the desktop only.** On web and iOS the captures covered one theme and one scale per target. The other themes and scales were not swept page by page ([wrap-up](../../Tasks/task-77-visual-determinism/closing.md)).
- **Scales are not compared by pixel**, by design. The scale result is the blur measure above, not a frame difference between scales.
- **The window is not compared with the headless.** The gallery pages were not captured through a window in phase 1. The check is open, expected to match ([phase 1a](../../Tasks/task-77-visual-determinism/phase-1a-probe.md)), and still pending at the closing.
- **The colour space of the metal layer is open.** The layer's `colorspace` is `nil` on this machine (measured in phase 1a). The check on a P3 display with pure sRGB primaries is still to do. If `nil` meant "already in the display's space", the fix would be to set sRGB on the layer, which changes the engine's colour contract and needs approval.
- **One GPU.** Differences between GPUs and drivers were not measured here. The candidate sources (the precision of `pow`, `exp`, `sqrt` and the polynomial `erf`; the sRGB encode and the blend in 8-bit targets; the MSAA resolve; the bilinear weights) come from reading the shaders (phase 1a). The scale-1 fixture compares only on apple M4 with metal; another machine must record its own ([phase 1b](../../Tasks/task-77-visual-determinism/phase-1b-blur.md)).
- **The frame test does not reach the layer effects.** The `App` frame clears the scene before drawing, and `draw_frame` applies no effects. The three lines of `apply_layer_effects` that now pass the scale have no test that reaches them; the effects themselves are tested at the `EffectProcessor` level (phase 1b).
- **Not measured.** The cost of the blur at 3x on a phone (phase 1b). iOS and android did not compile on this machine during phase 1 (phase 1b); their tools were installed later, with approval (closing).
- **One machine.** The second process drew in reverse order to give a different history. No run was made on another machine (phase 1a).
- **Not recorded.** The build profile of phase 1b and the GPU driver version, in both phases.
- **CI.** The mac job that would require `equal` on every frame waits for CI to come back ([phase 1a](../../Tasks/task-77-visual-determinism/phase-1a-probe.md), closing).

## Records

- [task-77 README](../../Tasks/task-77-visual-determinism/README.md): the task, its goal, scope, acceptance criteria and the list of its phases.
- [phase 1 (task-77a): the probe](../../Tasks/task-77-visual-determinism/phase-1a-probe.md): the environment, the comparator's classes, the desktop against itself (1312 frames), the divergences of this machine, and the handoff.
- [phase 1b handoff (task-77b): blur follows the scale](../../Tasks/task-77-visual-determinism/phase-1b-blur.md): the blur probe, the three options, the scale-1 test, the recapture, the cost, and what stayed open.
- [wrap-up (task-77)](../../Tasks/task-77-visual-determinism/closing.md): the closure against the acceptance criterion, with web, iOS and android, the causes of the silenced pages, and the open items.
