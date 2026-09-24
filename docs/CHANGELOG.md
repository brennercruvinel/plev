---
type: reference
tags: [changelog, versions]
date: 2026-06-20
status: living
---

# changelog

## unreleased

- a stale refusal names the chain since the state the agent saw
  (2026-09-23, task-90): `acted` named only the other actors' actions
  checked on the exact revision the agent saw; a person who unstaged two
  files in a row was named for the first only. now it names every action
  of other actors that ran from the first request that ran on that state
  on, in order, and the ide's toast says it in one sentence per actor
  ("the person unstaged B, opened the diff of C and unstaged C"). a
  change no request made still names nobody: from outside the app, and in
  the ide a selection, typing or a toast leaving, which move the state
  outside the dispatcher (task-91 anchors on the agent's look instead).

- an act rests only on what its agent was shown (2026-09-23, task-89,
  adr an-act-rests-on-what-its-actor-was-shown): the session inspector
  found that `act --from-revision` built the values an agent "saw" from
  the whole snapshot of the revision, including state outside the scope
  and keys it observed; an agent that looked only at the commit form after
  a person unstaged a file would have committed two files believing
  three. the port now remembers, per actor (the name of `hello`, over
  every connection under it), what each observation showed, and each
  read takes the value from the last observation up to that revision
  that showed it; a read never shown is refused before the dispatcher
  with the new error `not_observed` (additive in v1, `plev` exits 3). a
  terminal stage's revision shows no state, so an act chained on it reads
  what the agent observed before. reproduced on the laboratory: the
  submit after a form-only look is refused as stale, with the file the
  person took out.

- the ide a person uses opens the agent port (2026-09-23, task-86): with
  `PLEV_AGENT=1` in a debug build the `ide` window serves its workbench to
  agents as app `ide` (`ide::agent`, a `wire::Host` over the `Workbench`
  and the frame's visible bindings), the same dispatcher a click goes
  through. the shell serves the port when it wakes and again after every
  git result, so a stage or a commit that ends on the worker sends its
  last stage then, and it draws only when an act ran. proved in a real
  window: an agent staged two files and wrote the message, a person
  unstaged one with real clicks, the agent's submit was refused as stale
  naming the person, the toast on screen and the node the agent reads said
  the same words, and the agent committed what the person left staged.
  the engine gains `Binding::visible_rect` and
  `Compositor::visible_bindings`, which the showcase's port now reads too.

- a recorded session answers why (2026-09-23, task-64 phase a, adr
  the-inspector-reads-a-session-never-runs-it): the new crate `inspect`
  and its cli `plev-inspect <session.monster>` read a session and answer,
  for any request or control, the questions of the thesis: which control,
  what state, who observed what and when, which request with which
  precondition, what moved in what it read between the observation and
  the request and which receipt ran then (a correlation in time, said as
  such), the verdict with its delta, the result. it never runs anything.
  it opened all nine sessions of the two checkpoint a rounds.

- a regression says which control changed before it counts pixels
  (2026-09-23, task-67, adr regression-by-meaning-before-pixels):
  `cargo run -p showcase --example regress -- check` stages every
  baselined page, compares its meaning (`Snapshot::diff`), then a stable
  hash per layer and the bindings, and only then the pixels in the region
  of a layer that moved, naming the controls there. nothing changed:
  157 baselines in about 0.6 s, no frame drawn, no GPU opened. `bless`
  accepts a change and prints what it accepts. a CI job runs the check
  on macOS.

- the ide and urnaui sleep when nothing changes (2026-09-23, task-82
  phase b, adr a-tick-asks-for-a-deadline): the ide's workbench, the
  showcase's Git page and urnaui answer their tick with a `Wake`, and the
  shells wait for the next deadline instead of drawing every frame. a
  focused ide commit field went from every frame (720 in 6 s) to 11; a
  toast at rest is a deadline. the idle ide drew forever in an ordinary
  repository (`git status` took the optional index lock, the file watcher
  saw it and refreshed): every git process now runs with
  `GIT_OPTIONAL_LOCKS=0` and the watcher ignores `.git/index.lock`, and
  the idle ide draws 0 frames and runs 0 git processes. each shell
  catches time up as the loop wakes, so an animation started after a
  long idle runs from its start instead of jumping to its end.

- an agent chains its acts without observing only to learn the revision
  (2026-09-23, task-87, adr a-terminal-stage-carries-the-revision): a
  terminal stage (`completed`, `rejected`, `failed`) carries `revision`,
  the revision the app is at when it goes out, held in the port's ring
  so `--since` and `--from-revision` take it. in the ide laboratory the
  scripted agent went from 24 to 17 messages and from 91.5 to 45.9 KB
  with a person's intervention (16 to 12 messages, 54.5 to 28.8 KB
  without); the projection costs about 0.16 ms per terminal stage in
  debug. `plev --help` also says that `--keys` cuts state only.

- the commit a receipt announces is already on screen (2026-09-23,
  task-85): when the ide's commit receipt completes, the next observation
  shows the commit on top of `stacks` and its files out of the staged
  list, before the refresh reads git; `stacks.commit[<sha>]` carries
  `files` on the ten newest commits and on the last one the ide made. a
  commit's files have one definition everywhere (paths changed against
  the first parent, a rename at both paths), read in process with gix
  and cached by sha: `log(100)` went from about 3 to 23 ms cold and stays
  about 3 ms warm; a whole ide observation of this repository grew 3.8%.
