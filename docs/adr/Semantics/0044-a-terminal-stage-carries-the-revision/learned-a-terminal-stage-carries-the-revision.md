---
id: learning-cycle-0044
title: "what the autonomous cycle taught: a terminal stage carries the revision the app is at"
project: plev
type: learning
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-09-23
topics: [agents, semantics]
relations: [adr-0044, task-87, task-84]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a terminal stage carries the revision the app is at

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0044](0044-a-terminal-stage-carries-the-revision.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-23, task-87

- **o custo de idas e voltas era o protocolo, não o contrato:** com a
  revisão no estágio terminal, o agente roteirizado com intervenção caiu
  de 24 para 17 mensagens e de 91,5 para 45,9 KB, sem perder a recusa
  stale nem a recuperação. a derrota do plev em bytes e idas e voltas
  (H4) encolheu pela metade no laboratório; contra o `git` direto (1,5 KB
  em 20 invocações) continua perdendo em bytes, e isso é a observação
  inteira do ide, que é grande.
- **a revisão do estágio nunca enganou:** nas 25 vezes em que um ato foi
  seguido por uma observação, ela serviu a mesma revisão. só é verdade
  porque a task-84 e a 85 fizeram o ide mostrar o efeito no passo em que
  o ato termina: uma correção de invariante habilitou uma otimização.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0044: a terminal stage carries the revision the app is at](0044-a-terminal-stage-carries-the-revision.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../../../Tasks/task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
- [task-87: a terminal stage carries the revision the app was left in](../../../Tasks/task-87-a-terminal-stage-carries-the-revision/task-87-a-terminal-stage-carries-the-revision.md)
<!-- /related -->
