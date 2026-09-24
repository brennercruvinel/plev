---
type: adr
status: accepted
tags: [rec, recorder, session, replay, monster, sem, receipts, file-format, generative-runtime]
date: 2026-09-23
---

# session recording: replay, not re-execution

## context

a bug report today is a video: it shows pixels and explains nothing. after
task-57 and task-58 every control has an identity and a state
(`sem::Snapshot`), and every action has a receipt with the state its actor
observed, the verdict, the delta and the result (`sem::Receipt`, seen stage
by stage through `sem::ReceiptSink`). the thesis (sections 11 and 12) asks
for the session itself as data: who observed what, who asked for which
action on which state, what happened, and what was on screen at that
instant, kept for the last seconds in a bounded ring, so the report
becomes an inspectable excerpt.

two promises must stay apart. replay rebuilds what was presented and what
happened. re-execution would also restore files, network, clock and every
side effect, and run it all again. only replay is promised: watching a
commit being made must never make another commit.

no app recorded anything before this change. monster had the lowering
from its IR to `SceneNode` and delta discovery from sampled frames, but no
way from a frame back to the IR.

## decision

a new crate, `crates/rec` (deps: `sem`, `monster`, `engine`, serde and
serde_json, sha2). `sem` stays pure; `monster` gains only `raise`, the
mirror of `lower`. the recorder keeps the session in memory, and writes
it, whole or in part, as a session file (below).

### the semantic track

- `Recorder` takes explicit calls, `observed(t, actor, scope, revision)`,
  `snapshot(t, &Snapshot)`, `snapshot_full`, `input(t, kind, at,
  target)`, and hands a `Sink` to each dispatcher (`add_sink`). each sink
  has its own origin number, so request ids of two dispatchers never mix.
- every event is `Event { t, kind }`, with the caller's time; the
  recorder never reads a clock. a time earlier than the last one is
  recorded as the last one and counted.
- a snapshot is stored whole (`Kind::Snapshot`, the keyframe) when it is
  the first, when the cadence (`full_every`) is due, when asked, and when
  it adds, moves or reorders nodes, because a `Change` does not say
  where a node goes among its siblings nor the new order (updated on
  2026-09-23: since task-83 a `node_added` carries the node itself, not
  its place; the reader converts a `node_added` written by id alone, the
  form before, into the node the event's own snapshot holds, adr
  a-delta-carries-the-node-a-refusal-names-who-acted). otherwise it is stored as its
  `Snapshot::diff` (`Kind::Changes`), applied in place on read. an
  unchanged snapshot is not stored. any other event arriving when the
  cadence is due re-keys the track first with the state as it stands; an
  unchanged state shares its `Arc<Snapshot>` and is counted once.
- every stage of every receipt is an event; the first one holds the whole
  request, shared by the later stages. the `Receipt` is rebuilt on read.
- a secret never lands in the recorder. the snapshot already keeps only
  its length. the recorder reduces to a length, with the snapshot's own
  rule, a request's parameter aimed at a secret node, every observed value
  on a secret node, and a stale delta on one (its items dropped); the
  refusal reason, the outcome and the error of a secret target are not
  kept. a target the last snapshot does not know counts as secret: the
  recorder fails closed. input records its kind, position and the bound
  control, never a character.

### the visual track

- `Recorder::frame(t, &Compositor)` reads the visible layers in composite
  order and flattens their nodes, once per rendered frame (render on
  demand: a frame only when something changed). the frame is raised to
  the monster IR (`monster::raise_scene`); an identical frame is not
  stored.
- frames collect in an open chunk. a chunk closes on its cadence
  (`chunk_every`), at `chunk_frames`, or when its asset bank is full, and
  closing runs `monster::discover` then `monster::encode`: each chunk is a
  v0 file with an empty asset table and chunk-local u16 asset ids, the
  keyframe unit of the track. a chunk the codec refuses is split in
  halves.
