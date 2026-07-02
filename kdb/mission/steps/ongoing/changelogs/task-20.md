---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-08
domain: task-tracking
---

# changelog, task-20: WASM visual validation

## sessão 1 (2026-03-08)

### build
- `cargo check --target wasm32-unknown-unknown`
- `trunk build` (dev, 0.09s cached)
- `trunk build --release` (wasm-opt z, 30.5s)

### bundle size
- WASM bundle: **2.4mb** (release, wasm-opt z)
- total dist: 2.5mb

### pendente
- `trunk serve` visual validation no chrome com webgpu
- screenshot comparativo native vs WASM
- FPS counter / performance check
- showcase scene rendering verification

### warnings (esperados)
- `mut gpu` unused (cfg-condicional, usado no native mas não WASM)
- `touch_input` field never read (será usado quando input integration completa)

## sessão 2 (2026-06-16)

### avanço
- [x] trunk serve com showcase rodando no browser (webgpu)
- [x] web bate o desktop por amostra de pixel (fundo 48,48,48), via render em view sRGB
- [x] prime creatures também roda na web

### pendente
- screenshot diff formal native vs web em critical paths com baseline versionado
