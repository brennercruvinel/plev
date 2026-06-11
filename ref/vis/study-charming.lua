return {
  id  = "stu-charming",
  typ = "study",
  sts = "pendente",
  dom = "vis",
  dat = "2026-06-11",
  ttl = "wrapper rust declarativo do apache echarts",
  lnk = { "ref-study", "shc-absorcao" },
  txt = [=[
por que esta aqui: o vocabulario declarativo mais completo de graficos
(herdado do echarts) expresso em rust; minerar a taxonomia de tipos,
eixos e legendas para a aba charts.
o que extrair:
- o modelo option/series/axis/legend/tooltip como api declarativa
- cobertura de tipos de grafico (a lista e o checklist da aba charts)
- como a api rust tipa um schema json gigante sem virar sopa
- composicao de graficos (grid, multiplos eixos, dataset)
o que NAO copiar: a dependencia echarts/js no render; estilo visual
(usamos hoff).
consome: ws-showcase
]=],
}
