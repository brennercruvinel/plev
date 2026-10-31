---
id: mapa-das-fases
title: "plev, estado do projeto"
project: plev
type: registro
status: ativo
date: 2026-09-26
date_basis: cabecalho
topics: [project-status]
relations: []
source: docs/log/README.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# plev, estado do projeto

o plev é uma engine de composição GPU-first em rust que está virando um
runtime de interação generativa: cada controle é também dado (um contrato
semântico), que a pessoa, o leitor de tela e um agente operam pelo mesmo
dispatcher. o que é, para onde vai e o estado de cada task ficam no
programa ([`programa-runtime-generativo.md`](programa-runtime-generativo.md), seção 12); a tese está em
[`tese-runtime-generativo.md`](../pesquisas/tese-runtime-generativo.md), as regras dos agentes em
[`generative-runtime-guidelines.md`](../referencia/operacao/generative-runtime-guidelines.md) e o modo de operação do
orquestrador em [`autonomy.md`](../referencia/operacao/autonomy.md). cada task tem o seu arquivo em
`steps/`, e o estado dela fica no `status:` do cabeçalho (`in-progress`,
`pending`, `done`); as pastas `ongoing/`, `pending/` e `checked/` citadas
no estado abaixo e nos registros antigos viraram esse campo.

este arquivo guarda o mapa histórico das tasks 01 a 56 (de 2021 a
2026-06-29), a fase anterior ao programa. o estado atual vem em resumo
logo abaixo; o detalhe é o do programa.

## estado atual (2026-09-26)

- workspace de 23 crates membros, mais dois workspaces próprios
  (`apps/urnadbapp`, o explorador de `.urna`, e `rust/llmrun`, a inferência
  em processo); wgpu 29, rust 1.87.
- alvos: macOS e web entregues; android e iOS rodam a galeria (tasks 18,
  19 e 20 fechadas em 2026-09-25); linux e windows pendentes.
- abertas em `steps/ongoing/`: task-73 (dependências incrementais, o ADR
  de desenho espera a escolha do brenner), task-74 (texto: falta o roteiro
  de IME num campo), task-75 (accessibility: o roteiro do VoiceOver, as
  cores que falham no contraste e o web), task-78 (AI-native: o app de
  referência) e task-80 (a prova: a demo 2 e o checkpoint c).
- pendente em `steps/pending/`: task-41 (licença e release pública).
- o repositório no GitHub é privado; a licença MIT está em
  `LICENSE`.

## mapa de tasks

### fase 0, fundamental (p0), completa
| task | descricao | status |
|------|-----------|--------|
| task-01 | view trait + viewcontext | done |
| task-02 | component state + lifecycle | done |
| task-03 | layout engine (taffy 0.9) | done |
| task-04 | signal system (reactive) | done |
| task-05 | declarative builder API + #[component] | done |
| task-06 | WASM/webgpu validation | done |
| task-07 | layer system + composite pass | done |
| task-08 | effects (blur, shadow) | done |
| task-09 | input (keyboard, mouse, scroll) | done |
| task-10 | touch + gesture recognition | done |
| task-11 | android build + lifecycle | done |
| task-12 | ios build + lifecycle | done |
| task-13 | mobile specifics (safe areas, IME) | done |
| task-14 | plev_narrate! DSL verbal | done |

### fase 1, integracao end-to-end (p0), completa
| task | descricao | status |
|------|-----------|--------|
| task-15 | pipeline DSL -> builder -> compositor | done |
| task-16 | effects <-> layer integration | done |
| task-17 | input <-> layer hit-testing | done |

### fase 2, mobile/WASM testing (p1), parcial
| task | descricao | status |
|------|-----------|--------|
| task-18 | android build & device test | concluída (2026-09-25): APK no emulador, toque e lifecycle testados; arquivo em `checked/` |
| task-19 | ios simulator test | concluída (2026-09-25): o frame do simulador igual ao desktop fora da barra de status; arquivo em `checked/` |
| task-20 | WASM visual validation | concluída (2026-09-25): 12 capturas web iguais ao desktop em todo byte; arquivo em `checked/` |

