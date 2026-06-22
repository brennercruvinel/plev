---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# task-08 changelog

## 2026-03-08, fases a-d implementadas

### decisões técnicas
- **fragment shader only** (sem compute), portátil para WASM, conforme rules.md
- **13-tap separable gaussian blur**, radius 6, cobre sigma até ~6.0 com qualidade alta
- **texturepool grow-only** (padrão gpuvec), zero alocações em steady state
- **full-screen triangle trick**, 3 vértices via `@builtin(vertex_index)`, sem vbo
- **premultiplied alpha** nos effect passes, composite usa blend `One / OneMinusSrcAlpha`
- **texturehandle com view clonado**, resolve borrow conflict entre handle e pool mutável
- **worktree isolado**, necessário porque outros agentes modificam o working directory principal

### arquivos criados
- `src/texture_pool.rs`, texturepool com acquire/release
- `src/effects.rs`, effectprocessor, layereffect enum, gaussian_weights()
- `shaders/blur.wgsl`, 13-tap separável via fragment shader
- `shaders/shadow.wgsl`, silhouette extraction
- `shaders/composite.wgsl`, composição com opacity
- `examples/effects_demo.rs`, demo standalone

### testes
- 6 testes novos (4 gaussian_weights + 2 texture_pool)
- 10 testes total passando

### pendente
- fase e: integração com task-07 (layer system)
- fase f: resize handling, sigma=0 optimization, multi-pass, WASM validation, benchmark
