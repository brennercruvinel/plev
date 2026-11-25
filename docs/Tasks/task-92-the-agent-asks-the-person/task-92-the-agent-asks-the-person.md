---
id: task-92
title: "task-92: the agent asks the person"
project: plev
type: task
status: done
date: 2026-09-24
date_basis: header
recorded: 2026-09-24
topics: [agents, semantics, recording]
relations: [task-58, task-58b, task-80, task-89, task-91, adr-0033, adr-0039]
evidence: [f101359, 2f61463, 7623b7f, e3cf244, 01b9b2a, ae544b7, d56ef44, 64011de, 9792058, 82202ab, d293046]
source: docs/log/steps/task-92-the-agent-asks-the-person.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-92: the agent asks the person

Lets an agent ask, in the request itself, that a person confirm the action before it runs, and lets the person's refusal reach the agent. In the third round of checkpoint a, one agent re-staged a file that the person had removed from the index, and the other stopped with no way to ask; the probe showed that the refusal did not reach the agent, and the fix in the port (P0) came first. Integrated into `main` at `f101359`, the question is recorded, appears in the inspector's `why`, and does not enter the projection.

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

## handoff (2026-09-24): parada na sonda

(superada: o orquestrador classificou o achado como P0, escolheu a opção
1 e liberou `port.rs`, `redact.rs` e `why.rs`; o handoff final está no fim
deste arquivo. esta seção fica como o registro da parada.)

goal: task-92, o agente pergunta à pessoa.
status: parada na sonda. a condição "a sonda sair diferente" disparou; só
a sonda foi feita, como teste, e nada do pedido foi implementado.

### a sonda

dois testes em `apps/idestudio/tests/agent_port.rs`, na base (`88923f2`), com
o porto real, o git worker real e um shell que serve o porto como a
janela do ide serve (`apps/idestudio/src/main.rs`, `serve_agent`): depois do
wake do porto e depois de cada resposta do worker.

- `an_agent_discard_through_the_port_waits_for_the_person_who_confirms_it`:
  o agente pede `discard` de `README` pela porta; recebe `requested` e
  `awaiting_confirmation` (`Confirm::ForAgents`); o ide abre o modal; a
  pessoa clica em Discard, que confirma pelo dispatcher como
  `Actor::Human` (`UiRequest::Confirm`, `Workbench::confirmed_by_person`,
  `Dispatcher::confirm` com a segunda checagem); o discard roda, e
  `confirmed`, `executing` e `completed` chegam ao agente no drain depois
  da resposta do worker. confere com a direção.
- `a_decline_reaches_the_agent_only_at_the_next_drain`: a pessoa clica em
  Cancel, que recusa pelo dispatcher como `Actor::Human`; nada roda, e o
  toast diz "Declined: discard of new.txt by test-agent did not run". o
  sink do porto ouve o `rejected` na hora, mas o estágio fica parado no
  porto: nada serve o porto depois da entrada de uma pessoa, e uma recusa
  não manda nada ao worker. em 500 ms o agente não recebe nada; o
  `declined` só sai no drain da próxima mensagem dele (o teste manda um
  `observe` e recebe as duas coisas). aqui a sonda sai diferente da
  direção ("recusa e o agente recebe o estágio").

o que o modal mostra hoje, e onde o texto nasce: título "Discard
changes?", corpo "test-agent asks to discard all changes to README. This
cannot be undone.", botão "Discard" (e "Cancel"). o texto nasce em
`Workbench::confirmation` (`apps/idestudio/src/capability/notice.rs`), a partir
do ator, da ação e do alvo do request; `ask_next_confirmation`
(`person.rs`) o passa a `WorkspaceView::open_confirm_modal`
(`views/workspace/overlays.rs`), que abre o `comps::feedback::Modal` e o
overlay `OverlayKind::Modal`. o modal não entra na projeção: nenhum nó o
representa, e um agente só lê `notices` (os toasts).

### problema

