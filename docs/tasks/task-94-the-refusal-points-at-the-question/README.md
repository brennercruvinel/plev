---
id: task-94
title: "task-94: the refusal that names a person points to the question"
project: plev
type: task
status: done
date: 2026-09-24
date_basis: header
topics: [agents, semantics]
relations: [task-62, task-92, task-93]
source: docs/log/steps/task-94-the-refusal-points-at-the-question.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [2a6ec95, ef4f5e9]
---

# task-94: the refusal that names a person points to the question

Makes `plev act` write a line to stderr that points to `--ask` when a stale refusal names a person and the act did not ask beforehand. In the fourth round of task-62, none of the six agents opened `plev act --help`; the two that received the sentence about `--ask` asked on the first refusal, and the person's answer decided. The line does not come out after a `declined` or when only agents acted, and stdout (the JSON) and protocol v1 do not change. The handoff left the gate green on the branch; the integration is recorded in the history at `37c5e0c`.

## por que

na quarta rodada da task-62, com a pessoa tirando um arquivo logo antes
do submit, os quatro agentes sem dica nunca perguntaram: dois pararam
dizendo que não havia canal, e dois passaram por cima dela. os dois que
receberam uma frase sobre o `--ask` perguntaram na primeira recusa, e a
resposta da pessoa decidiu. nenhum dos seis abriu o `plev act --help`. o
que o agente vê na recusa é o JSON do estágio, e nada ali aponta para a
pergunta.

## escopo

- o `plev act`, quando o estágio terminal é uma recusa stale cujo
  `acted` tem uma pessoa (`human` ou `assistive`) e o ato não tinha
  `--ask`, escreve no stderr uma linha que diz quem agiu e como perguntar
  antes de agir por cima (`--ask "<por quê>"`, o limite de 280);
- nenhuma linha depois de um `declined` (a pessoa já respondeu; não se
  sugere perguntar de novo), nem quando o `acted` só tem agentes, nem
  quando o ato já perguntava;
- o stdout (o JSON) e o protocolo v1 não mudam.

## aceite

testes do `plev` para os quatro casos (pessoa no `acted`, só agentes,
`declined`, ato que já perguntava); o stdout igual byte a byte; a linha
no stderr pela porta real, num teste do ide com a pessoa; gate verde.
depois, a quinta rodada da task-62 mede se a linha basta.

## handoff (2026-09-24)

goal: task-94, a recusa que nomeia uma pessoa aponta para a pergunta.
status: feito na branch `task/94-refusal-points-at-question`, sobre
`main @ 7f382fb`; gate verde. falta a integração do orquestrador.

### a sonda (na base, por teste)

`a_plev_submit_refused_after_the_person_unstaged_prints_the_stages`, em
`apps/idestudio/tests/agent_port.rs` (commit `2a6ec95`): o cenário da quarta
rodada pela porta real, com o shell do ide sem janela e o agente sendo o
próprio binário `plev`, rodado como um agente roda num shell (`--app
<socket> --name test-agent`, um processo por comando). o agente observa,
põe `src/session.rs` e `tests/login_test.rs` no stage, tira `README.md`,
abre o form e escreve a mensagem, cada ato sobre a revisão em que o
anterior terminou; observa para decidir; a pessoa tira `src/session.rs`
(`unstage`, `Actor::Human`, request 6); `plev act commit.submit submit
--from-revision <a decisão>`. saiu como a direção dizia: exit 1, o stdout
com os dois estágios, o `acted` com o `unstage` da pessoa, e o stderr
vazio. nada ali aponta para o `--ask`.

o que o agente recebeu na base (`at` e as revisões mudam a cada corrida:
o relógio do workbench e o diretório temporário do repositório):

```text
stdout:
{"v":1,"id":"2","type":"stage","request":"7","stage":"requested","at":586,"asked":{"actor":{"agent":{"name":"test-agent"}},"target":"commit.submit","action":"submit","observed":{"revision":"4ccea7202cdd5328","values":[["changes:staged",{"kind":"set","value":["src/login.rs","src/session.rs","tests/login_test.rs"]}],["commit.message:text",{"kind":"text","value":"fix login session timeout"}]]}}}
{"v":1,"id":"2","type":"stage","request":"7","stage":{"rejected":{"stale":{"changes":[{"state_changed":{"at":"changes:staged","before":{"kind":"set","value":["src/login.rs","src/session.rs","tests/login_test.rs"]},"after":{"kind":"set","value":["src/login.rs","tests/login_test.rs"]},"added":[],"removed":["src/session.rs"]}}],"acted":[{"id":"6","actor":"human","target":"changes.file[src/session.rs]","action":"unstage"}]}}},"at":586,"revision":"cf2ab6ed86dece2d"}
stderr:
(vazio)
```

depois da task (o mesmo teste, renomeado para
`a_plev_submit_refused_after_the_person_unstaged_points_at_the_question`):

