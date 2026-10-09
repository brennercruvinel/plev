---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-21
domain: showcase
---

# task-53: editor e git no showcase, done

## resultado
o showcase ganhou duas abas de app: Editor (o `comps::editor::EditorView`
do example, no content rect) e Git (o workspace do ide, contra o repo do
cwd, com o rail dele). as duas ocupam a área toda sob o header, sem
gutter (`app_rect`).

## o que mudou pra isso
- `ide` virou lib + bin: `ide::views`, `ide::adapters`, `ide::status` e
  `ide::session::GitSession` (a ponte `UiRequest` <-> `GitCommand` /
  `GitEvent` saiu do `main.rs`)
- `WorkspaceView::set_frame(x, y, w, h)`: modo embedded, origem em
  coordenadas de janela, sem header próprio; `begin_frame` saiu do render
  pro host
- `comps::nav::Sidebar.origin`: um rail pode ser desenhado fora de (0, 0)
- showcase: `UserEvent::Wake` + callback `Wake` pro worker do git pedir
  frame; `ModifiersChanged` espelhado no view (alt/shift click no editor);
  3o argumento de launch rola a página (snapshot)
- editor: roda cai em linha inteira; nav link: hover só clareia o texto,
  wash e rim são a marca de ativo; sidebar: fade na borda da band que
  ainda tem link além dela
- docs sem "15 seções" literal, apontam pra `Section::ALL`

## fora
- git no web/mobile (precisa de thread e do git CLI): a aba explica
- o header do ide e o `Modal`/`ToastManager` ainda centram na janela, não
  num rect
