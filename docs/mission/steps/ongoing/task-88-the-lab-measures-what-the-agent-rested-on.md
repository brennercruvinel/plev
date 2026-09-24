---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-24
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

## decisão do orquestrador (2026-09-23)

a opção 2. um cenário novo, `unstage-before-submit:<path>`, com a
pessoa dentro de `LabHost::dispatch`, e `unstage-after-observe` como
está e padrão. a medição pelo valor visto e pelo valor vivo, e a
medição por revisão fora do veredito. o item 2 do briefing (a pessoa
depois de um estágio terminal) não entra. os itens 3 e 5 como no
briefing. o `--scripted chain` fica, com a nota no help.

## handoff (agente, 2026-09-24)

feito sobre a main em `0c1f629` (rebase antes da implementação, porque a
task-90 mudou a regra de `acted` no `sem`). nada no `wire`, no `sem` ou
no `rec` mudou.

### a medição como ficou

`agent.stale_measurement_valid` é verdadeira quando um `submit` que
chegou ao dispatcher depois do request da pessoa (id maior) carrega em
`observed` o `changes:staged` de antes dela (`staged_before`) e o app
tinha outro `changes:staged` naquele dispatch, qualquer que seja a
revisão que o `submit` nomeou.

- o valor visto sai do próprio request. o valor vivo é uma projeção que
  o `LabHost::dispatch` faz logo antes de cada `submit`. ela só vale
  quando a revisão dela é a do `validated_against` do receipt, a
  projeção que o dispatcher conferiu; senão fica desconhecida e a
  medição diz isso. o tempo dela fica fora do tempo dentro do `drain` e
  fora de `project_calls` (`cost.contract.lab_look_calls`,
  `lab_look_per_call_us`, de 130 a 445 us em debug).
- o texto diz os dois valores e o motivo. válida: "submit 7 reached the
  dispatcher after the person's request 6 carrying changes:staged
  {...}, the value before it, while the app held {...}". inválida, um
  de: ninguém interveio; nenhum `submit` depois da pessoa; o primeiro
  depois dela carregou o mundo novo (o agente viu a mudança); carregou o
  mundo de antes mas o app tinha o mesmo de novo (o agente repôs); o
  valor vivo é desconhecido. quando a pessoa agiu depois do primeiro
  `submit`, o texto soma "too late for it".
- `agent.stale_measured` traz `valid`, `request`, `seen` e `live`.
  cada ato em `agent.acts` ganha `request`, `seen_staged` e
  `live_staged`.
- a medição por revisão saiu do veredito. `form`,
  `revision_served_index`, `revision_served_at`,
  `observations_between` e `fresh_before` ficam como informação, com uma
  nota em `notes` sobre por que não decidem: a revisão de um estágio
  terminal não é de observação, e a revisão é hash do que o app projeta.

### onde a pessoa age

- `unstage-after-observe:<path>` (padrão, como antes): dentro de
  `Host::observed`, logo depois de servir uma observação, quando a
  tarefa está pronta.
- `unstage-before-submit:<path>` (novo): dentro de `LabHost::dispatch`,
  quando chega um `submit` de agente e a tarefa está pronta, antes de o
  `submit` ir para o dispatcher. uma vez, pelo dispatcher, como
  `Actor::Human`, gravada como input no controle do arquivo.
- `intervention.trigger` diz qual (`observation served` ou
  `submit arriving`). no cenário novo, `revision_served` é a revisão que
  o `submit` nomeou e `served_index` é o da última observação servida,
  como informação.

### o que o relatório conta

- as recusas stale, como antes (`stale`, com `changes`, `acted` e o
  HEAD). `acted` vazio num `submit` é o agente recusado pela própria
  escrita; `acted` com a pessoa é a checagem exercida.
- `agent.protocol_refusals`: `before_dispatch` (os atos que a porta
  tomou e não entregou ao app: `traffic.acts` menos os dispatches do
  host) em todo braço. `not_observed`, `unknown_revision`, `other` e
  `each` (alvo, ação, erro, reads, texto) só no roteirizado, que vê a
  recusa na conexão. no braço `plev` esses ficam `null` e `seen_by` diz
  "the port's counts": o tipo de cada recusa está na transcrição do
  agente.
- o roteirizado guarda a recusa do protocolo e para (a tarefa do
  laboratório nunca a produz). o teste unitário tira as duas de um
  `wire::Server` real e conta.

### as rodadas

