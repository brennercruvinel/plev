---
id: task-74/phase-3-bidi
title: "phase 3: bidi in the cursor and the selection"
project: plev
type: section
status: reference
parent: task-74
date: 2026-09-25
date_basis: text
topics: [text, text-editing]
relations: []
source: docs/log/steps/task-74-text-and-editing.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [61283afa, 555ac24a, 7a30e038, a746f2b3]
---

# phase 3: bidi in the cursor and the selection

Makes the caret and the selection follow the visual order in mixed text, without changing the logical movement of the rope. The new `TextMeasurer::clusters_styled` query returns each line's clusters from left to right, and the arrows, the click and the selection of `TextInput`, `TextField` and the editor use those stops. Adds 25 new tests and the opened PNG of `TextField`; the close-out at the end of the page resolves the pending shift+arrow item in the apps.

## fase 3: a sonda

data: 2026-09-25. branch `task/74c-bidi` sobre `13fed644`. as linhas da
biblioteca são do fonte travado no Cargo.lock: cosmic-text 0.18.2, no
registro do cargo. medida com Inclusive Sans a 16 px, sem wrap, pelas
faces embutidas (Noto Sans Hebrew e Arabic no fallback).

### (a) como o caret e a seleção são posicionados hoje, e onde quebram

o rope já anda por grafema e em ordem lógica: `move_left` e `move_right`
(`rust/editor/src/document/movement.rs:175-189`) vão ao grafema anterior e ao
seguinte (`movement.rs:43-78`, `grapheme_indices` por linha). em texto
misto isso está certo como movimento lógico; o que falta é o visual. o
editor liga as setas direto nessas funções (`rust/engine/src/widgets/editor/input.rs:60-75`),
então em trecho RTL a seta direita anda para a esquerda na tela.

o `TextBuffer` do `TextInput` anda por `char`, não por grafema
(`rust/engine/src/widgets/form/text_input/buffer.rs:98-110`, `153-167`), e não
tem seleção estendida: só `select_all` cria seleção.

o x do caret sai de `cursor_rect_in` (`rust/engine/src/drawing/text/measure.rs:179`),
usado por `TextMeasurer::cursor_x_styled` no `TextField`
(`rust/engine/src/widgets/form/text_field.rs:365`) e no editor
(`rust/engine/src/widgets/editor/view.rs:142`). ele devolve `glyph.x`, a borda
esquerda do glifo que contém o byte (`measure.rs:200`), e no fim da linha
`last.x + last.w` do último glifo do vetor (`measure.rs:209`). as duas
coisas supõem LTR. num glifo RTL o começo lógico é a borda direita, e num
parágrafo RTL o último glifo do vetor é o mais à esquerda. medido:

- `abc אבג def`: o byte 4 (começo do hebraico) cai em 48,0, entre ב e א,
  dentro do trecho; o certo é 33,0 (fim do espaço, lado LTR) ou 57,0. o
  byte 6 cai em 39,3 e o 8 em 33,0: todo caret dentro do trecho RTL fica
  um glifo deslocado.
- `hello مرحبا 123 world`: o byte 6 (logo depois de `hello `) cai em
  93,2, dentro do árabe entre ر e م, longe do espaço (39,3); os bytes de
  dentro do árabe saem um glifo deslocados, como no hebraico (o 8 em
  88,1, o certo é 93,2).
- `שלום עולם` (parágrafo RTL): o byte 0 cai em 55,1 e não na borda
  direita (66,2); o fim (17) cai em 9,6 e não em 0.

a seleção é um retângulo só, de `cursor_x(lo)` a `cursor_x(hi)`
(`text_field.rs:325-332`; no editor, `selection_rect_on_line`,
`view.rs:418`). em texto misto um intervalo lógico cobre pedaços que não
se tocam na tela, e a largura pode sair negativa: selecionar `א` e `ב`
(4..8) em `abc אבג def` dá x 48,0 e largura -15,0; o editor descarta
largura não positiva e não desenha nada.

o clique usa `Buffer::hit` do cosmic-text (`measure.rs:171`), que já
trata a metade do glifo RTL (`buffer.rs:966`, `right_half != is_rtl`), mas
devolve o byte sem a afinidade: clicar na metade esquerda de ג em
`abc אבג def` dá 10, que o caret então desenha em outro lugar.

