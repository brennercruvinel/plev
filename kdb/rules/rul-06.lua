return {
  id  = "rul-06",
  typ = "rule",
  sts = "reference",
  dom = "navigation",
  dat = "2026-03-13",
  ttl = "navegacao como enum",
  lnk = { "idx-rules" },
  txt = [[
telas e rotas da app sao variantes de um enum rust. transicao e mutar o valor do enum no estado. o render faz match exaustivo no enum para decidir o que renderizar. zero string matching, zero router framework.

rust garante exaustividade no match, adicionar uma tela nova e esquecer de trata-la e erro de compilacao, nao bug em producao. strings sao frageis: typo em /setings compila e mostra tela branca. enums com dados associados, screen::userprofile { id: userid }, screen::documenteditor { doc_id: docid, mode: editmode }, carregam parametros type-safe sem runtime de routing, sem regex de path matching, verificaveis em compile time.
]],
}
