---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2021-12-18
domain: task-tracking
---

# task-23: CI/CD, done

## objetivo
github actions workflow para CI.

## checklist
- [x] cargo check --workspace --examples (job: check)
- [x] cargo test --workspace (job: test)
- [x] cargo check --target wasm32-unknown-unknown (job: wasm)
- [x] cargo check --target aarch64-apple-ios-sim (job: ios, macos runner)
- [x] cargo clippy (job: clippy, -d warnings)
- [x] cargo fmt --check (job: fmt)
- [ ] trunk build --release para WASM, nao incluido no CI (requer trunk instalado)

## notas
- workflow nao foi testado em github (sem remote configurado)
- toolchain fixada em 1.94 (testada localmente)
- linux jobs instalam libwayland-dev e libxkbcommon-dev (deps do winit/wgpu)
- rustflags="-d warnings" global

## arquivos criados
- `.github/workflows/ci.yml`
