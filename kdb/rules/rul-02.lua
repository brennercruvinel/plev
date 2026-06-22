return {
  id  = "rul-02",
  typ = "rule",
  sts = "reference",
  dom = "state",
  dat = "2026-03-13",
  ttl = "estado de dominio fora do plev",
  lnk = { "idx-rules" },
  txt = [[
signals do plev armazenam exclusivamente estado de ui: scroll position, painel aberto, hover state, selecao ativa, modo de edicao. dados de dominio, usuario autenticado, lista de entidades, sessao, configuracao persistida, vivem em structs rust puros, owned pela app, sem dependencia de plev.

estado de dominio dentro de signals cria dependencia circular: testar logica de negocio exige instanciar o sistema reativo do plev, que exige event loop, que exige window. zero testes unitarios na pratica porque o setup e proibitivo.

para estado hibrido, dados de dominio que a ui precisa transformar localmente, como lista filtrada ou ordenada, o protocolo padrao e derivacao pura: o dominio expoe o dado bruto via referencia imutavel, a ui computa a view como funcao pura no momento do render, sem armazenar o resultado derivado como estado.

excecao obrigatoria para memoizacao: quando a transformacao opera sobre colecoes grandes ou tem custo computacional mensuravel, usar no memoize equivalente ao do xilem, que prune a view tree quando as dependencias nao mudaram entre ciclos. memoizacao e cache controlado com invalidacao explicita por dependencia, nao e estado de dominio, nao viola a separacao. derivacao e funcao nao cache e o padrao; memoizacao com dependencias declaradas e a excecao justificada por profile, nao por preferencia.
]],
}
