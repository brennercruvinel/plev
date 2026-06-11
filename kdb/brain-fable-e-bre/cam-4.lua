return {
  id  = "cam-4",
  typ = "catalog-descriptor",
  sts = "reference",
  dom = "camada-4",
  dat = "2026-06-11",
  ttl = "bibliotecas de componentes e design systems: a roupa",
  lnk = { "ths-compilador", "ref-study" },
  src = "kdb/gui/library.lua",
  txt = [[
22 entradas, tipo=design_system (specs: material, hig, fluent, carbon, polaris) vs tipo=biblioteca (codigo: mui, chakra, mantine, shadcn...). licao shadcn: componente e dado copiavel, nao dependencia.
o catalogo completo vive em kdb/gui/library.lua (dados crus do brenner,
2025/2026). este no e o descritor no grafo do brain; agentes mineram o
catalogo guiados por ths-compilador e ref-study. validacao: stars/forks
sao volateis, revalidar via api do github antes de decisao critica.
]],
}
