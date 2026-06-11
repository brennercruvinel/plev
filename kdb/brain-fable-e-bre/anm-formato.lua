return {
  id  = "anm-formato",
  typ = "poc",
  sts = "frente-lider",
  dom = "animation",
  dat = "2026-06-11",
  ttl = "h264 para vetores: delta encoding, keyframes, tweening binario",
  lnk = { "edt-flash-novo", "sem-boitata", "ref-study", "rsh-notes" },
  txt = [[
o brenner vem do flash. o insight a reviver: o swf fazia tweening binario
compacto. a apple matou o plugin, mas a codificacao estava certa, e vivemos
interacoes pasteurizadas onde quase tudo que anima entre frames e
redundante; algumas coisas nunca precisam ser reenviadas. o frame poetico:
um h264 para vetores. delta encoding a posteriori (o encoder descobre a
redundancia), keyframes + interframes, interpolacao feita pelo
renderizador. a ref do ruffle/flash e por intuicao de PERFORMANCE e
invalidacao, alem da inovacao: as intuicoes de delta, compressao e
otimizacao de memoria valem mais que o formato em si.

os quatro modelos de referencia (referencia, nao copia; criamos o nosso):
1. lottie: json do after effects; inchado mas prova o mercado
2. smil/css dentro do svg (animate, animateTransform): declara estado
   inicial e transicoes, nao frames. o endpoint declarativo que queremos
3. "video-como-animacao": a animacao emerge da interpolacao do renderer
   (shader-side onde der); frames nunca materializados
4. swf: tweening binario ultra-compacto, prova de que cabe em kilobytes.
   scripting: rhai como espirito actionscript em rust (ver edt-flash-novo)

plano (ws-anim, backend antes de ui):
a. estudo guiado por study.lua: ruffle (como o swf codifica: matrizes,
   shape morphs, place/remove object; e como invalida), lottie (keyframes/
   expressoes), thorvg, api do theatre (timeline realtime), api do
   auto-animate (ergonomia de transicao automatica)
b. spec v0: keyframe = snapshot de cena; interframe = delta binario +
   curvas de easing; inteiros LE; versionamento explicito; golden fixture
   congelada cedo (licao nest); track opcional de descricao textual por
   keyframe (a11y/seo via nlp leve no build)
c. codec em crate novo (3-char, ex: anm): encoder (cena A,B -> delta),
   decoder, player sobre Compositor+Tween/FrameClock. testes: round-trip
   estrutural, fixture, property tests (aplicar deltas == cena final),
   benchmark bytes/segundo vs lottie json e vs reenvio cru
d. auto-animate nativo no plev: bounds de um no mudaram entre frames ->
   interpolar automaticamente (api do formkit/auto-animate como
   inspiracao de ergonomia). entra em src/animation, todo consumidor
   ganha de graca
e. so depois do codec verde: a gui de autoria (edt-flash-novo) e a aba
   motion no showcase (play/pause/scrub via signal/)

restricoes: leve de verdade (kilobytes); playback deterministico em todo
alvo (FrameClock e o relogio; nada de Date::now no caminho quente);
cross-device desde o dia um (android, ios, macos, linux, windows, web,
embarcados); nenhuma ui antes do codec round-tripar com testes.

nota estudos (2026-06-11): 4 estudos guiados concluidos. baselines
medidos: lottie 1.9-321 KB/s (gzip para 10-13 por cento; nascer <= gzip
sem gzip), swf delta puro ~1.7 KB/s (10-15 bytes/objeto/frame, baked);
8 presets de easing cobrem 87 por cento de 6166 keyframes lottie; swf e
delta autorado SEM keyframes (seek O(n)) -> nosso I-frame e a inovacao;
rhai medido: 1.1us/call AST cacheada, 226KB gzip na dieta wasm -> feature
opcional do player, nunca requisito de playback; pre-requisito de core:
PartialEq em SceneNode (1 linha). spec consolidada: doc/anm-format-v0.md.
]],
}
