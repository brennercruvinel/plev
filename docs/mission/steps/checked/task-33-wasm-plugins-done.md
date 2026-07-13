---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2022-09-14
domain: task-tracking
---

# task-33: WASM plugin architecture, done (phase 1 research)

## resultado
research doc completo: `mission/knowledge/wasm-plugin-architecture.md`

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