```text
stdout:
{"v":1,"id":"2","type":"stage","request":"7","stage":"requested","at":209,"asked":{"actor":{"agent":{"name":"test-agent"}},"target":"commit.submit","action":"submit","observed":{"revision":"f5f8f4ea2cfddc15","values":[["changes:staged",{"kind":"set","value":["src/login.rs","src/session.rs","tests/login_test.rs"]}],["commit.message:text",{"kind":"text","value":"fix login session timeout"}]]}}}
{"v":1,"id":"2","type":"stage","request":"7","stage":{"rejected":{"stale":{"changes":[{"state_changed":{"at":"changes:staged","before":{"kind":"set","value":["src/login.rs","src/session.rs","tests/login_test.rs"]},"after":{"kind":"set","value":["src/login.rs","tests/login_test.rs"]},"added":[],"removed":["src/session.rs"]}}],"acted":[{"id":"6","actor":"human","target":"changes.file[src/session.rs]","action":"unstage"}]}}},"at":209,"revision":"bb29a3273a8d28b0"}
stderr:
plev: a person acted since this agent looked (request 6, human: unstage changes.file[src/session.rs]); to act over that, ask the person first: add --ask "<why>" to the act (one line, at most 280 characters)
```

o stdout é o mesmo: as asserções sobre ele não mudaram entre o commit da
sonda e o da linha, e as duas versões conferem que ele é exatamente a
recodificação canônica (`encode_server`) das mensagens, uma por linha,
sem nada a mais.

### a linha

o texto, com uma pessoa:

```text
plev: a person acted since this agent looked (request 6, human: unstage changes.file[src/session.rs]); to act over that, ask the person first: add --ask "<why>" to the act (one line, at most 280 characters)
```

com mais de uma pessoa, cada request dela entra nos parênteses, em ordem
de id, separada por `; ` (`request 3, human: toggle todo.item[2];
request 4, assistive: toggle todo.item[3]`). o limite vem de
`wire::MAX_ASK`.

quando aparece: só depois de um estágio terminal `rejected` com `stale`
cujo `acted` tem pelo menos uma pessoa (`human` ou `assistive`), e só se
o ato não tinha `--ask`. vai no stderr, uma vez, depois de o estágio sair
no stdout (o stdout vai linha a linha, então numa saída combinada a
linha vem depois do JSON). não aparece:
- depois de um `declined` (a pessoa já respondeu);
- quando o `acted` só tem agentes, ou está vazio (uma mudança que nenhum
  request fez não nomeia ninguém);
- quando o ato já perguntava, inclusive o stale da segunda checagem
  depois de um `confirmed`, e o stale de um ato com pergunta recusado
  antes de chegar à pessoa;
- em `completed`, `failed`, num estágio não terminal e num erro do
  protocolo (`not_observed` e os outros).

a função é `ask_hint(stage: &Stage, asked: bool) -> Option<String>`,
pura, em `rust/socket/src/bin/plev.rs`; o `act_once` guarda `asked`
antes de mandar o ato e escreve a linha depois de imprimir o estágio
terminal. o `plev act --help`, na seção do ask, ganhou: "when an act
without --ask is refused as stale and its acted names a person (human or
assistive), plev writes one line on stderr after the stage that names
the person's request and points at --ask; none after a decline."

### decisões

- o texto não diz que a pessoa mexeu no que o ato lê. o `acted` diz quem
  rodou algo desde que o agente olhou, não o que moveu o que ele lê (quem
  diz isso é o `changes`); "acted on what this act read", do exemplo do
  briefing, prometeria mais do que o contrato. ficou "a person acted
  since this agent looked", a mesma frase do `--help` sobre o `acted`.
- "to act over that, ask the person first: add --ask ... to the act": o
  ato que pergunta não precisa ser o recusado (na quarta rodada, quem
  perguntou perguntou num `stage`, não no `submit`). a frase fala com o
  agente; a pessoa é sempre "the person".
- a linha nomeia só as requests das pessoas, não as de agentes que
  estejam no mesmo `acted`; o JSON continua com todas.
- o limite de 280 entra na linha porque custou uma tentativa ao A4c2, que
  não leu o `--help`.
- `ask_hint` recebe o `Stage`, não o `StageReply`: é o que decide, e os
  testes montam cada caso direto.

### testes

- `plev.rs`, `a_stale_refusal_naming_a_person_points_at_the_question`: a
  pessoa (`human`, com um agente antes no mesmo `acted`) sem pergunta dá
  a linha inteira, conferida byte a byte; `assistive` dá a linha; nada com
  pergunta, com só agentes, depois de `declined`, em `completed` e em
  `failed`. e `act_takes_a_question_for_a_person_and_waits_longer_for_it`
  confere a frase nova do `--help`.
