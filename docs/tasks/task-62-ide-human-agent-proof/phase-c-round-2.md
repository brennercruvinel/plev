---
id: task-62/phase-c-round-2
title: "task-62 phase c, second round (2026-09-23)"
project: plev
type: section
status: reference
parent: task-62
date: 2026-09-23
date_basis: text
topics: [agents, semantics]
relations: [operations-autonomy, task-83, task-84, task-85]
source: docs/log/steps/task-62-ide-human-agent-proof.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase c, second round (after tasks 83, 84 and 85, 2026-09-23)

Second round of phase c, with the ide corrected by tasks 83, 84 and 85 and the same briefing, to see whether the corrections show up in the agents' behavior. The three runs each confirmed their own commit with a delta observation that already carried `stacks.commit[sha]` with `files`, and the two runs with an intervention refused the stale submit, naming the person. In the runs with intervention, the byte cost rose by 14 to 16%, because of the whole node in the delta, the `files` and the warning node.

o brenner decidiu no checkpoint a por "correções primeiro", com a onda 5
esperando outro checkpoint. este é ele: o mesmo experimento, com o app
corrigido. escrito antes de qualquer corrida.

o que mudou no app desde a primeira rodada: a view do ide nunca volta
para um status do git lido antes de uma mutação já mandada (task-84); um
`node_added` no delta traz o nó inteiro, a recusa stale traz `acted`
(quem, entre os atores do app, rodou algo sobre o estado que o agente
viu) e a regra de stale está no `plev act --help` (task-83); o commit
aparece em `stacks` no passo em que o receipt conclui, com `files` nos
dez commits mais recentes (task-85).

o que não mudou, de propósito: o briefing do agente (o mesmo texto,
`briefing-arm-a.md` da primeira rodada), o modelo (`claude-fable-5-1`,
o das corridas A1 e A2, conferido nas transcrições), o repositório do
laboratório, a intervenção roteirizada. a graça passa a 180 s em todas
as corridas (na primeira rodada, o A2 teve 1 s e perdeu a verificação
final).

corridas: A1' (controle, sem intervenção), A2'a e A2'b (intervenção
`unstage-after-observe:src/session.rs`). três corridas não fazem
estatística: o objetivo é ver se algo piorou e se as correções aparecem
no comportamento do agente.

hipóteses:

- **R1:** H1, H2, H3 e H5 continuam de pé nas três: tarefa feita só pelo
  contrato, recusa stale válida e recuperação nas corridas com
  intervenção, receipt igual ao git, todo ato com `--from-revision`.
- **R2:** o agente confirma o próprio commit com menos observações que
  na primeira rodada, porque o delta traz o nó e o nó traz `files`. na
  primeira rodada o A1 confirmou por `stacks.commit[sha]` e `show_diff`,
  e o A2 não confirmou (graça de 1 s).
- **R3:** nas corridas com intervenção, o agente cita quem agiu (o
  `acted` da recusa) quando explica a recuperação.
- **R4, sem resultado favorito:** bytes enviados ao agente e idas e
  voltas por corrida, contra A1 (21.639 B, 28 mensagens do cliente) e A2
  (22.040 B, 30). podem subir (nó inteiro no delta, `files`) ou descer
  (menos observações).

o que derruba: uma corrida que falhe a tarefa, confirme errado, ou
precise de mais idas e voltas para confirmar do que antes. o resultado
vai como sair.

## resultado da segunda rodada (2026-09-23)

as três corridas rodaram em paralelo, cada uma com seu laboratório
(`target/agents/exp2/<corrida>/`: `report.json`, `session.monster`,
`stdout.txt`, `audit.txt`), binários de `main @ 476bbb4`, modelo
`claude-fable-5-1` nas três (conferido na transcrição), graça de 180 s.

