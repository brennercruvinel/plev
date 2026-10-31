---
id: task-69/phase-4b
title: "task-69, phase 4b: the generated screen of the silences"
project: plev
type: section
status: reference
parent: task-69
date: 2026-09-24
date_basis: text
topics: [generated-surfaces, agents, ir]
relations: [adr-0049, adr-0051, adr-0052, adr-0053]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [3fb83d34, 50a815e9, af79fd3b, 191aa93e, b30a5b81]
---

# phase 4b: the generated screen of the silences

Records phase 4b: the generated screen of the silences, a materialized surface over the phase 4a app, which arises from a sentence, through the rules or through Claude, and operates the cuts through the app's dispatcher. The result is that the surface's image draws the frame the app has, through a `Frames` trait, with no ffmpeg in the drawing; the comparison keeps the identity of what continues, and the two screens ended up side by side.


branch `task/69g-silences-surface`, worktree próprio, sobre a `main` @
4a7a8abf (o fechamento da 4a). a pessoa diz "corta os silêncios desse
vídeo e depois quero revisar os cortes" e a tela de revisão nasce sobre
o app dos silêncios, com frames de verdade; diz "quero comparar o
original com os cortes" e a mesma tela vira comparação, mantendo a
identidade do que continua; terminou, some. esta fase deixa as duas
telas (a fixa da 4a e a gerada) prontas para o checkpoint b, que é
decisão do brenner.

## sonda da fase 4b (antes do primeiro tipo, 2026-09-24)

um teste descartável do showcase (`tests/probe69g.rs`, rodado com
`--nocapture` e jogado fora, nunca commitado) fez as três partes sobre o
golden `rust/plevir/tests/fixtures/uigallery-silences.snapshot.json` e sobre a
página real montada sem janela (`stage_page`). nenhuma janela, nenhum
ffmpeg, nenhum `claude`.

(a) o IR de revisão escrito à mão: um `card` com um `image` ligado ao
vídeo, quatro `stat` das contagens (`silences.counts:total`, `applied`,
`rejected`, `cut_secs`), uma linha com o `slider` do limiar
(`silences.threshold:value`, `set_value`), o botão de detectar
(`detect_silence`) e o de aplicar todos (`apply_all`), e uma `list` com
`repeat` sobre `silences:cuts` cujo item tem um `image` do corte, três
`text` (`start`, `end`, `duration`), um `badge` da `decision` e os botões
`apply_cut` e `reject_cut` sobre `silences.cut[{item}]`. três variantes
pelo `verify`:

- com `source` ligado a `silences.video:frame` e a
  `silences.cut[{item}]:frame` (o que a superfície precisa): recusado, 2
  erros, os dois da regra 4 (state): `the app declares no state
  silences.video:frame` e `no state of the app fits
  silences.cut[{item}]:frame`. só isso: nenhuma regra nova, falta o
  estado;
- com o que a projeção tem hoje (`silences.video:file` no vídeo e, como
  tapa-buraco, `silences.cut[{item}]:decision` no corte, os dois texto):
  ok; a ordem de foco é `review.threshold`, `review.detect`,
  `review.apply_all`, `review.cut[{item}].apply`,
  `review.cut[{item}].reject`;
- com um número (`silences.video:shown`, `silences.cut[{item}]:start`):
  recusado pela regra 4, `is num; a image's source takes text`.

o que o `image` de hoje liga: `source`, um estado de texto qualquer do
app (a regra 4 confere só o tipo). o que falta para um frame de um
instante: o instante. nada no vocabulário nem no verifier impede; o que
não existe é um estado de texto que nomeie o vídeo e o instante, e um
caminho do host até o cache de frames da página.

