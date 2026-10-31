---
id: task-62/phase-c-round-3
title: "task-62 phase c, third round: the task-89 rule (2026-09-24)"
project: plev
type: section
status: reference
parent: task-62
date: 2026-09-24
date_basis: text
topics: [agents, semantics]
relations: [operations-autonomy, task-87, task-88, task-89, task-90, task-91]
source: docs/log/steps/task-62-ide-human-agent-proof.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase c, third round (the task-89 rule, 2026-09-24)

Third round, already with the task-89 rule: a value never shown gives `not_observed`, and a terminal stage shows no state. With the person removing the file between the agent's decision and the `submit`, both runs had a valid stale measurement: one put the file back and committed, the other stopped without a commit, and the orchestrator read the stop as handoff behavior. The conclusion of checkpoint a held under the new rule; the run without a commit was recorded as an autonomous decision for review.

severidade P1 ([`autonomy.md`](../../ref/operations/autonomy.md)): a conclusão do checkpoint a
está em uso (a onda 5 abriu com ela), e o inspector mostrou que o acerto
do A2 da primeira rodada dependeu da ordem dos eventos. o submit tinha se
apoiado num `changes:staged` que o agente nunca viu. a task-89 fechou
isso, e a regra nova muda o que um agente encontra: um valor nunca
mostrado dá `not_observed` (exit 3), e um estágio terminal não mostra
estado. esta rodada confere se a conclusão se sustenta com a regra.
escrito antes de qualquer corrida.

o que mudou no app desde a segunda rodada: o estágio terminal traz a
revisão (task-87); um ato se apoia só no que foi mostrado ao ator
(task-89); o `acted` nomeia a cadeia desde o estado que o agente viu
(task-90). no laboratório: a medição de stale pelo valor visto e pelo
valor vivo e o cenário `unstage-before-submit`, com a pessoa entre a
decisão do agente e o submit (task-88).

o que não mudou, de propósito: o briefing do agente (o mesmo texto das
rodadas 1 e 2, só com os caminhos e o nome de cada corrida), sem ensinar
a regra nova. a mensagem de erro e o `plev act --help` a explicam, e
ensinar mediria obediência, não recuperação. também não mudaram o modelo
(`claude-fable-5-1`), o repositório do laboratório e a graça de 180 s.

corridas, em paralelo, cada uma com seu laboratório, binários de `main @
96c20a0`:
- A1'' (controle, `--human none`);
- A2''a e A2''b (`--human unstage-before-submit:src/session.rs`);
- A2''c (`--human unstage-after-observe:src/session.rs`, comparável às
  rodadas 1 e 2).

hipóteses:

- **S1:** H1, H2, H3 e H5 se sustentam nas quatro: tarefa só pelo
  contrato, receipt igual ao git e todo ato com `--from-revision`. nas
  corridas com a pessoa antes do submit, a medição é válida pelo valor, o
  HEAD não se move com o submit recusado e o agente se recupera.
- **S2:** nas corridas com intervenção válida, o `acted` da recusa nomeia
  a pessoa e o agente cita isso na recuperação.
- **S3, sem resultado favorito:** quantas recusas do protocolo
  (`not_observed`, `unknown_revision`) e quantas recusas stale pela
  própria escrita cada agente recebe, e se ele se recupera de cada uma
  sem ajuda.
