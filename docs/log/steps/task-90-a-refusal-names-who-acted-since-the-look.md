---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-23
domain: semantic-contract
task: task-90
depends: [task-83, task-86]
origin: a prova da task-86 numa janela real (limitação lida no código e escrita no handoff)
---

# task-90: a recusa nomeia quem agiu desde que o agente olhou

## por que

a task-83 fez a recusa stale dizer quem agiu (`Rejection::Stale.acted`),
com uma regra exata e limitada: as ações de outros atores validadas
contra a mesma revisão que o autor viu e que chegaram a rodar
(`rust/accord/src/dispatch.rs`, `acted`). o ADR
a-delta-carries-the-node-a-refusal-names-who-acted escreve o limite: de
uma cadeia de ações de outra pessoa, só a primeira é nomeada.

a prova da task-86 na janela real mostrou que esse limite cai na
sequência mais comum de uma pessoa no ide. o agente olha (revisão R); a
pessoa clica na linha de um arquivo (seleção e `show_diff`, validados
contra R, a revisão muda para R'); a pessoa tira o arquivo do index
(validado contra R'). o `submit` do agente é recusado com o delta certo,
mas o `acted` nomeia a seleção e não o unstage, e o toast do ide repete
isso para a pessoa. a prova só nomeou o unstage porque a pessoa clicou
na linha antes de o agente decidir.

é a garantia da task-83 ("a recusa diz quem agiu") falhando no uso
normal: bug de validação real, visto numa janela, com a pessoa e o
agente ao mesmo tempo.

## direção (orquestrador; confirme no código)

o dispatcher numera os requests em ordem (`RequestId`, crescente). uma
ação de outro ator validada contra R rodou sobre o estado que o autor
viu, então rodou depois do olhar do autor; toda ação de outro ator que
rodou depois dela também. regra candidata: `acted` = as ações de outros
atores que rodaram, com id maior ou igual ao da primeira que rodou
validada contra a revisão do autor. sem nenhuma validada contra R (a
mudança veio de fora do app, um terminal), o `acted` fica como hoje. a
revisão continua não decidindo nada: `check` compara os `reads`, o
`acted` só nomeia.

o que ela não resolve, e continua escrito no ADR: mudança de fora do app
não tem receipt; um estado que volta exatamente a R pode nomear a mais;
nomear quem agiu não diz que a ação moveu o que o ato lê (o `changes`
diz).

## aceite

teste no `sem` e no ide (`apps/idestudio/tests/human_agent.rs`) da sequência
da janela (o agente olha, a pessoa seleciona, a pessoa tira do index, o
`submit` é recusado): o `acted` nomeia a seleção e o unstage, em ordem
de id; o texto do toast legível com mais de uma ação (diga o texto antes
e depois); os testes da task-83 verdes; o ADR com os limites
reescritos; gate verde.

## a sonda e a decisão do orquestrador (2026-09-23)

a direção do briefing dizia que, na cena da janela, a seleção e o
`show_diff` eram validados na revisão que o agente viu. no código não
são: o clique muda a seleção direto na view
(`apps/idestudio/src/interfaces/panels/layout/input.rs:78-82`, `unassigned.select`
antes de `open_file_diff` enfileirar o pedido), a seleção é projetada
(`changes:selected`, `apps/idestudio/src/capability/project.rs:74`), e
`Workbench::act` (`apps/idestudio/src/capability/mod.rs:261-270`) só observa
e despacha o `show_diff` depois. uma sonda pelo clique real, sem commit,
deu:

```text
agent looks at ad22cdf185a0e0d2
click on B: before ad22cdf185a0e0d2, after 3a5df0f36ca80766
request 3: human changes.file[B] show_diff validated_against=62b7cb9ba436afe6
request 4: human changes.file[B] unstage   validated_against=3a5df0f36ca80766
refusal: Stale { changes: [staged {A,B} -> {A}], acted: [] }
notices: ["Refused test-agent's submit: B left the staged files"]
```

o `acted` saía vazio (não com o `show_diff`), e a regra por id sozinha
não muda isso: nada rodou na revisão que o agente viu. parei e reportei.
o orquestrador decidiu a opção A: a regra por id no dispatcher, como a
regra dentro do processo e reserva da âncora no olhar (task-91); o aceite
do ide reescrito com duas cenas pelo ponteiro, uma cadeia que começa numa
ação validada em R e a cena da janela como limite conhecido; liberou
`rust/accord/src/decision.rs` (doc do `Acted`), `rust/accord/src/lib.rs` (doc)
e o texto da regra em `rust/replay/src/prose/chain.rs`.

## handoff

```text
Goal: task-90, a recusa stale nomeia, em acted, as ações de outros atores
      que rodaram desde o estado que o autor viu (a cadeia dentro do
      processo), não só as validadas exatamente nessa revisão
Status: pronto para validação do orquestrador

Implementado:
- sem, Dispatcher::acted (o único lugar que preenche, chamado de push):
  entre os receipts retidos (open e finished), o ponto de partida é o
  menor id de um receipt que rodou (executing, completed, failed) e cuja
  validated_against é request.observed.revision, de qualquer ator, o
  autor incluído; acted são os receipts que rodaram, com id a partir
  dele, de ator diferente do autor, em ordem de id. sem ponto de
  partida, vazio, como antes
- decisões da regra: uma ação do próprio autor pode ser o ponto de
  partida (sem isso, "o agente faz stage em R, a pessoa tira do index em
  R', o submit sai sobre R" não nomearia ninguém); o autor nunca é
  nomeado e não corta a cadeia; um receipt que não rodou (recusa,
  decline, espera) não é ponto de partida, como no briefing: ele só
  prova que o estado estava em R, e o único caso em que ajudaria passa
  por uma mudança sem receipt, que a regra deixa para o changes
- docs: Acted e Rejection::Stale (check.rs), o doc do Dispatcher e o de
  lib.rs dizem "ran since the state the actor saw"; o doc do Acted diz a
  regra, que ela é a regra dentro do processo e a reserva da âncora no
  olhar, e os limites
- ide, toast: uma oração por ator, na ordem em que rodou, as ações do
  mesmo ator em série ("a, b and c"), outro ator depois de ponto e
  vírgula; show_diff vira "opened the diff of <arquivo>" e dismiss
  "dismissed a notice". com uma entrada, o texto é o de antes
- inspect: o texto que explica um acted vazio diz a regra nova
- ADR a-delta-carries-the-node-a-refusal-names-who-acted: a regra, o
  toast, os limites reescritos e a linha datada de origem

Arquivos principais:
- rust/accord/src/dispatch.rs (acted e docs), rust/accord/src/decision.rs
  (só doc), rust/accord/src/lib.rs (só doc), rust/accord/tests/dispatch.rs
- apps/idestudio/src/capability/notice.rs, apps/idestudio/src/capability/tests.rs,
  apps/idestudio/tests/human_agent.rs
- rust/replay/src/prose/chain.rs (só o texto)
- docs/adr/semantics/0043-a-delta-carries-the-node-a-refusal-names-who-acted.md, este
  arquivo (movido para ongoing no primeiro commit)

Interfaces adicionadas/alteradas:
- nenhuma assinatura. a semântica de Rejection::Stale::acted muda (nomeia
  a cadeia desde o ponto de partida); a forma serde de Acted e de
  Rejection::Stale, check por reads, a redação e a retenção não mudam;
  continua um só lugar que preenche (Dispatcher::push)

Testes:
- sem, rust/accord/tests/dispatch.rs, 4 novos:
  a_stale_refusal_names_the_chain_that_ran_since_the_state_the_agent_saw
  (a pessoa move o cursor em R, o agente move o dele, a pessoa tira B em
  R'; nomeia move e unstage da pessoa, em ordem de id, e não o agente),
  the_authors_own_act_on_the_state_it_saw_can_start_the_chain,
  a_change_no_request_made_leaves_the_chain_nowhere_to_start (a mudança
  sem receipt vem primeiro, e uma recusa da pessoa em R não é ponto de
  partida: acted vazio, já passava na base e tem que continuar),
  a_chain_is_read_from_the_receipts_the_dispatcher_holds (retenção 0, 1
  e 2 nomeiam 0, 0 e 2: o ponto de partida descartado leva a cadeia)
- antes (a regra da task-83, commit 168c897 com --ignored):
    left: [Acted { id: RequestId(1), .. action: "move" }]
    right: [move (1), unstage (3)]
    the_authors_own_act...: left: [], right: [unstage (2)]
    a_chain_is_read...: left: 1, right: 2
  depois: os quatro verdes
- task-83, muda de propósito:
  a_stale_refusal_names_who_ran_something_on_the_state_the_agent_saw.
  o segundo unstage da pessoa (C, validado no estado que o primeiro
  deixou) agora é nomeado: para o commit do agente, [unstage B, unstage
  C]; para o discard do cleaner na confirmação, [stage do agente,
  unstage B, unstage C]. é o limite que a task remove. os outros dois
  testes da 83 (point of truth, retenção) não mudaram
- ide, apps/idestudio/tests/human_agent.rs, 2 novos, pelo ponteiro (clique
  esquerdo na linha, clique direito, clique no primeiro item do menu; o
  menu é posto pelo mesmo widget do comps, meio xs à direita do ponteiro):
  a_chain_the_person_starts_on_what_the_agent_saw_is_named_whole (B
  selecionado antes da decisão; unstage B pelo menu dessa linha, validado
  em R; clique direito em C, que pede o show_diff de C, e unstage C;
  acted = [unstage B, show_diff C, unstage C] em ordem de id) e
  a_click_on_a_new_row_moves_the_state_outside_the_dispatcher_and_nobody_is_named
  (a cena da janela: acted vazio, limite conhecido escrito no teste; muda
  na task-91). na regra da 83 o primeiro nomeava só o unstage de B
  (conferido trocando o dispatch.rs pelo da base, depois restaurado)
- ide, apps/idestudio/src/capability/tests.rs, 1 novo:
  a_stale_refusal_toast_names_each_actors_run_in_order (dois atores)
- o aceite pedia "acted nomeia os dois"; pelo ponteiro, o segundo
  arquivo só sai do index depois de a linha dele ser selecionada, o que
  pede o show_diff dele, e esse show_diff rodou depois do olhar: são
  três entradas, todas da pessoa

O texto do toast (a cadeia do teste do ide):
- antes (regra da 83): "Refused test-agent's submit: B left the staged
  files, C left the staged files (the person unstaged B)"
- com a regra nova e o texto antigo: "... (the person unstaged B, the
  person ran show_diff on C, the person unstaged C)"
- depois: "Refused test-agent's submit: B left the staged files, C left
  the staged files (the person unstaged B, opened the diff of C and
  unstaged C)", o mesmo label do nó notices[..] que o agente lê
- dois atores: "(the person edited the commit message and opened the
  diff of b.txt; reviewer opened the diff of a.txt)"
- a cena da janela, antes e depois: "Refused test-agent's submit: B left
  the staged files"

Validação executada:
- ./tool/tasks/fullcheck.sh verde depois do rebase na main (e45a879), com a perna
  urnaui: 2225 passed, 0 failed, 12 ignored (88 "test result"). depois a
  main foi para 62026fb, um merge com a mesma árvore de e45a879; a branch
  está em cima dela, e a árvore do código é a do gate. a main
  tinha 2218 (fechamento da task-89); +7 desta task (sem 4, ide 3),
  nenhum removido, nenhum ignore novo. df -h /: 19 GiB livres
- cargo test -p wire -p rec -p inspect -p ide depois da regra e antes do
  toast: verdes sem mudar nenhum teste (wire server 24, inspect model 33,
  rec session 36, ide lab 2, agent_port 2, human_agent 20); o
  laboratório nomeia como antes
- sem por binário: dispatch 27 -> 31. ide: lib 54 -> 55, human_agent
  20 -> 22

Decisões:
- as da regra, acima
- o toast fala as ações do mesmo ator em série e separa atores; não
  filtra por nome de ação. proposta (não feita): o toast nomear só as
  ações que moveram o que o changes lista. o ide não sabe isso sem
  adivinhar: o contrato declara o que uma ação lê (reads), nunca o que
  ela escreve, e o receipt não guarda o que mudou. precisaria de um
  writes declarado ou da revisão depois de cada execução no receipt
- o teste do menu reproduz a posição do menu com o widget do comps
  (ContextMenu::entry_rects), porque os retângulos da view são
  pub(crate); se a view mudar onde abre o menu, o teste falha dizendo
  que o pedido não foi o unstage

Limitações:
- mudança projetada sem receipt não é ponto de partida: de fora do app,
  e no ide a seleção por clique ou teclado, a digitação, o foco, o toast
  saindo pelo timer, as linhas do diff chegando. a cena da janela
  (clicar numa linha nova, depois tirar do index) continua sem nomear
  ninguém. é a task-91
- um estado que volta exatamente à revisão do autor pode começar a
  cadeia antes do olhar
- um ponto de partida descartado pela retenção leva a cadeia junto
- com a task-89, um read que vem de uma observação anterior à revisão
  do ato: o que rodou entre aquela observação e a revisão não é nomeado
  (a âncora é Observation::revision); a task-91 ancora no olhar mais
  antigo

Dependências pendentes:
- nenhuma para integrar

Riscos de integração:
- textos fora da lista que ainda descrevem a regra da 83 (proponho o
  texto; o orquestrador aplica na integração):
  - rust/socket/src/protocol.rs:108-111 (doc do Act, na main e45a879):
      /// (`rejected`, `stale`) carries `changes`, what moved, and `acted`, who
      /// among the app's actors ran something since that state, from the first
      /// request that ran on it (the dispatcher knows the requests it took, so
      /// a change no request made, a `git` in a terminal, starts nothing and
      /// names nobody).
  - rust/socket/src/bin/plev.rs:73-74 (USAGE, na main e45a879):
      what moved, and its acted says who, among the app's actors, ran something
      since that state, from the first request that ran on it (a change no
      request made, as from outside the app, starts nothing and names nobody).
    (o teste the_help_says_what_makes_an_act_stale procura "its acted says
    who", que fica)
  - docs/ref/code-against-the-plev-engine.md:64-66: "`acted`: the
    actions of other actors that ran since the state the refused actor
    saw, from the first request that ran on it (task-83, task-90; a
    change no request made names nobody)"
  - docs/adr/recording/0046-the-inspector-reads-a-session-never-runs-it.md:19-22, uma
    linha datada embaixo: "updated on 2026-09-23 (task-90): acted names
    the chain from the first receipt that ran on the revision the author
    saw, so an act checked on a later revision is named when a chain
    started there; a change no receipt made still names nobody."
- a task-91 inverte o segundo teste do ide (a cena da janela passa a
  nomear) e usa esta regra como reserva

ADR proposto:
- nenhum novo; a-delta-carries-the-node-a-refusal-names-who-acted
  atualizado. linha do índice (docs/adr/README.md), se o orquestrador
  quiser marcar a mudança: "... a stale refusal names who acted: the
  other actors' requests that ran since the state the author saw, from
  the first request that ran on it (task-90; a change no request made
  starts nothing)"

Commits:
- 168c897 task-90: the chain since the look, as tests that fail on the rule of task-83
- 8b1ffa6 task-90: a stale refusal names who ran since the state its author saw
- 128ca11 task-90: the ide's two scenes through the pointer, and a toast for a chain
- f0de28a task-90: the inspector says what an empty acted means under the new rule
- o commit deste texto: o ADR e o handoff
```

## fechamento (orquestrador, 2026-09-23)

integrada na `main` em `d02b4d1` (cinco commits sobre `62026fb`).

o agente parou uma vez, antes de implementar: a premissa do briefing
estava errada. a seleção por clique muda a revisão fora do dispatcher, e
na cena da janela o `acted` saía vazio, não com a seleção. decisão do
orquestrador: opção A agora (a cadeia dentro do processo); a âncora no
olhar vira a task-91.

validação do orquestrador:
- diff inteiro lido;
- gate verde no diretório da task: 2225 passaram, 0 falharam, 12
  ignorados, com a perna urnaui;
- a regra antiga do `acted` recolocada no `dispatch.rs`: os quatro testes
  novos do `sem` falham com ela e passam com a nova;
- os dois testes do ide pelo ponteiro verdes.

os textos que o agente propôs para arquivos fora do escopo dele (o doc
do `Act`, o `plev --help`, o how-to e uma linha datada no ADR do
inspector) foram aplicados no fechamento.
