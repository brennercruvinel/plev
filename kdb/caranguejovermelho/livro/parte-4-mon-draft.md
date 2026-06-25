---
title: o experimento mon
parte: 4
status: amostra
rastros:
  - crates/lot
  - crates/monster
  - crates/parser
  - kdb/adr/monster-format-v0.md
  - kdb/adr/binary-animation-format-with-discovered-deltas.md
  - kdb/adr/import-foreign-formats-by-conversion-not-embedding.md
  - kdb/adr/transpiler-reports-every-unmapped-construct.md
  - kdb/adr/render-into-an-srgb-view-format.md
  - notes.md
---

# parte 4, o experimento mon

esta e a parte viva do livro. as outras contam o que ja aconteceu, com diff e
adr fechados. esta conta o que ainda esta acontecendo, e por isso e a unica que
deixei com as paginas abertas de proposito. o nome interno e mon: lottie, swf,
flash, motion UI, design system universal. eu nao vou fingir que resolvi.
prefiro mostrar onde a ideia e bonita, onde ela trava, e os numeros que ja
tenho de um pedaco que funciona.

## 4.0 a versao para a crianca de treze anos

imagina que voce desenha um botao uma vez. um botao so. e ele aparece igual no
iPhone, no celular Android, no navegador, no Mac. voce nao desenhou quatro
botoes, desenhou um, e o computador cuidou do resto. parece magica, e e mais ou
menos a promessa que todo mundo que faz app quer ouvir.

agora a parte que ninguem te conta na propaganda: as plataformas nao concordam
sobre o que um botao faz quando voce encosta nele, sobre como o teclado sobe na
tela, sobre o que acontece quando voce arrasta o dedo da beirada. desenhar o
mesmo botao e a parte facil. fazer ele se comportar como o usuario espera em
cada lugar, esse e o nó que ainda nao desatei.

## 4.1 a ideia coerente

a tese, na forma mais limpa que consegui escrever, esta no `notes.md` na raiz do
repo. listar o vocabulario de UI de cada plataforma, achar a intersecao (o grafo
de similaridades), definir um declarativo unico sobre essa intersecao, e ter
dois modos de saida. um modo desenha na GPU, identico em todo lugar. o outro
mapeia para o componente nativo de cada sistema.

eu escrevo `Button` uma vez. na saida GPU ele vira um quad que a engine desenha.
na saida nativa ele vira o `UIButton` do iOS e o botao Material do Android. o
mesmo declarativo, dois back-ends. essa e a ideia, e ela e coerente. o ponto de
honestidade ja aparece aqui: hoje, dos dois modos, so o caminho GPU existe de
verdade no codigo. o modo nativo e desenho, nao implementacao.

## 4.2 os dois caminhos que ja existem

isso nao e ideia inedita, e isso e uma boa noticia. tem precedente para estudar,
e o livro credita as fundacoes em vez de fingir que inventou a roda.

o React Native e o .NET MAUI pegaram o caminho do mapa para o nativo. voce
escreve um botao e ele vira o controle local de cada OS. ja funciona, ja esta em
producao em milhares de apps. o Flutter pegou o outro caminho: mesmo `Button`,
mesmo pixel em todo lugar, via Skia. desenha tudo, controla tudo. o plev se
posiciona desse segundo lado, o "skia para rust", e credita o Flutter
explicitamente como a referencia conceitual de rendering proprio cross-platform
(`kdb/mission/readme.md`).

os dois modos do mon nao sao invencao. sao os dois caminhos que a industria ja
tomou, separados, atras do mesmo declarativo. a aposta do experimento e que da
para ter os dois com uma camada de token no meio. a duvida, que eu carrego sem
resposta, e se isso resolve o problema ou so empurra o problema para a camada de
token.

## 4.3 o primeiro buraco: a intersecao e pequena, a diferenca e infinita

botao, texto, lista. faceis. todo mundo tem. mas o trabalho real de um app nao
mora ai.

mora no date picker do iOS que rola em tambor contra o calendario do Android. no
"voltar", que e botao fisico no Android, swipe de borda no iOS, e simplesmente
nao existe no desktop. no teclado virtual, que empurra a tela de um jeito
diferente em cada OS. na permissao de camera, com fluxo proprio em cada sistema.
no scroll, com bounce no iOS e overscroll glow no Android.

