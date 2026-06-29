---
title: "traits como contrato: View, Lifecycle, Interpolate"
parte: 1
status: rascunho
rastros:
  - crates/engine/src/view/trait_def.rs:9
  - crates/engine/src/component/mod.rs:17
  - crates/engine/src/animation/tween.rs
  - kdb/adr/view-trait-design.md
  - kdb/adr/component-design.md
---

# traits como contrato: View, Lifecycle, Interpolate

toda vez que eu sento pra desenhar uma arvore de interface, a primeira coisa que
a minha cabeca quer fazer e a errada. ela quer uma classe base. um `Widget` no
topo, com uns campos comuns (posicao, cor, filhos), e dali pra baixo um `Button`
que herda de `Widget`, um `Panel` que herda de `Widget`, um `Text` que herda de
`Widget` e sobrescreve o metodo de desenhar. e o reflexo de quem aprendeu UI em
linguagem de objeto. funciona por um tempo, depois vira uma arvore de heranca de
seis niveis onde ninguem lembra de onde veio o campo `padding`.

rust fecha essa porta. nao existe `class Button extends Widget`. nao da pra
herdar campo de struct nenhuma, nao tem classe base com estado pra estender. na
primeira semana isso parece uma parede. depois voce percebe que e outra porta, e
que a porta velha levava pra um lugar pior do que voce achava. no lugar da
heranca, rust te da a trait, e a trait nao e uma classe magra. ela e uma
promessa.

uma trait diz: quem implementa isto sabe fazer X. nao carrega dado, carrega
capacidade. e o compilador te cobra a promessa, nome por nome, tipo por tipo,
antes de rodar qualquer coisa. tres traits seguram todo o lado declarativo da
engine do plev: `View`, `Lifecycle` e `Interpolate`. cada uma e um contrato.
cada uma e pequena de proposito. e cada uma se recusa a virar classe base. este
capitulo abre nesse reflexo de heranca que eu ainda tenho que matar todo dia, e
desce ate o motivo de cada uma das tres pegar o `self` emprestado de um jeito
diferente, porque essa diferenca, a forma como cada contrato toca o proprio
estado, e a arquitetura inteira escondida em tres assinaturas.

## o contrato antes do objeto

vale parar um segundo no que uma trait e, em linguagem de gente, antes de abrir
codigo. heranca de classe junta duas coisas que nao tinham que andar juntas:
compartilhar comportamento e compartilhar dado. quando voce escreve `Button
extends Widget`, voce ganha os metodos do `Widget`, mas tambem ganha os campos
dele, o construtor dele, e o acoplamento de que mudar o `Widget` pode quebrar o
`Button` de um jeito que voce nao previu. a relacao e "e um". um botao e um
widget. parece inofensivo ate a quinta geracao da arvore.

a trait separa as duas coisas. ela so descreve comportamento, a lista de metodos
que alguem precisa saber responder. nao tem campo pra herdar, nao tem
construtor, nao tem estado embutido. a relacao deixa de ser "e um" e vira
"implementa". um `RectView` nao e um `View`, ele implementa `View`, ou seja, ele
promete saber responder os metodos que `View` exige. a diferenca soa sutil
escrita assim. na pratica ela muda quem depende de quem. o `RectView` depende do
contrato, nao de uma classe ancestral cheia de coisa que ele nem usa.

e quando voce quer compartilhar dado, voce nao herda, voce compoe. um container
nao herda dos filhos, ele guarda os filhos num campo. "tem um", nao "e um". o
resto deste capitulo e basicamente isso, tres vezes: o contrato (a trait) de um
lado, a composicao (o struct que guarda) do outro, e o compilador no meio
garantindo que a promessa foi cumprida.

## View, o contrato de "produz nos, nao desenha"

a primeira trait e a mais visivel pra quem escreve tela. ela vive inteira em
`crates/engine/src/view/trait_def.rs`, e cabe na palma da mao:

```rust
//! View trait definition.

use crate::compositor::SceneNode;
use crate::layout::LayoutStyle;

use super::context::ViewContext;

/// Produces SceneNodes without touching the compositor directly.
pub trait View {
    fn layout(&self) -> LayoutStyle {
        LayoutStyle::default()
    }

    fn children(&self) -> &[Box<dyn View>] {
        &[]
    }

    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode>;
}
```

tres metodos, e a forma deles ja conta a historia. dois tem corpo padrao,
`layout` e `children`, entao quem implementa `View` pode ignorar os dois e so
escrever o terceiro. so `render` e obrigatorio, e ele nao tem corpo, e uma
assinatura solta terminada em ponto e virgula. essa e a parte do contrato que
nao tem default: se voce diz que e uma `View`, voce tem que saber produzir
`SceneNode`. o resto a trait te da de graca.

