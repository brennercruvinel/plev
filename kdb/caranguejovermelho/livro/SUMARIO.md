---
title: caranguejo vermelho, sumario
status: stub
tags: [livro, estrutura, sumario]
destino: kdb/caranguejovermelho/livro
idioma: pt-br
---

# caranguejo vermelho

estrutura completa do livro. partes 0 a 7 mais apendices, com a faixa de paginas
por parte e, para cada capitulo, o bloco de rastros que vai ancorar o texto. o
rastro e placeholder: lista o adr de `kdb/adr/`, a crate, o diff ou commit e o
numero de benchmark que sustentam o capitulo. nada de afirmacao solta, cada
capitulo aponta para um diff, um adr e, quando couber, um numero medido.

faixa total: 569 a 963 paginas. a parte 4 fica com ~30 paginas abertas para o
experimento mon, que e trabalho vivo. as partes 0 e 4 sao escritas em outro
fluxo; aqui elas aparecem so para fechar o mapa.

## faixa de paginas por parte

| parte | titulo | faixa | nota |
|-------|--------|-------|------|
| 0 | origem | 30-50 | escrita em outro fluxo |
| 1 | rust de verdade | 90-150 | |
| 2 | a engine, por dentro | 120-200 | |
| 3 | um codebase, varios mundos | 60-110 | |
| 4 | o experimento mon | ~30 (abertas) | escrita em outro fluxo |
| 5 | medir, nao achar | 50-90 | |
| 6 | seo, wasm e desafios | 40-80 | |
| 7 | onde isto se encaixa | 40-70 | |
| A | apendices | 40-90 | |

---

## parte 0, origem (30-50)

aurora, o legado, por que rust sobrevive as ai, humildade e creditos. escrita em
outro fluxo; aparece aqui so para o mapa ficar inteiro.

### 0.1 a decisao, quando a aurora nasceu
### 0.2 os nomes, de phi a plev, e caranguejo vermelho
### 0.3 por que rust nao some quando o modelo de turno muda
### 0.4 humildade e creditos, areweguiyet sem ironia gratuita

rastros (placeholder):
- adr: pendente
- crate: repo inteiro
- diff/commit: pendente (historico parcial, forgejo perdido)
- bench: n/a

---

## parte 1, rust de verdade (90-150)

ownership, borrow, traits, edition 2024, async, ensinados pelo codigo real do
plev. cada conceito sai de um arquivo que existe e compila.

### 1.1 ownership e borrow pelo GpuVec e pelos buffers persistentes

rastros (placeholder):
- adr: render-on-demand-requires-explicit-invalidation
- crate: crates/engine/src/gpu (vec, GpuVec)
- diff/commit: pendente
- bench: engine/scene_build.rs (push_rects)

### 1.2 traits como contrato, View, Lifecycle, Interpolate

rastros (placeholder):
- adr: view-trait-design, component-design
- crate: crates/engine/src/view, crates/engine/src/component
- diff/commit: pendente
- bench: n/a

### 1.3 edition 2024 na pratica, #[unsafe(no_mangle)] e o que mudou

rastros (placeholder):
- adr: workspace-engine-at-root-libs-in-crates-demos-in-examples
- crate: crates/showcase/src/app.rs (android_main, showcase_ios_main)
- diff/commit: pendente
- bench: n/a

### 1.4 erro como valor, Result e o caminho da limpeza de lint

rastros (placeholder):
- adr: error-handling-lint-cleanup, clippy-zero-warnings
- crate: workspace inteiro
- diff/commit: pendente
- bench: n/a

### 1.5 modularizar por responsabilidade, o limite das 300 linhas

rastros (placeholder):
- adr: adr-003-srp-modularization, srp-modularization
- crate: crates/engine/src (44 monolitos divididos)
- diff/commit: pendente
- bench: n/a

### 1.6 workspace virtual, a crate como fronteira

rastros (placeholder):
- adr: workspace-engine-at-root-libs-in-crates-demos-in-examples
- crate: Cargo.toml (virtual), crates/engine, crates/rope, crates/git
- diff/commit: pendente (commit e6f7091, scene3d/snake para examples)
- bench: n/a

### 1.7 async sem runtime pesado, init da gpu e eventloopproxy

rastros (placeholder):
- adr: async-gpu-init-and-single-wasm-entry
- crate: crates/engine/src/window, crates/engine/src/gpu
- diff/commit: pendente
- bench: n/a

---

## parte 2, a engine, por dentro (120-200)

gpu, compositor, text, layout, input, animation, signal, perf, view, builder.
um capitulo por subsistema, ancorado nos adrs que registraram cada decisao.

