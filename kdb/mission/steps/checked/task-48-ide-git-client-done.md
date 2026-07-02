---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-24
domain: task-tracking
---

# task-48: ide, git client nativo em plev

## objetivo
um git client de verdade, todo desenhado na gpu pela engine, como prova de app real e nao de demo. e a maturacao do port iniciado em task-42 (dispatch + overlay + chrome), agora um crate proprio que consome o backend git e o nucleo de edicao.

## dependencias
- task-47 (git, backend via GitClient)
- task-46 (rope, edicao de texto)
- task-42 (dispatch tipado + overlay, que foram absorvidos pela engine)
- engine (builder, overlay, input dispatch, theme)

## contexto
o port do gitbutler comecou em `experiment/gitbutler-plev/` (task-42). o dispatch tipado e o overlay manager amadureceram e foram para a engine; o app em si virou o crate `ide`. nenhuma linha de kotlin ou swift, nenhum widget nativo, cada pixel desenhado pela engine.

## o que foi entregue
- crate `ide`, app de git client. estado em `app.rs`, render em `renderer.rs`, comandos em `actions.rs`, file watcher em `watcher.rs` (hot reload), tema em `theme.rs`, adapters de dados em `adapters.rs`.
- views: header, sidebar (workspace/branches/history/settings com codicons), commit_form (inline), diff_view, unassigned_view, multi_stack_view, e o modulo workspace (render, input, overlays).
- components reutilizaveis: avatar, badge, panel_header, context_menu, tabs, separator, hoff, checkbox, modal, button.
- consome o git backend via `GitClient` (a ui nunca bloqueia em git) e o `rope` para edicao.

## numeros honestos
- 33 arquivos .rs, ~5964 LOC, 55 testes (workspace, scrolling, overlays, panels, components).
- antes era `basic-ide` / `basicIDE`; virou `ide` na reorganizacao por crates e na limpeza de naming.
- e prova de app real, nao paridade com o gitbutler.

## referencias
- task-42 (origem: dispatch + overlay + port inicial)
- usa adr [git-backend-gix-reads-cli-mutations](../../../adr/git-backend-gix-reads-cli-mutations.md) e [text-editing-core-follows-helix](../../../adr/text-editing-core-follows-helix.md)
- commit f15198a (medicao real de texto no ide, fim da heuristica)

## fora de escopo
- paridade total de features com o gitbutler
- mobile (git client e desktop-only)
