---
id: task-86
title: "task-86: o ide que alguém usa abre a porta do agente"
project: plev
type: tarefa
status: concluido
date: 2026-09-23
date_basis: cabecalho
topics: [agent-surface]
relations: [task-61, task-62, task-82]
source: docs/log/steps/task-86-the-ide-opens-the-agent-port.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-86: o ide que alguém usa abre a porta do agente

## por que

o cenário da tese é "uma pessoa usa o app enquanto um agente o opera, os
dois pelo mesmo contrato". os dois checkpoints a rodaram no `ide-lab`: o
workbench sem janela e uma pessoa roteirizada. o binário `ide`, o que
alguém abre de verdade, não usa `wire` e não abre porta nenhuma
(`apps/idestudio/src/main.rs`). enquanto isso não existir, a afirmação
principal do programa só está provada em laboratório, e o teste que só o
brenner pode fazer (ele usando o ide enquanto um agente age) não tem como
acontecer.

## estado no código (main @ 91a7319)

- `apps/idestudio/src/main.rs`: o shell do ide (winit), com o `Workbench`,
  o worker do git, o watcher, e desde a task-82 o `pace` e o tick de
  alcance. 655 linhas, acima do limite de 639: precisa dividir para
  crescer.
- `apps/agenteval/src/host.rs:232-263`: o `Host` do laboratório
  sobre o `Workbench`, com gravação, custos e a pessoa roteirizada.
- `apps/uigallery/src/views/dispatcher/mod.rs:322-371` e
  `apps/uigallery/src/shell/mod.rs:178-185`, `:295`, `:486-494`: como o
  showcase abre a porta (`AgentPort::open`, `PLEV_AGENT=1` em debug),
  acorda o loop (`UserEvent::Agent`), serve no thread do app e desenha só
  se um ato rodou; o `bounds` com o retângulo visível de cada ligação.
- o laboratório drena a porta também depois de cada evento do worker do
  git, porque os estágios tardios (execução que termina no worker) só
  saem assim.

## escopo

1. um `Host` do `Workbench` na lib do ide (projeção, `bounds` das
   ligações do frame, dispatch, sinks), usado pelo shell; o laboratório
   pode delegar a ele a parte comum.
2. o shell do ide abre a porta como o showcase (opt-in, `PLEV_AGENT=1`
   em debug), acorda por evento, drena no thread do app, drena de novo
   depois de cada evento do worker, desenha só quando um ato rodou.
3. prova numa janela de verdade: o ide aberto num repositório, um agente
   pelo `plev` observa, faz stage e commita; e uma pessoa (teclado ou
   clique real) mexendo ao mesmo tempo, com uma recusa stale vista pelos
   dois lados.

## fora do escopo

a ponte de acessibilidade no ide (task-75); a página Git do showcase
alcançável por agente; o laboratório (task-88).

## aceite

teste do `Host` da lib por uma porta real sem janela; o ide real servindo
`plev observe`/`act` com saída colada; a cena com a pessoa e o agente e a
recusa; gate verde.

## o que foi feito (branch `task/86-ide-port`, sobre a main em `af8302f`)

- o `main.rs` do ide foi dividido antes de crescer: o pacing (alcance,
  wake do frame, a espera) foi para `apps/idestudio/src/interfaces/window/pace.rs` com os dois
  testes de relógio injetado, e o teclado (keymap, contextos, ações) para
  `apps/idestudio/src/interfaces/window/keyboard.rs`. mesmo código, mesmos testes. o `main.rs` tem
  461 linhas com a porta.
- o retângulo visível de uma ligação saiu do `Host` do showcase e foi
  para o engine, ao lado da ligação: `Binding::visible_rect` (o retângulo
  dentro do clip, `None` quando nada aparece) e
  `Compositor::visible_bindings` (as ligações que aparecem, com esse
  retângulo, na ordem do bind). o `bounds` do showcase lê dali; os testes
  de agente do showcase não mudaram e o `regress -- check` da task-67
  deu 157 baselines sem mudança.
