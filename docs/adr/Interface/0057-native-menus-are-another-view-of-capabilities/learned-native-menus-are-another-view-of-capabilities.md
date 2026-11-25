---
id: learning-cycle-0057
title: "what the autonomous cycle taught: a native menu is another view of the app's capabilities"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [platform, semantics, agents, input]
relations: [adr-0057, task-76]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a native menu is another view of the app's capabilities

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0057](0057-native-menus-are-another-view-of-capabilities.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-76 itens 4, 5 e 8

- **fora da janela, o macOS cobra o bundle:** menu de barra, diálogo e
  tray funcionam num binário de `cargo run`; notificação do sistema
  (`UNUserNotificationCenter`) e esquema de URL exigem um `.app` com
  `CFBundleIdentifier`, senão o processo estoura. o engine responde
  `NeedsBundle` e o bundle vira item do bring-up de plataforma. criar um
  tray mostra o ícone na hora, então nenhum teste cria um real: o falso
  é o único caminho de teste, como nos diálogos.
- **o diálogo nativo bloqueia o laço:** o `act` de um agente pela porta
  só volta quando a pessoa fecha o diálogo, por isso o modal desenhado
  continua o padrão e o nativo é opção. fonte:
  `task-76-platform-integration-done.md`, handoffs dos itens 4, 5 e 8.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0057: a native menu is another view of the app's capabilities](0057-native-menus-are-another-view-of-capabilities.md)
- [task-76: platform integration](../../../Tasks/task-76-platform-integration/task-76-platform-integration.md)
<!-- /related -->
