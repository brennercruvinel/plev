---
id: task-74/phase-5-emoji
title: "phase 5: color emoji"
project: plev
type: section
parent: task-74
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [text, fonts, rendering]
relations: []
evidence: [a5457d9d, 2460a0ea, 17f8393d, 787291fb]
source: docs/log/steps/task-74-text-and-editing.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 5: color emoji

Draws color emoji on the embedded path with Noto Color Emoji v2.051 in CBDT, a subset of 640 glyphs. The raster now accepts swash's color bitmap, and the R8 atlas keeps RGBA in four texels per pixel, with a negative `u` as a marker for the shader to draw the glyph's color. The audit measures color pixels on each emoji line and closes the criterion item; emoji outside the subset remain tofu on the embedded path.

## fase 5: a sonda

data: 2026-09-25. branch `task/74d-emoji` sobre `afb19a4c`. as linhas da
biblioteca são do fonte travado no Cargo.lock: cosmic-text 0.18.2 e swash
0.2.8, no registro do cargo. a fonte é Noto Color Emoji v2.051 em bitmap
CBDT/CBLC (`target/fonts-noto/NotoColorEmoji-CBDT.ttf`, fora do repo),
lida com fontTools 4.64.0.

### (a) como o raster pede a imagem ao swash, e o que muda

`rasterize_and_upload` pede a imagem com `SwashCache::get_image_uncached`
(`rust/engine/src/drawing/text/atlas.rs:209`) e descarta tudo que não é máscara
(`atlas.rs:211`, `image.content != SwashContent::Mask` devolve `None`: o
glifo reserva o avanço e não desenha). o cosmic-text já pede as três
fontes na ordem certa, `ColorOutline(0)`, `ColorBitmap(BestFit)` e
`Outline`, com `Format::Alpha` (`cosmic-text-0.18.2/src/swash.rs:56-65`).
o swash acha o bitmap de cor (`swash-0.2.8/src/scale/mod.rs:971-977`),
decodifica o PNG do CBDT em RGBA de alfa reto (4 canais,
`strike.rs:310-314`) e marca `Content::Color` (`mod.rs:760-763`). então o
que muda é só do nosso lado: aceitar `Content::Color` e subir os 4 bytes
por pixel em vez de 1. nenhuma chamada nova à biblioteca.

### (b) o atlas e o shader

o atlas é uma textura `R8Unorm` (`atlas.rs:28`), ligada no grupo 1 por um
layout de uma textura e um sampler (`gpu/utils.rs:39-50`, criado em
`gpu/context.rs:346`, fora de `text/`). a cor tem três caminhos:

- uma segunda textura RGBA: pede um segundo binding no layout e um segundo
  draw ou bind group, em `gpu/context.rs` e `window/render_passes.rs`, fora
  dos arquivos da fase.
- o atlas inteiro em `Rgba8Unorm`: cabe no mesmo layout, mas quadruplica a
  memória de todo texto (até 64 MB a 4096²) para um punhado de emoji, e
  mexe no formato que os glifos de máscara usam hoje.
- o RGBA do glifo de cor guardado no mesmo atlas R8, quatro texels por
  pixel (r, g, b, a lado a lado na linha): o PNG decodificado já vem nessa
  ordem, então a subida é uma cópia de bytes com largura 4w. o glifo de
  máscara não muda em nada (formato, subida, sampler, conta do shader).

escolho o terceiro. o shader lê o glifo de cor com `textureLoad` (quatro
leituras, sem filtro), o que funciona porque o quad cai em pixel físico
inteiro (`glyph_quads_land_on_whole_physical_pixels`, `tests_raster.rs:213`)
e o bitmap já vem na escala física (c). o `TextVertex` não ganha campo:
`compositor/tests.rs:447` e `:1367` montam o vértice por literal, fora dos
arquivos da fase. a marca de cor vai no `u`: negativo no glifo de cor,
nunca negativo no de máscara. a cor é a do glifo, em sRGB linearizado como
a do texto; da cor do texto só entra o alfa (opacidade), nunca o rgb. o
`textureSample` da máscara continua fora de qualquer `if` (o WGSL exige
controle uniforme para ele); o `textureLoad` não exige.

### (c) o tamanho do bitmap e a escala