### fase 3, polish (p2), parcial
| task | descricao | status |
|------|-----------|--------|
| task-21 | documentacao DSL narrate | done (falta JSX comparison) |
| task-22 | error messages & DX | parcial (did you mean feito) |
| task-23 | CI/CD (github actions) | done (nao testado remotamente) |
| task-24 | cleanup & readme | parcial (readme feito, license TBD) |
| task-30 | accessibility (accesskit) | done (feature-gated, lazy activation, focusgraph, 8 testes) |

### fase 4, proof of life (p2), completa
| task | descricao | status |
|------|-----------|--------|
| task-27 | animation system (easing, tweens, springs) | done (35 testes, ~500 LOC) |
| task-28 | editable text (cursor, input, IME) | done (44 testes, ~800 LOC) |
| task-29 | todo app demo (proof of life) | done (~530 LOC, funcional) |
| task-31 | vector paths (lyon tessellation) | done (~250 LOC, reusa quad pipeline, 11 testes) |
| task-32 | text upgrade assessment (parley vs cosmic-text) | done (research: wait, nao migrar agora) |

### fase 4b, quick wins & bug fixes (p0/p1), completa (derivada de task-34)
| task | descricao | status |
|------|-----------|--------|
| task-35 | fix spring<t>, analytical solver (bug frame-rate) | done, solver analitico 3 regimes, frame-rate independent |
| task-36 | signal hardening (RAII guard, fxindexset, peek) | done, fxindexset o(1), observerguard panic-safe, peek() (f3 sentinel skipped) |
| task-37 | animation enhancements (keyframeseq, repeat, step/hold) | done, keyframesequence, repeat/reverse/delay, step/hold, const-generic |
| task-38 | event batching | done, bufferedevent enum, batch-drain before render |

### fase 5, paper arxiv (p3), parcial
| task | descricao | status |
|------|-----------|--------|
| task-25 | benchmarks comparativos | done (criterion, 6 groups, m4 mac) |
| task-26 | paper arxiv | parcial (outline completo, texto pendente) |

### fase 6, extensibilidade (p4), parcial
| task | descricao | status |
|------|-----------|--------|
| task-33 | WASM plugin architecture | done (fase 1 research: wait, extism quando API estavel) |
| task-34 | exploracao/extracao de ideias (56 repos) | done, 38 patterns de 17 repos, top 10 priorizado |

### fase 7, release publica (p0), pendente
| task | descricao | status |
|------|-----------|--------|
| task-39 | scene cache memoization (b7 + b8) | pendente |
| task-40 | paper arxiv, texto completo | pendente |
| task-41 | license + cargo.toml + release prep | pendente |

### experiment, gitbutler port (proof of real app)
| task | descricao | status |
|------|-----------|--------|
| task-42 | gitbutler plev port (tauri/svelte -> rust GPU-native) | dispatch tipado + overlay foram para a engine; o app evoluiu para o crate `apps/idestudio` (ver task-48). resta polir context menu/modal render |

### research
| task | descricao | status |
|------|-----------|--------|
| task-ref | research briefing (50+ repos) | done, 12 docs em refs/, technology radar |

### fase 8, formatos e interop (2025), completa
| task | descricao | status |
|------|-----------|--------|
| task-43 | monster, codec binario de animacao v0 | done, MON0, seek O(1), delta descoberto, 124 testes |
| task-44 | lot, importer lottie + ponte para .monster | done, conversao offline, subset honesto, zero embedding |
| task-45 | parser, transpiler poc (react/gpui -> builder) | done, droplist file:line, contagens congeladas |

### fase 9, edicao e apps reais (2026), completa
| task | descricao | status |
|------|-----------|--------|
| task-46 | rope, nucleo de edicao (document/transaction/history) | done, estilo helix, 77 testes, headless |
| task-47 | git, backend (gix reads + cli mutations + client threaded) | done, ui nunca bloqueia, 25 testes |
| task-48 | ide, git client nativo em plev | done, workspace/diff/commit, 55 testes (evolui de task-42) |

### fase 10, demos e plataforma (2025), completa
| task | descricao | status |
|------|-----------|--------|
| task-49 | prime number creatures (port entropic life) | done, sim core puro, render fiel, desktop + web |
| task-50 | showcase multiplataforma como lib | done, entries desktop/web/android/ios, apk + simulador |

