---
id: learning-cycle-0046
title: "what the autonomous cycle taught: the inspector reads a session, it never runs it"
project: plev
type: learning
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-09-23
topics: [recording, semantics, agents]
relations: [adr-0046, task-64, task-89]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: the inspector reads a session, it never runs it

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0046](0046-the-inspector-reads-a-session-never-runs-it.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-23, task-64 fase a e o buraco que ela achou

- **a inspeção pagou na primeira leitura:** lendo a sessão do A2 da
  primeira rodada, o inspector mostrou um submit apoiado numa observação
  só de `commit.form`, com `changes:staged` fora do escopo, e o servidor
  dizendo que o agente tinha visto aquele valor. a promessa escrita no
  ADR ("the stale check compares what the agent saw") não valia com
  observação por escopo. nenhum teste, nenhuma revisão de diff e nenhum
  dos dois experimentos tinha pego isso: o experimento passou porque a
  pessoa agiu depois da observação com escopo; se tivesse agido antes, o
  commit errado teria saído. virou a task-89.
- **pressuposto errado:** "o anel guarda o que foi servido" foi lido como
  "guarda o que o agente viu". o anel guarda o snapshot inteiro; o que a
  conexão viu é um recorte dele, e ninguém registrava o recorte.
- **a pergunta do núcleo começa a ter resposta:** a CLI respondeu, sobre
  as nove sessões gravadas, qual controle, quem observou o quê, o que
  mudou no intervalo e quem mudou, o veredito e o resultado. a interface
  visual (fase b) ainda falta; o checkpoint c pergunta se isso basta para
  confiar numa delegação.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0046: the inspector reads a session, it never runs it](0046-the-inspector-reads-a-session-never-runs-it.md)
- [task-64: session inspector, bug report and session for testing](../../../Tasks/task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-89: an act relies only on what was shown to the agent](../../../Tasks/task-89-an-act-rests-only-on-what-was-shown/task-89-an-act-rests-only-on-what-was-shown.md)
<!-- /related -->
