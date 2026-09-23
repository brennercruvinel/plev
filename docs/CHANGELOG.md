---
type: reference
tags: [changelog, versions]
date: 2026-06-20
status: living
---

# changelog

## unreleased

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
