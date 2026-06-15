---
type: reference
tags: [changelog, versions]
date: 2026-06-11
status: living
---

# changelog

## unreleased

- prime number creatures (crates/prime_creatures): the Entropic Life XVI canvas
  demo ported to a native plev crate, desktop and wasm. pure, tested sim core
  (seeded rng, prime coherence matrix 250x250 in four modes, grid-local steering
  physics with kuramoto sync); faithful render through the layer encoder, motion
  trails by position history, cyan bond links via paths, glow halos, breathing
  cores, logical-pixel world, fixed-timestep loop, left-mouse brush. ADR:
  motion-trails-by-position-history. study clone under ref/prime-number-creatures.
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

- brain knowledge base: kdb/brain-fable-e-bre/ (vision, anim format poc,
  transpiler poc, semantics/a11y, research notes, org plan)
- agents.md as the single agent instruction source; doc/arc trio (md,
  yaml, mmd); doc/.conventions
- path: open sub-paths auto-finished before tessellation (lyon abort
  fixed at the root); 2 regression tests
- 11 phi-era demos ported as official examples (crate rename, msaa field,
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
