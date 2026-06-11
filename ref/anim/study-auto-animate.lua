return {
  id  = "stu-auto-animate",
  typ = "study",
  sts = "pendente",
  dom = "anim",
  dat = "2026-06-11",
  ttl = "utilitario zero-config que anima transicoes de layout com uma chamada",
  lnk = { "ref-study", "anm-formato" },
  txt = [=[
por que esta aqui: a ergonomia (uma chamada anima transicoes de layout
automaticamente); queremos isso NATIVO no plev, no nivel do engine de
layout, nao como plugin.
o que extrair:
- deteccao de mudanca de layout (MutationObserver + medicao before/after)
- tecnica FLIP aplicada a add/remove/move de filhos
- a api de uma linha: o que ela esconde e quais knobs expoe
- respeito automatico a prefers-reduced-motion
o que NAO copiar: a implementacao dom; no plev o taffy ja sabe o
before/after sem observar nada.
consome: ws-anim item auto-animate
]=],
}