um `declined` não chega a um agente que só espera por ele. o porto manda
estágios só quando o shell o drena (`AgentPort::drain`, `Server::forward`),
e a janela do ide drena só no wake do porto e depois de um resultado do
git. o `plev act` manda o act e fica esperando o estágio terminal: com a
recusa, espera até o timeout (30 s por padrão) e sai com 3. quando a
conexão fecha, `Server::close` esquece os requests dela, e o estágio que
sair depois se perde.

com o pedido da task, o buraco cresce: o agente poderá perguntar antes de
qualquer ação, e toda confirmação que termina sem o worker fica igual à
recusa. `open`, `cancel`, `set_value`, `show_diff` e `dismiss` terminam na
hora (`Execution::Done`), e a segunda checagem que recusa como stale na
confirmação também. no cenário do aceite (A2''b, a pergunta num `stage`),
confirmar funciona, porque o `stage` vai ao worker; recusar é exatamente
o caso quebrado ("o agente recebe o `declined` e para").

nada disso se resolve nos arquivos permitidos: o drain é de
`rust/socket/src/port.rs` e de quem o chama (`apps/idestudio/src/main.rs`,
`apps/idestudio/src/capability/agent.rs`, o laboratório).

### opções

1. o porto acorda o próprio loop quando o sink ouve, fora de um drain, um
   estágio de um request que ele emitiu. arquivos: `rust/socket/src/port.rs`
   (fora da lista) e `server.rs` (na lista; está com 610 das 639 linhas).
   impacto: todo host (a janela do ide, o `ide-lab`, o showcase) passa a
   entregar o estágio sem mudar o shell; nenhuma mudança de protocolo; um
   wake a mais por estágio ouvido fora de um drain, e os wakes já
   coalescem. o risco é pequeno: o sink roda na thread do app e o wake é
   o mesmo closure que o leitor já chama.
2. a janela do ide serve o porto depois de toda entrada de uma pessoa,
   como já serve depois de um resultado do git. arquivo:
   `apps/idestudio/src/main.rs` (proibido). impacto: uma linha, mas vale só
   para a janela; cada host precisa lembrar disso (o laboratório também,
   quando tiver uma pessoa que responde), e o teste precisa imitar o shell.
3. aceitar a espera e documentar: o agente precisa mandar outra mensagem
   para receber a resposta da pessoa. impacto: o `plev act --ask` que
   espera trava até o timeout, o aceite da recusa não fecha com o `plev`,
   e o próximo experimento com LLM mediria o timeout, não o handoff.

### recomendação

a opção 1. a causa está no porto: só ele sabe que tem requests emitidos
esperando, e ele entrega estágios só quando alguém o drena. corrigir ali
serve todo host de uma vez e deixa o shell como está. é aditivo e não muda
o protocolo.

### decisão necessária

1. autorizar `rust/socket/src/port.rs` (opção 1) ou `apps/idestudio/src/main.rs`
   (opção 2), ou aceitar a espera (opção 3). a sonda da recusa vira o
   teste que prova a escolha (ela hoje fixa a espera).
2. junto, dois arquivos fora da lista que a implementação vai pedir de
   qualquer jeito, aditivos:
   - `rust/accord/src/redactor.rs`: a regra única de segredo é
     `Secrets::request`, e é ela que o `rec` e o porto aplicam ao request.
     sem ela, a pergunta sobre um alvo secreto vai inteira para a gravação
     e para o `asked`. a mudança: a pergunta vira `REDACTED` quando o alvo
     é secreto, como o texto livre de um erro. três linhas e um teste.
   - `rust/replay/src/why.rs`: `text::why` recebe só o `Why`, e o
     `Asked` dele não carrega o campo novo do request; mexer só em
     `text/**` não põe a pergunta no `why`. a mudança: um campo em
     `Asked`, preenchido do request gravado (aditivo também no `--json`).
     quem respondeu já aparece hoje: "confirmed: by human" nos estágios, e
     "human declined to confirm" no veredito.

