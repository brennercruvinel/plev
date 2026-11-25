---
id: adr-0093
title: "the dev mode is the engine's own: one report a frame, shown once it is whole, in a panel that offers its places as it draws"
project: plev
type: adr
status: accepted
date: 2026-11-25
date_basis: git
recorded: 2026-11-25
topics: [rendering, process, performance]
relations: [task-105, adr-0008, adr-0048]
evidence: [f7695626, bb10319c]
source: reconstruction
source_revision: 7ba5cfab
revised: 2026-11-25
review: reviewed
---

# the dev mode is the engine's own: one report a frame, shown once it is whole, in a panel that offers its places as it draws

Decides how the engine shows a person what a frame did: with its own clocks and its own panel, from one report a frame, and never a frame that is still missing a part ([task-105](../../../Tasks/task-105-profiler-panel-in-the-gallery/task-105-profiler-panel-in-the-gallery.md)).

## context

The first try of task-105 put Puffin and Tracy behind two features of the gallery, with wgpu-profiler timing the GPU. Reading it before replacing it found the stages taken before the present (a frame's boundary was wrong), the GPU's stages never reaching the panel, a pass with no layer to trace its time to, a panel that woke the window four times a second, and six tools reading the frame with no two of them sharing a report.

The second try was the engine's own and measured well. In a window it still showed no GPU time at rest and flickered at every move of the pointer: the engine renders on demand ([adr-0008](../../Rendering/0008-render-on-demand-requires-explicit-invalidation/0008-render-on-demand-requires-explicit-invalidation.md)), a frame's GPU timestamps are read back by a later frame, and the panel drew the latest frame before its time arrived, in a frame nobody drew again. Its panel was built from the kit's buttons, each kept in step by hand with what the panel drew, and its picker chose only what a page had bound to an id: on a page of icons it chose nothing.

## decision

- **The clocks, the counters and the panel are the engine's.** No profiler is a dependency. The data is `engine::runtime::perf` (stages, zones, marks, heap, GPU timestamps); the panel is `engine::runtime::dev` and measures nothing.
- **A report is exactly one frame.** The stages, zones and marks are taken after the present. Every view reads that one `FrameReport`; the lines code said also go to a log that outlives the frames.
- **A frame shows once it is whole.** A view that follows the app shows `Recorder::settled`: the latest frame that has its GPU time, or the latest as it is once the wait for its timestamps is over. The panel asks for the frames that bring the timestamps back.
- **A frame the panel causes is the tool's.** The frames that bring timestamps back and the frames its own pointer causes are flagged and left out of the history, the charts and the findings.
- **A view says where a press does something as it draws.** `Painter::region` keeps a place and an `Act`; the next pointer event is read against that list. A view keeps no widget, and the pointer asks for a frame only when it moves from one place to another.
- **The picker chooses anything the frame draws.** What is under a point is the bound control there (`Compositor::hit`) or the scene node there (`Compositor::node_at`), whichever is the more particular: the node when it is smaller than the control it sits in, the control when the node is its whole ground.
- **The findings are rules over the numbers.** Each insight is read from the frame shown and the frames kept, with the numbers it was drawn from in its words; none is a guess about code the engine cannot see.
- **A launch without it pays nothing.** `PLEV_DEV` (the page's `?dev=` in the browser) gives a launch its dev mode. Without it nothing is drawn, no event is taken, no device feature is asked for, and each mark of the app's code is one relaxed load and a branch.

## consequences

What the panel shows is what the engine measured of itself, with no second clock to reconcile and nothing to install. The GPU's time shows at rest, and a frame appears once.

The panel is a layer of the frame it measures. Its cost is in the frame (the zone `dev mode`), and the findings say it when it is more than a millisecond; the page's own counts leave its layer out.

What Puffin and Tracy offer and the engine does not is not there: sampling with call stacks, the source view of a zone, context switches and lock contention, saving and comparing captures.

An app mounts it with three calls of its own loop (the pointer first, the page's share, the draw after the page). Only the gallery makes them today.

## alternatives

- Puffin and Tracy behind features: the first try. Two dependencies with their own clocks and windows, and a frame boundary the engine could not set.
- The kit's widgets for the panel's controls: the second try. Each needed its place computed twice, for the draw and for the event, and its state kept beside the panel's.
- Showing the latest frame at once and its GPU time when it arrives: the flicker, and at rest no GPU time at all.
- A picker over bound controls only: nothing to choose on a page that binds nothing, which is most of a design system's pages.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0008: render on demand makes invalidation a correctness contract](../../Rendering/0008-render-on-demand-requires-explicit-invalidation/0008-render-on-demand-requires-explicit-invalidation.md)
- [ADR 0048: view source for pixels: a control leads to its projection, a text to its literal](../0048-view-source-for-pixels/0048-view-source-for-pixels.md)
- [task-105: a dev mode over any app: the frame, the GPU, the zones, the heap and the page's controls](../../../Tasks/task-105-profiler-panel-in-the-gallery/task-105-profiler-panel-in-the-gallery.md)
<!-- /related -->
