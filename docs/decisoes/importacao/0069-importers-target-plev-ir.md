---
id: adr-0069
title: "importers target plev IR, with a report in four categories"
project: plev
type: decisao
status: aceito
date: 2026-09-25
date_basis: cabecalho
topics: [importacao, ir, formatos]
relations: [adr-0011, adr-0012, adr-0049, adr-0051, adr-0064, task-68, task-69, task-71]
source: docs/adr/importing/0069-importers-target-plev-ir.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# importers target plev IR, with a report in four categories

Decide que os importadores emitem a IR do plev por um segundo emissor sobre o mesmo `ParserNode`, e que o relatório classifica cada construção de origem como suportada, transformada, aproximada ou impossível, com o `at` (arquivo:linha). As tolerâncias de valor para tokens são medidas, não escolhidas a olho, e cada controle precisa de um binding nomeado que o importador não declara. O resultado é que nenhuma equivalência fica afirmada em silêncio: o relatório lista cada entrada, e um teste de censo prova que nenhuma construção ficou de fora.

## context

`rust/import` turns a React component (tsx and sass) and a gpui widget
into rust code against `engine::builder`, and reports what it drops on a
droplist with file:line ([adr-0012](0012-transpiler-reports-every-unmapped-construct.md)).
the thesis (sections 3 and 14) asks more of an importer: sources of UI
converge on plev IR, and every construct is supported, transformed,
approximated or impossible, never an equivalence claimed in silence. the
IR exists (task-68, `rust/plevir`, [adr-0049](../geracao/0049-plev-ir-typed-and-verified.md))
and so does its materialization (task-69, `rust/screen`,
[adr-0051](../geracao/0051-materialized-surfaces-from-verified-ir.md)). the runtime never
carries the source's execution model
([adr-0011](0011-import-foreign-formats-by-conversion-not-embedding.md)).

the probe of task-71 phase 1 (in
[`task-71-compiler-target.md`](../../tarefas/task-71-compiler-target.md)) found that the
`ParserNode` loses what the IR needs: the source tag (a `<button>` and a
`<div>` were one `Tag::Div`), the attributes other than `className`, and
where each prop came from (the resolvers only counted). the IR itself
needed no change.

## decision

### a second emitter over the same ParserNode

- `emit_ir` writes a `uir::Ir` from the same `Resolution` the code
  emitter prints; the code emitter stays until a decision removes it.
- the `ParserNode` carries its origin, additively: an `Origin` per node
  (the element or chain it was made from, its source tag and first class,
  its attributes, the constructs a rewrite folded into it), the source
  constructs of each `Prop` (outside its equality), notes for mapped
  constructs that became no prop, and a kind (lost or rewritten) and
  `covers` (the lines it answers for) per droplist entry. the code
  emitter reads none of it: the gpui golden is the same bytes, and so is
  the emission of 128 real React components swept before and after.
- what maps: a `div` is a container with its stack (`row`, `align_center`);
  the spacers of the margin-bottom rewrite are the stack's gap; a bar as
  thin as `theme.control.edge_width` in `theme.colors.divider` is the
  `separator` component; a text run is `text`, its literal the label, a
  text parameter bound to the app's state; a `<button>`, any element with
  `onClick` and a text `<input>` are `button` and `text_field`; an element
  slot is left out. ids come from the class names under
  `<surface>.root`, unique among siblings.
- the IR holds no pixel and no color: values go to tokens (below). the
  parser does not depend on the engine, so the theme's values come in as
  a table (`parser::Tokens`); the tests fill it from `Theme::hoff()`, the
  theme the emitted builder code reads.

### four categories, one report

- every construct of the source is one entry: `at` (file:line),
  `construct` (the droplist's own naming), and one of `supported` (with
  `ir`), `transformed` (with `ir` and `rule`), `approximated` (with `ir`
  and `difference`) or `impossible` (with `reason`). a construct met twice
  (a style pushed down onto two text runs) keeps its weakest use.
