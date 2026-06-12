
return {
  id  = "vis-unificada",
  typ = "thesis",
  sts = "living",
  dom = "vision",
  dat = "2026-06-11",
  ttl = "a tese: compositing unificado, um binario, seis alvos",
  lnk = { "ths-compilador", "plev", "monster-formato", "parser-transpiler" },
  txt = [[
a fronteira entre renderizacao nativa e renderizacao web nao e
constrangimento fisico, e acidente historico. se a separacao e acidental,
ela e eliminavel, e a eliminacao e realizavel agora: memoria segura e
abstracoes de custo zero de uma linguagem de sistemas, no momento exato de
maturidade do webgpu (estavel em todos os browsers 2024, maturidade
completa incluindo safari/ios em 2026). quem tentar em 2027 encontra um
ecossistema ja estabelecido.

o plev em um paragrafo: motor de compositing gpu-first em rust que dissolve
a dicotomia no nivel de COMPILACAO, nao de abstracao. o mesmo codebase nao
modificado compila para metal (macos/ios), vulkan (linux/android), d3d12
(windows) e webgpu (wasm32), com execucao de shader identica. nao e
abstracao sobre dois renderers: e um renderer com varios alvos de
compilacao.

compromissos arquiteturais:
- scene graph regenerado integralmente por frame em rust puro. sem virtual
  dom, sem diffing retained tradicional. o frame descreve o que existe; o
  compositor decide o que muda na gpu. elimina a categoria inteira de bugs
  de estado ui/dado divergente
- dirty region tracking: so regioes mutadas geram submissao por frame
  (entregue: hash por layer)
- texto, o ponto de falha historico de rust-para-wasm: atlas de glifos
  residente em gpu sobre cosmic-text, shaping unicode de producao,
  identico em todo alvo, sem raster cpu por frame em texto estavel
- um modelo de gamma: decode srgb uma vez entrando, encode uma vez na
  escrita da surface. provado por pixel: 48,48,48 no desktop E no browser

"camada unica": para quem usa, o plev e UMA camada; por dentro, as quatro
camadas do catalogo (linguagem, framework, headless/comportamento, design
system) estao incorporadas e posicionadas. ver ths-compilador para o
colapso delas num pipeline de lowering.

estado contra a tese (2026-06-11): macos e browser pixel-validados;
android com scaffolding ~80 (log de deploy em rsh-notes); ios sem entry
point; linux/windows sem bloqueio conhecido.

contrapeso honesto: o diferencial e a COMBINACAO (linguagem visual medida +
pixel identico em toda tela + tamanho possuivel), nao uma capacidade
isolada. slint/dioxus/makepad/flutter ja reivindicam "roda em tudo" com
anos de maturidade. "roda" e o marco barato; "parece nativo" (ime, fisica
de scroll, safe areas, lojas) e o caro.
]],
}