repara no que `render` devolve e no que ele nao faz. devolve `Vec<SceneNode>`.
nao chama a GPU, nao chama o compositor, nao desenha nada. uma `View` descreve, e
entrega a descricao pra quem chamou. o comentario na primeira linha do trait diz
isso sem rodeio: "Produces SceneNodes without touching the compositor directly".
essa e a decisao central do design, e ela esta registrada no adr
`kdb/adr/view-trait-design.md`. o `ViewContext` que entra no `render` nao guarda
referencia ao compositor, so carrega informacao de viewport. o adr lista quatro
razoes pra isso, e elas valem a leitura devagar porque cada uma cobra um preco em
outro lugar do sistema.

a primeira e testabilidade. um teste de `View` nao precisa de compositor, nao
precisa de GPU, nao precisa de nada da infraestrutura do wgpu. voce constroi um
`ViewContext`, chama `render`, e olha o `Vec<SceneNode>` que volta. a segunda e
composabilidade: se cada view devolve um vec, voce compoe a saida de varias views
concatenando vecs, sem cerimonia. a terceira e simplicidade, e ela e mais
profunda do que parece: como o `ViewContext` nao guarda referencia a nada, ele
nao precisa de lifetime parameter. quem ja brigou com `&'a mut Compositor`
atravessado por dez assinaturas sabe o tamanho desse alivio. a quarta e
desacoplamento: a view produz `SceneNode` puro, e quem chamou decide o que fazer
com ele. empurrar pro compositor, comparar com o frame anterior, jogar fora, o
que for. a view nao tem opiniao sobre isso.

todo desacoplamento cobra. o adr e honesto sobre o preco: cada `render` aloca um
`Vec<SceneNode>` novo. isso e uma alocacao de CPU toda vez. o doc argumenta que e
negligivel, e por tres motivos concretos. sao alocacoes de CPU temporarias, nao
de GPU. o caminho quente de performance nao e esse, e o dirty tracking via
fxhasher, que continua intacto (e o assunto de outro capitulo). e as views
tipicas produzem de um a cinco nos cada, entao o vec quase nunca cresce. e a
troca consciente de uma alocacao barata por uma fronteira limpa. eu prefiro isso
a um lifetime atravessado pela engine inteira pra economizar um `Vec` de cinco
elementos.

tem um detalhe de assinatura que so faz sentido olhando pro futuro. o `render`
recebe `&mut ViewContext`, mut, mesmo o `ViewContext` sendo hoje so leitura. o
adr explica: o `&mut` esta la pra permitir extensao depois, tipo estado de layout
acumulado ou cursor de posicao, sem ter que mudar a assinatura de toda `View` que
ja existe no dia em que isso for preciso. e uma aposta deliberada num ponto de
flexibilidade futuro, paga agora com um `mut` que ainda nao morde.

agora a parte honesta, porque o livro inteiro se sustenta em ler a fonte e nao a
legenda. o adr `view-trait-design.md`, nas linhas 12 a 16, mostra a trait `View`
com um unico metodo, so o `render`. o codigo de hoje, em `trait_def.rs`, tem
tres: ganhou `layout` e `children` como metodos com default. o adr documenta a
versao minima do contrato, e o codigo cresceu dois metodos defaultados desde
entao. confirmei isso lendo os dois arquivos lado a lado. nao e contradicao, e o
doc ficando um passo atras do source, que e o tipo de defasagem que vale marcar
em vez de esconder. tem mais um descompasso na mesma linha: o adr, na secao de
integracao com `window.rs`, ilustra a criacao do contexto como `ViewContext {
width: w, height: h }`, um struct literal de dois campos. o `ViewContext` real,
em `crates/engine/src/view/context.rs`, tem oito campos hoje (width, height,
bounds, safe_area, scale_factor, keyboard_visible, keyboard_height, theme) e se
constroi por `ViewContext::new(width, height)`. o literal do adr nao compila mais
contra o tipo atual. de novo, o source andou pra frente do doc, e marco isso
explicitamente.

## a arvore por composicao, o bound no lugar da heranca

aqui o reflexo de heranca volta a bater. eu tenho um container que precisa
desenhar um fundo e segurar um monte de filhos de tipos diferentes: um retangulo,
um texto, outro container. em linguagem de classe, o instinto seria um
`Container extends Widget` com uma lista de `Widget`. em rust, a coisa monta por
composicao. o container guarda os filhos num campo, e cada filho e um `Box<dyn
View>`. olha o `ContainerView`, de `crates/engine/src/view/views.rs`:

```rust
pub struct ContainerView {
    pub style: LayoutStyle,
    pub children: Vec<Box<dyn View>>,
    pub background: Option<[f32; 4]>,
}

impl View for ContainerView {
    fn layout(&self) -> LayoutStyle {
        self.style.clone()
    }

    fn children(&self) -> &[Box<dyn View>] {
        &self.children
    }

    fn render(&self, cx: &mut ViewContext) -> Vec<SceneNode> {
        if let Some(color) = self.background {
            vec![SceneNode::Rect {
                x: cx.bounds.x,
                y: cx.bounds.y,
                w: cx.bounds.width,
                h: cx.bounds.height,
                color,
            }]
        } else {
            vec![]
        }
    }
}
```