### (b) o que o shaping já dá e o que falta

cada `LayoutGlyph` traz `start`, `end` (bytes do cluster na linha), `x`,
`w` e o nível bidi (`layout.rs:14-36`); cada `LayoutRun` traz a direção do
parágrafo (`buffer.rs:26`). os níveis vêm do `BidiInfo` da linha
(`shape.rs:1155-1169`), só do texto: não dependem de face nem de tamanho.
as runs saem reordenadas (`reorder`, `shape.rs:1320`), e o parágrafo RTL
é montado da borda direita para a esquerda (`shape.rs:2580`,
`2708-2709`, `2752-2763`): no vetor, o primeiro glifo é o mais à direita.
glifos do mesmo cluster (marca separada, ligadura) repetem `start..end`.

medido em `hello مرحبا 123 world`, da esquerda para a direita: `hello `
em nível 0, `123` (17..20) em nível 2, o espaço 16..17 e as cinco letras
árabes (14..16 até 6..8) em nível 1, ` world` em nível 0. a ordem visual
está certa; o que falta é uma leitura que entregue isso ao widget. hoje o
engine só expõe o x de um byte (`cursor_x_styled`) e o hit, sem nível nem
ordem. falta também o `highlight` do cosmic-text (`buffer.rs:45`) como
saída: ele devolve um intervalo só por run, do menor ao maior x, o mesmo
defeito do retângulo único.

### (c) o desenho

- o movimento lógico fica no rope, puro, como está; entram testes em texto
  misto (as duas frases, uma só RTL, marcas combinantes).
- no engine, uma leitura nova e só ela: `TextMeasurer::clusters_styled`
  devolve os clusters de cada linha da esquerda para a direita, com bytes,
  `x`, `w`, nível e direção do parágrafo. nada do que existe muda.
- no comps, uma linha visual construída dessa leitura: cada fronteira de
  grafema vira uma parada com x e posição na ordem visual. na fronteira
  entre direções, o byte fica na borda do lado de nível menor (o lado do
  parágrafo), a regra usual: o caret em `abc |אבג def` fica logo depois do
  espaço, e o texto LTR digitado ali aparece onde o caret está. com isso
  cada byte tem uma parada e cada parada um byte, e a seta anda para a
  parada vizinha na tela. na borda da linha, a seta segue a ordem lógica
  na direção do parágrafo (no editor, vai para a linha vizinha).
- seleção continua lógica (âncora e cabeça em bytes) e desenha como um ou
  mais retângulos: cada cluster coberto vira uma caixa, e caixas que se
  tocam na tela se juntam. em latim sai o retângulo de sempre, com a mesma
  conta.
- home e end continuam pela linha lógica; shift estende a seleção lógica.
  o clique passa a achar a parada mais próxima, para cair onde o caret é
  desenhado.

## handoff da fase 3: bidi no cursor e na seleção

fecha o item "testes de movimento bidi" do critério de aceite, com gate
verde. escopo: movimento lógico e visual, seleção contínua em texto
misto, no `TextInput` e no editor, testes no rope e nos dois widgets.

### commits

- `61283afa` task-74: phase 3 probe, where the caret and the selection break in mixed text
- `555ac24a` task-74: logical movement in the rope, pinned in mixed text (Hebrew and Arabic in Latin, an RTL paragraph, niqqud and harakat)
- `7a30e038` task-74: visual caret movement and selection in TextInput, from a new per-cluster layout read
- `a746f2b3` task-74: visual caret movement and selection in the editor
- o commit deste handoff

### a sonda, curta

o rope já andava certo em ordem lógica, por grafema. o que quebrava era
o desenho: o x do caret (`cursor_rect_in`) supõe LTR, então dentro de um
trecho RTL todo caret saía um glifo deslocado, e num parágrafo RTL o
começo e o fim caíam no lugar errado; a seleção era um retângulo só, com
largura negativa em alguns intervalos. o shaping já dava tudo o que
faltava (nível bidi por glifo, direção do parágrafo, ordem visual); só
não havia leitura disso fora do engine. detalhe em "fase 3: a sonda".

