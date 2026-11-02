---
id: task-104
title: "task-104: the 35 empty or borrowed gallery tabs become kit components"
project: plev
type: task
status: in-review
date: 2026-10-31
date_basis: text
recorded: 2026-10-31
topics: [interface, process, accessibility]
relations: [adr-0028, adr-0030, adr-0068, adr-0073, task-75]
source: https://github.com/brennercruvinel/plev/issues/153
source_revision: 5d78b03b
revised: 2026-11-01
review: pending
---

# task-104: the 35 empty or borrowed gallery tabs become kit components

Of the 82 tabs of the gallery (`apps/uigallery`), 17 are placeholders and 18 are aliases that borrow another page's example. This task turns those 35 into pages of their own, each with its widget in `engine::widgets`, built on the TypeScript kit by the path the Button took ([ADR 0028](../../adr/Rendering/0028-core-theme-tailwind-classes-as-recipes/0028-core-theme-tailwind-classes-as-recipes.md), [the Button learning](../../adr/Rendering/0028-core-theme-tailwind-classes-as-recipes/core-buttons.md)): the class list becomes tokens, the tokens a recipe, the recipe a thin widget, and the page mirrors the kit's template. The 47 pages that already exist then go through an audit. The whole is held to one principle: design for content, constraints, capabilities and ranges, not screens and devices ([ADR 0073](../../adr/Rendering/0073-design-for-ranges-not-screens-and-devices/0073-design-for-ranges-not-screens-and-devices.md)).