o grafo de "o que cada um tem e nao tem" nao e uma lista que voce escreve uma
vez. e um espaco combinatorio que cresce a cada versao de cada OS. a Apple muda
coisa todo ano. no dia do lancamento, o seu grafo ja esta velho. essa e a
primeira parede, e ela e estrutural, nao um bug que da para corrigir.

## 4.4 o segundo buraco: comportamento nao e mapeavel, so aparencia e

esse e o que mais me tira do eixo, e e o assassino silencioso da unificacao.

eu consigo mapear que um botao iOS e um botao Android sao "o mesmo botao". a
aparencia mapeia. eu nao consigo mapear o que acontece quando o dedo arrasta da
borda esquerda da tela. no iOS isso e navegacao de sistema. no Android e outra
coisa. quando eu abstraio para um declarativo unico, sou obrigado a escolher UM
comportamento, e nesse instante quebro a expectativa nativa de pelo menos uma
plataforma.

a aparencia converge, o comportamento diverge, e o usuario sente o comportamento
mais do que a aparencia. e por isso que o Flutter, que desenha cada pixel, ainda
assim teve que criar Cupertino e Material separados. a unificacao empurra de
volta para a ramificacao. o mapa em si e a parte tratavel: e compilador, tabela
de equivalencia, geracao de codigo, deterministico. o comportamento e o que nao
cabe na tabela.

## 4.5 texto e sistema: o imposto do own-rendering

quem nunca escreveu um campo de texto acha que texto e desenhar letrinha. nao e.
texto e selecao, cursor piscando, teclado subindo, autocorrecao, menu de copiar
e colar, troca de idioma, escrita da direita para a esquerda (arabe, hebraico),
composicao de caracteres asiaticos (o IME). o campo de texto nativo te da esse
universo inteiro de graca. no modo GPU, voce reimplementa cada peca, e e
historicamente onde os toolkits de own-rendering mais sangram. ate o Flutter
levou anos para acertar selecao de texto.

o plev paga esse imposto de olhos abertos. o engine tem o sistema de texto
proprio, com a regra de uma TextStyle por run compartilhada entre medicao e
desenho (ADR one-text-style-for-measurement-and-drawing). essa regra nasceu de
um defeito real: shapes dimensionados por uma heuristica de `chars * size *
0.58` enquanto o rasterizador moldava com a fonte de verdade, texto vazando para
fora da forma (session responsiveness, commits f15198a e 5854941). a correcao
foi deletar a heuristica no unico ponto de definicao, consertando doze call
sites de uma vez. isso e o que own-rendering custa: cada coisa que o sistema
daria pronta, voce constroi e mantem.

o mesmo vale para a integracao com o resto do sistema. menu de contexto nativo,
share sheet do iOS, autofill de senha, o date picker que combina com o tema do
celular. desenhando na GPU, ou voce reconstroi cada um (fica parecido, nao
igual), ou abre mao. e o usuario sente aquele leve "isso nao e daqui" que ele
nao sabe nomear, mas percebe na pele.

em troca, o caminho 100% GPU compra controle absoluto do pixel, consistencia
total entre plataformas, e uma base de UI unica de verdade. e o mesmo motivo
pelo qual jogos sao todos GPU: a UI de jogo nao tem botao nativo nenhum, e tudo
desenhado, e ninguem reclama, porque ali consistencia e controle valem mais que
integracao com o sistema. a escolha do plev e essa, assumida.

## 4.6 a camada que segura o superset: design tokens

a peca que torna o superset coerente, em vez de virar uma bagunca de valores
soltos, e a camada de design token. cor, espacamento, raio, sombra, tipografia,
duracao de animacao, tudo como dado puro, independente de plataforma. o
componente nunca usa um valor cru, ele referencia um token. o plev ja faz isso
com tokens medidos em oklch (ADR measured-design-tokens-over-eyeballed-values,
`crates/engine/src/theme/hoff.rs`).

