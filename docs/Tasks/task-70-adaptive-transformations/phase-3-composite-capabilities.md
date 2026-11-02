---
id: task-70/phase-3-composite-capabilities
title: "task-70: phase 3, composite capabilities"
project: plev
type: section
parent: task-70
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [agents, recording, semantics]
relations: [task-70/phase-1-operations, task-70/phase-2-proposer, adr-0067, task-58, task-63]
evidence: [43cf8ff5, a3ecffd4, 6ccfbad6, d1106ad8, 5fa0874c, 4d96973a, f00af5ea, f74ce129]
source: docs/log/steps/task-70-adaptive-transformations.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 3: composite capabilities

Records phase 3 of task-70: a detector reads the stages of the receipts, live and recorded, and finds the sequence of actions that the same actor repeats N times in the same order; only the person turns the sequence into an app action. The composite runs as chained requests, each step checked against the state left by the previous one, and the first step that is refused halts the chain, with the reason in the receipt. In Silences the result is the "preview segment, then apply cut" button on each cut; there is no rollback, because no capability of the app offers undo.

branch `task/70c-composites`, sobre a `main` em `a0330678`. a fase 3 é
isto: um detector lê os receipts (ao vivo e de uma sessão gravada) e acha
a mesma sequência de ações, do mesmo ator, na mesma ordem, repetida N
vezes; a pessoa, e só ela, aceita a proposta; a composta passa a ser uma
ação do app que roda os passos como requests encadeados, cada um conferido
contra o estado que o anterior deixou, e o primeiro recusado para a
sequência com o receipt dizendo qual passo e por quê.

## sonda (antes do primeiro tipo, 2026-09-24)

### (a) os receipts, ao vivo e gravados

o `Receipt` guarda o id, o `Request` inteiro, a revisão contra a qual foi
validado e cada estágio com o tempo (`rust/accord/src/receipts.rs:176` a
`181`); o `Request` guarda o ator, o alvo, a ação, o parâmetro, o que o
ator viu e a pergunta de um agente (`rust/accord/src/requests.rs:75` a
`90`); o estágio final é o último (`receipt.rs:116` a `125`, `:209`).
ao vivo, todo estágio passa por `ReceiptSink::on_stage` na ordem em que
acontece (`rust/accord/src/dispatch.rs:54` a `73`, `:338` a `348`), e o
showcase tem um dispatcher só, o do todo, que empresta para as outras
páginas (`apps/uigallery/src/views/apps/todo/semantics/mod.rs`,
`with_dispatcher`; `view/dispatch.rs:420` a `426`, `add_sink`).

o `.monster` da 63 guarda um evento `Kind::Stage` por estágio, na mesma
ordem: a origem (um por dispatcher), o id, o request e o estágio
(`rust/record/src/event.rs:50` a `56`), redigidos contra o estado gravado
naquele instante (`rust/record/src/ring.rs:235` a `260`: o parâmetro e os
valores de um alvo secreto viram o tamanho, `rust/accord/src/redactor.rs:84`
a `96`). `Session::receipts` remonta os receipts desses eventos
(`rust/record/src/session.rs:236` a `271`) e o inspetor lê por aí
(`rust/replay/src/index.rs:72` a `104`).

sem duas fontes da verdade: o detector come a mesma coisa nos dois
casos, o fluxo de estágios `(id, request, estágio)` de um dispatcher, na
ordem. ao vivo quem entrega é um `ReceiptSink` (um leitor que divide o
detector com a página); gravado, os eventos `Kind::Stage` de uma origem,
na ordem do arquivo. a redação do gravador não muda nada do que o
detector olha (ator, alvo, ação, se houve parâmetro: um parâmetro
redigido continua sendo um parâmetro). a entrada é o fluxo, e não
`Session::receipts`, porque é pela ordem dos estágios que se sabe que um
request foi feito enquanto uma composta rodava (os passos dela, abaixo).