- a delta says what was born, and a refusal says who acted (2026-09-23,
  task-83, adr a-delta-carries-the-node-a-refusal-names-who-acted): a
  `node_added` carries the whole node, served by the agent port as a
  snapshot serves a node, so an agent that sees a commit appear in the
  ide's `stacks` knows what it is in one trip (427 B instead of 172 B
  plus a second observation of 402 B); a stale refusal carries `acted`,
  the actions of other actors that ran on the state the refused actor
  saw, and the ide's toast says it ("you unstaged src/session.rs"); the
  stale rule is in `plev act --help`. sessions recorded before still
  open: the reader puts the node the event's own snapshot holds in place
  of the bare id.
- the ide never shows a status older than what it already did
  (2026-09-23, task-84): a refresh read before a stage, an unstage, a
  discard or an ignore used to land over it, and for a moment
  `changes:staged` said a file was out of the index when it was in, or
  the reverse. the session now drops a read asked before a mutation the
  view already shows; the refresh the mutation's answer sends replaces
  it. the scripted agent of the ide laboratory no longer waits for the
  index to hold still: without that wait, on the old code, all ten runs
  with a person's intervention went wrong.
- a caret at rest costs two frames a second, not sixty (2026-09-23,
  task-82, adr a-tick-asks-for-a-deadline): text widgets answer their tick
  with a `Wake` (a frame now, and when the next is due) instead of "keep
  drawing", the engine's `AnimationTick` carries the real time since the
  last tick beside the clamped one, and the showcase shell sleeps until
  the caret's next flip. a focused, untouched field went from 120 of 120
  measured turns drawn to 4. the ide adopts it in a second phase.
- a frame as svg, and a page's meaning as json (2026-09-23, task-66b, adr
  svg-export-reads-the-scene): `showcase --render <page> --svg f.svg`
  writes the same frame the png shows as vectors, text as glyph outlines
  from the one shaping path, with every approximation (gradients,
  shadows, clips, images and backdrop blur left out) listed on stderr and
  to `--svg-report`; `--semantics f.json` writes the page's semantic
  projection in the lean form `plev observe` serves, `--source` keeping
  each node's source. `--out` is no longer required; one output is.
- what an agent receives is lean (2026-09-23, task-57b, adr
  lean-semantic-form-and-one-redaction): the sem serde form leaves a field
  at its default out and reads it back when absent, so a whole
  observation of the showcase's todo went from 6419 to 2039 bytes with
  three items (26 KB instead of 87 with 103). where a node was projected
  (`source`) now travels only when asked, `plev observe --source`; a
  recording keeps it as before. the rule that keeps a secret inside the
  process lives once, in `sem::redact`, for the port and the recorder
  alike.
- an agent operates a plev app from outside (2026-09-23, task-61, adr
  agents-act-through-the-semantic-contract): the new `wire` crate speaks a
  small json-lines protocol over a local socket, and `plev observe` /
  `plev act` use it. an agent sees the semantic snapshot (scoped, or as a
  delta since a revision it holds) and acts on what it saw; if the state
  moved, the action comes back refused with the delta. it goes through the
  same dispatcher as a click, never touches the backend, and the port is
  off unless `PLEV_AGENT=1` in a debug build.
