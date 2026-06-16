return {
  id  = "rul-14",
  typ = "rule",
  sts = "reference",
  dom = "components",
  dat = "2026-03-13",
  ttl = "props minimas com contexto explicito",
  lnk = { "idx-rules" },
  txt = [[
componente com mais de 5 props deve ser quebrado em componentes menores ou receber struct de configuracao. dado que o componente nao usa diretamente, so repassa para filhos, vai via contexto ou e passado diretamente ao filho que precisa.

contexto em φ segue o modelo do xilem env, dado disponivel para toda a subarvore sem passar explicitamente em cada nivel, com tipagem estatica garantindo que o dado existe no contexto antes de ser consumido. prop drilling profundo cria acoplamento vertical: mudar um campo no nivel 5 exige editar os niveis 1 a 4 que so repassam o valor.
]],
}
