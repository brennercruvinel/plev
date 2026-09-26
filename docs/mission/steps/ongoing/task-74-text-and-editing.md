---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-25
domain: generative-runtime
task: task-74
wave: trilha
depends: []
depends-optional: [task-60, task-66]
branch: task/74-text
---

# task-74: texto e edição

## objetivo

texto no plev deixa de parecer bom em demo e quebrar no uso diário.
seleção, cursor, bidi, árabe, CJK, IME, emoji, ligatures, clipboard,
undo e redo, accessibility de texto, syntax highlighting, documentos
grandes e milhões de linhas quando o domínio exigir. text editing
absurdamente sólido.

## ligação com a tese

seção 20 (texto não é detalhe; GPUI e Zed passaram anos nisso; é onde
engines alternativas quebram). é a primeira das montanhas de maturidade.

## estado atual no código

- shaping por cosmic-text 0.18 (com harfrust), raster de glifo
  determinístico no atlas em escala física (ADR
  `glyph-raster-identity-and-atlas-isolation`), um `TextStyle` por run
  para medir e desenhar (guard test em
  `crates/engine/tests/text_style_guard.rs`).
- fontes embutidas: Inclusive Sans (300 a 700), Inter (400 a 600),
  codicons (`crates/engine/src/text/fonts.rs:38-52`). árabe, hebraico,
  CJK e emoji colorido não estão cobertos; o sistema avisa uma vez por
  fonte não embutida.
- `crates/rope`: documento, transações, seleção, multi-cursor, movimento,
  history (undo e redo), puro e testado (ADR
  `text-editing-core-follows-helix`).
- `crates/comps/src/editor` (`EditorView`, multi-cursor sobre o rope) e
  `crates/comps/src/form` (`TextField` sobre o núcleo `TextInput`).
- `crates/engine/src/platform/ime.rs`: `ImeState` básico (evento, teclado
  virtual no mobile).
- `engine::clipboard`: arboard no desktop, local no wasm e mobile.
- tree-sitter já é dependência (parser).

## escopo (cada fase é uma entrega separada, na ordem)

1. auditoria de cobertura: corpus por escrita (latim com diacríticos,
   árabe, hebraico, devanágari, CJK, emoji com ZWJ, marcas combinantes,
   RTL misturado com LTR), render headless (66) e medida: o que falta,
   o que sai errado. saída: relatório.
2. decisão de fontes com o brenner (ADR): embutir subsets Noto por escrita
   (custo de tamanho) ou fallback de sistema (quebra determinismo, task-77)
   ou os dois por configuração.
3. bidi no cursor e na seleção: movimento lógico e visual, seleção
   contínua em texto misto, no `TextInput` e no editor; testes no rope e no
   widget.
4. IME: preedit desenhado inline, commit, posição da janela de candidatos
   (`set_ime_cursor_area`), no `TextField` e no editor; roteiro manual no
   macOS com japonês e chinês, resultado registrado.
5. emoji colorido: caminho RGBA no atlas para fontes de cor (COLR, CBDT,
   sbix conforme a fonte escolhida).
6. ligatures: ligar e desligar por estilo.
7. documentos grandes: abrir, rolar e editar um arquivo de um milhão de
   linhas; shaping por linha com cache, layout virtualizado; benchmarks com
   números.
8. syntax highlighting incremental com tree-sitter no editor.
9. accessibility de texto (runs, seleção, caret para o leitor de tela),
   depois da task-60.

## fora de escopo

rich text e editor de documento formatado; spellcheck.

## dependências

independente para 1 a 8; 9 depende de 60; a auditoria usa 66 se integrada.

## superfície sob responsabilidade

`crates/engine/src/text/**` (menos `probe.rs` enquanto a 66 roda),
`crates/engine/src/platform/ime.rs`, `crates/rope/**`,
`crates/comps/src/editor/**`, `crates/comps/src/form/text_input*`,
`crates/engine/assets/fonts` (depois da decisão de fontes). ADRs por fase.

## interfaces que consome

cosmic-text, headless (66), accessibility (60, fase 9).

## interfaces que pode alterar

`TextSystem`, `TextMeasurer`, `TextInput`, `EditorView` por adição; mudança
de assinatura pública com aprovação. o contrato "um TextStyle para medir e
desenhar" não muda.

## critérios de aceitação

por fase, no próprio commit: corpus renderizado sem tofu nas escritas
decididas; testes de movimento bidi; roteiro de IME executado e
registrado; emoji colorido com pixel medido; benchmark de um milhão de
linhas com números; gate verde.

## testes esperados

goldens de shaping por escrita (largura medida, nunca estimada), testes de
cursor e seleção bidi no rope, testes de preedit, benchmarks.

## riscos e perguntas abertas

- tamanho do binário com fontes de várias escritas (wasm principalmente).
- IME é o item que mais depende de teste manual por plataforma.

## commits esperados

um grupo por fase, cada um começando com `task-74:` e o nome da fase.

## pare e reporte se

- a decisão de fontes (sempre do brenner);
- cosmic-text não suportar algo necessário e a saída for trocar de
  shaper (há ADR `parley-vs-cosmic-text`: reabrir é decisão, não
  implementação).

## relatório da fase 1: auditoria de cobertura por escrita

data: 2026-09-24. branch `task/74a-text-audit` sobre `97634db6`. só
medição: nenhuma correção de render, nenhuma fonte entrou ou saiu.

### commits

- `df270334` task-74: the text probe reports shaping per glyph
  (`crates/engine/src/text/probe.rs`: `Faces`, `GlyphReport`,
  `ShapeReport`, `shape_report`; a única mudança em `src/text/**`)
- `ad6d0d6a` task-74: text corpus, one phrase file per script
  (`crates/engine/tests/text_corpus/`, 11 arquivos e um README)
- `36ac4257` task-74: text audit test with the embedded-only golden
  (`crates/engine/tests/text_audit.rs`, `crates/engine/tests/text_audit.txt`)
- o commit deste relatório

### a sonda

(a) uma linha por escrita, 16 px, Inclusive Sans, `probe::shape_report`
para o shaping e `probe::render` (escala 2) para a tinta. "embutidas" é o
shaping só com as nove faces do repositório (o que wasm, android e ios
têm); "desktop" é o mesmo shaping com as fontes do sistema deste mac por
baixo (o que `TextSystem` e `TextMeasurer` fazem no desktop). a tinta é do
caminho desktop: não existe `TextSystem` sem fontes de sistema no desktop
(ver limitações).

| linha | embutidas: glifos / notdef / largura | desktop: glifos / notdef / largura / face que preencheu | tinta desktop (px, borda esq..dir) |
|---|---|---|---|
| ação, coração e pão: não é fácil, mas você já viu à vontade? | 60 / 0 / 457,4 | 60 / 0 / 457,4 / só Inclusive Sans | 5963, 1,0..456,5 |
| السلام عليكم، مرحبا بالعالم | 27 / 24 / 251,7 | 26 / 0 / 150,9 / Geeza Pro | 1684, 0,0..150,0 |
| שלום עולם | 9 / 8 / 83,9 | 9 / 0 / 66,2 / Arial | 964, 1,0..65,5 |
| नमस्ते दुनिया | 13 / 12 / 123,5 | 10 / 0 / 74,8 / Devanagari Sangam MN | 1217, 0,0..75,0 |
| 你好，世界 | 5 / 5 / 49,4 | 5 / 0 / 80,0 / Arial Unicode MS | 1228, 0,5..79,5 |
| こんにちは、世界 | 8 / 8 / 79,1 | 8 / 0 / 128,0 / Hiragino Sans e Arial Unicode MS | 1721, 2,5..127,5 |
| 안녕하세요, 세계 | 9 / 7 / 77,8 | 9 / 0 / 105,4 / Apple SD Gothic Neo | 1739, 0,5..104,0 |
| família com ZWJ (7 code points) | 7 / 4 / 39,6 | 1 / 0 / 16,0 / Apple Color Emoji | 0, sem tinta |
| é ão ö ño ệ decomposto (17 code points, e + U+0301 entre eles) | 11 / 0 / 84,4 | 11 / 0 / 84,4 / só Inclusive Sans | 1076, 1,0..83,5 |
| abc אבג def | 11 / 3 / 92,4 | 11 / 0 / 86,8 / Arial | 1193, 1,0..86,5 |

