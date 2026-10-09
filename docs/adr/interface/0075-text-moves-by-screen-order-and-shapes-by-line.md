---
type: adr
status: accepted
tags: [text, bidi, caret, selection, editor, large-documents, cache, rope, comps, engine]
date: 2026-09-25
---

# text moves by screen order and shapes by line

## context

task-74 had two problems in one area. the caret and the selection were
placed by byte offset, which is wrong in right-to-left and mixed text.
the editor also had to open, scroll and edit a million-line file: it
already drew only the visible lines, but it shaped every drawn line again
on every frame and placed rows with f32.

## decision

caret and selection (phase 3):

- `TextMeasurer::clusters_styled` returns one `ClusterBox` per shaped
  cluster: its bytes, x, width, line top, bidi level and whether the
  paragraph is right to left. carets stop at cluster edges on screen.
- the arrows move visually within a line and logically between lines. at
  the end of an LTR line, right goes to the start of the next RTL line.
- a selection stays one logical range in the buffer and draws as the boxes
  it covers on screen, several where it crosses a change of direction.
- the rope stays logical. the visual rules live in the widgets
  (`comps/src/form/text_input/visual.rs`, the editor's input).

large documents (phase 7):

- row positions are f64. with f32 the rows drifted from line 100 000 on,
  and a click near the end of a big file landed on the wrong line.
- a shaping cache per line of text (`engine/src/text/cluster_cache.rs`):
  the key is the text with every `TextStyle` field and the width, the
  least recently used entry goes first, and the budget is 4 MiB. an edit
  reshapes only the lines it touched.
- `rope::Document::load` takes the text without copying it.

measured with a million lines, release, Apple M4: open 21 ms (peak 65 MiB,
121 MiB before); a scroll frame 0.36 ms; an edit in the middle, up to the
next frame, 0.27 ms. scrolling and editing cost the same with ten thousand
lines and with a million. the benchmark is
`rust/engine/tests/editor.rs` (ignored, release).

## consequences

- open: the engine's `ScrollState` keeps its offset in f32, so near the
  end of a million lines it moves in 2 px steps (changing it is public
  API); the `VisualLine` of a line with a selection is rebuilt each frame;
  a single line of megabytes is shaped whole; soft wrap in a large
  document needs a height index; opening is still O(n) in the text.
- open in bidi: alt+arrow and the vertical arrows stay logical; a cluster
  wider than one grapheme (a ligature such as lam-alef) puts the caret at
  a fraction of its width, with no pixel test yet.
