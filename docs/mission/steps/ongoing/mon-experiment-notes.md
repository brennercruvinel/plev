---
title: experimento mon, nota de trabalho
status: trabalho vivo
tags: [mon, ui-superset, design-tokens, aria-apg, lottie, monster, swf]
date: 2026-06-12
ancoras: [crates/lot, crates/monster, crates/parser, docs/adr/monster-format-v0.md, docs/adr/binary-animation-format-with-discovered-deltas.md, docs/adr/import-foreign-formats-by-conversion-not-embedding.md, docs/adr/transpiler-reports-every-unmapped-construct.md]
---

# experimento mon

nota de trabalho, nao conclusao. o mon e a parte viva do projeto, as ~30
paginas que o livro deixou abertas de proposito. registro aqui a tese, o que
ja existe la fora, e os buracos que ainda nao sei fechar. o valor desta nota
nao e ter resolvido, e ser honesto sobre o que e dificil.

## a tese: um superset de UI sobre a intersecao

a ideia, na forma mais limpa que consegui escrever: listar o vocabulario de UI
de cada plataforma, achar a intersecao (o grafo de similaridades), definir um
declarativo unico sobre essa intersecao, e ter dois modos de saida. um modo
desenha na GPU, identico em todo lugar. o outro mapeia para o componente nativo
de cada sistema. eu escrevo `Button` uma vez, e ele vira ou um quad desenhado
pela engine, ou o `UIButton` do iOS e o botao Material do Android.

isso e uma ideia coerente. nao e nova, e isso e bom, quer dizer que tem
precedente para estudar.

## o que ja existe

o React Native e o .NET MAUI escolheram o mapa para o nativo. voce escreve um
botao e ele vira o controle local de cada OS. e exatamente o "vira o botao da
plataforma". ja e assim, ja funciona, ja tem boné de produção.

o Flutter escolheu o outro lado: mesmo `Button`, mesmo pixel em todo lugar, via
Skia. e a referencia conceitual do plev, o "skia para rust". desenha tudo,
controla tudo.

os dois modos do mon nao sao invencao, sao os dois caminhos que a industria ja
tomou separados. a aposta do experimento e que da para ter os dois atras do
mesmo declarativo, com uma camada de token no meio. a duvida e se isso resolve
o problema ou so move o problema para a camada de token.

## buraco 1: a intersecao e pequena e a diferenca e infinita

botao, texto, lista. faceis, todo mundo tem. mas o trabalho real de um app nao
mora ai. mora no date picker do iOS que rola em tambor contra o calendario do
Android. no "voltar" que e botao fisico no Android, swipe de borda no iOS, e
nao existe no desktop. no teclado virtual que empurra a tela de um jeito
diferente em cada OS. na permissao de camera com fluxo proprio. no scroll com
bounce do iOS contra o overscroll glow do Android.

o grafo de "o que cada um tem e nao tem" nao e uma lista, e um espaco
combinatorio que cresce a cada versao de cada OS. a Apple muda coisa todo ano.
o grafo esta desatualizado no dia do lançamento. essa e a primeira parede.

## buraco 2: comportamento nao e mapeavel, so aparencia e

esse e o que mais me incomoda. eu consigo mapear que um botao iOS e um botao
Android sao "o mesmo botao". eu nao consigo mapear o que acontece quando o dedo
arrasta da borda esquerda. no iOS isso e navegacao de sistema. no Android e
outra coisa. quando abstraio para um declarativo unico, sou obrigado a escolher
UM comportamento, e ai quebro a expectativa nativa de pelo menos uma
plataforma. a aparencia converge, o comportamento diverge, e o usuario sente o
comportamento mais do que a aparencia.

e por isso que o Flutter, mesmo desenhando tudo, teve que criar Cupertino e
Material separados. a unificacao empurra de volta para a ramificacao. o mapa
em si e deterministico (e compilador, tabela de equivalencia, geracao de
codigo). o comportamento nao cabe na tabela.

## o buraco do texto nativo e da integracao com o sistema

texto nao e desenhar letrinha. e selecao, cursor piscando, teclado subindo,
autocorrecao, menu de copiar e colar, troca de idioma, escrita da direita para
a esquerda (arabe, hebraico), composicao de caracteres asiaticos (IME). o campo
de texto nativo entrega esse universo pronto. no modo GPU eu reimplemento tudo,
e e historicamente onde os toolkits de own-rendering mais sofrem. ate o Flutter
levou anos para acertar selecao de texto.

