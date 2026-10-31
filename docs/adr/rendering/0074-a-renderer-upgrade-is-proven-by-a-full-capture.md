---
id: adr-0074
title: "a renderer upgrade is proven by a full capture"
project: plev
type: adr
status: accepted
date: 2026-09-26
date_basis: header
recorded: 2026-09-26
topics: [gpu, determinism, process]
relations: [adr-0072, task-78]
source: docs/adr/rendering/0074-a-renderer-upgrade-is-proven-by-a-full-capture.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# a renderer upgrade is proven by a full capture

Fixes the order for moving the rendering dependency from wgpu 28 to 29, done in phase 2 of task-78: resolve offline for all targets, capture the baseline on the old version, move only the requested crate, fix the API, and prove it with a full capture in which every frame matches the baseline. The result was 1312 of 1312 frames equal, verified by the agent and again by the orchestrator. It matters because `regress check` compares meaning and layer hashes and does not draw when nothing changed, so on its own it does not prove the pixels remain the same.

## context

task-78 phase 2 moved plev from wgpu 28 to 29, so that burn (on wgpu 29)
can compute on the renderer's own device (decision 16 of the program,
[adr ai-native-single-process](../structure/0072-ai-native-single-process.md)). a graphics dependency can change pixels
without failing a test. the structural regression (`regress check`)
compares meaning and layer hashes and draws no frame when nothing moved,
so it cannot prove that the frames are the same after a renderer upgrade.

## decision

the way a renderer dependency moves, in this order:

1. before touching the manifests, check that the new version resolves
   offline for every target: change the version in a scratch copy,
   `cargo update -p <crate> --offline`, then
   `cargo fetch --offline --target <t>` for the host, wasm32, android and
   the iOS simulator, and revert. agents build offline; a crate missing
   from the cache is the orchestrator's to fetch before the briefing.
2. capture the base on the old version:
   `cargo run -p regress -- capture --out target/determinism/<old>`,
   every gallery page in 4 themes at 4 scales (1312 frames).
3. move the version with `cargo update -p <crate>` only. in a workspace of
   its own (urnaui), an offline update can also downgrade platform-only
   crates that were never downloaded; put each one back with
   `cargo update -p <crate> --precise <version>`, so the lock changes only
   what the upgraded crate pulls in.
4. fix the API where the compiler points, and read each change in the
   locked source (file:line in the handoff).
5. prove it: `regress -- capture --out target/determinism/<new>
   --against target/determinism/<old> --no-png`. every frame must be
   `equal`. any difference stops the work: report how many, which pages,
   the largest delta, and two before and after pairs. a difference is
   never accepted on the spot.
6. the gate, `regress check`, and `cargo check` for android and the iOS
   simulator.

what wgpu 29 changed for plev:

- `get_current_texture` returns `CurrentSurfaceTexture`. a covered window
  on Metal now answers `Occluded`: the engine skips the frame, and the
  existing `WindowEvent::Occluded(false)` handling redraws when the window
  shows again.
- `InstanceDescriptor` goes by value and has no `Default`; the engine uses
  `InstanceDescriptor::new_without_display_handle()`, the equivalent of the
  old default.
- pipeline layouts take `Option<&BindGroupLayout>`, and depth state takes
  `Option` for its write and compare.
- wgpu 29 needs rust 1.87, so `rust-version` went to 1.87. at that floor
  clippy's `manual_is_multiple_of` is active: `n % k == 0` becomes
  `n.is_multiple_of(k)`, one line of it in `rust/socket` (the behaviour and
  the wire format are the same).

the result: 1312 of 1312 frames equal to the wgpu 28 capture, checked by
the agent and again by the orchestrator. the engine now exposes
`GpuContext::instance()` and `adapter()`, and `RenderConfig::compute`, off
by default, which asks the adapter's limits (a model outgrows the default
256 MB buffer).

## consequences

- `target/determinism/` is the place for captures. it is build output and
  goes with a `cargo clean` of `target/`; a base worth keeping moves out
  first.
- the next renderer or text-stack upgrade (winit, cosmic-text, swash)
  follows the same six steps.
- burn's `metal` feature needs wgpu's passthrough shaders, experimental in
  29, so `rust/llmrun` refuses it on the engine's device. after a device
  loss the handles burn holds are dead: whoever uses burn on the renderer
  registers the new device (no test covers it yet).

## alternatives

O texto não registra alternativas pesadas. Ele recusa dois caminhos: aceitar uma diferença de quadro no ato, que o passo 5 proíbe (cada diferença para o trabalho e é relatada com a contagem, as páginas, o maior delta e dois pares antes e depois), e tomar o `regress check` como prova de pixels, já que ele compara significado e hashes de camadas e não desenha quando nada mudou (context).