o `ContainerView` nao herda dos filhos. ele tem filhos. `children:
Vec<Box<dyn View>>`. essa e a frase inteira do design da arvore, e a diferenca
com heranca e exatamente essa preposicao. "tem", nao "e". o container nao e um
filho generalizado, ele e uma coisa que guarda filhos e sabe onde se encaixam.
quando o `render` dele roda, ele desenha so o proprio fundo, e devolve o vec. os
filhos sao renderizados por quem caminha a arvore, lendo `children()` e chamando
o `render` de cada um. o `style` vira layout via `layout()`, que o taffy usa pra
calcular onde cada filho cai. a heranca empilharia comportamento numa cadeia de
ancestrais. a composicao empilha objetos numa lista, e o comportamento comum
mora num lugar so, a trait `View`, que todos implementam de forma plana, sem
cadeia.

o `Box<dyn View>` e a peca que faz a heterogeneidade funcionar, e ela merece uma
digressao porque e onde "bound de trait substitui heranca" vira concreto. `dyn
View` e um trait object: um ponteiro pro dado mais um ponteiro pra uma tabela de
metodos, a vtable. quando voce chama `view.render(cx)` num `Box<dyn View>`, o
programa olha a vtable em tempo de execucao pra descobrir qual `render` chamar, o
do `RectView`, o do `TextView`, o do `ContainerView`. e despacho dinamico, primo
direto do metodo virtual da heranca. a diferenca e que nao tem ancestral. o que
liga os tres tipos nao e uma classe base comum, e o fato de os tres
implementarem o contrato `View`. o bound substitui a base. um teste no proprio
crate mostra isso rodando, em `crates/engine/src/view/tests.rs`:

```rust
let views: Vec<Box<dyn View>> = vec![
    Box::new(RectView { /* ... */ }),
    Box::new(TextView { /* ... */ }),
];
let mut cx = test_cx();
let total_nodes: usize = views.iter().map(|v| v.render(&mut cx).len()).sum();
assert_eq!(total_nodes, 2);
```

um `RectView` e um `TextView`, dois tipos sem parentesco nenhum, vivem no mesmo
`Vec` e respondem `render` cada um do seu jeito, porque os dois prometeram ser
`View`. o `iter().map(|v| v.render(...))` nao sabe nem quer saber qual tipo
concreto esta ali dentro. ele confia no contrato. isso e o polimorfismo que voce
queria da heranca, sem a arvore de heranca.

e aqui cabe a digressao que sempre confunde quem vem de outra linguagem: dyn nao
e a unica ferramenta, e nem sempre e a certa. rust tem dois jeitos de ser
polimorfico. o `dyn View` resolve em runtime, pela vtable, e e o que voce usa
quando a colecao e heterogenea, quando voce precisa de tipos diferentes na mesma
lista, como os filhos de um container. o outro jeito e generico,
`fn coisa<V: View>(v: V)`, que o compilador monomorfiza: ele gera uma copia da
funcao pra cada tipo concreto que voce usar, e o despacho some, vira chamada
direta, custo zero em runtime. um e flexivel e cobra uma indirecao. o outro e
rigido e nao cobra nada. a arvore de views usa `dyn` porque precisa misturar
tipos numa lista so. ja o `Component`, que vem na proxima secao, usa generico,
porque cada componente sabe o seu tipo na hora de compilar. duas ferramentas pra
duas necessidades, e nos dois casos a frase de fundo e a mesma: o bound e o
contrato. `dyn View` ou `V: View`, o que une os tipos e a promessa, nunca um
ancestral.

## Lifecycle, e por que ela nao e uma View

ate aqui toda `View` foi pura. olha de novo a assinatura do `render`:
`fn render(&self, ...)`. `&self`, emprestado imutavel. uma view nao muda. ela
descreve o que ela e agora e devolve os nos. isso e otimo enquanto a tela e
estatica, mas interface de verdade tem estado. um contador que incrementa, um
campo de texto que acumula caractere, um switch que lembra se esta ligado. esse
estado precisa sobreviver entre frames, e precisa poder mudar. `&self` nao deixa
mudar nada. e ai que entra a segunda trait.

a `Lifecycle` vive em `crates/engine/src/component/mod.rs`, e o nome dela ja
avisa que ela e sobre tempo de vida, sobre montar, atualizar e desmontar:

```rust
pub trait Lifecycle {
    type State;

    fn initial_state(&self) -> Self::State;

    fn on_mount(&self, _state: &mut Self::State) {}

    fn on_update(&self, _state: &mut Self::State) {}

    fn on_unmount(&self, _state: &mut Self::State) {}

    fn render(&self, state: &Self::State, cx: &mut ViewContext) -> Vec<SceneNode>;
}
```

primeiro detalhe, `type State`. e um tipo associado. cada implementacao de
`Lifecycle` escolhe o proprio tipo de estado. um contador usa `u64`. um dashboard
usa um struct com `count` e `label`. um widget sem estado usa `()`. a trait nao
sabe qual e, ela so promete que existe um `State` e que da pra produzir um valor
inicial dele com `initial_state`. e o contrato deixando o "o que" pro
implementador e fixando so o "que existe um".

