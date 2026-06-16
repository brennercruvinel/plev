---
project: plev
audience: [ai-agents, contributors]
status: active
last-updated: 2026-04-05
domain: index
---

# knowledge base, plev

| data | tema | arquivo | resumo |
|------|------|---------|--------|
| 2026-03-08 | view trait design | `view-trait-design.md` | viewcontext sem ref ao compositor, views retornam vec<scenenode>, &mut para extensão futura |
| 2026-03-08 | WASM/webgpu validation | `wasm-webgpu-validation.md` | eventloopproxy pattern para GPU init async, limits default vs webgl2, build guards |
| 2026-03-08 | layout engine | `layout-engine.md` | taffy 0.9 wrappado, two-phase rendering, computedbounds absolutos, <1ms/1000 nodes release |
| 2026-03-08 | input system design | `input-system-design.md` | event queue (não closures), hit-testing linear reverso, click-to-focus, modifierstate próprio |
| 2026-03-08 | component design | `component-design.md` | lifecycle trait separada de view, component<l> com estado genérico, campos disjuntos para borrow checker |
| 2026-03-08 | signal system design | `signal-system-design.md` | push-pull hybrid, slotmap runtime, rc closures para borrow safety, comparefn type-erased para memos |
| 2026-03-08 | layer system | `layer-system.md` | premultiplied alpha, per-layer dirty tracking, offscreen textures, composite pass full-screen triangle, gpuvec compartilhado |
| 2026-03-08 | effects architecture | `effects-architecture.md` | fragment-only blur/shadow/composite, texturepool grow-only, 13-tap gaussian, premultiplied alpha |
| 2026-03-08 | touch & gesture design | `touch-gesture-design.md` | 6-state gesturerecognizer, touchtracker fxhashmap, explicit instant for testability, separate from mouse input |
| 2026-03-08 | ios build | `ios-build.md` | font loading embedded, metal via primary, lifecycle handlers, info.plist reqs, simulator script |
| 2026-03-08 | mobile specifics | `mobile-specifics.md` | safe area insets (#[cfg] android), IME state machine, lifecycle callbacks, keyboard height heurística, scale factor |
| 2026-03-08 | DSL narrate design | `dsl-narrate-design.md` | gramatica hibrida verbal, disambiguacao parser, modifier categories, format interpolation, builder stubs |
| 2026-03-08 | integration phase | (inline) | task-15: intof32/intoradius traits, text content merge, plev_narrate bridge. task-16: layereffect enum, effectprocessor no render loop. task-17: hitregion layer_visible/layer_opacity, set_current_layer API |
| 2026-03-08 | levenshtein dx | (inline) | levenshtein distance em keywords.rs para sugestoes de typo. threshold: <=1 para palavras curtas, <=2 para longas. 4 const arrays como source of truth |
| 2026-03-08 | android cross-compile | (inline) | NDK 27.2, cargo-ndk v4.1.2, #[unsafe(no_mangle)] requerido em edition 2024, game-activity feature necessaria |
| 2026-03-08 | ios link issue | (inline) | cargo check ok mas cargo build falha sem xcode.app (uikit framework nao encontrado com cli tools only) |
| 2026-03-09 | android emulator deploy | `android-emulator-deploy.md` | gradle + cargo-ndk, gameactivity 2.0.2, GPU host obrigatório (swiftshader trava), showcase renderizando |
| 2026-03-09 | animation pattern (pre-task-27) | `animation-pattern-lerp.md` | frame-based exponential lerp, pixel-snap com round(), font size fixo evita re-shaping jitter |
| 2026-03-11 | animation system (task-27) | (inline) | frameclock + web_time::instant, 31 easing variants, interpolate trait, tween<t> + spring<t>, 35 testes. web-time wraps performance.now() no WASM |
| 2026-03-11 | editable text (task-28) | (inline) | textbuffer com cursor char-aware, selection, cursor_to_x/x_to_cursor aprox (font_size*0.6), textinput component com blink 530ms, IME bridge desacoplada, 44 testes |
| 2026-03-11 | todo app demo (task-29) | (inline) | proof of life: textinput + tween + compositor juntos. ~530 LOC. add/toggle/delete/filter todos. fade-in animation ao adicionar |
| 2026-03-11 | ref: competitors | `refs/competitors.md` | makepad (GPU-first), dioxus (35k stars, fullstack), leptos (signals referencia), yew (vdom legacy), slint (embedded DSL), ribir (wgpu pre-alpha), compose mp (skia/kotlin) |
| 2026-03-11 | ref: linebender ecosystem | `refs/linebender-ecosystem.md` | xilem, vello, velato, parley, kurbo, peniko, fearless-simd |
| 2026-03-11 | ref: accessibility | `refs/accessibility.md` | accesskit analise profunda, mapeamento para plev |
| 2026-03-11 | ref: animation/motion | `refs/animation-motion.md` | dotlottie-rs, rive, natura, keyframe, mina, easing, interpolation |
| 2026-03-11 | ref: math/physics/geometry | `refs/math-physics-geometry.md` | glam, rapier, nalgebra, lyon |
| 2026-03-11 | ref: WASM tooling | `refs/wasm-tooling.md` | trunk, wasm-pack, extism, serde-wasm-bindgen, spin, lunatic, workers-rs |
| 2026-03-11 | ref: charts/visualization | `refs/charts-visualization.md` | plotters, charming, egui_graphs |
| 2026-03-11 | ref: tui patterns | `refs/tui-patterns.md` | padroes UX de 13+ tui apps |
| 2026-03-11 | ref: emulators/WASM runtimes | `refs/emulators-wasm-runtimes.md` | wasmboy, waforth, chasm, wizard-engine, pywasm |
| 2026-03-11 | ref: competitive positioning | `refs/competitive-positioning.md` | matriz comparativa atualizada, posicionamento |
| 2026-03-11 | ref: integration candidates | `refs/integration-candidates.md` | ranking: adopt / evaluate / watch / hold |
| 2026-03-11 | ref: technology radar | `refs/technology-radar.md` | resumo executivo 1 pagina |
| 2026-03-11 | repos clonados (bunker) | (externo) `bunker/databasematrix/projects/plev-refs.md` | 56 repos clonados em bunker/repos/, mapeados por categoria |
| 2026-03-11 | ref: pattern extraction (parley/lyon/glam) | `refs/pattern-extraction-parley-lyon-glam.md` | 7 patterns extraidos: byte-index cursor+affinity, selection geometry callback, editordriver, geometrybuilder trait, lyon+wgpu vertex layout, glam vec2 pod, inlinebox |
| 2026-03-11 | task-34 extraction: fase e+f | `refs/extraction-fase-ef.md` | 7 patterns: waforth shared-table WASM interop, extism plugin lifecycle, leptos fxindexset+3-state push-pull, dioxus peek/drop-guard, slint lazy-eval+constant-sentinel, leptos RAII observer, dioxus/vello delegation |
| 2026-03-11 | task-34 extraction: fase d (tui apps) | `refs/pattern-extraction-tui-apps.md` | 11 patterns from yazi/television/bottom: event batching+throttle, partial render flag, layer action routing, priority scheduler, plugin isolation, channel abstraction, render gating, mode transitions, auto-navigation, widget maximize, configurable update rate |
| 2026-03-11 | **task-34 extracted patterns (master)** | `extracted-patterns.md` | **38 patterns de 17 repos em 6 fases.** top 10 priorizado com LOC estimado. blueprints para task-30 (a11y), task-31 (lyon), signal.rs, animation.rs |
| 2026-03-12 | scene cache per-component | (inline component.rs) | implementado: `cached_nodes`, `needs_render`, `invalidate()` em `Component<L>`. state_mut() seta needs_render=true automaticamente. pendente: b7 memoizacao partialeq, b8 dirty flag bubbling (task-39) |
| 2026-03-11 | **parley vs cosmic-text** | `parley-vs-cosmic-text.md` | comparacao factual: cursor API, selection geometry, inlinebox, harfrust (ambos usam), WASM, estabilidade, custo migracao. recomendacao: wait (nao migrar agora) |
| 2026-03-11 | **benchmark results** | `benchmark-results.md` | criterion benchmarks m4 mac: push_rects 159-222m/s, dirty tracking 3.3us/1000, tessellation 1.5-3.7us/shape, signals 67ns/cycle |
| 2026-03-11 | **arxiv paper outline** | `arxiv-paper-outline.md` | 11 secoes: architecture, cross-platform, DSL, accessibility, vector paths, evaluation. abstract com metricas |
| 2026-03-11 | **WASM plugin architecture** | `wasm-plugin-architecture.md` | wasmtime vs wasmer vs extism. host function interface draft. 10-15ns/call overhead. recomendacao: wait (p4) |
| 2026-03-12 | **plev vs makepad report** | `plev-vs-makepad-report.md` | comparacao detalhada plev vs makepad v1.0 |
| 2026-03-13 | **brief strengths** | `brief-strengths.md` | bullet points de diferenciais tecnicos do plev |
| 2026-03-13 | **arxiv paper draft** | `arxiv-paper-draft.md` | rascunho do paper (abstract + 2 secoes, motivacao, gap analysis) |
| 2026-03-13 | **makepad gap analysis** | `makepad-gap-analysis.md` | inventario completo makepad visual features vs plev. 8 categorias, gaps documentados, prioridades |
| 2026-03-22 | **hot reload design** | `hot-reload-design.md` | 7 implementacoes estudadas (subsecond, makepad, vello, rerun, leptos, dioxus, hot-lib-reloader). 3 tiers: shader reload (vello pattern), DSL reload (makepad pattern), rust code (subsecond). boundary analysis: scenenode cruza, compositor nao. dylib descartado (typeid ub). |
| 2026-03-22 | **shader hot reload adr** | `shader-hot-reload-adr.md` | adr: decisao de implementar shader hot reload via notify watcher + pipeline recreation |
| 2026-03-22 | **narrate hot reload adr** | `narrate-hot-reload-adr.md` | adr: decisao de implementar narrate DSL hot reload via file watcher + override map |
| 2026-04-05 | **adr-003 SRP modularization** | `adr/adr-003-srp-modularization.md` | limite 300 linhas/arquivo, 44 monolitos convertidos em submodulos, API publica inalterada |
| 2026-04-05 | **SRP modularization session** | `srp-modularization.md` | padrao de divisao, armadilhas (worktree untracked, pub(crate) breakage, agentes paralelos), metricas antes/depois |
| 2026-04-05 | **clippy zero warnings** | `clippy-zero-warnings.md` | 107 warnings resolvidos em 12 categorias: float precision, collapsible if, default impls, too many args, complex types |
| 2026-04-05 | **adr-004 hidpi projection** | `adr/adr-004-hidpi-projection.md` | set_projection() para apps com layout logical-pixel em displays retina/hidpi |