(b) o aviso de fonte não embutida sai em `crates/engine/src/text/atlas.rs`
(`emit_glyphs`), nível `warn`, alvo `engine::text::atlas`, texto "text
fallback: cluster ... rasterized from non-embedded face '...'" (a mensagem
carrega um travessão, herdado). sai só no caminho de render, nunca no
`TextMeasurer`. sai uma vez por face por `TextSystem`
(`warned_fallback_fonts`): a sonda renderizou o corpus quatro vezes (duas
famílias, duas rodadas), cada rodada com um `Renderer` novo, e contou 30
avisos = 8 faces na rodada Inclusive Sans (Geeza Pro, Arial, Devanagari
Sangam MN, Arial Unicode MS, Hiragino Sans, Apple SD Gothic Neo, Apple
Color Emoji, Menlo) + 7 na rodada Inter (Inter tem o coração U+2764, Menlo
não entra), vezes duas. o cosmic-text tem o aviso dele ("Failed to find
script fallback for [Han] locale 'en-BR', used 'Arial Unicode MS'") em
`debug`, porque a feature `warn_on_missing_glyphs` está desligada.

(c) determinismo: o teste renderiza cada folha duas vezes com o mesmo
corpus e compara os bytes RGBA (`assert_eq!(img.rgba, again.rgba)`), verde
para as duas famílias; o PNG sai dos mesmos bytes. o golden do shaping
embutido é o mesmo em toda rodada (fontes no repositório, shaper pinado
pelo Cargo.lock).

### corpus

`crates/engine/tests/text_corpus/`, um arquivo por escrita, cada um com
`# escrita`, `# cobre`, `# significado`, `# fonte da frase`, e uma frase
por linha, escrita à mão para este corpus, escapes `\u{}` para marca, ZWJ e
VS16 ficarem visíveis:

01 latim com diacríticos (pt), 02 latim ligatures e kerning ("office
waffle fjord affix flow", "AVATAR WAVY Type LT"), 03 marcas combinantes
decompostas (e+0301, a+0303, o+0308, n+0303, e+0302+0323), 04 árabe (formas
contextuais, lam-alef, vírgula árabe), 05 hebraico (duas frases, a segunda
com niqqud), 06 devanágari (नमस्ते दुनिया e क्षत्रिय), 07 chinês, 08 japonês
(kana + kanji, katakana), 09 coreano, 10 emoji (família ZWJ, tom de pele,
U+1F600, coração U+2764 + VS16), 11 bidi (abc אבג def, hello مرحبا 123 world).

### medição por shaping: o que falta e o que sai errado, por escrita

só faces embutidas, 16 px, wrap em 80 px, as duas famílias de texto
(Inclusive Sans e Inter; codicons é ícone e não entra). números completos,
com id e avanço de cada glifo, em `crates/engine/tests/text_audit.txt`.

| escrita | o que falta | o que sai errado (exemplo) |
|---|---|---|
| latim com diacríticos | nada | nada: 60 glifos, 0 notdef, 457,4 px, 7 linhas em 80 px |
| latim ligatures | feature `liga` nas duas faces (Inclusive Sans tem `ccmp`, `kern`, `mark`; Inter tem `calt`, `dlig`, `kern`, `mark`, `mkmk`, sem `liga`) | "office waffle fjord affix flow" fica 30 glifos para 30 caracteres nas duas: nenhuma ligadura se forma. a fase 6 (ligar e desligar ligatures) não tem o que ligar com estas fontes |
| latim kerning | nada | kerning funciona: "AVATAR WAVY Type LT" mede 163,0 contra 168,4 da soma dos glifos soltos em Inclusive Sans (-5,4 px), 175,4 contra 184,8 em Inter (-9,4 px) |
| marcas combinantes | nada nas combinações do corpus | 17 code points viram 11 glifos, 0 notdef: o shaper compõe para o glifo precomposto (é, ã, ö, ñ, ệ). não medido: marca empilhada sem forma precomposta (Inclusive Sans não tem `mkmk`) |
| árabe | fonte (0 glifos árabes nas duas faces) | 24 de 27 glifos são tofu (só os 3 espaços não); largura 251,7 px contra 150,9 real (tofu de 9,9 px por letra, letra real 4 a 7 px); 4 linhas em 80 px contra 2; sem lam-alef (27 glifos contra 26 com Geeza Pro). a ordem RTL do parágrafo está certa mesmo sem fonte (R1:0..51) |
| hebraico | fonte | 8 de 9 tofu; com niqqud, 7 de 7: as marcas viram tofu de avanço zero (`0/0.0`) e carimbam por cima da caixa anterior |
| devanágari | fonte | 12 de 13 tofu; nenhum conjunto se forma (13 glifos contra 10 com Devanagari Sangam MN), a vogal pré-base não reordena; क्षत्रिय 8 tofu contra 4 glifos reais |
| chinês | fonte | 5 de 5 tofu; 49,4 px contra 80,0 real (tofu mais estreito que o ideograma de 16 px) |
| japonês | fonte | 8 de 8 e 4 de 4 tofu; em Inter (84,0 px) こんにちは、世界 quebra em 80 px no meio da palavra (byte 21, entre 世 e 界), porque não há espaço e o wrap é `WordOrGlyph` |
| coreano | fonte | 7 de 9 tofu (vírgula e espaço vêm do latim) |
| emoji | fonte, e o caminho de cor no atlas (fase 5) | família ZWJ: 4 tofu de 9,9 px = 39,6 px, os 3 ZWJ ficam ocultos (glifo 1, avanço 0), não viram tofu; polegar com tom de pele (U+1F44D U+1F3FD) vira 2 tofu; coração U+2764 + VS16 acha o coração em Inter (fallback dentro das faces embutidas) e o VS16 fica oculto |
| bidi misto | fonte para a parte RTL | ordem visual certa nas duas frases: "abc אבג def" dá L0:0..4 R1:4..10 L0:10..14; "hello مرحبا 123 world" dá L0:0..6 L2:17..20 R1:6..17 L0:20..26 (os dígitos em nível 2 dentro do trecho RTL, como manda o UBA); o que erra é só o tofu (3 e 5 notdef) |

leitura geral: hoje, fora do desktop, toda escrita não latina é tofu. o
tofu de Inclusive Sans mede 9,9 px e o de Inter 10,5 px a 16 px, mais
largo que a letra árabe ou hebraica real e mais estreito que o ideograma,
então a largura medida (e o layout que se apoia nela) sai errada nos dois
sentidos. bidi, composição de marcas e kerning funcionam no shaper sem
depender de fonte.

### medição por pixel

`pixels_match_the_measured_width`: cada frase numa linha, as duas
famílias, escala 2, caminho do frame (`probe::render`, que é
`headless::Renderer`). tinta por faixa de 40 px lógicos comparada com a
largura de `TextMeasurer::measure_styled` para o mesmo estilo (a largura
do `shape_report` bate com a do `TextMeasurer` em todas as linhas).

- 32 de 38 linhas "ok": tinta dentro de [-1, largura medida + 1] px, borda
  esquerda entre 0,0 e 2,5 px (bearing), borda direita 0,5 a 3 px antes
  da largura (bearing do último glifo). katakana テキスト é a que mais fica
  aquém: 61,0 de 64,0 px.
- 6 linhas "largura medida sem tinta": os três emoji de cor (família ZWJ,
  polegar com tom de pele, U+1F600) nas duas famílias. o shaping acha Apple Color Emoji e reserva
  16,0 px de avanço, o swash devolve `SwashContent::Color` (sbix), e
  `rasterize_and_upload` em `atlas.rs` devolve `None` para tudo que não é
  `Mask`: o quad não sai. na tela isso é um buraco do tamanho de um emoji.
  o coração U+2764 + VS16 tem tinta porque é contorno (Menlo em Inclusive Sans, 173
  px; o próprio Inter, 439 px).
- nenhuma linha com tinta além da largura medida ou antes da origem.
- folhas em PNG: `Inclusive-Sans.png` e `Inter.png` (1400 x 1600 físicos)
  e o relatório `pixels.txt` e `desktop.txt` saem em `TEXT_AUDIT_OUT`
  (padrão: diretório temporário do sistema, `plev-text-audit`). nesta
  rodada ficaram no scratchpad da sessão
  (`.../scratchpad/text-audit/`); regenerar é um comando (ver abaixo).

o que as folhas mostram no desktop: tudo desenha, mas de fontes
diferentes por escrita, e o han vem de Arial Unicode MS e não de PingFang
SC porque o fontdb varre `/System/Library/Fonts`, `/Library/Fonts`,
`~/Library/Fonts` e `/System/Library/AssetsV2`, e neste macOS 26 o
PingFang mora em
`/System/Library/PrivateFrameworks/FontServices.framework/.../Reserved/PingFangUI.ttc`.
consequência visível: こんにちは、世界 mistura Hiragino Sans (kana) com
Arial Unicode MS (kanji) na mesma linha. o locale do desktop é o do
sistema (`en-BR` aqui) e decide a unificação han; o embutido usa `en-US`.

### tamanho das fontes embutidas e estimativa para a fase 2

hoje: 1.744.532 bytes (1,7 MB) em nove TTF: Inclusive Sans 5 pesos
(69.428 a 77.960 bytes cada, 372.656 no total), Inter 3 pesos (411.640 a
419.744, 1.248.684 no total), codicons 123.192. o `include_bytes!` entra
no binário e no wasm tal como está (sem woff2; a compressão do servidor
reduz um TTF em algo como 30 a 50%, a confirmar).

estimativa de subset Noto por escrita, um peso (Regular), a partir de
tamanhos que lembro das releases do Noto e do Google Fonts, sem baixar
nada: são ordens de grandeza, não medidas; a fase 2 confirma com
`pyftsubset` (fontTools 4.64 já está nesta máquina) sobre as fontes que
escolher.

| escrita | fonte | tamanho aproximado, um peso | confiança |
|---|---|---|---|
| árabe | Noto Sans Arabic | 150 a 200 KB (cobre formas, marcas, dígitos) | média |
| hebraico | Noto Sans Hebrew | 40 a 60 KB | média |
| devanágari | Noto Sans Devanagari | 200 a 250 KB | média |
| chinês simplificado | Noto Sans SC, subset dos 3.500 hanzi comuns | 1,2 a 2 MB; GB2312 inteiro (6.763) 2 a 3 MB; fonte inteira 10 a 16 MB | baixa |
| japonês | Noto Sans JP, kana + JIS nível 1 (2.965 kanji) | 1,5 a 2 MB; inteira 5 a 6 MB | baixa |
| coreano | Noto Sans KR, 2.350 sílabas KS X 1001 | 500 a 700 KB; 11.172 sílabas 3 a 4 MB | baixa |
| emoji | Noto Color Emoji (CBDT) inteira 10 a 11 MB; COLRv1 8 a 9 MB; subset de 300 emoji com as sequências deles, em COLR | 0,5 a 1 MB | baixa; depende da fase 5 (caminho de cor) para valer alguma coisa |
| grego e cirílico | já estão em Inter (105 e 248 code points); Inclusive Sans não tem | 0 | alta |

somando árabe, hebraico e devanágari em um peso: cerca de 0,4 a 0,5 MB,
um quarto do que já embutimos. cada escrita CJK custa mais que tudo o
que está embutido hoje, mesmo em subset. cada peso a mais multiplica.

### gate e contagem

`./script/gate` verde na branch (`36ac4257`), com o ambiente de build do
briefing (`CARGO_TARGET_DIR=target/agents/task-74`), nesta máquina (metal,
macOS 26.6.2): `cargo test --workspace`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --check`, `cargo check --target
wasm32-unknown-unknown -p showcase` e a perna urnaui (test, clippy, fmt,
check wasm), todas verdes. o CI do github está fora; esta rodada é a
verificação. `text_audit` rodou no gate com GPU
(`pixels_match_the_measured_width ... ok`), sem `#[ignore]`.

contagem de `cargo test --workspace` (soma dos `test result:`):

- antes, na base `97634db6`: 2451 passed, 0 failed, 16 ignored, em 113
  binários de teste
- depois, na branch: 2454 passed, 0 failed, 16 ignored, em 114 binários
  (+3 testes de `text_audit`, +1 binário; nenhum teste saiu)
- a perna urnaui somou 82 passed, 0 failed, 1 ignored em 6 binários (só
  na rodada de depois; a de antes foi `cargo test --workspace` puro).
  total do gate: 2536 passed, 0 failed, 17 ignored.

disco: 16 GB livres antes, 15 GB depois (piso de 8 GB respeitado).

### o que a fase 2 (decisão de fontes) precisa saber

1. o caminho embutido (wasm, android, ios) mostra tofu em toda escrita
   não latina; o desktop esconde isso com fontes do sistema, por face e
   por máquina, e o fontdb não acha todas (PingFang). são dois produtos
   diferentes hoje.
2. o tofu tem largura própria (9,9 px em Inclusive Sans, 10,5 em Inter, a
   16 px): a medida de layout já erra antes do pixel, e uma marca sem
   glifo tem avanço zero e carimba a caixa anterior.
3. bidi, composição de marcas e kerning não dependem da decisão de
   fontes: o shaper resolve; o que falta para RTL é só glifo.
4. emoji precisa de duas coisas, não uma: uma fonte com emoji e o caminho
   de cor no atlas (fase 5). embutir uma fonte de emoji antes da fase 5 dá
   avanço sem tinta, o buraco que a sonda mediu no desktop.
5. ligatures (fase 6): nenhuma das faces embutidas tem `liga`; ou a fase 6
   escolhe outra face latina, ou aceita que não há o que ligar.
6. determinismo desktop x wasm pede um `TextSystem` (e um `TextMeasurer`)
   que possam nascer sem fontes de sistema no desktop; hoje é cfg de
   plataforma em `system.rs` e `measure.rs`, sem chave em tempo de
   execução. sem isso a auditoria de pixel do caminho embutido não roda
   nesta máquina.
7. locale entra na escolha da face CJK (unificação han) e vem do sistema
   no desktop e fixo `en-US` no embutido.

### limitações e dívidas declaradas

- a medição de pixel é do caminho desktop; a de shaping cobre os dois. o
  tofu em pixel (quantos px de tinta uma caixa `.notdef` deixa) não foi
  renderizado, pelo item 6 acima. o teste que existe não fica `#[ignore]`:
  ele pula sem adaptador, como os vizinhos `text_raster_pixels` e
  `text_drawn_extent`, e rodou de verdade nesta máquina (metal).
- `shape_report` repete a montagem de `Attrs` que `measure.rs` (`attrs_for`)
  e `system.rs` (`ensure_shaped`) já fazem, porque só `probe.rs` podia
  mudar. são três cópias; uma `attrs_for` `pub(super)` em `measure.rs`
  fecha isso (fora do escopo desta fase, registrado).
- o aviso "text fallback" em `atlas.rs` leva um travessão no texto
  (arquivo fora do escopo).
- o corpus não tem marca empilhada sem forma precomposta, nem kinsoku
  (regras de quebra CJK), nem números árabe-índicos.
- a estimativa Noto é de memória; nenhuma fonte foi baixada.

### como reproduzir

```sh
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/task-74
cargo test -p engine --test text_audit                      # golden + pixels (pula sem gpu)
TEXT_AUDIT_OUT=/tmp/text-audit cargo test -p engine --test text_audit -- --nocapture
#   escreve Inclusive-Sans.png, Inter.png, pixels.txt e desktop.txt
TEXT_AUDIT_UPDATE=1 cargo test -p engine --test text_audit  # regrava o golden após mudança intencional
```

## fase 2: a sonda

data: 2026-09-25. branch `task/74b-noto` sobre `5dad7ec1`. decisão do
brenner: "Noto, amigo. Noto, sans". escritas: árabe, hebraico, devanágari,
chinês simplificado, japonês e coreano; emoji fica na fase 5. as linhas
citadas da biblioteca são do fonte travado no Cargo.lock: cosmic-text
0.18.2 e fontdb 0.23.0, no registro do cargo.

### (a) como o cosmic-text escolhe a face de fallback

por palavra. `shape_run` (cosmic-text `src/shape.rs:273`) junta as
escritas do trecho na ordem em que aparecem, sem `Common`, `Inherited`,
`Latin` e `Unknown` (`shape.rs:288-297`), abre um `FontFallbackIter`
(`shape.rs:306`), molda com a primeira face e, enquanto sobra cluster sem
glifo, molda o trecho inteiro com a próxima face e copia só os clusters
que ela cobre (`shape.rs:326-400`). a ordem das faces, em
`FontFallbackIter::next_item` (`src/font/fallback/mod.rs`):

1. a família pedida pelo estilo (`mod.rs:305`), só com peso exato
   (`default_font_match_key`, `mod.rs:279`);
2. a lista por escrita do `Fallback` (`mod.rs:419`), só faces de peso
   exato (o filtro `font_weight_diff == 0`, `mod.rs:302`);
3. a lista comum do `Fallback` (`mod.rs:446`), também de peso exato;
4. todas as faces do banco, menos as proibidas (`mod.rs:462-463`), sem
   filtro de peso, na ordem de `FontMatchKey`: derivada dos campos
   `not_emoji`, `font_weight_diff`, `font_stretch_diff`,
   `font_style_diff`, `font_weight`, `font_stretch`, `id`
   (`src/font/system.rs:18-27`, ordenada em `system.rs:370`, com a face
   da consulta levada para a frente em `system.rs:380-394`). o `id` do
   fontdb é chave de slotmap (`fontdb src/lib.rs:92-95`), crescente na
   ordem de carga num banco novo.

qual `Fallback` cada caminho usa: o motor constrói o caminho embutido com
`FontSystem::new_with_locale_and_db("en-US", Database::new())`
(`crates/engine/src/text/system.rs:91-95`, `measure.rs:121-125`), que é
`PlatformFallback` do alvo (`system.rs:266-267` do cosmic-text). o alvo
escolhe a lista em `fallback/mod.rs:12-26`: wasm e android caem em
`other.rs`, listas vazias (`other.rs:30-42`); ios é `unix` e cai em
`unix.rs`, que nomeia "Noto Sans Arabic", "Noto Sans Devanagari" e "Noto
Sans Hebrew" (`unix.rs:76`, `:88`, `:98`) e, para han, kana e hangul,
"Noto Sans CJK SC/JP/KR" (`unix.rs:56-68`, `:95-102`). o desktop usa
`FontSystem::new()` com `macos.rs`, que nomeia fontes do sistema
(`macos.rs:61-109`).

o que faz o embutido achar as faces Noto sem as fontes do sistema: o
passo 4. em wasm e android as listas são vazias, então toda escrita sem
glifo na face pedida chega ao passo 4, que percorre todas as faces do
banco e pega a primeira que cobre o cluster; como as faces Noto são as
únicas com glifo dessas escritas, é nelas que o cluster cai, em qualquer
peso (o passo 4 não filtra peso, e a ordem por `font_weight_diff` faz 500
cair na Regular e 600 na Bold). no ios, árabe, hebraico e devanágari
batem no passo 2 pelo nome, em 400 e 700; os CJK não batem (a família do
subset é "Noto Sans SC", não "Noto Sans CJK SC") e vão ao passo 4. o
resultado é o mesmo nos três alvos: a única face que cobre, ou, para o
han, a primeira na ordem de carga (item b). nenhuma mudança na
biblioteca, nenhum `Fallback` próprio.

um `Fallback` próprio do embutido (kana para JP, hangul para KR, han pelo
locale) não compila sem pacote: o método `script_fallback` recebe
`unicode_script::Script` (`fallback/mod.rs:9`, `:76`), o cosmic-text não
reexporta esse tipo, e o engine não depende de `unicode-script`. o
pacote já está no Cargo.lock (0.5.8), mas pô-lo no `Cargo.toml` do engine
está fora do que esta rodada pode tocar. fica como pendência (ver
handoff).

### (b) a ordem das faces CJK para o han unificado

no embutido, o han não acha nome em lista nenhuma (vazias em wasm e
android; no ios os nomes "Noto Sans CJK *" não batem) e cai no passo 4,
onde as faces Noto Regular empatam em tudo menos no `id`. então a ordem é
a de carga em `register_embedded_fonts`: SC, depois JP, depois KR. SC vem
primeiro porque é o que o próprio cosmic-text faz com `en-US`: as tabelas
de unificação han dele mandam qualquer locale que não seja `ja`, `ko`,
`zh-HK` ou `zh-TW` para o chinês simplificado (`unix.rs:56-68`,
`macos.rs:45-57`), e o locale embutido é `en-US`. o subset KR não tem
hanja, então para o han ele nunca entra.

consequência medida: uma palavra chinesa sai inteira em SC; uma palavra
japonesa sai com kanji e pontuação CJK em SC e kana em JP, porque SC é
avaliada antes e cobre o que tem. é o mesmo que o cosmic-text faz no
desktop com `en-US` quando a palavra começa por kanji ("世界は" dá
Arial Unicode MS e Hiragino Sans aqui, abaixo). o conserto é o
`Fallback` próprio do item (a).

### (c) o que o desktop passa a mostrar

a fonte do sistema, como antes. `macos.rs` nomeia uma face do sistema
por escrita (Geeza Pro, Arial, Devanagari Sangam MN, Hiragino Sans, Apple
SD Gothic Neo; PingFang SC para han, que o fontdb não acha neste macOS e
cai na lista comum, Arial Unicode MS), e o passo 2 vem antes do 4. em 500
e 600 os passos 2 e 3 não acham peso exato e o passo 4 decide pela
distância de peso e depois pelo `id`; as faces do sistema carregam antes
das embutidas (`id` menor), então empatadas elas ganham. as faces por
peso, antes desta fase, neste mac (Inclusive Sans, `probe::shape_report`,
`Faces::Desktop`):

| linha | 400 | 500 | 600 | 700 |
|---|---|---|---|---|
| السلام عليكم... | Geeza Pro | Damascus | Damascus | Geeza Pro |
| שלום עולם | Arial | Lucida Grande | Lucida Grande | Arial |
| नमस्ते दुनिया | Devanagari Sangam MN | Kohinoor Devanagari | Kohinoor Devanagari | Devanagari Sangam MN |
| 你好，世界 | Arial Unicode MS | Hiragino Sans e Apple SD Gothic Neo | Hiragino Sans GB e Apple SD Gothic Neo | Songti SC e Apple SD Gothic Neo |
| こんにちは、世界 | Hiragino Sans e Arial Unicode MS | Hiragino Sans e Apple SD Gothic Neo | idem | idem |
| 안녕하세요, 세계 | Apple SD Gothic Neo | idem | idem | idem |

o chinês em 700 sai em Songti SC, uma serifada; é o desktop de hoje, e
esta fase não mexe nele (fonte de sistema por configuração está fora do
escopo). o regress: nenhuma página do showcase tem texto dessas escritas
(só testes de `rope`, `sem` e do editor de `comps` têm), então nenhuma
das 176 baselines passa por esse caminho; a rodada confirma com o
`regress check` no handoff.

### (d) se o `text_audit` mede o caminho embutido nesta máquina

o shaping, sim: `probe::shape_report` com `Faces::Embedded` já monta um
`FontSystem` só com as faces do repositório (`probe.rs:336-347`). o
pixel, não: `probe::render` passa pelo `headless::Renderer`
(`probe.rs:207`), que cria o `TextSystem` dele
(`crates/engine/src/headless/mod.rs:157`) e o guarda privado, e
`TextSystem::new` escolhe as fontes do sistema por `cfg` no desktop
(`system.rs:84-95`). falta uma chave em tempo de execução. o que esta
rodada faz, só em `src/text`: um construtor único do `FontSystem` em
`fonts.rs`, usado pelo `TextSystem`, pelo `TextMeasurer` e pela sonda,
com uma chave por thread que a sonda liga em volta do `Renderer::new`
(`probe::render_with(Faces::Embedded, ...)`). fora do `cfg` do desktop a
chave não existe; nada fora de `src/text` muda.

## handoff da fase 2: Noto Sans embutida

o critério "corpus renderizado sem tofu nas escritas decididas", com gate
verde, fechou para árabe, hebraico, devanágari, chinês simplificado,
japonês e coreano. a ADR é `docs/ADR/embedded-fonts-by-script.md`
(proposed).

### commits

- `3882aedd` task-74: phase 2 probe, how the embedded path finds a face per script
- `a0f7a0b6` task-74: script/font-subset and the Noto subsets for six scripts, with the OFL
- `fc304299` task-74: register the Noto faces, one font-system constructor, and the probe's embedded render
- `caad36f2` task-74: the text audit holds the six scripts to zero tofu and draws the embedded path
- o commit da ADR e deste handoff

### a sonda, curta

(a) o embutido acha as faces Noto no último passo do fallback do
cosmic-text (todas as faces, por distância de peso e depois ordem de
carga), sem nome de família e sem mudar a biblioteca; 500 cai na Regular,
600 na Bold. (b) o han vai na ordem de carga: SC, JP, KR, porque com
`en-US` o próprio cosmic-text escolhe o chinês simplificado; palavra
japonesa sai com kanji de SC e kana de JP. (c) o desktop não muda: as
listas do macOS acham a fonte do sistema antes; as faces por escrita e
peso (400 a 700) saíram iguais antes e depois nas oito linhas medidas.
(d) o pixel do embutido agora roda nesta máquina, por
`probe::render_with(Faces::Embedded, ...)`.

### arquivos novos

| arquivo | bytes |
|---|---|
| `crates/engine/assets/fonts/NotoSansArabic-Regular.ttf` | 217.320 |
| `crates/engine/assets/fonts/NotoSansArabic-Bold.ttf` | 243.820 |
| `crates/engine/assets/fonts/NotoSansHebrew-Regular.ttf` | 25.204 |
| `crates/engine/assets/fonts/NotoSansHebrew-Bold.ttf` | 25.204 |
| `crates/engine/assets/fonts/NotoSansDevanagari-Regular.ttf` | 225.236 |
| `crates/engine/assets/fonts/NotoSansDevanagari-Bold.ttf` | 231.752 |
| `crates/engine/assets/fonts/NotoSansSC-Regular.otf` | 946.616 |
| `crates/engine/assets/fonts/NotoSansSC-Bold.otf` | 956.892 |
| `crates/engine/assets/fonts/NotoSansJP-Regular.otf` | 794.888 |
| `crates/engine/assets/fonts/NotoSansJP-Bold.otf` | 813.168 |
| `crates/engine/assets/fonts/NotoSansKR-Regular.otf` | 350.204 |
| `crates/engine/assets/fonts/NotoSansKR-Bold.otf` | 361.128 |
| `crates/engine/assets/fonts/LICENSE-Noto-OFL.txt` | 4.823 |
| `script/font-subset` | 9.926 |
| `docs/ADR/embedded-fonts-by-script.md` | a ADR |

fontes embutidas: antes 1.744.532 bytes (834.203 com gzip -9), depois
6.935.964 bytes (4.873.679 com gzip -9). nenhum arquivo passa de 1 MB.
`script/font-subset --src target/fonts-noto --check` refaz tudo num
diretório temporário e compara byte a byte: 13 arquivos iguais.

### o audit, notdef antes e depois (caminho embutido, 16 px, 400)

| linha | Inclusive Sans | Inter |
|---|---|---|
| árabe | 24 para 0 | 24 para 0 |
| hebraico | 8 para 0 | 8 para 0 |
| hebraico com niqqud | 7 para 0 | 7 para 0 |
| devanágari नमस्ते दुनिया | 12 para 0 | 12 para 0 |
| devanágari क्षत्रिय | 8 para 0 | 8 para 0 |
| chinês | 5 para 0 | 5 para 0 |
| japonês こんにちは、世界 | 8 para 0 | 8 para 0 |
| japonês テキスト | 4 para 0 | 4 para 0 |
| coreano | 7 para 0 | 7 para 0 |
| bidi abc אבג def | 3 para 0 | 3 para 0 |
| bidi hello مرحبا 123 world | 5 para 0 | 5 para 0 |

o teste novo `decided_scripts_have_no_tofu` segura esses zeros nas duas
famílias e em 400, 500, 600 e 700. o golden (`text_audit.txt`) mudou só no
cabeçalho (os bytes) e nas linhas dessas escritas; latim, marcas
combinantes, emoji e a tabela de kerning ficaram iguais. o devanágari agora
forma os conjuntos (क्षत्रिय, 8 caracteres, 4 glifos) e o árabe mede 167,4
px em vez de 251,7 de tofu. emoji continua tofu no embutido (fase 5).

pixel: as quatro folhas saem em
`/Users/nn/Dev/hoff/plevdev/target/agents/one/text-audit-74b/`; a do
embutido é `Inclusive-Sans-embedded.png` (e `Inter-embedded.png`). as 38
linhas do embutido ficam "ok" (tinta dentro da largura medida), e toda
linha das escritas decididas tem tinta; o teste afirma as duas coisas.

### gate e regress

`./script/gate` verde com o ambiente do briefing: `cargo test --workspace`
3079 passed, 0 failed, 27 ignored (a base tinha um teste a menos: este
trabalho soma `decided_scripts_have_no_tofu` e não tira nenhum); clippy,
fmt e wasm verdes; perna urnaui 85 passed, 0 failed, 1 ignored.
`regress -- check`: 176 unchanged, 0 changed. fora do gate:
`cargo check -p engine --target aarch64-apple-ios-sim` com `-D warnings`
verde; `cargo ndk -t arm64-v8a check -p showcase --lib` compila, com dois
avisos de código morto que já existiam e ficam fora do escopo
(`crates/engine/src/window/mod.rs:57`, `crates/showcase/src/app.rs:407`).
disco: 13 GB livres antes do gate, 11 GB no fim.

### linha para o índice de ADR

```
| 2026-09-25 | [embedded-fonts-by-script](embedded-fonts-by-script.md) | proposed | Noto Sans embutida por escrita (decisão do brenner): árabe, hebraico, devanágari, SC, JP e KR em Regular e Bold, subsets reprodutíveis por script/font-subset (sha256 das fontes, conjuntos pelos codecs da stdlib); de 1,7 MB para 6,9 MB embutidos; o fallback acha as faces no último passo do cosmic-text, han na ordem SC, JP, KR; o desktop segue com as fontes do sistema; emoji fica para a fase 5 |
```

### pendências

- palavra japonesa sai com kanji de SC. o conserto é um `Fallback` próprio
  do embutido (kana para JP, hangul para KR, han pelo locale), que precisa
  de `unicode-script` como dependência direta do engine: o pacote já está
  no Cargo.lock (0.5.8), mas é mudança no `Cargo.toml`, decisão do
  orquestrador.
- o desktop desenha essas escritas com fontes do sistema, diferentes das
  do embutido (e o chinês em 700 sai em Songti SC, serifada). preferir Noto
  no desktop é uma decisão à parte.
- a cobertura é um conjunto: hanzi fora do GB2312 nível 1, kanji fora do
  JIS nível 1 e os jamo de compatibilidade do coreano (ㅋ, U+3131 a U+318E)
  não estão; alargar é editar `script/font-subset` e rodar de novo.
- o wasm cresce uns 5,2 MB crus (uns 4 MB com gzip); não medi o wasm de
  release (o briefing proíbe `--release`).
- herdadas da fase 1 e ainda abertas: o travessão no aviso de fallback de
  `atlas.rs` e as três cópias da montagem de `Attrs` (agora a sonda e o
  medidor dividem o construtor do `FontSystem`, mas não o de `Attrs`).
- os outros itens do critério de aceitação (bidi no cursor, IME, emoji,
  um milhão de linhas) são das fases 3, 4, 5 e 7.

## fase 2 integrada (orquestrador, 2026-09-25)

fecha o item "corpus renderizado sem tofu nas escritas decididas" do
critério de aceite. a decisão foi do brenner ("Noto, amigo. Noto, sans"):
árabe, hebraico, devanágari, chinês simplificado, japonês e coreano em
Noto Sans, Regular e Bold, subsets de `script/font-subset`.

validação do orquestrador: diff lido (as faces entram por último no
banco, emprestadas do binário sem cópia; `TextSystem`, `TextMeasurer` e a
sonda passam pelo mesmo construtor); o PNG do caminho embutido aberto: as
seis escritas desenham, o árabe ligado, o devanágari com os conjuntos, o
hebraico com niqqud; o emoji continua tofu, que é da fase 5. o desktop
não muda (as 176 baselines iguais).

os itens que continuam abertos no critério: testes de movimento bidi
(fase 3), roteiro de IME executado (fase 4, pede o brenner), emoji
colorido com pixel medido (fase 5), benchmark de um milhão de linhas
(fase 7). as fases 6, 8 e 9 não têm item no critério.

o agente gastou uns 250 mil tokens.

## fase 3: a sonda

data: 2026-09-25. branch `task/74c-bidi` sobre `13fed644`. as linhas da
biblioteca são do fonte travado no Cargo.lock: cosmic-text 0.18.2, no
registro do cargo. medida com Inclusive Sans a 16 px, sem wrap, pelas
faces embutidas (Noto Sans Hebrew e Arabic no fallback).

### (a) como o caret e a seleção são posicionados hoje, e onde quebram

o rope já anda por grafema e em ordem lógica: `move_left` e `move_right`
(`crates/rope/src/movement.rs:175-189`) vão ao grafema anterior e ao
seguinte (`movement.rs:43-78`, `grapheme_indices` por linha). em texto
misto isso está certo como movimento lógico; o que falta é o visual. o
editor liga as setas direto nessas funções (`crates/comps/src/editor/input.rs:60-75`),
então em trecho RTL a seta direita anda para a esquerda na tela.

o `TextBuffer` do `TextInput` anda por `char`, não por grafema
(`crates/comps/src/form/text_input/buffer.rs:98-110`, `153-167`), e não
tem seleção estendida: só `select_all` cria seleção.

o x do caret sai de `cursor_rect_in` (`crates/engine/src/text/measure.rs:179`),
usado por `TextMeasurer::cursor_x_styled` no `TextField`
(`crates/comps/src/form/text_field.rs:365`) e no editor
(`crates/comps/src/editor/view.rs:142`). ele devolve `glyph.x`, a borda
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
  `ClusterBox` (`crates/engine/src/text/measure.rs`): os clusters de cada
  linha da esquerda para a direita, com bytes, caixa, nível e direção do
  parágrafo. nada do que existia mudou.
- `rope`: só docs e testes. `move_left` e `move_right` passam a dizer que
  são lógicos (grafema anterior e seguinte na ordem do texto).
- `comps`, `TextInput`: `crates/comps/src/form/text_input/visual.rs`
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

- rope, `crates/rope/src/tests_bidi.rs` (6): as duas frases e a frase RTL
  andam em ordem de texto nos dois sentidos; niqqud e harakat andam junto
  da letra; seleção que cruza a troca de direção é um intervalo lógico
  (e colapsa nas pontas lógicas); início, fim de linha e palavra em texto
  misto.
- engine, `crates/engine/src/text/tests_measure.rs` (3): clusters da
  esquerda para a direita com os níveis (`abc אבג def` e um parágrafo
  RTL); em latim cada cluster começa no x do caret de antes; marcas
  entram no cluster da letra.
- `TextInput` e `TextField`, `crates/comps/src/form/text_input/tests/tests_bidi.rs`
  (11): a seta direita percorre a tela nas três frases, a esquerda volta,
  o caret anda para a direita a cada toque e cada byte aparece uma vez;
  num trecho RTL a esquerda avança no texto; o caret nas fronteiras; o
  parágrafo RTL na borda direita da caixa; shift com setas e com Home e
  End; clique em cada parada volta a ela; latim bit a bit igual; o
  `TextField` desenha o caret da fronteira, as duas lavagens da seleção
  cruzada e o caret do parágrafo RTL.
- editor, `crates/comps/src/editor/tests_bidi.rs` (5): as setas percorrem
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
`TEXT_FIELD_BIDI_OUT=<dir> cargo test -p comps --lib the_field_draws_the_boundary_caret`.

### gate e regress

`./script/gate` verde com o ambiente do briefing: `cargo test --workspace`
3104 passed, 0 failed, 27 ignored (a base tinha 3079: são os 25 novos,
nenhum saiu); clippy, fmt e wasm verdes; perna urnaui 85 passed, 0
failed, 1 ignored. `regress -- check` depois de tocar
`crates/showcase/examples/regress/main.rs`: 176 unchanged, 0 changed.
fora do gate: `cargo check -p comps --target aarch64-apple-ios-sim` com
`-D warnings` verde; `cargo ndk -t arm64-v8a check -p showcase --lib`
compila, com os dois avisos antigos de código morto. disco: 11 GB livres.

### pendências

- os apps não mandam shift+seta para o campo: `crates/ide/src/keys.rs`,
  `crates/showcase/src/keys.rs` e o `keys.rs` do urnaui mapeiam só a
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

## fase 5: a sonda

data: 2026-09-25. branch `task/74d-emoji` sobre `afb19a4c`. as linhas da
biblioteca são do fonte travado no Cargo.lock: cosmic-text 0.18.2 e swash
0.2.8, no registro do cargo. a fonte é Noto Color Emoji v2.051 em bitmap
CBDT/CBLC (`target/fonts-noto/NotoColorEmoji-CBDT.ttf`, fora do repo),
lida com fontTools 4.64.0.

### (a) como o raster pede a imagem ao swash, e o que muda

`rasterize_and_upload` pede a imagem com `SwashCache::get_image_uncached`
(`crates/engine/src/text/atlas.rs:209`) e descarta tudo que não é máscara
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

`script/font-subset` ganhou o emoji: conjunto fixo escrito no script
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

`./script/gate` verde com o ambiente do briefing: `cargo test --workspace`
3136 passed, 0 failed, 27 ignored (o diff só acrescenta 3 testes e não
tira nenhum); clippy, fmt e wasm verdes; perna urnaui 85 passed, 0
failed, 1 ignored. `regress -- check` depois de tocar
`crates/showcase/examples/regress/main.rs`: 176 unchanged, 0 changed.
fora do gate: `cargo check -p engine --target aarch64-apple-ios-sim` com
`-D warnings` verde; `cargo ndk -t arm64-v8a check -p showcase --lib`
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
desenha), subset de 640 glifos (1,4 MB) por `script/font-subset`; o
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

## fase 7: a sonda

documentos grandes no editor: o que ele faz hoje com N linhas, onde entram
o layout virtualizado e o cache de shaping por linha, e o que o rope já dá.
os números vêm do benchmark desta fase (`crates/comps/tests/editor_large_bench.rs`,
comando no fim da seção), em release, num Apple M4 com 16 GB, macOS 26.6.2.

### (a) o que o editor faz por frame com N linhas

- já é virtualizado na emissão. `render` percorre só `visible_lines()`
  (`crates/comps/src/editor/view.rs:288`, laço em `:299-339`), que é a
  conta pura de `visible_line_range` (`view.rs:468-483`): as linhas que
  cortam a viewport mais `overscan_lines` (8) de cada lado. a altura total
  vem da contagem de linhas, sem medir nenhuma
  (`view.rs:98`, `len_lines() * line_height`).
- por linha visível: o texto sai do rope (`view.rs:134-140`,
  `rope.line(n)`, O(log n)); o número da margem é medido com
  `TextMeasurer::measure_styled` (`view.rs:304-305`), que tem cache LRU de
  2048 entradas por conteúdo (`crates/engine/src/text/measure.rs:24`,
  `:327-346`); o texto vira um nó `SceneNode::Text` com o conteúdo como
  chave.
- quem molda o texto desenhado é o `TextSystem`, na hora de montar o frame:
  `ensure_shaped` (`crates/engine/src/text/system.rs:253-289`) molda uma
  vez por chave, num mapa por conteúdo, e `finish_frame` (`system.rs:346-352`)
  joga fora o que o frame não usou. isso já é o shaping por linha com cache
  do desenho, limitado às linhas da tela.
- o que o editor molda de novo a cada frame, sem cache: a linha de cada
  caret na tela (`view.rs:343-360`, `caret_x` em `:145-147`) e cada linha
  visível com seleção (`view.rs:319-329`, `selection_boxes_on_line` em
  `:423-441`). as duas passam por `VisualLine::new`
  (`crates/comps/src/form/text_input/visual.rs:51-52`), que chama
  `TextMeasurer::clusters_styled` (`measure.rs:400-414`): esse molda toda
  vez, é o único caminho de shaping do measure sem cache.
- a geometria das linhas é f32 contra o topo do documento:
  `line_top` (`view.rs:129-131`) faz `line as f32 * lh - offset`,
  `hit_test_point` (`view.rs:150-163`) faz `y - bounds.y + offset`, e o
  `scroll_to_cursor` (`view.rs:166-178`) também. com a linha de 19,6 px do
  tema, um milhão de linhas dá 19,6 milhões de px, acima dos 16,7 milhões
  que o f32 guarda com passo de 1 px. medido com os testes novos
  (`editor/tests_large.rs`), antes da correção: o passo entre duas linhas
  sai 19,5 na linha 99.990 e 20 na linha 499.963, e um clique a 31,1 px do
  topo, no fim do documento, cai na linha 999.972 em vez da 999.971.

custo medido antes de mudar (média / p99, ms; `scene` para no
`Compositor::resolve_scene`, sem GPU; `frame` passa pelo
`headless::Renderer::draw`, que molda, rasteriza e submete como a janela):

| linhas | até | abrir | roda, 3 linhas por frame | arrasto da barra | roda, tudo selecionado | editar no meio | heap mantido (MiB) | pico ao abrir (MiB) |
|---|---|---|---|---|---|---|---|---|
| 10 mil | scene | 0,3 | 0,021 / 0,026 | 0,055 / 0,060 | 0,589 / 0,629 | 0,028 / 0,033 | 0,6 | 1,1 |
| 100 mil | scene | 2,0 | 0,021 / 0,025 | 0,088 / 0,099 | 0,590 / 0,636 | 0,030 / 0,034 | 6,2 | 11,8 |
| 1 milhão | scene | 22,8 | 0,021 / 0,025 | 0,092 / 0,101 | 0,590 / 0,628 | 0,029 / 0,034 | 64,2 | 121,3 |
| 10 mil | frame | 1,0 | 0,364 / 0,585 | 0,701 / 0,907 | 0,932 / 1,063 | 0,262 / 0,341 | 0,6 | 1,5 |
| 100 mil | frame | 3,1 | 0,357 / 0,573 | 1,007 / 1,119 | 0,919 / 1,001 | 0,280 / 0,393 | 6,2 | 11,8 |
| 1 milhão | frame | 23,3 | 0,357 / 0,544 | 1,194 / 2,884 | 0,946 / 2,026 | 0,311 / 1,071 | 64,2 | 121,3 |

o frame não depende de N: rolar e editar custam o mesmo com 10 mil e com
um milhão de linhas. só abrir cresce com o documento, e o pico ao abrir é
quase o dobro do que fica, porque `Document::load` copia o texto inteiro
antes de montar o rope (`crates/rope/src/document.rs:51-62`, o
`text.into()`). com tudo selecionado, a cena custa 28 vezes a da roda
simples: é o `clusters_styled` moldando de novo cada linha da tela a cada
frame.

### (b) onde entram o layout virtualizado e o cache por linha

- layout virtualizado: a emissão já é. o que falta é a geometria aguentar
  um milhão de linhas. a posição de cada linha passa a ser calculada em
  f64 contra o deslocamento da rolagem e só o resultado, pequeno, volta a
  f32 (`line_top`, `hit_test_point`, `scroll_to_cursor` e a faixa
  visível). o `ScrollState` guarda o deslocamento em f32
  (`crates/engine/src/input/scroll.rs:42-56`) e é do engine, fora dos
  arquivos desta fase: perto do fim de um milhão de linhas ele anda em
  passos de 2 px. isso vira pendência, não mudança de API.
- cache de shaping por linha: no `clusters_styled` do measure, ao lado
  do cache de medida que já existe. chave: o texto da linha e o estilo
  (os mesmos campos do `MeasureKey`, com a largura exata). invalidação
  numa edição: a chave é o conteúdo, então só a linha cujo texto mudou
  deixa de bater; uma quebra de linha que empurra um milhão de linhas
  para baixo não invalida nenhuma, porque a chave não carrega o índice.
  limite de memória: orçamento em bytes (texto mais clusters de cada
  entrada), com despejo da menos usada. fica no engine e não no editor
  porque o que se repete é o shaping dentro do `clusters_styled`, e o
  editor não consegue guardar o `VisualLine` entre frames (ele empresta o
  texto e mora em `form/`, fora dos arquivos permitidos). o `TextField`
  ganha o mesmo cache sem mudar nada. o cache do desenho
  (`TextSystem`) continua como está.
- abrir: `Document::load` deixa de copiar o texto quando não há `\r`.

### (c) o que o rope já dá

o rope é o ropey 1.6, uma árvore B com contagem de linhas nos nós:
`line(n)`, `line_to_byte` e `byte_to_line` são O(log n) e `len_lines` é
O(1). uma edição é uma `Transaction` (`crates/rope/src/transaction.rs:91-105`):
para cada trecho, `byte_to_char`, `remove` e `insert`, todos O(log n). o
inverso guarda só o texto removido (`transaction.rs:110-124`), e o
histórico guarda a transação e o inverso, sem cópia do documento. inserir
e apagar um caractere na linha 500 mil não custa nada linear; o benchmark
confirma (a coluna de edição é a mesma com 10 mil e com um milhão).

### como reproduzir

```sh
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
cargo test --offline --release -p comps --test editor_large_bench -- --ignored --nocapture
```

## fase 7: o benchmark

mesmo teste, mesma máquina (Apple M4, 16 GB, macOS 26.6.2), perfil
release, depois das três mudanças (linhas em f64, `Document::load` sem
cópia, cache de clusters por linha). o documento gerado tem linhas de
código de 44 a 65 bytes, todas diferentes; com um milhão de linhas são
57 MiB de texto. cada cenário: a roda anda 3 linhas por frame por 2000
frames; o arrasto da barra vai do topo ao fim em 300 frames, cada um numa
viewport nova; a roda com tudo selecionado são 1000 frames depois de
`select_all`; a edição insere e apaga um caractere na linha do meio, 200
vezes cada, medindo até o frame seguinte montado. média / p99 em ms.

| linhas | até | abrir | roda, 3 linhas por frame | arrasto da barra | roda, tudo selecionado | editar no meio | heap mantido (MiB) | pico ao abrir (MiB) |
|---|---|---|---|---|---|---|---|---|
| 10 mil | scene | 0,2 | 0,021 / 0,025 | 0,054 / 0,064 | 0,146 / 0,177 | 0,019 / 0,022 | 0,6 | 0,6 |
| 100 mil | scene | 1,6 | 0,021 / 0,025 | 0,086 / 0,099 | 0,146 / 0,159 | 0,020 / 0,024 | 6,2 | 6,2 |
| 1 milhão | scene | 17,5 | 0,021 / 0,029 | 0,095 / 0,126 | 0,148 / 0,185 | 0,020 / 0,040 | 64,2 | 64,2 |
| 10 mil | frame | 0,8 | 0,365 / 0,593 | 0,673 / 0,826 | 0,495 / 0,607 | 0,304 / 0,771 | 0,6 | 1,5 |
| 100 mil | frame | 3,1 | 0,368 / 0,625 | 1,019 / 1,227 | 0,481 / 0,558 | 0,258 / 0,317 | 6,2 | 7,1 |
| 1 milhão | frame | 21,0 | 0,360 / 0,587 | 1,042 / 1,241 | 0,482 / 0,568 | 0,268 / 0,343 | 64,2 | 65,0 |

antes e depois, com um milhão de linhas (os números de antes estão na
sonda):

| um milhão de linhas | antes | depois |
|---|---|---|
| abrir até o primeiro frame (ms) | 23,3 | 21,0 |
| pico de heap ao abrir (MiB) | 121,3 | 65,0 |
| heap mantido pelo documento e pelo editor (MiB) | 64,2 | 64,2 |
| roda, frame, média / p99 (ms) | 0,357 / 0,544 | 0,360 / 0,587 |
| arrasto da barra, frame (ms) | 1,194 / 2,884 | 1,042 / 1,241 |
| roda com tudo selecionado, cena (ms) | 0,590 / 0,628 | 0,148 / 0,185 |
| roda com tudo selecionado, frame (ms) | 0,946 / 2,026 | 0,482 / 0,568 |
| editar no meio até o frame seguinte (ms) | 0,311 / 1,071 | 0,268 / 0,343 |

leitura: o frame já não dependia de N antes e continua sem depender; o
pior frame medido, com um milhão de linhas, fica abaixo de 1,3 ms no
p99, longe dos 16,7 ms de 60 Hz. o que mudou foi o pico ao abrir (a
cópia do texto saiu), o custo de redesenhar linhas com seleção (o shaping
dos clusters deixou de se repetir a cada frame) e, fora da tabela, a
posição das linhas: antes, a partir da linha 100 mil, o passo entre duas
linhas saía errado em até 0,4 px e o clique perto do fim caía na linha
vizinha.

cada tabela é a segunda de duas rodadas seguidas. entre rodadas, as
médias variam até uns 15%; o p99 do `frame` varia até o dobro, porque
passa pela submissão à GPU (a primeira rodada de antes deu 1,973 ms de
p99 na roda com 10 mil linhas; a segunda, 0,585). a memória é só o heap
da CPU, contado por um alocador no próprio teste; o atlas de glifos na GPU
não entra.

para reproduzir (o disco precisa de uns 2 GB para o perfil release):

```sh
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
cargo test --offline --release -p comps --test editor_large_bench -- --ignored --nocapture
```

## handoff da fase 7: documentos grandes

fecha o item "benchmark de um milhão de linhas com números" do critério
de aceite: o editor abre, rola até o fim e edita no meio um documento de
um milhão de linhas, com o frame abaixo de 1,3 ms no p99 em release, e o
que ele desenha bate com o documento.

### commits

- `e7427780` a sonda, com os números de antes e o benchmark
- `71e0a6f2` as linhas posicionadas em f64 e o `Document::load` sem cópia
- `8b4c58e9` o cache de clusters por linha, com orçamento em bytes
- `727c378e` o benchmark depois, e a tabela de antes e depois
- o commit desta seção: o handoff

### a sonda, curta

o editor já era virtualizado na emissão: por frame, só as linhas da tela
mais 8 de cada lado viram nós de cena, e a altura vem da contagem de
linhas. o texto desenhado já tinha shaping por linha com cache no
`TextSystem`. o rope (ropey) já ia à linha n e editava no meio em O(log
n). o que faltava: a posição das linhas era f32 e errava a partir da
linha 100 mil; a linha do caret e as linhas com seleção eram moldadas de
novo a cada frame (`clusters_styled` sem cache); e abrir copiava o texto
inteiro antes de montar o rope.

### o que mudou

- `crates/comps/src/editor/view.rs`: `line_y` calcula em f64 a distância
  do topo do documento até a linha; `line_top`, `hit_test_point` e
  `scroll_to_cursor` usam essa conta, e só a diferença pequena, na tela,
  volta a f32. `input.rs`: o encaixe da roda em linhas inteiras usa a
  mesma conta. para um documento pequeno com rolagem zero, o resultado é
  bit a bit o de antes (as 176 baselines iguais confirmam).
- `crates/rope/src/document.rs`: `Document::load` monta o rope direto do
  `&str` e só copia quando há `\r` para normalizar.
- `crates/engine/src/text/cluster_cache.rs` (novo): o cache de
  `TextMeasurer::clusters_styled`. chave: o texto e o estilo, com a
  largura exata; sem índice de linha, então uma edição invalida só as
  linhas cujo texto mudou, e uma quebra de linha que empurra o resto do
  documento não invalida nada. limite: 4 MiB contados por entrada (chave
  e clusters), sai a menos usada; uma linha maior que o orçamento inteiro
  não entra. `measure.rs` só consulta e guarda (633 linhas).
- nenhuma API pública mudou; nada foi removido.

### números, um milhão de linhas, release, Apple M4

| | antes | depois |
|---|---|---|
| abrir até o primeiro frame (ms) | 23,3 | 21,0 |
| pico de heap ao abrir (MiB) | 121,3 | 65,0 |
| roda, frame, média / p99 (ms) | 0,357 / 0,544 | 0,360 / 0,587 |
| arrasto da barra, frame (ms) | 1,194 / 2,884 | 1,042 / 1,241 |
| roda com tudo selecionado, frame (ms) | 0,946 / 2,026 | 0,482 / 0,568 |
| editar no meio até o frame seguinte (ms) | 0,311 / 1,071 | 0,268 / 0,343 |

as tabelas completas (10 mil, 100 mil e um milhão, cena e frame) estão na
sonda e na seção do benchmark. comando:

```sh
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
cargo test --offline --release -p comps --test editor_large_bench -- --ignored --nocapture
```

### testes (8 novos, 1 benchmark ignorado)

- `crates/comps/src/editor/tests_large.rs`, com um milhão de linhas, em
  debug e no gate: abre e rola até o fim; rola por onze pontos do
  documento; clica perto do fim e cai na linha sob o ponteiro; insere,
  apaga e quebra linha no meio. cada frame confere que cada nó de texto é
  a linha do documento naquela posição, que as linhas estão a uma altura
  de linha uma da outra e que a margem numera as linhas certas. os quatro
  falhavam antes da correção de f64.
- `crates/engine/src/text/cluster_cache.rs`: acerto e chave, despejo da
  menos usada no orçamento, linha maior que o orçamento e troca contada
  uma vez. `tests_measure.rs`: `clusters_styled` molda uma vez por texto
  de linha e uma edição molda só a linha editada.
- `crates/comps/tests/editor_large_bench.rs`: o benchmark, `#[ignore]`.
- os testes do editor pequeno, do bidi (fase 3) e do `TextField` passam
  sem mudança.

### gate e regress

`./script/gate` verde com o ambiente do briefing: `cargo test
--workspace` 3144 passed, 0 failed, 28 ignored (eram 3136 e 27: mais os
8 testes e o benchmark); clippy, fmt e wasm verdes; perna urnaui 85
passed, 0 failed, 1 ignored. `regress -- check` depois de tocar
`crates/showcase/examples/regress/main.rs`: 176 unchanged, 0 changed.
fora do gate: `cargo check -p engine -p comps --target
aarch64-apple-ios-sim` com `-D warnings` verde; `cargo ndk -t arm64-v8a
check -p showcase --lib` compila, com os dois avisos antigos de código
morto. disco: 9,3 GB livres.

### pendências

- o `ScrollState` guarda o deslocamento em f32 (engine, fora desta fase).
  com um milhão de linhas de 19,6 px, perto do fim ele anda em passos de
  2 px: um delta de trackpad menor que 1 px se perde, e o fim do
  documento fica a até 1 px da borda. as linhas entre si ficam certas
  (a conta delas é f64). resolver pede o deslocamento em f64 no
  `ScrollState` ou um deslocamento do editor em linha mais fração; os
  dois mudam API pública, então é decisão.
- com tudo selecionado a cena ainda custa 7 vezes a da roda simples
  (0,148 contra 0,021 ms): o shaping saiu, mas o `VisualLine` refaz as
  paradas do caret de cada linha a cada frame. guardar o `VisualLine`
  pede um construtor a partir dos clusters em
  `crates/comps/src/form/text_input/visual.rs`, fora dos arquivos desta
  fase.
- linha muito longa (um arquivo minificado de megabytes numa linha só)
  não foi medida: o editor molda a linha inteira, sem virtualização
  horizontal, e uma linha acima de 4 MiB de clusters não entra no cache.
- o editor não quebra linha: a virtualização depende da altura fixa por
  linha. quebra suave num documento grande pede um índice de alturas;
  medido sem quebra, como o briefing pede.
- abrir continua O(n) no texto (17,5 ms de cena para 57 MiB) e o chamador
  segura o `String` e o rope ao mesmo tempo enquanto abre; carregar em
  pedaços, direto do arquivo, não foi feito.
- o IME (fase 4) continua aberto e pede o brenner.

## fase 7 integrada (orquestrador, 2026-09-25)

fecha o item "benchmark de um milhão de linhas com números". o editor já
emitia só as linhas visíveis; a fase corrigiu a posição das linhas (f64,
errava a partir da linha 100 mil), pôs um cache de shaping por texto de
linha (4 MiB, o menos usado sai) e abriu o documento sem cópia. com um
milhão de linhas, em release no M4: abrir 21 ms (pico de 65 MiB, antes
121), rolagem pela roda 0,36 ms por frame, edição no meio até o frame
seguinte 0,27 ms; rolar e editar custam o mesmo com 10 mil e com um
milhão. tabela completa e comando acima.

validação do orquestrador: diff lido (a chave do cache cobre todo campo
do `TextStyle` e a largura; nenhuma API pública mudou; os quatro testes
de um milhão de linhas falhavam antes da correção em f64); 176 baselines
iguais.

continua aberto no critério: o roteiro de IME executado (fase 4), que
pede o brenner. pendência: a rolagem do engine em f32 anda em passos de
2 px perto do fim de um milhão de linhas (mudar é API pública).

o agente gastou uns 190 mil tokens.

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

## fase 4: o roteiro do IME no editor (brenner, 2026-09-25)

o brenner rodou o roteiro no macOS, no exemplo do editor
(`cargo run -p comps --example editor`, o único app que liga o IME:
`crates/comps/examples/editor/main.rs:153`), com as fontes de entrada
japonês (Romaji) e chinês simplificado (Pinyin): `nihongo`, espaço e
Enter para 日本語; `zhongwen` e espaço para 中文. o que ele disse: "no
editor de código funciona" e, perguntado pelo japonês e pelo chinês,
"funcionaram". o editor já desenhava a composição sublinhada
(`comps/src/editor/view.rs:369`) e punha a janela de candidatos no
caret (`EditorView::ime_cursor_rect`).

o que falta para fechar o item "roteiro de IME executado e registrado":
o campo de texto. o `TextInput` joga fora o texto em composição
(`comps/src/form/text_input/component.rs:218-219`), e nem a galeria nem
o ide ligam o IME (`set_ime_allowed`: só o exemplo do editor chama),
então lá o japonês sai em letras. depois do código, o brenner roda o
mesmo roteiro num campo da galeria.

## fase 4: a sonda do campo

winit travado em 0.30.13; as linhas da biblioteca são de
`winit-0.30.13/src/platform_impl/macos/view.rs`.

### (a) a tecla latina com o IME ligado

com o IME ligado, toda tecla passa por `interpretKeyEvents` (466). o
`insertText` só manda `Ime::Commit` quando há texto marcado (411-415).
uma letra comum, sem composição, não marca nada: o estado não muda e o
`KeyboardInput` sai com o texto (478-497), como hoje. durante a
composição o estado é `Preedit` e a tecla não vira `KeyboardInput`
(485); o commit põe o estado em `Committed`, e o Enter que confirmou
não chega ao app (426-428, 480-483). então cada tecla chega por um
caminho só: `KeyboardInput` sem composição, `Ime::Preedit` e
`Ime::Commit` com ela. não insere duas vezes nem perde tecla. o
`Preedit("")` vem antes de cada commit (412). setas, Tab e Escape sem
composição passam pelo `doCommandBySelector` e seguem como tecla (430).
desligar (`set_ime_allowed(false)`) limpa a marca e manda
`Ime::Disabled` (880-896); trocar a fonte de entrada também (447-451).
o exemplo do editor já roda assim, e o latim digitado nele funciona.

### (b) onde está o retângulo do campo e o x do caret

o campo focado da página Input é o de `InputPage::focused`, no retângulo
de `InputPage::rects(content)` (`view/components/input.rs:55`), com
`content` o `page_rect` da view (`view/mod.rs:284`, já com a rolagem).
dentro do campo, a origem do texto é `bounds.x + pad_x`, `text_y`
centrado (`text_field.rs:326-327`), e o caret é `TextInput::caret_x`
(`component.rs:188`) com o mesmo `TextStyle` do desenho. o campo ganha
`ime_area(bounds, theme)` com essa conta (o caret do preedit durante a
composição); a view junta os dois. o editor já dá o seu em
`EditorView::ime_cursor_rect` (`editor/view.rs:221`).

### (c) quando ligar e desligar

sem campo focado as letras são atalhos: `handle_key` tenta o campo
(`view/input.rs:80`) e depois o `t` troca o tema (102). com o japonês
ligado, o `t` viraria composição e o atalho morreria. então o IME fica
ligado só com um campo focado da página Input, sem overlay por cima, e
na página Editor (que já toma todas as teclas, `events.rs:93`); em
qualquer outro caso, desligado. a galeria acerta isso a cada frame
desenhado da janela principal (`events.rs:197`): o foco só muda por um
evento que pede frame. as outras páginas com texto ficam desligadas,
como hoje (pendência no handoff).

## handoff da fase 4: o IME no campo de texto

prepara o item "roteiro de IME executado e registrado": o campo compõe
e a galeria liga o IME. falta o brenner rodar o roteiro num campo da
página Input (japonês `nihongo`, espaço, Enter; chinês `zhongwen`,
espaço); aí o item fecha.

### commits

- `3579e5be` a sonda do campo
- `c4866d15` a composição no `TextInput` e no `TextField`
- `45c0f909` a galeria liga o IME no campo focado e no editor
- `a6bfcb24` os testes e o PNG
- `576417b8` o `Disabled` sem o fold (o clippy)

### o que mudou

- `TextInput` guarda a composição e o caret do IME fora do buffer.
  `handle_ime(committed, preedit)` mantém a assinatura (e agora guarda a
  composição); a irmã `handle_ime_at` recebe o caret. perder o foco ou
  `clear_preedit` largam a composição. `composed`, `preedit_boxes` e
  `caret_x` usam o mesmo `TextStyle` do desenho.
- `TextField` desenha o texto composto (a composição no lugar da
  seleção ou no caret), um sublinhado de `edge_width` em
  `glass.text_active`, e o caret parado no caret do IME. `compose` passa
  o commit pelo mesmo `insert` da digitação; `end_compose` e `ime_area`
  são novos. a projeção lê só o buffer.
- galeria: `ImeEvent` e `handle_ime`/`ime_area` na view, sem winit; a
  página Input e a Editor recebem. o shell (`app/events/ime.rs`) passa o
  `WindowEvent::Ime`, grava o commit como tecla de texto (o tipo, nunca
  o texto) e, depois de cada frame da janela principal, liga o IME só
  quando `ime_area` responde. `app.rs` não cresceu.

### testes

sete no comps (`text_input/tests/tests_ime.rs`), quatro na galeria
(`tests/component_pages/ime.rs`) e um no shell (a gravação guarda um
`text` para o commit e nada da composição, conferido nos bytes do
arquivo salvo).

### o PNG

`/Users/nn/Dev/hoff/plevdev/target/agents/one/task-74h/ime-field.png`:
a página Input a 2x, o primeiro campo com `日本語!` digitado e `にほんご`
compondo, sublinhado, caret no fim. para refazer:
`IME_FIELD_OUT=<dir> cargo test -p showcase --test component_pages the_focused_field`.

### gate e regress

`./script/gate` verde: `cargo test --workspace` 3160 passed, 0 failed,
28 ignored; perna urnaui 86 passed, 1 ignored (3246 no total, eram
3234); clippy, fmt e wasm verdes. `regress -- check`: 176 unchanged, 0
changed. disco: 8,4 GB livres.

### pendências

- a janela da página (`D`) não liga o IME: a área da janela de
  candidatos precisaria do layout da janela da página, fora da rota.
- Todo, Surface, Silences e o formulário do Git continuam sem IME:
  o Todo só aceita um caractere por tecla, os campos das superfícies
  geradas vivem no crate `surface`, e o formulário do Git é do ide.
- o ide e o urnaui não ligam o IME (o mesmo fio).
- os frames gravados mostram o que está na tela, com a composição, como
  mostram o texto digitado.
