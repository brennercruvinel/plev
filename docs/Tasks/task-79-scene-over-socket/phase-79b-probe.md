---
id: task-79/phase-79b-probe
title: "phase 2 (task-79b): the probe"
project: plev
type: section
parent: task-79
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [agents, semantics, accessibility]
relations: [task-79/phase-79a-probe, adr-0065, adr-0067]
evidence: [bfe12733, dfbd79de, c3b94b7a, a2b70151, 80d71a45]
source: docs/log/steps/task-79-scene-over-socket.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 2 (task-79b): the probe

The phase 2 probe measured, across the gallery's 82 pages, how many wired controls the mirror can press: 445 are visible, 134 have no primary action, and no node appears marked as a secret in the projection. With that, a click in the viewer returns to the app as an `act` backed by an observation, under the same hit and action rule the screen reader uses, and a node that `sem::redact` counts as a secret leaves the frame as a box with a position and size. Protocol v1 does not change, and the mirror honors the refusals that apply to any agent, such as `PERSON_ONLY` and confirmation by the person in the app.

branch `task/79b-mirror-input`, base `8f35c3ee`. worktree
`.claude/worktrees/agent`, target `target/agents/one`. a medição está em
`apps/uigallery/tests/mirror_probe.rs` (ignorada; `cargo test -p
showcase --test mirror_probe -- --ignored --nocapture`): as 82 páginas
da galeria nos temas core e hoff, 1280x800 @1, sem GPU (a cena montada
num `Compositor` puro), 164 frames.

```text
controles ligados e visíveis:                               445
ligados mas fora da projeção da galeria:                      6  (Git, core e hoff: changes, stacks, diff)
nós marcados secret na projeção:                              0
ligações numa layer escondida:                                0
pares que se sobrepõem com ordem de ligação != empilhamento:  0
o que um clique aperta num controle ligado (primary):
  select 141, activate 90, toggle 44, open 8, submit 8, propose 4,
  apply_all 2, close 2, detect_silence 2, open_file 2, warn 2,
  nenhuma 134, fora da projeção 6
```

## (a) o input

o acerto. o app acha o controle sob um ponto por `Compositor::hit`
(`engine/src/compositor/binding.rs:199-229`): entre as ligações cujo rect
e cujo clip contêm o ponto (bordas inclusivas, `binding.rs:122-126`),
numa layer visível e com opacidade acima de zero, ganha a de maior
`(z da layer, posição da layer, ordem de ligação)`, a ordem em que o
frame compõe. o visualizador recebe `Bound { id, rect }`, o rect já
cortado pelo clip (`Binding::visible_rect`, `binding.rs:49-54`), na ordem
de `visible_bindings()` (`binding.rs:189-192`, ordem de ligação, sem a
layer), que é o que `app::mirror::offer` põe no frame
(`showcase/src/app/mirror.rs:94-118`). sem a layer, o visualizador não
tem como refazer o empilhamento. o menor caminho: o app manda os
`bounds` do frame já na ordem de empilhamento (de baixo para cima) e sem
as ligações de layer escondida, e o visualizador pega a última que
contém o ponto. é a regra do `hit`, então:

- sobreposição: ganha a de cima, como no app (medido: na galeria nenhum
  par sobreposto tem ordem de ligação diferente do empilhamento, então
  hoje as duas ordens dão o mesmo; a ordem nova é a que continua certa
  quando não derem);
- overlay: um modal, um menu ou um toast numa layer de z maior ganha do
  que está embaixo, pelo z;
- clip: o rect que viaja é a parte visível; um ponto na parte cortada não
  acha aquele controle (acha o de baixo, ou nada);
- as ligações desenhadas pelo próprio visualizador (contorno e nome) não
  entram no acerto.

nada muda na forma do `bounds` (uma lista de `Bound`); muda a ordem que o
app escolhe, que o protocolo nunca fixou. a observação (`observe` com
`bounds`) segue na ordem de ligação, pelo `Host::bounds` de sempre.

