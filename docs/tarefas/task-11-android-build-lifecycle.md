---
id: task-11
title: "task-11: android build + lifecycle"
project: plev
type: tarefa
status: concluido
date: 2021-08-16
date_basis: cabecalho
topics: [task-tracking]
relations: []
source: [docs/log/steps/task-11.md, docs/log/changes/task-11.md]
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-11: android build + lifecycle

## objetivo
fazer o plev compilar e rodar em android via `android-activity` crate + winit. vulkan como backend GPU.

## checklist de conclusao
- [x] cargo.toml configurado para target android (feature android-game-activity, android_logger, metadata APK)
- [x] lifecycle: gpustate::suspended + drop_surface/recreate_surface
- [x] surface recreation apos rotate/resize (re-query capabilities)
- [x] font loading (embedded font via include_bytes, cfg compartilhado com WASM)
- [x] touch input (ja coberto por task-10, touch_input state machine integrada)
- [x] compilacao nativa nao regrediu (cargo check + cargo check --examples + smoke tests)
- [x] todos examples adaptados para option<surface>
- [ ] build gera .apk funcional, requer NDK instalado
- [ ] rendering em device, requer NDK + device android
- [ ] `cargo build --target aarch64-linux-android`, requer NDK toolchain

## nota
itens nao marcados dependem de infraestrutura externa (android NDK/SDK/device), nao de codigo.

## Registro de entrega

### sessão 1 (2021-08-11)

#### implementado
- cargo.toml: `android-game-activity` feature, `android_logger`, APK metadata
- `src/lib.rs`: `android_main` entry point com `android-activity` + `GameActivity`
- `src/gpu.rs`: `GpuState::Suspended` pattern para surface lifecycle
- surface drop/recreate no `suspended()`/`resumed()` com re-query de capabilities
- font loading: `cfg(any(target_arch = "wasm32", target_os = "android"))` usa embedded font
- examples adaptados para `Option<Surface>`, safe early-return quando surface não existe
- touch input já integrado via task-10 state machine

#### decisões
- embedded font compartilhado com WASM path (mesma condição cfg)
- `GpuState` enum em vez de `Option<Surface>` para clareza de estados
- não implementar vulkan validation layers, complexidade desnecessária para builds de dev
- itens de build (NDK cross-compile, .apk, device test) ficam para quando NDK estiver disponível

#### testes
- `cargo check` nativo: ok
- `cargo check --examples`: ok
- smoke test visual (`text_demo`, `layers_demo`): ok
- cross-compile para android: pendente (requer NDK)
