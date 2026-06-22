return {
  id  = "rul-13",
  typ = "rule",
  sts = "reference",
  dom = "errors",
  dat = "2026-03-13",
  ttl = "error handling tipado e visivel",
  lnk = { "idx-rules" },
  txt = [[
definir enum apperror com variantes semanticas: networktimeout, storagecorrupted, invalidinput { field: static str, reason: string }, authexpired, ratelimited { retry_after: duration }. todo result na app usa apperror. erros se tornam estado visivel, inline message, toast, retry button, via action no fluxo normal. zero unwrap() em codigo de producao. zero erro silencioso.

unwrap() e panic em producao. erros silenciosos criam dados corrompidos que aparecem horas depois. erros como strings genericas impedem handling granular: retry faz sentido para networktimeout, fallback para storagecorrupted, validacao inline para invalidinput. enum tipado com match exaustivo garante que todo erro tem tratamento explicito decidido em compile time.
]],
}
