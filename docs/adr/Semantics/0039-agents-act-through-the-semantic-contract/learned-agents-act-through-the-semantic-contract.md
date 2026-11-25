---
id: learning-cycle-0039
title: "what the autonomous cycle taught: agents act through the semantic contract"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [agents, semantics, formats]
relations: [adr-0039, task-94, task-62, task-86, task-83, task-90, task-84]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: agents act through the semantic contract

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0039](0039-agents-act-through-the-semantic-contract.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, checkpoint a, quinta rodada

- **o contrato ensina uma capacidade quando a mostra na hora em que ela
  serve:** com o mesmo briefing da quarta rodada, e o `plev` apontando
  para o `--ask` só na recusa que nomeia uma pessoa (task-94), os dois
  agentes perguntaram, e a resposta da pessoa decidiu (commit com
  `allow`, parada com `decline`). a documentação (`--help`) não tinha
  bastado: nenhum agente a abriu. vale para o resto do contrato: o que
  um agente precisa decidir deve vir junto do estado ou da recusa que
  pede a decisão. fonte: [`task-62-ide-human-agent-proof.md`](../../../Tasks/task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md), "resultado
  da quinta rodada".

## 2026-09-24, checkpoint a, quarta rodada (o handoff)

- **o agente usa o que o contrato mostra na hora da decisão:** com a
  pessoa tirando um arquivo antes do submit, nenhum dos quatro agentes
  sem dica perguntou. dois pararam dizendo que não havia canal, dois
  passaram por cima dela. nenhum abriu o `plev act --help`, onde o
  `--ask` estava. com uma frase no briefing, os dois perguntaram na
  primeira recusa, e a resposta da pessoa decidiu (commit com `allow`,
  parada com `decline`). a segurança não dependeu disso: nas seis, a
  recusa veio antes do efeito e nomeou a pessoa. o que muda no programa:
  a task-94 põe a pergunta na recusa que nomeia uma pessoa, e uma quinta
  rodada curta mede se basta. fonte: [`task-62-ide-human-agent-proof.md`](../../../Tasks/task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md),
  "resultado da quarta rodada".

## 2026-09-23, task-86: o cenário da tese numa janela de verdade

- **a tese saiu do laboratório:** no `ide` com janela, um agente pelo
  `plev` e uma pessoa com cliques reais operaram o mesmo workbench; a
  recusa stale nomeou a pessoa e o texto do toast é o nó que o agente
  lê. antes isso só existia no `ide-lab`, sem janela e com a pessoa
  roteirizada.
- **o limite escrito do `acted` cai no uso normal:** a regra da task-83
  (nomear quem agiu na revisão exata que o autor viu) foi aceita como
  limite documentado. na janela, a ordem natural de uma pessoa (clicar
  na linha, depois tirar do index) faz a recusa nomear a seleção e
  esconder o unstage. limite aceito por ser raro virou limite comum
  quando uma pessoa de verdade entrou: task-90.

## 2026-09-23, checkpoint a (task-62 fase c)

fonte: `steps/checked/task-62-ide-human-agent-proof-done.md`, "resultado da fase
c"; saídas em `target/agents/exp/`.

- **confirmado (H1, H2, H3, H5):** um agente LLM sem contexto do repositório
  opera o ide só pelo contrato (`plev observe`/`act`), sem screenshot e sem
  `git`; a ação sobre estado velho é recusada antes do efeito, com o delta,
  e o HEAD não se move; o agente relê, repara e conclui; o receipt gravado
  na sessão bate com o git real.
- **onde o plev ganha (H4):** a recusa vem antes do efeito e nomeia o que
  mudou. no braço `git` com a mesma intervenção, o commit errado existiu, o
  git não avisou (exit 0), e o conserto foi `commit --amend`, que só
  funciona em commit local.
- **onde não ganha (H4):** volume e idas e voltas. 22 KB em 30 mensagens
  (plev) contra 1,5 KB em 20 invocações (`git`). tokens comparáveis. uma
  conexão por comando do `plev` pesa nas idas e voltas.
- **o que mudou no programa:** as três lacunas de contrato que os agentes
  apontaram viraram a task-83 (delta com o nó, recusa com autor, regra de
  stale escrita) e a corrida do `Refresh` virou a task-84, antes da onda 5
  (decisão do brenner: "correções primeiro"). a onda 5 espera o
  experimento rodar de novo com as correções.
- **a medir, não a supor:** se a conexão persistente (um processo `plev`
  por sessão de agente) muda o placar de idas e voltas. é ferramenta, não
  contrato; entra só se o re-run mostrar que o custo importa.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0039: agents act through the semantic contract](0039-agents-act-through-the-semantic-contract.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../../../Tasks/task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-83: the delta carries the node that was just added, and the refusal names who acted](../../../Tasks/task-83-delta-carries-node-refusal-names-actor/task-83-delta-carries-node-refusal-names-actor.md)
- [task-84: an old refresh does not undo what the ide has already accepted](../../../Tasks/task-84-ide-refresh-never-goes-back/task-84-ide-refresh-never-goes-back.md)
- [task-86: the ide people use opens the agent port](../../../Tasks/task-86-the-ide-opens-the-agent-port/task-86-the-ide-opens-the-agent-port.md)
- [task-90: a refusal names who acted since the agent looked](../../../Tasks/task-90-a-refusal-names-who-acted-since-the-look/task-90-a-refusal-names-who-acted-since-the-look.md)
- [task-94: the refusal that names a person points to the question](../../../Tasks/task-94-the-refusal-points-at-the-question/task-94-the-refusal-points-at-the-question.md)
<!-- /related -->
