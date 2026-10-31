---
id: task-28
title: "task-28: editable text"
project: plev
type: tarefa
status: concluido
date: 2023-01-25
date_basis: cabecalho
topics: [texto, input, edicao-de-texto]
relations: [task-09, task-13, task-29]
source: [docs/log/steps/task-28.md, docs/log/changes/task-28.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-28: editable text

Registra a entrega do campo de texto editável de uma linha: um `TextBuffer` com cursor e seleção, um mapeamento aproximado entre cursor e pixel, cursor piscante e ponte para o IME, reunidos no componente `TextInput`. O Registro de entrega cita 325 testes passando no workspace; o builder `text_input()` ficou para uma task futura. Importa porque é a camada sobre a qual o app demo (task-29) cria e edita seus itens, e porque a decisão de não usar o `Editor` do cosmic-text define como o texto editável se liga ao scene graph.

## objetivo
input de texto editável com cursor, edição básica e integração com IME. mínimo viável para o app demo (task-29), single-line text input funcional.

## dependências
- task-09 (input system, keyboard events, focus)
- task-13 (IME state machine)
- text system (cosmic-text shaping + glyph atlas)

## contexto técnico

### o que já existe
- **IME** (`ime.rs`): state machine completa, preedit_text, committed_text, keyboard show/hide
- **input** (`input/mod.rs`): plevkeyevent com campo `text: Option<String>`, focus tracking, hit regions focáveis
- **text rendering** (`text.rs`): cosmic-text buffer -> layout_runs() -> glyph quads no atlas
- **compositor** (`compositor.rs`): scenenode::text com textnodekey (text, font_size, line_height, max_width)
- **window** (`window.rs`): já processa `WindowEvent::Ime` e `WindowEvent::KeyboardInput`

### o que não existe (deve ser construído)
- **textbuffer**: string mutável + posição do cursor + range de seleção
- **char<->pixel mapping**: iterar layout_runs() para mapear byte offset -> posição x/y
- **cursor rendering**: retângulo piscante na posição do cursor
- **key handling**: backspace, delete, arrows, home/end no textbuffer
- **hit-testing de texto**: pixel -> byte offset (clique posiciona cursor)
- **conexão IME->textbuffer**: committed_text inserido no buffer, preedit renderizado inline

### decisão: não usar cosmic-text editor
cosmic-text tem um `Editor` mas ele assume controle total do buffer e requer integração profunda. para plev (immediate mode, scene graph), é mais limpo ter nosso próprio textbuffer leve que usa cosmic-text apenas para shaping + glyph metrics.

## design

### textbuffer (estado editável)
```rust
pub struct TextBuffer {
    text: String,
    cursor: usize,        // byte offset no text
    selection: Option<(usize, usize)>,  // (start, end) byte offsets
}

impl TextBuffer {
    pub fn insert_char(&mut self, ch: char);
    pub fn insert_str(&mut self, s: &str);
    pub fn delete_back(&mut self);      // backspace
    pub fn delete_forward(&mut self);   // delete key
    pub fn move_left(&mut self);
    pub fn move_right(&mut self);
    pub fn move_home(&mut self);
    pub fn move_end(&mut self);
    pub fn text(&self) -> &str;
    pub fn cursor(&self) -> usize;
}
```

### textinput (view/component)
```rust
// Builder API
text_input()
    .placeholder("Type here...")
    .font_size(16.0)
    .on_change(|new_text| { ... })
```

internamente: registra hitregion focável, processa plevkeyevent quando focado, renderiza texto + cursor.

### cursor<->pixel via cosmic-text
para mapear cursor position -> x coordinate:
- iterar `buffer.layout_runs()` -> `run.glyphs`
- cada glyph tem `glyph.start` (byte offset) e posição x
- cursor entre glyph n e n+1 -> x = glyph[n].x + glyph[n].w

para pixel -> cursor (hit-test de clique):
- inverso: encontrar glyph mais próximo do x clicado
- cursor vai para o lado mais próximo do glyph (antes ou depois)

## checklist

### fase a, textbuffer (puro, sem GPU)
- [x] struct `TextBuffer` em `src/text_input.rs`
- [x] insert_char, insert_str na posicao do cursor
- [x] delete_back (backspace), delete_forward (delete)
- [x] move_left, move_right (boundary-safe, char-aware)
- [x] move_home, move_end
- [x] select_all (ctrl/cmd+a)
- [x] delete_selection (qualquer insert ou backspace com selecao ativa)
- [x] testes unitarios: 24 testes (insert, delete, movement, selecao, unicode, emoji)

### fase b, cursor<->pixel mapping
- [x] cursor_to_x (approximate: char_count * font_size * 0.6)
- [x] x_to_cursor (inverse mapping)
- [x] testes: 5 testes (start, end, middle, ascii)

### fase c, textinput component
- [x] struct `TextInput` com focus, blink, rendering
- [x] renderiza: background rect + border + texto + cursor rect + selection highlight
- [x] cursor blinking: 530ms interval
- [x] keyboard handling (char, backspace, delete, arrows, home, end, select_all)
- [x] placeholder text (cor diferente quando buffer vazio e nao focado)
- [x] click posiciona cursor (hit-test pixel->cursor)
- [x] build_scene() gera scenenodes
- [x] testes: 13 testes

### fase d, IME integration
- [x] handle_ime(committed, preedit) bridge method
- [x] testes: 2 testes

### fase e, builder API integration
- skipped: build_scene() ja gera scenenodes, suficiente para proof of life
- builder integration adiada para task futura

### fase f, example
- [x] `examples/text_input_demo.rs`: 3 campos, tab cycling, click focus, cursor blink, live preview
- [x] cargo test --workspace: 325 testes passando
- [x] cargo check --workspace --examples: zero warnings

## fora de escopo (v1)
- multiline editing (pode vir depois)
- clipboard (copy/paste, requer crate `arboard` ou `clipboard`, adiar)
- undo/redo
- text selection via mouse drag (v1 só cursor positioning via clique)
- rich text / múltiplos estilos no mesmo input

## estimativa
fase a: ~200 LOC + testes. fase b: ~100 LOC. fase c: ~300 LOC. fase d: ~100 LOC. fase e: ~150 LOC. fase f: ~200 LOC. total: ~1000-1200 LOC.

## riscos
- **cosmic-text cursor mapping**: layout_runs() expõe glyph positions mas o mapeamento byte offset -> x precisa de cuidado com unicode (clusters, combining chars). pesquisar se cosmic-text expõe `hit()` ou similar antes de implementar do zero
- **borrow checker**: textbuffer precisa ser mutável durante event handling e imutável durante render, pode precisar do pattern de signals (readsignal/writesignal) ou refcell
- **performance**: re-shaping a cada keystroke é ok para single-line (< 1ms), mas para multiline seria problema (fora de escopo)

## Registro de entrega

### fase a: textbuffer
- [x] textbuffer com cursor, selection, char-aware movement
- [x] insert/delete/move/select operations
- [x] testes (24 unit)

### fase b: cursor-pixel mapping
- [x] cursor_to_x / x_to_cursor (approximate, font_size * 0.6)
- [x] testes (5 unit)

### fase c: textinput component
- [x] textinput com focus, blink (530ms), placeholder, rendering
- [x] keyboard handling (char, backspace, delete, arrows, home, end, select_all)
- [x] mouse click cursor positioning
- [x] build_scene() generates scenenodes (bg, border, text, cursor, selection)
- [x] testes (13 unit)

### fase d: IME integration
- [x] handle_ime(committed, preedit) bridge method
- [x] testes (2 unit)

### fase e: builder API, skipped
- skipped: textinput already generates scenenodes via build_scene(), sufficient for proof of life
- builder integration deferred to future task

### fase f: example + validation
- [x] examples/text_input_demo.rs: 3 fields, tab cycling, click focus, cursor blink, live preview
- [x] cargo test --workspace: 325 tests passing
- [x] cargo check --workspace --examples: zero warnings
