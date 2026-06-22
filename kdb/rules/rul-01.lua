return {
  id  = "rul-01",
  typ = "rule",
  sts = "reference",
  dom = "boundary",
  dat = "2026-03-13",
  ttl = "fronteira app engine",
  lnk = { "idx-rules" },
  txt = [[
codigo de aplicacao nunca importa wgpu, scenenode, gpuvec, compositor, nem qualquer tipo do rendering pipeline. a app fala com plev exclusivamente via builder.rs (elements) e signal.rs (reatividade).

acoplamento direto com internals do renderer significa que qualquer refactor no plev, trocar packing do atlas, mudar pipeline de blur, adicionar backend metal ou vulkan, quebra codigo de app com blast radius imprevisivel. a fronteira forca que mudancas na engine sejam invisiveis para consumers. a mesma decisao esta documentada no architecture do xilem como separacao entre view e widget, a camada view e descartavel entre ciclos, a camada widget (masonry) e retained. a app nunca toca masonry diretamente.
]],
}
