---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-11: android build + lifecycle

## sessão 1 (2026-03-08)

### implementado
- cargo.toml: `android-game-activity` feature, `android_logger`, APK metadata
- `src/lib.rs`: `android_main` entry point com `android-activity` + `GameActivity`
- `src/gpu.rs`: `GpuState::Suspended` pattern para surface lifecycle
- surface drop/recreate no `suspended()`/`resumed()` com re-query de capabilities
- font loading: `cfg(any(target_arch = "wasm32", target_os = "android"))` usa embedded font
- examples adaptados para `Option<Surface>`, safe early-return quando surface não existe
- touch input já integrado via task-10 state machine

### decisões
- embedded font compartilhado com WASM path (mesma condição cfg)
- `GpuState` enum em vez de `Option<Surface>` para clareza de estados
- não implementar vulkan validation layers, complexidade desnecessária para builds de dev
- itens de build (NDK cross-compile, .apk, device test) ficam para quando NDK estiver disponível

### testes
- `cargo check` nativo: ok
- `cargo check --examples`: ok
- smoke test visual (`text_demo`, `layers_demo`): ok
- cross-compile para android: pendente (requer NDK)