| | A2 (1ª) | A2'a | A2'b | A1 (1ª) | A1' |
| --- | --- | --- | --- | --- | --- |
| resultado | MATCH | MATCH | MATCH | MATCH | MATCH |
| recusa stale | 1 | 1 | 1 | 0 | 0 |
| `acted` na recusa | (não existia) | human, unstage, request 6 | human, unstage, request 6 | | |
| confirmou o commit pela interface | não (graça de 1 s) | sim, num delta com o nó e `files` | sim, num delta com o nó e `files` | sim, `stacks.commit[sha]` e `show_diff` | sim, num delta com o nó e `files` |
| turnos do modelo | 15 | 14 | 14 | 13 | 12 |
| tokens de saída | 10.510 | 7.983 | 7.828 | 7.747 | 5.739 |
| contexto relido (cache lido) | 748.597 | 696.897 | 693.981 | 629.534 | 548.534 |
| comandos de shell | 18 | 15 | 13 | 12 | 12 |
| comandos fora do `plev` | 0 | 0 | 0 | 0 | 0 |
| bytes enviados ao agente | 22.040 | 25.602 | 25.196 | 21.639 | 19.274 |
| mensagens do cliente / do servidor | 30 / 52 | 30 / 52 | 30 / 52 | 28 / 49 | 24 / 42 |
| observações (snapshot / delta) | 2 / 5 | 2 / 5 | 3 / 4 | 4 / 3 | 1 / 5 |
| atos | 8 | 8 | 8 | 7 | 6 |
| primeira mensagem ao commit | 109 s | 115 s | 110 s | 66 s | 77 s |
| sessão gravada | 181.586 B, 67 eventos | 188.804 B, 67 | 181.715 B, 66 | 162.817 B, 61 | 142.158 B, 49 |

(o auditor de transcrição marcou 12 comandos do A2'b como "fora" porque o
agente guardou o caminho do `plev` numa variável e rodou `$P observe`;
conferidos à mão, os 13 são `plev` e um `echo` do exit code. o script
agora resolve a variável e as três corridas dão 0.)

as hipóteses:

- **R1, sustentada.** as três fizeram a tarefa só pelo contrato, com
  todo ato em `--from-revision`; nas duas com intervenção, a recusa foi
  válida (`stale_measurement_valid`), o HEAD não se moveu e o agente se
  recuperou; o receipt bate com o git nas três.
- **R2, sustentada.** as três confirmaram o próprio commit com uma
  observação por delta que já trazia `stacks.commit[<sha>]` com `files`;
  nenhuma pediu uma observação por escopo ou `show_diff` para isso.
- **R3, sustentada.** os dois agentes com intervenção citaram, no
  relatório, que o `acted` da recusa apontava a pessoa (request 6,
  `unstage` de `src/session.rs`) e se recuperaram por isso, sem repetir o
  submit às cegas.
- **R4, sem resultado favorito, e ele veio misto.** o controle ficou mais
  barato: 11% menos bytes, 4 mensagens a menos, 26% menos tokens de
  saída. as corridas com intervenção ficaram 14 a 16% mais caras em
  bytes (o nó inteiro no delta, `files`, o nó do aviso), com as mesmas 30
  mensagens, e 24 a 26% menos tokens de saída. o tempo até o commit não
  mudou de forma que três corridas possam dizer.

o que as corridas mostraram além das hipóteses, dito pelos três agentes:

- **o `completed` não traz a revisão nova.** para encadear um ato no
  outro, o agente precisa de uma observação só para saber a revisão; os
  três pediram isso, e é o maior custo de idas e voltas que sobrou (a
  derrota do plev em H4).
- **o aviso da recusa fala com a pessoa na segunda pessoa** ("you
  unstaged src/session.rs"), e o agente leu o mesmo nó achando que o
  "you" era ele: o texto projetado supõe quem lê. defeito.
- **`--keys` corta só mudanças de estado**; `actions_changed` e
  `reordered` passam (A1'). esperado pelo código, surpresa para o agente.
- **não se sabe por que um controle está desabilitado** (`commit.submit`
  só diz `enabled: false`), nem o que `constrained` restringe.

decisão do orquestrador (modo autônomo, [`autonomy.md`](../../ref/operations/autonomy.md)): a
hipótese central continua de pé com o app corrigido, e as correções
aparecem no comportamento dos agentes. nenhuma condição de parada. a
onda 5 abre. antes, duas correções pequenas do que esta rodada achou: o
aviso sem segunda pessoa e a revisão no `completed`.
