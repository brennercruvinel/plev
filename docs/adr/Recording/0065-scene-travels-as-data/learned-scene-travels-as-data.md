---
id: learning-cycle-0065
title: "what the autonomous cycle taught: the scene travels as data"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [agents, rendering, recording, semantics]
relations: [adr-0065, task-79]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: the scene travels as data

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0065](0065-scene-travels-as-data.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, um canal novo herda as promessas dos antigos

- **o canal de cena reabriu o que o item 10 fechou:** o briefing da
  task-79 falou de segurança de rede (localhost, pareamento), mas não
  lembrou que todo cliente do port é agente e que a câmera tinha
  promessa própria (pixels nunca vão para agente). o agente seguiu o
  briefing e o teste dele provou o vazamento. regra para briefing de
  canal ou de saída nova: listar as promessas de privacidade já
  aceitas nos ADRs (câmera, redação do `sem`, só pessoa) e exigir um
  teste de cada uma contra o canal novo.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0065: the scene travels as data](0065-scene-travels-as-data.md)
- [task-79: scene over socket](../../../Tasks/task-79-scene-over-socket/task-79-scene-over-socket.md)
<!-- /related -->
