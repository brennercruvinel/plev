return {
  id  = "stu-plotters",
  typ = "study",
  sts = "pendente",
  dom = "vis",
  dat = "2026-06-11",
  ttl = "biblioteca de plots em rust puro com multiplos backends (bitmap, svg, wasm)",
  lnk = { "ref-study", "shc-absorcao" },
  txt = [=[
por que esta aqui: eixos, escalas, legendas e tipos de grafico em rust
idiomatico; minerar a geometria e a taxonomia para a aba charts do
showcase.
o que extrair:
- modelo de coordenadas ranged (linear, log, data, categorico) e ticks
- anatomia de chart: area, mesh, eixos, legendas, labels
- tipos de serie (linha, barra, scatter, candlestick, histograma)
- a abstracao de backend de desenho (DrawingBackend) como contrato
o que NAO copiar: estilo visual (usamos hoff); o backend proprio.
consome: ws-showcase
]=],
}