### o que eu faria depois da decisão (nada implementado)

- a forma: `sem::Request.ask: Option<String>`, com `serde(default,
  skip_serializing_if = "Option::is_none")`, então um request sem pergunta
  e as sessões antigas (`MONS`) leem igual e o `rec` não muda; `Act.ask`
  no `wire`, aditivo no v1; `plev act <target> <action> --ask "<texto>"`.
- o limite: 280 caracteres, não vazio, sem caractere de controle, cobrado
  no porto como `bad_message` (onde o `MAX_NAME` já é cobrado) e no parse
  do `plev`.
- o dispatcher: `check` responde `NeedsConfirmation` quando a `Policy`
  exige ou quando um agente pergunta; quem já exigia continua igual, e a
  segunda checagem na confirmação não muda.
- uma pessoa não pergunta a outra: a pergunta de `Actor::Human` ou
  `Actor::Assistive` não pede nada, como o `Confirm::ForAgents` não pede
  nada a uma pessoa. recusar com um motivo novo exigiria uma variante em
  `Rejection`, e o `rec` tem um `match` completo sobre ela
  (`rust/record/src/weigh.rs`), o que é condição de parada.
- o modal: diz que o agente pergunta e mostra o texto dele, na terceira
  pessoa ("the person", nunca "you"); confirmar e recusar como hoje.
- a projeção: a pergunta não entra em `notices` nem num nó do modal (o
  modal não é projetado, e `project.rs` está fora da lista). o autor a vê
  no `asked` do `requested`; outro agente não vê requests alheios pela
  porta.
- ninguém responde: o request espera, como o `discard` espera hoje; não há
  api de expirar.
- o `rec` estima o peso de um request sem o texto da pergunta
  (`weigh.rs`); com 280 caracteres no máximo, é um erro pequeno e fica
  registrado.

### contagem

`cargo test --workspace` na base: 2267 passaram, 11 ignorados. com a sonda:
2269 e 11. `./tool/tasks/fullcheck.sh` verde no commit da sonda, com a perna urnaui
(82 passaram, 1 ignorado), o clippy, o fmt e o wasm check.

### ADR

nenhum escrito, porque parou. a linha que eu proporia: uma seção datada no
ADR da 58b ([`0033-actions-carry-observed-state.md`](../../adr/Semantics/0033-actions-carry-observed-state/0033-actions-carry-observed-state.md)): "um agente pode
pedir, no próprio request, que uma pessoa confirme antes; o request espera
como um `Confirm::ForAgents`".

### o que o próximo experimento com LLM precisa dizer sobre isso

- o laboratório não tem uma pessoa que responde ao modal: os modos
  `--human` só tiram um arquivo do index. para medir o handoff, ele precisa
  de uma pessoa que confirma ou recusa (o `ide-lab` está fora desta task),
  e a recusa precisa chegar ao agente (esta parada).
- o briefing do agente continua sem ensinar a regra, como na terceira
  rodada: a pergunta aparece no `plev act --help` e na doc do `Act`, e a
  corrida mede se o agente a encontra, não se obedece.
- a hipótese a escrever antes: com a pessoa antes do submit, o agente
  pergunta em vez de passar por cima (A2''a) ou de parar (A2''b), e a
  resposta da pessoa decide o commit.

### commits

- `2f61463` task-92: the probe, an agent's discard through the port and
  the person's answer

## handoff final (2026-09-24)

goal: task-92, o agente pergunta à pessoa.
status: feito na branch `task/92-ask-the-person`, rebaseada em `main @
e840904`; gate verde. falta a integração do orquestrador.

### a decisão que destravou

a sonda achou que a recusa da pessoa não chegava ao agente (seção acima).
o orquestrador classificou como P0: o porto promete os estágios dos
requests que o agente emitiu, e essa promessa quebrava num caminho que já
existia (58b). escolheu a opção 1, a correção veio primeiro e sozinha, e
liberou `rust/socket/src/port.rs`, `rust/accord/src/redactor.rs` e
`rust/replay/src/why.rs`.

