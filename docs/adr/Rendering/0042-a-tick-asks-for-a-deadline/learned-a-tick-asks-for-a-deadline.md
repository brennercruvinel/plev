---
id: learning-cycle-0042
title: "what the autonomous cycle taught: a tick asks for a deadline, not for every frame"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [animation, rendering, performance]
relations: [adr-0042, task-82]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a tick asks for a deadline, not for every frame

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0042](0042-a-tick-asks-for-a-deadline.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-23, task-82 fase b

- **a promessa do README era falsa num lugar que ninguém mediu:** o ide
  parado num repositório comum desenhava para sempre, porque o `git
  status` criava o lock opcional do index, o watcher via, e o refresh
  rodava `git status` de novo. só apareceu quando alguém contou frames e
  processos numa janela de verdade com o app parado. dois cortes
  independentes fecharam: leitura sem lock opcional, e o watcher ignora o
  lock (outro `git` na máquina ainda cria).
- **render on demand pede que o tempo alcance o evento:** com o shell
  dormindo até o prazo, o primeiro tick depois de um evento recebia o
  intervalo inteiro de ócio, e toda mola iniciada por um evento pulava
  para o fim. um tick de alcance quando o loop acorda resolve para todos
  os widgets de uma vez; a correção por widget era um sintoma.

## 2026-09-24, a lição do item 9 chegou ao item 10

- **o briefing citou o `print_wake`, e o agente aplicou:** o estado
  "começando" da câmera conta como vivo e pede prazo ao laço, e o frame
  entra só no wake visível. escrever o achado da revisão anterior no
  briefing seguinte evitou o mesmo defeito.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0042: a tick asks for a deadline, not for every frame](0042-a-tick-asks-for-a-deadline.md)
- [task-82: the caret cannot request 60 frames per second](../../../Tasks/task-82-caret-blink-render-on-demand/task-82-caret-blink-render-on-demand.md)
<!-- /related -->
