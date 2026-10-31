---
id: task-48
title: "task-48: ide, native git client in plev"
project: plev
type: task
status: done
date: 2026-05-11
date_basis: header
topics: [git, text-editing, platform]
relations: [adr-0020, adr-0021, task-42, task-46, task-47]
source: [docs/log/steps/task-48-ide-git-client.md, docs/log/changes/task-48.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [f15198a]
---

# task-48: ide, native git client in plev

Records the task for the ide, the native git client drawn by the engine as proof of a real app, not a demo. It consumes the `GitClient` from task-47 and the `rope` from task-46, and matures the port from task-42. The result is 33 files and 55 tests, with a UI that never blocks on git; it is not parity with gitbutler, and the text is measured, not heuristic, since commit f15198a.

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
- usa adr [git-backend-gix-reads-cli-mutations](../../adr/structure/0020-git-backend-gix-reads-cli-mutations.md) e [text-editing-core-follows-helix](../../adr/interface/0021-text-editing-core-follows-helix.md)
- commit f15198a (medicao real de texto no ide, fim da heuristica)

## fora de escopo
- paridade total de features com o gitbutler
- mobile (git client e desktop-only)

## Delivery record

### app
- [x] estado (app.rs), render (renderer.rs), comandos (actions.rs)
- [x] file watcher para hot reload (watcher.rs)
- [x] tema (theme.rs) e adapters de dados (adapters.rs)

### views
- [x] header com badge e theme toggle
- [x] sidebar com codicons (workspace/branches/history/settings)
- [x] commit_form inline (message input + commit/cancel)
- [x] diff_view, unassigned_view, multi_stack_view
- [x] modulo workspace (render, input, overlays)

### components reutilizaveis
- [x] avatar, badge, panel_header, context_menu, tabs, separator
- [x] checkbox, modal, button, hoff

### integracao
- [x] consome git backend via GitClient (ui nunca bloqueia)
- [x] consome rope para edicao de texto
- [x] dispatch tipado e overlay vindos da engine (origem task-42)

### validacao
- [x] 55 testes (workspace, scrolling, overlays, panels, components)
- [x] rename basic-ide -> ide na reorganizacao por crates
