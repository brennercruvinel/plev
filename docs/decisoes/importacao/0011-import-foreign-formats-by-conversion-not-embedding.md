---
id: adr-0011
title: "import foreign formats by conversion, never by embedding a foreign runtime"
project: plev
type: decisao
status: aceito
date: 2025-06-10
date_basis: cabecalho
topics: [importacao, formatos, animacao]
relations: []
evidence: [a044613]
source: docs/adr/importing/0011-import-foreign-formats-by-conversion-not-embedding.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# import foreign formats by conversion, never by embedding a foreign runtime

Decide importar formatos estrangeiros por conversão única, nunca por incorporação de um runtime estrangeiro: o `lot` lê o JSON do Lottie uma vez, offline, e grava `.monster`. O motivo é que um player embutido traz o próprio motor e paga o custo de parse a cada quadro. O resultado é que a reprodução roda no `monster::Player` sem nenhum código Lottie, e o importador registra o que não converte em vez de fingir.

## context

the obvious way to "support lottie" is to embed a lottie player and feed it
the json every frame. that is what the anti-lesson warns against: the
foreign runtime ships its own engine (thorvg embeds a js engine for
scripted lottie), the json stays the format, and the parse cost is paid on
every frame forever. an early attempt in this project did exactly that and
was correctly rejected: it was a foreign player wearing our window.

## decision

`lot` reads the lottie json exactly once, offline, and converts it to our
`.monster` format. after conversion, playback runs on `monster::Player`
over our binary, and no lottie code executes. the json dies at the door.

the bridge (`lot::cnv`) samples the composition through the renderer, dedups
the tessellated geometry by exact quantized bytes into the asset table (a
static shape becomes one asset referenced by every frame, so the delta
encoder emits zero bytes for it), runs delta discovery, and encodes. the
importer depends only on plev, serde, log, and our own codec; it contains
no embedded animation engine.

the same shape applies to any future importer (swf is the next candidate,
and its display list maps one to one onto our scene model).

## consequences

- the user's five corpus files convert and play on our player, proven on
  screen, not as a json embed.
- the importer is small (about 1000 lines of rust) and honest about its
  gaps: unsupported lottie features log once and are skipped, never
  silently faked.
- the size cost of motion lives in the asset table, which is where the v1
  morph-track lever will act.

## avoid

- never link a foreign format's runtime into the playback path.
- never keep the foreign format as the on-disk format. convert once, store
  ours, play ours.
- when an importer cannot represent a feature, log it once and drop it
  visibly; do not approximate it into a silent wrong result without saying
  so.