essa indirecao e o que deixa o sistema re-tematizavel e o que faz o mesmo
`Button` significar a mesma coisa no Mac, no iOS e na web. sem a camada de token,
sobram componentes bonitos e um sistema incoerente. o design system vive na
camada de token mais a definicao declarativa, nao na renderizacao. e isso que
da universalidade sem reescrever componente por plataforma, e os dois back-ends
de saida, GPU ou nativo, consomem a mesma camada.

## 4.7 o grafo canonico: ARIA APG, e a honestidade do accesskit

para o "grafo de componentes" existe uma fonte canonica, e nao precisei inventar
nenhuma: o W3C ARIA Authoring Practices Guide, o APG. cada padrao de UI com o
comportamento esperado, a interacao de teclado, os estados. o APG e a definicao
comportamental neutra de plataforma. ele nao diz como o Material desenha um
combobox, diz o que um combobox E e como ele se comporta. e o nivel de abstracao
certo para uma engine, porque o `Button` precisa saber o que e um botao
(semantica, teclado, estados), nao como o iOS o pinta.

aqui vem uma correcao honesta que eu mesmo precisei fazer. na primeira versao
desta nota, eu tinha escrito que nao via accesskit no projeto e que a
acessibilidade seria remendo depois. estava errado. o engine tem accesskit
atras da feature `accessibility` (accesskit 0.24, accesskit_winit 0.32,
`crates/engine/Cargo.toml`), com a arvore semantica de widget, focus graph e
ativacao lazy (task-30). a arvore semantica existe, nao e divida futura.

o que ainda NAO existe e o grafo APG materializado como tabela de padroes dentro
do mon. o APG define a semantica, o accesskit a expoe, e a tabela que casa os
dois e aspiracao, nao codigo. eu marco isso como nao implementado de proposito,
porque a diferenca entre "temos acessibilidade na fundacao" e "temos o grafo de
padroes APG ligado ao engine" e exatamente o tipo de coisa que um manifesto
vitorioso esconderia e este livro nao esconde.

## 4.8 o parser: o grafo de equivalencias, com numero

o `parser` (parse, resolve, emit) e a materializacao parcial do grafo de
equivalencias. ele le UI de outro framework (React TSX mais Sass, e gpui) via
tree-sitter, e cospe codigo builder do plev, mapeando cor para token de tema. o
que ele nao consegue representar nao some: cai num droplist, com arquivo, linha
e motivo (ADR transpiler-reports-every-unmapped-construct, commit 5eecb0a).

o detalhe que separa ferramenta de demo: o valor do parser depende inteiramente
da taxa de drop. um transpiler que cospe droplist gigante em todo input real e
um relatorio de incompatibilidade, nao um conversor. entao o numero importa.
rodando no corpus real do dono, 40 componentes em dois apps, o parser produziu
402 propriedades mapeadas e 709 entradas de droplist, zero crash. a card de
teste congela as contas em `mapped == 51` e `dropped.len() == 51`, de modo que
um mapper que comece a dropar algo novo quebra o build.

eu nao vou dourar isso. 709 drops contra 402 mapeados quer dizer que, no corpus
de verdade, o parser hoje deixa de fora mais do que converte. o droplist e o
roadmap: os tipos de entrada que mais se repetem sao exatamente as proximas
features a implementar. a honestidade aqui e o produto. um conversor que mente
sobre cobertura te manda cacar bug propriedade por propriedade, sem ponto de
partida, semanas depois.

## 4.9 o formato binario: swf/flash contra o .monster

esta secao e o pedaco do experimento que ja funciona e ja tem numero medido. e
sobre como enviar animacao vetorial que seja pequena, seekable, e que renderize
na mesma engine que desenha a UI.

### por que nem lottie, nem swf, nem video

cada opcao existente falha num ponto (ADR
binary-animation-format-with-discovered-deltas, commit a4ad0c0):

- lottie e JSON. de 1.9 a 321 KB por segundo de animacao, parseado a cada frame,
  apoiado num runtime estrangeiro que ja chegou a embarcar um motor JS inteiro
  para tocar clipe com script. gzip esmaga o JSON para 10 a 13 por cento, o que
  ja diz que o formato esta gordo de origem.
