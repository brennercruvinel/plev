---
id: task-30
title: "task-30: accessibility (accesskit)"
project: plev
type: tarefa
status: concluido
date: 2023-05-02
date_basis: cabecalho
topics: [acessibilidade, input]
relations: []
source: [docs/log/steps/task-30-accessibility.md, docs/log/changes/task-30.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-30: accessibility (accesskit)

Registra a integração de acessibilidade via AccessKit: um `AccessibilityState` que acumula os nós de cada frame, um grafo de foco com navegação sequencial e direcional, e o mapeamento entre `ViewId` e `NodeId`, atrás da feature `accessibility`, ativa por padrão. O resultado é uma árvore de acessibilidade atualizada a cada frame, e o código compila sem a feature (`--no-default-features`). Importa porque expõe a interface a leitores de tela e à navegação pelo foco.

## resultado
implementado sistema de acessibilidade completo via accesskit, feature-gated como `accessibility` (default on).

## implementacao

### arquivos criados/modificados
- `src/accessibility.rs` (novo, ~300 LOC), accessibilitystate, focusgraph, id mapping
- `src/window.rs`, a11y adapter integration, tree updates, event forwarding
- `src/input/mod.rs`, hit_regions() getter, set_focused_view()
- `src/lib.rs`, conditional pub mod accessibility
- `Cargo.toml`, accesskit 0.24 + accesskit_winit 0.32 (optional)

### arquitetura
- **feature-gated**: `default = ["accessibility"]`, compila sem accesskit com `--no-default-features`
- **lazy activation**: accesskit adapter criado no `resumed()`, tree updates per-frame
- **per-frame accumulator**: accessibilitystate acumula nodes durante build_scene, gera treeupdate
- **focusgraph**: navegacao sequencial (next/previous) e direcional (up/down/left/right)
- **id mapping**: viewid(u64) <-> nodeid(u64), root = u64::max

### handlers
- plevactivationhandler: retorna initial tree com window root
- plevactionhandler: log actions (click/focus) para debug
- plevdeactivationhandler: log desconexao de screen reader

### widget-to-role mapping
| plev type | accesskit role |
|-----------|---------------|
| hitregion(focusable) | button |
| hitregion(!focusable) | genericcontainer |
| scenenode::text | label |
| textinput | textinput |
| root | window |

### testes (8 novos)
- view_id_to_node_id round-trip
- root_node_id has no viewid
- empty tree update
- tree with nodes
- tree with hierarchy
- focus graph next/previous (wrapping)
- focus graph directional (up/down/left/right)
- focus graph skips non-focusable
- begin_frame clears state

## checklist
- [x] feature gate: `accessibility = ["dep:accesskit", "dep:accesskit_winit"]`
- [x] accessibilitystate accumulator com push_node/build_tree_update
- [x] focusgraph com navegacao sequencial e direcional
- [x] viewid <-> nodeid mapping
- [x] adapter integration em window.rs (with_direct_handlers)
- [x] window event forwarding para adapter
- [x] tree update per-frame no render
- [x] compila sem feature: `cargo check --no-default-features`
- [x] 8 testes unitarios

## Registro de entrega

### novos arquivos
- `src/accessibility.rs`, accessibilitystate, focusgraph, id mapping

### modificados
- `src/window.rs`, a11y adapter, handlers, tree updates
- `src/input/mod.rs`, hit_regions(), set_focused_view()
- `src/lib.rs`, conditional pub mod accessibility
- `Cargo.toml`, accesskit + accesskit_winit features

### testes adicionados
- 8 em accessibility.rs (id mapping, tree gen, focus graph)
