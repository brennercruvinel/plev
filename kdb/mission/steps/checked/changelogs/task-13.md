---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-13: mobile specifics

## 2026-03-08

### criação
- criado `src/platform.rs` com safeareainsets e from_window() (#[cfg] android)
- criado `src/lifecycle.rs` com appstate enum e lifecyclemanager
- criado `src/ime.rs` com imestate, handle_event(), keyboard helpers
- estendido viewcontext com safe_area, scale_factor, keyboard_visible, keyboard_height
- adicionado purge_caches() ao textsystem para memory warning
- integrado todos os módulos no window.rs (event handlers, render loop)
- criado examples/mobile_input.rs com demo funcional

### decisões
- `#[cfg]` blocks em platform.rs em vez de traits, winit já abstrai quase tudo
- keyboard height via heurística (40% mobile, 0 desktop), winit não expõe
- ios retorna zeros para safe area, inner_position() retorna err no winit 0.30
- gpu.rs não modificado, master já tem recreate_surface()/drop_surface()

### adaptações ao master atual
- rebased sobre master com layer system (task-07), effects (task-08), input (task-09/10), ios (task-12)
- window.rs integração feita sobre gpustate enum com suspended variant
- exemplo adaptado ao compositor com layers e composite pipeline