- swf, o velho Flash, assa uma tag por objeto por frame. e paga replay O(n) em
  todo rewind, porque nao tem ponto de acesso aleatorio. voce rebobina e ele
  reprocessa desde o inicio.
- video cru (webm) ganha em cel animation densa, mas joga fora a natureza
  vetorial e a estrutura semantica inteira. vira pixel, deixa de ser desenho.

### o frame poetico: h264 para vetores

o `.monster`, magic `MON0`. a imagem que organiza o codec e h264 para vetores.
keyframes sao os I-frames, interframes sao deltas descobertos, o renderizador
interpola. tres consequencias diretas, e cada uma resolve uma das falhas acima.

primeiro, o keyframe e um snapshot da cena inteira. seek vira O(1) em frames, a
coisa que o swf nao tinha. segundo, entre keyframes viaja so o delta descoberto.
e ai que a estrutura aparece, no proprio enum de operacao
(`crates/monster/src/container.rs`):

```rust
pub enum DeltaOp {
    Place {
        at_s: f32,
        node: Node,
    },
    Modify {
        at_s: f32,
        node_id: NodeId,
        props: Vec<(Prop, Vec<Segment>)>,
    },
    Replace {
        at_s: f32,
        node: Node,
    },
    Remove {
        at_s: f32,
        depth: Depth,
    },
}
```

a cena e um mapa plano de profundidade para no. um no que nao muda nao gera
op nenhuma, e custa zero byte. e a licao da display list do swf, aplicada na
granularidade de shape. o `Modify` carrega presence flags por campo: um campo
que nao mudou nao tem bit de presenca e tambem custa zero byte. so o que se mexeu
paga.

### o payload deduplicado

a terceira consequencia e a que mais me agradou quando os numeros bateram. um
shape estatico vira um asset unico, referenciado por todo frame. a comparacao e
por bytes quantizados exatos. a ponte `lot::cnv` (que converte lottie para
.monster) faz isso na hora de empacotar a geometria
(`crates/lot/src/cnv.rs`):

```rust
let next_id = assets.len();
let id = *intern.entry(payload).or_insert_with_key(|key| {
    assets.push(Asset {
        kind: AssetKind::Path,
        data: key.clone(),
    });
    next_id as u16
});
```

como as posicoes quantizam para a grade de twips (vintesimos de um pixel
logico), jitter de sub-twip empacota para bytes identicos, e a dedup-por-payload
se mantem entre frames. um shape parado e um asset, um no que nunca muda, zero
byte de delta. o custo do movimento vive na tabela de asset, nao na timeline, e
e exatamente onde a alavanca v1 (morph track) vai agir.

### o player deterministico

o player nao tem relogio de parede. ele e dirigido por um `AnimationTick` que o
runner entrega, avalia numa janela ao redor do tempo atual, e expoe play, pause
e scrub como superficie reativa via signal. a cena vai para o compositor por
frame, e o dirty-hash do compositor faz os pushes que nao mudaram saírem de
graca. nenhum codigo lottie executa no playback. essa e a regra do ADR
import-foreign-formats-by-conversion-not-embedding (commit a044613): o `lot` le
o JSON uma vez, offline, converte, e o JSON morre na porta.

ainda do lado do encoder, ha um otimizador que roda antes do encode, sobre a IR,
sem tocar no layout de arquivo (`crates/monster/src/optimize.rs`). os defaults
sao meia unidade de quantizacao em cada passo:

```rust
impl Default for OptimizeCfg {
    fn default() -> Self {
        Self {
            static_tol: 0.5,
            rdp_tol: 0.5,
            collapse_static: true,
            reduce_rdp: true,
            fuse_collinear: true,
        }
    }
}
```

meia unidade e o erro que a quantizacao ja comete, entao a otimizacao default e
lossless no fio. os passos (colapso de track estatico, reducao de keyframe por
Ramer-Douglas-Peucker, fusao de segmentos colineares) iteram ate um fixpoint, e
a ordem nao importa: otimizar duas vezes da o mesmo que otimizar uma.

### os numeros, com o asterisco

