---
id: task-ref
title: "task-ref: research briefing (50+ repos)"
project: plev
type: task
status: done
date: 2022-06-15
date_basis: reconstructed
recorded: 2026-03-11
topics: [research, competition]
relations: [task-30, task-31, task-32, task-33, research-competitors, research-linebender-ecosystem, research-accessibility, research-animation-motion, research-math-physics-geometry, research-wasm-tooling, research-charts-visualization, research-tui-patterns, research-emulators-wasm-runtimes, research-competitive-positioning, research-integration-candidates, research-technology-radar]
source: [docs/log/steps/task-ref-research-briefing.md, docs/log/changes/task-ref.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-ref: research briefing (50+ repos)

Records the reference research completed on 2022-06-15: the analysis of more than 50 repositories, organized into 12 categories, to inform plev's architecture and integration decisions. It delivered 12 reference documents, a technology radar (adopt, evaluate, watch, hold) and four derived tasks, from task-30 to task-33. The result and the execution phases are in the delivery record below.

## status: done (2022-06-15)

## objetivo
analisar 50+ repositorios de referencia e criar documentacao estruturada em 12 categorias para informar decisoes de arquitetura e integracao do plev.

## resultado
- 12 documentos de referencia em `mission/knowledge/refs/`
- technology radar (adopt/evaluate/watch/hold)
- 4 novas tasks derivadas: task-30 (a11y), task-31 (lyon), task-32 (parley), task-33 (WASM plugins)
- 56 repos clonados (shallow) em bunker/repos/

## Delivery record

### progresso

#### fase a: setup
- [x] branch criada
- [x] mission/knowledge/refs/ criado
- [x] refs/index.md template criado

#### fase b: categorias criticas
- [x] refs/competitors.md
- [x] refs/linebender-ecosystem.md
- [x] refs/accessibility.md

#### fase c: categorias de suporte
- [x] refs/animation-motion.md
- [x] refs/math-physics-geometry.md
- [x] refs/wasm-tooling.md
- [x] refs/charts-visualization.md
- [x] refs/tui-patterns.md
- [x] refs/emulators-wasm-runtimes.md

#### fase d: sintese
- [x] refs/competitive-positioning.md
- [x] refs/integration-candidates.md
- [x] refs/technology-radar.md

#### fase e: integracao com mission
- [x] mission/knowledge/index.md atualizado
- [x] mission/plan/readme.md atualizado (secoes 12, 13, + makepad na 5/6)
- [x] mission/readme.md atualizado (task-30..33, secao pesquisa de referencia)
- [x] task-30..33 .md criados em pending/
- [x] mission/rules.md atualizado (4 novas regras)
- [x] claude.md atualizado

#### fase f: finalizacao
- [x] refs/index.md final
- [x] cargo test --workspace passou (249 tests, 0 failures)
- [x] commit feito (27a3896)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-30: accessibility (accesskit)](../task-30-accessibility/task-30-accessibility.md)
- [task-31: vector path rendering (lyon integration)](../task-31-vector-paths/task-31-vector-paths.md)
- [task-32: text system upgrade assessment](../task-32-text-upgrade-assessment/task-32-text-upgrade-assessment.md)
- [task-33: WASM plugin architecture](../task-33-wasm-plugins/task-33-wasm-plugins.md)
- [task-34: exploration and extraction of ideas from the reference repos](../task-34-exploration-extraction/task-34-exploration-extraction.md)
- [reference analysis: accessibility (accesskit)](../../Research/Interface/accessibility.md)
- [reference analysis: animation and motion](../../Research/Rendering/animation-motion.md)
- [reference analysis: charts and visualization](../../Research/Rendering/charts-visualization.md)
- [competitive positioning, plev](../../Research/Ecosystem/competitive-positioning.md)
- [competitor analysis, gui/rendering frameworks](../../Research/Ecosystem/competitors.md)
- [reference analysis: emulators and WASM runtimes](../../Research/Ecosystem/emulators-wasm-runtimes.md)
- [integration candidates, ranking](../../Research/Ecosystem/integration-candidates.md)
- [linebender ecosystem, analysis for plev](../../Research/Ecosystem/linebender-ecosystem.md)
- [reference analysis: math, physics and geometry](../../Research/Rendering/math-physics-geometry.md)
- [technology radar, plev](../../Research/Ecosystem/technology-radar.md)
- [reference analysis: tui patterns](../../Research/Interface/tui-patterns.md)
- [reference analysis: WASM tooling ecosystem](../../Research/Ecosystem/wasm-tooling.md)
<!-- /related -->