- `apps/idestudio/src/capability/agent.rs` (lib, só unix): `Served { bench, compositor }`
  implementa `wire::Host` (`project` = `bench.snapshot().ok()`, com o
  erro de projeção no log; `bounds` = `visible_bindings`; `dispatch` =
  `bench.dispatch`; `add_sink` = `bench.add_sink`; `observed` padrão),
  `serve(port, bench, compositor) -> bool` (verdadeiro quando um ato
  rodou) e `port(wake)`, que abre com `AgentPort::from_env("ide", wake)`.
- o shell: `AppEvent::Agent` acorda o loop; `user_event` serve a porta
  nesse evento e de novo depois de aplicar cada `AppEvent::Git` (os
  estágios tardios de stage, unstage e commit só saem aí); desenha só se
  um ato rodou; o alcance da task-82 continua no `new_events`, antes de
  tudo. a porta abre no `main` depois do `App::new`, como o showcase.
- o laboratório ficou como estava (ver o handoff).

### o briefing e o código divergiam num ponto

o briefing pedia `AgentPort::open("ide", wake)` dizendo que ele "já
respeita `PLEV_AGENT=1` e build de debug". não respeita:
`AgentPort::open` é o opt-in explícito do app e abre em qualquer build
(`rust/socket/src/port.rs`, doc de `open`); quem olha a variável e o
debug é `AgentPort::from_env`. o contrato que não pode mudar é
"`PLEV_AGENT=1` só em debug, release fechado", então o ide usa
`from_env`, como o showcase.

## o teste do `Host` da lib

`apps/idestudio/tests/agent_port.rs`, só unix, dois testes. um `Rig` real
(repositório num tempdir, o worker do git de verdade) e uma porta real
num tempdir, sem janela. a thread do teste faz o papel do shell do ide:
no wake da porta chama `ide::agent::serve` e redesenha se um ato rodou;
num evento do worker aplica, serve e redesenha; entre duas mensagens do
agente pode aplicar eventos até os refreshes assentarem ou fazer a
pessoa tirar um arquivo do index (pelo dispatcher, como `Actor::Human`,
o que o menu "Unstage file" pede). a thread do agente fala o protocolo
com `wire::Client`.

- `an_agent_stages_a_file_and_the_last_stage_leaves_after_the_worker`:
  hello responde `welcome` com `app: "ide"`; a observação com `bounds`
  traz as linhas e onde cada id ligado aparece (dentro da janela, todo
  id ligado também projetado); o stage de `changes.file[A]` manda três
  estágios no drain que o despachou e o quarto (`completed`, com a
  revisão) só no drain que o shell roda logo depois de aplicar
  `GitEvent::Staged`, em nenhum antes; só esse drain teve ato; `since`
  a partir da revisão do estágio terminal responde delta (o anel
  guardou), e o git tem A no index. controle negativo, feito à mão: sem
  o `serve` depois do evento do worker o agente espera o `completed`
  até o timeout de 20 s e o teste falha.
- `a_person_unstaging_between_the_agents_look_and_its_submit_is_named_in_both_places`:
  o agente põe A e B no index, abre o formulário e escreve a mensagem,
  cada ato sobre a revisão que o anterior carregou; decide sobre uma
  observação; a pessoa tira B; o `submit` volta `rejected` `stale` com
  `changes:staged` {A, B} -> {A} e `acted` com um só ator, `human`,
  `unstage` em `changes.file[B]`; o toast projetado diz "Refused
  test-agent's submit: B left the staged files (the person unstaged
  B)", o mesmo texto que o agente lê em `notices`; o agente observa de
  novo e submete, e o commit sai só com A, um commit a mais que o
  inicial. a recuperação observa antes de submeter de propósito: a
  revisão de um estágio terminal nomeia um instante e não mostra estado,
  e a task-89 (aberta na main) faz um ato se apoiar só no que a conexão
  recebeu; encadeado na revisão da recusa, o `submit` leria um
  `changes:staged` mostrado antes da pessoa mexer.