- the Select panel scrolls (2026-09-23, task-81): a long list is reachable
  by wheel and by keyboard methods, and drawing, clicking and binding read
  one window of rows.
- the GPU's own time per frame (2026-09-23, task-73 phase 1b):
  `RenderConfig::gpu_timestamps` turns on timestamp queries around the
  layer and composite passes when measuring, and the measure example
  reports GPU work next to submit latency. on an M4 a frame's GPU work is
  0.2 to 1.2 ms, still 4 to 12 times its CPU; a dirty layer's pass costs
  the same with 3 draws or 22, and the composite runs on every drawn
  frame. off by default, the device asks for no feature.
- a session saves to a `.monster` file (2026-09-23, task-63 phase c, adr
  session-recording-replay-not-reexecution): `Recorder::persist` and
  `export` write a session file (magic `MONS`, next to the `MON0`
  animations) whose semantic track is JSON lines and whose frames are
  monster chunks byte for byte; `Session::open` reads it back strictly; a
  panic hook can save the last window. a 20 s window of the showcase todo
  is 76 KB. `monster_player` refuses a session with a clear message.
- a person can decline a confirmation (2026-09-23, task-58b): closing the
  confirmation modal in the ide now ends the agent's request as declined
  instead of leaving it waiting.
- accessibility is a projection of the contract (2026-09-23, task-60, adr
  accessibility-projects-the-semantic-snapshot): the screen reader tree is
  built from the same semantic snapshot agents and tests read, with stable
  node ids; a screen reader's click, value or step goes through the app's
  dispatcher like any other actor. the showcase attaches the bridge and
  its todo page is the first with a tree. the old tree, guessed from hit
  regions without labels, is gone.
- the git client proves the thesis (2026-09-23, task-62 phases a and b,
  adr commit-checks-the-index-it-observed): a person and an agent use the
  ide at the same time. an agent's commit decided on a staged set that
  changed is refused with the delta ("B left the staged files"), at the
  dispatcher and again at the real index right before `git commit`; a
  valid one ends with the real sha and file list. every click, key and
  menu entry goes through one dispatcher; refusals show as toasts; an
  agent's discard waits for a person to confirm. tested in real
  repositories against the real git worker.
- a session records itself, in memory (2026-09-23, task-63 phases a and
  b): the new `rec` crate keeps a bounded ring of what happened, as data:
  who observed what, the semantic state (whole or as a diff), every stage
  of every receipt with its request, input without the typed characters,
  and the frames on screen raised into the monster IR and chunked through
  discover. reading a session never creates a request; secrets fail
  closed. recording costs about 12 us a frame on the showcase todo.
  `monster::raise` is the mirror of `lower`. persisting to a `.monster`
  session file is the next step.
- the frame measures itself (2026-09-23, task-73 phase 1): `engine::perf::stages`
  counts and times each stage of a frame (semantic projection, scene
  build, text measure hits and misses, shaping, binds, hit tests, layer
  hashing, geometry, text resolve, upload bytes, encode, gpu wait), off by
  default at the cost of one relaxed load per probe;
  `cargo run --release -p showcase --example measure` runs nine real
  interactions through the window's frame path and prints the tables. the
  first finding: on an M4 the frame is bound by the GPU (0.45 to 2.2 ms of
  wait against 30 to 400 us of CPU), which reshapes the incremental work
  ahead (docs/mission/steps/ongoing/task-73-measurements.md).
- every interactive comps widget speaks the semantic contract (2026-09-23,
  task-59 phase b): chip, icon button, submit button, switch, slider,
  select, tabs, nav link, breadcrumb, split pane, virtual list, table,
  tree, context menu, modal and toast take `.sem(id)`, bind the rect they
  draw and hit-test with, and project their role, state and actions
  (children as keys under their id). every action reads the state it
  changes, so an actor never overwrites a value it did not see. fixed on
  the way: a full `Select` panel picked an invisible option from a 4px
  band at its foot.
