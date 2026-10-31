---
id: task-62/fase-ab-fechamento
title: "task-62 fase a e b: fechamento (2026-09-23)"
project: plev
type: secao
parent: task-62
status: referencia
date: 2026-09-23
date_basis: texto
topics: [semantica, git]
relations: [adr-0036, task-58b, task-61, task-63, task-73]
evidence: [8978602]
source: docs/log/steps/task-62-ide-human-agent-proof.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# fases a e b fechadas (2026-09-23)

Fecha as fases a e b da task-62: o ide passa a expor suas ações pelo contrato semântico, o commit confere o conjunto staged esperado antes de gravar, e o receipt chega ao sha e aos arquivos do commit. Integrada em `8978602` (sete commits), rebaseada sobre as tasks 63 e 73 sem conflito, com o gate verde de 1809 testes e os seis critérios de aceitação passando em repositório real. Ficam como limitações a corrida residual com um `git commit` externo, a falta de confirmação assistiva no modal e a resposta do worker casada com o request pela ordem.

integradas na `main` em `8978602` (sete commits), rebaseadas pelo
orquestrador sobre a 63 e a 73 sem conflito, gate verde (1809 testes). os
seis critérios de aceite passam em repositório real com o worker de
verdade, mais o clique real contra `Actor::Test` e o commit duplicado que
só commita uma vez; a suíte passou oito vezes seguidas. ADR:
[`0036-commit-checks-the-index-it-observed.md`](../../decisoes/semantica/0036-commit-checks-the-index-it-observed.md).

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