## prova numa janela de verdade

### método

build de debug desta branch em `68af0b1` (rebaseada sobre a main em
`c9cc8de`; o rebase seguinte, sobre `af8302f`, trouxe só a crate
`inspect` da task-64 e o arquivo da task-89, que o ide não usa), numa janela
real no macOS (apple m4). o ide sobe com `PLEV_AGENT=1
RUST_LOG=ide=debug,info cargo run -p idestudio -- <repo>` num repositório de
rascunho com um commit, dois arquivos modificados (`src/login.rs`,
`src/session.rs`) e um não rastreado (`todo.txt`). o agente é o `plev`
desta branch (`--app ide`, nome `claude` por `PLEV_AGENT_NAME`), uma
invocação por passo, cada ato sobre a revisão que o anterior carregou
(`--from-revision`). a pessoa é CGEvent de verdade: cliques postados no
tap do HID, depois de trazer o ide para a frente e conferir que a janela
de cima naquele ponto é a do ide; o Escape do fim, postado no pid. as
coordenadas do clique saem do `--bounds` do próprio `plev` (retângulo
lógico da ligação) mais a origem da janela e a barra de título que o
`CGWindowList` dá. os frames são a linha `frame:` que o shell escreve a
cada `RedrawRequested` (task-82), carimbada na chegada por um script que
lê o stderr. o harness (swift e shell) ficou no rascunho da sessão, fora
da árvore. rodei a cena duas vezes, antes e depois do rebase sobre a
main; as duas deram o mesmo resultado, e o que está abaixo é a segunda.

### parte 1: observar, fazer stage e commitar pelo contrato

o socket é `$TMPDIR/plev/ide-89878.sock` (pid 89878). a primeira
observação, com os nós cortados aqui (quatro: `changes` e as três
linhas) e os `bounds` inteiros:

```text
$ plev observe --app ide --scope changes --keys files,staged,status --bounds
{"v":1,"id":"2","type":"snapshot","revision":"7058d1b06c39f608","nodes":[...4 nós...],"focused":null,"bounds":[{"id":"changes","rect":[72.0,52.0,280.0,748.0]},{"id":"changes.file[src/login.rs]","rect":[84.0,104.0,256.0,44.0]},{"id":"changes.file[src/session.rs]","rect":[84.0,152.0,256.0,44.0]},{"id":"changes.file[todo.txt]","rect":[84.0,200.0,256.0,44.0]}]}
exit 0
```

o estado de `changes` nela: `{"files":{"kind":"list","value":["src/login.rs","src/session.rs","todo.txt"]},"staged":{"kind":"set","value":[]}}`.
mais quatro `plev observe --scope changes --keys staged` voltaram a
mesma revisão. as cinco observações e o segundo depois delas: 0 frames.

