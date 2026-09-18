---
type: adr
status: accepted
tags: [urnaui, media, ffmpeg, av1, blobs, worker]
date: 2026-09-18
---

# urnaui: decode media frames in place with ffmpeg's subfile protocol

## context

a forge-built `.urna` media corpus (the 38k-card mtg benchmark is the
reference) keeps every image as one frame of an av1 stream inlined in the
blob_data section (0x17): one mp4 of ~500 MB inside a ~530 MB file, one
frame per chunk, the overlay (0x16) mapping chunk `n` to frame `n`. the
explorer wants to show that frame next to a hit or a chunk.

three ways were on the table:

- decode av1 in-process (dav1d bindings + an mp4 demuxer): a C build, a
  new dependency tree in a crate that stays gpu-first, and a wasm story
  that does not exist.
- export the blob once to a cache dir and point ffmpeg at the copy: a
  500 MB write on first open of every corpus, and a cache to invalidate.
- hand ffmpeg the byte range of the blob inside the `.urna` itself:
  ffmpeg's `subfile` protocol reads `start..end` of any file as if it were
  the whole input, so the blob is never copied.

the forge encodes stills at 1 fps, all-intra (`keyint = 1`): every frame
is a keyframe and frame `n` sits at `n` seconds, so `-ss n` seeks straight
to it without decoding what came before.

## decision

- the worker decodes one frame per request with
  `ffmpeg -ss <frame> -i "subfile,,start,<abs>,end,<abs+len>,,:<path>"
  -frames:v 1 -vf scale=… -c:v png pipe:1`, the absolute range resolved at
  open from the 0x17 offset table (`UrnaBackend::blob_range`). the png
  bytes go into the engine's image atlas (`load_image_bytes`) and the
  panels draw the handle.
- ffmpeg is an optional tool, probed once per open. absent, the media
  corpus opens and searches exactly the same; the preview box says why
  it is empty.
- a frame is requested by the screen that shows it (`wanted_frame`), once
  per ordinal, after every input event and every drained worker batch;
  the result is cached per chunk for the life of the open db, `Err` included
  so a failed decode is never retried in a loop.
- the same machinery serves `urna media --export` inside the app: the
  blob is sha256-verified against its blob_refs record before a byte is
  written (`export_blob`), never after.

## consequences

- a card renders in ~60 ms on the mtg corpus with zero bytes copied out
  of the file; opening a 530 MB corpus costs one mmap, not a 500 MB read.
- the decode runs on the worker thread, so a slow seek never blocks a
  frame; the subprocess helper drains stdout on its own thread (a png is
  bigger than the 64 KiB pipe buffer, see the worker-results adr).
- the image atlas is grow-only with no eviction: previews are capped at
  340 px on the long edge, which is roughly 800 distinct frames per
  session before the 8192² atlas is full. eviction is engine work, not
  urnaui work.
- the web build lists media and spans but shows no frames (no ffmpeg in
  the browser); a wasm av1 decoder is a separate decision.

## avoid

- do not export the blob to decode it; do not decode av1 in-process
  without an adr that also answers wasm.
- do not read the frame index from anything but the 0x16 overlay span:
  `byte_start..byte_end == n..n+1` is the contract the forge writes and
  the only shape `BlobSpan::frame` accepts.
