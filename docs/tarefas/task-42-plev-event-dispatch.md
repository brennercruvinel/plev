---
id: task-42
title: "task-42: plev event dispatch + gitbutler UI port"
project: plev
type: tarefa
status: substituido
date: 2026-09-25
date_basis: cabecalho
topics: [task-tracking]
relations: []
source: [docs/log/steps/task-42-plev-event-dispatch.md, docs/log/changes/task-42.md]
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-42: plev event dispatch + gitbutler UI port

## objetivo
port do frontend tauri/svelte do gitbutler para plev nativo (100% rust, sem webview). demonstrar que plev e capaz de apps reais de producao. implementar infra de dispatch e overlays no core plev.

## branch
`task/TASK-42-plev-event-dispatch`

## dependencias
nenhuma task pendente. usa core plev (compositor, text, input, signal, layout).

## fases

### fase 0, framework extensions [x]
- [x] `shaders/rect_sdf.wgsl`, SDF rounded box (inigo quilez) + border + premultiplied alpha
- [x] `compositor.rs`: `SceneNode::RoundedRect`, `RectSdfVertex`, per-layer SDF buffers
- [x] `gpu.rs`: `rect_sdf_pipeline` (premultiplied blend)
- [x] layer clip rect (`set_scissor_rect`)
- [x] font family support (`TextNodeKey.font_family`)
- [x] codicons font embutido (303kb)

### fase 1, clickable shell [x]
- [x] hit regions em unassignedview e multistackview
- [x] click routing: file row / commit card -> diffview
- [x] hover real via cursormoved
- [x] teclado: up/down, enter

### fase 2, chrome + components [x]
- [x] sidebar (codicons icons)
- [x] header reformulado
- [x] checkbox, tabs, avatar, panelheader components

### fase 3, commit flow [x]
- [x] inline commit form
- [x] tecla c toggle
- [x] text input no form

### fase 4, typed dispatch + overlays (em andamento)
- [x] step 1-6: `src/dispatch.rs` (core), `src/overlay.rs` (core), `src/actions.rs` (gitbutler-plev)
- [x] 26 testes passando
- [ ] step 7: context_menu.rs render
- [ ] step 8: modal.rs render
- [ ] step 9: wire no main.rs (escape/click-outside dismiss)

### fase 5, dados reais (pendente)
- [ ] gix provider
- [ ] file watcher
- [ ] real git commits

### fase 6, polish (pendente)
- [ ] animacoes
- [ ] shortcuts cmd+
- [ ] scrollbar auto-hide

## notas
- dispatch.rs e overlay.rs ficam no core plev (infra generica)
- widgetaction trait = any + send + 'static
- crate: `experiment/gitbutler-plev/`
- ~3800 LOC total (incluindo dispatch/overlay no core)

## fechamento (orquestrador, 2026-09-25)

substituída, não terminada. parada desde 2022 na fase 4, num crate
(`experiment/gitbutler-plev/`) que não existe mais no repositório. o que
ela queria provar, um app de produção nativo em plev, o `ide` provou por
outro caminho: o despacho tipado virou o dispatcher do `sem` (task-58), os
overlays viraram o `OverlayManager`, o modal e o menu de contexto do comps
(projetados desde a task-75 fase 2a), e os dados reais vieram do crate
`git` e do workbench do ide com git de verdade (task-62). as fases 5 e 6
(watcher, animações, atalhos) não viram trabalho novo.

## Registro de entrega

### 2022-10-06

#### fase 0, framework extensions
- criado `shaders/rect_sdf.wgsl` com SDF rounded box (inigo quilez formula)
- adicionado `SceneNode::RoundedRect` e `RectSdfVertex` ao compositor
- pipeline `rect_sdf_pipeline` no gpu.rs
- layer clip rect via `set_scissor_rect`
- font family support em textnodekey
- codicons font embutido (303kb)

#### fase 1, clickable shell
- hit regions em unassignedview e multistackview
- click routing file row/commit card -> diffview
- hover real via cursormoved + hit-test
- teclado up/down/enter
- bug fixes: hover alpha, SHA panic guard

#### fase 2, chrome + components
- sidebar com codicons icons (workspace/branches/history/settings)
- header com roundedrect badge + theme toggle
- components: checkbox, tabs, avatar, panel_header

#### fase 3, commit flow
- inline commit form (message input + commit/cancel)
- tecla c toggle commit mode
- text input com character/backspace/enter/escape

### 2022-10-19

#### fase 4, typed dispatch + overlays (steps 1-6)
- `src/dispatch.rs` (211 LOC): actionqueue com typed drain via any downcast
- `src/overlay.rs` (349 LOC): overlaymanager z-ordered stack, hit_test_outside
- `src/actions.rs`: fileaction, modalaction enums
- 26 testes passando (dispatch + overlay + doctests)
- steps 7-9 pendentes (context_menu render, modal render, wiring)

#### reorganizacao (2022-10-19)
- crate extraido de `experiment/gitbutler/crates/gitbutler-plev/` para `experiment/gitbutler-plev/`
- clone monorepo gitbutler (2.4gb) removido
- path plev dependency atualizado: `"../../../.."` -> `"../.."`
