return {
  id  = "stu-gritql",
  typ = "study",
  sts = "pendente",
  dom = "parser",
  dat = "2026-06-11",
  ttl = "linguagem declarativa de query para buscar e reescrever codigo em massa",
  lnk = { "ref-study", "prs-transpiler" },
  txt = [=[
por que esta aqui: transformacao em massa por query; candidato a motor
de reescrita do transpiler (o brenner ja usou muito). patterns
declarativos sobre tree-sitter com rewrite embutido.
o que extrair:
- semantica de pattern/rewrite: capturas, where, condicoes compostas
- o motor sobre tree-sitter: como aplica rewrites preservando trivia
- migracoes em lote: como encadeia regras em workflows
- a fronteira do declarativo: quando uma regra precisa de codigo
o que NAO copiar: a cli e os servicos cloud da grit.
consome: ws-parser
]=],
}
