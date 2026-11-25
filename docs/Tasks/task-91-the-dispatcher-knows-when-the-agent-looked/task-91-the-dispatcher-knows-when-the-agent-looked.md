---
id: task-91
title: "task-91: the dispatcher knows when the agent looked"
project: plev
type: task
status: done
date: 2026-09-24
date_basis: header
recorded: 2026-09-23
topics: [semantics, agents]
relations: [adr-0043, adr-0046, adr-0047, operations-code-against-the-plev-engine, task-83, task-89, task-90]
evidence: [4d6eba3, 1f9deb3, cea6d81, ac89f01, 3886402, 63cd763, 29373d4]
source: docs/log/steps/task-91-the-dispatcher-knows-when-the-agent-looked.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-91: the dispatcher knows when the agent looked

Makes the dispatcher know when the agent looked: the port marks each look in the order of the requests, and the `acted` of a stale refusal now names the actions of any actor taken after the look the act relied on, the author's own included. As a result, the window scene, in which the person selects a line (a change that does not go through the dispatcher) and then takes the file out of the index, now names their `show_diff` and `unstage`, which previously came out with an empty `acted`. The task-90 rule remains as a fallback for whoever observes without the port; no verdict changes, because the check still goes through the reads. Integrated on `main` at `29373d4`, with severity P2 and a P1 in the lab report (green gate, 2277 tests passed).

## por que

a recusa stale nomeia quem agiu (`acted`, task-83; ampliado na task-90
para a cadeia que começa numa ação validada na revisão que o autor viu).
as duas regras ancoram no dispatcher, e o dispatcher só vê o que passa
por ele. a sonda da task-90 mostrou a cena mais comum da janela sem
nenhuma âncora: a pessoa clica numa linha nova (a seleção muda
`changes:selected` direto na view, sem receipt, e a revisão muda), depois
o `show_diff` e o `unstage` são validados em revisões seguintes. o
`acted` sai vazio, e o toast não diz quem tirou o arquivo do index.

a seleção não é a única mudança projetada fora do dispatcher no ide: a
digitação na mensagem de commit, o foco, o toast saindo pelo timer e as
linhas do diff chegando do worker também mudam a revisão sem receipt.

## direção

o dispatcher passa a saber do olhar: quem serve uma observação avisa o
dispatcher (ator, revisão, o último id despachado naquele instante), de
forma limitada. o `acted` de uma recusa passa a ser as ações de outros
atores despachadas depois do olhar em que o ato recusado se apoiou, com
a regra da task-90 como reserva quando não há olhar registrado (quem
observa pelo `Request::observe` no mesmo processo). com a task-89, o
servidor já sabe, por ator, de qual observação veio o valor de cada
`read`; a âncora é a mais antiga delas.

superfícies: API nova no `sem` (aditiva), o `wire` (servidor) e os
`Host` (`apps/idestudio/src/capability/agent.rs`, o do showcase, o do laboratório). o
formato das mensagens não muda.

## aceite

a cena da janela pelo clique real (clicar numa linha nova, depois tirar
do index) nomeia o unstage; os testes das tasks 83 e 90 verdes; o ADR
com a regra e os limites que sobram (mudança de fora do app); gate
verde.

## handoff

