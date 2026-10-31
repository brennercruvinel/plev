---
id: task-33
title: "task-33: WASM plugin architecture"
project: plev
type: task
status: done
date: 2023-07-11
date_basis: reconstructed
recorded: 2026-03-13
topics: [web-wasm, research]
relations: [research-wasm-plugin-architecture]
source: [docs/log/steps/task-33-wasm-plugins.md, docs/log/changes/task-33.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-33: WASM plugin architecture

Records phase 1 of the WASM plugin evaluation for plev: wasmtime, wasmer and extism compared, with overhead of 10 to 15 ns per host call, according to the page body. The recommendation is to wait, because the API still changes and WASM works well on only 3 of 6 platforms; once it stabilizes, the path is extism behind a feature, desktop only. The prototype (phase 2) was postponed; the full comparison is in [wasm-plugin-architecture.md](../../research/wasm-plugin-architecture.md).

## resultado
research doc completo: [`wasm-plugin-architecture.md`](../../research/wasm-plugin-architecture.md)

## conclusao
**wait**, plugin system e P4, API ainda evoluindo, e WASM plugins so funcionam bem em 3 de 6 platforms. quando pronto, usar extism com host function interface, feature-gated, desktop-only inicialmente.

## checklist (fase 1)
- [x] estudar extism, wasmtime, wasmer
- [x] avaliar chamada WASM->host (~10-15ns/call)
- [x] avaliar shared memory vs host functions
- [x] documentar viabilidade
- [x] conclusao: viavel para desktop, problematico para mobile/WASM
- [x] recomendacao: extism quando API estabilizar

## fase 2 (prototipo)
adiada, depende de API estavel e decisao de plataformas-alvo.

## Delivery record

### pesquisa concluida (fase 1)
- documento: [`wasm-plugin-architecture.md`](../../research/wasm-plugin-architecture.md)
- comparacao: wasmtime vs wasmer vs extism
- interface draft: host functions (plev_push_rect/text/path)
- overhead: 10-15ns/host call (negligivel)
- recomendacao: wait (API instavel, 3/6 plataformas)
- fase 2 (prototipo) adiada