segundo, os tres hooks de ciclo, `on_mount`, `on_update`, `on_unmount`. todos com
corpo vazio por default, entao implementar `Lifecycle` so obriga a escrever
`initial_state` e `render`, o resto voce sobrescreve se precisar. e repara que os
hooks recebem `&mut Self::State`. eles podem mudar o estado. mas a propria
`Lifecycle`, o `self`, continua `&self` em todo lugar, imutavel. a logica do
componente nao muda, o estado dele muda. essa separacao entre "a regra" (o
`self`, fixo) e "o estado" (o `State`, mutavel) e o coracao do design, e e por
isso que `Lifecycle` nao e e nao pode ser uma `View`.

o adr `kdb/adr/component-design.md` registra a decisao com uma frase que eu
queria ter escrito antes de sofrer pra chegar nela: `View::render` usa `&self`,
stateless, e `Component::render` usa `&mut self`, porque os hooks de ciclo mutam
estado. sao dois caminhos distintos. a view e declaracao pura, sem estado, logo
`&self`. o componente e um wrapper com estado persistente, logo `&mut self`. e o
adr crava o que pra mim foi o pulo do gato: o componente nao implementa `View`.
se voce forcasse `Component` a implementar `View`, o `render` da `View` e `&self`,
e voce teria que mudar o estado por tras de um emprestimo imutavel. em rust isso
exige `RefCell`, ou seja, mover a checagem de emprestimo pra runtime, com custo e
com a possibilidade de panico em tempo de execucao. isso viola a restricao de
zero overhead em runtime que o projeto se impos. entao a engine nao casa as duas
traits. ela deixa `View` pura e poe o estado num wrapper separado.

esse wrapper e o `Component<L>`, em
`crates/engine/src/component/lifecycle_impl.rs`. e ele e generico sobre a
`Lifecycle`, aquele monomorfismo de custo zero que eu citei na digressao:

```rust
pub struct Component<L: Lifecycle> {
    inner: L,
    state: L::State,
    mounted: bool,
    cached_nodes: Option<Vec<SceneNode>>,
    needs_render: bool,
}
```

`inner` e a logica (a sua `Lifecycle`), `state` e o estado dela, e os outros tres
campos sao a maquinaria de ciclo e cache. agora o `render` do wrapper, que e onde
o `&mut self` aparece e onde os hooks disparam:

```rust
pub fn render(&mut self, cx: &mut ViewContext) -> Vec<SceneNode> {
    if !self.mounted {
        self.inner.on_mount(&mut self.state);
        self.mounted = true;
        self.needs_render = true;
    } else {
        self.inner.on_update(&mut self.state);
    }

    if !self.needs_render
        && let Some(ref cached) = self.cached_nodes
    {
        return cached.clone();
    }

    let nodes = self.inner.render(&self.state, cx);
    self.cached_nodes = Some(nodes.clone());
    self.needs_render = false;
    nodes
}
```

tem uma linha aqui que parece banal e nao e: `self.inner.on_mount(&mut
self.state)`. o adr de component-design dedica uma secao inteira a ela, e com
razao. essa chamada toma `self.inner` emprestado imutavel (pra chamar o metodo) e
`self.state` emprestado mutavel (pra modificar o estado), na mesma expressao. em
muita linguagem isso seria um conflito. em rust compila, e compila porque `inner`
e `state` sao campos disjuntos do mesmo struct. o borrow checker enxerga campo a
campo, sabe que `inner` e `state` sao pedacos de memoria diferentes, e libera o
emprestimo imutavel de um junto com o mutavel do outro. sem `RefCell`, sem
lifetime, sem custo. e o adr resume seco: "sem lifetimes, sem refcell". essa e a
diferenca entre brigar com o borrow checker e usar ele a teu favor. ele nao esta
no teu caminho, ele esta separando duas coisas que de fato sao separadas, e
deixando voce tocar nas duas ao mesmo tempo justamente porque elas sao
separadas.

o resto do `render` e o ciclo virando codigo. na primeira vez, `mounted` e
`false`, entao `on_mount` dispara, marca `mounted = true`, e forca um render. nas
vezes seguintes, `on_update` dispara. depois vem o cache: se `needs_render` e
`false` e existe `cached_nodes`, devolve o cache clonado e nem chama o `render`
da `Lifecycle`. senao, roda o `render`, guarda o resultado em `cached_nodes`, e
zera `needs_render`. um componente que nao mudou de estado devolve os nos do
frame passado sem reconstruir nada. isso e o mesmo "nao refaca o que nao mudou"
do dirty tracking, so que uma escala acima, no componente, antes mesmo de a cena
chegar no compositor.

e quem liga o `needs_render` de volta? duas coisas. o acessor de estado:

```rust
pub fn state_mut(&mut self) -> &mut L::State {
    self.needs_render = true;
    &mut self.state
}
```