- discovery is told that no step is a discontinuity, so a change between
  two frames is one segment and not a whole-scene keyframe; the reader
  evaluates a chunk only at the frame times it kept, which discovery
  reproduces on the wire grid. between two frames the screen held the
  earlier one, and that is what a read returns, never an interpolated
  value.
- assets (text keys, image handles, paths) are held once per content
  beside the chunks, counted by the chunks that use them, dropped with the
  last one: the ring keeps only the assets its window references.
- `raise` covers every compositor kind the IR v0 carries. clip push and
  pop, backdrop blur and shadow have no IR node: they are reported
  (`monster::Skipped`) and counted (`Stats::skipped`), as are layers whose
  opacity, effects or clip rect the IR does not carry. the replay draws
  without them.
- where each bound control shows rides with the frame (added on
  2026-09-23, task-63 phase d): `Recorder::frame` takes the frame's
  bindings that show (`Compositor::visible_bindings`, the part of each
  rect inside its clip) on a layer the frame draws, in stacking order,
  the one on top last, as `Compositor::hit` ranks them; a scene given as
  nodes (`frame_scene`) binds nothing. a chunk keeps them beside its
  frames only when they differ from the frame before: it opens with the
  whole list (so a cut at a chunk loses nothing), then a step is the whole
  list again (an id came or went, or the order changed) or the rects that
  moved (the same ids in the same order: a toast springing in, a scroll).
  `Stats::bindings_whole` and `bindings_moved` count them. the read side
  answers `Session::bindings_at(t)`, `binding_at(id, t)` and `hit_at(t,
  x, y)`, the frame on screen at `t` as the compositor hit-tested it
  then; the recorder's signatures did not change, so every caller of
  `Recorder::frame` (the laboratory among them) records them.

### the ring

- `Limits { max_age, max_bytes, full_every, chunk_every, chunk_frames }`,
  by default the last 20 seconds in at most 32 MiB, keyed every 2 seconds.
- both limits cut only between keyframes, so what the ring keeps always
  opens at a full snapshot and at a chunk start. by time a segment or a
  chunk goes once the one after it opened at or before `now - max_age`:
  the ring keeps at least the last `max_age` and at most about one cadence
  more. by bytes the older of the two fronts goes while the ring is over;
  when only the newest segment and chunk are left, the next event re-keys
  and the next frame opens a chunk so the ring can cut there.
- bytes are an estimate: inline sizes plus owned text and collections for
  the semantic track, encoded bytes for closed chunks, raw IR for the open
  chunk, and the assets.

### the read side and the replay rule

`Session` is a copy of the window: events by time, `state_at(t)`,
`receipts_for(id, t)`, `scene_at(t)`. the semantic track reads without
decoding a frame. `state_at` returns `State`, the recorded nodes with the
source the app projected them with; a `sem::Snapshot` can only be rebuilt
through the builder, which would stamp the recorder's call site as every
node's source, so `State` computes its revision through a throwaway
rebuild instead.

replay never creates a request and never calls `execute`: a `Session`
holds no dispatcher and no app, and no method takes one. a test reads a
whole recorded session back, and a cold one rebuilt from its events alone,
and proves nothing ran, no request was created and the sink saw nothing
new.

### the session file

a session is a `.monster`, like an animation; the magic tells them apart:
`MON0` is an animation, `MONS` a session. tools dispatch on the magic
(`rec::probe`): `monster::decode` refuses a session as a bad magic, the
session reader refuses an animation by name, and `monster_player` exits
on a session with a message that points at the session inspector
(task-64). the monster container, its golden fixtures and its player are
untouched.