- **S4, sem resultado favorito:** bytes enviados ao agente e mensagens
  por corrida, contra a segunda rodada (A1' 19.274 B e 24 mensagens do
  cliente; A2'a 25.602 B e 30). o estágio com revisão pode baixar; as
  recusas novas podem subir.

o que derruba (P1 que para o avanço, e condição de parada de
[`autonomy.md`](../../ref/operations/autonomy.md) se a hipótese central cair):
- uma corrida que falhe a tarefa ou commite arquivos errados;
- um agente que fique preso em `not_observed` sem se recuperar;
- uma corrida com a pessoa antes do submit cuja medição saia inválida.

o resultado vai como sair.

## resultado da terceira rodada (2026-09-24)

as quatro corridas rodaram em paralelo (`target/agents/exp3/<corrida>/`:
`report.json`, `session.monster`, `stdout.txt`, `audit.txt`; `runs.txt`
liga cada corrida à transcrição), com binários de `main @ 96c20a0`, o
modelo `claude-fable-5-1` e 180 s de graça.

| | A1'' | A2''a | A2''b | A2''c |
| --- | --- | --- | --- | --- |
| pessoa | nenhuma | antes do submit | antes do submit | depois de uma observação |
| resultado | MATCH | MATCH | sem commit | MATCH |
| recusa stale | 1 (a própria escrita) | 1 | 1 | 0 |
| `acted` na recusa | vazio | human, unstage, request 6 | human, unstage, request 6 | |
| medição de stale | sem pessoa | válida | válida | inválida: o agente viu a mudança antes do submit |
| recusas do protocolo (`not_observed`, `unknown_revision`) | 0 | 0 | 0 | 0 |
| atos, todos com `--from-revision` | 7 de 7 | 8 de 8 | 6 de 6 | 7 de 7 |
| comandos fora do `plev` | 0 | 0 | 0 | 0 |
| turnos do modelo | 12 | 14 | 12 | 13 |
| tokens de saída | 6.718 | 9.955 | 10.432 | 8.056 |
| bytes enviados ao agente | 25.588 | 26.927 | 20.729 | 34.863 |
| mensagens do cliente / do servidor | 24 / 43 | 34 / 56 | 26 / 42 | 38 / 59 |
| observações (snapshot / delta) | 5 / 0 | 5 / 4 | 3 / 4 | 10 / 2 |
| primeira mensagem ao commit | 59 s | 83 s | | 84 s |

a coluna "atos com `--from-revision`" vem da transcrição. o campo `form`
do relatório chama de `observed` todo ato encadeado na revisão de um
estágio (task-87), então não serve para essa conta. é um P1 para a
próxima medição, e entra na task-91.

as hipóteses:

- **S1: a segurança se sustentou nas quatro, e a tarefa só em três.**
  nenhuma corrida commitou arquivo errado. todo ato foi pelo contrato,
  com `--from-revision`. onde houve commit, o receipt bate com o git. nas
  duas corridas com a pessoa antes do submit, a medição foi válida pelo
  valor e o HEAD não se moveu com o submit recusado.
  - o A2''a recolocou `src/session.rs` no index e commitou os três.
  - o A2''b parou. ele leu a recusa e o aviso, que diziam que a pessoa
    tinha tirado o arquivo, e escreveu que recolocar seria "desfazer uma
    ação deliberada de um humano no aplicativo dele, sem ter como
    perguntar o motivo". a tarefa ficou sem commit. o critério escrito
    antes ("uma corrida que falhe a tarefa") disparou ao pé da letra; a
    leitura dele está abaixo, na decisão.
- **S2: sustentada.** nas duas corridas com a pessoa antes do submit, o
  `acted` nomeou a pessoa (request 6, `unstage` de `src/session.rs`), e os
  dois agentes citaram isso para decidir: um para recolocar, o outro para
  parar.
- **S3: nenhuma recusa do protocolo.** nenhum agente recebeu
  `not_observed` ou `unknown_revision`. houve uma recusa pela própria
  escrita, no controle: o A1'' escreveu a mensagem e submeteu encadeado,
  foi recusado stale com o `acted` vazio, observou e se recuperou. mas
  explicou a causa errado ("a revisão devolvida por um act concluído não
  contém o efeito daquele act"). a recusa não diz que a mudança foi dele.
- **S4: sem resultado favorito, e veio mais caro em bytes.** o A1''
  gastou 25.588 B contra 19.274 do A1' (+33%) com as mesmas 24 mensagens.
  ele fez cinco snapshots onde o A1' fez um. o A2''a gastou 26.927 B
  contra 25.602 (+5%), com 34 mensagens contra 30. o estágio com revisão
  foi usado: todos encadearam atos na revisão do anterior, e três
  mandaram os três `stage` sobre a mesma revisão, porque cada um lê só o
  próprio arquivo. mesmo assim os agentes continuam observando antes de
  decidir, e quatro corridas não separam isso da variação do modelo.

o que as corridas mostraram além das hipóteses, dito pelos agentes:
- não há canal para falar com a pessoa. o aviso só oferece `dismiss`, e
  quando a tarefa e a pessoa divergem, o agente só pode agir por cima ou
  parar (A2''b).
- `commit.submit` depende de `commit.form:open` para estar habilitado, e
  isso não aparece nos `reads` (A2''b). é a mesma lacuna da segunda
  rodada: não se sabe por que um controle está desabilitado.
- a revisão muda sem ator quando o git reclassifica um arquivo (de
  `untracked` para `added`) ou reordena a lista. o delta explica, mas
  parece que alguém mexeu (A2''a, A2''b).
- um delta vazio no `--since` não traz o estado, e o agente precisa de
  mais uma observação (A2''c).
- `enabled` só aparece quando é falso (A1''), e o `at` dos estágios não
  tem unidade (A2''a).

decisão do orquestrador (modo autônomo, regra de severidade): a
conclusão do checkpoint a se sustenta com a regra da task-89. o contrato
fez o que promete nas quatro corridas: nenhum commit errado, a recusa
antes do efeito, quem agiu e o que mudou.

a corrida sem commit não é falha da hipótese central. o agente recebeu
do contrato o que precisava para ver que a tarefa e a pessoa divergiam,
e escolheu não passar por cima dela. é o comportamento de handoff que a
tese quer, e mostra uma lacuna: o agente não tem como perguntar à pessoa
(P2, registrado no programa). como o critério escrito antes disparou ao
pé da letra, esta leitura fica no journal como decisão autônoma para o
brenner revisar. nenhuma condição de parada.

o critério P1 da onda 5 ("a conclusão do checkpoint a revista sob a
regra da task-89") está satisfeito.

severidade do que saiu daqui:
- P1 (entra na task-91): a forma do ato no relatório do laboratório;
- P2: canal para o agente perguntar à pessoa (handoff); explicar a
  recusa pela própria escrita (entra na task-91); por que um controle
  está desabilitado;
- P3: delta vazio sem estado, `enabled` implícito, unidade do `at`.
