---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: research
---

# technology radar, φ

**data:** 2026-03-11
**escopo:** resumo executivo da analise de ecossistema (50+ repos, 12 categorias)

---

## posicao do φ

compositing engine GPU-first em rust. nicho unico: shaders identicos em 6 targets (metal, vulkan, dx12, webgpu), dirty tracking per-layer, text de producao. nenhum competidor resolve esse problema completo.

## ameacas

| ameaca | severidade | mitigacao |
|--------|-----------|-----------|
| makepad amadurece e adiciona a11y + text quality | media | executar task-30 (a11y) e manter text quality como diferencial |
| dioxus blitz (wgpu renderer) fica estavel | media | φ e camada abaixo, pode ser consumido por frameworks como blitz |
| parley torna cosmic-text obsoleto | alta | task-32 planeja avaliacao. migrar se necessario |
| iced adiciona dirty tracking real | media | φ ja tem, manter lideranca tecnica |

## acoes imediatas (fase 4)

1. **task-27 animation:** usar web-time, implementar easing internamente (~200 LOC). referencia: keyframe, natura
2. **task-28 editable text:** cosmic-text cursor API. documentar pain points para task-32
3. **task-29 demo app:** proof of life. critico para visibilidade

## acoes de medio prazo (fase 5+)

4. **task-30 accessibility:** accesskit (accesskit 0.24 + accesskit_winit 0.32). ~700-900 LOC. macos/linux/windows/android cobertos. ios/WASM sem adapter ainda
5. **task-31 vector paths:** lyon 1.0 tessellation. ~600-800 LOC. habilita charts, icones, shapes
6. **task-32 text assessment:** parley 0.7 vs cosmic-text 0.18. research only. decidir migrar ou manter

## acoes de longo prazo (fase 6+)

7. **task-33 WASM plugins:** extism como referencia. wasmtime/wasmer como runtime. habilita fortran/inform no browser
8. **glam adoption:** SIMD math, drop-in replacement, baixo risco
9. **vello patterns:** encoding em streams para dirty tracking mais granular (inspiracao, nao dependencia)

## principio

estas sao referencias para insight. φ nao precisa de nenhuma dessas libs para funcionar, ja funciona com 325 testes, 17 examples, 6 targets, animation system, editable text e todo app demo. as integracoes ampliam capacidade sem comprometer a arquitetura core.

## documentacao completa

| documento | escopo |
|-----------|--------|
| `refs/competitors.md` | 7 frameworks analisados |
| `refs/linebender-ecosystem.md` | 7 projetos linebender |
| `refs/accessibility.md` | accesskit profundo |
| `refs/animation-motion.md` | 7 libs de animacao |
| `refs/math-physics-geometry.md` | glam, rapier, nalgebra, lyon |
| `refs/wasm-tooling.md` | 7 ferramentas WASM |
| `refs/charts-visualization.md` | plotters, charming, egui_graphs |
| `refs/tui-patterns.md` | 13 tui apps |
| `refs/emulators-wasm-runtimes.md` | 5 runtimes WASM |
| `refs/competitive-positioning.md` | matriz comparativa |
| `refs/integration-candidates.md` | ranking adopt/evaluate/watch/hold |
