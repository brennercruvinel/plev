---
type: reference
tags: [monster, animation, format, codec, spec, delta, keyframes]
date: 2026-06-11
status: draft-v0
---

# monster format v0 specification

synthesis of four guided studies (swf/ruffle, lottie/thorvg, editors,
rhai/engine-fit; full reports in session transcripts, distilled findings
in kdb/brain-fable-e-bre/monster-formato.lua). the poetic frame: h264 for
vectors. keyframes are I-frames, interframes are discovered deltas, the
renderer interpolates.

## benchmark gates (measured baselines)

- lottie samples measure 1.9 KB/s to 321 KB/s of animation; gzip crushes
  them to 10-13 percent. gate: monster files are born at or below the gzip
  size of the equivalent lottie, without gzip
- swf pure animation delta measured ~1.7 KB/s (10-15 bytes per moved
  object per frame, pre-baked per frame). gate: monster beats swf bytes/s for
  eased motion because one segment replaces N per-frame tags
- round-trip: decode(encode(timeline)) is structurally identical, 100
  percent, property-tested
- golden fixture frozen at first release of the codec; byte-identical
  builds thereafter

## design decisions (and the study that earned them)

1. scene is a flat map depth -> node, delta ops are place | modify |
   replace | remove with presence flags per field; an unchanged node
   costs zero bytes (swf display list)
2. keyframe = full scene snapshot = random access. seek is O(1); swf
   lacked this and paid O(n) replay on every rewind (ruffle run_goto)
3. interframe = per-node, per-property segments: target value + easing +
   duration. the player interpolates; fps-independent, scrubbable. the
   swf baked one tag per frame per object; we send the curve
4. easing: 1 byte. 0x00 linear, 0x01 hold, 0x02 ae-default, 0x03..0x1f
   named presets (mapping to plev Easing variants), 0xff custom cubic
   bezier as 4 quantized u8 (x in [0,1], y in [-0.5,1.5] covers
   overshoot). dedup table of custom curves in the header, segments
   reference by index (8 presets covered 87 percent of 6166 lottie
   keyframes; thorvg dedups at parse time, we dedup in the format)
5. values quantized in the file, f32 in memory: coordinates as i32
   twentieths of a logical px (twips lesson: integer determinism),
   colors rgba8, angles/ratios u16 fixed. byte-aligned layout + optional
   zstd envelope; no variable-width bitfields (2026 verdict: alignment +
   zstd beats bit-packing and stays simd/mmap friendly)
6. definitions separated from instances: assets (text styles, image
   handles, paths) declared once with u16 ids, instanced by depth
   (swf DefineX/PlaceX)
7. description track: optional utf-8 text per keyframe, authored in the
   editor, nlp-completed at build. feeds screen readers and the semantic
   shell (plev); the antidote to flash's opacity
8. no script inside the playback format. scripting is an optional
   sidecar section consumed only by players built with the monster/script
   feature (rhai); playback of tweens never requires it (lottie
   embedding js was the anti-lesson: thorvg ships a whole js engine)
9. color animation via rgba lerp now; color transform multiply+add per
   channel reserved as a v1 op (swf lesson, composes through hierarchy)

## container layout (le, byte-aligned)

header: magic "MON0", u16 version, u16 flags, f32 duration_s, u16 fps_hint,
asset table (count + entries), easing table (count + custom curves),
description track offset, section index.
sections: K (keyframe snapshot: full node list at time t), D (delta block:
ops until next keyframe), X (script sidecar, optional), T (description
track). checksums per section (sha256, nest lesson).

## delta ops (D block semantics)

every op carries at_s, its offset from the owning keyframe's t. modify
carries node_id plus per-prop segment chains and lowers to IR tracks.
the structural ops address depth slots, because the scene is a flat map
depth -> node (decision 1): place and replace carry a full node whose
depth names the slot (an occupied slot is overwritten); remove carries
depth u16. in the IR they are the timeline lists PlaceNode{t, node},
ReplaceNode{t, depth, node} (node.depth must equal depth) and
RemoveNode{t, depth}. ops act only inside their keyframe segment; the
next snapshot resets the scene, which is what keeps seek O(1): the
player replays the current segment's ops only, never the file. at one
instant application order is place, replace, remove, each depth
ascending; the encoder serializes modify ops first (kept byte-identical
with pre-ops files), then structural ops in that same canonical order.

note (2026-06-11): the remove operand became decodable in this revision
as depth; earlier decoders rejected every structural op, so no file
ever carried the node_id reading an early container.rs comment
described. golden fixtures: golden_v0_minimal.monster (modify only, frozen
at first release) and golden_v0_ops.monster (all four ops, frozen when they
became decodable).

## node model (monster IR, decoupled from SceneNode)

the codec works on its own IR mirroring plev's animatable surface, so the
frozen format never chases the internal enum; the player lowers
monster::Node -> SceneNode at render time. v0 node kinds and animatable
props (from the engine-fit study):