`ide-lab --arm scripted`, build debug, cinco corridas por combinação com
pessoa, e uma sem pessoa por modo como referência. as 22 saíram exit 0.

| agente | pessoa | corridas | exit | verdict | medição | msgs in / out | bytes out | stale nomeando a pessoa | stale pela própria escrita | not_observed | unknown_revision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| observe | after-observe | 5 | 0 | MATCH | válida 5/5 | 17 / 39 | 45.875 a 45.878 | 1 | 0 | 0 | 0 |
| observe | before-submit | 5 | 0 | MATCH | válida 5/5 | 17 / 39 | 45.875 a 45.878 | 1 | 0 | 0 | 0 |
| chain | after-observe | 5 | 0 | MATCH | inválida 5/5, a pessoa chega tarde | 16 / 38 | 37.803 a 37.806 | 0 | 1 | 0 | 0 |
| chain | before-submit | 5 | 0 | MATCH | válida 5/5 | 16 / 38 | 39.646 a 39.649 | 1, com a escrita | 0 | 0 | 0 |
| observe | nenhuma | 1 | 0 | MATCH | inválida, sem intervenção | 12 / 30 | 28.787 | 0 | 0 | 0 | 0 |
| chain | nenhuma | 1 | 0 | MATCH | inválida, sem intervenção | 15 / 34 | 37.171 | 0 | 1 | 0 | 0 |

- o padrão tem os números da task-87 e da task-89: 12 in e 28,8 KB sem
  pessoa, 17 in e 45,9 KB com ela. o cenário novo deixa o padrão igual.
- `chain` com `after-observe` é o resultado esperado, não falha: o
  primeiro `submit` é recusado pela própria escrita da mensagem; a
  pessoa age na observação da recuperação; o agente repõe o arquivo e
  commita. a medição diz "the world was put back before the submit ...
  the person acted after the agent's first submit (6), too late for
  it".
- `chain` com `before-submit` dá uma recusa só, que nomeia
  `changes:staged` (a pessoa) e `commit.message:text` (a escrita do
  agente), e o `acted` nomeia a pessoa. o `submit` recusado se apoiou
  numa revisão de estágio: `revision_served_index` nulo, e a medição
  pelo valor válida.

### testes

- `crates/ide/tests/lab.rs`: os dois que existiam, sem mudança
  (`the_person_makes_the_agents_submit_stale_and_the_agent_recovers`,
  `without_the_person_the_agent_commits_the_task_with_no_refusal`), e
  quatro novos, um por combinação que faltava:
  `the_person_before_the_submit_makes_the_observed_submit_stale`,
  `the_chained_submit_is_refused_by_the_agents_own_write`,
  `after_the_look_the_person_comes_too_late_for_the_chained_submit`,
  `before_the_submit_the_person_makes_the_chained_submit_stale`. todos
  exigem MATCH, o commit confirmado pelo nó e nenhuma recusa do
  protocolo. o arquivo inteiro passou cinco vezes seguidas.
- unitários no bin: quatro da medição em `story.rs` (o mundo velho
  válido, o mundo reposto, o mundo novo, e sem pessoa, sem `submit`
  depois ou sem valor vivo), um em `refused.rs` com `not_observed` e
  `unknown_revision` vindos de um `wire::Server` real, e o parse do
  cenário novo e do `--scripted` nos testes de `args.rs`.

### validação

- `df -h /`: 15 GB livres antes do gate.
- `cargo test -p ide` e `cargo test -p wire` verdes (`wire`: 76).
- `./script/gate` verde depois do rebase na main (`0c1f629`), com a perna
  urnaui. depois: 2234 passed, 12 ignored (workspace 2152 e 11, urnaui 82
  e 1). a main em `0c1f629`: 2225, contada como o total de depois menos
  os 9 `#[test]` que a branch adiciona (`git diff 0c1f629`, nenhum
  removido, nenhum `#[ignore]` novo). os 2218 do briefing eram a main em
  `4163983`, antes da task-90.
- `crates/engine/tests/file_hygiene_guard.rs` verde sem o `lab.rs` em
  `FROZEN`. o maior arquivo do laboratório agora é o `finish.rs`, com 463
  linhas.

### decisões

- o valor vivo vem de uma projeção do próprio laboratório logo antes do
  dispatch de cada `submit`, e não da ordem interna do `wire` (que
  projeta antes do dispatch para os segredos). é uma projeção a mais por
  `submit`, duas por corrida, fora do custo do contrato.
