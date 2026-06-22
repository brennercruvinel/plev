return {
  id  = "rul-04",
  typ = "rule",
  sts = "reference",
  dom = "composition",
  dat = "2026-03-13",
  ttl = "composicao sobre heranca",
  lnk = { "idx-rules" },
  txt = [[
componentes sao funcoes fn(props) -> element. sem trait objects de widget como interface publica na camada de app, sem hierarquia de tipos, sem dyn widget em composicao estatica de ui. componente complexo e composicao de componentes simples via child().

para listas com tipos heterogeneos de item, o padrao correto e enum com variantes: listitem::text(textitem), listitem::image(imageitem), listitem::action(actionitem) com match exaustivo no render. exaustividade em compile time, zero dispatch dinamico.

para sistemas de plugin onde o tipo e genuinamente desconhecido em compile time, box<dyn component> e permitido exclusivamente no pluginregistry, modulo isolado em src/plugins/registry.rs, com interface publica que expoe apenas fn registered_components() -> vec<componentdescriptor>. componentdescriptor e struct serializable com metadata estatico. o box<dyn component> nunca escapa do registry para o codigo de composicao de ui. sem essa fronteira arquitetural explicita, camada de plugins expande ate contaminar a composicao estatica.
]],
}