```text
header := magic "MONS", version u16 (0, or 1 with section B), flags u16 0,
          start u64 (ms on the recorder's clock), crate version (u8 len,
          utf8), git sha (u8 len, utf8, "unknown" without git), sec_count
          u16, (tag u8, offset u64, len u64, sha256)*
S := one rec::Event per json line (its serde form), in time order
I := chunk_count u32, (start u64, frame_count u32, time u32*,
     ref_count u32, asset u32*, offset u64, len u64)*
C := the chunks, each the exact bytes monster::encode wrote
A := asset_count u32, (kind u8, len u32, payload)*
B := id_count u32, (len u16, utf8)*, chunk_count u32 (the index's),
     per chunk: step_count u32, (at u32, kind u8, count u32,
     (id u32 | position u32, x f32, y f32, w f32, h f32)*)*
     (version 1 only; kind 0 the whole list by id, kind 1 the moved
     rects by position in the list)
```

- the version says which sections a file has (added on 2026-09-23, task-63
  phase d): version 0 is S, I, C and A, the layout before bindings were
  recorded, and a window without bindings is still written in it, byte
  for byte; version 1 adds B and nothing else. the reader stays strict
  for both: B in a version 0 file is an unknown section, a version 1 file
  without it misses one, and a step out of order, an id that does not
  parse or shows twice in one list, a move before a whole list or past
  it, a rect that is not finite or has no area, and a step at or after
  the next chunk's start are typed errors (`FileError::Bindings`,
  `FileError::BoundId`). every file of version 0 reads as before, with no
  bindings (the nine checkpoint sessions of task-62 read by `plev-inspect`
  built on `rec` before and after the change give the same summary and
  timeline, byte for byte). a reader built before version 1 refuses a
  version 1 file by its version (`unsupported session version 1`), since
  a strict reader cannot skip a section it does not know.

- the build identity comes from `crates/rec/build.rs` (`git rev-parse
  --short HEAD`, never failing the build) and `CARGO_PKG_VERSION`; task-65
  follows it back to the code.
- every chunk is copied as it is and decodes with `monster::decode`
  unchanged, so the u16 counts of the monster container only ever see one
  chunk; the session's own counts are u32 and u64.
- the semantic section is json lines, so it reads, and an export cuts it,
  without decoding a chunk. an event whose json does not read back (a
  `Num` that is not finite) is left out with the changes that depended on
  it, and counted (`Persisted::dropped`).
- the asset table carries text styles with a payload of rec's own (every
  `TextNodeKey` field, bit for bit; monster's kind-0 layout stays
  undefined), paths through `monster::asset_path` (on the twips grid, in
  pieces so any size fits), and images as their size plus, when known,
  their RGBA8 pixels. the engine drops an image's pixels once they reach
  the GPU, so today images are written with their size only and draw
  nothing when read back.
- `Recorder::persist(path)` writes the window; `Recorder::export(t0, t1,
  path)` writes the part from the keyframe of each track that governs
  `t0` to `t1`, chunks copied verbatim, only the assets they reference.
  both copy the window under the lock and write after it, through a
  `.part` file renamed in place.
- `Session::open` and `Session::from_bytes` are strict: a truncated file,
  a bad sha256, an unknown or repeated section, the wrong magic, an
  animation, a bad version or flags, a start that disagrees with the
  content, events out of order, a chunk that does not decode or names an
  asset of another kind are typed `FileError`s, never a panic.
- `rec::install_panic_hook(recorder, dir)` writes the window to
  `dir/session-<unix ms>-<pid>-<n>.monster` on a panic and then runs the
  hook installed before. it never panics (a panic in a panic hook
  aborts), and it only tries the recorder's lock, since the panic may
  have come from inside the recorder on the same thread. no app installs
  it yet. (updated on 2026-09-23: the ide installs it when it records,
  task-63 phase d, see the consequences.)

## measured cost

`cargo test -p rec --release --test session measure -- --ignored
--nocapture`, on the development machine (apple silicon, headless GPU),
the showcase todo page at 1280x800 driven by a scripted person (typing,
enter, hover, toggle, filters, delete; 848 frames over 50 s simulated):