cada request entra no histórico do detector quando termina, na ordem do
id (a ordem em que o dispatcher os tomou, `receipt.rs:22` a `29`): um
request aberto segura os de depois até terminar. assim uma prévia que
decodifica o frame (`Execution::Started`) e um aplicar que termina antes
dela entram na ordem em que a pessoa pediu, ao vivo e gravado.

### (b) quando dois passos são o mesmo

um passo é o mesmo que outro quando tem a mesma ação e o mesmo alvo, ou
quando os dois alvos são itens da mesma coleção: o id termina num
segmento de chave e o resto é o mesmo (`silences.cut[3]` e
`silences.cut[4]` são itens de `silences.cut`; `SemId::parent_path`,
`rust/accord/src/identity.rs:74` a `99`). numa repetição, todos os passos por
item são do mesmo item (prévia do corte 3 e aplicar do corte 3); de uma
repetição para a outra o item muda. um passo cujo alvo é igual em todas
as repetições fica exato. a sequência inteira é do mesmo ator
(`Actor` igual, `request.rs:23` a `28`), contígua no histórico: uma ação
de outro ator no meio corta. tem dois passos ou mais, não é a repetição
de uma mais curta, e a mais curta que se repete N vezes no fim do
histórico é a que conta.

só entra um passo que terminou `completed`, sem parâmetro, que não
esperou confirmação e que não é uma composta; qualquer outro corta o
histórico ali (um passo recusado, um que falhou, um que esperou a
pessoa).

o parâmetro não entra: um passo com parâmetro não é passo de composta.
"livre" não cabe porque a composta vira um botão, e o `activate` de um
botão não manda parâmetro (o `promote` recusa uma ação com parâmetro,
`rust/plevir/src/transform/view.rs:164` a `169`). "o mesmo valor" faria o
detector guardar o que a pessoa digitou para repetir depois, um canal
novo, e a redação do `sem` só reduz o parâmetro de um alvo secreto
(`redact.rs:84` a `89`): o texto de um campo comum ficaria guardado como
foi digitado. sem parâmetro o detector não guarda valor nenhum, secreto
ou não: guarda o ator, o alvo e o nome da ação.

N: a Silences pede 2. a semente tem dois cortes, e uma oferta é só uma
oferta: a pessoa recusa e ela não volta na sessão. o detector recebe N de
quem o cria.

### (c) onde a composta mora

é uma ação do app, declarada na projeção dele, num nó dele, como qualquer
outra: a composta por item em cada nó que tem todas as ações dos passos
(cada `silences.cut[<id>]` ganha `preview_segment_then_apply_cut`,
"preview segment, then apply cut"); a exata, no nó do primeiro passo.
lê a união do que os passos leem, está ligada quando todos os passos
estão, e a política é a mais estreita: os atores que todos os passos
deixam, a confirmação mais forte, destrutiva se algum passo for. como
está na projeção, está no registro (`rust/plevir/src/capabilities.rs:72` a `93`):
o `promote` a acha e o verifier liga. nenhum `Dispatcher` novo, nenhuma
tabela de compostas no `sem`: o request da composta passa pelo
`dispatch` como todo request, o `execute` responde `Started`, e os passos
passam pelo mesmo dispatcher no mesmo empréstimo, como o cancelar já faz
(`apps/uigallery/src/views/apps/silences/generative/page.rs:67` a `79`);
a composta termina por `complete` ou `fail` (`dispatch.rs:242` a `257`).

o mínimo aditivo no `sem`: o `SnapshotBuilder` não deixa ler o que já foi
declarado nem voltar a um nó (`rust/accord/src/assembly.rs:38` a `42`, o
`NodeBuilder` só existe logo depois de `node`). a composta é declarada
depois da projeção do app, a partir das ações que o app declarou naquela
mesma projeção (o que leem, se estão ligadas, a política), então o
builder ganha dois métodos: `declared()`, os nós declarados até ali, e
`reopen(id)`, continuar declarando um nó declarado antes. sem eles, cada
projeção seria feita duas vezes. nenhum tipo muda.

