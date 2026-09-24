---
type: adr
status: accepted
tags: [wire, scene, compositor, codec, rec, socket, mirror, images, security, showcase, generative-runtime]
date: 2026-09-24
---

# the scene travels as data

## context

the ui runs in one place and could draw in another, at the level of the
scene: not pixels, not json of widgets. a window's frame is already data:
a vector of pure `SceneNode`s per layer, hashed per layer by the
compositor's dirty tracking (`compositor/scene.rs`, `layer/mod.rs`). what
was missing (task-79, thesis sections 2 and 27): a way for that data to
leave the process, a codec that gives back the same frame, and a viewer
that draws it.

the probe (task-79 phase 1, `crates/rec/tests/scene_probe.rs`, the task
file) measured the gallery, 82 pages in two themes at 1280x800:

- the agent transport (task-61) can carry it. a v1 app of before
  answers an unknown request type with `bad_message`, the id kept, the
  connection open (`codec.rs`), so a new request fits v1; `act` already
  answers many lines to one id. `MAX_LINE` (1 MiB) bounds only what a
  client sends.
- the monster path the recorder uses cannot mirror: the IR v0 has no
  node for a clip, a backdrop blur or a shadow (`raise.rs` skips them: 4
  to 12 clips per frame, up to 63 shadows, the glass) and keeps what it
  carries on the twips and rgba8 grids.
