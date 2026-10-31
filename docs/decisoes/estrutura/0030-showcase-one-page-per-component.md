---
id: adr-0030
title: "showcase: one page per component, the sidebar as a folding tree"
project: plev
type: decisao
status: aceito
date: 2026-09-21
date_basis: cabecalho
topics: [design-system, plataforma]
relations: [arquitetura-component-catalog]
source: docs/adr/structure/0030-showcase-one-page-per-component.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# showcase: one page per component, the sidebar as a folding tree

Decide que o showcase tem uma página por componente, com as páginas como dado (uma tabela `PAGES`) e a barra lateral como árvore dobrável. O motivo era a galeria antiga, com uma seção por categoria e uma lista plana de 17 links que, com cerca de 80, não caberia numa janela curta. O resultado é que cada página mostra um widget em todos os estados, e as lacunas do comps aparecem como placeholders na ordem da migração.

## context

the gallery had one section per comps category (Forms held seven
widgets, Chrome nine, Extras six) and a flat sidebar of 17 links keyed
by a positional enum (`Section::ALL`, `[ScrollState; 17]`, digit
hotkeys). the design system is migrating one component at a time from
the typescript kit, and a page per component is the unit that migration
needs: a place to look at one widget in every state, and a slot for the
widgets comps does not have yet. a flat list of ~80 links does not fit
a short window and does not read as a system.

## decision

- the pages are data: `view::pages::PAGES`, a const table of
  `Page { group, title, icon, blurb, body }`. the sidebar, the digit
  hotkeys, the launch argument and the per-page scroll all index it.
  nothing else lists the pages.
- three groups, in this order: Foundations (theme, typography, icons,
  effects), Components (one page per comps widget, alphabetical), Apps
  Demo (the todo app, the message dock, the embedded editor, the ide git
  workspace, the builder tour).
- a page's body is one of three: `Pane(p)`, the page is that module;
  `Alias { pane, gap }`, the page shows another pane's example under a
  one-line note ("shares the Input example. still missing: label, helper
  and error"); `Placeholder`, an empty state naming the slot, for a
  widget comps does not have. an alias shares state with its owner: it
  is the same example, not a copy.
- one module per pane under `view/{foundations,components,apps}`, each
  with `content_height`, `handle_event`, `render` and, where it
  animates, `tick`. `view/panes.rs` owns the structs; `view/dispatch.rs`
  is the only place that matches on `Pane` (height, event, tick,
  render). keyboard focus is per page: Tab cycles the page's own
  widgets, Escape clears.
- `comps::nav::Sidebar` takes `NavGroup`s: a caption row per group with
  a chevron, a press folds or unfolds the group's links, activating a
  link unfolds its group. the link index space does not change; folded
  links get a zero-height rect, so they neither draw nor hit. the rail
  draws the chevron only.
- a new comps widget ships its page in the same change; a widget that
  does not exist yet keeps its placeholder slot so the tree reads
  complete and the migration order is visible.

## consequences

- the Tab cycle that walked all seven Forms widgets is gone; each page
  cycles its own. the Progress bar no longer mirrors the volume slider
  (separate pages). Skeleton breathing toggles on the skeletons, not on
  a nav link.
- digit hotkeys reach the first ten pages of the table; the rest are
  sidebar only. a sidebar search is the next step once the tree is
  taller than a screen even folded.
- the launch argument is the page title as the sidebar shows it:
  `showcase "Data Table" core 1400`.
- [`component-catalog.md`](../../referencia/arquitetura/component-catalog.md) stays the inventory of what comps has;
  `view/pages/table.rs` is the inventory of what the gallery shows,
  including what is still missing.