toda vez que alguem pega o estado pra escrever, via `state_mut`, o cache se
invalida sozinho. voce nao precisa lembrar de marcar dirty, o ato de pedir o
estado mutavel ja marca. e tem o `invalidate()`, a valvula manual pra quando algo
muda fora do estado e o componente precisa redesenhar mesmo assim. o adr ja
antecipa que isso casa com o sistema de signals que vem depois: `state_mut`
permite que signals escrevam no estado de fora, e `on_update` pode ser
condicionado por flags de dirty de signal, sem mudar a trait `Lifecycle`. o
contrato foi desenhado pra nao precisar mudar quando o reativo chegar.

a ultima peca e o `Drop`, e ela e a parte que me fez confiar no design:

```rust
impl<L: Lifecycle> Drop for Component<L> {
    fn drop(&mut self) {
        if self.mounted {
            self.inner.on_unmount(&mut self.state);
        }
    }
}
```

quando o `Component` sai de escopo, o rust chama `drop`, e o `drop` dispara
`on_unmount`, mas so se `mounted` for `true`. um componente que foi construido e
nunca renderizou nunca montou, entao nunca desmonta. isso nao e teoria, esta
travado em teste. `on_unmount_fires_on_drop` cria um componente, renderiza, deixa
sair de escopo, e verifica que o `on_unmount` rodou.
`unmount_not_called_if_never_mounted` cria um componente, nunca renderiza, deixa
sair de escopo, e verifica que `on_unmount` nao rodou. o ciclo de vida e
simetrico e o `Drop` do rust garante que o desmonte acontece, sem voce ter que
lembrar de chamar um `dispose()` na mao como em runtime de UI que depende de
garbage collector. o destrutor deterministico do rust faz o trabalho que em outra
linguagem voce faria errado em algum canto e so descobriria com vazamento.

vale fechar essa secao com o que os testes garantem, porque eles sao o contrato
executavel. `on_mount_fires_on_first_render` prova que o estado ja esta montado
quando o primeiro `render` roda. `on_update_fires_on_subsequent_renders` prova
que o contador sobe a cada frame depois do primeiro.
`cache_returns_same_nodes_without_state_change` prova que dois renders seguidos
sem mudar estado devolvem os mesmos nos. `cache_invalidated_by_state_mut` prova
que escrever via `state_mut` muda a saida no proximo render.
`invalidate_forces_rerender` prova que `invalidate()` faz o `render` da
`Lifecycle` rodar de novo mesmo sem mudanca de estado. cada uma dessas frases do
contrato tem um teste com o mesmo nome guardando ela.

## Interpolate, o contrato menor de todos

a terceira trait e a menor, e e a que melhor mostra a ideia de bound como costura
do sistema. ela vive em `crates/engine/src/animation/tween.rs`, e tem um metodo
so:

```rust
pub trait Interpolate: Clone {
    fn lerp(&self, target: &Self, t: f32) -> Self;
}
```

`Interpolate: Clone` quer dizer que pra ser `Interpolate` voce ja precisa ser
`Clone`, e a trait so adiciona um metodo, `lerp`, a interpolacao linear: dado um
valor de partida (`self`), um alvo (`target`) e um fator `t` entre 0 e 1, devolve
o valor no meio do caminho. `t = 0` devolve o comeco, `t = 1` devolve o fim,
`t = 0.5` devolve o ponto medio. e so isso. um contrato de uma linha util.

a parte boa vem das implementacoes em branco, as blanket impls:

```rust
impl Interpolate for f32 {
    fn lerp(&self, target: &Self, t: f32) -> Self {
        self + (target - self) * t
    }
}

impl<const N: usize> Interpolate for [f32; N] {
    fn lerp(&self, target: &Self, t: f32) -> Self {
        std::array::from_fn(|i| self[i] + (target[i] - self[i]) * t)
    }
}
```

a primeira ensina o `f32` a se interpolar. a segunda, com const generic
`<const N: usize>`, ensina qualquer array de `f32` de qualquer tamanho de uma vez
so. um `[f32; 2]` (posicao), um `[f32; 4]` (cor RGBA), um `[f32; 5]`, todos ganham
`lerp` sem voce escrever uma linha por tamanho. uma impl, infinitos tamanhos. e
onde isso paga e no `Tween`, a struct de animacao, que e generica sobre o
contrato:

```rust
#[derive(Clone, Debug)]
pub struct Tween<T: Interpolate> {
    from: T,
    to: T,
    duration: f32,
    elapsed: f32,
    easing: Easing,
    state: TweenState,
    delay: f32,
    repeat: Repeat,
    reverse: bool,
}
```

`Tween<T: Interpolate>`. o tween nao sabe o que e `T`. nao sabe se e uma cor, uma
posicao, uma opacidade, um angulo. ele so sabe que `T` sabe se interpolar, porque
`T: Interpolate` e o bound. la dentro, na hora de calcular o valor atual, o tween
aplica a curva de easing pra achar o `t` e chama o contrato:

```rust
let eased = ease(t.clamp(0.0, 1.0), self.easing);
self.from.lerp(&self.to, eased)
```

