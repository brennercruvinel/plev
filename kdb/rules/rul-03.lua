return {
  id  = "rul-03",
  typ = "rule",
  sts = "reference",
  dom = "data-flow",
  dat = "2026-03-13",
  ttl = "fluxo unidirecional",
  lnk = { "idx-rules" },
  txt = [[
toda mutacao de estado segue: userinput -> action (enum tipado) -> handler centralizado -> estado mutado -> re-render. callbacks de componentes emitem actions via actionqueue.emit(), nunca mutam estado diretamente.

mutacao espalhada em callbacks cria estado inconsistente, componente a muta x, callback de b le x no estado antigo, render mostra dado stale. com fluxo unidirecional, toda mutacao e rastreavel via log do action stream, reproduzivel via replay de actions, e o estado e sempre consistente no momento do render porque mutacoes sao batch-processadas entre frames. o modelo e equivalente ao the elm architecture, que demonstrou escalar para aplicacoes de producao desde 2012. a diferenca em φ e que o enum de actions e tipado em rust com exaustividade garantida em compile time.
]],
}
