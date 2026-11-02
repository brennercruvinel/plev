---
id: learning-cycle-0060
title: "what the autonomous cycle taught: a covered window does not draw, and wakes with a new frame"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [rendering, platform, performance]
relations: [adr-0060, task-76]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a covered window does not draw, and wakes with a new frame

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0060](0060-a-covered-window-does-not-draw-and-wakes-new.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-76 item 7

- **a janela coberta desenhava para ninguém:** o `request_redraw` do
  winit entrega `RedrawRequested` mesmo com a janela minimizada ou
  coberta, e `Occluded` não era lido por nenhum shell; uma página que
  anima gastava GPU sem tela, e a textura guardava o frame velho
  enquanto o estado andava. agora um host não acordado retém o pedido e
  acorda com o estado de agora. o sleep do sistema não vem pelo winit
  (é `NSWorkspace`, atrás de feature que ninguém liga): fica para o
  bring-up. fonte: `task-76-platform-integration-done.md`, handoff do item 7.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0060: a covered window does not draw, and wakes with a new frame](0060-a-covered-window-does-not-draw-and-wakes-new.md)
- [task-76: platform integration](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)
<!-- /related -->
