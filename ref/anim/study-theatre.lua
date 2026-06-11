return {
  id  = "stu-theatre",
  typ = "study",
  sts = "pendente",
  dom = "anim",
  dat = "2026-06-11",
  ttl = "biblioteca + editor realtime de motion graphics na web; o norte de ux",
  lnk = { "ref-study", "edt-flash-novo" },
  txt = [=[
por que esta aqui: o editor realtime na web com scrub instantaneo; e o
NORTE de ux do nosso editor. a api sheet/object/prop e o modelo mental
que queremos igualar ou superar.
o que extrair:
- api sheet/object/prop: animacao como dado tipado, editor opcional
- scrub instantaneo: como o playhead reavalia tudo sem lag
- studio ui desacoplada do core (@theatre/core vs @theatre/studio)
- keyframes e sequencing serializados como estado de projeto
- extensoes (r3f) como modelo de integracao com mundos de render
o que NAO copiar: dependencia de dom/react no runtime.
consome: edt-flash-novo
]=],
}