o CBLC tem uma strike só, de 109 ppem e 32 bits (bitmaps de 136x128). o
cosmic-text pede o tamanho físico: a chave carrega `font_size * scale`
(`LayoutGlyph::physical`) e o scaler é montado com esse tamanho
(`swash.rs:31-33`). o swash escolhe a strike mais próxima
(`scale/mod.rs:688-694`), e, se o tamanho difere, reduz o bitmap com filtro
Mitchell para `size / ppem` e escala o placement junto, truncando
(`mod.rs:705-735`). a 16 px e escala 2 o emoji sai em 39x37 pixels
físicos. o offset de subpixel não entra no bitmap (só em contorno), então
bins diferentes dão chaves diferentes com a mesma imagem: gasta atlas, não
quebra a identidade.

### (d) ZWJ, tom de pele e VS16

a fonte faz tudo por GSUB, feature `ccmp`: a lookup 0 é de ligaduras
(4.164), com a família U+1F468 ZWJ U+1F469 ZWJ U+1F467 ZWJ U+1F466 e o
polegar com tom (U+1F44D U+1F3FD) cada um num glifo só; as lookups 1 a 5
são das bandeiras e das tags. o VS16 (U+FE0F) não está no cmap, só no
formato 14; o shaper o esconde como default ignorable. o cosmic-text molda
a palavra inteira com a face de fallback e troca os clusters que faltavam
(`shape.rs:318-400`): a ligadura começa no primeiro emoji, que faltava na
face latina, e leva junto os ZWJ que a face latina escondeu.

qual face chega primeiro: `FontMatchKey` ordena por `not_emoji` antes do
peso (`font/system.rs:18-33`, a face cujo nome PostScript tem "Emoji"),
e o último passo do fallback anda nessa ordem (`font/fallback/mod.rs:463-476`).
então, no caminho embutido, a face de emoji é a primeira do último passo,
antes das Noto Sans, em qualquer peso. consequência: o coração (U+2764)
em Inclusive Sans vai para o emoji; em Inter não, porque Inter tem o
coração e ele nunca falta. o cosmic-text não lê o VS16 para escolher face,
então em Inter o `U+2764 U+FE0F` fica coração de texto. o subset não põe
no cmap nada que as faces latinas tenham (dígitos, `#`, `*`, espaço, (c),
(r), TM): esses nunca chegariam à face, e o espaço dela tem avanço de
1,245 em.

## handoff da fase 5: emoji colorido

fecha, no caminho embutido, o item "emoji colorido com pixel medido" do
critério de aceite. a fonte é Noto Color Emoji v2.051 em CBDT; o caminho
de cor mora no mesmo atlas R8.

### commits

- `a5457d9d` a sonda
- `2460a0ea` o subset (CBDT, 640 glifos) e a licença
- `17f8393d` o caminho de cor no atlas
- `787291fb` a face registrada e o audit com emoji
- o commit desta seção: a ADR e o handoff

### a sonda, curta

o swash do lock entrega o bitmap de cor (`Content::Color`, RGBA de alfa
reto) e já o reduz para o tamanho físico com filtro Mitchell; o raster só
jogava fora tudo que não era máscara. a família ZWJ e o tom de pele são
ligaduras da `ccmp` da fonte, e o fallback do cosmic-text molda a palavra
inteira, então saem num glifo só. a face de emoji é a primeira do último
passo do fallback em qualquer peso (`FontMatchKey::not_emoji`).

### o subset

`tool/tasks/subsetter.py` ganhou o emoji: conjunto fixo escrito no script
(bloco Emoticons, os rostos do bloco suplementar, 197 símbolos comuns, os
cinco tons sozinhos e em 45 mãos e pessoas, 15 famílias e 30 profissões
em ZWJ). 369 code points e 270 sequências, 640 glifos, 1.421.996 bytes
(fonte de 10.673.480). sha256 da fonte conferido; `--check` refaz os 15
arquivos byte a byte e bate. o script falha se o cmap do emoji tiver algo
de outra face embutida, fora o coração (U+2764, que Inter tem) e o ZWJ.
licença em `LICENSE-NotoColorEmoji-OFL.txt`. fontes embutidas: 8.357.960
bytes em 22 arquivos (eram 6.935.964).

### o caminho de cor

o atlas continua uma textura `R8Unorm`. o glifo de cor guarda o RGBA
como quatro texels por pixel, começando num x múltiplo de 4, com um pixel
inteiro de margem vazia de cada lado; o quad leva o `u` negativo, e o
`text.wgsl` lê os quatro texels com `textureLoad` e desenha a cor do
próprio glifo (do texto só entra o alfa). o glifo de máscara não mudou:
mesmo formato, mesma subida, mesmo sampler, mesma conta. o `TextVertex`
não ganhou campo. 3 testes novos em `tests_raster.rs` (a margem da
máscara igual, o alinhamento e a margem do glifo de cor, o `u` negativo).

