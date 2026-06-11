return {
  id  = "stu-glyph-brush",
  typ = "study",
  sts = "pendente",
  dom = "ide",
  dat = "2026-06-11",
  ttl = "render de texto gpu com cache de rasterizacao; o classico do ecossistema rust",
  lnk = { "ref-study", "org-docs" },
  txt = [=[
por que esta aqui: o classico de texto gpu em rust; o draw cache dele
educou glyphon/wgpu-text e meio ecossistema. comparar com o que ja temos
(cosmic-text, glyphon e swash em ref/render-texts).
o que extrair:
- draw cache: texture atlas de glifos, insercao, realocacao, eviction
- api agnostica de render (geracao de vertices, nao draw calls)
- layout de ab_glyph: o minimo de shaping que eles aceitam
- o que envelheceu vs cosmic-text (sem shaping harfbuzz, sem bidi)
o que NAO copiar: usar como dependencia; ja temos cosmic-text/glyphon.
consome: ws-ide (fila)
]=],
}
