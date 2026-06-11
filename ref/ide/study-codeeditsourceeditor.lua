return {
  id  = "stu-codeeditsourceeditor",
  typ = "study",
  sts = "pendente",
  dom = "ide",
  dat = "2026-06-11",
  ttl = "editor de codigo swift estilo xcode, highlight via tree-sitter (codeedit)",
  lnk = { "ref-study", "org-docs" },
  txt = [=[
por que esta aqui: modelo de editor de texto de codigo bem fatorado:
highlight tree-sitter, minimap, mensagens inline, find/replace, bracket
matching; o checklist de features de um source editor serio.
o que extrair:
- integracao tree-sitter para highlight incremental por tema
- minimap: como renderiza e sincroniza barato
- inline diagnostics (warnings/errors no texto) e current line highlight
- a separacao texto/layout/render do textview subjacente
o que NAO copiar: appkit/swift specifics; so a arquitetura de camadas.
consome: ws-ide (fila)
]=],
}