```text
$ plev act changes.file[todo.txt] stage --from-revision 7058d1b06c39f608 --app ide
{"v":1,"id":"2","type":"stage","request":"1","stage":"requested","at":9279,"asked":{"actor":{"agent":{"name":"claude"}},"target":"changes.file[todo.txt]","action":"stage","observed":{"revision":"7058d1b06c39f608","values":[["changes.file[todo.txt]:staged",{"kind":"bool","value":false}],["changes.file[todo.txt]:unstaged",{"kind":"bool","value":true}]]}}}
{"v":1,"id":"2","type":"stage","request":"1","stage":"accepted","at":9279}
{"v":1,"id":"2","type":"stage","request":"1","stage":"executing","at":9279}
{"v":1,"id":"2","type":"stage","request":"1","stage":{"completed":{"result":{}}},"at":9293,"revision":"952eb78fcefa4863"}
exit 0
$ plev act commit.form open --from-revision 952eb78fcefa4863 --app ide
{"v":1,"id":"2","type":"stage","request":"2","stage":"requested","at":9304,"asked":{"actor":{"agent":{"name":"claude"}},"target":"commit.form","action":"open","observed":{"revision":"952eb78fcefa4863","values":[["commit.form:open",{"kind":"bool","value":false}]]}}}
{"v":1,"id":"2","type":"stage","request":"2","stage":"accepted","at":9304}
{"v":1,"id":"2","type":"stage","request":"2","stage":"executing","at":9304}
{"v":1,"id":"2","type":"stage","request":"2","stage":{"completed":{"result":{}}},"at":9304,"revision":"6a4c963d6494a378"}
exit 0
$ plev act commit.message set_value --param '"add the todo list"' --from-revision 6a4c963d6494a378 --app ide
{"v":1,"id":"2","type":"stage","request":"3","stage":"requested","at":9324,"asked":{"actor":{"agent":{"name":"claude"}},"target":"commit.message","action":"set_value","param":{"kind":"text","value":"add the todo list"},"observed":{"revision":"6a4c963d6494a378","values":[["commit.message:text",{"kind":"text","value":""}]]}}}
{"v":1,"id":"2","type":"stage","request":"3","stage":"accepted","at":9324}
{"v":1,"id":"2","type":"stage","request":"3","stage":"executing","at":9324}
{"v":1,"id":"2","type":"stage","request":"3","stage":{"completed":{"result":{}}},"at":9324,"revision":"4d3b949e8339e00b"}
exit 0
$ plev act commit.submit submit --from-revision 4d3b949e8339e00b --app ide
{"v":1,"id":"2","type":"stage","request":"4","stage":"requested","at":9343,"asked":{"actor":{"agent":{"name":"claude"}},"target":"commit.submit","action":"submit","observed":{"revision":"4d3b949e8339e00b","values":[["changes:staged",{"kind":"set","value":["todo.txt"]}],["commit.message:text",{"kind":"text","value":"add the todo list"}]]}}}
{"v":1,"id":"2","type":"stage","request":"4","stage":"accepted","at":9343}
{"v":1,"id":"2","type":"stage","request":"4","stage":"executing","at":9343}
{"v":1,"id":"2","type":"stage","request":"4","stage":{"completed":{"result":{"commit":{"kind":"text","value":"59e48e7a6eeb0800683220018c6846d4d6feeeae"},"files":{"kind":"list","value":["todo.txt"]}}}},"at":9380,"revision":"9b68d204aba4d22f"}
exit 0
```

o stage terminou 14 ms depois do `executing` e o commit 37 ms depois,
os dois no drain que o shell roda depois de aplicar o evento do worker.
o git depois: `59e48e7 add the todo list`, com `todo.txt`.

### parte 2: as duas mãos

o agente põe os dois arquivos no index, abre o formulário e escreve a
mensagem, cada ato sobre a revisão do anterior (as quatro saídas têm a
forma da parte 1; os estágios terminais):

```text
$ plev act changes.file[src/login.rs] stage --from-revision 9b68d204aba4d22f --app ide
{"v":1,"id":"2","type":"stage","request":"5","stage":{"completed":{"result":{}}},"at":13445,"revision":"fadd05238d8c2ef5"}
$ plev act changes.file[src/session.rs] stage --from-revision fadd05238d8c2ef5 --app ide
{"v":1,"id":"2","type":"stage","request":"6","stage":{"completed":{"result":{}}},"at":13474,"revision":"63913d1bcdbbd9d2"}
$ plev act commit.form open --from-revision 63913d1bcdbbd9d2 --app ide
{"v":1,"id":"2","type":"stage","request":"7","stage":{"completed":{"result":{}}},"at":13484,"revision":"50ea536b19adcef9"}
$ plev act commit.message set_value --param '"login and session"' --from-revision 50ea536b19adcef9 --app ide
{"v":1,"id":"2","type":"stage","request":"8","stage":{"completed":{"result":{}}},"at":13505,"revision":"c3f8c786fc6f197f"}
$ plev observe --app ide --scope changes.file[src/session.rs] --keys staged --bounds
{"v":1,"id":"2","type":"snapshot","revision":"c3f8c786fc6f197f","nodes":[...1 nó...],"focused":null,"bounds":[{"id":"changes.file[src/session.rs]","rect":[84.0,272.0,256.0,44.0]}]}
```