### 2.1 gpu, device, surface, pipelines, GpuVec

rastros (placeholder):
- adr: render-into-an-srgb-view-format
- crate: crates/engine/src/gpu
- diff/commit: pendente
- bench: engine/scene_build.rs (push_rects, 159-222m rects/s)

### 2.2 cor, srgb e linearizar uma vez na entrada

rastros (placeholder):
- adr: linearize-colors-before-the-gpu, render-into-an-srgb-view-format
- crate: crates/engine/src/gpu, crates/engine/src/theme/hoff.rs
- diff/commit: pendente
- bench: n/a

### 2.3 compositor e camadas, dirty tracking e premultiplied alpha

rastros (placeholder):
- adr: layer-system, render-on-demand-requires-explicit-invalidation
- crate: crates/engine/src/compositor
- diff/commit: pendente
- bench: engine (dirty tracking, 3.3us / 1000 layers)

### 2.4 effects, blur, shadow e o composite pass

rastros (placeholder):
- adr: effects-architecture
- crate: crates/engine/src/gpu (shaders), compositor
- diff/commit: pendente
- bench: n/a

### 2.5 text, shaping, atlas de glifos, uma TextStyle por run

rastros (placeholder):
- adr: one-text-style-for-measurement-and-drawing
- crate: crates/engine/src/text
- diff/commit: pendente
- bench: n/a

### 2.6 fontes, embed de cada peso em uso

rastros (placeholder):
- adr: embed-every-font-weight-in-use
- crate: crates/engine/src/text
- diff/commit: pendente
- bench: n/a

### 2.7 layout, taffy e a geometria que deriva do espaco

rastros (placeholder):
- adr: layout-engine, content-driven-layout-not-fixed-constants
- crate: crates/engine/src/layout
- diff/commit: pendente
- bench: layout (<1ms / 1000 nodes release)

### 2.8 tokens medidos e a projecao hidpi

rastros (placeholder):
- adr: measured-design-tokens-over-eyeballed-values, adr-004-hidpi-projection
- crate: crates/engine/src/theme/hoff.rs
- diff/commit: pendente
- bench: n/a

### 2.9 input, fila de eventos e hit-testing

rastros (placeholder):
- adr: input-system-design
- crate: crates/engine/src/input
- diff/commit: pendente
- bench: n/a

### 2.10 touch e gesto, o pointer sintetizado

rastros (placeholder):
- adr: touch-as-synthesized-pointer-events, touch-gesture-design
- crate: crates/engine/src/input
- diff/commit: pendente
- bench: n/a

### 2.11 animation, tween, spring, frameclock

rastros (placeholder):
- adr: animation-pattern-lerp, motion-trails-by-position-history
- crate: crates/engine/src/animation
- diff/commit: pendente
- bench: n/a

### 2.12 signal, o reativo push-pull

rastros (placeholder):
- adr: signal-system-design
- crate: crates/engine/src/signal
- diff/commit: pendente
- bench: engine (signals, 67ns / cycle)

### 2.13 view, component e builder, a arvore declarativa

rastros (placeholder):
- adr: view-trait-design, component-design, dsl-narrate-design
- crate: crates/engine/src/view, component, builder; crates/narrate
- diff/commit: pendente
- bench: n/a

### 2.14 render on demand, a invalidacao explicita

rastros (placeholder):
- adr: render-on-demand-requires-explicit-invalidation
- crate: crates/engine/src/window, compositor
- diff/commit: pendente
- bench: n/a

### 2.15 hot reload, shader e narrate

rastros (placeholder):
- adr: shader-hot-reload-adr, narrate-hot-reload-adr, hot-reload-design
- crate: crates/engine/src/gpu (shaders), crates/narrate
- diff/commit: pendente
- bench: n/a

### 2.16 perf, PerfMonitor e o hud

rastros (placeholder):
- adr: pendente
- crate: crates/engine/src/perf
- diff/commit: pendente
- bench: engine/scene_build.rs

---

## parte 3, um codebase, varios mundos (60-110)

macos/metal, web/webgpu, android, ios, wasm, hidpi, e os asteriscos honestos do
que ainda nao roda.

### 3.1 macos/metal, o alvo nativo

rastros (placeholder):
- adr: responsiveness-multiplatform-and-fidelity
- crate: crates/engine/src/window, gpu
- diff/commit: pendente
- bench: n/a

### 3.2 web/webgpu, init async e a entrada wasm unica

rastros (placeholder):
- adr: async-gpu-init-and-single-wasm-entry
- crate: crates/engine/src/window (wasm canvas), crates/showcase (run_web)
- diff/commit: pendente
- bench: n/a

### 3.3 wasm validation, limits, build guards

