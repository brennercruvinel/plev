return {
  id  = "stu-lottie-web",
  typ = "study",
  sts = "pendente",
  dom = "anim",
  dat = "2026-06-11",
  ttl = "player js oficial do formato lottie (after effects via bodymovin)",
  lnk = { "ref-study", "anm-formato" },
  txt = [=[
por que esta aqui: modelo de keyframes/expressoes/mascaras do after
effects serializado em json; e o contraexemplo de peso: o formato e
verboso e o nosso precisa ser o oposto.
o que extrair:
- esquema de keyframe (valores i/o de tangente, hold, time remap)
- expressoes: como logica viaja dentro do asset e o custo disso
- mascaras, mattes e precomps: composicao hierarquica em json
- onde o json incha (amostras em ref/anim/lottie-samples/ provam)
- a taxonomia de layers (shape, solid, text, image, null, precomp)
o que NAO copiar: o runtime js e o proprio formato verboso.
consome: ws-anim
]=],
}