- the droplist is extended, not lost: every entry is in the report at the
  same line, a lost one as `impossible` with its `why`, a rewritten one (the
  separator's flank rewrite and `mx_auto`) as `transformed`. an entry that
  drops a block whole lists the lines it answers for in `covers`.
- `bindings` lists what the IR binds and the source does not declare.
- the JSON form is a contract (documented in `rust/import/src/report.rs`):
  `plev_report: 1`, fixed names, keys written in order whatever map
  `serde_json` was built with. it is built with `serde_json`; the parser
  does not take `serde` with derive.
- proof that nothing is left out: a census read straight from the parse,
  independent of the resolver (each JSX element, attribute, text and
  expression child, each conditional class, each declaration of each sass
  rule; each method call reachable from gpui's `render`, through closures,
  match arms and the `Self::f` helpers), each item named by an entry at its
  line or covered by one; and every IR node is the node of some entry.

### the tolerance, measured

a value goes to the token nearest it. its tolerance is half the distance
from that token to its nearest neighbour of a different value in the same
table: within it the value is closer to that token than to any other.
distance 0 is `transformed`; within the tolerance `approximated`, with the
distance; past it `impossible`, and the IR leaves the field to the
component's default.

- color: the largest of the four channel differences, 8-bit units, alpha
  included (the measure of [adr-0064](../processos/0064-what-identical-means.md)).
- spacing, radius, width, height: pixels. typography: the larger of the
  size and line height differences, among the roles of the same weight.

measured on `Theme::hoff()`:

| token | tolerance | from |
|---|---|---|
| `bg`, `surface` | 1 level | each other, `#303030` and `#2E2E2E` |
| `bg_hover`, `divider` | 3.5 | each other (7) |
| `text_mid` | 25.5 | `text_dim` (51) |
| `text` | 31.9 | `text_mid` (63.75) |
| spacing `xs` to `lg` | 2 px | the 4 px steps |
| spacing `xl`, `xxl` | 4 px | the 8 px steps |
| shape `pill` (32) | 4 px | `dropdown` (24) |
| size `card_w` (368) | 16 px | `modal_max_w` (400), among the widths |
| `caption_r` (12/15.96) | 1 px | `readout` (10/15) |

two IR fields hold one token where the source has several values: the
padding (four sides) takes the smallest side's token and the stack's gap
the smallest spacing's, so the IR never adds room the source does not
have; every larger value is `approximated` with its difference, and a 0
side leaves no padding (`impossible`).

### binding required

a control's event is bound to an action the importer names on the app's
node `<surface>.props`: after the handler (`onOpen` is `open`) or, with
none, the label; a text parameter is bound to `<surface>.props:<name>`.
the IR only references (`uir::Bind`); the importer declares nothing. with
nothing declared the verifier refuses the IR once per binding, naming it
(rule 4, "the app declares no state", or rule 5, "the app declares no
action"), and the report lists the same references; with the registry the
app declares, it verifies. a control left unbound would be refused by rule
7 without saying which capability is missing, and a control emitted as
something else would claim an equivalence.

### the pixel comparison

the builder code and the materialized IR are drawn by the same
`engine::headless::Renderer`, same size, scale 1, `Theme::hoff()`, the
theme's background, the builder's root in a host column as wide as the
area (as the surface lays its root). the IR carries no pixel, so the
verdict is a difference, measured as [adr-0064](../processos/0064-what-identical-means.md)
measures one and explained by the report (apple M4, metal):

- separator: 1266 pixels differ, delta max 164. the IR's row is 36 px (the
  surface's separator takes `theme.control.height(Xs)` in a row), not 24,
  so everything is 6 px lower; moved back, 750 differ, delta max 18, all
  on the line's two rows (the surface centers the 1 px line at 17.5, the
  builder lands it on a whole row) and in 4 px of line beside the label on
  each side (the chip's padding is `xs` where the source has 8); the
  label's pixels are the same.
- note card: 99001 pixels differ, delta max 197. every run under the
  preview is 160 px higher (its height has no token) and 16 px further
  left (the details' side padding has no form); the title and the content
  are the same pixels, moved; the meta line differs in 217 pixels by at
  most 5 levels (its color approximated to `text_mid`); the field and the
  button are comps widgets in the theme's style; the card is 180 px
  shorter, each pixel of it accounted by an entry.

the tests assert the structure (where each run is, which regions differ,
the bounds), not the counts, so another GPU passes them.

## consequences

- the code emitter's droplist gained entries for two silences of the
  parse, an `{expression}` child that is not an identifier and a text style
  no text run takes: 26 of the 128 swept components report more entries
  (and the ones with a style that reached no run count fewer as mapped),
  the code is the same in all 128; the corpus card and the separator are
  unchanged (51/38, 18/6).
- the builder aligns a column's children at the start by default, the IR's
  stack stretches them, as the CSS block the source is: the builder draws
  an empty `<input>` 32 px wide, the IR a field as wide as its column.
- the corpus card (outside the repo) runs the same checks live when
  present: 103 entries (10 supported, 33 transformed, 15 approximated, 45
  impossible), three bindings, verified with its registry. the React
  fixture of the gate is written for the phase, not copied.
- what the IR v1 cannot say, reported as impossible: a size off the
  tokens (the preview's 160 px, a height other than `dropdown_max_h`),
  borders, single shadows, backdrop blur, clipping, per-side padding,
  justification, shrink, element slots, prop-driven variants, media
  queries, hover states, animations. each needs the `uir` vocabulary to
  grow, which is its own decision (a component needs comps first).

## alternatives

- tolerância escolhida a olho: rejeitada. A medida é a própria tabela de tokens (`## avoid`).
- importador que declara a capacidade para o IR verificar sem registro: rejeitado. Com nada declarado, o IR é recusado uma vez por binding, com o nome do que falta (`### binding required`, `## avoid`).
- controle sem binding emitido como outro componente, para o IR verificar sem nada declarado: rejeitado, porque afirmaria uma equivalência que não existe (`### binding required`).
- layout lido de posições sem a regra, ou sem a diferença registrada no relatório: rejeitado (`## avoid`).
- override, variável ou imagem descartada pela exportação sem marca: rejeitado (`## avoid`).

## what stays for phase 2

- HTML/CSS: a grammar (tree-sitter-html or alike, a new dependency to
  approve) and a CSS reader beyond the indented sass subset; the resolve
  and the two emitters stay.
- Figma: a saved export of the REST API as the fixture, never the network
  in a test; frames and auto layout to `stack`/`grid`, styles to tokens by
  the same measure.
- SwiftUI and Flutter, registered; gpui controls (`on_click`), a React list
  (`items.map`) as a `repeat`.

## phase 2: HTML/CSS and Figma (2026-09-25)

two more sources through the same `Resolution` and both emitters, each
with a fixture, goldens (IR, report, builder code) and a census test.

- HTML/CSS subset: `html.rs` reads the document with tree-sitter-html and
  the CSS of `<style>` and `style=""` with tree-sitter-css (both pinned,
  `=0.23.2` and `=0.25.0`, in `rust/import` only). `resolve_html/`
  cascades `.class`, `#id` and tag rules by specificity then order, as a
  browser does, and lets text styles inherit to every run below. the
  subset: `div`, `section`, `header`, `p`, `span`, `h1` to `h3`, `label`,
  `ul`, `li`, `img`, the controls, and flex (`display`, `flex-direction`,
  `gap`, `align-items`, `justify-content`, `flex-grow`). what CSS writes
  and the corpus sass does not (hex and named colors, `bold`) is
  normalized in `html_map.rs` so the declarations both share still go
  through `css_map::map_decl`, and react's output does not move. grid,
  position, float, media queries, pseudo-classes, selectors that are not
  simple, `<script>`, other `on*` handlers and the browser's default
  styles the stylesheet does not replace are each an impossible entry.
  `onclick="open()"` and `oninput="reply(this.value)"` name their action,
  so the HTML note card binds the same app as the react one.
- Figma: `tool/tasks/figmapull.py` (python stdlib) reads the .fig (a zip of
  fig-kiwi: the kiwi schema in raw deflate, the message in zstd) and
  writes one screen in the REST API's shape, only what the importer reads,
  one key per line, so `figma.rs` gives every key its line (serde_json
  keeps none). instances are expanded from their component, overrides
  matched by `guidPath` (one guid per instance level); what it cannot
  apply is written in `unappliedOverrides` and reported. a color bound to
  a variable is the variable's value in the mode in force, as the design
  tool shows it (the settings screen's status bar is dark in the light
  mode and white in the dark one). `resolve_figma/` maps auto layout to
  the stack, a fixed size to a size token (not a size that hugs, grows or
  stretches), a text to a run, a solid fill to a color. the test never
  reads the .fig nor the network.
- the position rule, `approximated`: a frame with no auto layout whose
  children do not overlap along y is a column in y order (along x, a row
  in x order); each child's offset from where the stack puts it is the
  difference, in px. children that overlap are impossible, in source
  order. vectors and ellipses have no IR component and are impossible.
- additive, and no source that existed produces it: the `gap` and
  `align_items(Align::..)` props (the builder's; the code emitter imports
  `Align` when used), `DropKind::Approximated` for a rule a resolver
  applies approximately, and `<img>` as the `image` component named by
  its `alt` (its `src` impossible: the IR carries no picture). a binding's
  `why` quotes the handler as the source writes it, which for react is the
  same text as before. the react and gpui goldens are unchanged.
- counts: the HTML note card 32 supported, 22 transformed, 17
  approximated, 13 impossible, two bindings (`reply`, `open`); the Figma
  settings screen 251, 27, 119, 196, no binding (a design names no
  capability; its IR verifies with nothing declared and invents no
  control).

## avoid

- a literal in the IR: a value is a token, or it is reported.
- an importer that declares a capability, or a control emitted as
  something else so the IR verifies with nothing declared.
- a tolerance chosen by eye: it is the table's own measure.
- a construct in no entry: the census test fails.
- a layout read from positions without its rule and its difference in the
  report, or an override, a variable or an image dropped by the export
  without a mark.
