---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: changelog
---

# task-29 changelog: demo app (todo app)

## fase a: estrutura e estado
- [x] `examples/todo_app.rs` (~530 LOC)
- [x] todoitem { id, text, completed, opacity tween, complete_opacity tween }
- [x] todoapp state: vec<todoitem>, textinput, filter, next_id
- [x] add_todo(), toggle_todo(), remove_todo(), visible_items(), active_count()

## fase b: layout e rendering
- [x] header com titulo + subtitle
- [x] textinput integrado (task-28)
- [x] lista de items com checkbox, texto, delete button
- [x] footer: counter + filter buttons (all/active/completed)
- [x] centered layout, responsive to window width (max 600px content)
- [x] dark theme with consistent colors

## fase c: interatividade
- [x] enter -> add todo + clear input
- [x] click checkbox/row -> toggle completed
- [x] click x -> remove todo
- [x] click filter buttons -> change filter
- [x] escape -> clear input
- [x] hover effects on items and delete buttons
- [x] empty state messages per filter

## fase d: animacoes (task-27)
- [x] fade-in ao adicionar (tween 0->1, easeoutcubic, 300ms)
- [x] opacity ao completar (1.0->0.6, easeinout, 200ms)
- [x] strikethrough visual for completed items

## fase e: validacao
- [x] cargo check --example todo_app: zero warnings
- [x] cargo check --workspace --examples: zero warnings
- [x] cargo test --workspace: 325 tests passing
