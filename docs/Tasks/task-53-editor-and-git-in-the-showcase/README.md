---
id: task-53
title: "task-53: editor and git in the showcase"
project: plev
type: task
status: done
date: 2026-09-21
date_basis: header
recorded: 2026-09-21
topics: [git, platform]
relations: [adr-0029, task-48, learning-core-buttons]
source: docs/log/steps/task-53-editor-and-git-in-the-showcase.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# task-53: editor and git in the showcase

Records the task that gave the showcase two app tabs, Editor and Git, occupying the area under the header, with the ide workspace embedded through `set_frame` and the rail with its own origin. The result depends on the `ide` having become both a lib and a bin, with `GitSession` as the bridge between the UI and the `GitClient`. Left out are git on the web and on mobile, which needs a thread and the git CLI, and the ide's header, which still centers on the window.

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
