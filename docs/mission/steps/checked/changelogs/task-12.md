---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2021-08-25
domain: changelog
---

# changelog, task-12: ios build + lifecycle

## 2021-08-25

### implementação completa
- branch `task/TASK-12-ios-worktree` criada a partir de `master` (e6f1c7e)
- worktree isolado usado para evitar conflitos com outros agentes trabalhando no mesmo repo

### mudanças:
1. **cargo.toml**: adicionado `staticlib` ao `crate-type` para gerar `.a` necessário para linking no xcode
2. **src/text.rs**: font loading com branch ios, `cfg(any(target_arch = "wasm32", target_os = "ios"))` usa embedded font via `include_bytes!`, desktop mantém `FontSystem::new()`
3. **src/window.rs**:
   - window attributes: `with_inner_size` só em `not(target_os = "ios")`, ios usa `with_valid_orientations(LandscapeAndPortrait)`
   - `suspended()` handler com logging (ios `applicationWillResignActive`)
   - `memory_warning()` handler com logging (ios `applicationDidReceiveMemoryWarning`)
   - touch event logging via `WindowEvent::Touch`
4. **ios/info.plist**: bundle metadata (cfbundleidentifier, uisupportedinterfaceorientations, uilaunchstoryboardname)
5. **scripts/ios-sim.sh**: script para build + deploy no ios simulator

### verificação:
- `cargo check --target aarch64-apple-ios-sim`, ok
- `cargo check --target aarch64-apple-ios`, ok
- `cargo check` (macos, regressão), ok

### apis verificadas:
- `winit::platform::ios::WindowAttributesExtIOS`, existe em winit 0.30
- `winit::platform::ios::ValidOrientations`, existe em winit 0.30
- `ApplicationHandler::suspended()`, existe em winit 0.30
- `ApplicationHandler::memory_warning()`, existe em winit 0.30
