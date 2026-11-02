---
id: task-74
title: "task-74: text and editing"
project: plev
type: task
status: in-progress
date: 2026-09-25
date_basis: header
recorded: 2026-09-23
topics: [text, fonts, text-editing, performance]
relations: [adr-0021, adr-0024, adr-0071, research-parley-vs-cosmic-text, task-60, task-66, task-77, experiment-text-coverage-and-large-documents, learning-generative-runtime]
source: docs/log/steps/task-74-text-and-editing.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# task-74: text and editing

The plev text and editing task: selection, cursor, bidi, IME, emoji, fonts for non-Latin scripts, clipboard and large documents. It is in progress. Phases 1, 2, 3, 4, 5 and 7 have a recorded delivery; the acceptance criterion still depends on the IME test script on a field of the Input page, which brenner runs. Phases 6, 8 and 9 of the scope have no recorded execution.

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
  `tool/tests/test_namerules.rs`).
- fontes embutidas: Inclusive Sans (300 a 700), Inter (400 a 600),
  codicons (`rust/engine/src/drawing/text/fonts.rs:38-52`). árabe, hebraico,
  CJK e emoji colorido não estão cobertos; o sistema avisa uma vez por
  fonte não embutida.
- `rust/editor`: documento, transações, seleção, multi-cursor, movimento,
  history (undo e redo), puro e testado (ADR
  `text-editing-core-follows-helix`).
- `rust/engine/src/widgets/editor` (`EditorView`, multi-cursor sobre o rope) e
  `rust/engine/src/widgets/form` (`TextField` sobre o núcleo `TextInput`).
- `rust/engine/src/runtime/platform/ime.rs`: `ImeState` básico (evento, teclado
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

`rust/engine/src/drawing/text/**` (menos `probe.rs` enquanto a 66 roda),
`rust/engine/src/runtime/platform/ime.rs`, `rust/editor/**`,
`rust/engine/src/widgets/editor/**`, `rust/engine/src/widgets/form/text_input*`,
`rust/engine/assets/fonts` (depois da decisão de fontes). ADRs por fase.

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

## Phases

1. [phase 1 report: coverage audit by script](phase-1-audit.md): the corpus per script, what is missing and what comes out wrong, without a fix.
2. [phase 2: embedded Noto Sans](phase-2-noto-sans.md): font decision per script, fallback probe and integration.
3. [phase 3: bidi in the cursor and selection](phase-3-bidi.md): visual movement of the caret and selection in mixed text, and the wrap-up of shift+arrow in the apps.
4. [phase 4: IME, clipboard and field menu](phase-4-ime.md): composition in the field, the test script in the editor, copy and paste, and the field menu.
5. [phase 5: color emoji](phase-5-emoji.md): color bitmap in the R8 atlas and pixel measurement of the emoji.
6. [phase 7: large documents](phase-7-large-documents.md): one million lines, position in f64, cluster cache and benchmark.

Phases 6, 8 and 9 of the scope have no page: there is no recorded execution.
