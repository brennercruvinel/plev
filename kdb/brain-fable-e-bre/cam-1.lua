return {
  id  = "cam-1",
  typ = "catalog-descriptor",
  sts = "reference",
  dom = "camada-1",
  dat = "2026-06-11",
  ttl = "linguagens de programacao usadas em ui: a fundacao",
  lnk = { "ths-compilador", "ref-study" },
  src = "kdb/gui/linaguens.lua",
  txt = [[
13 linguagens com tipagem, paradigma, uso em ui, embarcado, gc, pros/contras. rust e a escolha do plev (sem gc, no_std, wasm, ownership). zig anotado como tentacao seria (comptime) mas pre-1.0.
o catalogo completo vive em kdb/gui/linaguens.lua (dados crus do brenner,
2025/2026). este no e o descritor no grafo do brain; agentes mineram o
catalogo guiados por ths-compilador e ref-study. validacao: stars/forks
sao volateis, revalidar via api do github antes de decisao critica.
]],
}
