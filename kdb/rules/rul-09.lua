return {
  id  = "rul-09",
  typ = "rule",
  sts = "reference",
  dom = "accessibility",
  dat = "2026-03-13",
  ttl = "acessibilidade como constraint via accesskit",
  lnk = { "idx-rules" },
  txt = [[
toda arvore de elementos do φ mantem uma arvore de acessibilidade paralela via accesskit. nao e feature opcional, e parte do contrato de cada componente desde a primeira implementacao.

φ nao tem dom. o browser nao constroi a arvore de acessibilidade automaticamente porque nao ha html. em rendering custom via skia ou wgpu, a arvore precisa ser construida explicitamente ou o produto e inacessivel para screen readers em todas as plataformas. accesskit fornece adapters portaveis, at-spi no linux, nsaccessibility no macos, ui automation no windows, sem implementacao manual por plataforma. sem essa regra como constraint arquitetural, acessibilidade sera postergada ate producao, onde o custo de retrofit e uma ordem de magnitude maior.
]],
}
