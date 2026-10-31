---
id: adr-0078
title: "the web build fetches the Noto faces when a text first needs them"
project: plev
type: adr
status: accepted
date: 2026-10-27
date_basis: header
topics: [fonts, text, web-wasm]
relations: [adr-0071, adr-0079]
source: docs/adr/rendering/0078-the-web-fetches-the-noto-faces-on-demand.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [f79400d1, 71ee6291]
---

# the web build fetches the Noto faces when a text first needs them

Decides that the web version fetches the Noto faces (six scripts and the emoji) from the server when a text needs them, instead of embedding them in the wasm. The uigallery wasm dropped from 13.606.811 to 7.060.619 bytes (2.849.177 with gzip -9), about 65% less to download, and each file enters use only whole and with its sha256 verified. The cost is that text in a script not yet downloaded shows as tofu for a few frames until the face arrives; desktop, android and iOS keep embedding the faces.

## context

[adr embedded-fonts-by-script (0071)](0071-embedded-fonts-by-script.md) embedded the Noto Sans subsets for
arabic, hebrew, devanagari, SC, JP and KR, and the Noto Color Emoji, on
every target. on the web that put 6.97 MB of fonts into the wasm every
page downloads, whether or not it ever shows those scripts: the
uigallery's wasm weighed 13,606,811 bytes, 8,208,387 with gzip -9 (main
at 71ee6291). issue #137 asked to fetch them instead.

## decision

- on wasm the engine embeds the latin faces only (Inclusive Sans, Inter,
  codicons); `NOTO` and `NOTO_EMOJI` in `drawing/text/fonts.rs` are empty
  there. a desktop, android and ios embed them as before.
- `drawing/text/on_demand.rs` holds the packs in the order the desktop
  loads them (arabic, hebrew, devanagari, SC, JP, KR, emoji), each file
  with the sha256 it must hash to. a text asks for the packs of its
  scripts when it is shaped (`want`, from the measurer's `prepare` and
  `shape` and the renderer's `ensure_shaped`); an ascii text asks for
  nothing. han asks for SC, kana for JP, hangul for KR.
- each pack is asked for once. the web fetches its files from `fonts/`
  beside the page (`set_faces_url` moves it) and takes them only whole
  and hashed right; a pack that fails is logged and left out.
- an arrival bumps a generation. the measurer's context (now its own
  file, `measure_context.rs`, the one door `with_ctx`) and the
  `TextSystem` compare it with theirs; when it moved they load every
  arrived face again in the pack order, whatever order the network
  delivered them in, and drop what they measured, shaped or rasterized
  without them. `resolve_layer_text` then marks every layer dirty, since a
  layer whose scene did not change still holds the text as first drawn.
- the loop sleeps between frames, so a shell registers a waker
  (`on_faces_arrived`): the engine's `App` sends `AppEvent::FacesArrived`
  and redraws every window, the uigallery sends `UserEvent::Faces`.
- the uigallery and the urnadbapp copy `rust/engine/assets/fonts` into
  their `dist/fonts` (trunk `copy-dir`), the licenses with the faces.
- the provisional `experiment-no-noto` feature goes.

## consequences

- the uigallery's wasm is 7,060,619 bytes, 2,849,177 with gzip -9: 65%
  less to download (measured on this branch, `trunk build --release`).
- in headless chromium with WebGPU on Metal, the Theme page fetched no
  font and the Typography page fetched the 13 files of the seven packs
  and drew every script with them, color emoji included.
- a text in a script not fetched yet is shaped once without its face
  (tofu) and drawn again when the face arrives, a few frames later on a
  local server, more on a slow network.
- the faces are the desktop's own bytes: a test holds the sha256 table to
  the embedded files and to the files the web serves, so they never
  drift; others hold the pack detection, the refusal of a file that does
  not hash, and that a measurement asks for its packs.
- the Typography page gained a SCRIPTS section, one line per pack. a
  desktop still draws those lines with the system's faces, as adr 0071
  keeps it, so the web and the desktop measure them differently; making
  the desktop prefer the embedded faces is that [separate decision](0079-a-desktop-draws-the-scripts-with-the-embedded-faces.md).

## alternatives

- each app choosing by feature which scripts it embeds: no network, but
  an app that shows user text (the urna explorer) cannot know its scripts
  ahead, and every app would carry the choice.
- fetching on the desktop too: it embeds them already, and a desktop app
  works offline.
- the woff2 versions of the subsets: smaller on the wire, but fontdb reads
  sfnt only, and the server can compress the files as it serves them.
