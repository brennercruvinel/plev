---
id: learning-cycle-0037
title: "what the autonomous cycle taught: session recording: replay, not re-execution"
project: plev
type: learning
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-09-23
topics: [recording, formats, semantics, rendering]
relations: [adr-0037, task-83]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: session recording: replay, not re-execution

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0037](0037-session-recording-replay-not-reexecution.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-23
- **premissa de compatibilidade errada no briefing:** o briefing da
  task-83 afirmou que nenhum arquivo gravado continha `node_added`,
  conferido só no `Kind::Changes` do recorder. o `Kind::Snapshot` guarda o
  diff do passo, com `node_added`, e as sessões do checkpoint a deixariam
  de abrir. o agente achou abrindo as sessões de verdade. regra: toda
  afirmação de compatibilidade num briefing se confere contra os
  artefatos gravados de verdade (abrir os arquivos), não só lendo o
  código do escritor.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0037: session recording: replay, not re-execution](0037-session-recording-replay-not-reexecution.md)
- [task-83: the delta carries the node that was just added, and the refusal names who acted](../../../Tasks/task-83-delta-carries-node-refusal-names-actor/task-83-delta-carries-node-refusal-names-actor.md)
<!-- /related -->