### o P0: um estágio ouvido fora de um drain acorda o porto

- o sink do porto sabe quais requests o servidor espera (`Waits`, as
  chaves de `issued`) e chama o wake do porto quando ouve um estágio de um
  deles. `Server::wake_with` recebe o wake, e o `AgentPort` dá o mesmo que
  as threads de leitura já chamam. os estágios do próprio dispatch de um
  act vêm antes de o request entrar na espera, então não acordam nada. o
  clique de uma pessoa, o request de um agente que não veio pela porta e
  o request de uma conexão que fechou também não.
- cobre a recusa, a confirmação com o que roda depois, a segunda checagem
  que recusa como stale na confirmação e a ação que termina na hora, sem
  git (`open`, `set_value`, `dismiss`: provado com `open`). cobre também
  o que termina depois no worker, que antes dependia de o shell drenar
  depois do resultado do git.
- os três hosts: a janela do ide e o showcase drenam no wake do porto
  (`AppEvent::Agent`, `UserEvent::Agent`), então recebem a correção sem
  mudar o shell. o laboratório já drenava a cada `TICK`. não escrevi
  teste dentro do showcase nem do laboratório: os dois estão fora da
  lista, o showcase não tem ação que espere uma pessoa e o laboratório não
  tem pessoa que responda ao modal. a prova vale para qualquer host que
  drene no wake: um teste com socket real em que o shell do toy só drena
  no wake (`rust/socket/tests/socket.rs`) e os testes do ide com o shell
  servindo só no wake.
- custo: um drain vazio a mais por estágio tardio num shell que também
  drena depois do resultado do git (o wake que ficou na fila durante o
  `apply`). os wakes já coalescem.
- o `server.rs` estava com 610 das 639 linhas de produção: o sink foi para
  `rust/socket/src/sink.rs` num commit só de movimento, antes. agora o
  `server.rs` tem 599.

### a sonda, depois do P0

os dois testes da sonda (`apps/idestudio/tests/agent_port.rs`) usam agora um
shell que serve o porto só no wake. o do discard confirmado recebe
`confirmed`, `executing` e `completed`. o da recusa, que fixava a espera,
virou `a_decline_reaches_the_agent_with_no_message_from_it`: a pessoa
recusa e o agente recebe o `declined` sem mandar mensagem nenhuma. sem a
linha do wake em `port.rs`, os dois caem por timeout (conferido).

### a forma do pedido

- request: `sem::Request.ask: Option<String>`, escrito por último e só
  quando existe. `Request::asking(texto)` o põe, e
  `Request::asks_a_person()` diz se ele pede uma pessoa (a pergunta de um
  agente).
- `check`: o passo 8 (`NeedsConfirmation`) vale quando a `Policy` exige
  ou quando um agente pergunta. a pergunta é o último passo: um request
  stale, inerte ou desabilitado é recusado como seria sem ela, e quem já
  exigia confirmação continua igual. a segunda checagem na confirmação não
  mudou.
- `Act.ask` no `wire`, aditivo no v1. o porto limita o texto antes do
  dispatcher com `wire::MAX_ASK` (280 caracteres, uma linha, não vazio;
  `bad_message` fora disso), como já limita o nome do `hello`, e o
  `requested` devolve a pergunta em `asked`.
- `plev act <target> <action> --ask "<texto>"` imprime o
  `awaiting_confirmation` e espera. a confirmação sai com 0, ou com 1
  quando a segunda checagem recusa; a recusa sai com 1 (`declined`); um
  texto que não cabe sai com 3 antes de conectar. sem `--timeout`, espera
  300 s em vez de 30, porque uma pessoa lê antes de responder. a regra
  está no `plev act --help` e na doc do `Act`.

### o modal

