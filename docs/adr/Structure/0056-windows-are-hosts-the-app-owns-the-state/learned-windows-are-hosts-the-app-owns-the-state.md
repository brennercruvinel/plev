---
id: learning-cycle-0056
title: "what the autonomous cycle taught: a window is a host of what dies with it; the app owns the state"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [rendering, gpu, platform]
relations: [adr-0056, task-76]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a window is a host of what dies with it; the app owns the state

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0056](0056-windows-are-hosts-the-app-owns-the-state.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-76 itens 2 e 3

- **a segunda janela fechava a primeira:** no shell antigo do showcase,
  um segundo `create_window` sobrescrevia o `Arc<Window>`, o `Drop` do
  macOS fechava a primeira janela e os eventos dela passavam a ser
  roteados como da segunda. o `WindowId` que o winit entrega em todo
  evento era ignorado por todos os shells. agora cada janela é um host
  com o próprio contexto e o app roteia por id.
- **o drop de arquivo não tem posição em plataforma nenhuma:** o winit
  0.30 entrega `HoveredFile` e `DroppedFile` sem coordenadas e não manda
  `CursorMoved` durante o arraste; a posição fica opcional até ele expor.
  fonte: `task-76-platform-integration-done.md`, handoffs dos itens 2 e 3.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0056: a window is a host of what dies with it; the app owns the state](0056-windows-are-hosts-the-app-owns-the-state.md)
- [task-76: platform integration](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)
<!-- /related -->