`self.from.lerp(&self.to, eased)`. e a unica coisa que o tween faz com o tipo
animado: chama `lerp`. todo o resto, easing, delay, repeat, reverse, estado, e
maquinaria que funciona igual pra qualquer `T`. o sistema de animacao foi escrito
uma vez e anima tudo que implementa o contrato. o teste `tween_color` mostra a
mesma struct que anima um `f32` animando uma cor `[f32; 4]` sem nenhuma mudanca:

```rust
let mut tw = Tween::new([0.0, 0.0, 0.0, 1.0_f32], 1.0, Easing::Linear);
tw.set_target([1.0, 1.0, 1.0, 1.0]);
tw.tick(0.5);
let c = tw.get();
assert!((c[0] - 0.5).abs() < 0.01);
assert!((c[3] - 1.0).abs() < 0.01);
```

preto pra branco, na metade do tempo, da cinza no meio. o mesmo `tick`, o mesmo
`get`, o mesmo `Tween`, so com `T` diferente. o bound e a costura: ele e o ponto
onde o sistema de animacao se encosta no tipo animado, sem conhecer ele.

e tem uma camada a mais que mostra traits compondo entre si, que e o `Spring`, em
`crates/engine/src/animation/spring.rs`. mola precisa de mais do que `lerp`, ela
precisa somar, subtrair, escalar e medir magnitude pra integrar a fisica. entao
existe uma trait que estende a primeira:

```rust
pub trait SpringInterpolate: Interpolate {
    fn add(&self, other: &Self) -> Self;
    fn sub(&self, other: &Self) -> Self;
    fn scale(&self, s: f32) -> Self;
    fn magnitude_sq(&self) -> f32;
}
```

`SpringInterpolate: Interpolate`. isso e composicao de contrato, nao heranca de
classe. quem implementa `SpringInterpolate` ja tem que implementar `Interpolate`
(e por transitividade `Clone`). o contrato maior empilha em cima do menor, e o
compilador empilha junto: um `Spring<T: SpringInterpolate>` ganha de graca tudo
que `Interpolate` da, mais os quatro metodos de algebra vetorial. e onde isso
aparece de verdade na engine: o widget de switch, em
`crates/engine/src/ui/widgets/switch.rs`, guarda a posicao do botao numa mola,
`knob: Spring<f32>`, e move ela com `set_target` e `tick`. quando voce liga o
switch, `self.knob.set_target(1.0)`, e a mola desliza o botao com fisica de
verdade, sem voce escrever a integracao, porque `f32` implementa
`SpringInterpolate`, que e `Interpolate`, que e `Clone`, a pilha inteira de
contratos satisfeita por um numero de quatro bytes.

um aviso de honestidade: entre as ancoras deste capitulo nao ha um adr dedicado a
`Interpolate`. a decisao de design dela esta no proprio codigo, em `tween.rs` e
`spring.rs`, e eu ancorei tudo o que afirmei aqui direto na fonte. o sumario do
livro aponta a parte 2.11 (animation) pra um adr chamado `animation-pattern-lerp`,
que eu nao li pra escrever este capitulo. entao trato o design de `Interpolate`
como confirmado pelo source, e marco que nao ha adr lido sustentando ele.

## por que assim: tres contratos, tres jeitos de pegar o self

junta as tres traits e o padrao salta. cada uma escolhe o emprestimo minimo que
ela precisa do estado, e essa escolha e a arquitetura.

`View::render` usa `&self`. a view e pura, ela so descreve, nao muda. emprestimo
imutavel basta, e por causa disso ela e trivial de cachear, de testar, de
compor. `Lifecycle::render` tambem usa `&self`, mas a logica fica embrulhada num
`Component<L>` cujo `render` e `&mut self`, e e o wrapper que carrega o estado
mutavel e os hooks. o estado mutavel existe, mas mora fora da declaracao pura, num
campo separado que o borrow checker enxerga como disjunto. `Interpolate::lerp`
usa `&self` e um `target: &Self`, e devolve um `Self` novo. ele nao muda nem um
nem outro, produz um terceiro valor no meio do caminho. tres contratos, tres
formas de tocar o `self`, e cada forma e a resposta certa pra natureza daquele
contrato. essa coerencia nao foi sorte, foi o adr de component-design recusando
casar `View` com estado e o adr de view-trait-design recusando dar ao
`ViewContext` uma referencia ao compositor.

a heranca te daria uma cadeia. uma classe base, uma derivada, uma neta, e o
comportamento escorregando de cima pra baixo. as traits te dao uma malha. `View`
de um lado, `Lifecycle` de outro, `Interpolate` de outro, sem ancestral comum,
ligadas so onde voce explicitamente compoe, como `SpringInterpolate` empilhando
em `Interpolate`. e rust nem te oferece heranca de struct pra voce ter a tentacao,
o que no comeco eu achei limitacao e hoje acho disciplina. voce e obrigado a
perguntar, pra cada relacao, se ela e "e um" (e ai talvez uma trait) ou "tem um"
(e ai um campo). o `ContainerView` "tem" filhos. o `Component` "tem" estado e
"tem" uma `Lifecycle`. o `Tween` "tem" um `from` e um `to` que sabem se
interpolar. nenhum deles "e" o outro.

