---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
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
