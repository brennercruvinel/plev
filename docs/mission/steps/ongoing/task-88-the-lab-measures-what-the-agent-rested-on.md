---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: experiment
task: task-88
depends: [task-87, task-89]
branch: task/88-lab-rests-on
origin: fechamento da task-87 e a regra da task-89
---

# task-88: o laboratório mede no que o agente se apoiou

## por que

o laboratório da task-62 (`ide-lab`) mede a checagem de stale com uma
pessoa roteirizada. duas premissas dele envelheceram:

1. a medição é válida quando um `submit` depois da intervenção se apoiou
   numa revisão servida por observação antes dela (`story.rs`,
   `stale_measurement`). desde a task-87 um agente encadeia na revisão de
   um estágio terminal, que não é observação. desde a task-89 o valor de
   cada `read` vem da última observação que o mostrou ao agente, não da
   revisão do ato. a revisão do ato deixou de dizer em que o agente se
   apoiou; o valor que o request carrega diz.
2. a pessoa age logo depois de uma observação servida
   (`LabHost::observed`, `intervene`). um agente que olha uma vez e
   encadeia o resto nos estágios nunca dispara a pessoa, e a rodada sai
   sem intervenção.

e a task-89 criou uma recusa nova antes do dispatch (`not_observed`), que
o relatório não conta.

sem isso, o próximo experimento com LLM (conferir que agentes se
recuperam de `not_observed` e da regra da 89) mede errado.

## escopo

- a medição de stale pelo significado: válida quando um `submit` depois
  da intervenção carrega em `observed` o `changes:staged` de antes da
  intervenção (o mundo velho), qualquer que seja a revisão;
- a pessoa age também logo depois de um estágio terminal entregue ao
  agente, quando a tarefa está pronta;
- o relatório conta `not_observed` e `unknown_revision` como recusas do
  protocolo, com as reads;
- `lab.rs` dividido por responsabilidade abaixo do limite (sai da lista
  congelada do guard).

## aceite

testes do laboratório para os dois jeitos de agente (observa antes do
submit; encadeia nos estágios sem observar) com e sem a pessoa; medição
válida nos dois com a pessoa; o roteirizado com MATCH; gate verde.

## sonda (2026-09-23, antes de implementar): parada

o briefing pediu, antes de mudar a medição e a pessoa, duas corridas: o
modo que encadeia (item 4) na base, esperando que a pessoa não agisse e
que a medição saísse inválida; e o roteirizado padrão com a pessoa,
para ver se a medição de hoje ainda sai válida depois da task-89. a
segunda saiu como esperado. a primeira não, e a parada vale.

o modo que encadeia entrou como sonda, no commit `1693296`:
`ide-lab --arm scripted --scripted chain`. o primeiro `submit` se apoia
na revisão que o último estágio do `set_value` carregou, sem observação
antes; depois de uma recusa o agente observa e decide de novo, como o
padrão. o padrão (`--scripted observe`) não mudou. nada mais foi
implementado.

### as duas saídas pedidas

roteirizado padrão com a pessoa, na base
(`--arm scripted --human unstage-after-observe:src/session.rs`): exit 0,
MATCH, 17 mensagens in e 39 out, a medição de hoje válida.

```text
stale_measurement_valid  true
stale measurement        valid: the submit at 388 ms rested on revision 5c826c8bd93f6e5f
                         served before the intervention and ended rejected
act commit.message set_value  completed at 359 ms, revision e4684e9bfe63f702 (observed, ...)
act commit.submit submit      rejected at 388 ms, revision 5c826c8bd93f6e5f (from_revision, ...)
act changes.file[src/session.rs] stage  completed at 411 ms, revision 901d464ac711ec5c (from_revision, ...)
act commit.submit submit      completed at 443 ms, revision 89fe8e95532af599 (from_revision, ...)
intervention  t 385 ms, revision served just before 5c826c8bd93f6e5f, person's request 6
stale rejection  changes:staged removed ["src/session.rs"]; acted: human unstage changes.file[src/session.rs]
```

o modo que encadeia com a pessoa, na base
(`--scripted chain --human unstage-after-observe:src/session.rs`):
exit 0, MATCH, 16 in e 38 out. a pessoa agiu, e a medição de hoje saiu
válida.

```text
stale_measurement_valid  true
stale measurement        valid: the submit at 373 ms rested on revision 51a0df27537d768a
                         served before the intervention and ended completed
act commit.message set_value  completed at 287 ms, revision db3702a5dfaa3031 (observed, ...)
act commit.submit submit      rejected at 289 ms, revision a817c8b219f51414 (observed, ...)
act changes.file[src/session.rs] stage  completed at 337 ms, revision d6834792312146ad (from_revision, ...)
act commit.submit submit      completed at 373 ms, revision 51a0df27537d768a (from_revision, 2 observation(s) between, ...)
intervention  t 332 ms, revision served just before 51a0df27537d768a, person's request 7
stale rejection  request 6: commit.message:text before "" after "fix login session timeout"; acted: (nobody)
observations (index, revision)  0 c285a4e8, 1 d3fae36b, 2 51a0df27, 3 d6834792, 4 51a0df27, 5 00d95d5b, 6 00d95d5b
```

o mesmo em mais quatro corridas do modo que encadeia com a pessoa: as
cinco iguais nisso tudo. sem a pessoa (`--scripted chain --human none`):
exit 0, MATCH, e a mesma recusa no primeiro `submit`, pela mensagem, com
`acted` vazio.

