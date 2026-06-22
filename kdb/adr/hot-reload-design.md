---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-22
domain: hot-reload
---

# hot reload design - plev

data: 2026-03-22
task: gap-1

## pesquisa de campo

7 implementacoes estudadas, codigo-fonte lido:

| projeto | abordagem | velocidade | escopo | seguranca |
|---------|-----------|------------|--------|-----------|
| **subsecond** (dioxus, 35k stars) | jump table + thinlink | ~130ms | funcoes rust (tip crate) | safe (sem modificacao de memoria) |
| **hot-lib-reloader** (740 stars) | dylib via libloading | 0.5-2s | #[no_mangle] functions | unsafe (abi mismatch = ub) |
| **makepad live design** (6.2k stars) | DSL-only (sem rust reload) | instantaneo | propriedades visuais | fully safe |
| **vello** (bunker) | notify + debounce | instantaneo | shaders .wgsl | fully safe |
| **rerun** (bunker) | fileserver + #import resolve | instantaneo | shaders | fully safe |
| **leptos** (bunker) | AST diff + DOM patch | sub-segundo | view! macro blocks | safe |
| **dioxus rsx** (bunker) | AST diff + literal pool | sub-segundo | rsx macro blocks | safe |

flutter estudado como referencia: dart VM JIT + kernel snapshots + 3-tree reconciliation. impossivel replicar em rust (sem VM), mas o pattern element-tree-preserves-state e aplicavel.

## analise de fronteira - o que cruza o hot reload boundary no plev

### cruza (safe, todos send+sync):
- `SceneNode` - enum com f32, [f32;4], string. zero box<dyn>, zero closures
- `TextNodeKey` - string + u32 + u16 + option. hashable
- `TessellatedPath` - vec<quadvertex> + vec<u32> + u64 hash
- primitivos: u64, f32, [f32;4]

### nao cruza (fica no engine):
- `Compositor` - contem wgpu resources (not send)
- `GpuContext` - device/queue/surface (GPU-bound)
- `TextSystem` - font system + atlas
- `Layer` - texturas, buffers, bind groups
- `InputState` - hit regions (borrowed, nunca owned)

### boundary minimo:
```
USER CODE (reloadable) --> Vec<SceneNode> --> ENGINE (fixed)
```
`build_scene()` e uma funcao pura: recebe dimensoes + estado, retorna vec<scenenode>. o engine faz begin_frame/push/resolve/render.

## decisao: 3 tiers, pragmaticos

### tier 1 - shader hot reload (vello pattern)
**abordagem:**
- `notify_debouncer_full` watch em `shaders/*.wgsl`
- 500ms debounce
- on change: re-create shadermodule + re-create renderpipeline
- feature-gated: `#[cfg(debug_assertions)]` = disco, release = include_str!
- **referencia:** `bunker/repos/3d-graphics/rendering/vello/examples/with_winit/src/hot_reload.rs`
- **referencia:** `bunker/repos/rerun/crates/viewer/re_renderer/src/file_server.rs`

### tier 2 - DSL hot reload (makepad pattern, adaptado para plev_narrate!)
**abordagem:**
- plev_narrate! blocks compilam para element trees
- file watcher detecta mudanca em .rs contendo plev_narrate!
- re-parse apenas o bloco macro (syn AST visitor, pattern do leptos)
- override map: `HashMap<BlockId, Vec<Element>>` - override sempre consultado antes do compilado
- estado do componente preservado (element tree e recriada, state nao)
- **referencia:** `bunker/repos/rust-ecosystem/makepad/libs/live_reload_core/src/lib.rs` (livereloadwatchplan + dedup)
- **referencia:** `bunker/repos/rust-ecosystem/makepad/platform/src/live_reload.rs` (script_mod override)
- **referencia:** `bunker/repos/rust-ecosystem/leptos/leptos_hot_reload/src/lib.rs` (viewmacros + AST parse)

### tier 3 - rust code hot reload (subsecond)
**abordagem:**
- integrar `subsecond` crate (o que bevy 0.17 fez)
- `subsecond::call(|| { build_scene(...) })` wrapa o entry point
- thinlink como linker alternativo
- jump table atualizado via websocket
- **limitacoes:** so funciona no tip crate, nao WASM, struct layout nao muda
- **referencia:** bevy pr #19309, docs.rs/subsecond

## por que nao dylib (hot-lib-reloader)

- struct layout change = ub silencioso
- generics nao funcionam (#[no_mangle] requer monomorphizacao manual)
- linux: dlclose + tls = memory leak (nao descarrega)
- typeid muda entre loads (quebra dispatch.rs que usa any + downcast)
- macos: codesigning obrigatorio para cada dylib reload
- plev usa `Box<dyn Any + Send>` no actionqueue - typeid across dylib boundary = ub

## por que nao flutter-style

- requer VM com JIT + late binding
- rust e AOT compiled, sem vtable lookup implicito
- pattern de 3-tree (widget/element/renderobject) e aplicavel ao plev mas como **otimizacao de rendering** (gap 2: retained tree), nao como mecanismo de hot reload

## ordem de implementacao

1. **tier 1 (shader)** - vello pattern direto
2. **tier 2 (DSL)** - requer plev_narrate parser acessivel em runtime
3. **tier 3 (subsecond)** - avaliar compatibilidade com wgpu event loop

## cruzamento com arquitetura plev

| subsistema plev | tier 1 | tier 2 | tier 3 |
|----------------|--------|--------|--------|
| gpu.rs (pipelines) | re-create pipeline on shader change | n/a | n/a |
| compositor/ (scenenode) | n/a | override map injeta scenenodes | subsecond::call wrapa build_scene |
| builder/ (element) | n/a | re-parse plev_narrate! -> element tree | reload build_ui() |
| text.rs (shaping) | n/a | re-shape em text nodes alterados | automatico |
| signal.rs (state) | n/a | preservado (override nao toca state) | preservado (jump table nao reseta) |
| window.rs (event loop) | watcher thread paralelo | watcher thread paralelo | thinlink externo |

## dependencias

| crate | uso | tier |
|-------|-----|------|
| notify 7 | file system events (cross-platform) | 1, 2 |
| notify-debouncer-full 0.4 | debounce 500ms | 1, 2 |
| syn 2 (full, span-locations) | AST parsing de plev_narrate! | 2 |
| subsecond 0.7 | jump table + thinlink | 3 |

## arquivos de referencia no bunker 

- `bunker/repos/rust-ecosystem/makepad/libs/live_reload_core/src/lib.rs` - livereloadwatchplan, dedup, sink pattern
- `bunker/repos/rust-ecosystem/makepad/platform/src/live_reload.rs` - script_mod extraction + override map
- `bunker/repos/3d-graphics/rendering/vello/examples/with_winit/src/hot_reload.rs` - 37 LOC minimal shader reload
- `bunker/repos/rerun/crates/viewer/re_renderer/src/file_server.rs` - include_file! macro dual-mode
- `bunker/repos/rust-ecosystem/leptos/leptos_hot_reload/src/lib.rs` - viewmacros + AST parse via syn
- `bunker/repos/rust-ecosystem/leptos/leptos_hot_reload/src/diff.rs` - 25kb nested tree diff
- `bunker/repos/rust-ecosystem/dioxus/packages/rsx-hotreload/` - rsx diff + hotreloadliteral pool