### o que mudou, por crate

- `engine`: uma leitura nova, `TextMeasurer::clusters_styled` com o tipo
  `ClusterBox` (`rust/engine/src/drawing/text/measure.rs`): os clusters de cada
  linha da esquerda para a direita, com bytes, caixa, nível e direção do
  parágrafo. nada do que existia mudou.
- `rope`: só docs e testes. `move_left` e `move_right` passam a dizer que
  são lógicos (grafema anterior e seguinte na ordem do texto).
- `comps`, `TextInput`: `rust/engine/src/widgets/form/text_input/visual.rs`
  (`VisualLine`, uso interno do crate) faz de cada fronteira de grafema
  uma parada com x e lugar na ordem da tela. na troca de direção o byte
  fica na borda do lado de nível menor, o lado do parágrafo: em
  `abc |אבג def` o caret fica depois do espaço, onde a letra latina
  digitada aparece. setas andam para a parada vizinha na tela; Home e End
  pela linha lógica; `handle_select_left`, `_right`, `_home`, `_end`
  estendem a seleção, que continua um intervalo lógico só
  (`TextBuffer::select_to`, `set_cursor`); `caret_x` e `selection_boxes`
  dão a geometria; o clique (`handle_click_in`) cai na parada mais
  próxima. o `TextField` desenha caret e seleção por aí, um retângulo por
  trecho coberto, e ganhou `EditKey::SelectLeft`, `SelectRight`,
  `SelectHome`, `SelectEnd`. um parágrafo RTL fica pendurado na borda
  direita da caixa, como o renderer desenha.
- `comps`, editor: as setas usam a mesma linha visual; na ponta da linha,
  o movimento lógico na direção do parágrafo leva à linha vizinha. alt e
  cmd com seta, Home e End continuam lógicos no rope. caret, clique e
  seleção leem as mesmas paradas; a quebra de linha selecionada aparece
  do lado em que o parágrafo termina (à esquerda numa linha RTL).
- latim: a mesma conta de antes, bit a bit (há teste disso), e as 176
  baselines iguais.

### testes (25 novos)

- rope, `rust/editor/src/tests/bidi.rs` (6): as duas frases e a frase RTL
  andam em ordem de texto nos dois sentidos; niqqud e harakat andam junto
  da letra; seleção que cruza a troca de direção é um intervalo lógico
  (e colapsa nas pontas lógicas); início, fim de linha e palavra em texto
  misto.
- engine, `rust/engine/src/drawing/text/tests/measure.rs` (3): clusters da
  esquerda para a direita com os níveis (`abc אבג def` e um parágrafo
  RTL); em latim cada cluster começa no x do caret de antes; marcas
  entram no cluster da letra.
- `TextInput` e `TextField`, `rust/engine/src/widgets/form/text_input/tests/bidi.rs`
  (11): a seta direita percorre a tela nas três frases, a esquerda volta,
  o caret anda para a direita a cada toque e cada byte aparece uma vez;
  num trecho RTL a esquerda avança no texto; o caret nas fronteiras; o
  parágrafo RTL na borda direita da caixa; shift com setas e com Home e
  End; clique em cada parada volta a ela; latim bit a bit igual; o
  `TextField` desenha o caret da fronteira, as duas lavagens da seleção
  cruzada e o caret do parágrafo RTL.
- editor, `rust/engine/src/widgets/editor/tests/bidi.rs` (5): as setas percorrem
  cada linha e passam para a vizinha nas pontas; na linha RTL a esquerda
  avança; shift cruzando direções desenha duas caixas com os dígitos no
  meio; o caret da fronteira e o clique em cada parada das três linhas; a
  quebra de linha selecionada do lado certo.

### o PNG

`/Users/nn/Dev/hoff/plevdev/target/agents/one/task-74c/text-field-bidi.png`,
sem janela, pelo caminho do frame (2x). três `TextField` no tema hoff:
`hello مرحبا 123 world` com o caret na fronteira depois de `hello `
(esquerda do 123); a mesma frase com shift+direita seis vezes a partir de
`hel|lo`, duas lavagens (`lo ` e o árabe com seu espaço, o `123` fora) e
o caret entre o 3 e o espaço árabe; `שלום עולם` com o caret no começo,
na borda direita. para refazer:
`TEXT_FIELD_BIDI_OUT=<dir> cargo test -p engine --lib the_field_draws_the_boundary_caret`.

