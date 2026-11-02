---
id: learning-cycle-0043
title: "what the autonomous cycle taught: a delta carries the node it adds, and a stale refusal names who acted"
project: plev
type: learning
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-09-23
topics: [semantics, agents, recording]
relations: [adr-0043, task-90, task-91, task-62, task-87, task-83]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: a delta carries the node it adds, and a stale refusal names who acted

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0043](0043-a-delta-carries-the-node-a-refusal-names-who-acted.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-23, task-90 (sonda)

- **estado projetado que muda fora do dispatcher:** no ide, a seleção
  por clique ou teclado muda `changes:selected` direto na view, e a
  revisão muda sem receipt; o mesmo vale para a digitação na mensagem, o
  foco, o toast saindo e as linhas do diff. o `check` não sofre (as ações
  leem o que muda com elas), mas atribuição e inspeção sim: a recusa não
  tem onde ancorar "desde que o agente olhou", e o inspector vê a mudança
  sem ator. é a mesma classe da dívida "digitação fora do contrato".
  a correção de atribuição é a task-91 (o dispatcher sabe do olhar).

## 2026-09-23, checkpoint a, segunda rodada

fonte: `steps/checked/task-62-ide-human-agent-proof-done.md`, "resultado da
segunda rodada"; saídas em `target/agents/exp2/`.

- **confirmado de novo, com o app corrigido:** três agentes LLM
  (`claude-fable-5-1`, o mesmo briefing da primeira rodada) fizeram a
  tarefa só pelo contrato; as duas intervenções foram recusadas antes do
  efeito e recuperadas; receipt igual ao git nas três.
- **as correções aparecem no comportamento:** os três confirmaram o
  próprio commit numa observação por delta que já trazia o nó com
  `files`; os dois com intervenção citaram o `acted` (a pessoa, request 6)
  ao explicar a recuperação. tokens de saída caíram 24 a 26%.
- **bytes, não:** com intervenção, 14 a 16% mais bytes que antes, com as
  mesmas 30 mensagens. o que sobra de custo de idas e voltas é uma
  observação depois de cada ato só para saber a revisão nova; os três
  agentes pediram a revisão no `completed` (task-87).
- **texto projetado não pode supor quem lê:** o aviso dizia "you unstaged
  src/session.rs" para a pessoa na tela, e o agente leu o mesmo nó achando
  que o "you" era ele. um contrato único para humano e agente quer frases
  que nomeiam o ator; corrigido ("the person").

## 2026-09-23, task-83

- **confirmado no laboratório:** a regra exata do `acted` (ações de
  outros atores checadas na mesma revisão que o autor viu e que rodaram)
  nomeia no cenário da intervenção exatamente a pessoa que tirou o
  arquivo do index, e ninguém mais. a atribuição útil saiu barata porque
  o dispatcher já guardava `validated_against` em cada receipt.
- **bytes, pela primeira vez a favor:** o commit nascendo em `stacks`
  custava ao agente 172 B de delta mais 402 B de uma segunda observação;
  com o nó no delta, 427 B numa ida. o delta de um item novo no todo
  subiu de 423 para 647 B, ainda 28% da observação inteira com 3 itens.
- **limite que continua:** uma mudança de fora do app e uma digitação
  (sem request) nunca aparecem em `acted`; a recusa diz o que mudou e
  fica calada sobre quem.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0043: a delta carries the node it adds, and a stale refusal names who acted](0043-a-delta-carries-the-node-a-refusal-names-who-acted.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../../../Tasks/task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-83: the delta carries the node that was just added, and the refusal names who acted](../../../Tasks/task-83-delta-carries-node-refusal-names-actor/task-83-delta-carries-node-refusal-names-actor.md)
- [task-87: a terminal stage carries the revision the app was left in](../../../Tasks/task-87-a-terminal-stage-carries-the-revision/task-87-a-terminal-stage-carries-the-revision.md)
- [task-90: a refusal names who acted since the agent looked](../../../Tasks/task-90-a-refusal-names-who-acted-since-the-look/task-90-a-refusal-names-who-acted-since-the-look.md)
- [task-91: the dispatcher knows when the agent looked](../../../Tasks/task-91-the-dispatcher-knows-when-the-agent-looked/task-91-the-dispatcher-knows-when-the-agent-looked.md)
<!-- /related -->