rastros (placeholder):
- adr: wasm-webgpu-validation
- crate: crates/showcase, trunk
- diff/commit: pendente
- bench: build release 2.4mb

### 3.4 android, GameActivity e cargo-ndk

rastros (placeholder):
- adr: mobile-specifics
- crate: android/, crates/showcase/src/app.rs (android_main)
- diff/commit: pendente
- bench: n/a

### 3.5 ios, o shell objc, metal e fontes embedded

rastros (placeholder):
- adr: ios-build
- crate: ios/showcase, crates/showcase/src/app.rs (showcase_ios_main)
- diff/commit: pendente
- bench: n/a

### 3.6 mobile specifics, safe areas, IME, scale factor

rastros (placeholder):
- adr: mobile-specifics
- crate: crates/engine/src/platform (mod/ime/lifecycle)
- diff/commit: pendente
- bench: n/a

### 3.7 hidpi, a projecao em displays retina

rastros (placeholder):
- adr: adr-004-hidpi-projection
- crate: crates/engine/src/window (set_projection)
- diff/commit: pendente
- bench: n/a

### 3.8 os asteriscos honestos, o que ainda nao roda

rastros (placeholder):
- adr: android-emulator-deploy, responsiveness-multiplatform-and-fidelity
- crate: android/, ios/showcase
- diff/commit: pendente (linux/windows pendentes)
- bench: n/a

---

## parte 4, o experimento mon (~30, abertas)

lottie para .monster, swf/flash, motion ui, design system universal. escrita em
outro fluxo; aparece aqui so para o mapa ficar inteiro.

### 4.1 importar por conversao, nao por embedding
### 4.2 o formato binario com deltas descobertos
### 4.3 trilhas de movimento por historico de posicao
### 4.4 o transpiler que reporta cada construcao nao mapeada

rastros (placeholder):
- adr: import-foreign-formats-by-conversion-not-embedding,
  binary-animation-format-with-discovered-deltas, monster-format-v0,
  motion-trails-by-position-history, transpiler-reports-every-unmapped-construct
- crate: crates/lot, crates/monster, crates/parser
- diff/commit: pendente
- bench: monster/codec.rs, lot/convert.rs, parser/transpile.rs

---

## parte 5, medir, nao achar (50-90)

benchmarks criterion, notebooks jupyter, o caminho do paper arxiv. um numero por
afirmacao, e cada numero mostra como foi obtido.

### 5.1 por que medir, criterion e harness false

rastros (placeholder):
- adr: benchmark-results
- crate: crates/engine/benches
- diff/commit: pendente
- bench: criterion, m4 mac

### 5.2 rect throughput, o numero de capa

rastros (placeholder):
- adr: benchmark-results
- crate: crates/engine
- diff/commit: pendente
- bench: push_rects, 159-222m rects/s

### 5.3 scene build, o custo de montar a scene

rastros (placeholder):
- adr: benchmark-results
- crate: crates/engine/benches/scene_build.rs
- diff/commit: pendente
- bench: nb-scene-build

### 5.4 dirty tracking, custo por layer

rastros (placeholder):
- adr: benchmark-results, layer-system
- crate: crates/engine
- diff/commit: pendente
- bench: nb-dirty-tracking, 3.3us / 1000 layers

### 5.5 rope edit, build mais insert/delete roundtrip

rastros (placeholder):
- adr: benchmark-results
- crate: crates/rope/benches/edit.rs
- diff/commit: pendente
- bench: nb-rope-edit

### 5.6 tessellation, microssegundos por shape

rastros (placeholder):
- adr: benchmark-results
- crate: crates/engine/src/path (lyon)
- diff/commit: pendente
- bench: nb-tessellation, 1.5-3.7us / shape

### 5.7 signals, nanossegundos por cycle

rastros (placeholder):
- adr: benchmark-results, signal-system-design
- crate: crates/engine/src/signal
- diff/commit: pendente
- bench: nb-signals, 67ns / cycle

### 5.8 codec, convert e transpile, os benches das crates de borda

rastros (placeholder):
- adr: benchmark-results
- crate: crates/monster/benches/codec.rs, crates/lot/benches/convert.rs,
  crates/parser/benches/transpile.rs
- diff/commit: pendente
- bench: nb-monster-codec, nb-lot-convert, nb-parser-transpile

### 5.9 notebooks jupyter, a regra da reprodutibilidade

rastros (placeholder):
- adr: benchmark-results
- crate: kdb/briefing/09-benchmarks.md (hook do benchmark)
- diff/commit: pendente
- bench: kernel limpo, hardware/so/rust/crate declarados