```text
Goal: task-91, a recusa stale de um ato do agente nomeia, em acted, o
      que rodou depois do olhar em que o valor recusado se apoiou, mesmo
      quando a primeira mudança depois do olhar não passou pelo
      dispatcher (a seleção por clique)
Status: pronto para validação do orquestrador (severidade P2,
        explicabilidade; nenhum veredito muda)

A sonda, antes de implementar (base 44e5be9, testes do commit 1 com
--ignored):
- a cena da janela com o olhar e o ato do agente pelo port (o host do
  próprio ide), a pessoa clica em B e tira B do index pelo menu:
    left: []
   right: [(RequestId(3), ("human", "changes.file[B]", "show_diff")),
           (RequestId(4), ("human", "changes.file[B]", "unstage"))]
  e o teste da task-90 que escreve o limite, no processo, verde com
  acted vazio e o toast "Refused test-agent's submit: B left the staged
  files"
- o agente observa o formulário e depois changes, a pessoa tira B pelo
  dispatcher, o agente reescreve a mensagem e submete na revisão do
  estágio desse ato, com o staged da observação anterior:
    left: []
   right: [(RequestId(3), ("human", "changes.file[B]", "unstage")),
           (RequestId(4), ("agent:test-agent", "commit.message", "set_value"))]
- no toy do wire, a reprodução da task-89 como o orquestrador rodou
  (observa o counter, a pessoa marca o item 1, observa só o botão,
  clear): left: [] (right: o toggle da pessoa). o sync do próprio agente
  depois do olhar: left: [toggle (3)] (right: [sync do claude (2),
  toggle (3)]), a regra da 90 nunca nomeia o autor
  a sonda saiu como o briefing dizia; segui.
- depois: os quatro verdes sem #[ignore] (commit f42ed36)

A regra como ficou:
- com o olhar (o port sabe): um Mark é um ponto na ordem dos requests
  de um dispatcher (o id do último tomado). o acted de uma recusa stale
  são os receipts que rodaram (executing, completed, failed) e foram
  tomados depois do olhar, de qualquer ator, o autor incluído, em ordem
  de id, entre os que o dispatcher retém. o olhar de um ato com
  from_revision é a mais antiga das observações de onde vieram os
  valores das reads; o de um ato com observed explícito é a última vez
  em que o ator recebeu a revisão que ele nomeia
- sem olhar: a regra da task-90 como estava (a cadeia a partir do
  primeiro receipt que rodou validado na revisão que o autor observou,
  outros atores só). vale para quem observa pelo Request::observe no
  processo, para o request de uma pessoa, para um observed explícito
  numa revisão nunca dada ao ator ou esquecida (16 por ator), e para um
  host que descarta o sink
- a marca só nomeia: check continua decidindo pelas reads; um só lugar
  preenche o acted (Dispatcher::push, que pergunta ao sink); a forma
  serde de Rejection::Stale e Acted e o formato das mensagens não mudam;
  a marca não está em request, receipt nem mensagem

Implementado:
- sem: Mark (receipt.rs) com Mark::after(id) e Mark::precedes(id);
  Dispatcher::mark(); ReceiptSink ganha dois métodos com padrão vazio:
  attached(mark), chamado por add_sink com a marca de agora, e
  looked(receipt) -> Option<Mark>. Dispatcher::acted usa o primeiro
  looked que um sink der; sem nenhum, a regra da 90
- wire: o PortSink segue a ordem do dispatcher (Looks, em shown.rs): a
  marca quando entra, e cada Requested que ouve. o Server põe o sink no
  dispatcher na primeira observação servida a um ator ou no primeiro
  ato (antes: só no primeiro ato). cada revisão dada guarda a marca
  (Given.mark); Shown::values devolve os valores e o olhar (Rested);
  Shown::looked dá a marca da última entrega de uma revisão. logo antes
  do dispatch o Server diz ao sink o olhar do ato (ator e marca); o
  sink o arquiva sob o id quando o dispatcher toma o Requested daquele
  ator, e o devolve enquanto o request está aberto (até 256 abertos)
- Traffic::acts_from_revision e acts_observed (aditivo, contados no
  handle pela forma da mensagem; com os dois ou nenhum, só em acts)
- laboratório (story.rs, finish.rs): a forma de cada ato vem da
  contagem do port, não mais da revisão que ele nomeia: "form" de cada
  ato é a de todos (null se misturada), agent.acts_form diz "every act
  on from_revision (--from-revision)", agent.acts_by_form tem as
  contagens, e a linha impressa "acts rested on" repete
- ide, toast (notice.rs, só texto): a série do próprio autor sai na
  terceira pessoa, "test-agent itself edited the commit message"
- docs: doc do Act (proto.rs), de Acted, Rejection::Stale, Dispatcher,
  ReceiptSink, lib.rs do sem, módulos server.rs e shown.rs; os dois ADRs

Arquivos principais:
- rust/accord/src/{receipt.rs,dispatch.rs,check.rs,lib.rs}
- rust/socket/src/{server.rs,shown.rs,proto.rs}
- apps/idestudio/src/capability/notice.rs e tests.rs (só o texto do toast)
- apps/agenteval/src/{story.rs,finish.rs} (só a forma dos atos)
- testes: rust/accord/tests/dispatch.rs, rust/socket/tests/{look.rs
  (novo),server.rs,shown.rs,traffic.rs}, apps/idestudio/tests/{human_agent.rs,
  lab.rs}
- docs/adr/semantics/0043-a-delta-carries-the-node-a-refusal-names-who-acted.md,
  docs/adr/semantics/0047-an-act-rests-on-what-its-actor-was-shown.md, este arquivo

Interfaces adicionadas/alteradas:
- sem (aditivo): Mark; Dispatcher::mark; ReceiptSink::attached e
  ReceiptSink::looked com padrão; add_sink passa a chamar attached
- wire (aditivo): Traffic::acts_from_revision, acts_observed; o doc de
  Host::add_sink diz que ele é chamado na primeira observação servida a
  um ator ou no primeiro ato
- semântica: Rejection::Stale::acted com olhar nomeia o próprio autor
  (muda a decisão da task-90, escrita no ADR com o motivo)

Testes:
- sem (dispatch.rs, +6): a janela em memória com e sem olhar; o autor
  depois do olhar e nada antes; a_look_never_decides (os mesmos requests
  com e sem olhar, e com um olhar de outro ator, acabam nos mesmos
  estágios com o mesmo delta, só o acted muda); na confirmação e no
  ponto de verdade; retenção; a marca contada ao sink
- wire (look.rs novo, 5; traffic.rs, +1): a reprodução da 89; o sync do
  próprio agente enquanto roda; o observed explícito na última entrega
  da revisão e numa revisão nunca dada (volta para a 90); o que rodou
  antes do primeiro olhar não é nomeado; um olhar esquecido além do cap
  volta para a 90; as contagens por forma
- ide: human_agent.rs +2 pelo port (a cena da janela, que nomeia
  show_diff e unstage e o toast "Refused test-agent's submit: B left
  the staged files (the person opened the diff of B and unstaged B)";
  e a revisão de estágio, com a pessoa abrindo o diff de A entre os dois
  olhares do agente, nomeado porque a âncora é o olhar mais antigo:
  "(the person opened the diff of A and unstaged B; test-agent itself
  edited the commit message)"); workbench/tests.rs +1 (o toast do autor);
  story.rs +1 (a forma pela contagem)
- testes que mudaram, e por quê:
  - task-90: a_click_on_a_new_row_moves_the_state_outside_the_dispatcher_
    and_nobody_is_named passa a provar o contrário pelo port
    (through_the_port::a_click_on_a_new_row_and_an_unstage_are_named_
    since_the_agents_look); a forma no processo fica como a prova da
    reserva sem olhar, renomeada without_a_look_a_click_on_a_new_row_
    leaves_the_chain_nowhere_to_start. a cena virou um helper
    (click_a_new_row_then_unstage). os quatro testes da 90 no sem e o
    da cadeia do ide não mudaram
  - task-89 (wire): a_row_the_agents_own_act_moved_is_stale_until_it_
    observes_it nomeia agora o toggle do próprio agente;
    an_empty_delta_since_a_stage_revision_shows_nothing nomeia o toggle
    da pessoa e a escrita do agente (era o limite da 89)
  - task-83: nenhum mudou; agent_port.rs (task-86) não mudou
  - lab.rs: names_the_person_or_nobody virou names_the_person (exato);
    os três testes encadeados nomeiam a abertura do formulário e a
    escrita do próprio agente (antes vazio); commits_the_task confere a
    forma pela contagem; o comentário sobre form foi corrigido

Validação executada:
- ./tool/tasks/fullcheck.sh verde, com a perna urnaui: 2256 passed, 0 failed, 12
  ignored (90 "test result"). antes, na base 44e5be9 medida neste
  worktree: workspace 2158 passed, 11 ignored (83), mais a perna urnaui
  82 passed, 1 ignored (6), total 2240 e 12. +16 testes, nenhum
  removido, nenhum #[ignore] novo. df -h /: 17 GiB livres
- cargo test -p accord -p socket -p idestudio -p record -p replay -p uigallery:
  verdes (622 passed, 6 ignored, 45 binários)
- apps/agenteval/tests/experiment.rs dez vezes seguidas: 6 passed nas dez
- ide-lab --arm scripted --human unstage-before-submit:src/session.rs:
  --scripted observe: exit 0, MATCH, stale_measurement_valid true,
    acts_form "every act on from_revision (--from-revision)" (8 de 8),
    stale request 7 em changes:staged,
    acted: human unstage changes.file[src/session.rs] (request 6)
  --scripted chain: exit 0, MATCH, stale_measurement_valid true, a
    mesma forma (8 de 8), stale request 7 em changes:staged e
    commit.message:text,
    acted: agent:scripted open commit.form (4), agent:scripted
    set_value commit.message (5), human unstage
    changes.file[src/session.rs] (6)

Decisões:
- o caminho da marca até o dispatcher é o sink, não métodos novos no
  Host. contexto: a direção sugeria Host::mark e um despacho com a
  marca; o Workbench do ide guarda o dispatcher privado e
  apps/idestudio/src/capability/** está fora da lista, então nem o host do
  ide nem o do laboratório leem a marca ou despacham com ela sem mudar o
  Workbench. o único canal do port até o dispatcher que todo host já
  tem é Host::add_sink. razão: dentro da lista, sem mudar o formato, e
  sem segundo caminho. consequência: nenhum host mudou, e o showcase
  tem o olhar também (o add_sink dele vai ao dispatcher da página do
  todo). o custo é o ReceiptSink ter dois papéis (ouvir estágios e
  dizer olhares); está escrito no doc do trait
- o autor é nomeado com olhar (a mudança da decisão da 90, com o motivo
  no ADR: o agente recusado pela própria escrita, terceira rodada da
  fase c). sem olhar, continua fora
- observed explícito: o olhar é a última entrega da revisão que ele
  nomeia àquele ator; uma revisão nunca dada, ou esquecida, não tem
  olhar e cai na regra da 90
- humano: sem olhar (nada marca o frame em que a pessoa clicou), a regra
  da 90
- um olhar que saiu do anel: um ato com from_revision cujo read veio
  dele já é not_observed antes do dispatcher (task-89), então nunca há
  acted; o olhar de um observed explícito vale enquanto o Shown lembra
  a entrega (16 por ator), depois cai na regra da 90
- o olhar mais antigo entre as reads, como o briefing pede: nomeia a
  mais (o que o agente viu de novo num olhar posterior também entra)

Limitações:
- um request tomado antes do olhar cujo efeito chega depois (no ide, um
  stage da pessoa que o worker termina depois do olhar) não é nomeado
- o arquivamento do olhar casa pelo ator: se o host despachasse outro
  request do mesmo agente dentro do Host::dispatch antes do ato, o olhar
  iria para ele (nenhum host faz isso)
- até 256 requests abertos guardam o olhar; um além disso cai na regra
  da 90
- a forma de cada ato no relatório do laboratório só é exata quando
  todos têm a mesma (senão null); a contagem é exata sempre
- mudança de fora do app continua sem nome; ela só não esconde mais
  quem veio depois

Dependências pendentes:
- nenhuma para integrar

Riscos de integração (textos fora da lista que descrevem a regra da 90;
proponho, o orquestrador aplica):
- rust/socket/src/bin/plev.rs:73-75 (USAGE):
    what moved, and its acted says who, among the app's actors, ran
    something since the agent looked: every request the app took after
    the oldest observation the act rests on, the agent's own included (a
    change no request made, as from outside the app, names nobody).
  (o teste the_help_says_what_makes_an_act_stale procura "its acted says
  who", que fica)
- docs/ref/code-against-the-plev-engine.md:64-67: "`acted`: what ran
  since the refused actor looked: with the look the agent port knows
  (task-91), every request taken after it, the actor's own included;
  without one, the actions of other actors from the first request that
  ran on the state it saw (task-83, task-90; a change no request made
  names nobody)"
- docs/adr/recording/0046-the-inspector-reads-a-session-never-runs-it.md:25, uma linha
  datada: "updated on 2026-09-24 (task-91): for an act of the port,
  acted names every request taken after the look it rested on, the
  agent's own included; the recording does not keep the look yet."
- rust/replay/src/prose/chain.rs:292-300, o texto de um acted vazio:
    "(acted names the requests taken after the look the act rested on,
    the author's own included, when the port knew the look; without
    one, other actors' receipts from the first one, of any actor, that
    ran on the exact revision the author saw. nobody ran since the look,
    or, without one, none ran on that revision. a change no receipt made
    names nobody, and a refusal recorded before acted existed names
    nobody. before says what the recording shows)"
  e, numa entrada não vazia cujo ator é o autor, "(its own)" depois do
  nome

O que o rec, o inspector e o showcase precisam depois:
- rec: o gravador já grava cada observação (ator, escopo, revisão) e
  cada estágio, em ordem, então a marca de cada observação se reconstrói
  (o último request gravado antes dela). o que ele não sabe é em qual
  olhar um ato se apoiou (o Shown do port). proposta: um evento aditivo
  por ato do port com o olhar (a marca, e a revisão da observação de
  onde ela veio), passado por um método novo com padrão no Host (ex.:
  fn rested(&mut self, request: RequestId, look: Mark, from: Revision)),
  chamado pelo Server depois do dispatch; formato da sessão continua na
  versão 0 com o campo opcional
- inspector: com esse evento, explicar o acted ("nomeados: o que rodou
  depois do olhar na marca m, a observação de t em r") e distinguir
  "ninguém rodou depois do olhar" de "sem olhar: regra da 90"; até lá,
  o texto proposto acima
- showcase: não perde nada; o Host dele adiciona o sink do port ao
  dispatcher da página do todo (dispatch.rs:367 -> view.add_sink ->
  panes.todo.add_sink), então os atos dele têm o olhar. falta só um
  teste disso nos testes do agent port do showcase, que estão no
  dispatch.rs congelado; proponho para quando ele for dividido

ADR proposto:
- nenhum novo; a-delta-carries-the-node-a-refusal-names-who-acted
  reescrito (a regra com e sem olhar, a mudança do autor com o motivo,
  o toast, os limites, linha datada 2026-09-24) e uma linha datada em
  an-act-rests-on-what-its-actor-was-shown. linha do índice
  (docs/adr/README.md), acrescentada no fim da entrada existente:
  "; atualizado pela task-91: com o olhar que o agent port conhece (a
  observação mais antiga de onde vêm as reads do ato, marcada na ordem
  do dispatcher, sem.Mark), o acted nomeia todo request tomado depois
  dele, o do próprio autor incluído; sem olhar, a regra da task-90"

Commits:
- 31277a9 task-91: the probe, as tests that fail on the rule of task-90
- b546d82 task-91: the dispatcher names what ran since a look a sink knows
- f42ed36 task-91: the port marks each look and tells the dispatcher which one an act rests on
- ffbc755 task-91: the toast names the agent's own acts since its look, in the third person
- d4ebf5b task-91: the laboratory says what every act rested on from the port's count
- 473133d task-91: tests of the look, the fallback and what the look never does
- o commit deste texto: os ADRs e o handoff
```

