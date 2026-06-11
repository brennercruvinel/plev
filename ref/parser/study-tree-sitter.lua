return {
  id  = "stu-tree-sitter",
  typ = "study",
  sts = "pendente",
  dom = "parser",
  dat = "2026-06-11",
  ttl = "gerador de parsers e biblioteca de parsing incremental",
  lnk = { "ref-study", "prs-transpiler" },
  txt = [=[
por que esta aqui: o parser incremental; a fundacao de topiary, gritql e
difftastic. gramaticas tsx/css/rust prontas; nao reinventamos parsing.
o que extrair:
- api de parsing incremental: edit + reparse, custo proporcional a edicao
- queries .scm: o padrao de captura que todo o ecossistema reusa
- bindings rust (tree-sitter crate) e o ciclo de vida de Tree/Node
- como gramaticas externas (tsx, css, rust) se pluggam
- error recovery: arvores com ERROR/MISSING continuam utilizaveis
o que NAO copiar: nao reimplementar parser proprio; usar como fundacao.
consome: ws-parser
]=],
}