o módulo mora em `rust/offers/src/compose/`: o detector, a composta, a
proposta e o encadeamento. só depende do `sem`, que o `propose` já usa
(`rust/offers/Cargo.toml`), compila para wasm e mobile (o showcase
depende do `propose` em todo alvo), e não pede crate novo. não mora no
`rec` nem no `inspect`: o showcase só tem o `rec` no desktop e o
`inspect` como dev-dependency (`apps/uigallery/Cargo.toml`), e o
detector ao vivo tem que rodar em todo alvo; a sessão gravada chega nele
pelos eventos do `rec`. não mora no `sem`: é uma proposta para a pessoa,
não o contrato.

o `promote` e o template. a composta por item é ligada, na superfície,
dentro do template do item, com o alvo do template
(`silences.cut[{item}]`), e o verifier já aceita isso quando todo nó que
cabe no template declara a ação igual
(`rust/plevir/src/verify/bind.rs:169` a `213`). o `promote` da fase 1
procura a ação pelo alvo exato (`view.rs:154`, `caps.rs:138`), e o
registro não tem o template. o `uir` está fora desta fase, então o host
(`rust/screen/src/host/mod.rs`, o mínimo) entrega ao `promote` um registro
onde o template é mais um membro, com as ações de um membro de verdade:
a mesma regra do verifier (os membros declaram igual), só para aquela
operação; o que ela faz é verificado de novo como toda operação. o botão
vai para o contêiner de quem já manda um dos passos (o `into` é o pai do
controle da prévia ou do aplicar), um por item.

### (d) o encadeamento

cada passo é um `Request` do ator da composta. o passo 1 é observado
sobre o app logo depois do dispatch da composta (nada roda no meio, é o
mesmo empréstimo), então carrega o que quem pediu viu, e a composta lê a
união do que os passos leem, conferida contra o estado quando ela é
pedida. o passo k+1 é observado sobre o estado que o passo k deixou, logo
que o dispatch dele volta, e despachado quando o passo k termina: na
hora, se ele completou; ou, se ele começou e termina depois (a prévia de
um frame que ainda não foi decodificado,
`apps/uigallery/src/views/apps/silences/fixed_page/semantics.rs:328` a `340`),
quando o wake da página o encerra (`silences/jobs.rs:99`) e roda a
sequência adiante.

assim cada passo confere as próprias precondições contra o estado depois
do anterior, e o que mudou nesse meio, por outro ator enquanto a prévia
decodificava, faz o passo envelhecer: a recusa stale da 83 com o delta, e
o `acted` da 90/91 nomeando quem agiu (sem olhada, a partir da revisão que
o passo observou, `rust/accord/src/decision.rs:106` a `120`,
`dispatch.rs:359` a `394`). um passo que começou e cujo próprio efeito
chega depois também pode envelhecer o seguinte; a sequência para, com o
motivo, em vez de passar por cima do que não viu.

o primeiro passo recusado (stale, desligado, sumido, não autorizado,
inerte), que falhou, ou que ficou esperando uma pessoa, para a sequência.
o receipt da composta termina `failed` dizendo o passo (número, ação,
alvo, request) e por quê, e quais rodaram; completa com `steps`, `ran` e
os `requests` quando todos rodam. o `Stage` não tem lugar para "qual
passo" fora do texto, e um variante novo em `Stage` ou `Rejection`
quebraria quem casa neles; o receipt de cada passo guarda a recusa
inteira.

o que rodou fica rodado. nenhuma capacidade da Silences oferece
desfazer: `apply_cut` e `reject_cut` trocam a decisão, mas nenhuma volta
a `pending`, e a prévia só move o instante mostrado. o único desfazer do
sistema é o das transformações da superfície (`host.rs`, `undo`), que
não é ação do app. então não há rollback, e nenhuma transação foi
preciso (a task só pede rollback onde a capacidade oferece).

