return {
  id  = "stu-svelte",
  typ = "study",
  sts = "pendente",
  dom = "mining",
  dat = "2026-06-11",
  ttl = "compilador de componentes; reatividade resolvida em compile time (runes)",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: o argumento canonico de "framework e compilador, nao
runtime"; runes ($state, $derived, $effect) movem a reatividade pra
analise estatica.
o que extrair:
- runes: como $state/$derived/$effect viram codigo cirurgico
- o pipeline do compilador: parse -> analyze -> transform (packages/svelte/src/compiler)
- o output: updates diretos no dom sem diff, granularidade por binding
- como o compilador decide o que e reativo sem anotacao do usuario
o que NAO copiar: a sintaxe de template e o ecossistema sveltekit.
consome: tese
]=],
}
