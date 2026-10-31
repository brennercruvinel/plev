---
id: adr-0079
title: "a desktop draws the scripts with the embedded faces, not the system's"
project: plev
type: adr
status: accepted
date: 2026-10-27
date_basis: header
recorded: 2026-10-27
topics: [fonts, text, determinism, platform]
relations: [adr-0071, adr-0078]
evidence: [bee96c97]
source: docs/adr/rendering/0079-a-desktop-draws-the-scripts-with-the-embedded-faces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# a desktop draws the scripts with the embedded faces, not the system's

Decides that the engine builds each `FontSystem` with its own fallback, `EmbeddedFirst`, in which Arabic, Hebrew, Devanagari, Han, Kana and Hangul are answered only by the embedded Noto face, and that the desktop draws these scripts with it, instead of the system faces. Before, each machine drew these scripts with local fonts, and the web, with the embedded faces, measured them differently. The desktop keeps the locale and the system faces for the rest of the text, and `unicode-script` becomes a direct dependency of the engine. A test measures one line per script on the desktop path and requires the Noto face.

## context

[adr embedded-fonts-by-script (0071)](0071-embedded-fonts-by-script.md) embedded the Noto faces and kept the
desktop on the system's faces for those scripts, "a separate decision".
measured on 2026-10-27 (macOS 27.0.1, M4), `TextMeasurer::resolved_faces`
on the desktop path shaped arabic with Geeza Pro, hebrew with Arial,
devanagari with Devanagari Sangam MN, chinese with Arial Unicode MS,
japanese with Hiragino Sans, korean with Apple SD Gothic Neo and emoji with
Apple Color Emoji: cosmic-text's `PlatformFallback` names the platform's
face per script, and a query reaches that list before the embedded faces.
every machine drew those scripts with its own fonts, and since [adr 0078](0078-the-web-fetches-the-noto-faces-on-demand.md) the
web, with the embedded faces alone, measures them otherwise.

## decision

- the engine builds every `FontSystem` with its own `Fallback`
  (`EmbeddedFirst`, `drawing/text/fonts.rs`): arabic, hebrew, devanagari,
  han, kana and hangul name their Noto face alone (han SC, kana JP, hangul
  KR); every other script keeps the platform's list; the common list
  starts with Noto Color Emoji, then the platform's.
- a script names its Noto face alone, so a glyph a subset lacks still
  reaches the platform's faces past the common list.
- the desktop keeps the system's locale and faces (`FontSystem::new()`
  taken apart by `into_locale_and_db`); wasm, android and ios use the same
  rule over the embedded faces.
- `unicode-script` 0.5, already in Cargo.lock under cosmic-text, becomes a
  direct dependency of the engine: `Fallback::script_fallback` speaks its
  `Script`, which cosmic-text does not re-export.

## consequences

- the uigallery's Typography page, at 1280x900 in hoff, is equal in every
  byte between the desktop's headless render and the web build in
  headless chromium on Metal (`regress compare`), the SCRIPTS section
  included. with the platform's lists the desktop shapes those lines from
  other faces (the next point), so they cannot match; the two frames were
  not compared in that state.
- a test shapes one line per script on the desktop path and holds it to
  its Noto face; it fails with the platform's lists (Geeza Pro).
- the text audit golden moves by one glyph: in "こんにちは、世界" the
  ideographic comma now comes from SC, in the same han run as 世界, which
  was SC already; the faces and the width do not move.
- latin text does not move: the 176 regress baselines hold.
- a person who wants their system's face for these scripts no longer gets
  it; system fonts as an option by setting stays out, as in adr 0071.

## alternatives

- keeping the platform's lists: the desktop and the web measure these
  scripts differently, and two desktops differently from each other.
- dropping the system's faces from the desktop database: the scripts the
  embedded faces do not cover (thai, greek beyond the latin faces, ...)
  would lose their only faces.
