return {
  id  = "rul-12",
  typ = "rule",
  sts = "reference",
  dom = "i18n",
  dat = "2026-03-13",
  ttl = "internacionalizacao alem de text layout",
  lnk = { "idx-rules", "rul-10" },
  txt = [[
strings visiveis ao usuario vivem exclusivamente em arquivos de localizacao, formato fluent (project fluent da mozilla) por ser o unico sistema que resolve pluralizacao, genero gramatical e variacoes contextuais como dado, nao como logica condicional no codigo. zero string literal em portugues, ingles ou qualquer idioma dentro de componentes. zero formatacao manual de data, numero, moeda ou unidade, usar icu4x como unica fonte de formatacao locale-aware.

rtl layout e consequencia de locale, nao de configuracao manual. taffy suporta direction rtl como propriedade de layout, ativar globalmente quando o locale detectado e rtl. suporte a scripts (rul-10) e suporte a localizacao sao problemas ortogonais: parley garante que arabe renderiza corretamente; esta regra garante que o texto correto em arabe esta disponivel e formatado corretamente para o contexto.
]],
}
