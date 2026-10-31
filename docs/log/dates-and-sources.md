---
id: dates-and-sources
title: "Dates and sources"
project: plev
type: record
status: reference
date: 2026-10-30
date_basis: git
topics: [editorial, git]
relations: [adr-0001, adr-0002, learning-apps-bypass-the-engine, learning-the-two-gamma-bugs, truth-2026-09, task-26, task-40, hoff-visual-tokens]
source: reconstruction
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [176dd691, 704412f1, b1e820ba, 95d53881, 1a776d04, ec0a224c]
---

# Dates and sources

The archive contains dates declared since 2021. The available Git history has a
root commit on 10 June 2026. These two timelines represent different sources
and should not be merged automatically.

## Dates revised later

Commit `704412f1` changed the header of the note on `View` from 2026-03-08 to
2021-03-06, and the [layout ADR](../adr/rendering/0002-content-driven-layout-not-fixed-constants.md) from 2026-06-10 to 2024-09-18. The change documents a
retrospective revision of the date; it does not, by itself, prove when the
technical work took place.

Commit `b1e820ba` remapped hash references after the history purge. For example,
`34a9893` was replaced by `55c8aa7` in the [ADR on asynchronous GPU initialization](../adr/rendering/0001-async-gpu-init-and-single-wasm-entry.md). An old reference may have a recoverable match even if
the whole earlier history no longer exists.

## Reading the timeline

The pages keep the declared date and its basis in the YAML. The timeline orders
these declarations; it does not certify each date as contemporary with the event.
When the text and the header cover distinct periods, the internal entries may
have their own dates. A missing date is represented as missing, never filled in
for editorial convenience.

## Recovered material

[Task 40](../tasks/task-40-paper-arxiv-full-text/README.md) was found before its deletion in `95d53881`: its goal was to write the full article, with the checklist still pending. The completed outline of [task 26](../tasks/task-26-arxiv-paper/README.md) does not show that task 40 was also completed.

The draft, the outline, the benchmarks and the article's differentiators were removed in `1a776d04`; [`truth.md`](truth.md), in `ec0a224c`. Their recovered versions are historical sources. The texts on gamma, on apps bypassing the engine and on HOFF tokens were also recovered. The exact paths and revisions are recorded in `source` and in the catalog.

## Evidence

- [`704412f1`](https://github.com/brennercruvinel/plev/commit/704412f12300e0a42792745bfd6daccbfb9f666e)
- [`b1e820ba`](https://github.com/brennercruvinel/plev/commit/b1e820baf47d0be24e30890fefbcd0cfdceeb1f3)
- [`95d53881`](https://github.com/brennercruvinel/plev/commit/95d5388140568732f472c6be86205df113c8fcce)
- [`1a776d04`](https://github.com/brennercruvinel/plev/commit/1a776d04c6b392a3afad72e5b8223c116937128d)
- [`ec0a224c`](https://github.com/brennercruvinel/plev/commit/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713)
