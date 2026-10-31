---
id: adr-0066
title: "ui asset compiler: an asset leaves the runtime on a measured number"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
topics: [performance, fonts, gpu]
relations: [adr-0011, task-63, task-72, task-74]
source: docs/adr/importing/0066-ui-asset-compiler.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# ui asset compiler: an asset leaves the runtime on a measured number

Decides that an asset leaves the runtime only when measurement shows a gain of at least 5% of the showcase's first headless frame (4 ms of 79) or 16.7 ms summed over one session. With this threshold, no asset left in this phase: the icons cost 0.17 ms at startup, and a pre-generated table would cost about 105 KB. WGSL is now validated in the gate, with file, line and column, and the largest asset share measured in the first frame, the scan of the 805 system fonts, is deferred to task-74.

## context

the monster takes a foreign format once, offline, into a representation
of ours, and keeps the runtime small (thesis, sections 11 and 24;
[`import-foreign-formats-by-conversion-not-embedding`](0011-import-foreign-formats-by-conversion-not-embedding.md)). the other ui
assets never went through that: lucide icons are tessellated at runtime
and cached (`rust/engine/src/widgets/icons/mod.rs`, `tessellate_icon`), fonts
are whole ttf files embedded (`rust/engine/src/drawing/text/fonts.rs`), WGSL
is parsed and validated when a pipeline is built (hot reload reads it
from disk in debug), images are decoded at runtime.

task-72 phase 1 measured before moving anything. the probes are two
ignored tests (`rust/engine/tests/asset_cost.rs`,
`icons::tests::the_tessellation_cost` in comps) and the inventory is in
the task file. in the debug profile the showcase's first headless frame
(the launch page, `Theme`) takes 79 ms from process start to exit. of
that: the first metal device 26 ms; fonts about 23 ms, and nearly all of
it is the scan of the 805 system faces, done twice (the measurer's font
system and the text system's), not the 9 embedded faces (28 us to
register); shaders 2.7 ms for 11 pipelines, of which naga's parse and
validation 0.7 ms; icons 0.17 ms (9 tessellations); images nothing (the
showcase embeds none).

## decision

1. the bar. an asset moves before the runtime only when the measured
   gain is at least 5% of the showcase's first headless frame (4 ms of
   79) or a 60 Hz frame (16.7 ms) summed over a session, and the binary
   does not grow by more than the runtime saves in memory. the probes
   stay in the tree so the number is rerun, not remembered.

2. icons stay at runtime for now. 0.17 ms at launch and 0.97 ms over
   the whole gallery (83 distinct name and size pairs, 163 baselines).
   a baked table of the 42 icons at the three token sizes
   (`theme.control.icons`, 16, 20 and 24) would add about 105 KB to the
   binary (positions as two `f32`, `u16` indices) against about 3.5 KB
   of path data today, and could not retire the tessellator: 45 of the
   83 pairs the gallery asks for are off-token sizes (12, 14, 18, 22),
   and the tolerance is a runtime setting (`RenderConfig::path_tolerance`).

3. WGSL is checked in the gate, not in a build script.
   `rust/engine/tests/wgsl.rs` walks the engine's `src/` and
   `examples/`, reads every `.wgsl` from disk (the files the debug hot
   reload reads) and runs naga's parse and validation as `wgpu-core`
   does, with no optional capability, so a shader that passes needs
   nothing a device may lack. a failure names the file from the
   repository root, the line and the column, and shows the line; a
   validation error naga reports without a span still names the file.
   the runtime is unchanged: `wgpu-core` parses and validates again in
   every `create_shader_module`, and taking that out would mean shipping
   translated modules (naga IR, or msl and spir-v through passthrough)
   for 0.7 ms, below the bar. the hot reload is untouched.

4. the tool, when an asset does move: a CLI step, not a `build.rs`. a
   generator inside the crate that owns the data calls the same code the
   runtime calls (so the pixels are the same by construction) and
   writes a versioned file (binary, through `include_bytes!`). its
   header carries a hash of what produced it: the source bytes, the
   parameters that change the output (for icons: tolerance and sizes),
   and the format version. a gate test generates again in memory and
   compares bytes, which proves the output deterministic and catches
   drift; a script under `script/` regenerates. that header hash is the
   cache: same hash, nothing to do; another hash, the gate fails and
   names the command. a `build.rs` does not fit: a build script only
   sees `[build-dependencies]`, and the code that produces the output
   (the icon parser and tessellator in comps, naga behind wgpu) would
   come in as new dependency edges in `Cargo.lock` plus a second copy
   of code the crate already has, compiled again for the host.

5. fonts and images do not change here. fonts are task-74's (the
   brenner's decision); the number points at the system scan, not at
   compiling the embedded faces, and the task file records the
   recommendation (one face database for the text system and the
   measurer, or the system scan deferred to the first fallback, and
   `Source::Binary` over the static bytes instead of a heap copy per
   font system). the showcase decodes no image at startup.

6. what waits for task-63: recording a real interaction as a `.monster`
   asset (phase 2), and anything that touches the monster container,
   including baked icons stored as an `asset_path` of the monster, waits
   for task-63's session ADR.

## consequences

- a broken shader fails the gate with its place instead of the first
  run that builds its pipeline. the unit test in
  `rust/engine/src/drawing/gpu/tests.rs` (a hand list of nine shaders,
  `Capabilities::all()`, messages without a line) is covered by the new
  one; removing it is left to the integration.
- no new dependency, no new edge in `Cargo.lock`, no runtime change, no
  pixel moved: no asset left the runtime in phase 1.
- the numbers are the debug profile's; the release measure is for the
  brenner, and a release number that crosses the bar reopens item 2.
- `src/gpu/shaders/3Dscene.wgsl` is validated now but loaded by no code
  (the scene3d example keeps its own copy).

## alternatives

- tabela de 42 ícones pré-gerada nos três tamanhos de token: rejeitada. Custaria cerca de 105 KB contra 3,5 KB de path data hoje, e não aposentaria o tessellator: 45 dos 83 pares que a galeria pede são tamanhos fora do token (`## decision`, item 2).
- `build.rs` que gera o asset: rejeitado. Um build script só enxerga `[build-dependencies]`; o código gerador entraria como novas arestas no `Cargo.lock` e como uma segunda cópia do que a crate já tem (`## decision`, item 4).
- módulos WGSL traduzidos (naga IR, msl, spir-v) pré-gerados para evitar a validação de 0,7 ms no runtime: rejeitados, porque o ganho fica abaixo da barra (`## decision`, item 3).
- tirar um asset do runtime porque a filosofia pede: rejeitado; o número medido decide (`## avoid`).
- arquivo pré-gerado sem o hash do que o produziu: rejeitado (`## avoid`).
- mover as fontes nesta fase: adiado para a task-74 (`## decision`, item 5).

## avoid

- moving an asset out of the runtime because the philosophy says so;
  the measured number decides, and the probe that measured it stays.
- a `build.rs` that copies runtime code to precompute its output.
- a baked file without the hash of what produced it and a gate test
  that regenerates it and compares bytes.

## integração (2026-09-24, orquestrador)

aceito. nenhum asset saiu do runtime nesta fase, pelo número: os ícones
custam 0,17 ms na abertura contra uns 105 KB de tabela. o WGSL passou a
ser validado no gate. o maior custo medido do primeiro frame que é de
asset é a varredura das 805 fontes do sistema, feita duas vezes: fica
para a task-74, junto com a decisão de fontes do brenner.
