return {
  id  = "ref-study",
  typ = "index",
  sts = "ws-refs",
  dom = "references",
  dat = "2026-06-11",
  ttl = "mapa de clones com instrucao de estudo embutida (criar, nao copiar)",
  lnk = { "anm-formato", "edt-flash-novo", "prs-transpiler", "shc-absorcao", "ths-compilador" },
  txt = [[
regra do brenner: nao adianta clonar e largar. cada clone ganha um
study.lua ao lado (mesmo formato de no): por que esta aqui, o que extrair,
o que NAO copiar, qual workflow consome. shallow --depth 1. estudar para
criar e revolucionar, nunca copiar.

ja em ref/ (4.7GB, inventariado): gpui-component, hoff-research-social,
hoff-research-briefs, 4 hoff-research-cards (1.4GB, react+html), accesskit,
cushy, slang-rs, vello, glyphon, libcosmic, floem, swash, parley, slint
(render-texts), taffy, winit, OpenUSD, FFmpeg/openh264/sprite-dicing
(delta), spectrum-analyzer.

a clonar (por consumidor):
- ref/anim/ (ws-anim): ruffle-rs/ruffle (COMO o swf codifica tweens e como
  invalida; performance), 5-10 swfs classicos de archive.org, airbnb/
  lottie-web + amostras json, thorvg/thorvg (player vetor leve),
  mattbas/glaxnimate (gitlab), friction2d/friction, Wicklets/wick-editor,
  theatre-js/theatre (editor realtime: o norte de ux), formkit/auto-animate
  (api de ergonomia pro nativo); arqueologia se couber: synfig, opentoonz,
  pencil2d (prioridade baixa, repos grandes)
- ref/lang/ (ws-anim scripting + dsl): rhaiscript/rhai (actionscript em
  espirito), RayMarch/shame (shaders metaprogramados em rust),
  johnthagen/min-sized-rust (doc de dieta de binario)
- ref/parser/ (ws-parser): topiary (tweag), Ataraxy-Labs/weave (diff de
  referencia), biomejs/gritql (transformacao em massa; o brenner ja usou
  muito), Wilfred/difftastic (diff estrutural), tree-sitter
- ref/vis/ (ws-showcase charts): plotters, charming, egui_graphs;
  doc-only: rerun, poloto, plotlib, ratatui, textplots, nannou (sketches:
  reaction-diffusion, strange attractors, chladni, domain warping,
  l-systems, boids, space-filling curves), bevy_prototype_lyon, petgraph,
  Burn (sao exemplos para descobrirmos e replicarmos efeitos no plev)
- ref/mining/ (tese cam-2/3): chakra-ui/zag (statecharts canonico),
  chakra-ui/ark, solidjs/solid, sveltejs/svelte, shadcn-ui/ui,
  slint-ui/slint (ja ha copia em render-texts; nao duplicar), lvgl/lvgl,
  makepad/makepad
- ref/ide/ (ws-ide, fila): unhappychoice/gitlogue, Auto-Explore/GitComet,
  CodeEditApp/CodeEditSourceEditor (swift, modelo de editor), warpdotdev/
  warp (arquitetura rust de ide; "a falha e ser html"), alexheretic/
  glyph-brush (classico de texto gpu)
- doc-only (pesados/conceito): ffmpegwasm/ffmpeg.wasm (video em wasm;
  intuicao de delta), safishamsi/graphify (grafos de codebase; talvez
  refatorar a ideia pra visualizar arvores de ui no parser)

gate da fase: du -sh por item, total novo <8GB, cada clone com study.lua
apontando de volta pra este no, commit ao final.

nota ws-refs (2026-06-11): clones executados. synfig, opentoonz e
pencil2d NAO clonados (arqueologia pesada): ficam doc-only, estudar
pela documentacao/web se o ws-anim precisar. slint nao duplicado (copia
viva em ref/render-texts/slint; ponteiro em ref/mining/study-slint.lua).
warpdotdev/warp clonado shallow (752MB, abaixo do gate de 1.5GB).
shadcn-ui/ui clonado como ref/mining/shadcn-ui. amostras: 9 swf
classicos (<5MB cada, verificados com file) em ref/anim/swf-samples/ e
8 json em ref/anim/lottie-samples/ vindos do proprio clone lottie-web.

nota ws-anim (2026-06-11): brenner salvou amostras reais em ref/lottie/
(cards, explosion, girl, MONEY com state machines interativas, SNAKE;
json + dotlottie + webm + svg lado a lado). study-samples.lua ao lado
carrega os baselines medidos de bytes/s e o modelo de interatividade
declarativa do dotlottie v2; leitura obrigatoria antes de tocar no anm.
]],
}
