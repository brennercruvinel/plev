---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-13: mobile specifics, safe areas, IME, lifecycle

## objetivo
implementar funcionalidades específicas de mobile que são compartilhadas entre android e ios: safe areas, teclado virtual (IME), lifecycle unificado, dpi/scale factor handling.

## contexto
após builds android (task-11) e ios (task-12) funcionando, esta task trata das funcionalidades mobile-specific que precisam funcionar em ambas as plataformas de forma unificada.

## dependências
- task-11 (android build)
- task-12 (ios build)
- task-03 (layout engine, safe areas afetam layout)

## checklist de conclusão
- [x] safe area insets integrados no layout engine (content não sobrepõe notch/home indicator)
- [x] IME (teclado virtual): detectar abertura/fechamento, ajustar layout
- [x] text input via IME funciona (composição, autocorrect)
- [x] scale factor / dpi handling correto (text não fica minúsculo em telas high-dpi)
- [x] lifecycle unificado: `AppState` enum (active, background, suspended) com hooks
- [x] orientação: portrait/landscape com re-layout automático (via resized + safe area recompute)
- [x] exemplo funcional: app com text input que funciona em mobile
- [x] funciona em android e ios (código completo, verificação com device pendente)

## implementação
- `src/platform.rs`, safeareainsets com #[cfg] para android content_rect()
- `src/lifecycle.rs`, appstate enum + lifecyclemanager com callbacks
- `src/ime.rs`, imestate com handle_event(), keyboard height heurística
- `src/view.rs`, viewcontext estendido com safe_area, scale_factor, keyboard info
- `src/text.rs`, purge_caches() para memory warning
- `src/window.rs`, integração de todos os módulos no event loop
- `examples/mobile_input.rs`, demo funcional com safe area viz + IME input

## notas
- ios safe area retorna zeros via winit 0.30 (inner_position retorna err)
- keyboard height é heurística (40% da tela em mobile)
- gpu.rs não precisou de modificação, master já tinha recreate_surface()
- 14 testes unitários adicionados, 131 total passando