- headless is first class (2026-09-23, task-66, adr
  headless-render-shares-the-frame-path): any frame renders without a
  window through the window's own path. `GpuContext::new_headless` builds
  every pipeline with the same code as the windowed constructor,
  `window::draw_frame` draws into any srgb view (`present_frame` is now
  acquire, draw, present, unchanged for every app), and `engine::headless`
  reads the texture back to png. `cargo run -p showcase -- --render <page>
  --out f.png` and `monster_player <file> --frame <t> --out f.png` use it;
  the text probe's private offscreen pipeline is gone. 78 pages, two
  themes, two viewports: byte-identical across runs.
- the showcase todo runs on the semantic contract (2026-09-23, task-59
  phase a, adr semantic-binding-beside-the-scene): the compositor keeps a
  per-frame table of which capability sits where (`bind`, `hit`), beside
  the scene and outside every layer hash; `TextField`, `Checkbox` and
  `Button` take a `SemId` and project their role, state and actions; the
  todo page projects itself, owns a dispatcher, and every click, the Enter
  key and the filters reach the model only through it, as `Actor::Human`
  requests. a test drives the same page by clicks and by test requests
  and gets the same state; an open modal makes the page inert.
- actions carry the state their actor observed (2026-09-23, task-58, adr
  actions-carry-observed-state): `sem` gains `Request::observe`, a pure
  `check` with a fixed order (not found, unknown action, not authorized,
  stale, inert, disabled, invalid param, needs confirmation, accepted) and
  one `Dispatcher` for every actor. a request goes stale only when a value
  its action reads moved, and the rejection names the delta ("removed:
  B"); a hover or a cursor never invalidates it. execution runs only after
  acceptance or a person's confirmation (at the screen or through a screen
  reader), may end later with the id it received, and can be refused again
  at the point of truth. every step lands in a `Receipt` that sinks see as
  it happens. `cargo run -p sem --example commit` plays the thesis
  scenario.
- the semantic contract, crate `sem` (2026-09-23, task-57, adr
  semantic-contract-as-pure-projection): every control can now be
  described as data independent of its pixels: a persistent `SemId`
  chosen by the app (`commit.submit`, `changes.file[src/lib.rs]`), a
  `Role`, a label, state under keys, the actions it accepts with the state
  each one reads and who may request it, and its constraints. the app
  projects it on demand from live state (`Semantics::project`), the
  builder reports every violation at once, a `Snapshot` has a stable
  `Revision` that means the same across processes, and `Snapshot::diff`
  names what changed ("removed: lexer.rs"). secret fields keep only a
  length. pure: no gpu, no window, compiles for wasm. nothing in the
  engine or the apps uses it yet; task-58 adds preconditions and the
  dispatcher on top.
- hot reload on by default (2026-09-22, adr
  hot-reload-on-by-default-in-gpu-context): the shader watcher moved from
  the engine's `window::App` into `GpuContext`, wakes the loop with
  `request_redraw`, is polled by `present_frame` (`effects` is now `&mut`
  there) and marks every layer dirty on a swap; every debug desktop build
  of every app has it. fixes on the way: `shader_dir` pointed at a
  directory that did not exist, `reload_shader` used the raw surface
  format instead of the srgb view format, the narrate root did not match
  `file!()`. `cfg(hot_reload)` from build.rs replaces the feature cfg;
  release, wasm and mobile are unchanged. literal hot reload
  (`hot_reload/literals.rs`, the rsx-hotreload idea without the macro):
  a saved `.rs` whose only change is strings or hex colors is applied to
  the open window, state intact, at `TextNodeKey`, `TextMeasurer`,
  `push_to_layer` and the clear color; a code change rebuilds
  (`cargo build -p $CARGO_PKG_NAME --bin`) and re-execs the running
  binary. `cargo run -p showcase` alone is the dev loop.
- showcase, one page per component (2026-09-21, adr
  showcase-one-page-per-component): the sidebar is a folding tree of
  three groups (Foundations, Components alphabetical, Apps Demo) over
  the page table `view::pages::PAGES`; a page is its own pane, an alias
  of a neighbour's example with a "still missing" note, or a placeholder
  slot for a widget comps does not have yet; Forms, Chrome, Extras,
  Overlays and Lists split into one module per widget; `comps::nav::
  Sidebar` gains `NavGroup`; the launch argument is the page title.
  docs/adr/ became docs/ADR/ with README.md as its index, the changelog
  moved to docs/CHANGELOG.md.
