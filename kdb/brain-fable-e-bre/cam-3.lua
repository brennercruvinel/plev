return {
  id  = "cam-3",
  typ = "catalog-descriptor",
  sts = "reference",
  dom = "camada-3",
  dat = "2026-06-11",
  ttl = "primitivos headless: comportamento sem visual",
  lnk = { "ths-compilador", "ref-study" },
  src = "kdb/gui/headless.lua",
  txt = [[
15 libs de comportamento puro (a11y, teclado, foco, aria). flag usa_state_machine: zag.js (canonico), ark-ui (consumidor), downshift (state reducer). a tese ths-compilador compila esses statecharts para tabelas em build.
o catalogo completo vive em kdb/gui/headless.lua (dados crus do brenner,
2025/2026). este no e o descritor no grafo do brain; agentes mineram o
catalogo guiados por ths-compilador e ref-study. validacao: stars/forks
sao volateis, revalidar via api do github antes de decisao critica.
]],
}