## fechamento (orquestrador, 2026-09-24)

integrada na `main` em `29373d4` (sete commits, rebaseados sobre
`804e57d` sem conflito). severidade: P2, com um P1 dentro (a forma dos
atos no relatório do laboratório).

validação do orquestrador:
- diff lido: `Mark` no `sem` e o `ReceiptSink` com `attached` e `looked`
  (padrões vazios); `Looks` no `wire`, que segue a ordem do dispatcher
  pelo sink do port e guarda o olhar de cada ato; `acted` com e sem
  marca; `Traffic.acts_from_revision` e `acts_observed`; o toast na
  terceira pessoa para o próprio agente;
- gate verde no diretório da task: 2277 passaram, 0 falharam, 12
  ignorados, com a perna urnaui;
- reprodução no laboratório (`--arm plev`, a pessoa depois de uma
  observação), com o `plev` da branch. o agente observa `changes` (três
  arquivos), a pessoa tira um, o agente olha só o formulário e submete.
  recusa stale com `acted` = `human unstage changes.file[src/session.rs]`.
  na regra da task-90 essa cena saía com `acted` vazio.

decisão do agente aceita: a marca chega ao dispatcher pelo sink do port,
não por métodos novos no `Host`. nenhum host mudou, e o showcase também
ganhou o olhar. o custo é o `ReceiptSink` com dois papéis, escrito no
doc do trait.

