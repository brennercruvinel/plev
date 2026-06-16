---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-29: app demo real (proof of life)

## objetivo
app não-trivial construído inteiramente com φ. prova que o engine funciona para algo além de demos isolados. deve usar: animações (task-27), texto editável (task-28), signals, layout, layers, input, tudo junto.

## dependências
- task-27 (animation system), transições e easing
- task-28 (editable text), input de texto funcional

## escolha do app: todo/notes app

**por que um todo app:**
- usa text input (criar/editar items)
- usa lista dinâmica (children_each com signals)
- usa animações (add/remove transitions, hover effects, check animation)
- usa layers (modal overlay, toast notification)
- usa input system (click, keyboard shortcuts)
- simples o suficiente para ser demo, complexo o suficiente para ser prova
- universal, qualquer reviewer do paper entende

**funcionalidades planejadas:**
1. adicionar todo via text input + enter
2. marcar como feito (click no checkbox, com animação de strikethrough/fade)
3. deletar todo (click no x, com animação de slide out)
4. filtrar: all / active / completed
5. counter de items restantes
6. layout responsivo (adapta ao tamanho da janela)

## design visual

```
┌─────────────────────────────────┐
│         ✦ φ todos            │  ← header, fonte grande
│                                 │
│  ┌───────────────────────┬────┐ │
│  │ What needs to be done?│ Add│ │  ← text input + botão
│  └───────────────────────┴────┘ │
│                                 │
│  ○ Buy groceries            ✕  │  ← item ativo
│  ● Learn Rust          ✕  │  ← item completo (fade/strike)
│  ○ Write paper              ✕  │
│                                 │
│  3 items left                   │
│  [All] [Active] [Completed]     │  ← filtros
└─────────────────────────────────┘
```

## checklist

### fase a, estrutura e estado
- [x] `examples/todo_app.rs` (~530 LOC)
- [x] struct todoitem { id, text, completed, opacity tween, complete_opacity tween }
- [x] estado: vec<todoitem>, textinput, filter enum, next_id counter
- [x] funcoes: add_todo, toggle_todo, remove_todo, visible_items, active_count

### fase b, layout e rendering
- [x] header com titulo estilizado + subtitulo
- [x] text input para novo todo (task-28 textinput)
- [x] lista de items com checkbox + texto + botao delete
- [x] footer: counter + filtros (all/active/completed)
- [x] layout centrado, responsive (max 600px content width)
- [x] dark theme consistente

### fase c, interatividade
- [x] enter no input -> add_todo + limpar input
- [x] click no checkbox/row -> toggle_todo
- [x] click no x -> remove_todo
- [x] click nos filtros -> muda filtro
- [x] escape limpa o input
- [x] hover effects em items e delete buttons
- [x] empty state messages por filtro

### fase d, animacoes (task-27)
- [x] fade-in ao adicionar (tween 0->1, easeoutcubic, 300ms)
- [x] opacity transition ao completar (1.0->0.6, easeinout, 200ms)
- [x] strikethrough visual para completed items
- slide-out ao remover: skipped (immediate remove)

### fase e, polish
- [x] responsivo: layout centrado adapta ao resize
- [x] empty state: mensagem por filtro
- [x] foco automatico no input ao abrir

### fase f, validacao
- [x] cargo check --workspace --examples: zero warnings
- [x] cargo test --workspace: 325 testes
- [x] codigo legivel como showcase da API φ

## uso das apis do φ (prova de integração)

| feature do engine | uso no todo app |
|-------------------|-----------------|
| signals | estado global (todos, filtro), memos derivados |
| builder API | toda a UI declarada com `div()`, `text()`, `text_input()` |
| layout (taffy) | flexbox para lista, header, footer |
| input system | click (toggle, delete, filter), keyboard (enter, escape) |
| text system | rendering de texto, text input editável |
| animation | fade, slide, hover effects |
| layers | possível overlay/modal para confirmação de delete |
| effects | shadow no card principal, blur em modal (se houver) |
| component | cada todoitem como #[component] com estado local |

## estimativa
~500-800 LOC. é "só" um app, mas depende da completude de task-27 e task-28.

## fora de escopo
- persistência (localstorage, file, database)
- drag-and-drop para reordenar
- multiline todos
- categorias/tags
- múltiplas listas
