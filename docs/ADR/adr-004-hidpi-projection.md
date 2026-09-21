---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2024-06-17
domain: rendering
commit: 73c46ff
---

# adr-004: adotamos projecao ortografica em logical pixels para apps hidpi

## contexto

o engine plev renderiza via wgpu com projecao ortografica configurada em
physical pixels (surface_config.width/height). o showcase principal
funciona corretamente porque dimensoes e font sizes sao calibrados para
physical pixels. o gitbutler-plev, porem, utiliza constantes em logical
pixels (sidebar 48px, header 48px) resultando em elementos minusculos
em displays retina (scale_factor=2, physical=2560x1600, logical=1280x800).

## decisao

adicionamos `GpuContext::set_projection(logical_w, logical_h)` que
sobrescreve o projection buffer com dimensoes logical apos o resize
configurar a surface com physical pixels.

## consequencias

1. apps que usam logical pixels chamam `set_projection()` apos `resize()`.
   o engine principal nao precisa mudar (usa physical diretamente).
2. cursor positions devem ser divididos por scale_factor no app.
3. `ScaleFactorChanged` deve ser tratado para atualizar o fator.
4. o pattern e: surface em physical, projecao em logical, coordenadas
   de scene graph em logical. a GPU faz o upscale automaticamente.

> **atualizacao 2026-08-30:** a consequencia 4 valia para geometria
> (SDF/vetorial escala sem perda) mas NAO para texto: glyph bitmaps eram
> rasterizados em tamanho logical e esticados pela projecao — texto
> visivelmente borrado em retina. corrigido: `TextSystem` rasteriza
> glyphs em `font_size * raster_scale` (derivado de `clip_scale()` no
> `resolve_layer_text`, central para todos os apps) e mapeia os quads de
> volta para logical. o detector de fallback de fonte (warn quando um
> glyph rasteriza de uma face nao-embutida) foi adicionado junto.
