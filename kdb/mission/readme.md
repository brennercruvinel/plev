---
project: plev
audience: [ai-agents, contributors]
status: active
last-updated: 2026-03-13
domain: project-status
---

# plev, estado do projeto

## o que e
compositing engine GPU-first em rust. um codebase, seis targets (macos/metal, ios/metal, linux/vulkan, android/vulkan, windows/dx12, browser/webgpu). nao e framework de widgets. e a camada que transforma scene graphs em draw calls na GPU de forma identica em todos os targets.

## estado atual (2026-03-13)
- v0.3 funcional: 34 tasks concluidas + task-42 em andamento, 404 testes, ~15.000 LOC core + ~3.800 LOC gitbutler-plev, 20 examples
- build nativo funcionando no macos com metal
- fase 0 (fundamental) completa, 14 subsistemas core
- fase 1 (integracao end-to-end) completa, DSL + builder + effects + input integrados
- fase 2 (mobile/WASM) parcial, android emulador ok, ios check ok, WASM build ok
- fase 3 (polish) parcial, docs, DX, CI/CD feitos
- fase 4 (proof of life) completa, animation + editable text + todo app demo + vector paths + accessibility
- fase 4b (quick wins) completa, spring solver, signal hardening, animation enhancements, event batching
- fase 5 (paper) parcial, benchmarks feitos, outline criado
- research completo, 50+ repos analisados, 12 docs, technology radar, pattern extraction

---

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
| task-18 | android build & device test | parcial (.so ok, APK wrapper + deploy + lifecycle tests pendentes) |
| task-19 | ios simulator test | parcial (check ok, link pendente xcode) |
| task-20 | WASM visual validation | parcial (build 2.4mb ok, visual test pendente) |

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
| task-42 | gitbutler plev port (tauri/svelte -> rust GPU-native) | em andamento, phases 0-3 done (~3800 LOC), phase 4 parcial (dispatch+overlay core done, context menu/modal render pendente). crate em `experiment/gitbutler-plev/` |

### research
| task | descricao | status |
|------|-----------|--------|
| task-ref | research briefing (50+ repos) | done, 12 docs em refs/, technology radar |

---

## proximo passo

### prioridade 1, release publica
- task-41: license + cargo.toml fields + readme screenshot, blocker para qualquer publicacao
- task-40: texto completo do paper arxiv (outline pronto, benchmarks prontos)

### prioridade 2, arquitetura
- task-39: component memoization via partialeq (b7) + dirty flag bubbling (b8)
- c3: stateanimator ja implementado (task-37), animationstate<s, t>

### prioridade 3, mobile/WASM
- task-18: APK wrapper + deploy + lifecycle tests (bloqueado: APK wrapper ainda pendente)
- task-19: ios link (bloqueado: requer xcode.app, nao apenas cli tools)
- task-20: WASM visual validation, pode ser desbloqueado agora com `trunk serve`

### prioridade 4, futuro
- text migration: parley quando atingir 0.9+ (task-32 recomenda wait)
- plugin system: extism quando API estabilizar (task-33 recomenda wait)

---

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
- [x] android .so build (cargo-ndk, GPU host), task-18 parcial (deploy pendente)
- [x] ios check (aarch64-apple-ios-sim), task-19 parcial
- [x] WASM trunk build release (2.4mb), task-20 parcial
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
analise de ecossistema com 50+ repos em 12 documentos, ver `mission/knowledge/refs/index.md`

novas tasks derivadas: task-30 (a11y), task-31 (lyon), task-32 (parley assessment), task-33 (WASM plugins)

sintese: `refs/integration-candidates.md` (adopt/evaluate/watch/hold), `refs/technology-radar.md`

## decisoes tecnicas chave
ver `knowledge/index.md`
