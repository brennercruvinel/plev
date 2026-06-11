return {
  id  = "stu-solid",
  typ = "study",
  sts = "pendente",
  dom = "mining",
  dat = "2026-06-11",
  ttl = "ui declarativa com signals e compilacao para dom real, sem vdom",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: mover reatividade pro compile time; signals com
reacoes fine-grained provam que ui declarativa nao precisa de vdom nem
de re-render de componente.
o que extrair:
- createSignal/createEffect/createMemo: o grafo reativo minimo
- o que o compilador faz com jsx (templates clonados + bindings diretos)
- ownership/cleanup do grafo reativo (createRoot, onCleanup)
- por que componentes rodam uma vez so (setup vs render)
o que NAO copiar: o runtime dom; queremos signals compilados para
invalidacao do plev.
consome: tese
]=],
}
