return {
  id  = "rul-10",
  typ = "rule",
  sts = "reference",
  dom = "text",
  dat = "2026-03-13",
  ttl = "text layout via parley com suporte bidi",
  lnk = { "idx-rules" },
  txt = [[
todo rendering de texto passa por parley (linebender). zero implementacao manual de text layout. bidi, scripts complexos (devanagari, tailandes, arabe, hebraico) e features opentype sao suportados por construcao via harfrust.

text layout correto e um dos problemas computacionalmente mais complexos em ui, line breaking, word wrapping, ligatures, kerning, bidi reordering, combining characters. implementacao manual produz resultado que parece correto em ingles e quebra silenciosamente em qualquer outro script. parley resolve isso com o mesmo rigor do browser, em rust, sem dependencia de sistema operacional.
]],
}
