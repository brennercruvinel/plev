---
type: adr
status: proposed
tags: [rec, recorder, session, replay, monster, sem, receipts, generative-runtime]
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

a new crate, `crates/rec` (deps: `sem`, `monster`, `engine`; serde behind
a default feature). `sem` stays pure; `monster` gains only `raise`, the
mirror of `lower`. this round keeps everything in memory; the file format
is pending (below).

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
  it adds, moves or reorders nodes, because a `Change` carries neither the
  new node nor the new order. otherwise it is stored as its
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
  us mean (the diff of an unchanged projection included); 20 KB/s
  estimated, 17 KB/s as json.

the cost does not ask for a change in the frame architecture. the chunk
close is the one spike; moving it off the frame path is a phase c option.

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
  the control under the pointer).
- `State` duplicates the reads of `Snapshot` it needs (`get`, `value`,
  `focused`, `revision`). a `sem` constructor that keeps `source` (a
  `Snapshot::from_nodes`) would let the reader return a `Snapshot`; it
  belongs to `sem` and was not added here.
- replay draws without clips, blurs and shadows until the IR has nodes for
  them (a monster IR v1 candidate, reported, not decided here).
- image pixels are not in a `SceneNode`, only an atlas handle; persisting
  a session with images needs the engine's image store to give the bytes
  of a handle.
- text styles have no payload layout in monster v0 (asset kind 0 is
  declared, its bytes are not); persistence has to define one.
- a session holds file paths, messages and anything else the app
  projects. recording is opt-in and local; persistence and export must
  never leave the machine without a person's action.

## pending: the file format (phase c)

where a persisted session lives is the orchestrator's decision, proposed
in the handoff of this round:

- a) a session profile inside the monster container, with new sections
  and a new container version;
- b) a sibling format with its own magic that stores the semantic track
  on its own and the visual chunks as unchanged monster v0 files.

the recommendation of this round is (b): the golden fixtures and the
animation player stay untouched by construction, the u16 counts of the
container only ever see one chunk, the semantic track is read without
touching the codec, and the build identity task-65 needs gets its own
header. persistence, export and the panic hook wait for that decision.

## avoid

- a request created while reading a session, or any api that takes a
  dispatcher or an app on the read side.
- a clock read inside the recorder.
- a secret, or a hash of it, in any event.
- a cut that leaves the window opening between keyframes.
- widening monster's IR to follow the compositor enum; report what raise
  cannot carry instead.
- evaluating a chunk between frame times.
