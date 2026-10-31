---
id: task-07
title: "task-07: independent layer system"
project: plev
type: task
status: done
date: 2021-06-21
date_basis: header
topics: [rendering, gpu]
relations: [task-01, task-02, task-03, task-04, task-06, task-09]
source: [docs/log/steps/task-07.md, docs/log/changes/task-07.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-07: independent layer system

Records the compositor's layer system: each `Layer` has its own offscreen texture, with z-order, per-layer dirty tracking and an API to create and remove layers. The final composition combines the textures into the framebuffer with premultiplied alpha, and clean layers are no longer redrawn. The delivery closed with 89 tests passing, 7 of them new, and the `layers_demo` example; the checklist marks the static layer with no draw calls after the first frame.

## objetivo
implementar camadas (layers) independentes no compositor. cada camada renderiza para uma textura offscreen propria. camadas estaveis (toolbar, sidebar) nao re-renderizam quando conteudo dinamico muda.

## contexto
hoje o compositor tem uma lista flat de scenenodes. com camadas, grupos de nodes renderizam para texturas separadas que sao compostas no final. isso habilita: caching de camadas estaveis, efeitos por camada (blur, opacity), e z-ordering correto.

## dependencias
- task-01 (view trait, views declaram em qual camada pertencem)

## checklist de conclusao
- [x] `Layer` struct com textura offscreen propria (render target)
- [x] compositor gerencia multiplas camadas com z-order
- [x] dirty tracking por camada, camada limpa = zero re-render
- [x] API para criar/destruir camadas
- [x] composicao final: combinar texturas de todas as camadas no framebuffer
- [x] exemplo funcional: camada de background estatica + camada de conteudo dinamico
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo test` passa (89 testes, incluindo 7 novos de layers)
- [x] performance: camada estatica = zero draw calls apos primeiro frame

## armadilhas
- texturas offscreen consomem vram, reusar e pooling
- composicao de camadas com alpha precisa de blend mode correto (premultiplied alpha)
- nao criar camada por view, camadas sao agrupamentos logicos (poucas por cena)
- wgpu no WASM pode ter limites de texturas simultaneas, testar

## Delivery record

### 2021-06-17

#### inicio da task
- branch `task/TASK-07-layer-system` criada a partir de `master` (pos task-03)
- usou worktree isolada para evitar conflitos com outros agentes paralelos

#### implementacao completa

##### fase 1: premultiplied alpha
- `shaders/quad.wgsl`: output `rgb * a, a`
- `shaders/text.wgsl`: output `rgb * a, a`
- `src/gpu.rs`: blend state `One/OneMinusSrcAlpha` em todos os pipelines

##### fase 2: layer system
- `src/gpu_vec.rs`: gpuvec extraido como modulo compartilhado
- `src/compositor.rs`: reescrito com layerid, layer struct, per-layer dirty tracking, per-layer quad/text buffers, offscreen textures
- API: `create_layer()`, `remove_layer()`, `set_layer_opacity()`, `set_layer_visible()`, `push_to_layer()`
- 7 testes unitarios para layers

##### fase 3: textsystem
- `src/text.rs`: refatorado com `resolve_for_layer()` (retorna verts/indices) e `finish_frame()` para eviction

##### fase 4: composite pipeline
- `shaders/composite.wgsl`: full-screen triangle, amostra textura de layer, multiplica por opacity
- `src/gpu.rs`: composite pipeline, bind group layouts, sampler adicionados ao gpucontext

##### fase 5: render loop
- `src/window.rs`: per-layer render passes (offscreen) + composite pass (surface)

##### fase 6: exemplos
- `examples/layers_demo.rs`: background estatico + foreground dinamico com 80% opacity
- `examples/hello.rs`, `examples/text_demo.rs` atualizados para nova API
- `examples/counter.rs`, `examples/signal_counter.rs`, `examples/input_demo.rs`, `examples/builder_demo.rs` atualizados na integracao com master

#### integracao com master (2021-06-17)
- cherry-pick do commit original tinha conflitos com task-02, task-04, task-06, task-09
- resolucao manual: mantidas features do master (inputstate, layoutengine, component, signal, appevent/eventloopproxy)
- adaptada renderizacao para usar layer system (per-layer render passes + composite pass)
- corrigido WASM limits para `Limits::default()` (fix do task-06)
- todos os 89 testes passando

#### verificacao
- `cargo check --examples`: 0 erros
- `cargo test`: 89 testes passando (7 novos de layers + 82 existentes)
