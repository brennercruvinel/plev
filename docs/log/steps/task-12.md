---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2021-08-29
domain: task-tracking
---

# task-12: ios build + lifecycle

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
- [x] `ios/Info.plist`: bundle metadata
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
