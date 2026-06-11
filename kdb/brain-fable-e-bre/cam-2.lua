return {
  id  = "cam-2",
  typ = "catalog-descriptor",
  sts = "reference",
  dom = "camada-2",
  dat = "2026-06-11",
  ttl = "frameworks de ui: o motor que renderiza pixels",
  lnk = { "ths-compilador", "ref-study" },
  src = "kdb/gui/frameworks.lua",
  txt = [[
30+ frameworks com estrategia de render, devices, performance, licenca + tabela_performance por cenario (desktop/mobile/web/mcu). mineracao prioritaria: slint (mcu a desktop, <300KiB, render por linha) e lvgl (c puro, dirty regions).
o catalogo completo vive em kdb/gui/frameworks.lua (dados crus do brenner,
2025/2026). este no e o descritor no grafo do brain; agentes mineram o
catalogo guiados por ths-compilador e ref-study. validacao: stars/forks
sao volateis, revalidar via api do github antes de decisao critica.
]],
}
