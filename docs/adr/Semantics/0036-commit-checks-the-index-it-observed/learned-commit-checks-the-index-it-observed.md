---
id: learning-cycle-0036
title: "what the autonomous cycle taught: a commit checks the index it observed"
project: plev
type: learning
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-09-23
topics: [semantics, git, agents]
relations: [adr-0036, task-85, task-84, task-83, task-64]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a commit checks the index it observed

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0036](0036-commit-checks-the-index-it-observed.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-23, task-85

- **o receipt e a projeção podem se contradizer sem ninguém mentir:** o
  receipt dizia "commitado" e a projeção dizia "ainda staged, sem commit"
  até o refresh. cada um estava certo no seu tempo; o agente que
  confere logo depois via um mundo que contradizia o receipt que acabou
  de receber. regra que ficou: o app mostra o efeito no passo em que o
  receipt conclui.
- **duas fontes para o mesmo fato divergem:** os arquivos do commit
  vinham de `git show --name-only` no receipt e do diff de árvores no
  log; concordavam até o refresh e discordavam depois, numa renomeação.
  uma definição só, lida por uma função só.
- **o custo de uma observação inteira do ide é o log:** neste repositório
  a observação inteira tem uns 43 KB, 97% dela o `stacks` com 100 commits.
  pôr `files` em todos triplicaria. um agente que observa por escopo não
  paga isso; o limite (`files` nos 10 mais recentes) mantém o aumento em
  3,8%. quantos commits a projeção carrega é uma pergunta que ainda não
  foi feita.

## 2026-09-23, task-84 e auditoria de verdade

fonte: `steps/checked/task-84-ide-refresh-never-goes-back-done.md`,
`truth.md`.

- **pressuposto errado:** "a view é otimista e reconcilia com um refresh"
  era tratado como inofensivo. não era: um refresh lido antes de uma
  mutação pousava por cima dela, e a projeção mostrava por um instante um
  estado que não era nem o do git nem o que o ide aceitou. o agente
  roteirizado sem a espera que o contornava deu errado nas dez corridas
  com intervenção na base (nove recusas à toa ou decisões erradas, e um
  commit de dois arquivos em vez de três).
- **o que isso ensina sobre o contrato:** a checagem de stale protege
  contra o mundo que mudou, não contra o app que mostra um mundo errado.
  o contrato ficou coerente o tempo todo; quem mentiu foi a projeção.
  projeção verdadeira é uma invariante de cada `Operable`, que o `sem`
  não tem como garantir: cada app precisa de teste que prove que a
  projeção nunca fica mais velha que uma mutação aceita e converge para
  a verdade.
- **o checkpoint a teve sorte que não mediu:** os dois agentes LLM
  escaparam da corrida por tempo. H1 e H2 continuam de pé, mas a
  confiabilidade do resultado dependia de algo que o experimento não
  controlava. a nova rodada do experimento, depois das tasks 83 a 85, é
  necessária também por isso.
- **o cenário da tese ainda não rodou num app de verdade:** o binário
  `ide` não abre a porta do agente; o checkpoint a rodou no `ide-lab`,
  com o workbench sem janela e uma pessoa roteirizada. "uma pessoa usa o
  app enquanto um agente o opera" nunca aconteceu num app que alguém usa.
- **digitação fora do contrato:** no todo e no ide, digitar muda o texto
  projetado sem request nem receipt (o ADR de ligação isenta de
  propósito). consequência: nem o `acted` da task-83 nem o recorder sabem
  dizer quem digitou. fica como limite registrado até a task-64
  (inspector) precisar.
- **cobertura semântica fina:** dos 19 widgets com `project`, só
  `TextField` e `Toast` têm chamador de produção; só o todo e o ide
  projetam. para superfícies geradas (69) isso vai importar.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0036: a commit checks the index it observed](0036-commit-checks-the-index-it-observed.md)
- [task-64: session inspector, bug report and session for testing](../../../Tasks/task-64-session-inspector-and-tests/task-64-session-inspector-and-tests.md)
- [task-83: the delta carries the node that was just added, and the refusal names who acted](../../../Tasks/task-83-delta-carries-node-refusal-names-actor/task-83-delta-carries-node-refusal-names-actor.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../../../Tasks/task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
- [task-85: the commit that the receipt announces is already in the projection, with the files](../../../Tasks/task-85-the-commit-a-receipt-announces-is-projected/task-85-the-commit-a-receipt-announces-is-projected.md)
<!-- /related -->
