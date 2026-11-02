---
id: task-55
title: "task-55: showcase, one page per component"
project: plev
type: task
status: done
date: 2026-09-21
date_basis: header
recorded: 2026-09-21
topics: [design-system, workspace]
relations: [adr-0030, changelog, learning-showcase-pages]
source: docs/log/steps/task-55-showcase-one-page-per-component.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# task-55: showcase, one page per component

Records the task that reorganized the showcase into one page per component: the `PAGES` table with 79 pages in three groups, one module per widget, and the sidebar with collapsible groups. The result is a reorganization with no new widget and no new example, with keyboard focus per page and a launch argument that is the page title. Some pages are only aliases or placeholders, and each alias's note says what is missing.

## objetivo
o showcase tinha uma seção por categoria (Forms, Chrome, Extras,
Overlays, Lists) e uma sidebar plana de 17 links. a migração do design
system é componente a componente, e a galeria precisa de uma página por
widget, em árvore, com slot pros widgets que o comps ainda não tem.
reorganização só: nenhum widget novo, nenhum exemplo novo.

## resultado
feito, PR #41 e o fechamento desta sessão. gate verde (workspace test,
clippy, fmt, wasm em CI).

## este passo
- `view::pages::PAGES`: tabela de 79 páginas em três grupos (Foundations
  4, Components 70 alfabético, Apps Demo 5); página própria, alias com
  nota do que falta, ou placeholder
- `comps::nav::Sidebar` com `NavGroup`: caption com chevron que dobra o
  grupo; `set_active` desdobra; rail só chevron
- um módulo por widget em `view/components/`; `view/foundations/` e
  `view/apps/` só mudaram de pasta; `view/panes.rs` guarda o estado,
  `view/dispatch.rs` é o único match em `Pane`
- foco de teclado por página (Tab, Escape)
- launch arg é o título da página (`showcase "Data Table" core 1400`)
- `comps::nav::sidebar` dividido em mod / input / render
- docs: `docs/adr/` virou `docs/adr/` com README.md como índice, changelog
  em [`changelog.md`](../../CHANGELOG.md); typos exclui os docs em português do root

## o que mudou de comportamento, de propósito
- o ciclo de Tab que atravessava os sete widgets do Forms é por página
- a barra de Progress não espelha mais o slider (páginas separadas)
- o breathing dos Skeleton liga clicando nos skeletons, não num nav link
- hotkey de dígito alcança as dez primeiras páginas da tabela

## fora deste passo
- busca na sidebar (a árvore dobrada já cabe, a aberta não)
- as páginas parciais: cada alias diz na nota o que a própria página
  ainda precisa; a ordem de migração segue a lista de aliases e
  placeholders em `view/pages/table.rs`
