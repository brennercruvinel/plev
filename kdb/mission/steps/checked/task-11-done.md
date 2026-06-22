---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-11: android build + lifecycle, done

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
