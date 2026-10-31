---
id: task-13
title: "task-13: mobile specifics, safe areas, IME, lifecycle"
project: plev
type: task
status: done
date: 2021-09-12
date_basis: header
topics: [mobile, input, layout]
relations: [task-03, task-07, task-08, task-09, task-10, task-11, task-12]
source: [docs/log/steps/task-13.md, docs/log/changes/task-13.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-13: mobile specifics, safe areas, IME, lifecycle

Records the mobile functions shared by Android and iOS: safe-area insets in the layout, virtual keyboard with text composition, DPI scaling, unified lifecycle state (`AppState`) and re-layout on rotation. The result is the `mobile_input` example, with 14 unit tests and 131 tests passing in total. The insets return zero on iOS with winit 0.30, the keyboard height is a heuristic of 40% of the screen, and verification on a device is still pending.

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

## Delivery record

### 2021-09-08

#### criação
- criado `src/platform.rs` com safeareainsets e from_window() (#[cfg] android)
- criado `src/lifecycle.rs` com appstate enum e lifecyclemanager
- criado `src/ime.rs` com imestate, handle_event(), keyboard helpers
- estendido viewcontext com safe_area, scale_factor, keyboard_visible, keyboard_height
- adicionado purge_caches() ao textsystem para memory warning
- integrado todos os módulos no window.rs (event handlers, render loop)
- criado examples/mobile_input.rs com demo funcional

#### decisões
- `#[cfg]` blocks em platform.rs em vez de traits, winit já abstrai quase tudo
- keyboard height via heurística (40% mobile, 0 desktop), winit não expõe
- ios retorna zeros para safe area, inner_position() retorna err no winit 0.30
- gpu.rs não modificado, master já tem recreate_surface()/drop_surface()

#### adaptações ao master atual
- rebased sobre master com layer system (task-07), effects (task-08), input (task-09/10), ios (task-12)
- window.rs integração feita sobre gpustate enum com suspended variant
- exemplo adaptado ao compositor com layers e composite pipeline
