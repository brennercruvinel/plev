---
type: adr
status: accepted
tags: [parser, transpiler, droplist, honesty, diagnostics]
date: 2026-06-12
commit: 5eecb0a
---

# the transpiler reports every unmapped construct, never drops silently

## context

a universal ui transpiler can never cover 100% of the source language on
day one. css alone has long-tail features the target engine does not have
(z-index, media queries, gradient masks). a transpiler facing an unsupported
construct has three options: stop with an error (useless, every real file
has something unsupported), drop it silently (the file looks converted but
is subtly wrong, with no trail), or convert what it can and hand back the
exact list of what it could not.

silent dropping is how transpilers lie. it is the same failure class as a
demo that bypasses the engine: the result looks complete until a later
visual bug sends you hunting property by property with no starting point.

## decision

`parser` (parse, resolve, emit) emits the converted plev builder code plus a
droplist: every source construct it did not represent, each entry carrying
the file, line, and reason. the counts are frozen in tests (`mapped == 51`,
`dropped.len() == 51` for the corpus card), so a mapper change that starts
dropping something new breaks the build. the list can only shrink by
implementing the missing feature, never grow in silence.

the emitted code obeys the engine manual: one TextStyle per run, content
driven layout, theme tokens where a color hits the palette. the parser is a
user of the engine's rules, not a bypass of them.

## consequences

- running the parser on the owner's real corpus (40 components across two
  apps) produced 402 mapped properties and 709 droplist entries, zero
  crashes; the gaps are named, not hidden.
- the goldens are validated against the corpus originals, and the emitted
  code compiles and renders (examples preview and transpile prove it live).
- the droplist is the roadmap: the five recurring entry types are exactly
  the next features to implement.

## avoid

- never let an unmapped construct vanish without a droplist entry with
  file:line and a reason.
- never let the mapped/dropped counts drift without a conscious test
  update; they are an api contract.
- never emit code that bypasses the engine manual to fake coverage.
