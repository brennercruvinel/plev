---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-24
domain: changelog
---

# task-48 changelog: ide, git client nativo em plev

## app
- [x] estado (app.rs), render (renderer.rs), comandos (actions.rs)
- [x] file watcher para hot reload (watcher.rs)
- [x] tema (theme.rs) e adapters de dados (adapters.rs)

## views
- [x] header com badge e theme toggle
- [x] sidebar com codicons (workspace/branches/history/settings)
- [x] commit_form inline (message input + commit/cancel)
- [x] diff_view, unassigned_view, multi_stack_view
- [x] modulo workspace (render, input, overlays)

## components reutilizaveis
- [x] avatar, badge, panel_header, context_menu, tabs, separator
- [x] checkbox, modal, button, hoff

## integracao
- [x] consome git backend via GitClient (ui nunca bloqueia)
- [x] consome rope para edicao de texto
- [x] dispatch tipado e overlay vindos da engine (origem task-42)

## validacao
- [x] 55 testes (workspace, scrolling, overlays, panels, components)
- [x] rename basic-ide -> ide na reorganizacao por crates