aqui esta a tabela honesta. os tamanhos sao do `.monster` contra o JSON do
lottie, medidos nos cinco arquivos de corpus (ADR monster-format-v0).

| arquivo | tamanho relativo ao JSON lottie | leitura |
|---------|-------------------------------|---------|
| cards | 0.36x | ganha forte |
| explosion | 0.74x | ganha |
| girl | 6.5x | perde |
| snake | 42x | perde feio |
| money | 53x | perde feio |

o movimento discreto ja ganha. o movimento de corpo inteiro a 60fps paga o
custo do v0, porque morph aqui e re-tessela: cada shape em movimento vira um
asset novo por amostra, e a tabela de asset incha. a alavanca v1 nomeada e o
morph track, guardar a curva do path em vez das amostras, a licao do
DefineMorphShape do proprio swf.

e o asterisco que eu nao deixo passar: essa tabela e contra o JSON do lottie,
nao contra bytes de swf. a afirmacao "o mon bate o swf" e um gate de design,
nao um head-to-head medido no corpus. o swf mediu cerca de 1.7 KB por segundo de
animacao pura (10 a 15 bytes por objeto movido por frame), e o gate diz que o
mon bate isso para movimento com easing, porque um segmento com curva substitui
N tags por frame. mas comparar isso de verdade exige medir os mesmos clipes nos
dois formatos, e eu nao fiz. marcado como nao confirmado.

um detalhe que vale o paragrafo: o `.monster` tem uma description track, UTF-8
opcional por keyframe. e a semente de acessibilidade e busca que o Flash nunca
teve, o antidoto a opacidade dele. so que e a acessibilidade DA ANIMACAO, texto
por keyframe, distinta da arvore de widget que o accesskit expoe. duas coisas
diferentes, e juntar as duas seria mentira de marketing.

## 4.10 os asteriscos honestos

o "identico em todas as plataformas" tem asterisco em duas pontas, e elas
fecham o capitulo porque sao o ponto inteiro dele.

o primeiro asterisco e o pixel. mesmo o wgpu nao garante pixel identico de
graca. rasterizacao de borda, arredondamento de subpixel no texto, e blending
sRGB podem divergir entre backends, Metal contra Vulkan contra o WebGPU do
browser. o contrato esta no lugar certo: sRGB decode uma vez na entrada, encode
uma vez no surface write, pelo unico construtor sancionado `surface_render_view`,
e measure == draw com uma TextStyle so. e foi verificado por pixel: o fundo da
pagina mediu (8,8,8) antes da correcao e (48,48,48) depois, a web batendo com o
desktop (ADR render-into-an-srgb-view-format, commit 2a33933).

mas (48,48,48) e UM ponto de amostra, nao um snapshot pixel-a-pixel cross-
backend. a igualdade entre Metal e WebGPU hoje e "identica por construcao,
confiando no contrato", nao provada por teste de snapshot que compare a saida
inteira dos dois. e exatamente nessa fresta que os toolkits serios gastam anos.
o guard test que existe cobre o caminho do texto, nao a igualdade de saida entre
os backends. eu sei a diferenca, e prefiro escreve-la a esconde-la.

o segundo asterisco e o alcance. mobile, iOS principalmente, e web sao os elos
fracos da stack. winit e wgpu rodam lindamente no desktop, na web via WebGPU com
fallback, mas iOS ainda e terreno imaturo, suporte parcial, caminho acidentado.
a stack honesta e "universal no desktop, Android e web", nao "universal incluindo
iOS com a mesma maturidade". e o mesmo padrao que ja tinha notado no GPUI.

## 4.11 fecho, sem manifesto

eu poderia fechar este capitulo dizendo que o mon resolve a UI universal. nao
resolve, e o livro inteiro perde a credibilidade se eu disser isso. o que tenho
de concreto e um codec binario que ja ganha em movimento discreto e perde em
morph denso, com a alavanca de melhora nomeada. um transpiler que e honesto
sobre o que nao mapeia, e cujo numero de drop diz que ainda ha muito a mapear.
uma camada de token que segura o superset. um engine com acessibilidade na
fundacao, e um grafo APG que e aspiracao, nao codigo.

