---
id: learning-cycle-0052
title: "what the autonomous cycle taught: proposers propose, the runtime verifies"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [generated-surfaces, ir, llm, agents, semantics]
relations: [adr-0052, task-69]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: proposers propose, the runtime verifies

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0052](0052-proposers-propose-the-runtime-verifies.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-69 fase 3a

- **gerar sem template funcionou pela estrutura do registro:** o
  proponente por regras monta a superfície a partir de roles, tipos de
  valor, parâmetros e parents, e as palavras da intenção só escolhem
  entre as do app. as mesmas regras, sem mudança, leem o todo do
  showcase e o workbench do ide. o limite apareceu onde o registro é
  pobre: "os itens ativos" não sai dele porque o todo não projeta os
  itens como lista (P2). o que falta a um proponente é informação no
  contrato, não regra no proponente.
- **o autor da proposta é quem pediu:** a página verifica a proposta sob
  a policy de quem pediu, então um agente só gera superfície com o que
  agentes podem pedir. a verificação usa o mesmo contrato que já decide
  cada ato.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0052: proposers propose, the runtime verifies](0052-proposers-propose-the-runtime-verifies.md)
- [task-69: ephemeral and generated surfaces](../../../Tasks/task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
<!-- /related -->