a ação. a primária do nó, pela mesma regra do leitor de tela:
`engine::accessibility::primary` (`engine/src/accessibility/action.rs:98-109`):
a primeira de `activate`, `toggle`, `submit`, `select` sem parâmetro
(`action.rs:45`), senão a única sem parâmetro de um botão, link ou item
de menu (`action.rs:138-149`), senão o `open`/`close` de um select
(`action.rs:111-121`). e só quando a árvore do leitor ofereceria o
clique: a ação ligada e o nó ligado (`accessibility/tree.rs:316`). por
quê: um clique no espelho é o clique de uma pessoa, e o leitor já é a
tradução de um clique para o contrato; outra regra seria uma segunda
tradução. um controle sem primária (um campo de texto, um rótulo, uma
região: 134 dos 445) não manda nada, e o visualizador diz que ali não
há o que apertar. digitar, arrastar e rolar ficam de fora: pedem
parâmetro, e um parâmetro vindo de quem assiste é outra decisão.

a observação. a v1 tem dois apoios (`wire/src/proto.rs:87-146`):
`observed`, os valores dados pelo agente, tomados como dados, ou
`from_revision`, uma revisão que o app deu a esse ator, de onde o app
lê o que ele viu. o watch não dá revisão nenhuma: ele avisa o host
(`Host::observed`) e não grava nada no `Shown` do ator
(`wire/src/server.rs:314-327`), e o `rest` só aceita revisão dada ao
ator (`server.rs:576-601`, `shown.rs:200`). então o visualizador observa.
escolha: `from_revision` de uma observação inteira (`Observe` sem
escopo, que mostra todo nó e todo estado, então nenhuma leitura cai em
`not_observed`), refeita depois de cada frame que chega (coalescida:
uma por vez), e o clique age sobre a última. assim a observação anda
junto com a cena que a pessoa vê: se o app mudou depois (a cena está
velha), o ato volta `stale`, com o delta e quem agiu, como para qualquer
agente. observar só no clique seria mais barato e apoiaria o ato num
estado que a pessoa não viu (um toggle desfeito por outro ator seria
refeito sem recusa).

a conexão. outra conexão, com o mesmo `hello` ("mirror"): o ator é o
nome, e duas conexões sob um nome são um ator (`wire/src/shown.rs:15-19`),
então a observação feita numa vale para o ato da mesma. na mesma conexão
não dá: o `Client` lê por id e pula o que não é do id que espera
(`wire/src/client.rs:84-97`), então um ato engoliria os frames; e um ato
pode esperar uma pessoa sem prazo (`awaiting_confirmation`,
`proto.rs:123-133`), o que pararia a leitura da cena, que no `--show` é
de uma thread própria (`examples/mirror/show.rs:47-63`).

o ator. toda conexão é `Actor::Agent` pelo `hello`
(`server.rs:380-385`), então o espelho é o agente "mirror". fica assim,
como o orquestrador recomenda: o app não sabe quem está do outro lado do
socket. uma ação `PERSON_ONLY` (`view/dispatch/shell.rs:51-60`; a da
câmera, `view/apps/capture/semantics.rs:98-109`; o `open_file` e o
`warn` do Files) é recusada `not_authorized` pelo dispatcher, e o
visualizador diz a recusa. uma confirmação pedida pela política
(`sem/src/decl.rs:120-131`, `Confirm::ForAgents`) chega como
`awaiting_confirmation`: o visualizador diz que a pessoa do app confirma
lá, e mostra o fim quando ele vier; confirmar e recusar pelo socket
seguem proibidos (`server.rs:183-190`).

o protocolo v1: nada muda. nenhum pedido, resposta ou campo novo; o
espelho usa `hello`, `observe`, `act` e `scene` como são. um cliente de
antes não percebe nada.

## (b) a redação

onde marcar: já está marcado. o snapshot diz `constraints.secret` por
`SemId` (`sem/src/decl.rs:16-18, 51-52`; o valor sai só como tamanho,
`sem/src/validate.rs:173-177`), e a ligação diz o rect e a layer de cada
`SemId` (`Binding`, `binding.rs:25-42`; `bindings()`, `binding.rs:181-184`,
inclusive as cortadas pelo clip). o que falta é juntar os dois antes de a
cena sair.

