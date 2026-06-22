return {
  id  = "rul-15",
  typ = "rule",
  sts = "reference",
  dom = "testing",
  dat = "2026-03-13",
  ttl = "testabilidade por camada sem gpu",
  lnk = { "idx-rules" },
  txt = [[
tres niveis obrigatorios. dominio: unit tests puros com test, sem plev, sem window, em milissegundos. componentes: snapshot do element tree retornado, element e struct rust inspecionavel sem necessidade de render. integracao: headless render para screenshot diff apenas em critical paths com baseline versionado no repositorio.

para snapshot de componentes, element expoe apenas campos deterministicamente comparaveis: tipo, props estruturais, filhos, intent tokens. closures de callback sao opacas em teste, representadas por identificador de tipo, nao por conteudo. igualdade estrutural do element tree nao inclui callbacks. testes de componente verificam estrutura e semantica, nao identidade de funcao. testes que precisam de gpu sao lentos, flaky por diferencas de driver e impossiveis em ci sem hardware dedicado. a separacao em camadas nao e preferencia, e a condicao para que testes sejam executados com frequencia suficiente para ter valor.
]],
}
