---
id: adr-0007
title: "render into an sRGB view when the surface format cannot be sRGB"
project: plev
type: decisao
status: aceito
date: 2024-12-04
date_basis: cabecalho
topics: [web-wasm, gpu, renderizacao]
relations: [adr-0004]
evidence: [2a33933]
source: docs/adr/rendering/0007-render-into-an-srgb-view-format.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# render into an sRGB view when the surface format cannot be sRGB

Decide configurar a superfície com o formato base e a variante sRGB em `view_formats`, e alvejar toda passagem de render por uma view sRGB criada por `surface_render_view`. O motivo é que a superfície do WebGPU só aceita formatos não-sRGB, e a codificação na escrita, que o pipeline assumia, era pulada na web, medindo (8,8,8) em vez de (48,48,48). O resultado é o mesmo fundo nas duas plataformas, medido por pixel, e `create_view(&Default::default())` sobre textura de superfície não deve ser usado em código novo.

## context

the WebGPU canvas API only accepts non-sRGB surface formats (bgra8unorm,
rgba8unorm). on desktop the surface itself is sRGB, so the encode-on-write
that the whole pipeline assumes (see linearize-colors-before-the-gpu)
happens implicitly. on the web that encode was silently skipped: linearized
values were written raw, and the page background measured (8,8,8) instead
of (48,48,48). this is the exact inverse of the desktop gamma bug, produced
by the same root assumption.

## decision

the surface is configured with its base format plus the sRGB variant in
`view_formats` (`TextureFormat::add_srgb_suffix()`), and all render passes
target a view created with the sRGB format.

- `GpuContext::surface_format()` returns the view format, so pipelines,
  compositor layer textures and the surface view stay consistent
- `GpuContext::surface_render_view(&output)` is the only sanctioned way to
  create a surface render target. a plain
  `texture.create_view(&Default::default())` inherits the texture's own
  non-sRGB format and silently skips gamma encoding
- on desktop `add_srgb_suffix()` is the identity and the whole mechanism is
  a no-op

## consequences

- web background measured (8,8,8) before, (48,48,48) after, identical to
  desktop. one code path, every platform encodes the same way
- all seven render call sites (engine window, showcase, ide, scene3d,
  snakeGame, both examples) were migrated to `surface_render_view`

## avoid

- never call `create_view(&Default::default())` on a surface texture in
  new code. the failure is invisible on desktop and catastrophic on web,
  which makes it survive review
- do not branch per platform to "fix colors on web". the format mechanism
  expresses the difference once, at configuration time