e tenho dois buracos que ainda nao sei fechar: a intersecao entre as plataformas
e pequena enquanto a diferenca cresce a cada release, e o comportamento nativo
nao mapeia para uma tabela do jeito que a aparencia mapeia. esses dois nao sao
bug, sao a forma do problema. o valor desta parte do livro nao e a vitoria. e
ter escrito, com numero e file:line, o que e dificil de verdade e por que.

## rastros

o que sustenta cada afirmacao acima, com file:line onde da. numeros nao
confirmados estao marcados no corpo.

### adr e specs

- `kdb/adr/monster-format-v0.md:18-24` gates medidos: lottie 1.9 a 321 KB/s,
  gzip 10-13 por cento; swf delta ~1.7 KB/s, 10-15 bytes por objeto/frame
- `kdb/adr/monster-format-v0.md:43` 8 presets de easing cobrem 87 por cento de
  6166 keyframes lottie
- `kdb/adr/monster-format-v0.md:158-165` ratios do corpus contra o JSON lottie:
  cards 0.36x, explosion 0.74x, girl 6.5x, snake 42x, money 53x; morph = re-
  tessela, alavanca v1 = morph track
- `kdb/adr/binary-animation-format-with-discovered-deltas.md` (commit a4ad0c0)
  decisao do .monster: keyframe I-frame, delta descoberto, no inalterado = zero
  byte
- `kdb/adr/import-foreign-formats-by-conversion-not-embedding.md` (commit
  a044613) conversao, nao embedding; o JSON morre na porta
- `kdb/adr/transpiler-reports-every-unmapped-construct.md:41` (commit 5eecb0a)
  402 mapped / 709 droplist no corpus real; card congela 51/51
- `kdb/adr/render-into-an-srgb-view-format.md:38` (commit 2a33933) fundo
  (8,8,8) antes, (48,48,48) depois, web == desktop num ponto
- `kdb/adr/responsiveness-multiplatform-and-fidelity.md:33` (commits f15198a,
  5854941) uma TextStyle por run, heuristica de medida deletada
- `kdb/adr/one-text-style-for-measurement-and-drawing.md` regra measure == draw
- `kdb/adr/measured-design-tokens-over-eyeballed-values.md` tokens em oklch

### crate e codigo

- `crates/monster/src/container.rs:225-244` enum `DeltaOp` (place, modify,
  replace, remove)
- `crates/monster/src/container.rs:1-43` layout do container v0, K/D/X/T,
  sha256 por secao
- `crates/lot/src/cnv.rs:118-147` loop de dedup por payload quantizado na
  conversao lottie -> .monster
- `crates/monster/src/optimize.rs:52-60` `OptimizeCfg::default`, tolerancias
  0.5 (meia unidade de quantizacao, lossless no fio)
- `crates/monster/src/discover.rs:73` `discover`, delta discovery do encoder
  modo B
- `crates/monster/src/lib.rs:1-37` mapa do codec (ir, write, discover, optimize,
  read, play, lower)
- `crates/lot/src/lib.rs:1-19` importer lottie, subset suportado, skip com log
- `crates/engine/Cargo.toml:22` feature `accessibility` (accesskit 0.24,
  accesskit_winit 0.32), Cargo.toml:96-97
- `crates/engine/examples/lot2monsters/main.rs` CLI de conversao, imprime a
  tabela de bytes
- `crates/engine/src/theme/hoff.rs` tokens medidos em oklch

### versoes conferidas (Cargo.toml workspace)

- edition 2024, rust-version 1.85, wgpu 28, winit 0.30, cosmic-text 0.18,
  taffy 0.9, sha2 0.10 (default-features off, wasm-safe)

### nao confirmado (marcado no corpo)

- head-to-head de bytes .monster contra swf no corpus: nao medido. o "mon bate
  swf" e gate de design (ADR), nao comparacao no corpus
- grafo APG como tabela de padroes dentro do mon: aspiracional, nao implementado
- modo de saida nativo: desenho, so o caminho GPU existe no codigo
- snapshot pixel-a-pixel cross-backend (Metal vs Vulkan vs WebGPU): nao existe;
  igualdade e por construcao, nao por teste