- per frame, record 12 us mean, 4 us median, 7 to 9 us p95, under 1 ms
  max (the chunk close: discover and encode, once per chunk); scene build
  8 to 17 us mean, headless draw (cpu side) 99 to 109 us mean. recording
  is 9 to 11 percent of build plus draw and under 0.1 percent of a 60 fps
  frame. a first cold run measured about 3 times these values.
- 582 of 848 frames were identical to the one before (the caret blink
  redraws) and were not stored.
- the window held 20.6 s: 48 KB of encoded chunks (2.3 KB/s), 42 KB of
  assets (every keystroke is a new text asset), about 90 KB of open chunk
  raw IR; 8.7 KB/s of visual ring in total.
- the todo page drew 2004 clip pushes and pops the IR v0 does not carry,
  no blur and no shadow, in the core and hoff themes.
- the semantic track under stress (a toy app projected 60 times a second,
  its text changing every 100 ms, one commit a second): `snapshot()` 0.7
  to 1.2 us mean (the diff of an unchanged projection included); 20 KB/s
  estimated, 17 KB/s as json.
- persisted: the todo window (20.6 s) is a 76 KB file (3.7 KB/s),
  written in 1.0 to 1.5 ms and read back, strictly, in 0.5 ms (S 7.5 KB,
  I 3.5 to 4.4 KB, C 48 KB, A 16 to 17 KB); the file is smaller than the
  ring (180 KB) because the open chunk is encoded on the way out and
  paths and texts pack tighter. the stressed semantic window (21.3 s, 338
  events) is 362 KB (17 KB/s), written in 1.9 ms (the json round trip of
  every event included) and read in 1.1 ms.

- the bindings (added on 2026-09-23, task-63 phase d; the same measure
  in a debug build, and the `ide-lab` scripted arm with the person):
  the todo page binds its controls in every frame, and its bindings were
  kept whole in 34 of 368 frames (every chunk opens with them, and the
  list changes as rows come and go), never as moved rects, unchanged in
  the other 334; section B is 3.4 KB of a 79 KB file (75.5 KB without
  it), 160 B/s over the 21.1 s window, 4.5 percent. the laboratory's run
  (0.2 s, 9 frames, 3 chunks, 14 ids) writes 1.2 KB of B in a 68.5 KB
  file (67.2 KB without it, 1.8 percent). a point of the replayed todo
  frame, on a 10 px grid, is the control `Compositor::hit` named live.
- the ide's window recording itself (added on 2026-09-23, task-63 phase
  d; a debug build on apple m4, the window under `PLEV_BACKGROUND`; an
  agent through `plev` observes, stages a file, opens the form, writes
  the message and commits; `Instant` around the frame's build and present
  and around `Recording::frame` in each redraw, logged per frame, the same
  scene run with and without `PLEV_RECORD`): recording costs 1.1 ms per
  frame on average, 0.5 ms median (the state projected and diffed, the
  frame raised, the bindings), and the frame that closes a chunk
  (discovery and encoding, once every 2 s while frames come) 2.0 to 8.2
  ms; drawing took 2.1 ms mean with it and 2.4 ms without, the same within
  noise. the window's session is 76 KB over 28.9 s (2.6 KB/s), 2.3 KB of
  it section B (3 percent). with nothing changing on screen nothing is
  recorded (ten idle seconds: no frame, no event); with the commit form
  open and its caret blinking, each 2-second chunk is 4 frames in 4.5 KB
  (2.2 KB/s), plus its whole binding list (about 0.2 KB). in a debug build
  the chunk close is the spike to move off the frame path first, when a
  scene needs it.

the cost does not ask for a change in the frame architecture. the chunk
close is the one spike; moving it off the frame path is left for when a
scene needs it.

## reason

- a flight recorder that explains is data, and the data already exists:
  snapshots, diffs, receipts. the recorder only has to keep it, in order,
  bounded, without a secret.
- the caller's time keeps recording and reading deterministic, like
  `sem`.
