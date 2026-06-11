return {
  id  = "stu-warp",
  typ = "study",
  sts = "pendente",
  dom = "ide",
  dat = "2026-06-11",
  ttl = "terminal/ide em rust com render gpu proprio; repo recem-aberto (open source)",
  lnk = { "ref-study", "org-docs" },
  txt = [=[
por que esta aqui: arquitetura rust de ide/terminal; "a falha e ser
html" e o warp e o contraponto: ui propria em rust com render gpu.
clone shallow ficou em 752MB, abaixo do gate de 1.5GB, mantido.
o que extrair:
- a arquitetura de blocos (comando+output como unidade de dado)
- o pipeline de render gpu de texto e ui propria (sem webview)
- modelo de input/editor dentro do terminal (multicursor, completions)
- como estruturam um app rust gigante (crates, camadas, eventos)
o que NAO copiar: agentic workflows/cloud/login; nada de produto, so
engenharia.
consome: ws-ide (fila)
]=],
}
