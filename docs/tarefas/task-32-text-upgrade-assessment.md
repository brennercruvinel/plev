---
id: task-32
title: "task-32: text system upgrade assessment"
project: plev
type: tarefa
status: concluido
date: 2022-08-16
date_basis: cabecalho
topics: [texto, pesquisa]
relations: [pesquisa-parley-vs-cosmic-text]
source: [docs/log/steps/task-32-text-upgrade-assessment.md, docs/log/changes/task-32.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-32: text system upgrade assessment

Registra a avaliação de migrar o sistema de texto do plev do cosmic-text 0.18 para o parley 0.7. A conclusão foi esperar: o parley tem APIs melhores (cursor, seleção, inlinebox e ponte para accesskit), mas está em alpha, não tem rasterizador e o suporte a WASM tem issue aberta. A página lista os gatilhos que reabrem a decisão; a comparação completa está em [parley-vs-cosmic-text.md](../pesquisas/parley-vs-cosmic-text.md).

## resultado
documento completo: [`parley-vs-cosmic-text.md`](../pesquisas/parley-vs-cosmic-text.md)

## conclusao
**wait**, nao migrar para parley agora. re-avaliar em parley 0.9+.

parley tem apis superiores (cursor, selection, inlinebox, accesskit bridge), mas:
- self-described "alpha", breaking changes a cada release
- sem rasterizador (swashcache nao incluido, precisaria skrifa ou manter swash)
- WASM support tem issue aberto (#70), sem exemplos oficiais
- ambos usam harfrust (sem vantagem de shaping)
- custo migracao: ~300 LOC tocando path critico (text.rs)

### quando migrar
- parley 0.9+ com API estabilizada
- issue #70 (WASM) fechada
- skrifa com capacidade de bitmap masks para atlas
- ou quando plev precisar de inlinebox / accesskit text bridge

## checklist
- [x] comparar API de cursor: parley global byte index + geometry() vs cosmic-text line-local
- [x] comparar selection: parley geometry_with (zero-alloc) vs cosmic-text opaque editor.draw
- [x] inlinebox: parley tem, cosmic-text nao
- [x] harfrust: ambos usam (puro rust, sem harfbuzz c)
- [x] WASM: cosmic-text maduro, parley funcional mas issue aberto
- [x] estabilidade: cosmic-text 0.18 (cosmic de), parley 0.7 "alpha"
- [x] rasterizacao: cosmic-text inclui swashcache, parley layout-only
- [x] accesskit: parley tem bridge builtin, cosmic-text nao
- [x] custo migracao: ~300 LOC, text.rs + text_input.rs
- [x] recomendacao: wait com decision matrix

## Registro de entrega

### pesquisa concluida
- documento: [`parley-vs-cosmic-text.md`](../pesquisas/parley-vs-cosmic-text.md)
- 13 secoes comparando cosmic-text 0.18 vs parley 0.7
- recomendacao: wait, nao migrar agora
- decision matrix com triggers para migracao futura