### fase 11, organizacao profissional (2024/2026), completa
| task | descricao | status |
|------|-----------|--------|
| task-51 | workspace restructure + rebrand phi->plev | done, 3 tiers, renames, cargo workspace |
| task-52 | modularizacao SRP (limite 300 linhas) | done, 44 monolitos divididos, API inalterada, 470 testes |

---

## proximo passo

o próximo passo sai do programa ([`programa-runtime-generativo.md`](programa-runtime-generativo.md), seções
9 e 12) e das tasks abertas em `steps/ongoing/`, não deste mapa. a
release pública é a task-41, em `steps/pending/`.

## marcos
- [x] pipeline de quads com alpha blending
- [x] atlas de glifos com etagere + LRU
- [x] shaping cache com fxhashmap (zero shaping em steady state)
- [x] gpuvec, buffers persistentes com write parcial
- [x] dirty tracking per-layer via fxhasher
- [x] premultiplied alpha em todo o pipeline
- [x] view trait + viewcontext + rectview + textview, task-01
- [x] component state + lifecycle, task-02
- [x] layout engine (taffy flexbox), task-03
- [x] signal system (reactive primitives), task-04
- [x] declarative builder API + #[component] macro, task-05
- [x] WASM/webgpu validation (build 2.4mb, eventloopproxy fix), task-06
- [x] camadas independentes + composite pass, task-07
- [x] effects (blur, shadow, composite), task-08
- [x] input system (teclado, mouse, scroll, hover), task-09
- [x] touch + gesture recognition (6 gestos), task-10
- [x] android build + lifecycle, task-11
- [x] ios build + lifecycle, task-12
- [x] mobile specifics (safe areas, IME, scale factor), task-13
- [x] DSL verbal (plev_narrate!), 66 testes, format interpolation, task-14
- [x] pipeline DSL -> builder -> compositor end-to-end, task-15
- [x] effects integrados com layers, task-16
- [x] input layer-aware hit-testing, task-17
- [x] android APK, deploy no emulador, toque e lifecycle, task-18
- [x] ios build, deploy e captura no simulador, task-19
- [x] WASM trunk build release (2.4mb) e validação visual, task-20
- [x] documentacao DSL (EBNF, 30 modifiers), task-21
- [x] error suggestions (levenshtein), task-22 parcial
- [x] CI/CD (6 jobs), task-23
- [x] readme publico + cargo doc limpo, task-24 parcial
- [x] messagedock example, componente animado com hover, click, expand, pixel-snap
- [x] animation system, 31 easing variants, tween<t>, spring<t>, frameclock, web_time, task-27
- [x] editable text, textbuffer, cursor blink, IME bridge, task-28
- [x] todo app demo, proof of life completo, task-29
- [x] research briefing, 50+ repos, 12 docs, technology radar, 4 novas tasks, task-ref
- [x] pattern extraction, 38 patterns de 17 repos, top 10 priorizado, blueprints para task-30/31, task-34
- [x] quick wins, spring solver, signal hardening, animation enhancements, event batching, task-35/36/37/38
- [x] vector paths, lyon tessellation, quad pipeline reuse, pathbuilder API, task-31
- [x] accessibility, accesskit feature-gated, lazy activation, focusgraph, per-frame treeupdate, task-30
- [x] benchmarks, criterion 6 groups, 159-222m rects/s, 3.3us dirty tracking, task-25
- [x] text assessment, parley vs cosmic-text, recomendacao wait, task-32
- [x] WASM plugins research, wasmtime/wasmer/extism, recomendacao wait, task-33
- [x] arxiv outline, 11 secoes estruturadas, task-26 (parcial)

---

## pesquisa de referencia
analise de ecossistema com 50+ repos, em `docs/ref/`

novas tasks derivadas: task-30 (a11y), task-31 (lyon), task-32 (parley assessment), task-33 (WASM plugins)

sintese: [`integration-candidates.md`](../pesquisas/integration-candidates.md) (adopt/evaluate/watch/hold), [`technology-radar.md`](../pesquisas/technology-radar.md)

## decisoes tecnicas chave
ver [`indice-historico.md`](indice-historico.md)