a regra de quem é segredo: a de sempre, `sem::redact::is_secret`
(`sem/src/redact.rs:29-34`), a que o port já aplica ao pedido e a cada
estágio (`server.rs:25-28, 531-532, 266`): um nó marcado secret, ou que
o snapshot não tem, ou nenhum snapshot, conta como segredo (falha
fechada). medido: nenhum nó secret hoje, e 6 ligações fora da projeção,
os três painéis da página Git (changes, stacks, diff: o workbench do ide
liga os ids, e a galeria não projeta a página Git). com a regra, o
espelho mascara os três na página Git até a galeria projetar a página
(fora desta fase: `src/view`). é o custo da falha fechada, e o que ela
protege: o diff mostra conteúdo de arquivo que o contrato nunca mostrou a
agente nenhum.

onde mascarar: no app, antes de codificar. o showcase decide o que é
segredo (`app/mirror.rs`: a projeção da galeria no mesmo instante do
frame, e as ligações do compositor da janela) e entrega ao codec uma
lista de máscaras (a layer e o rect de cada segredo); o codec
(`rec::scene::Sender`, `rec/src/scene/send.rs:62-103`) escreve cada layer
já mascarada. assim o segredo não entra no corpo, no patch, no hash nem
num asset, e o `AgentPort::offer_scene` e o `Server` recebem bytes que
nunca o tiveram. no codec do `rec` só a máscara (um módulo novo); o
`SceneNode`, o tema e o `engine` não mudam.

o que é mascarado: a caixa, com lugar e tamanho, como a câmera
(`rec/src/scene/mirror.rs:349-383`): no lugar do primeiro nó que o
segredo desenha, uma caixa escura com a borda e "secret, not mirrored",
cortada no rect. os nós da layer do segredo que tocam o rect não viajam:
todo texto, imagem e path que toca, e toda forma que toca sem cobrir o
rect inteiro (o caret, a seleção, uma marca). a forma que cobre o rect
inteiro viaja (o chão: a página, o painel, o fundo e a borda do próprio
campo). um texto é medido pelo mesmo medidor do engine
(`TextMeasurer::measure_styled`, o do desenho), então um texto rolado
para a esquerda num campo, com a origem fora do rect, também não viaja.
por que a caixa e não pontos do mesmo comprimento: os pontos precisariam
ser medidos e desenhados pelo app, deixariam o caret e a seleção (que
andam com a largura de cada letra) para mascarar do mesmo jeito, não
servem a um segredo que não é texto (uma imagem, um path), e diriam o
comprimento, que a observação já diz a quem observa. a caixa não diz
nada além do lugar e do tamanho, e é o mesmo desenho de "não
espelhado" que a câmera já tem.

o hash e o patch: o corpo da layer mascarada não depende do segredo (a
caixa depende só do rect), então o hash não muda quando o segredo muda,
e o patch é escrito contra a layer mascarada de antes. a pessoa digitar
num campo secreto não move layer nenhuma e não manda nada (nem o caret
piscando): nem o conteúdo nem o ritmo das teclas atravessam. o campo
mudar de lugar ou de tamanho move a layer, como qualquer nó.

o limite: a máscara vê o que é desenhado dentro do rect ligado, na layer
da ligação. um segredo desenhado em outro lugar (um status que cita a
senha, um tooltip noutra layer) precisa ser marcado e ligado ele mesmo,
como o leitor de tela já pede (um status secret não é anunciado,
`accessibility/tree.rs:283, 312-315`). um nó secret projetado e não
ligado não tem lugar, e o que ele desenha viaja: é da página ligar o que
marca.

## (c) o segredo de verdade

não existe campo de senha no comps (nenhum "password", "secret" ou
"obscure" em `rust/engine/src/widgets`), e nenhuma página projeta um nó secret.
o menor segredo desenhado, sem widget novo: uma cena de teste com um
campo desenhado como um campo (o chão, a borda, o texto do segredo, o
caret, dentro de um clip) ligado a um `SemId` que a projeção do teste
marca secret, e um status secret desenhado (a live region do teste do
leitor, `accessibility/tree/tests.rs:556-568`), passando pelo mesmo
caminho do shell (as máscaras da projeção, o codec, o `Server`, o
`Viewer`).

