---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-23: CI/CD

## sessao 1 (2026-03-08)

### criado
- `.github/workflows/ci.yml` com 6 jobs paralelos
- check: cargo check --workspace --examples (ubuntu)
- test: cargo test --workspace (ubuntu)
- clippy: cargo clippy --workspace --all-targets -d warnings (ubuntu)
- fmt: cargo fmt --all --check (ubuntu)
- wasm: cargo check --target wasm32-unknown-unknown (ubuntu)
- ios: cargo check --target aarch64-apple-ios-sim (macos)

### decisoes
- toolchain fixada em 1.94 (agente original colocou 1.84, corrigido)
- linux deps: libwayland-dev, libxkbcommon-dev (requeridos por winit/wgpu)
- rustflags="-d warnings" global
- sem android (requer setup NDK complexo)
- sem trunk build (requer instalar trunk no CI)

### nao testado
- workflow nao foi executado no github (sem remote)