- keyframes on both tracks give O(log n) seek to the governing keyframe
  and a ring that can cut anywhere a keyframe is, without a state that
  depends on what was cut.
- raising frames into the existing codec reuses discovery, quantization
  and the player instead of a second encoder, and keeps monster's IR as
  its own model: raise maps onto it, never widens it.
- evaluating only at frame times keeps render on demand honest: the
  screen changes in steps, and the recording shows steps.

## consequences

- the inspector (task-64) reads a `Session`: scrub by frame time, select a
  control, see its state, the receipts that touch it and their actor.
  `Compositor::bindings` is not recorded yet; the inspector needs it to
  map a point on a replayed frame to a control (input events already carry
  the control under the pointer). (updated on 2026-09-23: task-63 phase d
  records them, see the visual track and the session file; the inspector
  reads them through `Session::bindings_at` and `hit_at`.)
- `State` duplicates the reads of `Snapshot` it needs (`get`, `value`,
  `focused`, `revision`). a `sem` constructor that keeps `source` (a
  `Snapshot::from_nodes`) would let the reader return a `Snapshot`; it
  belongs to `sem` and was not added here.
- replay draws without clips, blurs and shadows until the IR has nodes for
  them (a monster IR v1 candidate, reported, not decided here).
- image pixels are not in a `SceneNode`, only an atlas handle, and the
  engine's image store keeps them only until the GPU upload drains them.
  a persisted image draws once the store can give a handle's pixels
  (keeping them, or reading the atlas back); the format already carries
  them.
- a session holds file paths, messages and anything else the app
  projects. recording is opt-in and local; persistence and export must
  never leave the machine without a person's action.
- the showcase wiring (a debug shortcut that persists, the panic hook)
  waited for task-60, which edited the showcase shell. task-60 is
  integrated and the showcase still does not depend on `rec`
  (`crates/showcase/Cargo.toml`): the wiring is task-63 phase d now.
  outside tests, the one caller of `Recorder` today is the `ide-lab`
  binary (task-62 phase c). (updated on 2026-09-23: the orchestrator
  took the showcase wiring and the image pixel retention out of phase d,
  and phase d wired the ide instead, below.)
- the ide records itself (added on 2026-09-23, task-63 phase d). with
  `PLEV_RECORD=1` in a debug build, the window's shell keeps a `Recorder`
  on the workbench's clock through `ide::record::Recording`: a sink on
  the workbench's dispatcher for every stage, the agent port's
  observations through `Host::observed` of `ide::agent::Served`
  (`ide::agent::serve_recorded`), the state after an agent's act, after a
  git result, after a person's click or key that changed something and
  before each frame, every frame drawn with its bindings, and a person's
  clicks and keys (kind, position, the control under the pointer, never
  a character). it saves the window on `session::Save` (cmd-shift-s or
  ctrl-shift-s, bound only while recording), when the loop exits (the
  window closed, or Escape), and on a panic, to `$TMPDIR/plev/sessions/`
  (made private to the user): `ide-<pid>-<n>.monster` for a save, the
  panic hook's own name for a panic, each path in the log. without the
  variable nothing is created, and the shell pays an `Option` check per
  frame and per input. the laboratory keeps its own recording
  (task-88).

## the format decision

two places were weighed for the file: a session profile inside the monster
container (new sections, a new container version) or a sibling container
with its own magic. the sibling was chosen: the golden fixtures and the
animation player stay untouched by construction, the u16 counts of the
monster container only ever see one chunk, the semantic track reads
without touching the codec, and the build identity has a header of its
own. the extension stays `.monster`, the thesis's word for a session, and
the magic carries the difference.

## avoid

- a request created while reading a session, or any api that takes a
  dispatcher or an app on the read side.
- a clock read inside the recorder.
- a secret, or a hash of it, in any event.
- a cut that leaves the window opening between keyframes.
- widening monster's IR to follow the compositor enum; report what raise
  cannot carry instead.
- evaluating a chunk between frame times.