(b) o IR verificado (a variante que passa) materializado sobre a página
real, sem janela, desenhado em 900 px: 28 nós, `SceneNode::Image` no
frame: zero. o `image` do vídeo desenha o que desenha hoje: o retângulo
arredondado (`glass.surface`) com o label "the video" centrado, ligado
em `[308, 138, 860, 140]` (140 é `card_min_w / 2`, a altura de bloco da
fase 1). o `draw::image` de `rust/screen/src/draw.rs` é onde o host
precisaria perguntar o frame ao app antes de desenhar o label. e um
achado que a sonda não esperava: os itens saíram como
`review.cut[silences.cut[1\]]`, e os `text` do corte vazios. a projeção
da 4a põe em `silences:cuts` os ids inteiros (`"silences.cut[1]"`), não
as chaves (`"1"`), e o `repeat` põe o item onde está o `[{item}]`: as
ligações viraram `silences.cut[silences.cut[1\]]:start`, que não existe.
o todo projeta `todo:items` como `["1", "2", "3"]` (task-95), e a regra
das listas do proponente pede o mesmo ("os valores são as chaves dos
filhos do nó"). o app dos silêncios é que está fora da convenção, não o
`repeat` nem o verifier: a projeção passa a dar as chaves, o golden é
regravado, e o ADR da 4a ([`0053-the-silences-app-is-the-fixed-screen.md`](../../adr/structure/0053-the-silences-app-is-the-fixed-screen.md), que
esta fase não pode editar) fica com "the ids in order" onde é "the keys
in order": uma linha para o orquestrador.

(c) as regras (`Rules`) sobre o golden, sem superfície atual:

- "the cuts as a list with the video and the counts": `"video" is not a
  word of the app; its words are applied, cut, duration, end, pending,
  rejected, sec, shown, start, total, value`. as regras não têm parte de
  vídeo, então "video" não é palavra de nada;
- "the cuts as a list with the counts" e "the cuts as a list": `a list
  (items) asked for, and the app declares no list of items`: a mesma
  causa de (b), os ids em vez das chaves;
- "the counts": verifica, com quinze `stat`: `duration` e `shown` do
  vídeo, o `value` dos dois sliders (os dois com o label "value"),
  `start`, `end` e `duration` de cada corte, e os cinco de
  `silences.counts`. a regra das contagens é "cada int ou num vira um
  stat"; sobre o todo isso eram dois. fica como está (fora do escopo), e
  a observação vai para o handoff;
- "everything": o mesmo que "the counts" (nenhuma outra parte);
- "compare the original with the cuts": `"compare" is not a word of the
  app` (e "original" também não seria).

o que falta nas regras: uma parte de vídeo/imagem achada pela estrutura,
as palavras da gramática para ela ("video", "compare") e, na lista, o
estado de frame de cada item virando `image` em vez de `text`.

decisão da forma do frame, pela sonda: a forma (i), aditiva na projeção.
o app projeta um estado de texto por frame que quer mostrar, com a chave
do frame no valor: o id do nó de mídia e o instante em segundos,
`"silences.video@4.017052"`. no vídeo, `original` (o frame do original
no instante mostrado) e `result` (o frame do resultado no mesmo instante:
dentro de um trecho aplicado, o fim do trecho; fora, o mesmo instante);
em cada corte, `frame` (o meio do trecho). o `image` liga `source` a um
deles; o host pergunta o frame ao app por um trait (`surface::Frames`,
`fn frame(&self, source: &str) -> Option<ImageHandle>`) passado por quem
desenha a superfície; sem frame, desenha o que desenha hoje, e a página
pede a decodificação no `wake` (um job por vez, em cache). a forma (ii)
(um componente `media` com `at`) mudaria o vocabulário, o schema, o
verifier e o prompt para dizer o que um estado de texto já diz, e a
regra do proponente achá-lo-ia pelo mesmo caminho: o valor nomeia um nó
de mídia do app e um instante, que é estrutura, não nome de app.

## handoff da fase 4b (2026-09-24)

goal: task-69 fase 4b, a tela gerada dos silêncios: uma superfície
materializada sobre o app da 4a mostra frames de verdade (o do instante
mostrado e um por corte), nasce de uma frase pelas regras e pelo Claude,
opera os cortes pelo dispatcher do app, vira uma comparação lado a lado
mantendo a identidade do que continua, e se desfaz quando a pessoa
termina. as duas telas (a fixa da 4a, a gerada) ficam prontas para o
checkpoint b, que é decisão do brenner.

status: implementado. `cargo test -p screen -p uigallery -p offers -p
uir` verde, `./tool/tasks/fullcheck.sh` verde com a perna urnaui e a perna wasm,
`regress check` verde (as duas baselines dos silêncios abençoadas, dito
abaixo), sobre a `main` @ 4a7a8abf (a contagem abaixo). duas chamadas
reais ao Claude, as duas verificadas.

### commits

branch `task/69g-silences-surface`:

- 3fb83d34 task-69g: a sonda da fase 4b no arquivo da task
- 50a815e9 task-69g: the silences page projects the cuts by their keys, a
  frame ref per instant and the range (a projeção; o golden e as duas
  baselines regravados, nada visual mudou)
- af79fd3b task-69g: the frames trait and the image drawn from the app's
  frame (`rust/screen`; as duas fixtures; `tests/frames.rs`)
- 191aa93e task-69g: the generated surface beside the fixed screen, on
  the same dispatcher (a página; o bloco compartilhado da Surface;
  `dispatch.rs` e `input.rs`; as baselines com a linha de intenção)
- b30a5b81 task-69g: the rules find the media by structure and show a
  frame per item (`rust/offers`; o prompt; o exemplo com `--app
  silences` e `--shown`)
- e o commit dos adendos de ADR e deste handoff

o briefing pedia (2) o trait e (3) as fixtures com a página; a mudança da
projeção (as chaves em `silences:cuts`, os frame refs, o `range`) que a
sonda pediu veio antes, num commit só dela, para o golden ser o de
verdade quando o trait e as fixtures chegaram. as fixtures vieram com o
trait (o teste dele precisa de um IR sobre o golden).

### a sonda

acima ("sonda da fase 4b"). em uma linha: o verifier não precisa de
nada (o `image` liga qualquer texto; faltava um texto que nomeie o
frame), o `image` de hoje desenha o retângulo com o label, as regras não
tinham parte de vídeo, e a projeção da 4a listava os cortes pelos ids
inteiros em vez das chaves, o que quebrava o `repeat` e a parte de lista.
nenhuma condição de parada disparou.

### a forma do frame: (i), aditiva na projeção

o app projeta um estado de texto por frame que oferece, com o id do nó
de mídia e o instante em segundos no valor (`jobs::frame_ref`):
`silences.video:original` (o frame do original no instante mostrado),
`silences.video:result` (o frame do resultado no mesmo instante: dentro
de um trecho aplicado, o fim do trecho, o primeiro frame que fica; fora,
o mesmo instante) e `silences.cut[<id>]:frame` (o meio do trecho). um
`image` liga `source` a um deles. nada muda no vocabulário, no schema ou
no verifier; o prompt ganhou uma frase dizendo o que o `source` de um
`image` liga. a forma (ii) (um `media` com `at`) mudaria os quatro para
dizer o que um texto já diz, e as regras achariam o frame pelo mesmo
caminho (o valor nomeia um nó de mídia do app, achado pelo role).

junto, dois estados a mais que a sonda pediu: `silences:cuts` passa a
dar as chaves (`["1", "2"]`, como `todo:items`), e cada corte ganha
`range` (o texto do label, `0:03.0 to 0:05.0`), para a superfície mostrar
o intervalo sem formatar número. o golden e as baselines foram
regravados (commit 50a815e9); a projeção em repouso tem os mesmos dez
nós.

### o trait

`surface::Frames`, `fn frame(&self, source: &str) -> Option<ImageHandle>`
(`rust/screen/src/host/mod.rs`, ao lado do `Dispatch`), e `NoFrames`.
`Surface::render_with(c, rect, theme, &dyn Frames)` e
`SurfaceHost::draw_closing_with` perguntam o frame de cada `image` pelo
texto ligado no `source` quando desenham (`draw::picture`); `render` e
`draw_closing` são os mesmos com `NoFrames`, então a página Surface e os
testes de antes não mudam. um ghost de `image` pergunta o mesmo, e a
imagem some com o resto. `Surface::sources` (e o do host) lista os frame
refs que as imagens mostram: é o que o app decodifica. a superfície nunca
guarda bytes, nunca decodifica, nunca pede decodificação.

o desenho (`draw.rs`): a casca de antes e, dentro, a imagem ajustada e
centrada, nunca ampliada; sem imagem, a casca com o label. o tamanho é o
bloco (`card_min_w / 2`) ou, com `control_size`, a altura desse controle
em 16:9 (thumbnail numa linha; `FRAME_ASPECT`, geometria do conteúdo,
não token). o layout é o mesmo com e sem imagem, então um frame que
chega não move nada.

o app: `jobs::PageFrames` (o cache da página por instante, o ref
parseado em `instant_of`) implementa o trait; a página pede ao
trabalhador, no `wake` e nunca num frame, os refs que a superfície mostra
e ela não tem (`want_sources`), um job por vez, a falha em cache para
não pedir em laço. quando um frame chega, `settle` diz que moveu, a
página pede um frame, a superfície desenha.

### as fixtures

`rust/screen/tests/fixtures/uigallery-silences-review.ir.json` (id
`review`, "review the cuts"): o card com o `image` do vídeo
(`silences.video:original`), quatro `stat` das contagens, a linha da
detecção (o slider do limiar com `set_value`, detectar, aplicar todos), e
a `list` com `repeat` sobre `silences:cuts` cujo item é uma linha:
`image` do corte (`control_size` xl, `silences.cut[{item}]:frame`), uma
coluna (`review.cut[{item}].text`, flex 1) com o `text` do `range` e o
`badge` da `decision`, e os botões `apply_cut` e `reject_cut`.

`showcase-silences-compare.ir.json` (o mesmo id, "compare the original
with the cuts"): um `grid` sobre `field_min_w` com duas colunas, "the
original" com `review.video` (o mesmo id, `silences.video:original`) e
"with the cuts" com `review.result.video` (`silences.video:result`); os
`stat` de aplicados e segundos cortados (os mesmos ids); e a mesma lista
(`review.cuts`, o mesmo template) como as marcas da linha do tempo, cada
corte com o frame, o intervalo, a decisão e os dois botões. a linha da
detecção sai. as duas verificam sobre o golden real.

a primeira versão das fixtures punha o intervalo, a duração, o badge e os
botões numa linha só: a metade da página tem uns 430 px dentro do card, a
linha não cabia e os textos quebravam letra a letra. a coluna de textos
entre o thumbnail e os botões resolve; a duração por corte (`1.996916`,
sem formatação) saiu da revisão.

### a página

`apps/uigallery/src/views/apps/silences/generative/mod.rs` (o bloco: o
`SurfaceHost`, a superfície aberta, a linha de intenção da Surface, o
botão de fechar, as metades) e `generated/page.rs` (o `Operable` sobre a
página inteira, o padrão do `page.rs` da Surface sobre este app:
propor pelas regras dentro do `execute`, pelo Claude fora da thread com
`Started` e o `settle` no `wake`, cancelar, e `close`). o resto em
`silences.rs` (os eventos, as teclas, o `wake`, `with_host`), `draw.rs`
(a linha, a tela fixa, a superfície com os frames), `semantics.rs`
(`projection`, `act`, `focus`) e `jobs.rs`.

a linha de intenção é a da página Surface, compartilhada: `proposing.rs`
e `asking.rs` viraram `pub(crate)` e a linha toma o nome da região como
único parâmetro (`surface_page` lá, `silences_surface` aqui); a Surface
passa `REGION` e não muda de resto. o `Operable` (`page.rs` da Surface)
não foi compartilhado: ele fala com o todo pelo `with_dispatcher` com o
todo dentro; o dos silêncios copia o padrão (~250 linhas) sobre este
app. dito e a favor de uma generalização depois, se um terceiro app
precisar.

o dispatcher: o mesmo caminho da 4a, o do todo emprestado por
`with_dispatcher`. a superfície chega ao app por `App { page, todo }`
(`surface::Dispatch`: `page.dispatch(req, todo)` e `page.snapshot()`, a
projeção da tela fixa); como o host mora dentro da página, `with_host`
o empresta para os eventos de uma superfície como o todo empresta o
dispatcher. o `dispatch.rs` mudou uma linha (`semantics` projeta a
página inteira, `projection()`) e o `input.rs` duas (digitar e Enter
chegam à página), ditos no commit. nada no `sem`, nada no `app/**`.

a projeção em dois níveis: `snapshot()` é o app (a tela fixa, o registro
que a superfície verifica e lê; o golden), `projection()` é a página
(o app, a superfície, a linha e o botão de fechar: o que o chrome e a
porta veem). o golden por isso mudou de teste: de
`apps/uigallery/tests/plevir_fixture.rs` (que só chega à página inteira)
para `silences::tests::the_app_projection_is_the_uir_golden`
(`UIR_BLESS=1` regrava).

onde as coisas ficam (`Generated::halves`): a linha em cima, o botão de
fechar à direita dela; sem superfície, a tela fixa na largura toda; com
uma, larga, a fixa na metade esquerda e a superfície na direita; estreita,
a superfície embaixo da fixa na altura natural. fechada, a fixa toma a
largura de volta.

`close`: `silences_surface.close`, botão com a ação `close`, habilitado
só com superfície aberta, pela tela (o clique) e pela porta (um `act`);
`SurfaceHost::close`, a transição de saída da fase 2 (os frames somem
com o resto), a linha volta a dizer nada.

### as regras

`rust/offers`: uma quinta parte, mídia, achada pela estrutura: um nó
com role `media` ou `image` cujos estados de texto são frame refs (o
valor nomeia um nó do app com esse role, e um instante); cada estado é
uma vista (`original`, `result`); as palavras da parte são as do nó e as
chaves das vistas. palavras da gramática: `video`, `image`, `frame`
(`vídeo`, `imagem`, `quadro`, `media`, `preview`) para a parte;
`compare`, `comparar`, `comparison`, `versus`, `diff` para todas as
vistas lado a lado (um `grid` sobre `field_min_w`), e `compare` nomeia a
parte. sem `compare`, a primeira vista, ou as que uma palavra nomeia
pela chave ("the original", "the result"). na lista, o frame ref de um
item vira `image` (thumbnail, `control_size` xl) antes dos textos, nunca
`text`. a mídia vai no topo, antes das entradas e das listas.

uma mudança a mais, forçada: os números de um item de uma lista achada
não viram `stat` (são da lista). sobre os silêncios, "everything" fazia
um `stat` de `silences.cut[1]:start` com o id que o template faz para o
item 1, e o verifier recusava tudo (regra 2). o todo e o ide não têm
número em item: os 12 testes das regras e os 4 do llm passam sem
mudança. e uma observação que fica: `cut` é palavra da lista (o nome dos
itens) e de uma contagem (`cut_secs`), então "the cuts as a list with the
video and the counts" mostra a lista e só a contagem `cut_secs`, pela
regra da 3a ("a parte nomeada fica com o que a palavra nomeia"); "the
list with the video and the counts" mostra as cinco. e "the counts" sobre
os silêncios traz também `duration` e `shown` do vídeo e o `value` dos
dois sliders (os dois com label "value"): a regra "cada int ou num é um
stat" lê o valor de um slider como número que o app mostra. estreitar
isso (um número que uma ação do próprio nó recebe é do controle) é regra
de outra fase.

`rust/offers/tests/media.rs` (5 testes): a revisão (a vista do vídeo
em cima, `cut_secs`, a lista com `image` do frame, `range`, `decision` e
os dois botões, nenhum `text` de frame ref, nenhum `stat` de corte), a
comparação (as duas vistas lado a lado, `original`/`result` sozinhas), a
comparação sobre a revisão mantendo os ids da vista, da lista e do
template (e de volta, os mesmos ids), "everything" com a mídia em cima
e as contagens certas, e o todo e o ide recusando as palavras de mídia
com `Nothing` e sem `image` nas listas.

### as chamadas reais ao Claude

pelo `cargo run -p intent -- --app silences`,
sonnet, prompt de 16.958 bytes (a comparação, com a revisão como atual
pelo `--shown`, 22.032 bytes). duas chamadas, as duas verificadas; as
duas de reserva não foram usadas.

| chamada | intenção | superfície atual | tentativas | tempo | custo | o que veio |
| --- | --- | --- | --- | --- | --- | --- |
| 1 (07:20:40) | "cut the silences of this video, then I want to review the cuts" | nenhuma | 2: a primeira recusada com um erro só (o root com o id da superfície, `cuts_review`, regra 2); a segunda verificada | 123,0 s + 14,6 s = 137,6 s | 0,155 + 0,055 = 0,210 USD | a revisão (abaixo), 6.169 bytes |
| 2 (07:23:33) | "compare the original with the cuts" | a da chamada 1 | 1, verificada | 40,4 s | 0,076 USD | a comparação (abaixo), 3.748 bytes |

a revisão da chamada 1, resumida: `cuts_review` "cut silences, then
review", um `stack` vertical com um `text` de título, um `image` "video
preview" ligado a `silences.video:result`, um `card` "detection settings"
com os dois `slider` (`set_value`) e o botão de detectar (com `busy`
ligado a `silences.status:busy`, um `SubmitButton`), um `text` do status,
cinco `stat` das contagens, o botão de aplicar todos, e uma `list` com
`repeat` sobre `silences:cuts` cujo item é um `card` com o `text` do
`range`, o `text` da `decision`, um `image` "cut preview frame" ligado a
`silences.cut[{item}]:frame`, e três botões (`preview_segment`,
`apply_cut`, `reject_cut`). o template tomou os ids do app
(`silences.cut[{item}]`, o mesmo que a 3b-ii viu no ide): o verifier
aceita e o host trata a raiz da instância como nó do app; os filhos são
da superfície. a frase nova do prompt bastou para ele ligar o frame de
cada corte ao ref, sem instrução de `repeat`. sem as regras: "then" e
"review" não são palavras do app.

a comparação da chamada 2: o mesmo id `cuts_review`, o título trocado, e
em cima um `split` horizontal 50/50 com `cuts_review.original` (novo,
`silences.video:original`) e `cuts_review.preview` (o id da chamada 1,
mantido, `silences.video:result`); o card da detecção, o status, as
contagens, o aplicar todos e a lista inteira com os mesmos ids. os ids do
que continua foram mantidos: sim, todos, a superfície inclusa. as regras
dão 17 nós para a mesma frase.

os dois IRs inteiros ficaram em
`/private/tmp/claude-501/-Users-nn-Dev-hoff-plevdev/ffe9cd40-0b24-414e-be8c-32dec2cf8b3d/scratchpad/claude-1.txt`
e `claude-2.txt` (a saída do exemplo), e o da chamada 1 em
`claude-1-review.ir.json`; nada foi commitado.

### PNG e projeção

`SILENCES_OUT=<dir> cargo test -p uigallery --lib
silences::tests::generated::the_real -- --ignored --nocapture` (o ffmpeg
real) escreveu `silences-review-real.png` e `silences-compare-real.png`
com os `.json`, em 0,68 s, em
`/private/tmp/claude-501/-Users-nn-Dev-hoff-plevdev/ffe9cd40-0b24-414e-be8c-32dec2cf8b3d/scratchpad/out/`;
o teste do gate (`the_page_draws_headless_with_the_review_and_the_comparison`,
o falso, com GPU) escreve `silences-review.png` e `silences-compare.png`
no mesmo `SILENCES_OUT`.

a revisão (1200x800 core): a linha de intenção em cima (o campo, o
select `rules`, propor desabilitado sem intenção, fechar; cancelar na
linha do status), a tela fixa na metade esquerda (os cards empilhados,
a caixa do vídeo com o frame do `testsrc` aos 0,0 s, a banda com as duas
marcas), e na direita o card gerado: o frame do vídeo em cima (320x180
ajustado em 140 de altura), `2 silences / 0 applied / 0 rejected / 0.0
seconds cut`, o slider do limiar com "detect silences" e "apply all", e
as duas linhas de corte, cada uma com o thumbnail do meio do trecho
(78x44), "0:03.0 to 0:05.0" sobre o badge "pending", "apply" e "reject":
6 `SceneNode::Image` no frame, 3 da fixa e 3 da gerada.

a comparação, depois de `preview_segment` e `apply_cut` no primeiro
corte: "the original" com o frame aos 4,0 s (o dígito 4 do `testsrc`)
ao lado de "with the cuts" com o frame aos 5,0 s (o fim do trecho
aplicado, o dígito 5), `1 applied / 1.996916 seconds cut`, e as duas
linhas com o primeiro corte em "apply" (o botão aplicar desabilitado): 7
imagens no frame (a quarta da gerada é o frame de 5,016 s). a tela fixa
ao lado mostra a agulha dentro da primeira marca, vermelha.

a projeção das duas (a página inteira, 44 nós cada; os ids dos cortes
são 3 e 4 porque a detecção real cunhou ids novos): o app (os dez de
sempre, com `original`, `result`, `range` e `frame`), a superfície
(`review`, o card, o `image` do vídeo, os `stat`, a linha da detecção
com `set_value`/`activate`, a lista com `review.cut[3]` e `review.cut[4]`
nomeados pelo item, cada um com o `image`, a coluna, o `range`, o badge
e os dois botões) e a linha (`silences_surface` com `intent`,
`proposer`, `propose`, `cancel`, `status`, `close`). na comparação,
`review.video` continua com o mesmo id e `review.result.video` nasce; a
lista é a mesma; `review.cut[3].apply` lê desabilitado. os `.json` estão
no diretório acima.

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 4a7a8abf (o handoff da 4a sobre a mesma árvore) | 2428 passam, 0 falhas, 15 ignorados (109 suítes) | 82 passam, 1 ignorado (6 suítes) | 2510 |
| depois, esta branch @ b30a5b81 (`./tool/tasks/fullcheck.sh`, saída 0) | 2444 passam, 0 falhas, 16 ignorados (111 suítes) | 82 passam, 0 falhas, 1 ignorados (6 suítes) | 2526 |

16 testes novos passando (3 em `surface/tests/frames.rs`, 7 em
`silences/tests/generated.rs`, 5 em `propose/tests/media.rs`, 1 em
`jobs.rs`, e o golden dos silêncios que saiu de `uir_fixture.rs` e entrou
em `silences/tests.rs`) e 1 ignorado a mais (o ffmpeg real das duas
formas). a contagem não cai. clippy `-D warnings`, `cargo fmt --check`
e o check wasm do showcase com `-D warnings` verdes, dentro do gate, com
a perna urnaui (nenhuma perna pulada; `../urna` resolve deste worktree).
`df -h /` antes do gate e depois: 16 GB livres, os dois. o gate rodou uma
vez nesta branch, inteiro verde, em 2 min 13 s (07:24:51 a 07:27:04).
`cargo run -p regress -- check`: 159 baselines
conferidas, 159 iguais, nenhum frame desenhado (as duas dos silêncios
abençoadas duas vezes nesta branch: pela projeção em 50a815e9, nada
visual mudou, e pela linha de intenção em 191aa93e, com a imagem).
`view/dispatch.rs`: 363 linhas antes, 363 depois (uma linha trocada);
`view/input.rs`: duas linhas de rota a mais.

### testes

- `rust/screen/tests/frames.rs` (3): a revisão sobre o golden com
  `sources` iguais aos três refs, zero imagem sem frames, três com o
  provedor falso (uma imagem 32x18 por ref, gerada em memória com
  `load_image_rgba`), cada uma dentro do rect do seu nó e nunca ampliada,
  o layout igual com e sem, o label do vídeo de volta quando só os
  cortes têm frame, a projeção (`image`, a instância nomeada pelo item,
  o `range`); a transformação em comparação mantendo a instância dos
  cortes (um press começado no botão de um corte completa depois e
  aplica o corte pelo dispatcher do app de teste), o `image` do vídeo
  deslizando, a linha da detecção saindo, quatro imagens, e o `close`
  desenhando os ghosts com as imagens até sumir; o todo sem `source`;
- `silences/tests/generated.rs` (8, um `#[ignore]`): a linha projetada
  com o app e fora do registro (o `close` desabilitado recusado como
  `Disabled`, o select só com `rules`); a revisão aberta ao lado (a
  fixa na esquerda, a superfície na direita; embaixo em estreito), os
  frames pedidos no `wake` (o worker ocupado, `POLL`) e desenhados
  quando chegam (6 imagens); aplicar pela gerada mudando a fixa (o
  receipt do `Human` em `silences.cut[1] apply_cut`, o botão da linha
  escuro, o badge e a contagem da gerada), rejeitar pela fixa mudando
  a gerada, e um `act` de `Assistive` num botão da gerada roteado para o
  app; a comparação mantendo a intenção digitada e a instância de um
  corte (o press em flight completa), o vídeo movendo de coluna, e o
  `close` por clique (fora da projeção, a fixa na largura toda, a saída
  em vários frames, o receipt `Completed`) e por request depois
  (`Disabled`); a galeria roteando a linha, as teclas, uma proposta pelas
  regras por Enter ("the counts"), Escape e o `close` por `act`; o
  Claude falso respondendo a revisão fora da thread (`Started`, o poll,
  `Completed` com `cost_usd`, a superfície aberta); a página sem janela
  com as duas formas sobre o falso (6 e 7 imagens, PNG com GPU); e o
  ignorado com o ffmpeg real;
- `silences/tests.rs` (+1, 2 mexidos): o golden do app; os dois de
  layout medindo a tela fixa no rect que as metades dão;
- `rust/offers/tests/media.rs` (5, acima); `words.rs` (+1 caso);
  `jobs.rs` (1: o frame ref e o instante);
- os testes de antes: os 6 da Surface page (`surface_page::tests`, 29),
  os 12 das regras e os 4 do llm, os do `surface` (todos), e o golden do
  todo, sem mudança.

### validação executada

- `cargo test -p screen -p uigallery -p offers -p plevir` (e
  `claude-model` pelo gate);
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt`, o check wasm
  do showcase com `-D warnings`, dentro do gate, com a perna urnaui;
- `cargo run -p regress -- check`;
- o ignorado do ffmpeg real (`the_real_ffmpeg_draws_the_review_and_the_comparison`),
  o headless do falso com `SILENCES_OUT`, o exemplo do `claude-model`
  duas vezes (a tabela). nenhuma janela foi aberta; nenhum teste do gate
  chama o `claude` nem exige o ffmpeg.

### seções de ADR

- [`0051-materialized-surfaces-from-verified-ir.md`](../../adr/generated/0051-materialized-surfaces-from-verified-ir.md), adendo datado
  "an image draws the frame the app holds (task-69 phase 4b, proposed)":
  o frame ref como forma do app, o trait `Frames`, `sources`, a imagem
  ajustada, a página, as consequências (as fixtures, as chaves na lista,
  o tamanho do frame falso, a largura da metade) e o que evitar;
- [`0052-proposers-propose-the-runtime-verifies.md`](../../adr/generated/0052-proposers-propose-the-runtime-verifies.md), adendo datado
  "the rules show the media by structure (task-69 phase 4b, proposed)":
  a quinta parte, as vistas e o `compare`, o frame de item como imagem,
  os números de item fora das contagens, a mídia no topo, o prompt, o
  exemplo, e as consequências (o que cada frase dá, `cut` em duas
  partes, as contagens dos sliders);
- [`0049-plev-ir-typed-and-verified.md`](../../adr/generated/0049-plev-ir-typed-and-verified.md) e o schema não mudaram (forma (i)).
- para o orquestrador (fora do que esta fase pode editar): no índice
  [`adr/`](../../adr/README.md) os dois adendos; em
  [`0053-the-silences-app-is-the-fixed-screen.md`](../../adr/structure/0053-the-silences-app-is-the-fixed-screen.md) a projeção diz "the
  ids in order" onde agora é "the keys in order", e os estados
  `original`, `result`, `range` e `frame` entram na lista da seção "the
  projection"; o [`changelog.md`](../../log/changelog.md) (a superfície gerada na página
  Silences); no `docs/ATLAS.toml` os módulos `view::apps::silences::{jobs,
  generated, generated::page}` e o `tests/frames.rs` do `surface`.

### decisões

- a forma (i) do frame, pela sonda (acima);
- `Frames::frame(&self, ..)` só consulta: a decodificação é pedida pela
  página no `wake` a partir de `sources`, nunca num frame (a regra da 4a
  de não rodar ffmpeg num frame vale para a gerada também), e nunca pela
  superfície;
- o tamanho de um `image` não depende do frame (o layout não pula quando
  o frame chega); `control_size` é a única alavanca do IR para um
  thumbnail;
- a linha de intenção compartilhada por nome de região, o `Operable`
  copiado (acima);
- a tela fixa fica na largura toda em repouso e cede a metade quando a
  gerada nasce (as baselines dos silêncios mudam pela linha em cima,
  nada mais);
- o golden é o app, a semântica da página é a página inteira;
- `result` como o fim do trecho aplicado que contém o instante mostrado:
  o único "resultado" que existe sem cortar o vídeo de verdade; fora de
  um trecho aplicado é o mesmo frame do original;
- os números de um item de lista saem das contagens (forçada pelo
  verifier, acima).

### limitações

- o frame falso das provas é 32x18 e nunca é ampliado: no PNG do falso
  cada imagem é uma pílula pequena no meio da casca; o real é 320x180 e
  enche a altura;
- a metade gerada tem uns 430 px dentro do card em 1200 de largura: os
  textos de uma linha de corte quebram em duas linhas ("0:03.0 to" /
  "0:05.0"), e um IR do Claude com mais coisa numa linha quebra pior. o
  motor de layout dá a um `text` o que sobra e o texto quebra letra a
  letra em vez de o container transbordar: comportamento do engine, fora
  desta fase;
- a revisão do Claude nomeou o template com o id do app
  (`silences.cut[{item}]`), como a 3b-ii já tinha visto no ide: o host
  aceita e a raiz da instância fica sendo o nó do app; pedir ids
  próprios é mudança de prompt de outra fase;
- o `Operable` da página é uma cópia adaptada do da Surface (~250
  linhas); dois apps, duas cópias;
- "the cuts as a list with the video and the counts" pelas regras dá só
  `cut_secs` como contagem (a palavra `cut` nomeia essa contagem); e "the
  counts" traz os valores dos sliders como stats;
- as contagens de uma detecção real cunham ids novos (3 e 4), então os
  `.json` da projeção real não são o golden;
- o `close` não confirma: uma superfície com texto digitado num campo
  some com ele (nenhuma fixture tem campo);
- o clippy com `-D warnings` recusou um `mod support` sem `allow` no
  teste novo do `propose` (os outros binários usam tudo dele); ficou o
  `#[allow(dead_code)]` com o comentário, como nos testes do `surface`.

### dependências pendentes e riscos de integração

- para o orquestrador: as linhas de ADR e o CHANGELOG acima; o
  `arc.toml`;
- `Cargo.toml` e `Cargo.lock` não mudaram: nenhuma dependência nova;
- a Surface page depende de `Proposing::new(REGION)` (uma linha
  mudada); quem mover o dispatcher para a `ShowcaseView` troca duas
  linhas aqui (`App::dispatch` e `Page::dispatch`);
- a projeção dos silêncios mudou (as chaves, quatro estados): um agente
  que lia `silences:cuts` como ids lê chaves agora;
- as duas baselines dos silêncios mudaram duas vezes nesta branch (a
  projeção, depois a linha em cima); `regress check` verde.

### o que a 4c (o vídeo da demo) e o checkpoint b precisam

- as duas telas estão prontas e lado a lado na página Silences: a fixa
  (a 4a) e a gerada, pela mesma frase da tese no campo de intenção com
  `claude` escolhido (137 s na chamada 1; a página mostra a espera e
  cancela), ou pelas regras em menos de um milissegundo com "the list
  with the video and the counts" e "compare the original with the cuts";
  o `close` desfaz. o roteiro: abrir a página, "detect silences" (o
  ffmpeg gera o vídeo e detecta), a frase, a revisão nasce ao lado com os
  frames chegando, aplicar um corte em qualquer das duas telas, "compare
  the original with the cuts", a comparação com o resultado diferente
  onde há corte aplicado, fechar;
- para gravar (task-63, se integrada): a janela real só com
  `PLEV_BACKGROUND=1`; sem janela, os PNGs do teste ignorado e do falso;
- o que o checkpoint pode pedir e não existe: o corte real (`export`),
  um `result` que reflita mais de um trecho aplicado (hoje é o instante
  mostrado dentro do trecho que o contém), ids próprios no template do
  Claude, e a linha de intenção sem o botão de fechar quando nada está
  aberto.
