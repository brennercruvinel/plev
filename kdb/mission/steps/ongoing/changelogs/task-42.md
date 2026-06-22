---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-13
domain: task-tracking
---

# changelog, task-42: plev event dispatch + gitbutler UI port

## 2026-03-12

### fase 0, framework extensions
- criado `shaders/rect_sdf.wgsl` com SDF rounded box (inigo quilez formula)
- adicionado `SceneNode::RoundedRect` e `RectSdfVertex` ao compositor
- pipeline `rect_sdf_pipeline` no gpu.rs
- layer clip rect via `set_scissor_rect`
- font family support em textnodekey
- codicons font embutido (303kb)

### fase 1, clickable shell
- hit regions em unassignedview e multistackview
- click routing file row/commit card -> diffview
- hover real via cursormoved + hit-test
- teclado up/down/enter
- bug fixes: hover alpha, SHA panic guard

### fase 2, chrome + components
- sidebar com codicons icons (workspace/branches/history/settings)
- header com roundedrect badge + theme toggle
- components: checkbox, tabs, avatar, panel_header

### fase 3, commit flow
- inline commit form (message input + commit/cancel)
- tecla c toggle commit mode
- text input com character/backspace/enter/escape

## 2026-03-13

### fase 4, typed dispatch + overlays (steps 1-6)
- `src/dispatch.rs` (211 LOC): actionqueue com typed drain via any downcast
- `src/overlay.rs` (349 LOC): overlaymanager z-ordered stack, hit_test_outside
- `src/actions.rs`: fileaction, modalaction enums
- 26 testes passando (dispatch + overlay + doctests)
- steps 7-9 pendentes (context_menu render, modal render, wiring)

### reorganizacao (2026-03-13)
- crate extraido de `experiment/gitbutler/crates/gitbutler-plev/` para `experiment/gitbutler-plev/`
- clone monorepo gitbutler (2.4gb) removido
- path plev dependency atualizado: `"../../../.."` -> `"../.."`