o mesmo vale para a integracao com o sistema: menu de contexto, share sheet do
iOS, autofill de senha, o date picker que combina com o resto do celular.
desenhando na GPU, ou eu reconstruo cada um (fica parecido, nao igual), ou abro
mao. e o usuario sente aquele leve "isso nao e daqui" que nao se nomeia mas se
percebe.

## a troca que o 100% GPU te da

em troca de abrir mao do nativo, o caminho 100% GPU compra controle absoluto do
pixel, consistencia total entre plataformas, e uma base de UI unica de verdade.
e por isso que jogos sao todos GPU: a UI de jogo nao tem botao nativo nenhum, e
tudo desenhado, e ninguem reclama, porque ali consistencia e controle valem
mais que integracao. o plev e desse lado por padrao. o modo nativo seria a
saida de escape para quando integracao importar mais que consistencia.

## a camada que segura o superset: design tokens

a peca que torna o superset coerente e a camada de design token. cor,
espacamento, raio, sombra, tipografia, duracao de animacao, tudo como dado puro,
independente de plataforma. o componente nunca usa um valor cru, ele referencia
um token. essa indirecao e o que deixa o sistema re-tematizavel e o que faz o
mesmo `Button` significar a mesma coisa no Mac, no iOS e na web. sem a camada de
token, sobram componentes bonitos e um sistema incoerente. o `parser` ja faz a
ponta disso quando mapeia cor para token de tema.

o design system vive na camada de token mais a definicao declarativa, nao na
renderizacao. e isso que da universalidade sem reescrever componente por
plataforma. os dois back-ends de saida (GPU ou nativo) consomem a mesma camada.

## o grafo canonico: ARIA APG

a fonte canonica para o "grafo de componentes" e o W3C ARIA Authoring Practices
Guide (APG). cada padrao de UI com o comportamento esperado, a interacao de
teclado, os estados. o APG e a definicao comportamental neutra de plataforma:
ele nao diz como o Material desenha um combobox, diz o que um combobox E e como
ele se comporta. e o nivel de abstracao certo para uma engine, porque o `Button`
precisa saber o que e um botao (semantica, teclado, estados), nao como o iOS o
pinta.

honestidade aqui: o engine ja tem accesskit (feature `accessibility`,
accesskit 0.24), entao a arvore semantica de widget existe, nao e remendo
futuro. o que ainda NAO existe e o grafo APG materializado como tabela de
padroes dentro do mon. o APG define a semantica, o accesskit a expoe, e a tabela
que casa os dois e aspiracao, nao codigo. marcar como nao implementado.

## o parser: o grafo de equivalencias materializado (numeros honestos)

o `parser` (parse, resolve, emit) e a materializacao parcial do grafo de
equivalencias: le UI de outro framework e cospe codigo builder do plev,
mapeando cor para token de tema e reportando num droplist, com arquivo e linha,
tudo que nao consegue representar. nada sai em silencio (ADR
transpiler-reports-every-unmapped-construct, commit 5eecb0a).

o detalhe honesto: o valor do parser depende inteiramente da taxa de drop. um
transpiler que cospe droplist gigante em todo input real e um relatorio de
incompatibilidade, nao um conversor. no corpus real do dono (40 componentes em
dois apps) ele produziu 402 propriedades mapeadas e 709 entradas de droplist,
zero crash. a card de teste congela `mapped == 51` e `dropped.len() == 51`. a
pergunta de estudo segue de pe: qual a cobertura num index.tsx de verdade, nao
de brinquedo. esse numero e o que diz se o parser e ferramenta ou demo.

## o formato binario: swf/flash contra .monster

a sub-area mais madura do experimento. o problema: como enviar animacao vetorial
que e pequena, seekable, e renderiza na mesma engine que desenha a UI. as
opcoes existentes falham cada uma em um ponto (ADR
binary-animation-format-with-discovered-deltas, commit a4ad0c0):

- lottie e JSON, 1.9 a 321 KB por segundo de animacao, parseado a cada frame, e
  apoiado num runtime estrangeiro que ja embarcou um motor JS inteiro para tocar
  clipe com script.