- rect: x y w h color
- rounded_rect: + corner_radius border_width border_color
- gradient_rect: + color2 angle_deg
- text: x y color (typography static via asset TextStyle id)
- image: x y w h corner_radius (asset image id)
- path: tessellated; morph = cpu re-tessellation in v0, shader-side later
- group opacity via layer; per-node opacity = color alpha in v0
- transform (rotation/scale/skew) is NOT in v0: requires a future
  PushTransform/PopTransform compositor pair (queued workstream)

## player contract

- own deterministic timeline; samples with plev::animation::ease() and
  Interpolate (both pub). Tween stays for ui animation; the player does
  not depend on Tween seek
- driven by AnimationTick received from the runner (FrameClock is the
  clock; the player never owns a wall clock)
- reactive surface via signal/: playing ReadSignal, time ReadSignal,
  play/pause/scrub writers; the showcase motion tab binds to these
- scene pushed per frame; compositor dirty-hash makes unchanged pushes
  free (measured behavior of the engine)
- skip sub-epsilon frame updates (thorvg lesson)

## encoder modes

- a (primary, v0): lowering from an authored track model (the mot editor
  project or test-built IR timelines): tracks already exist, encoder
  packs segments and discovers per-field presence
- b (basic, v0): from a sampled frame sequence: diff consecutive
  snapshots, emit modify ops, linear segments, keyframe insertion on
  discontinuity. curve fitting (easing recovery) is v1. implemented as
  discover (crates/monster/src/discover.rs): samples are quantized to the
  wire grid before diffing, slot transitions become place | replace |
  remove (an id moving depth is remove + place), per-prop runs merge
  greedily into linear segments while every interior sample stays
  within half a quantization step of the line (linear motion is one
  segment at any sample rate), and snapshots are also inserted on a
  configurable random access cadence, with continuous props landing on
  them so motion crosses cadence keyframes smoothly. a re-placed id is
  pinned against the held tail of a dead chain from an earlier life in
  the same segment. output feeds encoder mode a unchanged

## path asset payload (v0, defined 2026-06-11)

asset payloads are opaque to the container; the Path kind's bytes are
defined by crates/monster/src/asset_path.rs so importers and players meet
through one codec: color rgba8 (tessellation assigns one color to every
vertex), vertex_count u16, index_count u32, vertices as i32 twips pairs,
indices u16. positions quantize to the twips grid, so sub-twip float
jitter packs to identical bytes and dedup-by-payload holds across
frames (a static shape is one asset, one unchanged node, zero delta
bytes). a payload too large for the u16 asset limit splits at triangle
boundaries deterministically. the description track's first entry
carries `stage WxH` (container::stage_size), so a player learns the
composition bounds from the file alone.

the lot::cnv bridge (lottie json sampled once -> dedup -> discover ->
encode) measured on the 5 corpus files: cards 0.36x and explosion 0.74x
of the json size; girl 6.5x, snake 42x, money 53x. discrete motion wins
already; 60fps full-body morphs pay v0's sampled-geometry cost
(morph = cpu re-tessellation, every moving shape is a new asset per
sample). the v1 lever is morph tracks: interpolated path assets, the
swf DefineMorphShape lesson, so the file carries the curve instead of
the samples.

## optimizer passes (encoder-side, format-neutral)

optimize (crates/monster/src/optimize.rs) runs between authoring or
discovery and encode, on the IR only; the wire layout and frozen
fixtures never change. three passes over any validated timeline:
static track collapse (a chain that never strays from its base value
past the tolerance is removed; the snapshot already carries the
value), RDP keyframe reduction (Ramer-Douglas-Peucker over each
track's value x time polyline; deviation is measured at the sample's
own time, endpoints and extremes survive), and collinear fusion
(consecutive linear segments whose shared landing sits on the pair's
chord merge). tolerances are counted in quantization steps of each
prop's wire grid, so one number means the same visual error on every
prop; the defaults are half a step, the error quantization already
commits, so default optimization is lossless on the wire. only linear
segments merge: an eased curve does not survive re-parameterization,
so non-linear segments bound the reduction runs. the passes iterate to
a fixpoint and collapse decisions are order independent: optimizing
twice equals optimizing once. discover's pin tracks (twin tracks on
one (node, prop) in one keyframe window) are never collapsed, and a
track whose base a later place/replace would shift keeps its first
segment intact.

## core prerequisite (one line)

SceneNode must derive PartialEq for structural round-trip tests
(TextNodeKey and ImageHandle already do).

## crate plan

crates/monster: ir.rs (node model), write.rs (encoder), read.rs (decoder),
play.rs (player), fixtures/ (golden), benches vs the lottie sample table.
rhai behind feature "script", config default-features=false,
features=["std","f32_float"] (wasm-safe; measured 226 KB gzip dieted).
every file under 369 lines.