## (d) as promessas que já valem, e o teste de cada uma

1. a cena só com `PLEV_MIRROR=1` da pessoa: `wire/src/scene.rs:169-178`
   (`MIRROR_ENV`, `mirror_enabled`), `scene.rs:213-222` (a recusa
   `scene_not_shared`), `wire/src/port.rs:188-198` (`from_env`). contra o
   input: sem a variável o watch é recusado, o visualizador não tem
   ligação nenhuma, e um clique não manda nada ao app.
2. a imagem de fluxo nunca atravessa: `rec/src/scene/send.rs:114-118` (a
   câmera vai como tamanho e lugar), `rec/src/scene/mirror.rs:236-241` (a
   caixa). contra o input e a máscara: com a câmera falsa ligada e a cena
   mascarada, nenhum byte dos frames dela no fio, e um clique na caixa da
   câmera não aperta nada (`capture.frame` não tem primária).
3. quem assiste fica no log e na gravação: `wire/src/scene.rs:252-256,
   277-285` (os eventos), `server.rs:323-325` (`Host::observed` no
   watch), `showcase/src/app/mirror.rs:71-92` (a linha do log),
   `view/dispatch/agent.rs` (`Served::observed` grava). contra o input:
   o olhar e o ato do espelho ficam na gravação em nome de "mirror"
   (a observação e o receipt), como o watch.
4. a redação do `sem`: `sem/src/validate.rs:173-177` (o snapshot guarda o
   tamanho), `sem/src/redact.rs` (o pedido e cada estágio), a live region
   (`accessibility/tree.rs:312-315`). contra a máscara e o input: um
   status secret desenhado não está em byte nenhum do fio de cena, e a
   observação em que o clique se apoia tem só o tamanho dele.
5. ação `PERSON_ONLY` recusada a agente: `view/dispatch/shell.rs:51-60`,
   `sem/src/check.rs:41` (`NotAuthorized`). contra o input: o clique no
   toggle da câmera pelo espelho volta `not_authorized`, o visualizador
   diz a recusa, e a câmera fica como estava.
6. a preferência de accessibility da pessoa nunca sai pelo port nem vai
   para o prompt: [`0067-adaptive-transforms-preserve-identity.md`](../../adr/Generative/0067-adaptive-transforms-preserve-identity/0067-adaptive-transforms-preserve-identity.md)
   (fase 4: a faixa do Silences, `PERSON_ONLY`, e a projeção igual com e
   sem preferência), `view/apps/silences/generated/access.rs:12, 182`.
   contra o input: o clique do espelho em "larger text" é recusado, e a
   observação em que ele se apoia é a mesma, byte a byte, com e sem a
   preferência (e é dela que o prompt sai). o que a cena mostra dela,
   registrado sem mudar: a superfície desenhada com o texto maior e mais
   contraste, que é pixel; a pessoa pediu para compartilhar a tela, e a
   tela mostra a preferência como mostra qualquer coisa desenhada. quem
   assiste pode inferir que ela está ligada olhando o tamanho do texto;
   não recebe o valor, não pode mudá-lo, e o modelo não recebe nada.

## (e) quem mais lê o canal de cena

- o laboratório (`apps/agenteval/src`): fala `observe` e `act`, nunca
  `scene`; nada muda.
- a gravação (`Host::observed`): além do watch, cada olhar do espelho é
  uma observação de "mirror" (uma por frame que chega, coalescida), e
  cada clique um receipt de "mirror". nenhuma mudança de código na
  gravação.
- os testes do `wire` (`tests/scene.rs`, `env.rs`, `codec.rs`): o
  protocolo não muda; nada muda.
- os testes do showcase (`tests/mirror.rs`): os `bounds` do frame passam
  a vir na ordem de empilhamento (na galeria é a mesma de hoje); o app
  filho do teste de socket redesenha depois de um ato e usa as máscaras
  do shell; as páginas do teste de socket não têm segredo nem id fora da
  projeção, então o pixel igual segue.
- o codec (`rec/tests/scene.rs`): sem máscara, os bytes de sempre.

## handoff da fase 2 (task-79b)

