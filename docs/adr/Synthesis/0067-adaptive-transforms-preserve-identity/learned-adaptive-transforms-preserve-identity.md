---
id: learning-cycle-0067
title: "what the autonomous cycle taught: adaptive transformations are small verified operations that preserve identity"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [ir, generated-surfaces, semantics, agents]
relations: [adr-0067, task-70, task-69]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: adaptive transformations are small verified operations that preserve identity

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0067](0067-adaptive-transforms-preserve-identity.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-25, task-70 fase 4

- **privacidade e accessibility puxam para lados opostos na mesma
  projeção:** a preferência da pessoa não pode ir para o agente, e a
  projeção que vai para o agente é a que vai para o leitor de tela; tirar
  o estado para um tira para o outro. regra: um dado pessoal que o leitor
  precisa entra só na árvore do AccessKit, nunca no snapshot do port.
  fonte: `apps/uigallery/src/views/apps/silences/generative/access.rs`.
- **preferência é camada, não passo do histórico:** guardar a superfície
  como foi pedida e refazer as preferências por cima depois de toda
  mudança resolve o desligar sem desfazer o que outros pediram, coisa que
  uma pilha de desfazer não faz. fonte: `rust/screen/src/host/prefer.rs`.

## 2026-09-24, task-70 fase 1

- **um gráfico que passa no verifier pode mentir:** o `Chart` aceita
  qualquer `List`, e o único da Silences são os ids dos cortes ("1",
  "2"). o verifier checa o tipo, não o sentido; o gráfico desenharia ids
  como barras. regra: trocar uma vista exige que o dado seja o que a
  vista nova mostra, e isso vem do registro do app, não do verifier.
  fonte: handoff da fase 1, sonda (a).

## 2026-09-24, task-69 fase 2

- **o mantra funciona numa superfície gerada:** a mesma capacidade
  mantém a instância e o estado (foco, caret, arraste) enquanto a
  representação muda de forma. identidade por `SemId` foi o que
  bastou: nada no comps ou no engine precisou mudar. a continuidade
  depende de o proponente (fase 3) manter os ids das capacidades que
  continuam.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0067: adaptive transformations are small verified operations that preserve identity](0067-adaptive-transforms-preserve-identity.md)
- [task-69: ephemeral and generated surfaces](../../../Tasks/task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-70: adaptive transformations](../../../Tasks/task-70-adaptive-transformations/task-70-adaptive-transformations.md)
<!-- /related -->
