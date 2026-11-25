---
id: task-101
title: "task-101: on-demand fonts and deterministic fallback"
project: plev
type: task
status: done
date: 2026-10-27
date_basis: git
recorded: 2026-10-31
topics: [fonts, text, web-wasm]
relations: [adr-0078, adr-0079, task-74]
evidence: [f79400d1, e2bdaa45, bee96c97, 9bdc266f, c243b124]
source: reconstruction
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-101: on-demand fonts and deterministic fallback

The delivery separated two decisions: reducing the initial bytes of the wasm and
making the desktop prefer the same embedded faces used by the web.

## Web loading

The wasm keeps the embedded Latin faces: Inclusive Sans, Inter and codicons
(`rust/engine/src/drawing/text/fonts.rs`, lines 206 to 226). The text requests
the Noto packages for the scripts it uses; each file must match the expected
sha256, and a package enters only in full (`on_demand.rs`, lines 218 to 237).
The arrival of fonts increments a generation (line 243), invalidates
measurements and shaping, and wakes the loop (lines 246 and 273 to 277). The
measurer and the renderer load the faces in the same order as `PACKS`,
regardless of the order in which the network delivers them (`on_demand.rs`,
lines 9 to 13 and 254 to 271).

The [ADR 0078](../../adr/Rendering/0078-the-web-fetches-the-noto-faces-on-demand/0078-the-web-fetches-the-noto-faces-on-demand.md)
records the size measured at delivery (6,97 MB of fonts in the wasm, line 23) and
the need to serve `fonts/` next to the build (line 39). A file that does not
match the hash keeps the package out; the text continues with fallback or a
missing glyph.

## Desktop

`EmbeddedFirst` chooses, for each script covered by the embedded faces, that
script's Noto family ahead of the platform lists; the remaining scripts keep the
system lists as fallback (`fonts.rs`, lines 160 to 199). The
[ADR 0079](../../adr/Rendering/0079-a-desktop-draws-the-scripts-with-the-embedded-faces/0079-a-desktop-draws-the-scripts-with-the-embedded-faces.md)
records the comparison of the Typography page, byte-for-byte identical between the
desktop headless render and the web build (lines 50 and 51).

## Code and learning

[rust/engine/src/drawing/text/on_demand.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/drawing/text/on_demand.rs) and
[rust/engine/src/drawing/text/fonts.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/drawing/text/fonts.rs).

Invalidating only the shaping cache is not enough: `resolve_layer_text` marks all
layers as dirty, because a clean layer still holds the text as it was drawn
before the face arrived (`f79400d1`).

## Evidence

- [`f79400d1`](https://github.com/brennercruvinel/plev/commit/f79400d1614813f8e2ed7d465da958b3a0d99522)
- [`e2bdaa45`](https://github.com/brennercruvinel/plev/commit/e2bdaa456eb7c5c20f2386e30908c9e853b532fe)
- [`bee96c97`](https://github.com/brennercruvinel/plev/commit/bee96c970a711d43e8754ea9683bb5f3dd7ddb13)
- [`9bdc266f`](https://github.com/brennercruvinel/plev/commit/9bdc266f916b29702c5524973e5bdd2a50c909e3) (merge of PR #139)
- [`c243b124`](https://github.com/brennercruvinel/plev/commit/c243b1248ac191f8974a7727b2822b9d48a3aee6) (merge of PR #140)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0078: the web build fetches the Noto faces when a text first needs them](../../adr/Rendering/0078-the-web-fetches-the-noto-faces-on-demand/0078-the-web-fetches-the-noto-faces-on-demand.md)
- [ADR 0079: a desktop draws the scripts with the embedded faces, not the system's](../../adr/Rendering/0079-a-desktop-draws-the-scripts-with-the-embedded-faces/0079-a-desktop-draws-the-scripts-with-the-embedded-faces.md)
- [task-74: text and editing](../task-74-text-and-editing/task-74-text-and-editing.md)
<!-- /related -->