e tudo isso e contrato compilado, nao acordo de cavalheiros. o compilador checa a
promessa antes de rodar. se voce diz que um tipo e `View` e esquece o `render`,
nao compila. se voce poe num `Vec<Box<dyn View>>` algo que nao implementa `View`,
nao compila. se voce tenta animar com `Tween` um tipo que nao implementa
`Interpolate`, nao compila. a promessa e verificada estaticamente, e onde ela e
generica (o `Component<L>`, o `Tween<T>`) ela e monomorfizada, o despacho some, o
custo de runtime e zero. onde ela precisa de heterogeneidade (a lista de filhos)
ela e `dyn`, e paga uma indirecao de vtable de boa vontade. o titulo do capitulo
nao e metafora. a trait e literalmente um contrato que o compilador compila e
exige.

uma nota de escopo, pra fechar honesto: este capitulo nao tem numero de
benchmark. o sumario do livro lista a secao 1.2 com `bench: n/a`, e e correto. o
ganho aqui nao se mede em microssegundo, ele se mede em quantas classes base voce
nao escreveu e quantos bugs de estado compartilhado o borrow checker recusou
antes de virarem crash. o numero, quando vem, vem nos capitulos de compositor, de
signals, de layout. aqui o resultado e estrutural. e o tipo de coisa que so
aparece no boletim seis meses depois, quando voce muda o sistema de animacao
inteiro e nenhuma view quebra, porque elas nunca souberam que o animation
existia.

## o que isso me ensinou

o reflexo de heranca morre devagar. eu ainda abro o editor querendo uma classe
base, e ainda levo um segundo pra lembrar que o lugar daquele campo comum nao e
um ancestral, e um campo, e o lugar daquele comportamento comum nao e uma
superclasse, e uma trait que varios tipos implementam de forma plana. o que
substituiu a heranca na minha cabeca e menor e mais severo. um contrato que cabe
em nove linhas pro `View`, em treze pro `Lifecycle`, em quatro pro `Interpolate`.
cada um pedindo a unica coisa que precisa, e nada alem.

a parte que eu levei mais tempo pra internalizar nao foi a sintaxe da trait, foi
que o `&self` contra o `&mut self` nao e detalhe de assinatura, e decisao de
arquitetura escrita na menor unidade possivel. a engine inteira do lado
declarativo esta naquela escolha repetida tres vezes: a view nao muda, o
componente muda por baixo de um wrapper, o valor interpolado nasce novo. quando
eu vejo um `&self` num `render` hoje, eu leio "isto e cacheavel, isto e
testavel, isto nao vai te surpreender". e quando vejo o `&mut self` do
`Component`, eu leio "aqui mora o estado, e o `Drop` vai limpar depois de voce".

se eu fosse deixar uma coisa so disto pra quem pega o codebase depois: nao
procure a classe base, ela nao existe e e melhor assim. procure o contrato.
quando voce achar a trait, voce achou a promessa que todo o resto cumpre, e ela
e pequena de proposito, porque a unica promessa que vale e a que o compilador
consegue cobrar inteira.

## rastros

adr e decisoes de design
- `kdb/adr/view-trait-design.md:12-16` (assinatura `View` no adr com so o
  `render`; o codigo de hoje tem `layout` e `children` defaultados a mais)
- `kdb/adr/view-trait-design.md:18-31` (viewcontext sem referencia ao compositor;
  razoes: testabilidade, composabilidade, simplicidade sem lifetime,
  desacoplamento; trade-off do `Vec` por render)
- `kdb/adr/view-trait-design.md:32-34` (`&mut ViewContext` para extensao futura
  mesmo sendo read-only hoje)
- `kdb/adr/view-trait-design.md:40-48` (snippet de integracao com `window.rs`
  usando `ViewContext { width, height }`, que nao compila contra o tipo atual)
- `kdb/adr/component-design.md:11-18` (`View::render` `&self` stateless vs
  `Component::render` `&mut self`; dois caminhos; component nao implementa View;
  `RefCell` violaria zero overhead)
- `kdb/adr/component-design.md:20-23` (borrow checker, campos disjuntos,
  `self.inner.on_mount(&mut self.state)` compila sem lifetimes nem refcell)
- `kdb/adr/component-design.md:24-29` (estado via acessores, nao via viewcontext;
  `Lifecycle::render` recebe estado; `state()`/`state_mut()`)
- `kdb/adr/component-design.md:30-32` (drop chama `on_unmount` so se `mounted`)
- `kdb/adr/component-design.md:33-39` (compatibilidade com signals: `on_update`
  gated por dirty flags, `state_mut` para escrita externa)

codigo (crate engine)
- `crates/engine/src/view/trait_def.rs:9-19` (trait `View`: `layout` e `children`
  com default, `render` obrigatorio devolvendo `Vec<SceneNode>`)
