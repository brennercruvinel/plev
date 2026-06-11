return {
  id  = "stu-difftastic",
  typ = "study",
  sts = "pendente",
  dom = "parser",
  dat = "2026-06-11",
  ttl = "diff estrutural que compara arvores de sintaxe, nao linhas de texto",
  lnk = { "ref-study", "prs-transpiler" },
  txt = [=[
por que esta aqui: diff estrutural (arvores, nao texto); junto com o
weave, a tecnica de VERIFICACAO do output do transpiler. ignora mudanca
de indentacao e mostra mudanca real.
o que extrair:
- o algoritmo: diff como caminho minimo num grafo de edicoes de arvore
- como degrada com gracas (fallback textual quando a arvore explode)
- apresentacao lado a lado alinhada por no, nao por linha
- o uso das gramaticas tree-sitter como fonte unica de parsing
o que NAO copiar: a ui de terminal; nossa verificacao e programatica.
consome: ws-parser (verificacao)
]=],
}
