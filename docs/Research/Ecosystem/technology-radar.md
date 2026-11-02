---
id: research-technology-radar
title: "technology radar, plev"
project: plev
type: research
status: reference
date: 2022-06-15
date_basis: reconstructed
recorded: 2026-03-11
topics: [research, competition]
relations: [research-accessibility, research-animation-motion, research-charts-visualization, research-competitive-positioning, research-competitors, research-emulators-wasm-runtimes, research-integration-candidates, research-linebender-ecosystem, research-math-physics-geometry, research-tui-patterns, research-wasm-tooling, task-27, task-28, task-29, task-30, task-31, task-32, task-33]
source: docs/ref/technology-radar.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# technology radar, plev

Executive summary of the ecosystem analysis of 2022-06-15, with more than fifty repositories in twelve categories. It sets out plev's position, the threats with severity and mitigation, and the actions by phase of the plan. The central result is that no competitor combines identical shaders across six targets, per-layer dirty tracking and production text, and that the integrations expand the capability without compromising the core architecture.

**data:** 2022-06-15
**escopo:** resumo executivo da analise de ecossistema (50+ repos, 12 categorias)

---

## posicao do plev

compositing engine GPU-first em rust. nicho unico: shaders identicos em 6 targets (metal, vulkan, dx12, webgpu), dirty tracking per-layer, text de producao. nenhum competidor resolve esse problema completo.

## ameacas

| ameaca | severidade | mitigacao |
|--------|-----------|-----------|
| makepad amadurece e adiciona a11y + text quality | media | executar task-30 (a11y) e manter text quality como diferencial |
| dioxus blitz (wgpu renderer) fica estavel | media | plev e camada abaixo, pode ser consumido por frameworks como blitz |
| parley torna cosmic-text obsoleto | alta | task-32 planeja avaliacao. migrar se necessario |
| iced adiciona dirty tracking real | media | plev ja tem, manter lideranca tecnica |

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

estas sao referencias para insight. plev nao precisa de nenhuma dessas libs para funcionar, ja funciona com 325 testes, 17 examples, 6 targets, animation system, editable text e todo app demo. as integracoes ampliam capacidade sem comprometer a arquitetura core.

## documentacao completa

| documento | escopo |
|-----------|--------|
| [`competitors.md`](competitors.md) | 7 frameworks analisados |
| [`linebender-ecosystem.md`](linebender-ecosystem.md) | 7 projetos linebender |
| [`accessibility.md`](../Interface/accessibility.md) | accesskit profundo |
| [`animation-motion.md`](../Rendering/animation-motion.md) | 7 libs de animacao |
| [`math-physics-geometry.md`](../Rendering/math-physics-geometry.md) | glam, rapier, nalgebra, lyon |
| [`wasm-tooling.md`](wasm-tooling.md) | 7 ferramentas WASM |
| [`charts-visualization.md`](../Rendering/charts-visualization.md) | plotters, charming, egui_graphs |
| [`tui-patterns.md`](../Interface/tui-patterns.md) | 12 tui apps e o oh-my-zsh |
| [`emulators-wasm-runtimes.md`](emulators-wasm-runtimes.md) | 5 runtimes WASM |
| [`competitive-positioning.md`](competitive-positioning.md) | matriz comparativa |
| [`integration-candidates.md`](integration-candidates.md) | ranking adopt/evaluate/watch/hold |

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-27: animation system](../../Tasks/task-27-animation-system/task-27-animation-system.md)
- [task-28: editable text](../../Tasks/task-28-editable-text/task-28-editable-text.md)
- [task-29: app demo real (proof of life)](../../Tasks/task-29-app-demo-real/task-29-app-demo-real.md)
- [task-30: accessibility (accesskit)](../../Tasks/task-30-accessibility/task-30-accessibility.md)
- [task-31: vector path rendering (lyon integration)](../../Tasks/task-31-vector-paths/task-31-vector-paths.md)
- [task-32: text system upgrade assessment](../../Tasks/task-32-text-upgrade-assessment/task-32-text-upgrade-assessment.md)
- [task-33: WASM plugin architecture](../../Tasks/task-33-wasm-plugins/task-33-wasm-plugins.md)
- [task-ref: research briefing (50+ repos)](../../Tasks/task-ref-research-briefing/task-ref-research-briefing.md)
- [reference analysis: accessibility (accesskit)](../Interface/accessibility.md)
- [reference analysis: animation and motion](../Rendering/animation-motion.md)
- [reference analysis: charts and visualization](../Rendering/charts-visualization.md)
- [competitive positioning, plev](competitive-positioning.md)
- [competitor analysis, gui/rendering frameworks](competitors.md)
- [competitors by tier](competitors-by-tier.md)
- [reference analysis: emulators and WASM runtimes](emulators-wasm-runtimes.md)
- [integration candidates, ranking](integration-candidates.md)
- [linebender ecosystem, analysis for plev](linebender-ecosystem.md)
- [reference analysis: math, physics and geometry](../Rendering/math-physics-geometry.md)
- [reference analysis: tui patterns](../Interface/tui-patterns.md)
- [reference analysis: WASM tooling ecosystem](wasm-tooling.md)
<!-- /related -->
