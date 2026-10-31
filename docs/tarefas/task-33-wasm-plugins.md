---
id: task-33
title: "task-33: WASM plugin architecture"
project: plev
type: tarefa
status: concluido
date: 2022-09-14
date_basis: cabecalho
topics: [web-wasm, pesquisa]
relations: [pesquisa-wasm-plugin-architecture]
source: [docs/log/steps/task-33-wasm-plugins.md, docs/log/changes/task-33.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-33: WASM plugin architecture

Registra a fase 1 da avaliação de plugins WASM para o plev: wasmtime, wasmer e extism comparados, com overhead de 10 a 15 ns por chamada de host, segundo o corpo. A recomendação é esperar, porque a API ainda muda e WASM funciona bem só em 3 de 6 plataformas; quando estabilizar, o caminho é extism atrás de uma feature, só para desktop. O protótipo (fase 2) ficou adiado; a comparação completa está em [wasm-plugin-architecture.md](../pesquisas/wasm-plugin-architecture.md).

## resultado
research doc completo: [`wasm-plugin-architecture.md`](../pesquisas/wasm-plugin-architecture.md)

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

## Registro de entrega

### pesquisa concluida (fase 1)
- documento: [`wasm-plugin-architecture.md`](../pesquisas/wasm-plugin-architecture.md)
- comparacao: wasmtime vs wasmer vs extism
- interface draft: host functions (plev_push_rect/text/path)
- overhead: 10-15ns/host call (negligivel)
- recomendacao: wait (API instavel, 3/6 plataformas)
- fase 2 (prototipo) adiada
