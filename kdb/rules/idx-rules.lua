return {
  id  = "idx-rules",
  typ = "index",
  sts = "reference",
  dom = "architecture-rule",
  dat = "2026-03-13",
  ttl = "indice das regras de arquitetura do phi",
  lnk = {
    "rul-01", "rul-02", "rul-03", "rul-04", "rul-05",
    "rul-06", "rul-07", "rul-08", "rul-09", "rul-10",
    "rul-11", "rul-12", "rul-13", "rul-14", "rul-15",
  },
  txt = [[
as 15 regras-principio (antes mantras + rules, fundidos em um no por principio) que governam a arquitetura do φ. cada no rul-nn e uma regra com seu corpo cru. as arestas lnk ligam regras relacionadas: rul-07 -> rul-11 (side effects e persistencia via trait), rul-12 -> rul-10 (i18n depende de text layout).

ordem: 01 fronteira app/engine, 02 estado de dominio fora do φ, 03 fluxo unidirecional, 04 composicao sobre heranca, 05 layout declarativo, 06 navegacao como enum, 07 side effects isolados, 08 theming como struct, 09 acessibilidade via accesskit, 10 text layout via parley, 11 persistencia via trait, 12 internacionalizacao, 13 error handling tipado, 14 props minimas, 15 testabilidade por camada.
]],
}