branch `task/79b-mirror-input`, base `8f35c3ee`. worktree
`.claude/worktrees/agent`, target `target/agents/one`. a rodada não
parou: nada pediu mudança no `sem`, no `engine`, no comps, no
`SceneNode`, no tema ou no protocolo v1. commits, em ordem:

- `bfe12733` task-79: the phase 2 probe, the mirror's click and the secret's mask
- `dfbd79de` task-79: the viewer finds the control under a point as the app does
- `c3b94b7a` task-79: a click at the mirror goes back as an act on the control
- `a2b70151` task-79: a secret travels masked, as its place and size
- `80d71a45` task-79: the channel's promises, against the click and the mask
- (este) task-79: ADR phase 2 and the phase 2 handoff

nenhuma janela foi aberta (o `--show` compila e não rodou), nenhum
clique ou tecla do sistema: os testes chamam `Viewer::hit` e
`Hand::click` com um ponto no espaço da cena, dentro do processo. o app
do teste de socket é o filho headless da fase 1, que sai pelo `quit` ou
pelo `Drop` do teste, pelo handle dele; nenhum processo ficou. o
`claude -p` não foi chamado (o `PLEV_CLAUDE` apontou para o stub).

### a sonda

a seção "fase 2 (task-79b): a sonda" acima, com a medição em
`apps/uigallery/tests/mirror_probe.rs` (ignorada). em uma linha: o
espelho acha o controle pelas ligações em ordem de empilhamento, aperta a
primária do leitor de tela com um `act` apoiado num olhar que ele faz
depois de cada frame, numa segunda conexão com o mesmo nome; o app
mascara o que o `sem::redact` conta como segredo, antes do codec
escrever a layer; o protocolo v1 não muda.

### o input, como ficou

- `app::mirror::stacked`: o app manda os `bounds` do frame de baixo para
  cima, na ordem do `Compositor::hit`, sem as ligações de layer
  escondida. `Viewer::hit(x, y)` pega a última que contém o ponto.
- `app::mirror::Hand`: a mão do espelho. `connect(path, name)` abre uma
  conexão própria e diz `hello` com o nome do watch; `look()` observa o
  app inteiro; `click(viewer, x, y, each)` acha o controle e chama
  `press(target, each)`, que pega a primária do nó no último olhar
  (`engine::accessibility::primary`, só se a ação e o nó estão ligados)
  e manda `act` com `from_revision` do olhar. `each` ouve cada estágio.
  `shown()` devolve o último olhar.
- `Pressed` diz o que deu: `Nothing` (nenhum controle), `Unpressable`
  (nada a apertar, ou desligado), `Acted` (os estágios), `Refused` (o
  port recusou antes do dispatcher). `said()` e `said(target, action,
  stage)` dão a frase do espelho: "todo.item[1]: toggle done",
  "capture.camera: toggle refused: only a person asks it, and the
  mirror acts as agent:mirror", "... refused: the mirror was behind the
  app (todo.item[1]:completed moved, agent:claude acted since); look
  again", "... waits for the person at the app, who confirms or declines
  it there".
- o exemplo: no `--show`, clique esquerdo aperta o controle sob o
  ponteiro (numa thread própria, a mão; a mão olha o app depois de cada
  frame) e o que veio sai no stderr e no título da janela. headless,
  `--click x,y` aperta depois do primeiro frame e imprime os estágios.

### a máscara, como ficou

- `app::mirror::masks(now, compositor)`: cada ligação que
  `sem::redact::is_secret` conta como segredo na projeção da galeria
  daquele frame (marcada, fora da projeção, ou sem projeção: falha
  fechada) vira um `rec::scene::Mask` (a layer e o rect ligado, a parte
  cortada pelo clip junto). `Scenes::offer` ganhou um parâmetro, a
  projeção (uma closure, chamada só quando alguém assiste).
- `rec::scene::Sender::frame_masked` (e `scene/mask.rs`): antes de
  escrever a layer, tira o que o segredo desenha (todo nó dentro de um
  clip que cabe no rect; todo texto, imagem e path que toca o rect, o
  texto medido pelo medidor do engine; toda outra forma que toca sem
  cobrir) e põe no lugar do primeiro uma caixa escura com a borda e
  "secret, not mirrored" (`SECRET_LABEL`). o chão que cobre o rect
  inteiro e os clips seguem. `frame` é `frame_masked` sem máscara, os
  bytes de antes.
