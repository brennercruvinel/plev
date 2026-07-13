---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2023-12-06
domain: task-tracking
---

# task-40: paper arxiv, texto completo

## objetivo
escrever o texto completo do paper "plev: a GPU-first compositing engine for cross-platform UI rendering" para submissao ao arxiv.

## contexto
- outline completo disponivel: `mission/knowledge/arxiv-paper-outline.md`
- benchmarks prontos: `mission/knowledge/benchmark-results.md`
- posicionamento competitivo: `mission/knowledge/refs/competitive-positioning.md`
- ~14.500 LOC, 370+ testes, 18 examples, material tecnico solido

## dependencias
- license definida (MIT or apache-2.0), necessario para citar codebase publica
- WASM visual validation (task-20), ideal ter screenshot para fig. 1

## checklist

### estrutura (11 secoes do outline)
- [ ] abstract, 150 palavras, metricas concretas (ja rascunhado no outline)
- [ ] 1. introduction, problema, solucoes existentes, gap, contribuicao
- [ ] 2. related work, tabela + paragrafos (dados do refs/competitors.md)
- [ ] 3. architecture, frame lifecycle, scene graph, dois pipelines, text system, gpuvec, effects
- [ ] 4. cross-platform strategy, wgpu abstraction, android/ios/WASM especificos
- [ ] 5. verbal DSL plev_narrate!, gramatica EBNF, codegen, levenshtein DX
- [ ] 6. accessibility, accesskit lazy activation, focusgraph, WASM null adapter
- [ ] 7. vector paths, lyon + fillvertexconstructor, dirty tracking integration
- [ ] 8. reactive primitives, signal system, RAII guard, fxindexset, peek()
- [ ] 9. evaluation, criterion methodology, tabela de benchmarks, test coverage
- [ ] 10. limitations and future work, sem widget lib (intencional), parley pendente, WASM visual
- [ ] 11. conclusion

### producao
- [ ] formato latex (arxiv standard) ou markdown (pre-print)
- [ ] figuras: diagrama de arquitetura (ja tem mermaid no readme), frame lifecycle, benchmark charts
- [ ] revisar e validar todas as metricas contra codigo atual
- [ ] submeter ao arxiv cs.hc ou cs.pl

## notas
- tom: tecnico, factual, sem marketing. descrever decisoes e tradeoffs com justificativa.
- audiencia: pesquisadores e engenheiros de sistemas de UI em rust
- citacoes chave: wgpu, cosmic-text, accesskit, lyon, taffy, criterion
- evitar claims nao-quantificados ("fast", "easy"), substituir por numeros ou "we defer measurement to future work"