- `rust/socket/tests/cli.rs`, com o binário contra o app de brinquedo:
  `a_stale_refusal_naming_a_person_points_at_the_question_on_stderr`
  (outro agente agiu: recusa sem linha; uma pessoa na tela e outra por
  tecnologia assistiva: a linha com as duas, e o stdout exatamente a
  recodificação canônica; o mesmo ato com `--ask`: recusado stale antes
  da pergunta, sem linha). o teste do `--ask` recusado pela pessoa agora
  confere que o stderr fica vazio.
- `apps/idestudio/tests/agent_port.rs`, pela porta real, sem janela: a sonda
  com a linha no stderr e o stdout igual; em seguida o agente faz o que a
  linha diz (observa e pede o `stage` de `src/session.rs` com `--ask`),
  a pessoa recusa no modal, sai `declined` com exit 1 e o stderr vazio,
  nada é commitado e `src/session.rs` fica fora do index.
  `a_question_the_person_allows_runs_and_prints_no_line`: a mesma rodada,
  a pessoa permite, o stage roda até `completed` e o stderr fica vazio.
  os testes do porto do ide rodaram três vezes seguidas, verdes.

### validação e contagem

`./tool/tasks/fullcheck.sh` verde no fim, com o `CARGO_TARGET_DIR` da task: teste,
clippy `-D warnings`, fmt, wasm check e a perna urnaui. `cargo test
--workspace`: na base, 2347 passando e 12 ignorados (no commit da sonda,
que só acrescenta um teste, 2348 e 12); no fim, 2351 e 12 (+4: a função,
o binário contra o brinquedo, e dois pela porta real do ide). a perna
urnaui: 82 passando e 1 ignorado. `cargo test -p socket -p idestudio` verde.

### limitações

- `rust/socket/src/bin/plev.rs` está com 638 das 639 linhas que o guard
  conta. o guard deixa de fora o módulo `#[cfg(test)]` só na coluna 0, e o
  do `plev.rs` está dentro de `mod cli`, indentado: os testes contam como
  produção, ao contrário do que a convenção diz. a próxima mudança no
  `plev` não cabe. a recomendação: um commit só de movimento que leva o
  módulo de testes para o nível do arquivo (`#[cfg(test)] mod tests {`,
  com `#![cfg(unix)]` dentro e `use super::cli::*`), o que libera umas 240
  linhas. não fiz porque é reorganização fora do pedido.
- o teste do ide acha o `plev` ao lado do próprio diretório
  (`target/<profile>/plev`), porque o cargo só dá `CARGO_BIN_EXE_*` aos
  testes do mesmo pacote. `cargo test -p socket -p idestudio` e o workspace
  constroem o binário; `cargo test -p idestudio` sozinho não, e o teste recusa
  um `plev` ausente ou mais velho que `plev.rs`, dizendo como construir.
  a checagem compara a data do binário com a de `plev.rs`, não com a das
  dependências dele.
- a linha é do `plev`, não do protocolo: um cliente que fala o protocolo
  direto não a recebe, e o relatório do laboratório não a vê (quem vê é a
  transcrição).
- uma pessoa cujo request passou da retenção do dispatcher não é nomeada
  no `acted`, e aí não há linha.
- a doc do topo de `agent_port.rs` ainda diz que um decline só chega ao
  agente no drain seguinte, o que deixou de valer com o P0 da task-92.
  não mexi.

### o que a quinta rodada precisa saber

- o binário: a quarta rodada usou uma cópia (`target/agents/exp4/bin/plev`,
  de `7d9c569`). a quinta precisa de um `plev` construído da `main` depois
  desta task; uma cópia velha não escreve a linha. `plev act --help`
  mostra a frase nova, um jeito rápido de conferir.
- a linha vai no stderr. na quarta rodada os agentes rodaram `plev ...;
  echo "exit=$?"`, sem redirecionar nada, e a ferramenta de shell deles
  mostra o stderr (o A4c2 leu a recusa do `--ask` de 340 caracteres, que
  é stderr). um agente que fizer `2>/dev/null` ou `| jq` perde a linha:
  vale contar isso na transcrição junto com a citação da linha (U2).
- no cenário `unstage-before-submit` a linha sai na primeira recusa do
  submit, nomeando a request 6 da pessoa, como na quarta rodada. depois
  de um `declined` não sai linha nenhuma; um agente que pergunte de novo
  depois de uma recusa faz isso por conta própria.
- a linha manda perguntar antes de agir por cima; ela não diz que a
  pessoa pode recusar. isso está no `--help`, que nenhum agente leu na
  quarta rodada.

### commits

- `2a6ec95` task-94: the probe, a plev submit refused after the person
  unstaged, as a test
- `ef4f5e9` task-94: a stale refusal naming a person points at --ask on
  stderr
- o commit deste handoff, com a frase do `--help`