### o audit

`10-emoji` entrou nos `DECIDED`: 0 notdef nas quatro linhas de emoji, nas
duas famílias e nos pesos 400, 500, 600 e 700, pelo caminho embutido. o
pixel: o `probe` ganhou `Rendered::color_pixels`, os pixels que a tinta
branca do teste não consegue fazer sobre o cinza (com matiz, ou mais
escuros que o fundo). toda linha que a face de emoji desenha tem que
deixar pelo menos 150; toda outra linha, zero. medido a 2x: família 374
(é o desenho da Noto 2.051, silhuetas num quadrado cinza, então conta
pelo escuro), polegar com tom 599, sorriso 921, coração 819; Inter com o
coração, 0 (é o coração de texto de Inter, ver pendências). o golden
mudou só nas oito linhas de emoji, nas quatro do detalhe e no total de
bytes do cabeçalho.

### o PNG

`/Users/nn/Dev/hoff/plevdev/target/agents/one/text-audit-74d/Inclusive-Sans-embedded.png`
(as linhas de emoji são as quatro a partir da 14ª faixa) e
`Inter-embedded.png` na mesma pasta, com `pixels.txt`.

### gate e regress

`./tool/tasks/fullcheck.sh` verde com o ambiente do briefing: `cargo test --workspace`
3136 passed, 0 failed, 27 ignored (o diff só acrescenta 3 testes e não
tira nenhum); clippy, fmt e wasm verdes; perna urnaui 85 passed, 0
failed, 1 ignored. `regress -- check` depois de tocar
`tool/bench/regress/src/main.rs`: 176 unchanged, 0 changed.
fora do gate: `cargo check -p engine --target aarch64-apple-ios-sim` com
`-D warnings` verde; `cargo ndk -t arm64-v8a check -p uigallery --lib`
compila, com os dois avisos antigos de código morto. disco: 12 GB livres.

### pendências

- em Inter, `U+2764 U+FE0F` fica coração de texto: Inter tem o coração e
  o cosmic-text 0.18 não lê o VS16 para escolher face. tirar o U+2764 de
  Inter ou um `Fallback` próprio resolve; os dois são decisão.
- fora do subset (tofu no caminho embutido): keycaps (o dígito vem da
  face latina e a ligadura não forma nem com ele no subset), bandeiras,
  sequências com tag, variantes com gênero (U+2640, U+2642), profissões
  e casais com tom. aumentar o conjunto é editar as listas do script,
  uns 2,2 KB por emoji.
- o glifo de cor é lido sem filtro: sob uma transformação de camada que
  escala (animação), o emoji sai no vizinho mais próximo; a máscara
  continua bilinear.
- o swash reduz o bitmap em alfa reto; uma franja escura na borda é
  possível e não foi medida.
- no desktop o mesmo caminho agora desenha a face de emoji do sistema
  (Apple Color Emoji, sbix, no macOS), que antes reservava o avanço sem
  tinta. o desktop e o embutido desenham emoji com faces diferentes,
  como já era com as escritas.

## fase 5 integrada (orquestrador, 2026-09-25)

fecha o item "emoji colorido com pixel medido": Noto Color Emoji em
bitmap (CBDT v2.051; o COLRv1 atual do Google Fonts o swash do lock não
desenha), subset de 640 glifos (1,4 MB) por `tool/tasks/subsetter.py`; o
atlas continua R8, com o emoji em quatro texels por pixel e o `u`
negativo como marca no shader, que desenha a cor do emoji e só a
opacidade do texto. o audit: 0 notdef nas linhas de emoji e pixel de cor
medido em cada uma (família 374, polegar com tom 599, sorriso 921,
coração 819; as outras linhas dão zero).

validação do orquestrador: diff lido (o glifo de máscara não muda, o
`TextVertex` não ganhou campo); o PNG aberto: família, polegar com tom,
sorriso e coração em cor, o resto do corpus como antes; 176 baselines
iguais.

continuam abertos no critério: o roteiro de IME executado (fase 4, pede
o brenner) e o benchmark de um milhão de linhas (fase 7).

o agente gastou uns 235 mil tokens.
