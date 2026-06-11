return {
  id  = "stu-zag",
  typ = "study",
  sts = "pendente",
  dom = "mining",
  dat = "2026-06-11",
  ttl = "maquinas de estado finitas para componentes acessiveis (chakra)",
  lnk = { "ref-study", "ths-compilador" },
  txt = [=[
por que esta aqui: statecharts como logica de componente, o caso
canonico: toda a logica de um widget (menu, combobox, slider) vive numa
machine pura, frameworks so conectam.
o que extrair:
- anatomia da machine: states, transitions, guards, actions, activities
- a api connect(): machine -> props/aria de cada parte do widget
- normalizacao cross-framework (mesma machine, react/vue/solid)
- a11y embutida na machine, nao colada depois
- o catalogo de machines como espec de comportamento de widgets
o que NAO copiar: o runtime js; no plev a machine compila, nao interpreta.
consome: tese + biblioteca
]=],
}
