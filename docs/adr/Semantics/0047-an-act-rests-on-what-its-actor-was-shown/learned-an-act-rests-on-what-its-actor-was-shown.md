---
id: learning-cycle-0047
title: "what the autonomous cycle taught: an act rests on what its actor was shown"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [agents, semantics]
relations: [adr-0047, task-91, task-89, task-87, task-62, task-88]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: an act rests on what its actor was shown

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0047](0047-an-act-rests-on-what-its-actor-was-shown.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-91

- **a âncora certa de "quem agiu" é o instante do olhar, não uma
  revisão.** revisão é hash de conteúdo e muda sem receipt (uma seleção,
  um refresh do git). o dispatcher numera os requests na ordem, e o port
  marca cada observação nessa ordem. com isso a cena comum do ide nomeia
  quem tirou o arquivo, o que duas regras por revisão (83 e 90) não
  conseguiam. e o agente recusado pela própria escrita agora lê que foi
  ele.

## 2026-09-24, checkpoint a, terceira rodada (a regra da task-89)

- **a segurança se sustentou com a regra nova:** quatro corridas com
  agentes LLM, nenhum commit errado, nenhuma recusa do protocolo, todo ato
  com `--from-revision`. com a pessoa agindo entre a decisão e o submit, a
  recusa veio antes do efeito e nomeou a pessoa nas duas corridas.
- **um agente escolheu não passar por cima da pessoa:** com a recusa e o
  aviso dizendo que a pessoa tirou o arquivo, um agente recolocou o
  arquivo e commitou; o outro parou e explicou por quê. o contrato deu a
  informação para os dois; o que falta é o agente poder perguntar à
  pessoa. é o primeiro sinal concreto de handoff, e a lacuna é de
  capacidade, não de segurança.
- **a recusa pela própria escrita confunde:** o agente que encadeou
  depois de escrever foi recusado, se recuperou, mas explicou a causa
  errado. a regra está certa; a recusa não diz que a mudança foi dele.
- **os agentes não economizaram:** mesmo com o estágio trazendo a
  revisão, eles observam antes de decidir, e o controle gastou mais bytes
  que na segunda rodada. o ganho da task-87 aparece no roteirizado, não
  num agente LLM que prefere olhar.

## 2026-09-23, task-88 (sonda)

- **com a regra da task-89, escrever e encadear exige olhar:** um agente
  que escreve a mensagem (`set_value`) e submete encadeado na revisão do
  estágio é recusado pela própria escrita. um estágio terminal não mostra
  estado, e a mensagem em que ele se apoia vem da última observação, que
  estava vazia. o `acted` sai vazio, porque ninguém mais agiu. é
  conservador e está escrito no `plev act --help`. o próximo experimento
  com LLM precisa medir quanto isso custa em idas e voltas.
- **revisão é hash de conteúdo:** um estado que volta exatamente ao que
  era tem a mesma revisão. a medição de stale do laboratório, que achava
  o ato pela revisão, deu "válida" numa rodada em que a pessoa nunca
  exerceu a checagem. medir pelo valor que o request carrega e pelo valor
  vivo no dispatch, nunca pela revisão.

## 2026-09-23, task-89

- **o furo fechou e ficou reproduzível:** no laboratório, olhar
  `changes` (três arquivos), a pessoa tirar um, olhar só o formulário e
  submeter agora é recusado stale, com o arquivo que saiu. antes, o
  commit saía com dois arquivos e o agente achava que eram três.
- **a identidade do agente é o nome, não a conexão:** "o que foi mostrado
  àquela conexão" recusaria todo agente pelo cli, que abre uma conexão
  por comando. o nome do `hello` já era a identidade dos receipts e do
  `acted`, e virou também a do que foi mostrado. o custo é que dois
  processos com o mesmo nome contam como um agente só.
- **`keys` também corta o que foi mostrado:** um agente que observa com
  `--keys staged` não viu o `unstaged` que o `stage` lê, e o ato sai
  `not_observed`. as `reads` de cada ação estão na observação, então o
  agente sabe o que precisa ver.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0047: an act rests on what its actor was shown](0047-an-act-rests-on-what-its-actor-was-shown.md)
- [task-62: proof in the ide: stale-state rejection and human-agent handoff](../../../Tasks/task-62-ide-human-agent-proof/task-62-ide-human-agent-proof.md)
- [task-87: a terminal stage carries the revision the app was left in](../../../Tasks/task-87-a-terminal-stage-carries-the-revision/task-87-a-terminal-stage-carries-the-revision.md)
- [task-88: the lab measures what the agent rested on](../../../Tasks/task-88-the-lab-measures-what-the-agent-rested-on/task-88-the-lab-measures-what-the-agent-rested-on.md)
- [task-89: an act relies only on what was shown to the agent](../../../Tasks/task-89-an-act-rests-only-on-what-was-shown/task-89-an-act-rests-only-on-what-was-shown.md)
- [task-91: the dispatcher knows when the agent looked](../../../Tasks/task-91-the-dispatcher-knows-when-the-agent-looked/task-91-the-dispatcher-knows-when-the-agent-looked.md)
<!-- /related -->
