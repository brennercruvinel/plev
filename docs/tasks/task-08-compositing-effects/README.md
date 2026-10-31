---
id: task-08
title: "task-08: compositing effects (blur, shadow, opacity)"
project: plev
type: task
status: done
date: 2021-07-06
date_basis: header
topics: [effects, gpu]
relations: [task-07, task-16]
source: [docs/log/steps/task-08.md, docs/log/changes/task-08.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-08: compositing effects (blur, shadow, opacity)

Records the compositing effects on the layers: a separable 13-tap Gaussian blur, drop shadow by silhouette extraction and opacity group, all in fragment shaders, with a grow-only texture pool. Phases a, b, c and d are implemented, with 10 tests passing and the `effects_demo` example. Still open are phase e (configurable effects per layer, depends on task-07) and phase f (blur benchmark at 1080p below 2 ms, resize and WASM validation).

## objetivo
implementar efeitos visuais que operam sobre camadas: gaussian blur, drop shadow, opacity groups. efeitos são aplicados na composição de camadas, não por primitiva.

## contexto
com o layer system (task-07), efeitos são post-processing sobre texturas de camada. blur e shadow são os mais pedidos para UI de produção (backgrounds frosted-glass, elevation via shadow).

**decisão: fragment shader only** (sem compute shaders, conforme rules.md).

## dependências
- task-07 (layer system, integração final, fase e)
- fases a-d são independentes de task-07

## checklist de conclusão
- [x] gaussian blur (separável, two-pass horizontal+vertical)
- [x] drop shadow (silhouette extraction + blur + color)
- [x] opacity group (composite shader com alpha uniform)
- [x] texturepool para texturas temporárias (grow-only, reuso)
- [x] shaders WGSL para blur (fragment shader, 13-tap separável)
- [x] shaders WGSL para shadow (silhouette extraction)
- [x] shaders WGSL para composite (premultiplied alpha blending)
- [x] effectprocessor com apply_blur, apply_shadow, composite_pass
- [x] exemplo funcional: effects_demo com shadow + blur + opacity
- [x] `cargo check --examples` passa sem warnings
- [x] `cargo test` passa (10 testes, 0 falhas)
- [ ] efeitos configuráveis por camada via API (fase e, depende task-07)
- [ ] performance: blur em resolução 1080p < 2ms por frame (fase f)

## arquivos criados
| arquivo | propósito |
|---------|-----------|
| `src/effects.rs` | layereffect enum, effectprocessor, bluruniforms, compositeuniforms, shadowuniforms, gaussian_weights() |
| `src/texture_pool.rs` | texturepool com acquire/release, keyed por (width, height, format) |
| `shaders/blur.wgsl` | gaussian blur separável 13-tap, full-screen triangle |
| `shaders/shadow.wgsl` | extração de silhueta (alpha × shadow color) |
| `shaders/composite.wgsl` | composição com opacity uniform, premultiplied alpha |
| `examples/effects_demo.rs` | demo: quads -> offscreen -> blur + shadow -> composite to surface |

## arquivos modificados
| arquivo | o que muda |
|---------|-----------|
| `src/lib.rs` | +pub mod effects; +pub mod texture_pool; |

## armadilhas
- compute shaders não usados, fragment shader 13-tap é suficiente e portátil para WASM
- blur separável (h+v) é o(n) vs. o(n²) para kernel direto
- shadow: render silhueta -> blur -> composite (não re-renderiza cena inteira)
- texturepool grow-only, texturas nunca destruídas em steady state
- texturehandle armazena clone do textureview para evitar borrow conflicts com pool

## Delivery record

### 2021-07-01, fases a-d implementadas

#### decisões técnicas
- **fragment shader only** (sem compute), portátil para WASM, conforme rules.md
- **13-tap separable gaussian blur**, radius 6, cobre sigma até ~6.0 com qualidade alta
- **texturepool grow-only** (padrão gpuvec), zero alocações em steady state
- **full-screen triangle trick**, 3 vértices via `@builtin(vertex_index)`, sem vbo
- **premultiplied alpha** nos effect passes, composite usa blend `One / OneMinusSrcAlpha`
- **texturehandle com view clonado**, resolve borrow conflict entre handle e pool mutável
- **worktree isolado**, necessário porque outros agentes modificam o working directory principal

#### arquivos criados
- `src/texture_pool.rs`, texturepool com acquire/release
- `src/effects.rs`, effectprocessor, layereffect enum, gaussian_weights()
- `shaders/blur.wgsl`, 13-tap separável via fragment shader
- `shaders/shadow.wgsl`, silhouette extraction
- `shaders/composite.wgsl`, composição com opacity
- `examples/effects_demo.rs`, demo standalone

#### testes
- 6 testes novos (4 gaussian_weights + 2 texture_pool)
- 10 testes total passando

#### pendente
- fase e: integração com task-07 (layer system)
- fase f: resize handling, sigma=0 optimization, multi-pass, WASM validation, benchmark
