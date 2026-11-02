---
id: learning-cycle-0061
title: "what the autonomous cycle taught: printing is the frame on paper"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [platform, rendering, agents]
relations: [adr-0061, task-76]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: printing is the frame on paper

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0061](0061-printing-is-the-frame-on-paper.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-76 item 9

- **a página vai para o papel pelo mesmo caminho da tela:** o render sem
  janela a 300 dpi mantém o layout e rasteriza o texto na escala física
  (a tinta de um texto cresce na mesma razão da escala), então imprimir
  não precisa de um segundo renderer. o svg do mesmo frame já tem a
  geometria de um PDF vetorial; falta só o escritor. fonte:
  `task-76-platform-integration-done.md`, handoff do item 9.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0061: printing is the frame on paper](0061-printing-is-the-frame-on-paper.md)
- [task-76: platform integration](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)
<!-- /related -->
