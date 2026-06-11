return {
  id  = "stu-ark",
  typ = "study",
  sts = "pendente",
  dom = "mining",
  dat = "2026-06-11",
  ttl = "biblioteca headless multi-framework construida sobre as machines do zag",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: como o zag vira componentes multi-framework com
paridade perfeita (react, solid, vue, svelte); a prova de que logica
unica + adaptadores finos funciona em escala de design system.
o que extrair:
- a camada adaptadora por framework: o que e gerado, o que e escrito
- anatomia de componente headless (parts, context, props forwarding)
- como garantem paridade de api entre frameworks (testes? codegen?)
- versionamento de machines compartilhadas entre 4 targets
o que NAO copiar: a superficie de api 1:1; nosso target e plev, nao dom.
consome: tese
]=],
}
