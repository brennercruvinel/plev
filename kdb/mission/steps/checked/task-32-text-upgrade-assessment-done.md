---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2022-08-16
domain: task-tracking
---

# task-32: text system upgrade assessment, done

## resultado
documento completo: `mission/knowledge/parley-vs-cosmic-text.md`

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