### gate e regress

`./tool/tasks/fullcheck.sh` verde com o ambiente do briefing: `cargo test --workspace`
3104 passed, 0 failed, 27 ignored (a base tinha 3079: são os 25 novos,
nenhum saiu); clippy, fmt e wasm verdes; perna urnaui 85 passed, 0
failed, 1 ignored. `regress -- check` depois de tocar
`tool/bench/regress/src/main.rs`: 176 unchanged, 0 changed.
fora do gate: `cargo check -p engine --target aarch64-apple-ios-sim` com
`-D warnings` verde; `cargo ndk -t arm64-v8a check -p uigallery --lib`
compila, com os dois avisos antigos de código morto. disco: 11 GB livres.

### pendências

- os apps não mandam shift+seta para o campo: `apps/idestudio/src/interfaces/window/keyboard.rs`,
  `apps/uigallery/src/keyboard.rs` e o `keys.rs` do urnaui mapeiam só a
  tecla, sem modificador, e estão fora dos arquivos desta fase. o
  `TextField` já aceita `EditKey::Select*`; falta a linha em cada app. o
  editor não precisa (recebe os modificadores).
- no editor, entre uma linha LTR e a seguinte RTL, a direita vai e volta:
  do fim da LTR vai ao começo da RTL (na borda direita dela), e de lá a
  direita volta à linha anterior. é a regra "visual dentro da linha,
  lógico entre linhas"; mudar é decisão.
- alt+seta (palavra) e as setas de cima e de baixo continuam lógicas: numa
  linha RTL alt+esquerda vai para a palavra anterior do texto, que na
  tela fica à direita; a coluna-alvo vertical conta grafemas, não x.
- um cluster maior que um grafema (ligadura, como lam-alef) põe o caret
  numa fração da largura; o corpus não tem ligadura, então isso não tem
  teste com pixel.
- o `TextField` desenha com largura de quebra e mede sem ela: texto maior
  que o campo quebra na tela e o caret segue na linha única. já era
  assim no latim; campo de linha única pede rolagem horizontal, fora do
  bidi.
- o `TextInput` tira a ordem das setas do estilo padrão, porque a ordem
  vem só dos níveis bidi do texto; o x do caret usa o estilo do dono.

## fase 3 integrada (orquestrador, 2026-09-25)

fecha o item "testes de movimento bidi" do critério de aceite: 25 testes
novos no rope (ordem lógica), no engine (a leitura por cluster,
`TextMeasurer::clusters_styled`), no `TextInput`/`TextField` e no editor
(setas na tela, caret nas fronteiras, seleção cruzando direções, clique,
latim igual a antes).

validação do orquestrador: diff lido (o engine só ganha uma leitura; nada
do que existia mudou); o PNG aberto: o caret depois de `hello ` fica no
lado do parágrafo, a seleção que cruza as direções desenha em duas faixas
com o `123` de fora, e a frase em hebraico começa na borda direita.

continuam abertos no critério: roteiro de IME executado (fase 4, pede o
brenner), emoji colorido (fase 5), um milhão de linhas (fase 7).

o agente gastou uns 260 mil tokens.

## shift+seta nos apps (orquestrador, 2026-09-25)

fecha a pendência da fase 3 "os apps não mandam shift+seta para o
campo", com o ok do brenner, que a reproduziu na página Input da
galeria. a galeria (`showcase/src/keys.rs`), a linha de intenção e os
campos da superfície do ide (`ide/src/keys.rs`) e o urnaui
(`urnaui/src/keys.rs`) mandam shift com as setas, Home e End como as
teclas `Select` do campo; onde nenhum campo as toma, andam como sem shift
(abas, árvore, os sliders da Silences). testes: três na galeria
(`tests/component_pages/keyboard.rs`), um no ide, um no urnaui. sem
agente.

pendência: o formulário de commit do ide não anda com as setas (o
keymap dele liga só escape, enter e backspace).