- core theme and the buttons poc (2026-09-21, adr
  core-theme-tailwind-classes-as-recipes): `Theme::hoff_core` is the
  typescript kit (nine shades, six accents, depths as shadow stacks,
  Inter -0.02em, radii 8/10/12/16/22) and the showcase default;
  `theme.button` holds one recipe per button variant and state, the css
  class lists as tokens; `comps::action` rewritten (Button, IconButton,
  SubmitButton over a shared press model); the showcase Buttons page is
  the kit's ButtonsPage. two global fixes: `rect_sdf.wgsl` mixes in
  premultiplied linear (the dark fringe on every rounded corner is
  gone) and blur-0 shadows are anti-aliased.
- editor and git in the showcase (2026-09-21, adr
  embed-an-app-view-by-frame-origin): two app sections filling the area
  under the header; the ide is a lib + bin with `GitSession`;
  `WorkspaceView::set_frame`, `Sidebar.origin`; editor wheel snaps to
  lines; nav link hover is text-only; the sidebar band fades where it
  still has links.

- comps, the design system crate (2026-09-18, adr
  comps-one-design-system-crate): `engine::ui`, `text_input`, `overlay`,
  `charts`, `GraphView` and the editor view moved out of the engine into
  crates/comps (git mv), organized by category (core, recipe, icons,
  action, form, nav, content, feedback, overlay, charts, graph, editor,
  shell); the legacy `Ui` builder and `UiTheme` are gone. the theme grew
  the scales the widgets were improvising (shape by role, control sizes,
  container sizes, durations, breakpoints + sidebar mode, shadow stacks
  by elevation, `text_active`/`text_default`/`tabs`/wash alphas,
  `headline`/`mono`/`readout`, `Theme::hoff_light`); every widget reads
  them, none restates a number; widgets that lay out from tokens take
  `&Theme` on `handle_event`. new widgets: TextField (glass field over the
  editing core), Badge, Avatar, Separator, Panel, Stat, CodeBlock,
  Skeleton, Table (columns by weight, drop by priority), NavLink, Sidebar
  (full / rail / drawer by breakpoint, pinned footer links), PanelHeader,
  Breadcrumb (collapses), AppShell (sidebar + header + content rect +
  menu button + safe area). modal and toast fit a phone width. the ide
  lost its theme and components copies (2 160 lines) and draws from
  comps over hoff / hoff light (status.rs is its only token); the
  showcase is framed by AppShell, gained the Chrome section and its App
  section's keyboard finally reaches the field (it sat behind an
  allow(dead_code)); urnaui's Field is a TextField alias; the todo and
  text_input demos were absorbed by the showcase sections, the editor
  demo moved to comps, the other demo palettes read Theme::default().
  `TextMeasurer::elide_path` came from the ide (and now drops several
  middle segments, the old helper dropped one). matrix test: every widget
  under 13 themes at phone and desktop widths. inventory and migration
  record: docs/catalog.md. workspace tests 855 -> 1414.