título "Question from test-agent", corpo `test-agent asks the person:
"recolocar src/session.rs?" If allowed, it will stage src/session.rs.`,
botões "Allow" e "Cancel". num discard, o botão continua "Discard" e o
corpo mantém "This cannot be undone.". fica na terceira pessoa ("the
person", nunca "you"). confirmar e recusar seguem como antes, pelo
dispatcher como `Actor::Human`. o texto nasce em `Workbench::confirmation`
(`apps/idestudio/src/capability/notice.rs`).

### as decisões

- a pergunta não entra na projeção: nem em `notices` nem num nó do modal,
  que não é projetado. o autor a vê no `asked` do `requested`; um outro
  agente não vê requests alheios pela porta. o toast de recusa continua
  dizendo o que dizia.
- uma pessoa não pergunta a outra: a pergunta de `Actor::Human`, de
  `Actor::Assistive` ou de um teste não pede nada, como o
  `Confirm::ForAgents` não pede nada a eles. recusar exigiria uma
  variante nova em `Rejection`, e o `rec` faz um `match` completo nela.
- ninguém responde: o request espera, como o `discard`; não há api de
  expirar. o `plev` desiste depois do `--timeout`, e aí o estágio se perde
  para aquela conexão, mas o agente ainda pode observar o resultado.
- segredo: a pergunta sobre um alvo secreto vira `REDACTED` em
  `Secrets::request`, a regra única que o `rec` e o porto aplicam.
- dentro do processo, o código do app não é limitado: como o nome do
  agente, o limite vale na porta, que é por onde entra texto de fora.
- o `why` do inspector ganha duas linhas quando há pergunta: "asked" (o
  ator e a pergunta) e "answered" (quem confirmou ou recusou, e quando).

### o aceite

- sem: seis testes em `rust/accord/tests/dispatch.rs` (o pedido espera,
  confirmado roda com a segunda checagem, recusado não roda, quem já
  exigia continua igual, pessoa e teste não pedem nada, a pergunta nunca
  passa de uma recusa), a forma serde em `rust/accord/tests/encoding.rs` e a
  redação em `redact.rs`.
- wire: a forma de `Act` com e sem pergunta (`codec.rs`); pela porta, a
  espera, a confirmação, a recusa e a segunda checagem, o texto limitado
  antes do dispatcher e redigido num alvo secreto (`server.rs`); o wake
  (`server.rs`, `socket.rs`); o `plev act --ask` com
  `awaiting_confirmation` e depois o estágio terminal, 0 na confirmação e
  1 na recusa (`cli.rs`); o parse e o help (`plev.rs`).
- ide: o A2''b pela porta real, com a pessoa respondendo no modal pelo
  dispatcher. o agente observa, põe os três arquivos no stage e escreve a
  mensagem; a pessoa tira `src/session.rs`; o submit é recusado stale
  nomeando a pessoa; o agente pergunta "recolocar src/session.rs?", e o
  modal mostra o texto.
  - `the_agent_asks_the_person_who_confirms_and_the_three_files_are_committed`:
    a pessoa confirma, o stage roda e o agente commita os três.
  - `the_agent_asks_the_person_who_declines_and_it_stops`: a pessoa
    recusa, o agente recebe o `declined` e para; nada é commitado, e
    `src/session.rs` fica fora do index.
  - `what_ends_at_once_or_at_the_second_check_reaches_the_agent`.
  - `an_agents_question_is_asked_in_the_modal_and_never_projected`, em
    `workbench/tests.rs`.
- inspector: a sessão gravada do teste confirmado fica em
  `$CARGO_TARGET_TMPDIR/ide-ask-test.monster`. `plev-inspect <arquivo>
  --why 8` mostra:

```text
request 8 (origin 0): stage on changes.file[src/session.rs] by agent:test-agent, at +0.196s
  control       changes.file[src/session.rs]: list_item "src/session.rs", enabled
  state         status = "added", staged = false, unstaged = true
  initiated by  agent:test-agent
  action        stage, no parameter
  asked         agent:test-agent asked a person first: "recolocar src/session.rs?"
  answered      human confirmed it, at +0.197s
  observed      +0.196s, the whole app, at revision 684de68c20da1cee
  ...
  stages        +0.196s requested
                +0.196s awaiting_confirmation
                +0.197s confirmed: by human
                +0.197s executing
                +0.219s completed
  verdict       waited for a person to confirm, from +0.196s
  result        completed at +0.219s
```

### validação e contagem

`./tool/tasks/fullcheck.sh` verde depois do rebase em `e840904`, com teste, clippy
`-D warnings`, fmt, wasm check e a perna urnaui. `cargo test --workspace`:
na base eram 2267 passando e 11 ignorados; agora são 2289 e 11 (+22). a
perna urnaui tem 82 passando e 1 ignorado, igual. os testes novos da porta
(`agent_port`, `socket`) rodaram seis e três vezes seguidas, todas verdes.

### ADR

seção datada "updated on 2026-09-24 (task-92): an agent asks the person"
em [`0033-actions-carry-observed-state.md`](../../adr/Semantics/0033-actions-carry-observed-state/0033-actions-carry-observed-state.md), com o P0 descrito. a linha
para o índice, se o orquestrador quiser: "um agente pode pedir, no próprio
request, que uma pessoa confirme antes (`Request::ask`, `Act::ask`, `plev
act --ask`); o porto se acorda quando ouve, fora de um drain, um estágio
de um request seu (P0 da task-92)".

### limitações e o que fica para a integração

- `apps/idestudio/src/capability/agent.rs` (proibido) ainda diz que os estágios tardios
  esperam a próxima mensagem se o shell não drenar depois do worker. com
  o P0 isso deixou de valer.
- [`0039-agents-act-through-the-semantic-contract.md`](../../adr/Semantics/0039-agents-act-through-the-semantic-contract/0039-agents-act-through-the-semantic-contract.md), o `README.md`,
  o [`docs/ATLAS.toml`](../../ATLAS.toml) e o [`changelog.md`](../../CHANGELOG.md) não mudaram. o ADR do
  porto não cita o `ask` nem o wake do sink; a seção do ADR da 58b cobre
  os dois.
- o `rec` estima o peso de um request sem o texto da pergunta
  (`weigh.rs`): no máximo 280 caracteres por request, um erro pequeno.
- não medi em pixel um modal com 280 caracteres, porque não há janela
  nesta task.
- o `Traffic` do porto não conta atos com pergunta; a transcrição conta.

### o que o próximo experimento com LLM precisa dizer sobre isso

- o laboratório não tem uma pessoa que responde ao modal: os modos
  `--human` só tiram um arquivo do index. para medir o handoff, precisa de
  um modo em que a pessoa confirma ou recusa (por exemplo, depois de um
  atraso), em `apps/agenteval/src/**`, fora desta task. o P0 já faz a
  resposta chegar ao agente.
- o briefing do agente não ensina a pergunta, como a terceira rodada não
  ensinou a regra da task-89: ela está no `plev act --help` e na doc do
  `Act`, e a corrida mede se o agente a encontra, não se obedece.
- escrever antes, como hipótese: com a pessoa antes do submit
  (`unstage-before-submit`), o agente pergunta em vez de passar por cima
  (A2''a) ou de parar (A2''b), e a resposta da pessoa decide o commit.
  contar, por corrida: perguntas feitas, confirmadas e recusadas; atos
  que passaram por cima da pessoa sem perguntar; o que o agente faz
  depois de um `declined`.
- o `plev act --ask` espera 300 s por padrão. o laboratório deve dizer em
  quanto tempo a pessoa responde, para não medir o timeout.

### commits

- `7623b7f` task-92: the probe, an agent's discard through the port and
  the person's answer
- `e3cf244` task-92: stopped at the probe; the handoff with the decision
  it needs
- `01b9b2a` task-92: the port's sink moves to its own module, with no
  change
- `ae544b7` task-92: a stage heard outside a drain wakes the port (P0)
- `d56ef44` task-92: an agent's request can ask a person first (sem)
- `64011de` task-92: the act asks a person over the port, plev act --ask
- `9792058` task-92: the why shows the question and who answered it
- `82202ab` task-92: the ide asks the person the agent's question
- `d293046` task-92: what ends at once or at the second check reaches the
  agent too
- o commit deste handoff, com o ADR

## fechamento (orquestrador, 2026-09-24)

integrada na `main` em `f101359` (dez commits, rebaseados sobre
`426e4b7` sem conflito).

validação do orquestrador:
- diff lido: o sink em `wire/src/sink.rs` (movimento) com `Waits`, o
  wake no `port.rs`, `Request.ask` e `asks_a_person`, a ordem no `check`
  (a pergunta é o último passo), o limite e a redação, o modal, o `why`;
- gate verde no diretório da task: 2401 passaram, 0 falharam, 13
  ignorados, com a perna urnaui;
- os testes do porto do ide rodaram três vezes seguidas, verdes;
- a prova do P0 repetida pelo orquestrador: sem a linha do wake no
  `port.rs`, `a_decline_reaches_the_agent_with_no_message_from_it` falha
  por timeout em 20 s; com ela, passa. o arquivo foi restaurado.

aplicado no fechamento: a doc de `apps/idestudio/src/capability/agent.rs` e a
consequência no ADR agents-act-through (o wake e o `ask`); o índice de
ADR e o CHANGELOG.

severidade do que ficou:
- P2: o laboratório não tem pessoa que responde ao modal. o próximo
  experimento com LLM precisa dela, para medir se o agente pergunta em
  vez de passar por cima ou parar;
- P3: o `rec` estima o peso do request sem a pergunta (no máximo 280
  caracteres).

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0033: actions carry the state their actor observed](../../adr/Semantics/0033-actions-carry-observed-state/0033-actions-carry-observed-state.md)
- [ADR 0039: agents act through the semantic contract](../../adr/Semantics/0039-agents-act-through-the-semantic-contract/0039-agents-act-through-the-semantic-contract.md)
- [ADR 0058: a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline](../../adr/Interface/0058-native-dialogs-answer-the-same-contract/0058-native-dialogs-answer-the-same-contract.md)
- [what the autonomous cycle taught: a native dialog answers the same contract: the alert is a person's capability, the confirmation is the same confirm or decline](../../adr/Interface/0058-native-dialogs-answer-the-same-contract/learned-native-dialogs-answer-the-same-contract.md)
- [task-58: action with precondition, dispatch and receipts](../task-58-actions-preconditions-receipts/task-58-actions-preconditions-receipts.md)
- [task-58b: declining a confirmation](../task-58b-decline-a-confirmation/task-58b-decline-a-confirmation.md)
- [task-80: proof of qualitatively different software](../task-80-proof-qualitatively-different-software/task-80-proof-qualitatively-different-software.md)
- [task-89: an act relies only on what was shown to the agent](../task-89-an-act-rests-only-on-what-was-shown/task-89-an-act-rests-only-on-what-was-shown.md)
- [task-91: the dispatcher knows when the agent looked](../task-91-the-dispatcher-knows-when-the-agent-looked/task-91-the-dispatcher-knows-when-the-agent-looked.md)
- [task-93: the person in the lab answers](../task-93-the-lab-person-answers/task-93-the-lab-person-answers.md)
- [task-94: the refusal that names a person points to the question](../task-94-the-refusal-points-at-the-question/task-94-the-refusal-points-at-the-question.md)
- [task-62 phase c, fourth round: the handoff (2026-09-24)](../task-62-ide-human-agent-proof/phase-c-round-4.md)
- [task-76, item 8: native alert and confirmation dialogs](../task-76-platform-integration/item-8-native-alerts.md)
- [task-80, phase 1: probe, five-demo method, demos 3 and 4 and handoff](../task-80-proof-qualitatively-different-software/phase-1-probe-and-demos.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
