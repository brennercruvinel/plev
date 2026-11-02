---
id: task-12
title: "task-12: ios build + lifecycle"
project: plev
type: task
status: done
date: 2021-08-29
date_basis: reconstructed
recorded: 2026-03-13
topics: [mobile, platform, fonts]
relations: [task-13]
source: [docs/log/steps/task-12.md, docs/log/changes/task-12.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-12: ios build + lifecycle

Records plev's adaptation to iOS on winit 0.30 and Metal, via wgpu, with an embedded font (desktop keeps `FontSystem::new()`), handling of suspension and memory warnings, and a build script for the simulator. The checks for `aarch64-apple-ios` and `aarch64-apple-ios-sim` and the regression on macOS passed; the test on the simulator was kept out of CI. Safe areas and touch are left for task-13.

## objetivo
fazer o plev compilar e rodar em ios via winit + metal (wgpu). font loading com embedded font (mesmo padrão WASM).

## contexto
winit 0.30 suporta ios. wgpu usa metal no ios (mesmo backend que macos). o código platform-specific existente (`not(target_arch = "wasm32")`) já cobre ios para GPU e window init. apenas font loading precisa de branch ios-specific.

**nota:** task-12 é independente das tasks não-mergeadas. safe areas e touch hit-testing ficam para task-13.

## decisões de design
- font loading ios: embedded font via `include_bytes!` (mesmo padrão WASM), `fontdb` no ios não encontra fontes do sistema
- cfg strategy: `cfg(target_os = "ios")` como terceiro branch só em `text.rs`
- safe areas: deferido para task-13
- build system: script bash para simulator + xcode project para device
- lifecycle: `suspended()` + `memory_warning()` com logging (ios não destrói surface metal no background)

## checklist de conclusão
- [x] `Cargo.toml`: adicionar `staticlib` ao crate-type
- [x] `text.rs`: font loading com branch ios (embedded font)
- [x] `window.rs`: lifecycle handlers (suspended, memory_warning)
- [x] `window.rs`: ios window attributes (orientations, sem inner_size)
- [x] `window.rs`: touch event logging
- [x] `apps/uigallery/ios/Info.plist`: bundle metadata
- [x] `scripts/ios-sim.sh`: build + deploy para simulator
- [x] compilação `cargo check --target aarch64-apple-ios-sim` sem erros
- [x] compilação `cargo check --target aarch64-apple-ios` sem erros
- [x] compilação `cargo check` (macos, regressão) sem erros
- [ ] teste no simulator (requer simulator rodando, fora do escopo de CI)
- [x] documentação em knowledge/ e changelogs/

## armadilhas
- ios simulator em apple silicon suporta metal, ok
- binary size debug ~50mb+, aceitável para dev
- winit `run_app` nunca retorna no ios, comportamento esperado
- `with_inner_size` irrelevante no ios (sempre fullscreen)

## Delivery record

### 2021-08-25

#### implementação completa
- branch `task/TASK-12-ios-worktree` criada a partir de `master` (e6f1c7e)
- worktree isolado usado para evitar conflitos com outros agentes trabalhando no mesmo repo

#### mudanças:
1. **cargo.toml**: adicionado `staticlib` ao `crate-type` para gerar `.a` necessário para linking no xcode
2. **src/text.rs**: font loading com branch ios, `cfg(any(target_arch = "wasm32", target_os = "ios"))` usa embedded font via `include_bytes!`, desktop mantém `FontSystem::new()`
3. **src/window.rs**:
   - window attributes: `with_inner_size` só em `not(target_os = "ios")`, ios usa `with_valid_orientations(LandscapeAndPortrait)`
   - `suspended()` handler com logging (ios `applicationWillResignActive`)
   - `memory_warning()` handler com logging (ios `applicationDidReceiveMemoryWarning`)
   - touch event logging via `WindowEvent::Touch`
4. **ios/info.plist**: bundle metadata (cfbundleidentifier, uisupportedinterfaceorientations, uilaunchstoryboardname)
5. **scripts/ios-sim.sh**: script para build + deploy no ios simulator

#### verificação:
- `cargo check --target aarch64-apple-ios-sim`, ok
- `cargo check --target aarch64-apple-ios`, ok
- `cargo check` (macos, regressão), ok

#### apis verificadas:
- `winit::platform::ios::WindowAttributesExtIOS`, existe em winit 0.30
- `winit::platform::ios::ValidOrientations`, existe em winit 0.30
- `ApplicationHandler::suspended()`, existe em winit 0.30
- `ApplicationHandler::memory_warning()`, existe em winit 0.30
