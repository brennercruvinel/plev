---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-07: sistema de camadas independentes

## 2026-03-08

### inicio da task
- branch `task/TASK-07-layer-system` criada a partir de `master` (pos task-03)
- usou worktree isolada para evitar conflitos com outros agentes paralelos

### implementacao completa

#### fase 1: premultiplied alpha
- `shaders/quad.wgsl`: output `rgb * a, a`
- `shaders/text.wgsl`: output `rgb * a, a`
- `src/gpu.rs`: blend state `One/OneMinusSrcAlpha` em todos os pipelines

#### fase 2: layer system
- `src/gpu_vec.rs`: gpuvec extraido como modulo compartilhado
- `src/compositor.rs`: reescrito com layerid, layer struct, per-layer dirty tracking, per-layer quad/text buffers, offscreen textures
- API: `create_layer()`, `remove_layer()`, `set_layer_opacity()`, `set_layer_visible()`, `push_to_layer()`
- 7 testes unitarios para layers

#### fase 3: textsystem
- `src/text.rs`: refatorado com `resolve_for_layer()` (retorna verts/indices) e `finish_frame()` para eviction

#### fase 4: composite pipeline
- `shaders/composite.wgsl`: full-screen triangle, amostra textura de layer, multiplica por opacity
- `src/gpu.rs`: composite pipeline, bind group layouts, sampler adicionados ao gpucontext

#### fase 5: render loop
- `src/window.rs`: per-layer render passes (offscreen) + composite pass (surface)

#### fase 6: exemplos
- `examples/layers_demo.rs`: background estatico + foreground dinamico com 80% opacity
- `examples/hello.rs`, `examples/text_demo.rs` atualizados para nova API
- `examples/counter.rs`, `examples/signal_counter.rs`, `examples/input_demo.rs`, `examples/builder_demo.rs` atualizados na integracao com master

### integracao com master (2026-03-08)
- cherry-pick do commit original tinha conflitos com task-02, task-04, task-06, task-09
- resolucao manual: mantidas features do master (inputstate, layoutengine, component, signal, appevent/eventloopproxy)
- adaptada renderizacao para usar layer system (per-layer render passes + composite pass)
- corrigido WASM limits para `Limits::default()` (fix do task-06)
- todos os 89 testes passando

### verificacao
- `cargo check --examples`: 0 erros
- `cargo test`: 89 testes passando (7 novos de layers + 82 existentes)
