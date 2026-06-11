---
type: reference
tags: [anm, animation, format, codec, spec, delta, keyframes]
date: 2026-06-11
status: draft-v0
---

# anm format v0 specification

synthesis of four guided studies (swf/ruffle, lottie/thorvg, editors,
rhai/engine-fit; full reports in session transcripts, distilled findings
in kdb/brain-fable-e-bre/anm-formato.lua). the poetic frame: h264 for
vectors. keyframes are I-frames, interframes are discovered deltas, the
renderer interpolates.

## benchmark gates (measured baselines)

- lottie samples measure 1.9 KB/s to 321 KB/s of animation; gzip crushes
  them to 10-13 percent. gate: anm files are born at or below the gzip
  size of the equivalent lottie, without gzip
- swf pure animation delta measured ~1.7 KB/s (10-15 bytes per moved
  object per frame, pre-baked per frame). gate: anm beats swf bytes/s for
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
   shell (sem-boitata); the antidote to flash's opacity
8. no script inside the playback format. scripting is an optional
   sidecar section consumed only by players built with the anm/script
   feature (rhai); playback of tweens never requires it (lottie
   embedding js was the anti-lesson: thorvg ships a whole js engine)
9. color animation via rgba lerp now; color transform multiply+add per
   channel reserved as a v1 op (swf lesson, composes through hierarchy)

## container layout (le, byte-aligned)

header: magic "ANM0", u16 version, u16 flags, f32 duration_s, u16 fps_hint,
asset table (count + entries), easing table (count + custom curves),
description track offset, section index.
sections: K (keyframe snapshot: full node list at time t), D (delta block:
ops until next keyframe), X (script sidecar, optional), T (description
track). checksums per section (sha256, nest lesson).

## node model (anm IR, decoupled from SceneNode)

the codec works on its own IR mirroring plev's animatable surface, so the
frozen format never chases the internal enum; the player lowers
anm::Node -> SceneNode at render time. v0 node kinds and animatable
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
  discontinuity. curve fitting (easing recovery) is v1

## core prerequisite (one line)

SceneNode must derive PartialEq for structural round-trip tests
(TextNodeKey and ImageHandle already do).

## crate plan

crates/anm: ir.rs (node model), write.rs (encoder), read.rs (decoder),
play.rs (player), fixtures/ (golden), benches vs the lottie sample table.
rhai behind feature "script", config default-features=false,
features=["std","f32_float"] (wasm-safe; measured 226 KB gzip dieted).
every file under 369 lines.