a janela está em (116, 63), 1280 x 832 pontos com 32 de barra de
título; o centro da linha `src/session.rs` é (212, 294) na janela,
(328, 389) na tela.

1. a pessoa clica com o botão esquerdo na linha `src/session.rs`: ela
   fica selecionada e o diff dela abre (um `show_diff` da pessoa pelo
   dispatcher).
2. o agente decide (extrato da resposta: a revisão e o estado de
   `changes`):

   ```text
   $ plev observe --app ide --scope changes --keys staged,selected
   "revision":"d9bddbd9b434c731" ... {"id":"changes",...,"state":{"selected":{"kind":"text","value":"src/session.rs"},"staged":{"kind":"set","value":["src/login.rs","src/session.rs"]}}}
   ```

3. a pessoa clica com o botão direito na mesma linha (o menu do arquivo
   abre; nada projetado muda) e clica em "Unstage file" em (411, 418).
4. o agente submete sobre a decisão, e a recusa diz quem foi:

   ```text
   $ plev act commit.submit submit --from-revision d9bddbd9b434c731 --app ide
   {"v":1,"id":"2","type":"stage","request":"11","stage":"requested","at":18782,"asked":{"actor":{"agent":{"name":"claude"}},"target":"commit.submit","action":"submit","observed":{"revision":"d9bddbd9b434c731","values":[["changes:staged",{"kind":"set","value":["src/login.rs","src/session.rs"]}],["commit.message:text",{"kind":"text","value":"login and session"}]]}}}
   {"v":1,"id":"2","type":"stage","request":"11","stage":{"rejected":{"stale":{"changes":[{"state_changed":{"at":"changes:staged","before":{"kind":"set","value":["src/login.rs","src/session.rs"]},"after":{"kind":"set","value":["src/login.rs"]},"added":[],"removed":["src/session.rs"]}}],"acted":[{"id":"10","actor":"human","target":"changes.file[src/session.rs]","action":"unstage"}]}}},"at":18782,"revision":"2c0325523ec92702"}
   exit 1
   ```

5. na tela do ide, um segundo depois (captura da janela), o toast no
   canto inferior direito: "Refused claude's submit: src/session.rs
   left the staged files (the person unstaged src/session.rs)"; a linha
   `src/session.rs` sem o ponto de staged, `src/login.rs` com ele, o
   formulário com "login and session". o agente lê o mesmo texto, na
   mesma revisão que a recusa carregou:

   ```text
   $ plev observe --app ide --scope notices
   {"v":1,"id":"2","type":"snapshot","revision":"2c0325523ec92702","nodes":[{"id":"notices[0]","role":"alert","label":"Refused claude's submit: src/session.rs left the staged files (the person unstaged src/session.rs)","parent":"notices","actions":[{"name":"dismiss"}]},{"id":"notices","role":"region","label":"notices"}],"focused":null}
   exit 0
   ```

