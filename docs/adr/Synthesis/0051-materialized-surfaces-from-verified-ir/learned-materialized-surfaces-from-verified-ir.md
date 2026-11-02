---
id: learning-cycle-0051
title: "what the autonomous cycle taught: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [generated-surfaces, ir, semantics, layout, agents]
relations: [adr-0051, task-69]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0051](0051-materialized-surfaces-from-verified-ir.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-69 fase 1

- **uma interface que nunca foi escrita como tela funciona pelo mesmo
  contrato:** a superfície do todo, materializada de um JSON verificado,
  opera o mesmo estado que o todo escrito à mão, pelo mesmo dispatcher, e
  um agente a observa e age nela pela porta como em qualquer página. do
  IR verificado à cena do primeiro frame leva menos de um milissegundo em
  debug; o que pesa é o texto e a GPU, como em qualquer frame.
- **o IR prometia mais que o widget cumpre:** dois props passam no
  verifier e o widget ignora. o verifier deve recusar o que a
  materialização não honra, senão o "só o verificado existe" vira "o
  verificado existe, em parte".

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0051: a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher](0051-materialized-surfaces-from-verified-ir.md)
- [task-69: ephemeral and generated surfaces](../../../Tasks/task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
<!-- /related -->