- `crates/engine/src/view/trait_def.rs:8` (doc comment: produz SceneNodes sem
  tocar o compositor)
- `crates/engine/src/view/context.rs:7-21` (`ViewContext` com oito campos)
- `crates/engine/src/view/context.rs:23-40` (`ViewContext::new(width, height)`)
- `crates/engine/src/view/views.rs:13-41` (`ContainerView`, `children:
  Vec<Box<dyn View>>`, render so do fundo)
- `crates/engine/src/view/views.rs:47-73` (`RectView`)
- `crates/engine/src/view/views.rs:79-103` (`TextView`)
- `crates/engine/src/view/tests.rs:118-141` (`dyn_view_dispatch_works`, RectView e
  TextView no mesmo `Vec<Box<dyn View>>`)
- `crates/engine/src/component/mod.rs:17-29` (trait `Lifecycle`: `type State`,
  `initial_state`, hooks default, `render` com estado)
- `crates/engine/src/component/lifecycle_impl.rs:8-14` (`Component<L>` struct,
  campos `inner`, `state`, `mounted`, `cached_nodes`, `needs_render`)
- `crates/engine/src/component/lifecycle_impl.rs:28-47` (`Component::render`,
  mount/update, cache por `needs_render`)
- `crates/engine/src/component/lifecycle_impl.rs:30` (`self.inner.on_mount(&mut
  self.state)`, campos disjuntos)
- `crates/engine/src/component/lifecycle_impl.rs:53-56` (`state_mut` liga
  `needs_render`)
- `crates/engine/src/component/lifecycle_impl.rs:59-61` (`invalidate`)
- `crates/engine/src/component/lifecycle_impl.rs:64-70` (`Drop`, `on_unmount` so
  se `mounted`)
- `crates/engine/src/component/tests.rs:47-68` (`on_mount_fires_on_first_render`)
- `crates/engine/src/component/tests.rs:70-80`
  (`on_update_fires_on_subsequent_renders`)
- `crates/engine/src/component/tests.rs:111-132` (`on_unmount_fires_on_drop`)
- `crates/engine/src/component/tests.rs:134-153`
  (`unmount_not_called_if_never_mounted`)
- `crates/engine/src/component/tests.rs:198-227`
  (`cache_returns_same_nodes_without_state_change`)
- `crates/engine/src/component/tests.rs:229-260` (`cache_invalidated_by_state_mut`)
- `crates/engine/src/component/tests.rs:262-284` (`invalidate_forces_rerender`)
- `crates/engine/src/animation/tween.rs:3-5` (trait `Interpolate: Clone`, `lerp`)
- `crates/engine/src/animation/tween.rs:7-11` (`impl Interpolate for f32`)
- `crates/engine/src/animation/tween.rs:13-17` (`impl<const N: usize> Interpolate
  for [f32; N]`)
- `crates/engine/src/animation/tween.rs:33-44` (`Tween<T: Interpolate>` struct)
- `crates/engine/src/animation/tween.rs:162-163` (`self.from.lerp(&self.to,
  eased)`)
- `crates/engine/src/animation/tests_tween.rs:98-106` (`tween_color`, mesma struct
  animando `[f32; 4]`)
- `crates/engine/src/animation/spring.rs:3-8` (trait `SpringInterpolate:
  Interpolate`, supertrait)
- `crates/engine/src/ui/widgets/switch.rs:31` (`knob: Spring<f32>`)
- `crates/engine/src/ui/widgets/switch.rs:79` (`self.knob.set_target(...)`)
- `crates/engine/src/ui/widgets/switch.rs:84` (`self.knob.tick(dt)`)

versoes (conferidas contra o Cargo.toml)
- `Cargo.toml:23` edition 2024
- `Cargo.toml:24` rust-version 1.85
- `Cargo.toml:50` wgpu 28
- `Cargo.toml:68` rustc-hash 2.1 (dirty tracking, citado de passagem)
- `Cargo.toml:70` web-time 1.1 (`Instant` do `FrameClock`)
- `Cargo.toml:99` criterion 0.5

nao confirmado
- nao ha adr entre as ancoras deste capitulo cobrindo `Interpolate`. o design
  esta ancorado direto em `tween.rs` e `spring.rs`. o sumario aponta a parte 2.11
  para um adr `animation-pattern-lerp` que nao foi lido aqui.
- o adr `view-trait-design.md` esta defasado em relacao ao source em dois pontos:
  a assinatura de `View` (so `render` no adr, tres metodos no codigo) e o struct
  literal de `ViewContext` (dois campos no adr, oito no codigo). confirmado lendo
  adr e source lado a lado.
- secao 1.2 sem benchmark (`bench: n/a` no SUMARIO.md). o capitulo nao traz numero
  medido de proposito.
- commit de origem das tres traits nao rastreavel com seguranca: o historico git
  anterior foi parcialmente perdido (forgejo self-hosted deletado). os arquivos
  estao hoje sob o branch `refactor/workspace-restructure`; os adrs
  `view-trait-design.md` e `component-design.md` marcam `last-updated: 2026-03-08`.
