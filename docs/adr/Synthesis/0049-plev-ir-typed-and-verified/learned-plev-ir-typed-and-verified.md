---
id: learning-cycle-0049
title: "what the autonomous cycle taught: the plev IR is typed, references only declared capabilities, and runs only once verified"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [ir, generated-surfaces, semantics, design-system]
relations: [adr-0049, task-68]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: the plev IR is typed, references only declared capabilities, and runs only once verified

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0049](0049-plev-ir-typed-and-verified.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-68

- **o IR é pequeno porque a projeção já declara tudo:** o registro de
  capacidades sai do `Snapshot` que o app já projeta para agentes e
  leitor de tela. o IR só pode referenciar o que existe, sem uma segunda
  declaração que envelheça. o contrato semântico virou o espaço formal
  da geração, como a tese pedia.
- **o vocabulário do comps não é todo semântico:** 9 dos 21 componentes
  não têm `project`. para a interface gerada ser operável pelo mesmo
  contrato, a materialização (69) precisa que cada componente se projete.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0049: the plev IR is typed, references only declared capabilities, and runs only once verified](0049-plev-ir-typed-and-verified.md)
- [task-68: plev IR and verifier](../../../Tasks/task-68-plev-ir-and-verifier/task-68-plev-ir-and-verifier.md)
<!-- /related -->
