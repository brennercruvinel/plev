---
id: learning-cycle-0026
title: "what the autonomous cycle taught: background results wake the event loop; child pipes drain from the start"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [urna, performance, rendering]
relations: [adr-0026]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: background results wake the event loop; child pipes drain from the start

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0026](0026-worker-results-wake-the-event-loop.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, trabalho de fundo precisa acordar o laço

- **o teste chamava o `settle` direto e não via o laço dormindo:** no
  item 9, os testes do job de impressão liam a resposta chamando
  `settle_print` num loop, então passavam. na janela real, com render on
  demand e nada animando, ninguém chamava o `settle` até o próximo
  input. a revisão do diff achou pela ausência de um `Wake::after` onde
  a Surface e a Silences têm um. regra para briefing e revisão: todo
  trabalho que responde por canal ao lado do laço pede um prazo no
  `Wake` enquanto está fora, e o teste prova o prazo, não só a resposta.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0026: background results wake the event loop; child pipes drain from the start](0026-worker-results-wake-the-event-loop.md)
<!-- /related -->