aplicado no fechamento: os textos da regra no `plev --help`, no how-to
do engine, no ADR do inspector e no texto do inspect para um `acted`
vazio. o "(its own)" do inspect ficou para depois (P3).

o que fica (P2): o gravador não sabe em qual olhar cada ato se apoiou
(proposta no handoff, um evento aditivo por ato do port).

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0043: a delta carries the node it adds, and a stale refusal names who acted](../../adr/Semantics/0043-a-delta-carries-the-node-a-refusal-names-who-acted/0043-a-delta-carries-the-node-a-refusal-names-who-acted.md)
- [ADR 0046: the inspector reads a session, it never runs it](../../adr/Recording/0046-the-inspector-reads-a-session-never-runs-it/0046-the-inspector-reads-a-session-never-runs-it.md)
- [ADR 0047: an act rests on what its actor was shown](../../adr/Semantics/0047-an-act-rests-on-what-its-actor-was-shown/0047-an-act-rests-on-what-its-actor-was-shown.md)
- [ADR 0067: adaptive transformations are small verified operations that preserve identity](../../adr/Synthesis/0067-adaptive-transforms-preserve-identity/0067-adaptive-transforms-preserve-identity.md)
- [what the autonomous cycle taught: a delta carries the node it adds, and a stale refusal names who acted](../../adr/Semantics/0043-a-delta-carries-the-node-a-refusal-names-who-acted/learned-a-delta-carries-the-node-a-refusal-names-who-acted.md)
- [what the autonomous cycle taught: an act rests on what its actor was shown](../../adr/Semantics/0047-an-act-rests-on-what-its-actor-was-shown/learned-an-act-rests-on-what-its-actor-was-shown.md)
- [what the autonomous cycle taught: integration into main is a pull request merged by merge commit](../../adr/Workflows/0085-integration-by-merge-commit/learned-integration-by-merge-commit.md)
- [task-83: the delta carries the node that was just added, and the refusal names who acted](../task-83-delta-carries-node-refusal-names-actor/task-83-delta-carries-node-refusal-names-actor.md)
- [task-89: an act relies only on what was shown to the agent](../task-89-an-act-rests-only-on-what-was-shown/task-89-an-act-rests-only-on-what-was-shown.md)
- [task-90: a refusal names who acted since the agent looked](../task-90-a-refusal-names-who-acted-since-the-look/task-90-a-refusal-names-who-acted-since-the-look.md)
- [task-92: the agent asks the person](../task-92-the-agent-asks-the-person/task-92-the-agent-asks-the-person.md)
- [task-62 phase c, third round: the task-89 rule (2026-09-24)](../task-62-ide-human-agent-proof/phase-c-round-3.md)
- [task-68 implementation phase: probe, handoff and close-out](../task-68-plev-ir-and-verifier/phase-implementation.md)
- [program: from GPU UI engine to generative interaction runtime](../../Research/Synthesis/generative-runtime-program.md)
- [how to code against the plev engine](../../Research/Workflows/code-against-the-plev-engine.md)
- [autonomous cycle journal](../../Research/Workflows/autonomous-cycle-journal.md)
<!-- /related -->