6. o agente se recupera sobre a revisão da recusa, respeitando o que a
   pessoa deixou no index:

   ```text
   $ plev act commit.submit submit --from-revision 2c0325523ec92702 --app ide
   {"v":1,"id":"2","type":"stage","request":"12","stage":"requested","at":19787,"asked":{"actor":{"agent":{"name":"claude"}},"target":"commit.submit","action":"submit","observed":{"revision":"2c0325523ec92702","values":[["changes:staged",{"kind":"set","value":["src/login.rs"]}],["commit.message:text",{"kind":"text","value":"login and session"}]]}}}
   {"v":1,"id":"2","type":"stage","request":"12","stage":"accepted","at":19787}
   {"v":1,"id":"2","type":"stage","request":"12","stage":"executing","at":19787}
   {"v":1,"id":"2","type":"stage","request":"12","stage":{"completed":{"result":{"commit":{"kind":"text","value":"74b5c05483688631df3bc2a5c8781002d2e886b9"},"files":{"kind":"list","value":["src/login.rs"]}}}},"at":19827,"revision":"308fc36de5ac0142"}
   exit 0
   ```

   o git depois: `74b5c05 login and session` só com `src/login.rs`, e
   ` M src/session.rs` na worktree, como a pessoa deixou.

o que a tela disse e o que o agente recebeu batem: a recusa nomeia o
mesmo ator e a mesma ação nos dois lados, e o texto do toast é o nó que
o agente observa.

7. Escape (postado no pid) fecha o ide; o socket some com a porta.

### frames

contados pela linha `frame:` do shell, da mesma rodada:

| trecho | frames |
| --- | --- |
| subida | 2 |
| da subida até a primeira observação, ninguém mexendo (7,4 s) | 0 |
| cinco observações e o segundo depois | 0 |
| parte 1, quatro atos e os resultados do worker | 12 |
| parte 2, os quatro atos do agente | 14 |
| clique esquerdo na linha (seleção, diff, caret parado) | 4 |
| a observação da decisão | 0 |
| clique direito, menu, "Unstage file" | 10 |
| recusa: o toast entrando | 55 |
| recuperação: o commit e o refresh | 8 |
| o toast saindo, 4 s depois da recusa | 56 |
| depois do último frame, ninguém mexendo (13,9 s) | 0 |

a porta aberta e parada não desenha, e observação não desenha: 0 frames
nos 7,4 s entre a subida e a primeira observação (o aceite pedia 6 s),
e 0 até o primeiro ato, 8,9 s depois da subida, com as cinco
observações no meio. o
que desenha é ato (do agente ou da pessoa) e o que ele pede depois: os
eventos do worker (cada um invalida, como antes desta task) e a mola do
toast (~1 s para entrar e para sair nesta rodada).

## handoff