- o segredo mudar não muda byte da layer: digitar num campo secreto não
  manda nada.
- efeito medido da falha fechada: na galeria, só a página Git tem
  máscara (changes, stacks e diff, nos dois temas), porque a galeria não
  projeta a página Git. **decisão para o orquestrador**: manter a falha
  fechada (a regra do port, e o diff mostra conteúdo de arquivo que o
  contrato nunca mostrou a agente) ou mascarar só o marcado secret (a Git
  volta inteira no espelho). o custo da troca é uma linha em
  `app::mirror::masks` e o teste que fixa as páginas mascaradas.

### os testes

- `showcase`, `app::mirror::tests`: o acerto por empilhamento e não por
  ordem de ligação, com uma layer escondida; a gravação guarda o olhar e
  o clique do espelho em nome dele.
- `showcase/tests/mirror.rs`: o acerto do espelho igual ao do app em
  todas as páginas nos dois temas (centro de cada controle e grade de
  20 px); o clique entre dois processos (o item vira feito, o frame
  volta com o hash do app, o ato de um agente cai no mesmo frame e na
  mesma revisão; o toggle da câmera recusado e nada redesenhado). o
  teste da câmera da fase 1 agora procura todas as 200 cores do fake, não
  só as 16 primeiras (o número do frame do fake segue o relógio).
- `showcase/tests/mirror_input.rs` (em processo, sem GPU): o clique chega
  ao estado e à revisão do ato de um agente; `PERSON_ONLY` recusado e
  dito; ponto vazio, região e submit desligado não mandam nada; cena
  velha volta `stale` e passa depois de olhar; e as promessas (abaixo).
- `showcase/tests/mirror_mask.rs` (com GPU): o formulário de login
  desenhado de verdade (senha e pin secretos) em nenhum byte do watch, o
  espelho com a caixa em cada lugar e igual ao app fora deles, uma tecla
  manda 0 bytes; todas as páginas pelo caminho mascarado, pixel igual
  onde nada é mascarado, só a Git mascarada e diferente só dentro das
  máscaras.
- `rec`, `scene/tests_mask.rs`: nenhum byte do texto do segredo, dos
  pixels do ícone ou do path no frame (uma linha rolada para fora e um
  texto com origem fora do rect incluídos); os mesmos bytes qualquer que
  seja o segredo; layer sem máscara igual a antes.
- o segredo desenhado de teste: `showcase/tests/login/mod.rs` (sem widget
  novo).

### as promessas

um teste por promessa da sonda (d), contra o clique e a máscara:

1. a cena só com `PLEV_MIRROR=1`: `unshared_the_mirror_has_nothing_to_click`
   (o watch recusado com a razão, nenhuma ligação, o clique não manda
   nada, observar segue funcionando).
2. a imagem de fluxo nunca atravessa:
   `a_camera_s_frames_never_cross_whatever_the_mirror_clicks` (a caixa da
   câmera não aperta nada, o toggle é recusado, nenhuma das 200 cores do
   fake no fio, com os frames dela provados na cena do app).
3. quem assiste fica no log e na gravação:
   `the_recording_keeps_the_mirror_s_look_and_click_under_its_name` (o
   watch e o olhar como observações de "mirror", o clique como receipt
   de "mirror").
4. a redação do `sem`, inclusive numa live region:
   `a_secret_leaves_as_its_length_and_never_as_what_it_draws` (senha e
   pin como tamanho no olhar, como caixa na cena, em nenhuma linha).
5. `PERSON_ONLY` recusado a agente:
   `a_click_on_an_action_kept_for_a_person_is_refused_and_said` (e o de
   dois processos).
