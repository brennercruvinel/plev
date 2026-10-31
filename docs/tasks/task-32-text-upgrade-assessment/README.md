---
id: task-32
title: "task-32: text system upgrade assessment"
project: plev
type: task
status: done
date: 2023-06-25
date_basis: reconstructed
recorded: 2026-03-13
topics: [text, research]
relations: [research-parley-vs-cosmic-text]
source: [docs/log/steps/task-32-text-upgrade-assessment.md, docs/log/changes/task-32.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-32: text system upgrade assessment

Records the assessment of migrating plev's text system from cosmic-text 0.18 to parley 0.7. The conclusion was to wait: parley has better APIs (cursor, selection, inlinebox and a bridge to accesskit), but it is in alpha, has no rasterizer, and its WASM support has an open issue. The page lists the triggers that reopen the decision; the full comparison is in [parley-vs-cosmic-text.md](../../research/parley-vs-cosmic-text.md).

## resultado
documento completo: [`parley-vs-cosmic-text.md`](../../research/parley-vs-cosmic-text.md)

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

## Delivery record

### pesquisa concluida
- documento: [`parley-vs-cosmic-text.md`](../../research/parley-vs-cosmic-text.md)
- 13 secoes comparando cosmic-text 0.18 vs parley 0.7
- recomendacao: wait, nao migrar agora
- decision matrix com triggers para migracao futura