### (e) o aceite humano

o `confirm` e o `decline` do `sem` respondem a um request que alguém
fez e que espera (`dispatch.rs:195` a `236`). uma oferta não é request de
ninguém: nenhum ator pediu, e o detector não é ator (`request.rs:14` a
`19`). o aceite é o request da própria pessoa: `install` e `dismiss` no
nó da oferta, com a política `PERSON_ONLY` do showcase
(`apps/uigallery/src/views/dispatcher/shell/mod.rs:55` a `60`): um agente é
recusado `not_authorized` pelo `check` (`check.rs:186` a `190`), como nas
ações só da pessoa do shell; um teste faz as vezes da pessoa, como lá.
nada instala sozinho: só o `install` instala.

onde a pessoa vê: uma linha logo abaixo da linha de proposta da Silences
(`silences_surface.composite`, com "make it one action" e "not now"),
projetada só enquanto há oferta, então nenhum baseline de hoje muda. o
toast do shell não é controle (não se age sobre ele), e a linha de
proposta é dividida com a página Surface (os baselines dela). recusar
some com a oferta e guarda a sequência como recusada, na memória, até o
fim da sessão.

### (f) a demo

a Silences: "preview segment" e depois "apply cut", em cada corte. é o
"review, approve" da tese: a prévia é olhar o corte, o aplicar é
aprovar. o ide pede o desktop e o commit leva a mensagem (um parâmetro);
no todo, o add leva o texto, e "toggle e depois remove" não é o que se
faz. com a revisão pelas regras aberta, a pessoa (o harness, no regress)
olha e aplica os dois cortes, a oferta aparece; aceita, cada corte ganha
na superfície o botão "preview segment, then apply cut".

### nenhuma parada

a composta não pede tipo incompatível no `sem` (dois métodos novos no
builder) nem mudança no protocolo v1, nem transação, nem crate novo.

## handoff da fase 3 (2026-09-24)

```text
Goal: task-70 fase 3: a sequência que uma pessoa repete vira uma proposta
      de composta, só a pessoa aceita, e a composta roda como requests
      encadeados, cada passo conferido contra o estado depois do
      anterior; o primeiro recusado para a sequência com o motivo
Status: pronto para o orquestrador; adição ao ADR proposta
```

### commits

- 43cf8ff5 task-70: the probe of phase 3, the composite capabilities
- a3ecffd4 task-70: a detector reads the stages of the dispatcher, live and recorded, and finds the sequence an actor repeats
- 6ccfbad6 task-70: the repeated sequence is offered to the person, and only a person makes it one action
- d1106ad8 task-70: the composite accepted is an action of the app on every item, and the host promotes it into the item's template
- 5fa0874c task-70: the composite runs its steps as chained requests, and the first refused stops it with which step and why
- 4d96973a task-70: the demo, a sequence repeated on the review is offered and made one action, in a test and in the regress
- f00af5ea task-70: a step that waits for a person stops the chain and stays waiting
- f74ce129 task-70: a review the rules propose again shows the composite on each cut by itself
- o commit deste texto e do ADR

### a sonda

está na seção "sonda" da fase 3, acima, de (a) a (f). nenhuma condição
de parada aconteceu: nenhum tipo do `sem` mudou de forma incompatível
(dois métodos novos no `SnapshotBuilder`), o protocolo v1 não mudou,
nenhuma transação foi preciso, nenhum crate novo, nenhum baseline
existente mudou, o disco ficou em 18 GB livres.

um desvio da sonda, na (c): a composta fica ligada quando o primeiro
passo está, e não quando todos estão. com todos, "review, approve,
publish" nunca ligaria (publicar só liga depois de aprovar); cada passo
seguinte é conferido quando chega.

### o detector e a igualdade de passos