6. a preferência de accessibility: `the_person_s_preference_never_leaves_by_the_port`
   (o olhar igual, nó a nó e revisão, com e sem "larger text"; o clique
   do espelho nela recusado). o que a cena mostra, registrado sem mudar:
   a revisão desenhada com o texto maior e mais contraste é pixel que a
   pessoa pediu para compartilhar; quem assiste pode inferir que a
   preferência está ligada pelo tamanho do texto, não recebe o valor,
   não pode mudá-lo, e nada disso vai para o prompt (a projeção que o
   proponente lê é a mesma).

### o gate e o regress

- `./tool/tasks/fullcheck.sh` (`scratchpad/gate-79b.txt`): verde, pernas urnaui e
  wasm rodaram. 3 120 passados, 0 falhas, 28 ignorados (antes: 3 103 e
  27). +17: 2 unitários do espelho no showcase, 2 no `mirror.rs`, 8 no
  `mirror_input.rs`, 2 no `mirror_mask.rs`, 3 no `rec`; +1 ignorado, a
  sonda. `df -h /`: 24 GB livres antes.
- `regress check`: 176 baselines, 176 iguais.
- linhas de produção: `app/mirror.rs` 580, `rec/scene/mask.rs` 201,
  `rec/scene/send.rs` 163, `examples/mirror/show.rs` 299, `main.rs` 284.
- nenhuma dependência nova, nenhuma aresta nova no `Cargo.lock`. arquivos
  fora da lista: nenhum (os novos são `rec/src/scene/mask.rs` e
  `tests_mask.rs`, dentro de `rust/record/src/scene/**`, e testes).

### a linha do ADR para o índice

```text
| 2026-09-25 | [scene-travels-as-data](scene-travels-as-data.md) | proposed | task-79 fase 2: o espelho responde e o segredo não atravessa. um clique no visualizador volta como `act` pelo contrato, nunca evento nem coordenada: o app manda os `bounds` do frame em ordem de empilhamento (a do `Compositor::hit`), o visualizador acha o controle sob o ponto e aperta a primária do leitor de tela (`engine::accessibility::primary`), com `from_revision` de um olhar que a mão do espelho faz depois de cada frame, numa segunda conexão com o mesmo nome; o app o toma como o agente "mirror" (`PERSON_ONLY` recusado, confirmação pela pessoa no app, cena velha `stale`). o que `sem::redact::is_secret` conta como segredo (falha fechada) sai do frame antes do codec escrever a layer (`Sender::frame_masked`) e uma caixa "secret, not mirrored" fica no lugar, com lugar e tamanho só; digitar num campo secreto não manda nada; na galeria só a página Git, não projetada, fica mascarada; protocolo v1 sem mudança |
```

### o roteiro para o brenner ver o clique

num checkout desta branch, dois terminais:

```text
# 1. o app, com o port ligado e a cena compartilhada (build de debug)
PLEV_AGENT=1 PLEV_MIRROR=1 cargo run -p uigallery

# 2. o espelho numa janela
cargo run -p mirroring -- --show
```

- na página Todo, clique num item no espelho: o item vira feito no app, e
  o espelho diz "todo.item[1]: toggle done" no terminal e no título.
- na página Capture, clique no toggle da câmera no espelho: "refused:
  only a person asks it". a câmera não liga.
- clique num lugar sem controle: "no control at (x, y): nothing went to
  the app".
- sem janela: `cargo run -p mirroring -- --frames 1
  --click 640,300` aperta o que estiver em (640, 300) depois do primeiro
  frame e imprime o que o clique deu.

### o que fica para depois

- a página Git projetada na galeria (tira a máscara dela), ou a decisão
  de mascarar só o marcado secret (acima).
- um widget de senha no comps que marque e ligue o segredo (hoje só o
  teste desenha um).
- digitar, arrastar e rolar pelo espelho: pedem parâmetro vindo de quem
  assiste, outra decisão.
- a revisão do frame no próprio frame (o app dizer em que revisão
  desenhou), para o clique se apoiar exatamente na cena, sem o olhar por
  frame: pede mudar o `Server` e o `Shown` do `wire`.
- o resto da fase 1 segue: sair do localhost (pareamento, transporte
  cifrado), o gravador guardar os pixels das imagens fixas, o custo de
  uma imagem fixa grande no fio do app, override da câmera, vídeo de
  pixels, multiusuário, iOS e android.
