---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: research
---

# integration candidates, ranking

**data:** 2026-03-11
**baseado em:** todas as analises em refs/

---

## metodologia

cada lib avaliada em 4 dimensoes:
- **maturidade:** versao, downloads, estabilidade API
- **fit arquitetural:** compatibilidade com wgpu 28, scene graph, premultiplied alpha, 6 targets
- **custo de integracao:** LOC estimado, breaking changes, deps transitivas
- **valor:** problema que resolve, alternativas

categorias:
- **adopt**, usar na proxima task relevante
- **evaluate**, prototipar/testar antes de decidir
- **watch**, acompanhar, nao integrar agora
- **hold**, nao usar (overkill, incompativel, ou imaturo)

---

## adopt (usar agora)

### web-time, tempo cross-platform
- **versao:** crate estavel, wrapper fino sobre std::time (native) e performance.now() (WASM)
- **quando:** task-27 (animation system)
- **custo:** ~5 LOC de mudanca (substituir std::time::instant por web_time::instant)
- **justificativa:** std::time::instant causa panic em wasm32. web-time e a solucao padrao do ecossistema

### easing patterns (implementar internamente)
- **quando:** task-27 (animation system)
- **custo:** ~200 LOC (30 penner easings + lerp + cubic bezier)
- **justificativa:** todos os crates de easing (easing, interpolation, keyframe) convergem no mesmo pattern: `trait Interpolate + enum Easing + fn ease(t, easing) -> f32`. implementar internamente e trivial e evita dependencia para <200 linhas
- **referencia:** keyframe (cantween trait), interpolation (lerp trait), easing (30 funcoes penner)

### accesskit tree model (design)
- **versao:** accesskit 0.24.0, accesskit_winit 0.32.2 (compativel winit 0.30)
- **quando:** task-30 (accessibility)
- **custo:** ~700-900 LOC, 2 crates no cargo.toml
- **justificativa:** unica solucao viavel para accessibility em rust custom-rendered UI. 11m+ downloads, usado por egui/slint/bevy
- **nota:** ios e WASM sem adapter ainda, macos/linux/windows/android cobertos

---

## evaluate (prototipar antes de decidir)

### parley, text layout
- **versao:** 0.7.0, 521k downloads
- **quando:** task-32 (apos task-28 completar)
- **custo:** ~500-800 LOC de migracao em text.rs, breaking changes significativas
- **justificativa:** stack 100% rust (harfrust, skrifa, icu4x), apis de cursor/selection superiores, inline boxes. egui e slint migrando para parley. cosmic-text pode ficar para tras
- **risco:** API ainda em evolucao (0.x), menos battle-tested que cosmic-text. recomendacao: implementar via feature flag, comparar quantitativamente
- **referencia:** refs/linebender-ecosystem.md secao 3

### lyon, path tessellation
- **versao:** 1.0.16, 3.3m downloads, estavel
- **quando:** task-31 (vector paths)
- **custo:** ~600-800 LOC, 1 crate + novo shader + novo scenenode variant
- **justificativa:** path -> triangles para GPU e exatamente o que φ precisa para shapes customizados. exemplo oficial wgpu existe. integra naturalmente com gpuvec
- **risco:** baixo, API estavel (1.x), pattern de integracao documentado

### glam, SIMD math
- **versao:** 0.32.1, 38.6m downloads
- **quando:** qualquer momento (baixo custo)
- **custo:** ~100 LOC de substituicao de [f32; 2/4] por vec2/vec4, bytemuck feature para GPU upload
- **justificativa:** SIMD automatico (sse2/NEON/wasm-simd128), sem generics (inline agressivo), usado por bevy e ecossistema game-dev inteiro
- **risco:** muito baixo, drop-in replacement para math existente

---

## watch (acompanhar, nao integrar)

### vello, 2d compute renderer
- **porque watch:** arquitetura de encoding em streams e inovadora (dirty tracking mais granular), mas requer compute shaders. complexidade enorme para quem so precisa de quads + texto. acompanhar backend hibrido (webgl2 fallback)
- **trigger para reavaliar:** se φ precisar de path rendering complexo alem do que lyon oferece

### dotlottie-rs / rive, animacao lottie/rive
- **porque watch:** dotlottie-rs usa thorvg (c++), rive-rs sem release estavel. ambos resolvem animacao vetorial complexa que φ nao precisa ainda
- **trigger para reavaliar:** se demo app (task-29) precisar de animacoes importadas de design tools

### velato, lottie para vello
- **porque watch:** depende de vello. relevante apenas se φ adotar vello no futuro

### fearless-simd, SIMD portavel
- **porque watch:** prematuro ate task-25 (benchmarks) identificar hot paths CPU-bound
- **trigger para reavaliar:** profiling mostra gargalo em operacao SIMD-eligible

### extism, WASM plugin model
- **porque watch:** referencia para task-33 (plugin architecture). modelo host/plugin com pdks multi-linguagem. component model emergente
- **trigger para reavaliar:** quando task-33 fase 1 iniciar

### kurbo, 2d geometry
- **porque watch:** viria como dependencia transitiva se adotar parley. standalone, menos relevante que lyon para φ
- **trigger para reavaliar:** se task-32 recomendar parley

### serde-wasm-bindgen
- **porque watch:** relevante se φ expor API javascript. menor binario e mais rapido que serde_json para WASM<->JS
- **trigger para reavaliar:** se φ publicar npm package

---

## hold (nao usar)

### nalgebra, algebra linear completa
- **porque hold:** overkill para UI. generics pesados, compile time 2-3x pior que glam, 1.5-3x mais lento em operacoes 3d/4d comuns
- **alternativa:** glam para tudo que φ precisa

### rapier, physics 2d/3d
- **porque hold:** φ nao precisa de physics engine. spring animations sao ~50 LOC de verlet integration manual (referencia: natura crate). rapier traz nalgebra + parry + simba como deps
- **alternativa:** implementar damped harmonic oscillator internamente (~100 LOC, unlicense do natura permite vendor)

### dioxus/leptos/yew como dependencia
- **porque hold:** sao frameworks de aplicacao, nao libs. φ e a camada abaixo. sem ponto de integracao
- **alternativa:** inspirar-se nos patterns (signals, hot-reload) sem depender

### lunatic, WASM runtime erlang-inspired
- **porque hold:** aparentemente abandonado (3 anos sem release). conceito interessante mas risco alto
- **alternativa:** wasmtime ou wasmer para runtime WASM embedavel

---

## resumo visual

```
ADOPT NOW          EVALUATE           WATCH              HOLD
-----------        ----------         ---------          ------
web-time           Parley             Vello              nalgebra
easing (interno)   Lyon               dotlottie-rs       rapier
AccessKit          glam               Rive               Dioxus/Leptos/Yew
                                      fearless-simd      Lunatic
                                      Extism
                                      Kurbo
                                      Velato
                                      serde-wasm-bindgen
```