### 5.10 do benchmark ao paper arxiv

rastros (placeholder):
- adr: arxiv-paper-outline, arxiv-paper-draft
- crate: n/a
- diff/commit: pendente
- bench: 11 secoes do outline

---

## parte 6, seo, wasm e desafios (40-80)

descoberta por ai, json-ld @graph, crawlability de wasm, ssr e pre-render. boa
parte das ancoras aqui vive no blog (zola), nao em adr; isso fica marcado no
rastro.

### 6.1 descoberta por ai, nao so por humano

rastros (placeholder):
- adr: pendente (sem adr, ancora no blog)
- crate: kdb/caranguejovermelho/blog
- diff/commit: pendente
- bench: n/a

### 6.2 json-ld @graph, o grafo da serie building plev

rastros (placeholder):
- adr: pendente (sem adr, ancora no blog @graph)
- crate: kdb/caranguejovermelho/blog
- diff/commit: pendente
- bench: n/a

### 6.3 crawlability de wasm, o conteudo que o crawler nao ve

rastros (placeholder):
- adr: wasm-webgpu-validation
- crate: crates/showcase (run_web), trunk
- diff/commit: pendente
- bench: build release 2.4mb

### 6.4 ssr e pre-render, servir html antes da gpu acordar

rastros (placeholder):
- adr: async-gpu-init-and-single-wasm-entry
- crate: crates/engine/src/window (wasm canvas)
- diff/commit: pendente
- bench: n/a

### 6.5 a entrada wasm unica e o custo de bundle

rastros (placeholder):
- adr: async-gpu-init-and-single-wasm-entry
- crate: crates/showcase/src/app.rs, crates/engine (web-entry, android-entry)
- diff/commit: pendente
- bench: build release 2.4mb

---

## parte 7, onde isto se encaixa (40-70)

panorama rust, ecossistema, a lacuna editorial, por que este livro. credita as
fundacoes e explica o que foi feito diferente e por que.

### 7.1 panorama rust gui, areweguiyet com ironia

rastros (placeholder):
- adr: brief-strengths
- crate: kdb/adr/refs (competitors, competitive-positioning)
- diff/commit: pendente
- bench: n/a

### 7.2 makepad, o concorrente mais proximo

rastros (placeholder):
- adr: makepad-gap-analysis, plev-vs-makepad-report
- crate: kdb/adr/refs
- diff/commit: pendente
- bench: n/a

### 7.3 zed/gpui, bevy, flutter como skia para rust

rastros (placeholder):
- adr: brief-strengths
- crate: kdb/adr/refs (competitors)
- diff/commit: pendente
- bench: n/a

### 7.4 linebender, vello, xilem, parley, a abordagem oposta

rastros (placeholder):
- adr: extracted-patterns
- crate: kdb/adr/refs (linebender-ecosystem)
- diff/commit: pendente
- bench: n/a

### 7.5 a decisao text, parley vs cosmic-text

rastros (placeholder):
- adr: parley-vs-cosmic-text
- crate: crates/engine/src/text
- diff/commit: pendente
- bench: n/a

### 7.6 plugins wasm e hot reload, o que foi adiado e por que

rastros (placeholder):
- adr: wasm-plugin-architecture, hot-reload-design
- crate: crates/engine
- diff/commit: pendente
- bench: n/a

### 7.7 a lacuna editorial, por que este livro

rastros (placeholder):
- adr: brief-strengths, extracted-patterns
- crate: kdb/caranguejovermelho
- diff/commit: pendente
- bench: n/a

---

## apendices (40-90)

pessoas e projetos, glossario, e os indices de diffs, commits e adrs. e o aparato
que torna o livro verificavel paragrafo a paragrafo.

### A.1 pessoas e projetos creditados

rastros (placeholder):
- adr: extracted-patterns
- crate: kdb/adr/refs (competitors, linebender-ecosystem,
  integration-candidates)
- diff/commit: pendente
- bench: n/a

### A.2 glossario tecnico

rastros (placeholder):
- adr: pendente
- crate: doc/arc/arc.yaml, kdb/how-to
- diff/commit: pendente
- bench: n/a

### A.3 indice de diffs e commits

rastros (placeholder):
- adr: changelog
- crate: repo inteiro
- diff/commit: pendente (correlacionar; forgejo perdido)
- bench: n/a

### A.4 indice de adrs

rastros (placeholder):
- adr: index (kdb/adr/index.md), todos os adrs de kdb/adr/
- crate: kdb/adr
- diff/commit: pendente
- bench: n/a

### A.5 changelog

rastros (placeholder):
- adr: changelog
- crate: repo inteiro
- diff/commit: pendente
- bench: n/a