`rust/offers/src/compose/detect.rs`: `Detector::stage` (`:102`) lê o
fluxo de estágios `(id, request, estágio)` de um dispatcher; um request
entra no histórico (`:30`, 64 no máximo) quando termina, na ordem do id;
um request feito enquanto uma composta do mesmo ator roda é passo dela e
não entra. `found` (`:173`) acha a sequência mais curta, de dois passos
ou mais e primitiva, repetida N vezes seguidas no fim, pelo mesmo ator;
`runs` (`:195`) é a igualdade: a mesma ação no mesmo nó, ou em itens da
mesma coleção (`item_of`, `compose/mod.rs`), um item por repetição. só
entra como passo o que completou sem parâmetro, sem esperar pessoa e não
é composta. ao vivo, `Listener` (`:278`) é um `ReceiptSink`; gravado, os
eventos `Kind::Stage` do `rec` na ordem do arquivo, pela mesma chamada.
o detector não guarda valor de parâmetro nenhum.

testes: `rust/offers/tests/compose/detect.rs` (repetida N vezes
acha; outro ator no meio, outra ordem, um passo recusado, dois itens numa
repetição, uma ação só repetida, um passo que esperou a pessoa: não
acha; um passo que termina depois entra na ordem pedida; o parâmetro
secreto não fica no detector) e `apps/uigallery/tests/composites.rs` (a
sessão real da galeria gravada pelo `rec`, escrita num `.monster`, lida
de volta: a mesma resposta do detector ao vivo; e a demo gravada da 69,
`fixtures/demo/silences-demo.monster`, onde o agente repetiu "escolher o
proponente, escrever a intenção, propor": nada é achado, porque dois
desses passos levam parâmetro).

### onde a composta mora e o que mudou no `sem`

o módulo é `rust/offers/src/compose/` (sem crate novo, sem
dependência nova, compila em wasm e mobile). a composta é uma ação do
app: `Composite::declare` (`composite.rs:57`) a declara depois da
projeção do app, em cada item que tem as ações dos passos (ou no nó do
primeiro passo, se exata), com a união do que os passos leem, ligada
pelo primeiro passo, e a política mais estreita (`decl`, `:103`:
atores que todos deixam, confirmação mais forte, destrutiva se algum
for). `calls` (`:145`) dá os passos para o nó pedido.

no `sem`, só adição: `SnapshotBuilder::declared` (`rust/accord/src/assembly.rs:99`)
e `reopen` (`:107`), com o teste em `rust/accord/tests/assembly.rs`. na
Silences, `SilencesPage::project` termina com as compostas
(`apps/uigallery/src/views/apps/silences/fixed_page/semantics.rs:300`).

no host, o mínimo: `registry` (`rust/screen/src/host/mod.rs:123`, usado
em `apply_all`, `:310`) dá ao `promote` da fase 1 um registro onde o
template do item (`silences.cut[{item}]`) é mais um membro, com as ações
de um membro de verdade, só para aquela operação; o verifier confere o
resto como sempre. testes: `rust/screen/tests/promote.rs` (um botão
por item, cada um pede a composta do seu corte, desfazer volta; um corte
sem a ação faz o verifier recusar).

### o aceite

`Composer` (`rust/offers/src/compose/offer.rs`): `offer` (`:53`) só
oferece o que o app admite e que não foi recusado nem instalado na
sessão; `decline` (`:72`); `accept` (`:82`) instala e diz ao detector
que a ação é composta. nada instala sozinho. na Silences, a faixa
`silences_surface.composite` sob a linha de proposta
(`apps/uigallery/src/views/apps/silences/generative/offer.rs`: `band`
`:169`, `project` `:188`), com `install` e `dismiss` `PERSON_ONLY`; a
resposta roda em `Page::answered`
(`apps/uigallery/src/views/apps/silences/generative/page.rs:309`), que
promove o botão na revisão aberta (`promote`, `:360`, dentro do
contêiner de um passo, `beside`, `offer.rs:252`).

