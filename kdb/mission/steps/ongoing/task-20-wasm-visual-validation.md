---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-08
domain: task-tracking
---

# task-20: WASM visual validation

## objetivo
validar rendering WASM/webgpu no browser.

## dependências
- task-06 (WASM validation code)

## checklist
- [x] `cargo check --target wasm32-unknown-unknown` compila
- [x] `trunk build` funciona (dev)
- [x] `trunk build --release`, bundle size: **2.4mb** (wasm-opt z)
- [ ] `trunk serve` + visual validation no chrome com webgpu
- [ ] rendering idêntico ao native
- [ ] showcase scene renderiza corretamente
- [ ] FPS/performance check
