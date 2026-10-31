---
id: task-62/phase-ab-closing
title: "task-62 phases a and b: close-out (2026-09-23)"
project: plev
type: section
status: reference
parent: task-62
date: 2026-09-23
date_basis: text
topics: [semantics, git]
relations: [adr-0036, task-58b, task-61, task-63, task-73]
source: docs/log/steps/task-62-ide-human-agent-proof.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [8978602]
---

# phases a and b closed (2026-09-23)

Closes phases a and b of task-62: the ide now exposes its actions through the semantic contract, the commit checks the expected staged set before writing, and the receipt reaches the sha and the files of the commit. Integrated at `8978602` (seven commits), rebased onto tasks 63 and 73 without conflict, with the green gate of 1809 tests and the six acceptance criteria passing on a real repository. The remaining limitations are the residual race with an external `git commit`, the missing assistive confirmation in the modal, and the worker's response being matched to the request by order.

integradas na `main` em `8978602` (sete commits), rebaseadas pelo
orquestrador sobre a 63 e a 73 sem conflito, gate verde (1809 testes). os
seis critérios de aceite passam em repositório real com o worker de
verdade, mais o clique real contra `Actor::Test` e o commit duplicado que
só commita uma vez; a suíte passou oito vezes seguidas. ADR:
[`0036-commit-checks-the-index-it-observed.md`](../../adr/semantics/0036-commit-checks-the-index-it-observed.md).

decisões aceitas: o nó de arquivo tem `unstaged` além de `staged` (uma
path com mudança nos dois é uma linha por lado mas um nó só), e
`stage`/`unstage`/`discard`/`ignore` leem os dois; `ignore` virou
capacidade; `changes:selected` é navegação, projetado e lido por ninguém;
`commit.submit` declarado à mão para não ter `activate` e `submit` para a
mesma coisa; o workspace não fica inerte com overlay aberto (agentes
seguem agindo, as checagens garantem); o keymap continua nomeando ações do
engine, mapeadas para capacidades num lugar só do Workbench.

limitações que seguem:

- a resposta do worker é casada com o request pela ordem (FIFO por tipo),
  correto porque há um worker só e ele responde em ordem; se um dia houver
  mais de um, o `RequestId` precisa ir e voltar no comando.
- fechar o modal sem confirmar deixa o receipt aguardando: falta recusar
  uma confirmação no `sem`. virou a task-58b.
- o modal de confirmação não é projetado, então não há confirmação por
  tecnologia assistiva ainda (entra com a 58b e a 60).
- resta a corrida entre a leitura do index pelo worker e o `git commit`
  por outro processo; o ADR descreve o caminho por write-tree/commit-tree
  que a fecharia.
- o `Workbench` não é `Send` (o dispatcher não é): a task-61 leva os
  requests do socket para a thread da UI.

falta a fase c: o experimento com agente externo pelo socket (61) e a
sessão gravada (63).