### o que aconteceu, e por que é outro

1. o `submit` encadeado é recusado pelo próprio `set_value` do agente,
   com ou sem a pessoa. é a regra da task-89 fazendo o que diz: a
   mensagem em que o `submit` se apoia é a da última observação que a
   mostrou, vazia, e o agente escreveu depois. `acted` vazio, porque o
   único ator foi ele. um agente que olha uma vez e encadeia o resto não
   consegue commitar sem observar de novo.
2. então a pessoa age, só que tarde: na primeira observação da
   recuperação (o `since`, índice 2), depois do primeiro `submit` já
   recusado. o agente observa o mundo novo, repõe o arquivo e commita. a
   checagem de stale nunca foi exercida pela pessoa nessa corrida.
3. a medição de hoje diz válida nesse `submit` que concluiu. as revisões
   são hash do conteúdo: o mundo que o agente restaurou (os três no
   index, a mensagem, o aviso da recusa) projeta igual ao que foi
   servido quando a pessoa agiu e tem a mesma revisão (`51a0df27`,
   índices 2 e 4), e o `served_index` acha a primeira. é um falso
   válido.
4. a regra pelo valor como o briefing escreve (o `submit` depois da
   intervenção carrega o `staged_before`) cai no mesmo falso válido: o
   `submit` da recuperação carrega de novo os três arquivos. ela precisa
   do valor vivo: válida quando o `submit` carrega o `staged_before` e o
   app tinha outro `changes:staged` quando ele chegou. com isso, nessa
   corrida as duas medições se aplicam e discordam (a de hoje válida, a
   pelo valor inválida), o que também é condição de parada.

### o item 2 como está escrito, medido

para não decidir pela leitura, rodei o item 2 como descartável (fora de
commit, revertido): a pessoa age também depois de um estágio terminal
entregue ao agente, quando a tarefa está pronta.

- no modo padrão: exit 1. a tarefa fica pronta no estágio `completed` do
  `set_value`, a pessoa age ali, a observação da decisão já mostra
  `{src/login.rs, tests/login_test.rs}`, o agente submete isso e o
  commit sai com dois arquivos, sem recusa. o teste do gate
  `the_person_makes_the_agents_submit_stale_and_the_agent_recovers`
  cairia.
- no modo que encadeia: exit 0, MATCH, recusa por `changes:staged` e
  pela mensagem, `acted` nomeando a pessoa, recuperação.

nos dois modos a tarefa fica pronta no mesmo instante (o `set_value`), e
a pessoa não tem como saber se o próximo passo do agente é observar ou
submeter.

### uma alternativa, medida

também descartável: a pessoa age logo antes de o `submit` do agente
chegar ao dispatcher, quando a tarefa está pronta (a mesma condição de
`Person::ready`), dentro de `LabHost::dispatch`, como `Actor::Human`,
pelo mesmo dispatcher, sem mudar o `wire`, e sem o gatilho na
observação. três corridas de cada modo:

| modo | exit | recusa | `acted` | recuperou | mensagens in / out | bytes out |
| --- | --- | --- | --- | --- | --- | --- |
| padrão | 0, 0, 0 | `changes:staged` | human unstage | sim | 17 / 39 | 45.878 a 45.879 |
| encadeia | 0, 0, 0 | `changes:staged` e a mensagem | human unstage | sim | 16 / 38 | 39.648 a 39.656 |

o padrão fica com as mesmas mensagens de hoje (17 in, 39 out; na base,
45.871 e 45.882 bytes out em duas corridas). e some a assimetria que a
task-62 escreveu entre os braços: o hook do braço `git` já dispara no
instante do commit.

### decisão pedida

- problema: com a regra da task-89, o agente que encadeia sem observar
  é recusado pelo próprio ato antes de a pessoa importar; a pessoa da
  base age na recuperação dele e a medição de hoje dá falso válido por
  colisão de revisão; o item 2 como está quebra o modo padrão com a
  pessoa; e a regra pelo valor precisa do valor vivo.
- opções:
  1. o item 2 como está. impacto: o padrão com a pessoa sai exit 1; para
     salvar, o agente padrão teria que conferir o index antes do
     `submit`, e aí nunca é recusado e o A0 deixa de provar a recusa.
  2. a pessoa logo antes do `submit` chegar ao dispatcher (medida
     acima), num cenário novo (`unstage-before-submit:<path>`), e
     `unstage-after-observe` fica como está, comparável às rodadas 1 e
     2. impacto: os dois modos exercem a checagem; os campos
     `intervention.served_index` e `revision_served` perdem o sentido no
     cenário novo e ficam como informação (a revisão em que o `submit`
     se apoiou); o texto da task-62 sobre o protocolo ganha a nota.
  3. a pessoa como hoje, só a medição pelo valor e o relatório. impacto:
     mínimo, mas um agente que encadeia nunca tem a checagem exercida
     pela pessoa, e a medição diz isso como inválida.
- recomendação: a opção 2, com a medição pelo valor e pelo valor vivo
  (visto igual ao `staged_before`, vivo diferente), e o texto dizendo os
  dois. o próximo experimento com LLM roda o cenário novo. os itens 3
  (as recusas do protocolo no relatório) e 5 (dividir o `lab.rs`) não
  dependem da decisão.
- decisão necessária: qual gatilho a pessoa usa (1, 2 ou 3); se a
  medição pelo valor inclui o valor vivo; na opção 2, o nome do cenário
  e se o antigo fica.