- swf assa uma tag por objeto por frame e paga replay O(n) em todo rewind,
  porque nao tem ponto de acesso aleatorio.
- video cru (webm) ganha em cel animation densa mas joga fora a natureza vetorial
  e a estrutura semantica.

o `.monster`, magic `MON0`. o frame poetico: h264 para vetores.

- keyframe = snapshot de cena inteira = acesso aleatorio. seek O(1) em frames. o
  swf nao tinha isso.
- interframe = delta descoberto: place, modify, replace, remove, mais segmentos
  por propriedade com easing e duracao. o player interpola. fps-independente,
  scrubbable. o swf assava uma tag por frame; o mon manda a curva.
- payload deduplicado: um shape estatico vira um asset referenciado por todo
  frame, comparado por bytes quantizados exatos. o no nunca muda e o encoder de
  delta emite zero byte para ele. e a licao da display list do swf, na
  granularidade de shape.
- player deterministico: dirigido por AnimationTick do runner, sem wall clock,
  avaliacao em janela, superficie reativa via signal.
- description track: UTF-8 opcional por keyframe, a semente de acessibilidade e
  busca que o flash nunca teve. e a a11y DA ANIMACAO, distinta da arvore de
  widget do accesskit. nao confundir as duas.

numeros honestos do corpus, e aqui mora o asterisco: cards 0.36x e explosion
0.74x do tamanho do JSON lottie (ganha). girl 6.5x, snake 42x, money 53x. o
movimento de corpo inteiro a 60fps paga o custo do v0, porque morph = re-
tessela cada shape em movimento num asset novo por amostra. a alavanca v1 e
morph track: guardar a curva, nao as amostras (licao do DefineMorphShape do
swf). e a comparacao 0.36x..53x e contra o JSON do lottie, nao contra bytes de
swf. o "mon bate o swf" e gate de design (swf mediu ~1.7 KB/s, 10-15 bytes por
objeto movido por frame), nao head-to-head medido no corpus. marcar.

import por conversao, nunca por embedding (ADR
import-foreign-formats-by-conversion-not-embedding, commit a044613): o `lot` le
o JSON do lottie uma vez, offline, e converte. depois disso o playback roda no
`monster::MonsterPlayer` e nenhum codigo lottie executa. o JSON morre na porta.

## os asteriscos honestos

o "identico em todas as plataformas" tem asterisco em duas pontas.

primeiro, o pixel. mesmo o wgpu nao garante pixel identico de graca. rasterizacao
de borda, arredondamento de subpixel no texto, e blending sRGB podem divergir
entre backends (Metal contra Vulkan contra o WebGPU do browser). o contrato esta
no lugar certo (sRGB decode once na entrada, encode once no surface write via
`surface_render_view`; measure == draw com uma TextStyle so). e foi verificado
por pixel: o fundo mediu (8,8,8) antes e (48,48,48) depois, web batendo com o
desktop (ADR render-into-an-srgb-view-format, commit 2a33933). mas isso e UM
ponto de amostra, nao snapshot pixel-a-pixel cross-backend. a igualdade entre
Metal e WebGPU e "identica por construcao, confiando no contrato", nao provada
por teste de snapshot. e exatamente ai que os toolkits serios gastam anos.

segundo, o alcance. mobile (iOS principalmente) e web sao os elos fracos da
stack. winit e wgpu rodam lindamente no desktop, na web via WebGPU com fallback,
mas iOS ainda e terreno imaturo. a stack honesta e "universal no desktop +
Android + web", nao "universal incluindo iOS com a mesma maturidade". e o mesmo
padrao que ja tinha notado no GPUI.

## estado e proximos passos

- [x] `lot`: importer lottie, render direto ou conversao para .monster
- [x] `monster`: codec binario v0, delta discovery, optimizer, player deterministico
- [x] `parser`: transpiler poc com droplist file:line, contas congeladas em teste
- [ ] morph track (v1): guardar a curva do path, nao as amostras
- [ ] grafo APG materializado como tabela de padroes dentro do mon
- [ ] modo de saida nativo (hoje so o caminho GPU existe de verdade)
- [ ] snapshot pixel-a-pixel cross-backend (Metal vs Vulkan vs WebGPU)

referencia de fundo: areweguiyet.com (o estado das GUIs em rust, citado de
proposito, com ironia: a meta nao e fazer mais uma).
