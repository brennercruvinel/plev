---
id: adr-0021
title: "the text editing core follows helix: document is rope plus selections plus history"
project: plev
type: decisao
status: aceito
date: 2026-03-04
date_basis: cabecalho
topics: [edicao-de-texto, texto]
relations: []
source: docs/adr/interface/0021-text-editing-core-follows-helix.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# the text editing core follows helix: document is rope plus selections plus history

Decide que o núcleo de edição segue o modelo do Helix: um documento é uma rope, mais seleções, mais histórico, num crate `rope` sem dependência de UI nem de GPU. O motivo é que uma string plana custa O(n) por edição e perde as posições do cursor a cada mudança. O resultado é que toda edição passa por uma `Transaction`, com undo exato e seleções que se mapeiam sem caso especial por cursor, e o núcleo é testável sem janela.

## context

plev needs a text editing core for the ide and the editable-text widgets: a
buffer, multiple cursors, undo and redo, and a way to keep selections correct
across edits. doing this on a flat `String` is O(n) per edit and loses cursor
positions on every change. tying the editing model to the GPU or UI layer
would make it untestable headless.

## decision

a standalone crate (`rope`) with no UI or GPU dependency, modeled on Helix:
`Document = Rope + Selections + History`.

- the buffer is a rope (ropey), so edits and slicing are sublinear.
- edits are expressed as `Transaction`s: an ordered, pairwise non-overlapping
  list of `(range, replacement)` addressed in pre-transaction coordinates,
  applied atomically. a pure insertion has an empty range, a pure deletion an
  empty replacement.
- a transaction carries an internal op-based change set (retain, delete,
  insert) so two transactions compose without touching the document text, and
  any transaction can be inverted for undo.
- positions map through a transaction with an explicit `Bias` (before, after)
  deciding which side a position sticks to on an edit boundary. selections are
  just positions, so they map across edits for free.
- `History` stores undo steps, `SelectionSet` holds multi-cursor state,
  `movement` provides cursor motion with a goal column.

## consequences

- the whole core is pure data, fully testable headless (77 tests, a bench on
  the build plus insert/delete roundtrip), no window or GPU needed. this is
  rul-15 (testability by layer) in practice.
- undo is exact because each transaction inverts and redo replays, no
  snapshotting the whole buffer.
- multi-cursor edits and selection mapping come from one mechanism
  (transaction plus bias), not from a special case per cursor.
- the model is Helix's, not novel. that is deliberate: a studied, production
  design beats an invented one here.

## avoid

- do not edit the rope directly around the transaction layer. an edit that
  bypasses `Transaction` breaks undo and selection mapping.
- do not address edits in post-transaction coordinates. the contract is
  pre-transaction offsets, sorted and non-overlapping.
- do not leak UI or GPU types into this crate. its value is being
  headless-testable.