- urnaui, the .urna explorer (2026-09-18): nestui renamed to urnaui end to
  end (crate, modules, types, web/urnaui, script/web-urnaui, gate, ci)
  against hoffresearch/urna v0.4.0; the portable reader accepts both
  magics so pre-rename .nest corpora open. the poc's real bugs fixed
  against the 38k-card mtg corpus (530 MB): the file was read into ram
  twice (now one memmap2 map, sections decoded through UrnaView over it),
  results carried no text (the worker ships ChunksLoaded right behind
  Opened), results waited for the next mouse move (worker wakes the loop,
  adr worker-results-wake-the-event-loop), the subprocess bridges hung on
  a full pipe (shared draining helper), the chunks tab showed 0x03 spans
  while cite/search report the 0x16 overlay (now one span_label), blobs
  and spaces were not parsed from inspect. urna pillars surfaced: search
  over a named multimodal space (`space: <name>` in the select, registry
  embedder with the space's dim/model_hash gate), explain panel with the
  selected hit's citation + text + decoded frame, chunk filter by text
  terms / id / urna:// citation, validate, media card with hash-verified
  export, frame previews straight out of the inlined av1 via ffmpeg
  subfile (adr urnaui-media-frames-in-place-via-ffmpeg-subfile), Browse…
  native dialog (rfd). tests 57 -> 80 incl. a media corpus built with the
  urna writer (blob + overlay + space); wasm check green.
- arc unified (2026-09-18): docs/arc/{arc.yaml, arc.md, arc.mmd} became one
  file, docs/arc/arc.toml: the machine map as tables, the human reference
  under [prose], the frame-flow mermaid as a literal string under
  [diagram]. the sync guard, AGENTS.md, the conventions, the pr template
  and the README point at it. content unchanged except nest/nestui ->
  urna/urnaui, which is the rename in flight on this branch.
- glyph raster hardened (2026-08-31): atlas cache key is cosmic-text's
  CacheKey verbatim; fresh swash context per rasterization; one-texel
  zeroed gutter on all four sides of every slot; quad UVs in texels,
  normalized in text.wgsl against textureDimensions (mid-frame atlas grows
  keep emitted quads valid); atlas grows to max before evicting; every
  slot reuse marks atlas_disturbed and re-resolves all layers next frame;
  set_raster_scale is #[must_use]; LRU capacity drops return their
  rectangles; empty glyphs carry alloc_id None; headless pixel validation
  via engine::text::probe (incl. multi-frame render_frames). see
  docs/ADR/glyph-raster-identity-and-atlas-isolation.md
- jetbrains mono removed (2026-08-31): inclusive sans is the single
  embedded family; ~550kb less embedded font data, monospace pinned to it
  as well since no mono face ships.

- engine wave (2026-08): inclusive sans replaces rubik+inter as the one
  embedded family (SIL OFL, five upright weights 300-700, sans family
  pinned in fontdb, ~2.1mb lighter); body letter-spacing zeroed (the
  0.025em token was calibrated for rubik). HiDPI text fixed at the root:
  glyphs rasterize at physical scale (TextSystem::set_raster_scale,
  hooked centrally in resolve_layer_text, cache purged on change), no
  more stretched bitmaps on retina. register_embedded_fonts now returns
  the registered ids and emit_glyphs warns once per non-embedded font
  actually shaped (caught a CJK/math demo string and a text "play"
  glyph, both fixed). ADR updates: embed-every-font-weight-in-use,
  adr-004-hidpi-projection.
- widgets for real apps: chip, empty state, indeterminate spinner,
  split pane, icon button; engine::charts promoted from the showcase
  (model + draw + meter/hbars) and engine::graph + GraphView (force
  layout, pan/zoom canvas) with the same handle_event/render contract
  as every widget. TextMeasurer::truncate_to_width (grapheme-aware),
  engine::clipboard re-export, actions::ShortcutMap for the simple
  keymap case. ADR: official-app-pattern.
- nestui (crates/nestui): .nest vector-db explorer on plev, 6 tabs
  (open/overview/search/chunks/graph/stats), native backend via
  nest-format/nest-runtime path deps + worker thread, portable wasm
  reader (nestread) for the web target.
- showcase 12 -> 14 tabs (typography, effects), scrollable sidebar.
  ci matrix: macos + ubuntu + wasm (.github/workflows/ci.yml);
  script/gate skips the wasm leg with a warning when the rust-std wasm
  target is absent (Homebrew rust), CI stays the full verification.
- one knowledge tree: kdb/ renamed to docs/ and the arc trio moved into
  it (docs/arc/, from doc/arc/); doc/ was retired (its changelog copy
  was the stale duplicate of this file). the mon working note left the
  root for docs/mission/steps/ongoing/mon-experiment-notes.md. web/ owns
  the web target end to end (Trunk.toml, index.html, dist output) and
  script/ owns the commands (gate, web). the arc sync guard, the
  instruction source (.contracts/.agents/AGENTS.md, the only
  instruction file; the root carries none), the pr template, typos.toml
  and the crate docstrings follow the new paths; the svg crate and the
  svg2monster cli entered the arc trio. the root gained LICENSE (mit),
  rust-toolchain.toml (stable + wasm32 pinned) and .gitattributes, and
  lost .env.example and the Makefile (script/gate runs the four-part
  gate; the other targets were one cargo call each). study-clone
  references in docs now name the repository and the inner path, never
  a machine path. ADR: one-knowledge-tree-and-a-minimal-root.
- prime number creatures (crates/prime_creatures): the Entropic Life XVI canvas
  demo ported to a native plev crate, desktop and wasm. pure, tested sim core
  (seeded rng, prime coherence matrix 250x250 in four modes, grid-local steering
  physics with kuramoto sync); faithful render through the layer encoder, motion
  trails by position history, cyan bond links via paths, glow halos, breathing
  cores, logical-pixel world, fixed-timestep loop, left-mouse brush. ADR:
  motion-trails-by-position-history. study clone under refs/prime-number-creatures.
- workspace organized to tiers: engine at root, libraries and apps in
  crates (git, ide, lot, monster, narrate, narrate-macro, parser, rope,
  showcase), demos in examples. crate renames (editor_core to rope,
  git_backend to git, basic-ide to ide, prs to parser, anm to monster,
  narrate_macro to narrate-macro). shaders moved into src/gpu/shaders.
  cargo to workspace.package + workspace.dependencies + workspace.lints +
  tuned profiles; every crate publish = false with a description.
- kdb consolidated to adr + how-to. the brain .lua graph and the framework
  catalogs were retired (drifted, parallel source of truth); the monster
  spec moved to kdb/adr/monster-format-v0.md; conventions moved to
  doc/.conventions/conventions.lua; AGENTS.md points to both. four new ADRs
  (monster format, import-by-conversion, transpiler droplist, workspace).
- qa pass: clippy --all-targets -D warnings clean (uninlined format args
  made idiomatic), naming residue fixed (basicIDE to plev ide), README in
  english added, arc trio refreshed. 1274 tests green.
- monster bridge: lottie retired at the door. lot::cnv samples a lottie once,
  dedups tessellated payloads into the asset table (exact quantized bytes:
  a static shape is one asset and zero delta bytes) and encodes .monster;
  asset_path defines the Path payload wire (uniform color, twips vertices,
  u16 indices, deterministic chunk split); stage size travels in the
  description track (stage WxH). examples: lot2monster (converter cli),
  monster_player (decodes and plays .monster, zero lottie linked).
  measured on the 5 corpus files: cards 0.36x and explosion 0.74x of the
  json (format wins on discrete motion); girl 6.5x, snake 42x, money 53x
  (60fps full-body morphs pay v0's sampled-geometry cost; v1 lever:
  morph tracks, the swf DefineMorphShape lesson)
- monster: encoder mode B (discover) + optimizer passes + full delta ops
  decodable; bench vs json/gzip/webm on 4 fixtures; mode B e2e gate
  (max deviation 0.0375 px / 0.0035 per channel); 124 crate tests
- parser transpiler poc: react tsx+sass card and gpui separator emitted as
  plev builder source; goldens byte-identical to the corpus copies;
  honest droplist (38 entries with file:line and reason, count frozen in
  test); emitted code compiles and renders (examples/parser_card; known
  defect: body text run does not wrap yet)
- workspace clippy clean again (7 parser lints); fmt clean

- brain knowledge base: kdb/brain-Mythos-e-bre/ (vision, anim format poc,
  transpiler poc, semantics/a11y, research notes, org plan)
- agents.md as the single agent instruction source; doc/arc trio (md,
  yaml, mmd); doc/.conventions
- path: open sub-paths auto-finished before tessellation (lyon abort
  fixed at the root); 2 regression tests
- 11 plev-era demos ported as official examples (crate rename, msaa field,
  srgb view, linear clears); makepad_charts verified alive
- test surface: 1022 passing, 0 failed

## earlier (pre-changelog, summarized)

- responsiveness wave: content-driven layout in showcase, real text
  measurement in ide (heuristic deleted), percent dims, hidpi
  resize projection, touch-to-pointer synth
- web wave: showcase runs in the browser (webgpu, trunk); async gpu init;
  web gamma fixed via srgb view formats; pixel-identical to desktop (48,48,48)
- fidelity waves: rubik embedded, measured hoff tokens (#303030), desktop
  gamma fix, glass/backdrop blur, analytic shadows
- foundation: compositor with dirty layers, render-on-demand, rope,
  git, design system (~15 widgets), actions/keymap
- history purge: 417mb of accidental build artifacts removed with
  filter-repo before first push to the private remote (plevdev)
