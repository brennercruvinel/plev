return {
  id  = "rul-05",
  typ = "rule",
  sts = "reference",
  dom = "layout",
  dat = "2026-03-13",
  ttl = "layout declarativo nunca manual",
  lnk = { "idx-rules" },
  txt = [[
posicionamento usa exclusivamente as primitivas de layout do φ via taffy: col, row, gap, p, w, h, grow, shrink, basis. zero coordenadas absolutas calculadas manualmente. zero offsets hardcoded.

φ executa em 6 plataformas com densidades de pixel radicalmente diferentes, retina 2x, android mdpi ate xxxhdpi, browser zoom, hidpi linux. coordenadas manuais quebram em variacoes de tela, dpi, orientacao e resize. taffy resolve constraints via flexbox e grid automaticamente com o mesmo algoritmo do browser mas sem dom. posicao manual cria bugs que so aparecem em devices especificos, os mais caros de diagnosticar porque nao sao reproduziveis em desenvolvimento.
]],
}
