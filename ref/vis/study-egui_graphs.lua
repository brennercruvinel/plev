return {
  id  = "stu-egui_graphs",
  typ = "study",
  sts = "pendente",
  dom = "vis",
  dat = "2026-06-11",
  ttl = "widget egui de visualizacao interativa de grafos sobre petgraph",
  lnk = { "ref-study", "shc-absorcao" },
  txt = [=[
por que esta aqui: visualizacao interativa de grafos (node-link) em
imediate mode; minerar interacao e layout para graficos de rede na aba
charts e para visualizar arvores de ui no parser.
o que extrair:
- interacao: drag de nos, pan/zoom, selecao, hover
- separacao dados (petgraph) vs apresentacao (widget)
- layouts de grafo disponiveis e onde o calculo roda
- como mantem responsivo em imediate mode com grafos grandes
o que NAO copiar: estilo visual (usamos hoff); acoplamento ao egui.
consome: ws-showcase
]=],
}