- "depois da intervenção" é pela ordem do dispatcher, pelo id do
  request, não pelo relógio.
- o `lab.rs` virou `lab.rs` (a corrida: laço, drain, gravação, receipts,
  336 linhas) e `finish.rs` (o fim: a sessão reaberta, git, o relatório
  em json e em tabela), com `Lab::finish` no mesmo nome.
- as recusas do protocolo ficaram num módulo próprio (`refused.rs`), com
  o nome do erro lido da forma serde do `ProtoError` (a tag `error`),
  para um erro novo do protocolo aparecer sem mudar o laboratório.
- o `--scripted chain` se recupera como o padrão: observa e decide de
  novo depois da recusa.

### limitações e riscos

- no braço `plev` o laboratório só conta as recusas do protocolo, sem o
  tipo. com a gravação de ligações (task-63d) isso poderia vir da
  sessão; hoje vem da transcrição.
- `before_dispatch` conta como entregue um ato que o `wire` recusou
  depois do dispatch ("the app's dispatcher reports no stage"). isso não
  acontece com o `Workbench`.
- o roteirizado não se recupera de `not_observed` nem de
  `unknown_revision`: ele guarda a recusa e para. a tarefa do
  laboratório não produz nenhuma das duas.
- o formato do json só cresceu: `agent.stale_measured`,
  `agent.protocol_refusals`, `agent.acts[].request`, `seen_staged`,
  `live_staged`, `intervention.trigger`,
  `cost.contract.lab_look_calls` e `lab_look_per_call_us`. nenhum campo
  saiu. o que mudou foi o sentido de `agent.stale_measurement_valid` e
  o texto de `agent.stale_measurement`, e a nota `form: ...` em `notes`,
  que virou duas notas.
- o `README.md` não mudou: a linha do `ide-lab` não cita flags.

### protocolo para o próximo experimento com LLM

- cenário: nas corridas com intervenção,
  `--human unstage-before-submit:src/session.rs`; no controle,
  `--human none`. ele exerce a checagem se o agente observar antes do
  `submit` e também se encadear, e é o mesmo instante do hook do braço
  `git`. `unstage-after-observe` só para comparar com as rodadas 1 e 2:
  num agente que encadeia depois de escrever, a pessoa chega tarde.
- medição: `agent.stale_measurement_valid`, lida com
  `agent.stale_measured` (`seen`, `live`, `request`) e o texto. os
  campos por revisão não decidem nada.
- recusas: em `stale`, `acted` com a pessoa é a checagem exercida, e
  `acted` vazio num `submit` é o agente recusado pela própria escrita.
  `agent.protocol_refusals.before_dispatch` conta os atos recusados
  antes do dispatcher. o tipo vem da transcrição: `plev act` sai 3 com
  "not observed: this agent was never shown <reads>; observe it and
  decide again" ou com "revision <r> is not held; observe again". o
  auditor de transcrição conta esses exits 3 por mensagem.
- o briefing dos agentes LLM:
  - o mesmo texto das rodadas 1 e 2, para comparar;
  - um `--name` próprio, o mesmo em todo comando: o que um agente viu é
    guardado pelo nome (task-89), e dois agentes sob o nome padrão
    `plev` se apoiam no que o outro viu;
  - o exit 3 do `plev act` é recusa do protocolo antes do app
    (`not_observed`, `unknown_revision`), e o remédio está na própria
    mensagem: observar e decidir de novo;
  - sobre encadear depois de escrever, recomendo não ensinar. o
    `plev act --help` já diz a regra, e a recusa se explica: `changes`
    mostra a mensagem indo de "" para o texto do próprio agente, com
    `acted` vazio. ensinar mede obediência, e calar mede o que o
    experimento quer, se o agente se recupera. se quiser as duas coisas,
    uma corrida a mais com a frase "um estágio terminal não mostra
    estado: depois de escrever, observe antes de agir sobre o que
    escreveu".

### commits

- `9a788f0` a sonda: o roteirizado que encadeia o primeiro `submit`
- `c094390` a sonda e a parada, no arquivo da task
- `743824f` o `lab.rs` dividido (corrida e fim), fora do `FROZEN`
- `13203d8` a medição pelo valor visto e pelo valor vivo
- `1a55c48` a pessoa logo antes de o `submit` chegar ao dispatcher
- `5742826` as recusas do protocolo no relatório
- `ce6bc15` os testes das combinações e a nota na task-62
- este: o handoff
