---
id: task-23
title: "task-23: CI/CD"
project: plev
type: task
status: done
date: 2021-12-18
date_basis: header
topics: [process, workspace]
relations: []
source: [docs/log/steps/task-23.md, docs/log/changes/task-23.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-23: CI/CD

Records plev's CI workflow on GitHub Actions, with six parallel jobs: check, test, clippy, fmt, wasm and ios. At the time there was no remote, and the workflow was not run on GitHub; `trunk build --release` and Android were left out of CI. It matters because it defines which checks run on the code and on which targets.

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

## Delivery record

### sessao 1 (2021-12-14)

#### criado
- `.github/workflows/ci.yml` com 6 jobs paralelos
- check: cargo check --workspace --examples (ubuntu)
- test: cargo test --workspace (ubuntu)
- clippy: cargo clippy --workspace --all-targets -d warnings (ubuntu)
- fmt: cargo fmt --all --check (ubuntu)
- wasm: cargo check --target wasm32-unknown-unknown (ubuntu)
- ios: cargo check --target aarch64-apple-ios-sim (macos)

#### decisoes
- toolchain fixada em 1.94 (agente original colocou 1.84, corrigido)
- linux deps: libwayland-dev, libxkbcommon-dev (requeridos por winit/wgpu)
- rustflags="-d warnings" global
- sem android (requer setup NDK complexo)
- sem trunk build (requer instalar trunk no CI)

#### nao testado
- workflow nao foi executado no github (sem remote)
