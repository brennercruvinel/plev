---
project: plev
audience: [ai-agents, contributors]
status: active
last-updated: 2026-06-29
domain: index
---

# knowledge base, plev

indice dos architecture decision records do plev. cada linha aponta para o arquivo, com status e data reais do frontmatter. a ordem e cronologica de proposito: o git so preservou de 2026-06-10 em diante (expurgo de historico antes do primeiro push), entao este indice e a fonte da linha do tempo que o git perdeu. fundacao em 2021, ondas seguintes ano a ano ate 2026.

status usados: `reference` (conhecimento congelado de uma sessao), `accepted` (decisao em vigor), `draft-v0` (spec em rascunho), `living` (atualizado continuamente), `active` (este indice).

## fundacao, fase 0 (2021)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2021-03-06 | [view-trait-design](view-trait-design.md) | reference | viewcontext sem ref ao compositor, views retornam vec<scenenode>, &mut para extensao futura |
| 2021-03-24 | [component-design](component-design.md) | reference | lifecycle trait separada de view, component<l> com estado generico, campos disjuntos para o borrow checker |
| 2021-04-15 | [layout-engine](layout-engine.md) | reference | taffy 0.9 wrappado, two-phase rendering, computedbounds absolutos, <1ms/1000 nodes em release |
| 2021-05-02 | [signal-system-design](signal-system-design.md) | reference | push-pull hibrido, slotmap runtime, rc closures para borrow safety, comparefn type-erased para memos |
| 2021-05-27 | [input-system-design](input-system-design.md) | reference | event queue (nao closures), hit-testing linear reverso, click-to-focus, modifierstate proprio |
| 2021-06-16 | [layer-system](layer-system.md) | reference | premultiplied alpha, per-layer dirty tracking, offscreen textures, composite pass, gpuvec compartilhado |
| 2021-07-04 | [effects-architecture](effects-architecture.md) | reference | blur/shadow/composite fragment-only, texturepool grow-only, 13-tap gaussian, premultiplied alpha |
| 2021-07-26 | [dsl-narrate-design](dsl-narrate-design.md) | reference | gramatica hibrida verbal, disambiguacao no parser, modifier categories, format interpolation, builder stubs |
| 2021-08-15 | [ios-build](ios-build.md) | reference | font loading embedded, metal via primary, lifecycle handlers, requisitos de info.plist, script de simulador |
| 2021-08-30 | [mobile-specifics](mobile-specifics.md) | reference | safe area insets (#[cfg] android), IME state machine, lifecycle callbacks, heuristica de teclado, scale factor |
| 2021-09-17 | [wasm-webgpu-validation](wasm-webgpu-validation.md) | reference | eventloopproxy para GPU init async, limits default vs webgl2, build guards |

## mobile e animacao inicial (2022)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2022-01-19 | [android-emulator-deploy](android-emulator-deploy.md) | reference | gradle + cargo-ndk, gameactivity 2.0.2, GPU host obrigatorio (swiftshader trava), showcase renderizando |
| 2022-02-24 | [animation-pattern-lerp](animation-pattern-lerp.md) | reference | frame-based exponential lerp, pixel-snap com round(), font size fixo evita jitter de re-shaping |

## pesquisa, benchmarks e competitivo (2022 a 2023)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2022-04-26 | [benchmark-results](benchmark-results.md) | reference | criterion m4 mac: push_rects 159-222m/s, dirty tracking 3.3us/1000, tessellation 1.5-3.7us/shape, signals 67ns/cycle |
| 2022-05-24 | [arxiv-paper-outline](arxiv-paper-outline.md) | reference | 11 secoes: architecture, cross-platform, DSL, accessibility, vector paths, evaluation |
| 2022-07-06 | [extracted-patterns](extracted-patterns.md) | reference | 38 patterns de 17 repos em 6 fases, top 10 priorizado com LOC, blueprints para task-30/31 |
| 2022-08-11 | [parley-vs-cosmic-text](parley-vs-cosmic-text.md) | reference | comparacao factual: cursor API, selection geometry, inlinebox, harfrust, WASM, custo. recomendacao: wait |
| 2022-09-08 | [wasm-plugin-architecture](wasm-plugin-architecture.md) | reference | wasmtime vs wasmer vs extism, host function interface draft, 10-15ns/call. recomendacao: wait (p4) |
| 2022-10-27 | [touch-gesture-design](touch-gesture-design.md) | reference | 6-state gesturerecognizer, touchtracker fxhashmap, explicit instant para testabilidade, separado do mouse |
| 2022-11-19 | [makepad-gap-analysis](makepad-gap-analysis.md) | reference | inventario de features visuais do makepad vs plev, 9 categorias, gaps e prioridades |
| 2022-12-10 | [plev-vs-makepad-report](plev-vs-makepad-report.md) | reference | relatorio detalhado plev vs makepad: core, learnings, philosophy, priority roadmap |
| 2023-01-14 | [brief-strengths](brief-strengths.md) | reference | bullets dos diferenciais tecnicos: dirty tracking, text atlas, signal system, spring solver, SDF |
| 2023-02-09 | [arxiv-paper-draft](arxiv-paper-draft.md) | reference | rascunho do paper: abstract + 2 secoes, motivacao, gap analysis |

## hot reload (2024)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2024-01-22 | [hot-reload-design](hot-reload-design.md) | reference | 7 implementacoes estudadas, 3 tiers: shader (vello), DSL (makepad), rust (subsecond). dylib descartado (typeid UB) |
| 2024-02-06 | [shader-hot-reload-adr](shader-hot-reload-adr.md) | reference | shader reload via notify watcher + pipeline recreation, errorscopeguard, channel polling |
| 2024-02-18 | [narrate-hot-reload-adr](narrate-hot-reload-adr.md) | reference | narrate DSL reload via runtime parser + override map, re-parse on access |

## lint e modularizacao (2024)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2024-04-14 | [error-handling-lint-cleanup](error-handling-lint-cleanup.md) | reference | pleverror como tipo de erro unificado, setup_wasm_canvas(), consolidacao de #[allow(dead_code)] |
| 2024-05-21 | [adr-003-srp-modularization](adr-003-srp-modularization.md) | reference | limite de 300 linhas/arquivo, 44 monolitos em submodulos, API publica inalterada, 271 .rs total |
| 2024-05-23 | [srp-modularization](srp-modularization.md) | reference | mesma decisao do adr-003 com detalhes de sessao: padrao de divisao, armadilhas, metricas antes/depois |
| 2024-06-17 | [adr-004-hidpi-projection](adr-004-hidpi-projection.md) | reference | set_projection() para apps com layout em logical pixels em displays retina/hidpi |
| 2024-07-09 | [clippy-zero-warnings](clippy-zero-warnings.md) | reference | 107 warnings resolvidos em 12 categorias: float precision, collapsible if, default impls, too many args |

## onda responsividade, web e fidelidade (2024 a 2025)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2024-09-18 | [content-driven-layout-not-fixed-constants](content-driven-layout-not-fixed-constants.md) | accepted | geometria do container deriva do espaco disponivel, nunca de constantes; formula de grid; min/max so como limites |
| 2024-10-07 | [render-on-demand-requires-explicit-invalidation](render-on-demand-requires-explicit-invalidation.md) | accepted | event handler que muda estado visual deve retornar true; invalidacao e contrato de correcao, nao otimizacao |
| 2024-10-29 | [one-text-style-for-measurement-and-drawing](one-text-style-for-measurement-and-drawing.md) | accepted | uma TextStyle por run, compartilhada por measure_styled e textnodekey; heuristica chars*0.58 deletada |
| 2024-11-20 | [linearize-colors-before-the-gpu](linearize-colors-before-the-gpu.md) | accepted | cores sRGB linearizadas: to_linear_array() para clears/uniforms, vertex colors no shader; bug #303030 |
| 2024-12-04 | [render-into-an-srgb-view-format](render-into-an-srgb-view-format.md) | accepted | view sRGB quando a surface nao pode ser sRGB; surface_render_view(); web bate (48,48,48) com o desktop |
| 2024-12-19 | [measured-design-tokens-over-eyeballed-values](measured-design-tokens-over-eyeballed-values.md) | accepted | token entra no tema so com medicao do render vivo, nao de stylesheet nem de olho; graphite #303030 pinned por teste |
| 2025-01-08 | [embed-every-font-weight-in-use](embed-every-font-weight-in-use.md) | accepted | rubik 400/500/600/700 embutidos, familias default pinadas, determinismo |
| 2025-01-27 | [touch-as-synthesized-pointer-events](touch-as-synthesized-pointer-events.md) | accepted | touch sintetiza pointer events no caminho de mouse existente, sem vocabulario paralelo de evento |
| 2025-02-10 | [async-gpu-init-and-single-wasm-entry](async-gpu-init-and-single-wasm-entry.md) | accepted | spawn_local + eventloopproxy para GPU init async, feature web-entry, canvas 100vw/100vh |
| 2025-02-19 | [responsiveness-multiplatform-and-fidelity](responsiveness-multiplatform-and-fidelity.md) | reference | registro de sessao: text spill, distribuicao responsiva, gamma do browser, touch, licoes |

## onda formatos: monster, lot, parser (2025)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2025-03-26 | [monster-format-v0](monster-format-v0.md) | draft-v0 | spec do formato: i-frames (keyframes), delta ops (place/modify/replace/remove), tabela de easing, secoes com sha256 |
| 2025-05-14 | [binary-animation-format-with-discovered-deltas](binary-animation-format-with-discovered-deltas.md) | accepted | a decisao do .monster: h264 para vetores, seek O(1), delta descoberto, no que nao muda custa zero bytes |
| 2025-06-10 | [import-foreign-formats-by-conversion-not-embedding](import-foreign-formats-by-conversion-not-embedding.md) | accepted | lot le o json do lottie uma vez offline e converte para .monster; nenhum runtime estrangeiro embarcado |
| 2025-07-08 | [transpiler-reports-every-unmapped-construct](transpiler-reports-every-unmapped-construct.md) | accepted | o parser emite codigo + droplist com file:line, contagens congeladas em teste, nunca dropa em silencio |

## organizacao do workspace (2025)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2025-08-18 | [workspace-engine-at-root-libs-in-crates-demos-in-examples](workspace-engine-at-root-libs-in-crates-demos-in-examples.md) | accepted | tres tiers: engine na raiz, libs e apps em crates/, demos em examples/; workspace.package/dependencies/lints; shaders em src/gpu/shaders |

## demos (2025)

| data | adr | status | resumo |
|------|-----|--------|--------|
| 2025-09-24 | [motion-trails-by-position-history](motion-trails-by-position-history.md) | accepted | trails de particula via ring buffer de posicoes, nao acumulacao de framebuffer; cena segue limpa por frame (prime) |

## documentos vivos

| arquivo | status | resumo |
|---------|--------|--------|
| [changelog](changelog.md) | living | log de versoes commit a commit: prime creatures, workspace tiers, kdb consolidado, qa pass, ponte monster |

## notas

- duplicados conscientes: [adr-003-srp-modularization](adr-003-srp-modularization.md) (decisao formal) e [srp-modularization](srp-modularization.md) (detalhes de sessao) cobrem o mesmo limite de 300 linhas por angulos diferentes. o trio de hot reload ([hot-reload-design](hot-reload-design.md) e a pesquisa, [shader-hot-reload-adr](shader-hot-reload-adr.md) e [narrate-hot-reload-adr](narrate-hot-reload-adr.md) sao as duas decisoes).
- idioma: ADRs ate meados de 2024 estao em portugues, do fim de 2024 em diante em ingles, casando com o idioma do codigo e dos commits da respectiva onda. mantido de proposito.
- a spec do monster ([monster-format-v0](monster-format-v0.md)) e o registro tecnico do formato; a decisao por tras dela esta em [binary-animation-format-with-discovered-deltas](binary-animation-format-with-discovered-deltas.md).
