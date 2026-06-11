return {
  id  = "stu-topiary",
  typ = "study",
  sts = "pendente",
  dom = "parser",
  dat = "2026-06-11",
  ttl = "formatador uniforme guiado por queries tree-sitter (tweag)",
  lnk = { "ref-study", "prs-transpiler" },
  txt = [=[
por que esta aqui: regras declarativas sobre arvores tree-sitter; o
padrao de "regra sobre AST" que o transpiler precisa, aplicado aqui a
formatacao mas generalizavel a transformacao.
o que extrair:
- queries .scm com capturas que viram instrucoes de formatacao
- a arquitetura gramatica -> regras -> render (camadas separadas)
- quanto custa suportar uma linguagem nova (so um arquivo de regras)
- tratamento de comentarios e nos "extra" fora da gramatica
o que NAO copiar: o foco em formatacao; nosso alvo e transformacao
semantica, formatacao e subproduto.
consome: ws-parser
]=],
}
