---
id: task-104
title: "task-104: the 35 empty or borrowed gallery tabs become kit components"
project: plev
type: task
status: in-progress
date: 2026-10-31
date_basis: text
recorded: 2026-10-31
topics: [interface, process, accessibility]
relations: [adr-0028, adr-0030, adr-0068, adr-0073, task-75]
source: https://github.com/brennercruvinel/plev/issues/153
source_revision: 5d78b03b
revised: 2026-10-31
review: pending
---

# task-104: the 35 empty or borrowed gallery tabs become kit components

Of the 82 tabs of the gallery (`apps/uigallery`), 17 are placeholders and 18 are aliases that borrow another page's example. This task turns those 35 into pages of their own, each with its widget in `engine::widgets`, built on the TypeScript kit by the path the Button took ([ADR 0028](../../adr/rendering/0028-core-theme-tailwind-classes-as-recipes.md), [the Button learning](../../learnings/core-buttons.md)): the class list becomes tokens, the tokens a recipe, the recipe a thin widget, and the page mirrors the kit's template. The 47 pages that already exist then go through an audit. The whole is held to one principle: design for content, constraints, capabilities and ranges, not screens and devices ([ADR 0073](../../adr/rendering/0073-design-for-ranges-not-screens-and-devices.md)).

The plan, the evidence behind each base and the definition of done live in the epic, [#153](https://github.com/brennercruvinel/plev/issues/153), and its five sub-issues. This page is the control panel: the state of each front and each tab against the definition of done, updated by the orchestrators as reviews close.

## How the work runs

- **One branch, one pull request.** Everything lands in `plevcomponentes`. Each front works in its own worktree and local branch (`task/104a-fields` and so on), and the orchestrators merge it in after review. This replaces the epic's one-PR-per-front plan (decision of 2026-10-31).
- **Agents edit, the Linux host compiles.** One agent per sub-issue edits its worktree; every build, test, regression check, semantics dump and screenshot runs on the Linux host through a wrapper that syncs the tree, waits for a slot in a shared build semaphore and runs under a memory cap. Nothing heavy runs on the editing machine.
- **Cycle per front:** a study and POC plan, the POC, the review by Tiamati and Brenner, then the full wave as an item-by-item task list until the sub-issue's acceptance holds in full.
- **Orchestrators:** Tiamati and Brenner. They own the scaffold, this page, the adaptive foundation ADR, the audit of the 47, every bless of a regression baseline and the final gate.

## Phases

| phase | what | state |
|---|---|---|
| 0 | tools on the Linux host, the build wrapper, the browser capture proved on Buttons | in progress |
| 0b | scaffold: anchors per front, token slots, empty modules, test files | in progress |
| 1 | the five POCs: Field, Popover, Message, Accordion, Toggle | study and plan |
| 1b | adaptive foundation: study, contracts in an ADR reviewed by Brenner, engine, evidence | not started |
| 2 | audit of the 47 existing pages | not started |
| 3 | full wave of the five fronts | not started |
| 4 | integration, final gate, closing records | not started |

## Fronts

| front | sub-issue | tabs | POC | state |
|---|---|---|---|---|
| A, fields | [#154](https://github.com/brennercruvinel/plev/issues/154) | Field, Label, Input Group, Input OTP, Textarea, Combobox, Command | Field | plan |
| B, overlays | [#155](https://github.com/brennercruvinel/plev/issues/155) | Popover, Hover Card, Dropdown Menu, Alert Dialog, Menubar, Sheet, Drawer | Popover | plan |
| C, conversation and feedback | [#156](https://github.com/brennercruvinel/plev/issues/156) | Message, Bubble, Message Scroller, Alert, Attachment, Marker, Questionnaire | Message | plan |
| D, disclosure and display | [#157](https://github.com/brennercruvinel/plev/issues/157) | Accordion, Collapsible, Item, Kbd, Aspect Ratio, Carousel, Pagination, Table | Accordion | plan |
| E, selection and dates | [#158](https://github.com/brennercruvinel/plev/issues/158) | Toggle, Toggle Group, Button Group, Radio Group, Calendar, Date Picker | Toggle | plan |

## Quality control, tab by tab

The columns are the eight items of the definition of done in the epic: **1** own page in `pages/table.rs`; **2** widget in `engine::widgets` with every visual value a token; **3** every kit state on the page; **4** the five range rules with the sweep, the motion, contrast and input tests and the six screenshots; **5** a `Component` in `PROJECTING`, with the three-actor test and the keyboard; **6** regression reviewed, blessed and `regress check` at 0 in core and hoff; **7** behaviour tests with edge and error cases, count not falling; **8** organisation: one widget per file, no dead or parallel code, the front's adoptions, no production file above 639 lines.

A cell is empty until the orchestrators have checked it against its evidence; `ok` means checked, `open` names a finding still being fixed.

| front | tab | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|---|
| A | Field (POC) | | | | | | | | |
| A | Label | | | | | | | | |
| A | Input Group | | | | | | | | |
| A | Input OTP | | | | | | | | |
| A | Textarea | | | | | | | | |
| A | Combobox | | | | | | | | |
| A | Command | | | | | | | | |
| B | Popover (POC) | | | | | | | | |
| B | Hover Card | | | | | | | | |
| B | Dropdown Menu | | | | | | | | |
| B | Alert Dialog | | | | | | | | |
| B | Menubar | | | | | | | | |
| B | Sheet | | | | | | | | |
| B | Drawer | | | | | | | | |
| C | Message (POC) | | | | | | | | |
| C | Bubble | | | | | | | | |
| C | Message Scroller | | | | | | | | |
| C | Alert | | | | | | | | |
| C | Attachment | | | | | | | | |
| C | Marker | | | | | | | | |
| C | Questionnaire | | | | | | | | |
| D | Accordion (POC) | | | | | | | | |
| D | Collapsible | | | | | | | | |
| D | Item | | | | | | | | |
| D | Kbd | | | | | | | | |
| D | Aspect Ratio | | | | | | | | |
| D | Carousel | | | | | | | | |
| D | Pagination | | | | | | | | |
| D | Table | | | | | | | | |
| E | Toggle (POC) | | | | | | | | |
| E | Toggle Group | | | | | | | | |
| E | Button Group | | | | | | | | |
| E | Radio Group | | | | | | | | |
| E | Calendar | | | | | | | | |
| E | Date Picker | | | | | | | | |

The audit of the 47 existing pages gets its own table here when phase 2 starts.

## Open decisions

- **Adaptive foundation ADR.** The contracts go to Brenner before any engine code (phase 1b).
- **Questionnaire.** No base in the kit or in plev; the tab stays blocked until Brenner settles what it is.
- **Textarea.** The `TextInput` is single-line; front A reports the options for a multi-line base before building it.

## Evidence

Screenshots, semantics dumps and logs stay outside git, under `tmps/task-104/<front>/` on the build host and in each worktree. The pull request of `plevcomponentes` carries the reviewed selection.
