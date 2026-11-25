---
id: learning-cycle-0050
title: "what the autonomous cycle taught: sessions become regression tests"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [recording, process, semantics]
relations: [adr-0050, task-80, task-64, task-84]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: sessions become regression tests

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0050](0050-sessions-become-regression-tests.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-25, task-80 fase 1

- **um teste que espera tudo assentar não vê corrida:** o rerun da
  task-64 drenava toda leitura antes de cada ato, e com isso a corrida da
  task-84 passava. só uma fixture que deixa as leituras em voo, como a
  janela deixa, pegou o bug. regra: teste de concorrência reproduz o
  ritmo de quem usa, não o repouso. fonte:
  `apps/idestudio/tests/rerun/inflight.rs`.
- **a gravação semântica vê o que o vídeo não vê:** os lapsos da corrida
  duraram 29 e 24 ms, menos que um quadro a 30 fps; o `plev-inspect`
  respondeu as oito perguntas da tese só pelo arquivo. fonte: a demo 3 no
  arquivo da task-80.

## 2026-09-24, task-64 fase c

- **uma sessão real virou teste que pega regressão:** a sessão de um
  agente LLM com a pessoa agindo antes do submit, reaplicada num
  repositório novo, deu os mesmos vereditos. com a precondição do commit
  quebrada, o teste nomeou o request que mudou de veredito. o contrato
  torna a reexecução determinística onde importa: o request carrega o
  que o ator viu, e o veredito depende só disso e do estado.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0050: sessions become regression tests](0050-sessions-become-regression-tests.md)
- [task-64: session inspector, bug report and session for testing](../../../Tasks/task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-80: proof of qualitatively different software](../../../Tasks/task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../../../Tasks/task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
<!-- /related -->