The plan, the evidence behind each base and the definition of done live in the epic, [#153](https://github.com/brennercruvinel/plev/issues/153), and its five sub-issues. This page is the control panel: the state of each front and each tab against the definition of done, updated by the orchestrators as reviews close.

## How the work runs

- **One branch, one pull request.** Everything lands in `plevcomponentes`. Each front works in its own worktree and local branch (`task/104a-fields` and so on), and the orchestrators merge it in after review. This replaces the epic's one-PR-per-front plan (decision of 2026-10-31).
- **Agents edit, the Linux host compiles.** One agent per sub-issue edits its worktree; every build, test, regression check, semantics dump and screenshot runs on the Linux host through a wrapper that syncs the tree, waits for a slot in a shared build semaphore and runs under a memory cap. Nothing heavy runs on the editing machine.
- **Cycle per front:** a study and POC plan, the POC, the review by Tiamati and Brenner, then the full wave as an item-by-item task list until the sub-issue's acceptance holds in full.
- **Orchestrators:** Tiamati and Brenner. They own the scaffold, this page, the adaptive foundation ADR, the audit of the 47, every bless of a regression baseline and the final gate.

## Phases

| phase | what | state |
|---|---|---|
| 0 | tools on the Linux host, the build wrapper, the browser capture proved on Buttons | done |
| 0b | scaffold: anchors per front, token slots, empty modules, test files | done |
| 1 | the five POCs: Field, Popover, Message, Accordion, Toggle | done |
| 1b | adaptive foundation: study, contracts in an ADR reviewed by Brenner, engine, evidence | done ([ADR 0086](../../adr/Interface/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme.md)) |
| 2 | audit of the 47 existing pages | done; its findings went to fronts J, L, O, P and T |
| 3 | full wave of the five fronts | done, in [#171](https://github.com/brennercruvinel/plev/pull/171) |
| 4 | integration, final gate, closing records | in review, [#177](https://github.com/brennercruvinel/plev/pull/177); the macOS regression bless and the merge are Brenner's |

## Fronts

| front | sub-issue | tabs | POC | state |
|---|---|---|---|---|
| A, fields | [#154](https://github.com/brennercruvinel/plev/issues/154) | Field, Label, Input Group, Input OTP, Textarea, Combobox, Command | Field | integrated |
| B, overlays | [#155](https://github.com/brennercruvinel/plev/issues/155) | Popover, Hover Card, Dropdown Menu, Alert Dialog, Menubar, Sheet, Drawer | Popover | integrated |
| C, conversation and feedback | [#156](https://github.com/brennercruvinel/plev/issues/156) | Message, Bubble, Message Scroller, Alert, Attachment, Marker, Questionnaire | Message | integrated |
| D, disclosure and display | [#157](https://github.com/brennercruvinel/plev/issues/157) | Accordion, Collapsible, Item, Kbd, Aspect Ratio, Carousel, Pagination, Table | Accordion | integrated |
| E, selection and dates | [#158](https://github.com/brennercruvinel/plev/issues/158) | Toggle, Toggle Group, Button Group, Radio Group, Calendar, Date Picker | Toggle | integrated |
| F, kit foundation and type by role | [#164](https://github.com/brennercruvinel/plev/issues/164) | every tab: one type scale by role, tabular figures | | integrated |
| G, navigation and data | [#165](https://github.com/brennercruvinel/plev/issues/165) | Chart, Code Block, Graph, Stat, Toolbar, Sidebars | | integrated |
| H, the gallery's shell | [#163](https://github.com/brennercruvinel/plev/issues/163) | the kit's sidebar, ⌘K, the drawer | | integrated |
| I, focus scrolls into view | [#166](https://github.com/brennercruvinel/plev/issues/166) | every tab | | integrated |
| J, fluid showcase mold and page frame | [#167](https://github.com/brennercruvinel/plev/issues/167) | every tab | | integrated |
| K, touch targets, word breaks, Escape, system preferences | [#168](https://github.com/brennercruvinel/plev/issues/168) | every tab | | integrated |
| L, page fixes from the audit | [#169](https://github.com/brennercruvinel/plev/issues/169) | Menus, Modals, Topbar, Share, Command, sidebar | | integrated |
| M, AA contrast and More contrast | [#170](https://github.com/brennercruvinel/plev/issues/170) | every theme | | integrated |
| N, text scale to 2x | [#172](https://github.com/brennercruvinel/plev/issues/172) | every tab | | integrated |
| O, page polish | [#173](https://github.com/brennercruvinel/plev/issues/173) | shell, Home, Card, View controller | | integrated |
| P, the sweeps' known list | [#174](https://github.com/brennercruvinel/plev/issues/174) | every tab | | integrated |
| R, session format version 2 | [#175](https://github.com/brennercruvinel/plev/issues/175) | the recording | | integrated |
| T, end of the sweep | [#176](https://github.com/brennercruvinel/plev/issues/176) | Collapsible, fields, empty states, View controller, Menus | | integrated |

## Quality control, tab by tab

The columns are the eight items of the definition of done in the epic: **1** own page in `pages/table.rs`; **2** widget in `engine::widgets` with every visual value a token; **3** every kit state on the page; **4** the five range rules with the sweep, the motion, contrast and input tests and the six screenshots; **5** a `Component` in `PROJECTING`, with the three-actor test and the keyboard; **6** regression reviewed, blessed and `regress check` at 0 in core and hoff; **7** behaviour tests with edge and error cases, count not falling; **8** organisation: one widget per file, no dead or parallel code, the front's adoptions, no production file above 639 lines.

The per-tab cells were not filled one by one. The closing checked each column against evidence that covers every tab at once, on `plevcomponentes` (Linux host, 2026-11-01):

| column | evidence | state |
|---|---|---|
| 1 own page | no placeholder or alias left but Questionnaire; `component_pages` 145 ok | ok, except Questionnaire |
| 2 widget with tokens | reviewed per front in its sub-issue; not re-checked tab by tab at the closing | per front |
| 3 every kit state | reviewed per front in its sub-issue; empty states added by front T | per front |
| 4 range rules | `text_scale` (1x to 2x, pseudo-locale, low window) and `hit_targets` (whole page, pointer and finger) over every tab, ok; contrast in `theme/tests` (AA in every theme, More) ok | ok; the six screenshots per tab not taken |
| 5 projection and keyboard | `access_tree` and `component_pages` ok | ok |
| 6 regression | Linux base blessed in core and hoff, `regress check` 180/180 unchanged; the committed macOS base is Brenner's bless | Linux ok, macOS pending |
| 7 behaviour tests | engine lib 1829, uigallery lib 598, record 62; the engine count went from 1812 to 1829 over the last integrations, never down | ok |
| 8 organisation | no production file past 639 lines (`tool/tests/test_filepaths`) | ok |

## Open decisions

Settled:
- **Adaptive foundation ADR.** Accepted as [ADR 0086](../../adr/Interface/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme.md): `Prefs` read once into `Theme::adapted`, the text scale to 2x, `Contrast::More` aiming at AAA for text where it fits.
- **Textarea.** A multi-line base exists (`engine::widgets::form::textarea`).
- **Kit colours below WCAG.** Front M brings every text and mark pair to AA in the four themes. Brenner chose the field border's option B: the kit's `#E2E2E2` at rest, a fade over the motion token to at least 3:1 on hover and focus (instant with reduced motion), strong in More. The twelve rest pairs stay in the contrast test's known list with that reason.

Open, for Brenner:
- **Questionnaire.** Still a placeholder: no base in the kit or in plev.
- **How a toggle is announced.** `Role::Toggle` still reaches AccessKit as `Switch`.
- **Colour inside a paragraph.** The mention colour waits for per-span text style (rich text).
- **Colours front M changed.** Core secondary text `#6E6E6E` (and a little darker on the glass, a ladder step of 1.15 above the faint, front M's choice), success `#28A048`, the link as the dark ink at the least alpha that reads AA, a darker warning orange; hoff danger `#FD6D5D`, warning `#FF6D36`; light warning `#AE7D00`.
- **Delete confirm, stacked.** Cancel on top or the destructive action on top.
- **Share file, with a finger.** The remove's touch area stops at the select (under 44 px) so the select keeps its place; reserving 44 always moves the select about 20 px with a pointer too.

## Evidence

Screenshots, semantics dumps and logs stay outside git, under `tmps/task-104/<front>/` on the build host and in each worktree. The pull request of `plevcomponentes` carries the reviewed selection.
