---
id: adr-0045
title: "regression by meaning before pixels"
project: plev
type: adr
status: accepted
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [determinism, semantics, rendering]
relations: [task-57, task-59, task-66, task-77, learning-cycle-0045]
source: docs/adr/workflows/0045-regression-by-meaning-before-pixels.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# regression by meaning before pixels

Decides that visual regression asks about meaning first and about pixels only afterward: the check answers, in this order, whether the semantic projection changed (by `Revision`), whether any layer or binding changed (by content hash with a stable name), and only then which pixels changed, within the regions of the layers that moved. The verdict is the first two answers, so a machine without a GPU adapter reaches the same result as one with a GPU. Measured across the 157 committed baselines, the check finishes in about 0.61 s without opening a GPU; it matters because a failure now says which control changed, not just how many pixels differ.

## context

a visual regression test that compares whole frames answers with a count
("37,821 pixels differ") and leaves the person to find out which control
moved, and whether it was supposed to. the pieces to answer by name
exist: every control the todo page draws has a `SemId`, a projection
(`sem::Snapshot`, with `Snapshot::diff`, task-57) and a binding
`SemId -> rect, layer` beside the scene (task-59); any page stages without
a window and renders through the window's own frame path (task-66). what
was missing is a baseline to compare against, and a way to know that a
frame changed without drawing it.

the compositor keeps a hash per layer, but it is the dirty tracking's:
`FxHasher` over `SceneNode::hash_u64`, crate-private, with no promise to
stay the same across builds or releases of the hasher. a layer has no
name either: its `LayerId` is the order of creation.

## decision

a check asks three questions, in this order, and stops at the first that
has nothing to say.

1. did the meaning change. the page's projection is compared by its
   `Revision`; equal revisions are equal snapshots, so the baseline's
   `semantics.json` is read and diffed (`Snapshot::diff`) only when they
   differ.
2. did the drawing change. every layer, matched by a stable name, is
   compared by a stable content hash, and every binding by its rect,
   clip and layer. nothing moved: the check ends here, without opening a
   GPU.
3. which pixels changed, only when a layer's hash moved and the baseline
   keeps its image. the frame is drawn through `engine::headless`, the
   baseline png decoded, and pixels compared only inside the regions of
   the layers that moved (what they draw, on both sides). the changed
   pixels are then crossed with the bindings: each control says how many
   of its pixels changed and where.

the verdict is the first two answers: a semantic change, a layer hash
that moved or a binding that moved fails the check. pixels explain a
failure, they never decide one, so a machine without a GPU adapter gets
the same verdict as one with it (the report says the pixels were not
compared).

### the harness

`cargo run -p regress -- check|bless`
(`tool/bench/regress/src/`), an example of the showcase like
`measure`, run by the gate's tests through `[[example]] test = true`.

- a baseline is one (page, theme, viewport) under
  `tool/bench/regress/baselines/<page>@<theme>@<w>x<h>[@<s>x]/`:
  `scene.json` (format `regress.baseline.v1`, hash `regress.layer.v1`,
  the key, how the page settled, the projection's revision, every layer
  with name, z, hash, node count and bounds, every binding), and
  `semantics.json` (the lean form `showcase --render --semantics`
  writes) for a page that projects. `frame.png` only when asked for
  (`bless --png`); a baseline that keeps one keeps it on the next bless
  unless `--no-png` drops it. the committed set: the 78 pages in core and
  hoff at 1280x800 and the todo at 390x844, images for the todo at
  1280x800 only (see consequences).
- `bless` prints what it accepts (the check of the baseline it
  replaces), then writes. `check --json` writes every report for a tool.
- the report goes by `SemId`: every id the page projects or binds says
  semantics changed or not (each change in a few words and as `sem`
  serializes it), visual changed or not (pixels in its rect, a binding
  that moved, or "the layer moved and its pixels were not compared"),
  the layer, the pixels and the region. then the layers that moved, with
  the pixels no bound control covers.

### the layer name

the roots of the `SemId`s bound in the layer (`todo` for `todo`,
`todo.input` and `todo.item[3]`), in bind order, joined by `+`; a layer
with no binding is named by the z-order the app gave it (`z1000`); a name
that repeats gets `#2` in composite order. the compositor's API is
untouched: `Compositor::layers`, `Layer::nodes` and `Compositor::bindings`
were already public.

it is stable where it has to be: an item added under `todo` keeps the
name, a layer created at another z renames nothing. two other forms were
weighed. the full list of bound ids renames the layer every time an item
is added, so a content change would read as one layer removed and
another added. the position among the page's layers renames every layer
above one that is created below.

### the hash

