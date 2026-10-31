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