testes (`apps/uigallery/src/views/apps/silences/tests/composites.rs`):
oferecida e nada instalado; a pessoa aceita com um clique; um agente é
recusado `not_authorized` e a oferta fica; recusada, some e não volta
com a mesma sequência; pelo port de verdade, o agente repete os dois
cortes, vê a oferta e o `install` volta `requested, rejected`;
instalada com a revisão aberta, é ação de cada corte no registro e botão
de cada corte na revisão; sem superfície, instala e não promove.
`rust/offers/tests/compose/offer.rs` e `declare.rs` (o nome e o
rótulo vindos do app, um nome tomado ganha número, uma sequência que
passa por uma ação só da pessoa vira composta que um agente não pede).

### o encadeamento e a parada no meio

`Chain::advance` (`rust/offers/src/compose/chain.rs:75`): o `execute`
da composta responde `Started` (`start_composite`, `offer.rs:352`) e a
página roda a cadeia no mesmo empréstimo (`composed`, `offer.rs:334`;
`semantics.rs:437`) e no wake, quando um passo que começou termina
(`silences.rs:506`). cada passo é um request do ator da composta; o
seguinte é observado sobre o estado que o anterior deixou e despachado
quando ele termina. a composta completa com `steps` e `requests`, ou
falha dizendo o passo, o request, o porquê e o que rodou (`stop`,
`:138`). o que rodou fica rodado.

testes: `rust/offers/tests/compose/chain.rs` (todos os passos, do
ator da composta; um passo que o anterior desligou para, com o motivo, e
o que rodou fica; um passo cujo estado outro ator mudou enquanto o
anterior rodava para stale, com o delta e quem agiu; um agente não roda
a composta com um passo só da pessoa; um passo que pede confirmação
para a cadeia e fica esperando a pessoa) e, na Silences, a composta
clicada na revisão olha e aplica o corte, e a parada no meio: depois de
uma detecção nova, a pessoa pede a composta num corte cujo frame não foi
decodificado, um agente rejeita o corte enquanto isso, e o receipt da
composta diz `stopped at step 2 of 2, apply_cut on silences.cut[3]
(request ...): stale: silences.cut[3]:decision was "pending", is
"reject" (agent:bot ran reject_cut on silences.cut[3]); step 1 ran and
stays`; a prévia ficou, a decisão do agente ficou.

### a demo

a Silences com a revisão pelas regras aberta a 1280x800: o harness olha
e aplica cada corte (`examples/regress/state.rs:148` e `:157`, `acts`
novo no `State`, `:321`), a oferta aparece sob a linha; aceita, cada
corte da revisão ganha "preview segment, then apply cut", ao lado dos
botões do corte, e a faixa some. em teste:
`the_demo_offers_the_sequence_then_a_button_in_each_cut_of_the_review`
(a faixa dentro da janela, os dois botões desenhados, e uma revisão que
as regras propõem de novo mostra a composta em cada corte sozinha).

| | imagem |
| --- | --- |
| antes (a revisão pelas regras, baseline da fase 2) | `tool/bench/regress/baselines/silences-review-by-the-rules@core@1280x800/frame.png` |
| a proposta à vista | `tool/bench/regress/baselines/silences-review-by-the-rules-a-sequence-repeated@core@1280x800/frame.png` |
| a composta instalada | `tool/bench/regress/baselines/silences-review-by-the-rules-the-sequence-as-one-action@core@1280x800/frame.png` |

na proposta, sob "proposed by the rules for ...", a linha "you did
"preview segment, then apply cut" on each cut, 2 times in a row" com
"make it one action" e "not now"; os dois cortes aplicados. na
instalada, a linha sumiu, e cada corte tem, sob "preview segment",
"apply cut" e "reject cut", o botão "preview segment, then apply cut".

### baselines

- novos (2): `silences-review-by-the-rules-a-sequence-repeated@core@1280x800`
  e `silences-review-by-the-rules-the-sequence-as-one-action@core@1280x800`,
  com imagem.
