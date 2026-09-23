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
