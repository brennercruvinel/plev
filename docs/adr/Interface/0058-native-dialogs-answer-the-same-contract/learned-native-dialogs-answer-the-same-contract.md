---
id: learning-cycle-0058
title: "what the autonomous cycle taught: a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [platform, semantics, agents]
relations: [adr-0058, task-92]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0058](0058-native-dialogs-answer-the-same-contract.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-92

- **o handoff ganhou a peça que faltava:** um agente que encontra a
  tarefa em conflito com o que a pessoa fez pode perguntar a ela, pelo
  mesmo modal que o `discard` já usava. a pessoa responde, e o agente
  recebe a resposta como estágio. ainda sem experimento: o laboratório
  precisa de uma pessoa roteirizada que responda.

## 2026-09-23
- **teste do caminho inteiro pega o que teste de peça não pega:** a
  recusa de confirmação (58b) foi testada no `sem` e no ide, e o porto foi
  testado com atos que terminam pelo git. ninguém testou a recusa pela
  porta real com o shell servindo só no wake. a sonda da 92, que
  atravessou tudo, achou o estágio parado. um contrato que atravessa
  crates pede um teste pelo caminho inteiro, porta real e host real.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0058: a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline](0058-native-dialogs-answer-the-same-contract.md)
- [task-92: the agent asks the person](../../../Tasks/task-92-the-agent-asks-the-person/task-92-the-agent-asks-the-person.md)
<!-- /related -->