- nenhum baseline existente mudou.

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ a0330678 (a contagem da fase 2) | 2809 passam, 25 ignorados (147 suítes) | 85 passam, 1 ignorado (6 suítes) | 2894 |
| depois, esta branch @ f74ce129 (`./tool/tasks/fullcheck.sh` offline, saída 0, 23:56:01 a 23:58:22) | 2847 passam, 0 falhas, 25 ignorados (150 suítes) | 85 passam, 0 falhas, 1 ignorado (6 suítes) | 2932 |

38 testes novos e três suítes (`propose`: `compose`, 23; `surface`:
`promote`, 2; `showcase`: `composites`, 2; mais 10 na lib do showcase e
1 no `builder` do `sem`); o gate inteiro verde, com a perna urnaui e o
check wasm com `-D warnings`. `df -h /`: 18 GB livres antes e depois.
`cargo run --offline -p regress -- check` (com o
`main.rs` tocado): 175 baselines, 175 iguais, nenhum frame desenhado,
nenhuma GPU aberta (os 173 de antes iguais, 2 novos). todo cargo com o
`PLEV_CLAUDE` apontando para o stub que só falha; nenhum `claude -p`
chamado.

### linha de ADR para o índice (o [`adr/`](../../adr/README.md) não está na minha lista)

| 2026-09-24 | [adaptive-transforms-preserve-identity](../../adr/Generative/0067-adaptive-transforms-preserve-identity/0067-adaptive-transforms-preserve-identity.md) | adição proposta | fase 3: `propose::compose`; o detector lê o fluxo de estágios de um dispatcher, ao vivo (`Listener`, um `ReceiptSink`) e gravado (os `Kind::Stage` do `rec`), pela mesma chamada; passos iguais pela ação e pelo nó, ou por itens da mesma coleção com um item por repetição, do mesmo ator, contíguos; passo com parâmetro, recusado, que esperou pessoa ou composto corta, e nenhum valor de parâmetro fica; a composta é ação do app declarada depois da projeção (`SnapshotBuilder::declared` e `reopen`, a única adição ao `sem`), ligada pelo primeiro passo, com a política mais estreita dos passos; o aceite é o request da pessoa (`install`/`dismiss` `PERSON_ONLY`), não `confirm`; o host promove no template do item com o template como membro do registro; a cadeia roda cada passo como request do mesmo ator, observado sobre o que o anterior deixou, e o primeiro recusado falha a composta dizendo qual passo e por quê; sem rollback (task-70 fase 3) |

### o que fica aberto para fechar a task-70

- accessibility como transformação (escala de fonte, contraste,
  densidade, reduced motion), depois da task-75, como a task manda.
- o `promote` de um template está no host (`registry`), pelo mínimo que
  esta fase podia; o lugar certo é o `promote` do `uir` entender o
  template, e aí o registro do host sai. decisão do orquestrador.
- o botão promovido tem o tamanho padrão ao lado dos botões pequenos da
  regra (o `promote` não lê os irmãos) e, na largura do cartão, desce
  para a linha de baixo.
- a composta liga pelo primeiro passo: num corte já aplicado o botão
  fica ligado, e a sequência para no aplicar (desligado), com o motivo. a
  imagem da instalada mostra os dois cortes aplicados, porque a semente
  tem dois.
- N é 2 na Silences (a semente tem dois cortes); num uso real, 3 talvez
  seja o certo. é uma constante (`REPEATS`, `generated/offer.rs:34`).
- a tela fixa da Silences não desenha o botão da composta: sem
  superfície aberta, ela está na projeção (agente, leitor de tela) e em
  toda superfície proposta depois.
- compostas não se aninham, não passam de uma sessão para outra, e o
  nome é das regras (os rótulos dos passos); nomear pelo Claude fica para
  depois, se valer.
- um passo que começa e cujo próprio efeito chega depois pode envelhecer
  o seguinte; a cadeia para com o motivo, em vez de agir sobre o que não
  viu.
- a adição ao ADR está como proposta; o índice é do orquestrador.
