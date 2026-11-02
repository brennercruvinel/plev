---
id: learning-cycle-0045
title: "what the autonomous cycle taught: regression by meaning before pixels"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [determinism, semantics, rendering]
relations: [adr-0045, task-67, task-59]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: regression by meaning before pixels

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0045](0045-regression-by-meaning-before-pixels.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-23, task-67

- **o contrato serve para verificar, não só para operar:** a regressão
  do Todo diz `todo.item[2] semantics changed: label ...` com a região,
  em vez de "727 pixels diferentes". só o Todo e o ide projetam; nas
  outras páginas a regressão cai para layer e pixel. a cobertura
  semântica fina (task-59 e a auditoria) é o limite prático dela.
- **o hash é estável fora desta máquina:** o primeiro job `structural
  regression` no macOS do GitHub (PR #53) passou com as 157 baselines
  gravadas aqui; o medo dos bits de `sin`/`cos` do sistema não se
  confirmou nesse runner.
- **o hash da cena pega mudança que o pixel não vê:** o Button no tema
  hoff mudou de hash com 0 pixels diferentes quando um token mudou. o
  check falha por isso e diz 0 px; se isso é ruído ou sinal, o uso vai
  dizer.

## 2026-09-25, o regress espelha as ligações em mais de um lugar

- **ligar um widget muda o `scene.json` em três pontos:** o bloco
  `semantics`, o `bindings` e o nome da camada (o regress nomeia a camada
  pelas raízes dos ids ligados nela). a regra do briefing para baseline
  que passa a projetar: `frame.png` igual e, em `layers`, z, hash,
  contagem de nós e limites iguais; nome, `semantics` e `bindings` podem
  mudar. o orquestrador confere por script, não pelo relato.
- **"cole o diff inteiro" de arquivo novo despeja o arquivo:** o briefing
  pediu um diff por tipo de widget e o arquivo da task ganhou 3100 linhas
  de JSON. para baseline, pedir a lista, um resumo por tipo e o sha do
  commit; o diff mora no git.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0045: regression by meaning before pixels](0045-regression-by-meaning-before-pixels.md)
- [task-59: semantic binding with the render and one operational path](../../../Tasks/task-59-semantic-binding-and-one-path/task-59-semantic-binding-and-one-path.md)
- [task-67: structural regression before pixels](../../../Tasks/task-67-structural-regression/task-67-structural-regression.md)
<!-- /related -->