```text
Goal: task-86, o binário ide com janela abre a porta do agente (opt-in)
      e uma pessoa e um agente operam o mesmo workbench pelo mesmo
      contrato
Status: pronto para validação do orquestrador

Implementado:
- ide: main.rs dividido (pace.rs com os testes de pacing, keys.rs com o
  teclado), sem mudar comportamento
- engine: Binding::visible_rect e Compositor::visible_bindings, aditivos,
  com teste; o bounds do showcase passa a ler dali
- ide lib: apps/idestudio/src/capability/agent.rs (só unix): Served (Host sobre
  &mut Workbench e &Compositor), serve(port, bench, compositor) -> bool,
  port(wake) (from_env, app "ide"), APP = "ide"
- ide shell: AppEvent::Agent; serve no wake da porta e depois de cada
  AppEvent::Git aplicado; desenha só quando um ato rodou; alcance da
  task-82 intacto no new_events

Arquivos principais:
- apps/idestudio/src/{main.rs, pace.rs (novo), keys.rs (novo), agent.rs
  (novo), lib.rs}, apps/idestudio/Cargo.toml (só o comentário do wire: a
  lib agora o usa; ele já estava em [dependencies])
- rust/engine/src/drawing/compositor/binding.rs
- apps/uigallery/src/views/dispatcher/mod.rs (só o bounds; 776 -> 775 linhas,
  continua na lista congelada do file_hygiene_guard)
- apps/idestudio/tests/agent_port.rs (novo), apps/idestudio/tests/support/mod.rs
  (Rig::event)

Interfaces adicionadas/alteradas:
- adicionadas: engine Binding::visible_rect, Compositor::visible_bindings;
  ide::agent::{APP, port, serve, Served}; Rig::event nos testes
- nenhuma alterada; wire, sem, Workbench e o protocolo v1 sem mudança

Testes:
- 3 novos: engine the_visible_rect_is_the_part_inside_the_clip; ide
  an_agent_stages_a_file_and_the_last_stage_leaves_after_the_worker e
  a_person_unstaging_between_the_agents_look_and_its_submit_is_named_in_both_places
- o primeiro de ide falha (timeout de 20 s) sem o serve depois do evento
  do worker, conferido à mão

Validação executada:
- ./tool/tasks/fullcheck.sh verde em 8e383b3 (rebase sobre a main em af8302f), com a
  perna urnaui: 2202 passed, 0 failed, 12 ignored (87 "test result").
  3 desses testes são desta task (engine 1, ide 2); o resto da subida
  desde os 2128 da task-82b veio com a main (task-67, o
  file_hygiene_guard, a task-64)
- antes, o mesmo gate em 68af0b1 (sobre a main em c9cc8de): 2166 passed,
  0 failed, 12 ignored (83 "test result")
- cargo test -p idestudio, -p uigallery, -p engine verdes dentro do gate; ide
  por binário: bin 9, agent_port 2, human_agent 20, lab 2, observe_cost
  0 (1 ignorado)
- cargo run -p regress -- check: 157 baselines, 0
  mudaram
- prova na janela real, duas rodadas (antes e depois do rebase), saída
  colada acima

Decisões:
- from_env em vez de open: open abre em qualquer build, o contrato é
  PLEV_AGENT=1 só em debug (o briefing dizia o contrário do código)
- o retângulo visível mora no engine, na ligação, e os dois hosts leem
  dali; o mapeamento para wire::Bound fica em cada host (o engine não
  depende do wire)
- o shell serve depois de cada AppEvent::Git, não só dos que encerram um
  pedido: o drain sem mensagem é barato (forward vazio não projeta) e o
  shell não precisa saber quais eventos encerram o quê
- o laboratório ficou como estava (ver abaixo)

Limitações:
- acted nomeia só os pedidos validados contra exatamente a revisão em
  que o agente se apoiou (rust/accord/src/dispatch.rs, acted:
  validated_against == observed.revision). na cena, a pessoa selecionou
  a linha antes da decisão do agente; se ela clicasse numa linha nova
  depois da decisão (seleção e show_diff mudam a revisão) e só então
  tirasse o arquivo do index, o unstage seria validado contra outra
  revisão, a recusa sairia stale com o delta certo e acted vazio, e o
  toast sem "(the person ...)". a tela e o agente continuariam dizendo a
  mesma coisa. lido no código, não rodado na janela. é do sem, fora
  desta task
- todo AppEvent::Git invalida (apply_git_event, antes desta task): o
  refresh do watcher depois de um commit desenha uns 3 frames sem
  mudança visível
- a árvore de accessibility (rust/engine/src/control/accessibility/tree/mod.rs,
  physical) calcula o mesmo retângulo à mão; pode usar
  Binding::visible_rect (arquivo fora da lista desta task)

Dependências pendentes:
- nenhuma para integrar

Riscos de integração:
- o teste agent_port sobe processos git com a porta aberta; o vazamento
  de descritor que o ADR do wire descreve só aparece se a porta cair com
  um filho vivo, o que o teste não confere
- a prova na janela é macOS (apple m4, debug); o shell compila a porta
  só em unix
- task-89: a prova na janela encadeia `plev act --from-revision` na
  revisão do ato anterior, cada `plev` numa conexão nova (truth 4f). com
  a regra da task-89 (um ato se apoia no que a própria conexão recebeu),
  esse encadeamento pelo cli não carrega nada mostrado àquela conexão, e
  a recuperação sobre a revisão da recusa leria um staged velho. o teste
  agent_port já observa antes de cada decisão que lê estado; a cena da
  janela, refeita depois da task-89, precisa de um observe antes do
  submit da recuperação (e a task-89 decide o que um plev sem estado de
  conexão faz com --from-revision)

ADR proposto:
- cabe como consequência no agents-act-through-the-semantic-contract:
  trocar o "updated on 2026-09-23: the ide binary does not use wire and
  opens no port" por "task-86: the ide's window opens the port
  (PLEV_AGENT=1, debug) through ide::agent, a Host over the Workbench and
  the frame's bindings; the shell drains after the port's wake and after
  every git result it applies, so late stages go out; the visible part of
  a binding is Binding::visible_rect in the engine, read by both hosts"

Para a task-88 (o laboratório):
- LabHost pode delegar a ide::agent::Served (campos públicos) o que é
  comum: project, dispatch, add_sink, e ganharia bounds (o laboratório
  hoje não serve bounds). não coube aqui: o LabHost é montado no
  lab.rs (drain), que está congelado em 733 linhas e fora da lista, e os
  métodos dele embrulham project e dispatch em Costs e guardam o
  primeiro erro de projeção, então a delegação troca a montagem no
  lab.rs
- o loop do laboratório já drena depois de cada evento do worker, o
  mesmo padrão do shell; ide::agent::serve é a forma de uma linha disso
- o teste agent_port tem um Shell de teste (porta, frame, drains) que o
  laboratório pode copiar para dizer em qual drain cada estágio saiu

Commits (depois do rebase sobre af8302f; a prova na janela rodou em
68af0b1, que é d220b63 antes desse rebase):
- 583d6d1 task-86: the ide shell's pacing and keys leave main.rs, nothing changes
- 42cf31d task-86: engine: a binding says where it shows, and the showcase's port reads it there
- 64f4fd0 task-86: the ide lib serves its workbench through the agent port
- d220b63 task-86: the ide's window opens the agent port and serves it
- 8e383b3 task-86: the test's agent looks again before it recovers
- o commit deste texto: a prova na janela e o handoff
```