- a codec of the compositor's own nodes costs 0.02 to 0.11 ms per frame
  (debug), 30 to 134 KB for a whole frame (of which 28 to 124 KB are the
  sidebar's icon paths), and, written as a patch of the dirty layer, 195
  to 4 812 bytes for one change, near the monster delta (63 to 4 047) and
  exact.
- the pixels of an `ImageHandle` could not be read outside the engine:
  the store kept them (task-76) with no reader. the round stopped; the
  orchestrator released `gpu/image.rs` for one read-only accessor.

## decision

### the channel: a request of protocol v1 on the agent's connection

```text
-> {"v":1,"id":"1","type":"hello","name":"mirror"}
<- {"v":1,"id":"1","type":"welcome","app":"showcase","actor":{"agent":{"name":"mirror"}}}
-> {"v":1,"id":"9","type":"scene","window":"main"}
<- {"v":1,"id":"9","type":"scene_blob","key":"8a1f...","at":0,"total":24012,"bytes":"AQAAAA..."}
<- {"v":1,"id":"9","type":"scene","seq":1,"head":"AAAgRA...","layers":[{"key":0,"hash":"3c9e...","body":"..."},...],"bounds":[...]}
<- {"v":1,"id":"9","type":"scene","seq":2,"layers":[{"key":0,"hash":"77d0...","patch":"..."},{"key":1,"hash":"51aa..."}]}
```

- `scene` watches one window (`main` by default; the showcase also
  offers `page`, the page's own window). its frames come as replies to
  that id for as long as the connection lasts; one watch per
  connection, after `hello`. an app of before refuses it as
  `bad_message`, which is how a client learns there is no scene there;
  an app whose person did not share its scenes refuses it as
  `scene_not_shared` (see privacy). `plev observe`, `plev act` and
  `--ask` are untouched: no client of before sends it or gets its
  replies.
- the transport does not read a frame (`wire::scene`). the app offers
  each frame it draws (`AgentPort::offer_scene`): per layer a key (the
  app's `LayerId`), a hash, a body and maybe a patch against the frame
  before; per asset a key and bytes; the head; the bindings. per watch
  the server keeps what it sent and sends only what moved: a layer whose
  hash changed (its patch when the watch holds the patch's base, its body
  otherwise), an asset the watch does not hold, the head and the
  bindings when they changed. a frame where nothing moved for a watch
  sends nothing.
- one framing: json lines, binary parts in base64. an asset, and a body
  too large for its frame's line, go before the frame in `scene_blob`
  lines of at most `SCENE_PART` (384 KiB), so every line stays under
  `MAX_LINE`, the limit a client's line has. a layer that carries
  neither body nor patch is the one the watch holds, or the blob just
  sent under its hash. `SceneReader` puts a watch's lines back together.
- the app never waits: the port takes the offer on the app's thread and
  queues lines to the writer threads (`try_send`). a watcher whose waiting
  lines pass `SCENE_QUEUE` (16 MiB) is cut off, like a client whose queue
  is full. the shell writes a window's frame only while someone watches
  it (`showcase::app::mirror::Scenes`), after each frame it draws there
  and after each drain of the port, so a watch that just came gets the
  frame on screen.

### the codec: per dirty layer, in `rec`

`rec::scene` (rec already depends on the engine and on monster, already
keeps assets once per content, and tests against the gallery):

- a layer's body is its properties (z-order, visibility, opacity, clip,
  effects) and every node as it is: a tag and its fields, an `f32` as
  its bits, a text as its `TextNodeKey` (the string and the typography)
  and its place, a clip, a blur and a shadow like any other node. an
  image is named by an asset key (a fixed image's handle, one per
  content; a stream image's place and size, the same for all its
  frames), a path by one (its engine hash and sizes). the hash
  is FNV-1a 64 of the body; the layers go in composite order.
- a patch is the layer against the frame before's: how many nodes of the
  start and of the end stay, and the nodes between. the sender writes it
  when the layer was there and the patch is the smaller; the receiver
  checks what it makes against the layer's hash.
- the receiver (`Mirror`) reads a body strictly (a cut or broken input
  is a `SceneError`, never a panic), holds each layer at its hash, and
  lets go every asset the frame it just took does not reference: the
  rule the server keeps for each watch, so both sides agree without
  saying it.
- `SceneNode` does not change: every field is public and it is
  `PartialEq`. a text key is read back field by field, never through
  `TextNodeKey::new`, which takes the string the source has now under
  hot reload.

### what travels and what does not

- travels: every node of every layer, bit for bit; each layer's
  properties; the head (logical size, scale, clear in linear space);
  each fixed image's pixels, once per content while frames reference
  it; each path's geometry, the same way; a stream image's size and
  place; where each bound control shows (its `SemId` and visible rect,
  the form an observation's `bounds` has).
- does not travel: a stream image's pixels (a camera's frames), ever;
  the glyphs (a text is shaped and rasterized on the other side with the
  same embedded fonts: the frame is equal because the engine is the
  same); the fonts; anything of the app's state or semantics beyond the
  bindings (the snapshot is `observe`'s); input (phase 2).

### images

`ImageStore::pixels(&handle)` and `image_pixels(&handle)` (task-79,
`gpu/image.rs`) read the pixels the store's log keeps: a stream's latest
frame through any of its handles, `None` for a handle of another size or
of a placement no slot sits at. `ImageStore::is_stream(&handle)` and
`image_is_stream(&handle)` tell a stream image (a slot `stream` made,
whose pixels `replace` changes) from a fixed one. no field, no behavior
and no memory was added.

the sender asks, per image node, whether it is a stream. a fixed image
travels with its pixels, once per content; on the receiver it enters
the store by `load_image_rgba` (the store's dedupe answers the same
handle when it comes again). a stream image travels as its size and
place and nothing else (the asset's pixel flag is 2, its id is written
0), under a key its frames share, so a new frame of a camera moves no
layer and sends nothing; the receiver draws in its place a dark box
with its edge and the words "live image, not mirrored"
(`rec::scene::LIVE_LABEL`). the first version of this phase had the
receiver keep a stream slot per camera (commit `7728da47`); with no
stream pixels coming, it makes none.

### the viewer

`showcase::app::mirror::Viewer` puts a watch back together and draws it
into a compositor of its own, layer for layer; with `bindings`, each
bound control outlined and named on a layer above every other (the
task-59 bindings, the inspector's view). `examples/mirror` is the
viewer: headless by default (a hash per frame), in a window with
`--show`.

### privacy

every connection to the port is an agent (`Actor::Agent`, by its
`hello`), and the semantic contract decides what an agent is shown: a
snapshot keeps a secret's length only (`sem::redact`), and the capture
ADR keeps a camera's frames and a microphone's samples from every agent.
a scene carries everything the window draws, which the redaction never
sees. so:

- **the scene is shared apart, by the person.** the port alone does not
  share it: the person who launches the app asks for it, `PLEV_MIRROR=1`
  beside `PLEV_AGENT=1` (a debug build; `AgentPort::from_env` reads it),
  or an app that opens its port itself calls `share_scenes`. unshared, a
  `scene` request is refused with a code of its own, `scene_not_shared`,
  whose reason says what the person does to share; everything else on
  the connection goes on as before.
- **a stream image never crosses.** a camera's frames (any stream image)
  travel as their size and place only, with the scenes shared or not;
  the viewer draws a box that says the live image is not mirrored. this
  phase has no override.
- **every watch has a name, and the app says it.** a watch comes after
  `hello`. the app logs who begins and who ends watching which window
  ("scene: claude watches main", "scene: claude no longer watches
  main"); a watch that begins reaches the host as an observation of its
  agent (`Host::observed`), so a recording keeps it.
- **the redaction's gap is known.** a field that drew a secret would
  reach whoever watches: the scene is pixels as data, and `sem::redact`
  only governs the semantic channel. no comp draws a secret today (no
  masked field exists yet); phase 2 closes this before anything leaves
  localhost: a node that draws a secret is marked and travels masked.

### security

- local only. the channel is the agent port: a unix socket `0600` in a
  private directory of this user (`$TMPDIR/plev`), off by default,
  opened by `PLEV_AGENT=1` in a debug build or by an app that opts in,
  and the scene on it only with `PLEV_MIRROR=1` (privacy, above).
  nothing listens on a network interface; nothing leaves the machine.
- leaving localhost is out of this phase. it needs explicit pairing (a
  person on both ends agreeing, a key per viewer), an encrypted
  transport, the redaction's gap closed, and never remote execution:
  phase 2's input goes back as semantic requests through the
  dispatcher, the same path an agent's act takes, never as code.

## measured

on the gallery, 1280x800 @1, debug build (the workspace crates at
opt-level 0):

- in process, every page in both themes (164 frames) and, over the
  socket between two processes, the first frame, Card (images), Todo (a
  hover), Effects (glass) and Capture: the viewer's frame has the app's
  frame hash. with the fake camera on, none of its frames' bytes is in
  the watch's output, and the viewer's frame differs from the app's only
  inside the camera's box.
- the band, a watch's bytes on the wire (`band`, ignored): the first
  frame 41 899 to 561 420 bytes (median 43 811; Card's two images are
  506 KB of it; the sidebar's icon paths, about 37 KB as base64, are most
  of a plain page's);
  one change 243 to 32 288 (median 347, over 100 frames; a hover's patch
  318 bytes in one line); a frame with no change 0; the camera turning
  on, 1 433 bytes (the box in its place); a frame of the camera, 0.
- on the app's thread, per offer: a hover 0.2 ms, a whole page 2 to 10
  ms, a frame of the camera 0.1 to 0.3 ms (nothing moves).

## reason

- one transport, one protocol: the scene rides the connection an agent
  already has, under the same version, trust and limits; a second socket
  would be a second protocol with its own access rule.
- json lines with base64 over a binary switch after `hello`: frames are
  small once the first one is there (a patch is hundreds of bytes), and
  the 33% of base64 only applies to what travels; a binary switch would
  need an envelope for every reply after it (a version 2) and would take
  `socat` and `plev` out of reading the stream.
- our own codec over monster: the mirror must give back the frame drawn,
  bit for bit, clips, glass and shadows included; monster's IR is a
  playback format with a quantized grid.
- the transport keeps the per-watch state and the codec stays in `rec`:
  `wire` cannot depend on the engine without changing the lockfile, and
  should not (`plev` and wasm stay light); one encode per frame serves
  every watch, whatever each one holds.

## consequences

- a window's frame has a second consumer: whatever the compositor draws
  must be in the codec. `put_node` matches every `SceneNode` variant, so
  a new one fails to compile until it travels (and `take_node` reads its
  tag back).
- the recorder's gap closes with the same accessor, and is not closed
  here: `rec/src/payload.rs` writes an image asset with `has_pixels = 0`
  because a recorder could not read pixels from a handle; with
  `image_pixels` it can, and a replay would draw its images.
- a mirror of a window that shows a camera is not pixel-equal there:
  the box stands where the frames are, by design.
- a large fixed image costs its base64 on the app's thread once per
  watch (Card's two images, 9 ms of an offer in a debug build); encoding
  the lines on the writer threads is the next step if that matters.

## phase 2

input comes back as semantic requests through the agent channel: the
viewer hit-tests a click against the bindings it already has and sends
an `act` on the control under it (resting on an observation, like any
agent), never raw events and never coordinates. a person at the viewer is
then an actor the app's dispatcher sees, with the same receipts,
confirmations and refusals. pairing, if the scene ever leaves the
machine, comes with it.

## avoid

- a scene over the network without pairing, or on any socket but the
  agent port's.
- raw input events from the viewer: the viewer acts through the
  semantic contract, like an agent.
- quantizing, dropping or approximating a node to save bytes: the patch
  already makes a change small, and a mirror that is not equal is not a
  mirror.
- a second copy of the image pixels for the channel: the store's log is
  the one copy, read through `image_pixels`.
- a stream image's pixels on the wire, or a scene without the person's
  `PLEV_MIRROR=1`: the port is an agent's door, and the contract decides
  what an agent sees.

## integração (2026-09-24, orquestrador)

aceito depois de uma devolução. a primeira entrega mandava os frames da
câmera a quem assistisse, e todo cliente do port é agente: contra o ADR
`capture-is-a-job-the-person-turns-on`. a correção entrou na mesma
branch: o canal só existe com `PLEV_MIRROR=1` dado pela pessoa (recusa
`scene_not_shared` sem ele), a imagem de fluxo viaja sem pixels nem id
e o espelho desenha uma caixa no lugar, e quem assiste fica no log e na
gravação. o `image_is_stream`, só de leitura, entrou no store para o
primeiro frame da câmera também não sair; aceito pelo orquestrador.

