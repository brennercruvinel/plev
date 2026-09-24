---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-24
domain: agent-surface
task: task-92
depends: [task-58b, task-89, task-91]
branch: task/92-ask-the-person
origin: terceira rodada do checkpoint a (task-62): o A2''b parou por não ter como perguntar à pessoa
severity: P2 (completude do handoff; nada que existe fica inválido)
---

# task-92: o agente pergunta à pessoa

## por que

na terceira rodada do checkpoint a, dois agentes LLM receberam a mesma
recusa: a pessoa tinha tirado um arquivo do index que a tarefa pedia.
um agente recolocou o arquivo por cima dela; o outro parou, dizendo que
recolocar seria "desfazer uma ação deliberada de um humano no aplicativo
dele, sem ter como perguntar o motivo", e que `notices` só oferece
`dismiss`. o contrato deu aos dois o que aconteceu e quem fez; não deu um
jeito de o agente consultar a pessoa antes de passar por cima.

o handoff humano-agente é um dos itens que a tese promete provar (goal
10, task-80). o mecanismo mais perto já existe: um request pode esperar a
confirmação de uma pessoa (`Stage::AwaitingConfirmation`, task-58 e 58b),
hoje só quando a `Policy` da ação exige (o `discard` do ide pede a um
agente).

## direção (orquestrador; a sonda confirma)

o agente pode pedir, num ato, que a pessoa confirme antes: o request
espera a confirmação de uma pessoa como o `discard` já espera, com um
texto curto do agente dizendo por quê ("você tirou src/session.rs; a
tarefa pede os três arquivos: posso recolocar?"). a pessoa confirma ou
recusa no modal que o ide já tem; o agente recebe o estágio como recebe
hoje. é uma extensão do que existe: nada de canal novo de conversa.

## aceite

um agente pelo `plev` pede para recolocar o arquivo com a pergunta; o
ide mostra o modal com o texto do agente; a pessoa (pelo dispatcher,
como `Actor::Human`) confirma e o stage roda, ou recusa e o agente recebe
o `declined`; tudo gravado e legível pelo inspector; formato das
mensagens só cresce; gate verde.
