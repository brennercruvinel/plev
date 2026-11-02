---
id: learning-cycle-0085
title: "what the autonomous cycle taught: integration into main is a pull request merged by merge commit"
project: plev
type: learning
status: reference
date: 2026-09
date_basis: text
recorded: 2026-09-23
topics: [git, process, agents]
relations: [adr-0085, task-93, task-91]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: integration into main is a pull request merged by merge commit

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0085](0085-integration-by-merge-commit.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, um milissegundo que derruba o gate

- **o gate caiu num teste que não era da task:** na integração da fase 4c,
  `the_agent_asks_the_person_who_declines_and_it_stops` (task-93) caiu
  uma vez em 20 corridas sob carga e nenhuma sozinha. a causa: o
  laboratório lia o relógio para gravar o clique da pessoa e o dispatcher
  lia de novo dentro do clique para carimbar o estágio; num limite de
  milissegundo os dois divergem por 1, e o teste exige o mesmo instante.
  correção na fonte, não no teste: o clique é gravado no instante do
  estágio que ele causou (o mesmo `answered_at` do relatório). regra: um
  fato gravado em dois lugares vem de uma leitura só do relógio.
- **o orquestrador publicou antes de ler o gate:** o comando que
  commitava e abria o PR estava encadeado depois do `tail` do gate, sem
  condição. o PR #76 ficou aberto com o gate vermelho até a correção.
  regra: a publicação só entra no mesmo comando quando está condicionada
  ao `exit 0` do gate (`&&` sobre o resultado, não sobre o `tail`).

## 2026-09-24

- **o CI parou por cobrança, e o merge espera:** às 06:22 UTC todo job
  do PR #68 e o da `main` depois do #67 falharam em segundos, sem
  runner. a anotação do github diz que os jobs não começaram porque um
  pagamento da conta falhou ou o limite de gasto precisa subir. é
  serviço pago (condição de parada da autonomia): o orquestrador não
  mergeia sem o CI, continua integrando na `main` local com o gate, e a
  publicação espera o brenner (ele decidiu: mergear com o gate local
  verde enquanto o CI estiver fora, journal). uma falha de todos os jobs em segundos
  se lê pelas anotações antes de se ler como código quebrado.

## 2026-09-23
- **a guarda de merge nunca guardou:** o orquestrador decidia o merge
  com `if ! grep -qv pass`, e o `grep` desta shell é uma função de
  snapshot que devolveu 1 com uma linha `fail` no arquivo. todo PR até o
  #58 estava verde, então nada entrou vermelho por isso. o PR #59 entrou
  vermelho por outro caminho (merge feito na conta do github às 00:24, com
  o teste do macOS já falhando), e a guarda também não teria segurado.
  regra: a decisão de merge lê as colunas com `awk -F'\t' '$2!="pass"'` e
  só mergeia com zero linhas; e o teste que falhou era de tempo (task-91),
  corrigido em seguida (PR #60).

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0085: integration into main is a pull request merged by merge commit](0085-integration-by-merge-commit.md)
- [task-91: the dispatcher knows when the agent looked](../../../Tasks/task-91-the-dispatcher-knows-when-the-agent-looked/task-91-the-dispatcher-knows-when-the-agent-looked.md)
- [task-93: the person in the lab answers](../../../Tasks/task-93-the-lab-person-answers/task-93-the-lab-person-answers.md)
<!-- /related -->