FNV-1a 64 over a byte encoding written out in
`tool/bench/regress/src/scene.rs`: the version tag, the layer's
z-order, visibility, opacity, scissor and effects, then every node in
push order as a tag byte and its fields, each `f32` by its bits. a golden
value computed by a separate script pins the encoding; another encoding
needs another version tag and a bless of every baseline. the dirty
tracking's hash is not touched.

an image is hashed by its size, not by its id or atlas placement: the
image store numbers images in the order a process loads them, so the
same page gets another id after other pages loaded theirs.

### the clock

`showcase::app::stage_page` already steps animations by a fixed 1/60 s
and never reads the wall; it stops when nothing moves, or after 120
steps. `StagedPage` now says how many steps it took and whether the page
settled (every page of the gallery settles in 2 steps), and a test stages
three pages twice and compares the scenes node for node. the check draws
the compositor it hashed: nothing ticks between the hash and the pixels.

## consequences

- measured on this machine (apple m4, debug build; a release build was
  not measured): the 157 committed baselines (the 78 pages in core and
  hoff at 1280x800, the todo at 390x844) check in about 0.61 s, no frame
  drawn and no GPU opened. per page: staging and projecting 3.6 ms
  (median), naming and hashing the layers 0.1 ms, comparing 1 us; the
  first page of a process pays 9 to 26 ms more.
- when a layer moves and the baseline keeps an image (the 78 hoff pages
  blessed with images, `text_mid` changed, 28 pages moved): opening the
  renderer 17 to 21 ms once, drawing and reading back 5.7 ms median (19.7
  max), decoding the png 2 ms, comparing 8 ms median (the region, 37k to
  684k pixels). 78 baselines in 1.17 s instead of 0.3 s.
- a hash can move with no visible change: with the same token change the
  Button page's layer hash moved and 0 of 673k compared pixels differ (a
  node whose color changed draws nothing visible). the check fails and
  the report says 0 px; `bless` accepts it.
- the hash sees the scene, not the raster: a shader, a font file or the
  rasterizer can change the pixels without moving any hash. those need a
  pixel comparison of whole frames (task-77), not this check.
- regression by meaning covers the pages that project: the todo and its
  alias Toggle Group. the Git page binds three ids without projecting;
  every other page is compared by layers, bindings and pixels only.
- of the committed baselines, only the todo in core and hoff at 1280x800
  keeps its image. the todo is the page that projects and binds its
  controls (Toggle Group is its alias), so its image is what turns "every
  control bound in the layer that moved" into the exact control and
  region. no other page projects: Git binds three regions of the
  embedded workbench, every other page binds nothing, so there an image
  would only narrow the layer to a region. core is the launch theme and
  hoff the dark glass one, the pair the headless tests render. each
  image costs about 90 KB per bless and is the pixels of one GPU. every
  other baseline, when a layer moves, names that layer and every control
  bound in it, pixels not compared; `bless --png --dir <tmp>` makes a
  local copy with images when regions are needed.
- the hashes are exact `f32` bits of a scene built on the CPU. a few
  pages take coordinates from the system libm (`sin` and `cos` in the
  donut chart, the spinner, the skeleton and the Effects page); another
  macOS could give another last bit there. the CI job on macOS is the
  first place that says so.
- every change that moves a page is blessed in the same change; the CI
  job (`structural regression (macos)`) fails otherwise, and uploads the
  JSON report.
- `Approximation.node` of the svg export is still formatted text; this
  harness did not need it.

## avoid

- `Layer::compute_hash`, `SceneNode::hash_u64` or any std or seeded
  hasher for a baseline.
- a verdict that depends on having a GPU.
- comparing whole frames when a layer's region says where to look.
- a bless whose accept report was not read.

## learnings

- [what the autonomous cycle taught: regression by meaning before pixels](learned-regression-by-meaning-before-pixels.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [what the autonomous cycle taught: regression by meaning before pixels](learned-regression-by-meaning-before-pixels.md)
- [task-57: semantic contract (crate `sem`)](../../../Tasks/task-57-semantic-contract/task-57-semantic-contract.md)
- [task-59: semantic binding with the render and one operational path](../../../Tasks/task-59-semantic-binding-and-one-path/task-59-semantic-binding-and-one-path.md)
- [task-66: headless as a first-class citizen](../../../Tasks/task-66-headless-first-class/task-66-headless-first-class.md)
- [task-67: structural regression before pixels](../../../Tasks/task-67-structural-regression/task-67-structural-regression.md)
- [task-77: visual determinism](../../../Tasks/task-77-visual-determinism/task-77-visual-determinism.md)
- [program: from GPU UI engine to generative interaction runtime](../../../Research/Synthesis/generative-runtime-program.md)
- [how to validate visuals by pixel](../../../Research/Workflows/validate-visuals-by-pixel.md)
<!-- /related -->
