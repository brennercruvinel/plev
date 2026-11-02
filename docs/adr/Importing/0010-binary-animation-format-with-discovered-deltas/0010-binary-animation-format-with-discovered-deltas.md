---
id: adr-0010
title: "the monster animation format: keyframes plus discovered deltas"
project: plev
type: adr
status: accepted
date: 2025-05-14
date_basis: reconstructed
recorded: 2026-06-12
topics: [animation, formats]
relations: []
evidence: [a4ad0c0]
source: docs/adr/importing/0010-binary-animation-format-with-discovered-deltas.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# the monster animation format: keyframes plus discovered deltas

Decides on a binary format, `.monster` (magic `MON0`), with keyframes and discovered deltas, for small, seekable vector animation rendered by the same engine as the UI. The reason: Lottie is JSON parsed on every frame, SWF pays an O(n) replay on every rewind, and raw video loses its vector nature. The result is O(1) seek between keyframes and zero cost for a node that does not change; the cost of full-body morph at 60 fps is deferred to v1, with morph tracks.

## context

the project needed a way to ship vector animation that is small, seekable,
and renders on the same engine that draws the ui. the existing options each
fail one of those:

- lottie is json: 1.9 to 321 KB per second of animation, parsed every
  frame, and it leans on a foreign runtime that has shipped a whole js
  engine to play scripted clips.
- swf bakes one tag per object per frame and pays O(n) replay on every
  rewind because it has no random access point.
- raw video (webm) wins on dense cel animation but throws away the vector
  nature and the semantic structure entirely.

## decision

a binary format, `.monster`, magic `MON0`, frozen at v1. the poetic frame
is h264 for vectors:

- a keyframe is a full scene snapshot, so seek is O(1) in frames.
- between keyframes only the discovered deltas travel (place, replace,
  remove, and per-property eased segments). a node that does not change
  costs zero bytes, the swf display-list lesson applied at shape
  granularity.
- values are quantized on the wire (twips for coordinates, rgba8 for
  colors), byte aligned, with a sha256 per section.
- a description track carries optional utf-8 per keyframe, the seed for
  accessibility and search that flash never had.
- no script in the playback format; scripting is an optional sidecar, never
  required to play a tween.

the codec works on its own ir, decoupled from the engine `SceneNode`, so
the frozen format never chases the internal enum; the player lowers ir to
`SceneNode` at render time.

## consequences

- discrete motion already wins: the corpus cards file is 0.36x and the
  explosion file 0.74x of the source json size, measured.
- full-body 60fps morphs are still large (snake 42x, money 53x) because v0
  re-tessellates every moving shape into a new asset per sample. the named
  v1 lever is morph tracks: store the curve, not the samples (the swf
  DefineMorphShape lesson).
- two golden fixtures are frozen byte for byte; a wire change is a version
  bump and a new spec entry, never a fixture refresh.

## avoid

- do not add a scripting requirement to the playback path. tween playback
  must never need the script feature.
- do not let the codec ir track the engine enum field for field. lower at
  render time so the frozen format stays stable.
- do not claim a size win on morph-heavy clips without the measured bytes
  on the table.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-43: monster, binary animation codec v0](../../../Tasks/task-43-monster-codec/task-43-monster-codec.md)
- [mon experiment](../../../Research/Importing/mon-experiment-notes.md)
<!-- /related -->
