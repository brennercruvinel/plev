---
project: phi
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-13
domain: task-tracking
---

# task-42: φ event dispatch + gitbutler UI port

## objetivo
port do frontend tauri/svelte do gitbutler para φ nativo (100% rust, sem webview). demonstrar que φ e capaz de apps reais de producao. implementar infra de dispatch e overlays no core φ.

## branch
`task/TASK-42-φ-event-dispatch`

## dependencias
nenhuma task pendente. usa core φ (compositor, text, input, signal, layout).

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
- [x] step 1-6: `src/dispatch.rs` (core), `src/overlay.rs` (core), `src/actions.rs` (gitbutler-φ)
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
- dispatch.rs e overlay.rs ficam no core φ (infra generica)
- widgetaction trait = any + send + 'static
- crate: `experiment/gitbutler-φ/`
- ~3800 LOC total (incluindo dispatch/overlay no core)