## fechamento (orquestrador, 2026-09-23)

integrada na `main` em `7f866e8` (seis commits, rebaseados sobre
`eea529b` sem conflito). validação do orquestrador: diff inteiro lido
commit a commit; a divisão do `main.rs` conferida linha a linha como
movimento puro (as linhas removidas e as adicionadas diferem só em
imports, visibilidade `pub(crate)` e doc de módulo). gate verde no
diretório da task depois do rebase: 2202 passaram, 0 falharam, 12
ignorados, com a perna urnaui.

reprodução do orquestrador numa janela real: o ide subiu com a porta
(`ide-<pid>.sock` no log) e o `plev observe --scope changes --keys
files,staged` respondeu com os dois arquivos e as ações com `reads`.
a contagem de frames parados não deu para repetir: a janela nova tomou o
foco com o brenner na máquina, e o log mostra `show_diff` da pessoa e
movimento de mouse durante a janela de 6 s. fechei o ide pelo pid e
removi o socket que o kill deixou. a contagem de 0 frames fica com a
prova do agente (7,4 s depois da subida e 13,9 s no fim) e com o código:
o shell só invalida quando o `serve` diz que um ato rodou, e o teste da
lib confere que só o drain com ato devolve verdadeiro.

o que saiu daqui:
- a limitação do `acted` que o handoff aponta vira a task-90: na
  sequência mais comum de uma pessoa (clicar na linha, depois tirar do
  index), a seleção é validada na revisão que o agente viu e o unstage
  na seguinte, então a recusa nomeia a seleção e não o unstage.
- o briefing dizia que `AgentPort::open` respeita `PLEV_AGENT=1` e
  debug; quem respeita é `from_env`. o agente seguiu o contrato e não
  o briefing (learnings, processo).
- incidente: na primeira tentativa da prova, um clique sintético caiu no
  navegador do brenner, que estava na frente. o harness passou a
  conferir a janela no ponto antes de clicar (learnings, processo).
