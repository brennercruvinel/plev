---
title: "o layer system: camadas, offscreen e composite pass"
parte: 2
status: rascunho
rastros:
  - kdb/adr/layer-system.md
  - crates/engine/src/compositor/layer/mod.rs
  - crates/engine/src/compositor/layer/texture.rs
  - crates/engine/src/compositor/mod.rs
  - crates/engine/src/gpu/utils.rs
  - crates/engine/src/gpu/shaders/composite.wgsl
  - crates/engine/src/gpu/shaders/quad.wgsl
  - crates/engine/src/gpu/pipelines.rs
  - crates/engine/src/window/render_passes.rs
  - crates/engine/src/window/render.rs
  - kdb/caranguejovermelho/livro/cap-amostra-dirty-tracking.md
---

# o layer system: camadas, offscreen e composite pass

pega uma folha de acetato, daquelas de retroprojetor. desenha um fundo cinza
numa, um painel branco noutra, um menu que abre na terceira. pra ver a tela
montada, voce empilha as folhas e olha de cima. nada na folha de baixo precisa
saber o que tem na de cima. se o menu fecha, voce so tira uma folha, as outras
duas continuam onde estavam, intactas. ninguem redesenha o fundo porque o menu
sumiu.

o plev faz a tela assim. a imagem que voce ve nao e uma superficie unica que a
engine repinta inteira a cada frame. ela e uma pilha de camadas, cada uma na sua
folha de acetato, e cada folha mora numa textura propria fora da tela, uma
textura offscreen. no fim de tudo a engine empilha as folhas com um unico
desenho por camada e manda pro monitor. esse capitulo e sobre as tres pecas que
fazem essa pilha funcionar: a textura offscreen onde cada layer vive, o
premultiplied alpha que deixa empilhar dar o resultado certo, e o composite pass
que faz o empilhamento com um truque de um triangulo so. o dirty tracking, a
parte que decide quando uma folha precisa ser repintada, ja tem o seu proprio
capitulo (`cap-amostra-dirty-tracking.md`), entao aqui eu encosto nele de leve e
foco no resto.

a versao do que estou olhando: wgpu 28, edition 2024 (`Cargo.toml:50` e
`Cargo.toml:23`). o resto da stack, winit 0.30, cosmic-text 0.18, taffy 0.9, nao
entra muito aqui, esse e um capitulo de GPU.

## a tela e uma pilha, nao uma folha

a estrutura comeca seca. uma layer tem um id e uma ordem.

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(pub u32);

impl LayerId {
    pub const DEFAULT: LayerId = LayerId(0);
}
```

esse `LayerId(0)`, o `DEFAULT`, e a primeira folha da pilha, e ela sempre existe.
quando o `Compositor` nasce, ele ja empurra a layer default pra dentro:

```rust
pub fn new() -> Self {
    let mut comp = Self {
        layers: Vec::new(),
        next_layer_id: 1,
        sorted: true,
        invalidated: false,
        stats: RenderStats::default(),
    };
    comp.layers.push(Layer::new(LayerId::DEFAULT, 0));
    comp
}
```

a consequencia disso e o que torna a API gostosa de usar. quando voce chama
`draw_rect`, voce nem pensa em layer. o retangulo cai na default:

```rust
pub fn push(&mut self, node: SceneNode) {
    self.push_to_layer(LayerId::DEFAULT, node);
}

pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
    self.push(SceneNode::Rect { x, y, w, h, color });
}
```

camada e uma coisa que voce so cria quando precisa que um pedaco da tela viva em
separado. um painel que anima sozinho, um overlay que entra por cima, um HUD de
performance que pisca numero. cada `create_layer` recebe uma ordem z e marca a
pilha como nao ordenada:

```rust
pub fn create_layer(&mut self, z_order: i32) -> LayerId {
    let id = LayerId(self.next_layer_id);
    self.next_layer_id += 1;
    self.layers.push(Layer::new(id, z_order));
    self.sorted = false;
    id
}
```

a ordenacao por z so acontece quando a engine vai resolver a cena, e so quando
`sorted` esta falso. e um `sort_by_key` preguicoso: se ninguem mexeu na ordem das
camadas desde o ultimo frame, o vetor ja esta ordenado e o sort nem roda. quando
roda, e por `z_order`, do menor pro maior, que e a ordem de empilhamento de
baixo pra cima.

esse e o esqueleto: um vetor de layers, a default no fundo, as outras ordenadas
por z. cada uma carrega a sua propria lista de coisas pra desenhar (os
`SceneNode`), o seu proprio hash de cena, os seus proprios buffers de geometria.
o adr do sistema (`kdb/adr/layer-system.md`) resume a regra em uma linha: cada
layer tem textura offscreen RGBA, hash de cena proprio, buffers proprios, e as
layers saem ordenadas por z. o motivo de existir camada separada nao e
organizacao de codigo, e isolamento de trabalho. quando o menu anima, so a folha
do menu e tocada. o fundo nem acorda.

## a textura offscreen: onde cada folha mora

aqui esta o coracao fisico da coisa. cada layer renderiza pra uma textura que
nao e a tela. ela desenha o seu conteudo num retangulo de pixels guardado na
VRAM, e so depois esse retangulo e estampado na superficie de verdade. a funcao
que garante essa textura existir e a `ensure_texture`, e o nome dela e honesto:
ela garante, nao recria a toa.

```rust
pub(crate) fn ensure_texture(&mut self, res: &ResolveResources<'_>, width: u32, height: u32) {
    let want_msaa = res.msaa_samples > 1;
    if self.tex_width == width
        && self.tex_height == height
        && self.texture.is_some()
        && self.msaa_texture.is_some() == want_msaa
    {
        if let Some(ref buf) = self.opacity_buffer {
            res.queue
                .write_buffer(buf, 0, bytemuck::bytes_of(&self.opacity));
        }
        return;
    }
    // ... daqui pra baixo, cria a textura nova
```

repara no early return. se a largura e a altura batem com o que ja existe, se a
textura ja esta la, e se o estado de MSAA nao mudou, ela nao recria nada. ela so
reescreve o buffer de opacidade (4 bytes, volto nisso) e sai. isso e o que o adr
chama de criar a textura lazily no resolve e recriar so no resize. o caso comum,
frame depois de frame com a janela do mesmo tamanho, nunca aloca textura nova. a
alocacao de VRAM acontece uma vez, no primeiro frame, e depois so quando voce
arrasta a borda da janela.

quando a textura precisa nascer mesmo, o bloco e direto:

```rust
let texture = res.device.create_texture(&wgpu::TextureDescriptor {
    label: Some("layer_texture"),
    size: tex_size,
    mip_level_count: 1,
    sample_count: 1,
    dimension: wgpu::TextureDimension::D2,
    format: res.format,
    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    view_formats: &[],
});
let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
```

o `usage` e o detalhe que conta a historia inteira da camada num par de flags.
`RENDER_ATTACHMENT` quer dizer "eu vou desenhar nesta textura", e
`TEXTURE_BINDING` quer dizer "eu vou ler desta textura num shader depois". a
camada e as duas coisas ao mesmo tempo: alvo de desenho na fase offscreen, fonte
de leitura na fase de composicao. esse e o ciclo de vida de uma folha de
acetato traduzido pra GPU. primeiro voce pinta nela, depois voce olha atraves
dela.

junto com a textura, a `ensure_texture` monta o resto do aparato da camada. um
`composite_bind_group`, que amarra a textura recem criada com um sampler, e o
que o shader de composicao vai usar pra ler a folha. e um par opacidade: um
buffer uniform de 4 bytes e o bind group dele.

```rust
let opacity_buffer = res.device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("layer_opacity_buf"),
    size: 4,
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
});
res.queue
    .write_buffer(&opacity_buffer, 0, bytemuck::bytes_of(&self.opacity));
```

quatro bytes. a opacidade de uma camada inteira e um unico `f32` na GPU. e por
isso que la em cima, no early return, a engine podia reescrever a opacidade sem
recriar nada: mudar quanto uma folha esta transparente custa um `write_buffer` de
4 bytes, nao um redesenho. animar a opacidade de um painel inteiro entrando na
tela e barato nesse nivel, voce nao toca na textura, so no numero que multiplica
ela na hora de compor.

e o ultimo gesto da funcao, quando ela de fato cria textura nova, e o que liga o
offscreen ao dirty tracking:

```rust
self.tex_width = width;
self.tex_height = height;
self.dirty = true;
```

`self.dirty = true`. textura nova significa textura vazia, e textura vazia
precisa ser redesenhada uma vez, custe o que custar, mesmo que a cena nao tenha
mudado em nada. e por isso que resize forca redesenho de todas as camadas: o
`resolve` chama `ensure_texture` pra cada layer antes de qualquer outra coisa, e
cada uma que recriou a textura ja sai marcada como suja. o hash da cena pode
estar identico ao do frame passado, nao importa, a folha de acetato e outra
folha agora, em branco, e tem que ser repintada. essa linha unica e a ponte
entre "a geometria fisica da textura mudou" e "o pipeline de dirty tracking
precisa saber disso". sem ela, depois de um resize voce veria a textura velha
esticada ou lixo de VRAM, porque o dirty tracking, sozinho, olharia o hash, veria
que e igual, e mandaria pular o desenho.

sobre o tamanho dessa folha: o adr registra cerca de 8mb por layer em 1920x1080.
faz sentido na conta de guardanapo, 1920 vezes 1080 da por volta de 2 milhoes de
pixels, 4 bytes por pixel em RGBA da uns 8mb. esse e o preco de manter cada folha
pendurada na VRAM entre frames. e o que voce paga pra poder nao repintar: a
camada limpa do frame passado ainda esta inteira na memoria, pronta pra ser
estampada de novo sem nenhum trabalho de desenho. eu nao medi esses 8mb de forma
independente, e numero do adr, marco como tal.

### o caso MSAA, de leve

tem uma ramificacao que vale citar sem afundar. se o app pediu antialiasing por
multisample, a `ensure_texture` cria uma segunda textura, a de MSAA, com
`sample_count` alto e usage so de `RENDER_ATTACHMENT`. a camada entao desenha na
textura MSAA, com varios samples por pixel, e resolve o resultado pra textura
normal de um sample so, que e a que vai ser lida na composicao. quem decide isso
e a `render_attachment`:

```rust
pub fn render_attachment(&self) -> Option<(&wgpu::TextureView, Option<&wgpu::TextureView>)> {
    match (self.msaa_view.as_ref(), self.texture_view.as_ref()) {
        (Some(msaa), target) => Some((msaa, target)),
        (None, Some(target)) => Some((target, None)),
        (None, None) => None,
    }
}
```

com MSAA, ela devolve a view de MSAA como alvo e a textura normal como resolve
target. sem MSAA, devolve a textura normal direto, sem resolve. o resto do
pipeline nao muda. a composicao sempre le a textura de um sample so, ja
resolvida. o MSAA fica contido dentro da fase offscreen e nao vaza pra fora. e
uma boa divisao: o antialiasing e um detalhe de como a folha foi pintada, nao de
como ela e empilhada.

## premultiplied alpha: por que empilhar exige multiplicar antes

agora a peca que parece detalhe de blend e e na verdade o que faz a pilha inteira
dar o resultado certo. quando voce empilha duas folhas semitransparentes, a cor
que sai e uma mistura. a folha de cima, com a sua cor e a sua transparencia,
deixa passar parte da folha de baixo. o nome matematico dessa mistura e o
operador `over`. e a forma de alimentar a GPU pra que o `over` funcione tem dois
jeitos: o ingenuo e o premultiplicado. o plev usa o premultiplicado em todo o
pipeline, e o adr e explicito que essa foi uma mudanca deliberada, de
`SrcAlpha/OneMinusSrcAlpha` pra `One/OneMinusSrcAlpha`.

o blend state vive numa funcao so, reusada por todo pipeline de desenho:

```rust
pub(crate) fn premultiplied_blend() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    }
}
```

le isso como uma formula. a GPU calcula `resultado = src_factor * src + dst_factor
* dst`. com `src_factor = One` e `dst_factor = OneMinusSrcAlpha`, vira `resultado
= src + (1 - src.a) * dst`. esse e o operador `over` na sua forma premultiplicada.
o `src` ja entra com a cor multiplicada pela propria transparencia, entao ele
contribui com o valor cheio dele (fator One), e o que sobra de transparencia
nele, `1 - src.a`, e quanto da folha de baixo (`dst`) passa atraves.

o "ja entra multiplicado" e a outra metade do contrato. os shaders de desenho
nao mandam a cor crua pra GPU. eles mandam a cor ja multiplicada pelo alpha. olha
o fim do shader de quad:

```rust
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let rgb = srgb_to_linear(in.color.rgb);
    return vec4<f32>(rgb * in.color.a, in.color.a);
}
```

o retorno e `vec4(rgb * a, a)`. a cor sai multiplicada pelo alpha, o alpha sai
puro. e o mesmo padrao no shader de retangulo SDF (`return vec4(rgb * a, a)`), no
de texto, no de imagem, no de sombra. o adr resume como "shaders outputam rgb *
a, a", e isso bate linha por linha com o que esta nos arquivos `.wgsl`. o
premultiplied nao e uma config solta no blend state, e um acordo que atravessa o
pipeline inteiro: todo shader que produz cor a entrega ja multiplicada, e o blend
state foi montado contando com isso.

tem um detalhe de cor enfiado no meio que vale a digressao, porque ele explica o
`srgb_to_linear` ali em cima. as cores do tema vivem em sRGB, o espaco em que
hex de cor e pensado. a GPU faz blend em espaco linear, e a superficie e um
formato sRGB que reencoda de linear pra sRGB na hora de escrever. se o shader
mandasse a cor sRGB crua, a GPU trataria ela como se ja fosse linear, e um cinza
`#303030` apareceria por volta de duas vezes e meia mais claro do que devia. por
isso o shader lineariza primeiro, depois multiplica pelo alpha. a ordem importa:
lineariza, dai premultiplica. isso casa com a regra de engine do projeto, cor e
sRGB, lineariza uma vez ao entrar na GPU. o premultiplied alpha e o
gerenciamento de cor andam juntos no mesmo retorno de duas linhas.

e por que premultiplicado, em vez do alpha blend classico com `SrcAlpha`? a razao
que o adr da e direta: e necessario pra o operador `over` funcionar corretamente
na composicao de layers, e e visualmente identico pra cores opacas. a parte de
"composicao de layers" e onde a escolha deixa de ser cosmetica. quando a engine
le a textura de uma folha pra estampar na tela, ela amostra a textura com
filtragem, e nas bordas onde o alpha de uma camada cai de 1 pra 0, a amostragem
mistura pixels vizinhos. com cor nao premultiplicada, o canal de cor dos pixels
transparentes vaza pra dentro da borda e voce ve uma franja escura ou colorida
em volta do conteudo. com cor premultiplicada, o canal de cor dos pixels
transparentes ja foi multiplicado por zero, entao nao tem cor pra vazar, e a
borda fica limpa. essa parte do mecanismo, a franja de borda, e o raciocinio
padrao por tras do premultiplied, eu coloco como explicacao do porque, o adr
documenta a necessidade pro `over`, nao a franja em detalhe.

tem ainda a propriedade de o `over` premultiplicado ser associavel. compor a
folha A sobre a B, e o resultado disso sobre a C, da o mesmo que compor A sobre o
resultado de B sobre C. isso e o que deixa a engine empilhar uma pilha de
qualquer altura, uma camada de cada vez, e chegar no resultado certo sem ter que
fazer tudo de uma vez so. cada folha estampa sobre o que ja foi composto abaixo
dela, e a matematica fecha. sem essa associatividade, empilhar camada por camada
nao seria igual a misturar todas juntas, e o modelo de folhas de acetato
desmontaria.

## o composite pass: um triangulo que cobre a tela

chega o fim do frame. cada camada ja foi desenhada na sua textura offscreen, ou
nem foi, porque estava limpa e a textura do frame passado ainda serve. agora a
engine precisa empilhar tudo na superficie de verdade, a que o monitor mostra.
esse e o composite pass, e ele e quase ridiculo de simples: pra cada camada
visivel, um desenho que estampa a textura dela na tela. um desenho. nao um por
retangulo, nao um por widget. um por folha.

o jeito de estampar uma textura na tela inteira costuma ser desenhar um retangulo
de tela cheia, dois triangulos formando um quad. o plev nao faz isso. ele usa um
truque conhecido, o triangulo de tela cheia, um triangulo unico grande o
bastante pra cobrir a tela sozinho, sem nenhum vertex buffer. o shader gera os
tres vertices a partir do indice deles:

```rust
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index >> 1u) * 4 - 1);
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}
```

faz a conta dos tres indices na mao, vale a pena. pro indice 0: `x = (0 & 1) * 4
- 1 = -1`, `y = (0 >> 1) * 4 - 1 = -1`, ponto `(-1, -1)`. pro indice 1: `x = (1
& 1) * 4 - 1 = 3`, `y = (1 >> 1) * 4 - 1 = -1`, ponto `(3, -1)`. pro indice 2: `x
= (2 & 1) * 4 - 1 = -1`, `y = (2 >> 1) * 4 - 1 = 3`, ponto `(-1, 3)`. tres
vertices: `(-1,-1)`, `(3,-1)`, `(-1,3)`. um triangulo retangulo cujos catetos
vao de -1 ate 3 em cada eixo. o quadrado visivel da tela em coordenadas de clip
e `[-1, 1]` nos dois eixos, e esse triangulo cobre ele inteiro com folga, sobrando
metade pra fora, que e cortada pelo clipping da GPU. a UV e mapeada junto, de
clip pra `[0, 1]`, com Y invertido porque coordenada de textura cresce pra baixo.
nos pixels que ficam dentro da tela, a UV interpola exatamente de 0 a 1.

por que um triangulo e nao o quad de dois triangulos? duas razoes praticas. um
vertice a menos pra processar, e tres em vez de quatro, detalhe pequeno. e a
ausencia da diagonal: o quad de dois triangulos tem uma aresta compartilhada
cortando a tela na diagonal, e dependendo do hardware essa costura pode gerar
artefato de rasterizacao ou duplicar trabalho na linha do meio. o triangulo unico
nao tem costura interna. e o vertex buffer some inteiro: nenhum vertice e enviado
da CPU, o shader sintetiza tudo a partir de `vertex_index`. o pipeline declara
isso com `buffers: &[]`:

```rust
vertex: wgpu::VertexState {
    module: &shader,
    entry_point: Some("vs_main"),
    buffers: &[],
    compilation_options: Default::default(),
},
```

o fragmento e o anticlimax. amostra a textura da camada e multiplica pela
opacidade:

```rust
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(layer_texture, layer_sampler, in.uv);
    return color * opacity;
}
```

a cor que sai da textura ja esta premultiplicada, foi assim que ela foi escrita
na fase offscreen. multiplicar a cor premultiplicada inteira por um escalar de
opacidade mantem ela premultiplicada (multiplica o rgb e o a pelo mesmo numero),
entao a saida continua valida pro blend state. e aqui a opacidade de 4 bytes da
camada finalmente e usada: ela escala a folha inteira de uma vez, no momento de
estampar. um painel a 50% de opacidade entra no `* opacity` com `0.5` e toda a
textura dele sai pela metade, sem que nenhum dos retangulos e textos que o
compoem soubessem disso.

o bind group layout do composite tem essa cara: grupo 0 e a textura da camada
mais o sampler, grupo 1 e a opacidade.

```rust
@group(0) @binding(0)
var layer_texture: texture_2d<f32>;
@group(0) @binding(1)
var layer_sampler: sampler;

@group(1) @binding(0)
var<uniform> opacity: f32;
```

e o loop que de fato empilha as folhas, o `encode_composite_pass`, fecha o
modelo. uma render pass na superficie, limpa com a cor de fundo, e pra cada
camada visivel um `draw(0..3)`:

```rust
pass.set_pipeline(&gpu.composite_pipeline);

let mut draw_calls = 0u32;
for layer in compositor.layers() {
    if !layer.visible {
        continue;
    }
    // ... escolhe o bind group (com ou sem efeito aplicado)
    if let (Some(bg), Some(opacity_bg)) = (final_bg, layer.opacity_bind_group()) {
        pass.set_bind_group(0, bg, &[]);
        pass.set_bind_group(1, opacity_bg, &[]);
        pass.draw(0..3, 0..1);
        draw_calls += 1;
    }
}
```

`draw(0..3, 0..1)`, tres vertices, uma instancia. um por camada visivel. o adr
poe assim, um draw por layer visivel no composite pass, com o triangulo de tela
cheia. se voce tem quatro folhas na pilha e todas visiveis, o composite pass
inteiro sao quatro draw calls. quatro. nao importa quantos milhares de retangulos
e glifos cada folha contem, porque eles ja foram desenhados na fase offscreen e
agora viraram quatro texturas que so precisam ser estampadas. essa e a economia
estrutural da arquitetura de camadas: o custo da composicao final escala com o
numero de folhas, nao com a quantidade de coisa dentro delas. eu nao tenho um
benchmark isolado do custo desse pass em microssegundo, o que existe sao essas
afirmacoes estruturais do adr e a contagem de draw calls; marco a ausencia do
numero explicitamente.

## o frame inteiro, na ordem certa

vale juntar as pecas na sequencia real, porque a ordem entre offscreen e
composite e o que mantem tudo coerente. o `render` no `window/render.rs` conduz
o frame, e a coreografia e essa.

primeiro, `begin_frame` limpa os nos de todas as camadas. a cena vai ser
redescrita do zero neste frame, entao a lista de coisas a desenhar de cada folha
e zerada antes de qualquer `draw_rect` novo. depois o app descreve a cena de
novo, empurrando retangulo, texto, o que for, pra dentro das camadas.

dai vem o `resolve`. ele faz, em ordem: garante a textura de cada camada com
`ensure_texture` (e e aqui que resize marca tudo como sujo), resolve a cena com
dirty tracking e build de geometria pras camadas sujas, e faz o upload da
geometria pra GPU, tambem so pras sujas. o capitulo de dirty tracking destrincha
essa parte, o `compute_hash`, o `resolve_dirty`, o porque de uma camada limpa
custar zero. o que importa aqui e o resultado: depois do `resolve`, cada camada
suja tem geometria nova na VRAM, e cada camada limpa tem a geometria de antes,
intocada.

agora a engine separa quem vai pra fase offscreen:

```rust
let dirty_layer_ids: Vec<_> = self
    .compositor
    .layers()
    .iter()
    .filter(|l| l.visible && l.is_dirty())
    .map(|l| l.id)
    .collect();
```

so as camadas visiveis e sujas entram na lista. e essa lista que alimenta o
`encode_layer_passes`, a fase offscreen. pra cada id sujo, uma render pass na
textura daquela camada, limpando ela primeiro pra transparente:

```rust
let mut pass = begin_layer_pass(
    encoder,
    view,
    resolve_target,
    wgpu::LoadOp::Clear(wgpu::Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    }),
);
```

esse clear pra `(0, 0, 0, 0)` e o detalhe que amarra o offscreen de volta no
premultiplied. a textura da folha comeca preta e totalmente transparente. onde a
camada nao desenha nada, fica `(0,0,0,0)`, que e exatamente o elemento neutro do
`over` premultiplicado: cor zero, alpha zero, contribuicao zero quando essa folha
for estampada sobre as de baixo. preto premultiplicado com alpha zero e
transparente de verdade, sem cor pra vazar. se o clear fosse pra preto opaco, cada
folha taparia tudo abaixo dela com um retangulo preto, e a pilha viraria so a
folha de cima. o clear transparente e o que deixa as folhas serem de fato
semitransparentes umas sobre as outras.

dentro dessa pass, a camada desenha a sua sequencia de comandos na ordem em que
foram empurrados, quad, retangulo SDF, sombra, imagem, texto, cada um com o seu
pipeline e o seu blend premultiplicado. e o conteudo da folha sendo pintado.

depois do offscreen, e antes do composite, a engine marca as camadas como limpas:

```rust
for id in &dirty_layer_ids {
    self.compositor.mark_layer_clean(*id);
}
```

`mark_layer_clean` so apaga o flag `dirty`. a camada acabou de ser redesenhada,
entao ela esta em dia com a sua cena ate alguem mexer de novo. isso fecha o ciclo
do dirty tracking: `resolve_dirty` liga o flag quando o hash muda ou a textura e
recriada, e `mark_layer_clean` desliga depois que a folha foi pintada. quem liga
e quem desliga sao funcoes diferentes em momentos diferentes do frame, e essa
separacao e o que deixa o resize, que liga o dirty dentro da `ensure_texture`, se
comportar igual a uma mudanca de cena qualquer.

e por fim o `encode_composite_pass`, a pilha sendo montada na superficie, um
triangulo de tela cheia por folha visivel, como ja vimos. submit, e o frame
foi.

um ultimo cuidado que vale citar porque e fácil de errar: a cor de fundo com que
o composite pass limpa a superficie e linearizada antes de ir pra GPU.

```rust
// wgpu clear values are linear; the sRGB surface re-encodes on write.
let bg = self.theme.colors.bg.to_linear_array();
```

os valores de clear do wgpu sao lineares, e a superficie sRGB reencoda na
escrita. se a cor de fundo do tema fosse passada crua em sRGB, ela apareceria
lavada, clara demais, do mesmo jeito que o cinza do shader apareceria sem o
`srgb_to_linear`. a superficie de saida e sempre acessada por
`gpu.surface_render_view`, que cria a view ja com o formato sRGB correto pra
forcar a reencodificacao gamma. e a mesma disciplina de cor do shader, agora no
nivel do frame inteiro: lineariza uma vez ao entrar na GPU, deixa a superficie
reencodar na saida.

## por que essa forma, e nao outra

da pra desenhar tela sem camada nenhuma. um buffer so, immediate mode, repinta
tudo todo frame, manda pra tela. funciona, e e simples. o custo aparece quando
qualquer coisa pequena muda. mexeu um pixel de um menu? repinta a tela inteira,
fundo, painel, texto, tudo. a GPU aguenta, mas e desperdicio puro, e em mobile
isso e bateria queimada.

a arquitetura de camadas troca um pouco de memoria por muito menos trabalho
repetido. cada folha guarda o seu desenho numa textura propria, e essas texturas
sao um cache de pixel entre frames. uma folha que nao muda nao e redesenhada, a
textura dela do frame passado e reusada direto no composite pass. o preco e os
~8mb por folha em 1080p que ficam ocupando VRAM. o ganho e que o trabalho de
desenho de cada frame escala com o que mudou, nao com o que esta na tela. essa e
a mesma logica do dirty tracking, vista pelo lado da memoria: a textura offscreen
e o que torna o "nao redesenhar" possivel, ela e o lugar onde o resultado de nao
redesenhar fica guardado.

a divisao em tres pecas, offscreen, premultiplied, composite, tambem nao e
acidente. cada uma resolve uma coisa e so ela. a textura offscreen resolve
isolamento: cada folha vive sozinha, sem saber das outras. o premultiplied alpha
resolve correcao: empilhar folhas semitransparentes da o resultado certo, sem
franja de borda, e o `over` e associavel entao a pilha pode ter qualquer altura.
o composite pass resolve custo de juncao: empilhar N folhas custa N draw calls,
independente do que ha dentro. tira qualquer uma das tres e o modelo quebra. sem
offscreen, nao tem o que cachear nem o que isolar. sem premultiplied, a juncao da
cor errada. sem o triangulo de tela cheia, a juncao seria cara de novo. as tres
juntas e que fazem o modelo de folhas de acetato sair do papel.

e tem a fronteira com o resto da engine, que e onde a coisa fica limpa de
manter. o app que usa o plev nunca toca em textura, em bind group, em blend
state. ele chama `draw_rect`, `create_layer`, `set_layer_opacity`. o `SceneNode`
e o contrato, e nada do que esta neste capitulo vaza pra fora dele. isso quer
dizer que dava pra trocar o triangulo de tela cheia por outra coisa, ou mudar o
formato da textura offscreen, ou ajustar o blend, sem que uma linha de codigo de
app precisasse mudar. a complexidade de GPU fica contida no compositor, e a
superficie que o app ve continua sendo "descreve o que voce quer ver". essa e a
mesma dualidade que o capitulo de dirty tracking aponta: API com cara de
immediate mode, comportamento de retained mode por baixo. a textura offscreen e
metade do retained, ela e onde o estado retido mora.

## o que isso me ensinou

a parte que demorou pra assentar na minha cabeca foi que o composite pass, a
coisa que faz a imagem final, e a mais simples do pipeline inteiro. um triangulo,
um sample, uma multiplicacao por opacidade, um draw por folha. toda a inteligencia
esta antes dele: em decidir quais folhas redesenhar (o dirty tracking), em manter
o resultado de cada folha numa textura propria (o offscreen), em garantir que a
cor entre certa pra empilhar (o premultiplied). quando essas tres coisas estao no
lugar, juntar tudo na tela e quase nada.

isso bate com uma coisa que eu venho aprendendo nesse projeto inteiro: a parte
visivel e cara de um sistema costuma ser a ponta de um trabalho que foi feito
antes, escondido. o frame que aparece no monitor e barato porque o offscreen
guardou o caro, o premultiplied deixou a juncao correta de graca, e o dirty
tracking decidiu o que nem precisava acontecer. a folha de acetato e uma ideia
velha, de retroprojetor, e ela continua certa: a melhor forma de nao redesenhar o
fundo quando o menu fecha e nunca ter desenhado os dois na mesma folha.

se eu fosse deixar uma frase disso pra alguem ler depois: camada nao e
organizacao, e onde o resultado mora pra poder nao ser refeito. o premultiplied e
o detalhe chato que faz empilhar dar certo, e o triangulo de tela cheia e a piada
de um vertex a menos que vira um draw call por folha. less, but better, de novo,
agora em pixel.

## rastros

arquitetura e adr
- `kdb/adr/layer-system.md:11-16` (premultiplied alpha: mudanca de
  `SrcAlpha/OneMinusSrcAlpha` para `One/OneMinusSrcAlpha`, shaders outputam
  `rgb * a, a`, necessario para o `over`, identico para cores opacas)
- `kdb/adr/layer-system.md:18-22` (cada layer com textura offscreen RGBA, hash
  proprio, buffers proprios, criadas lazily no resolve, recriadas no resize,
  default id=0 sempre existe, ordenadas por z_order)
- `kdb/adr/layer-system.md:24-29` (composite pass: full-screen triangle via
  vertex_index sem vb, 3 verts, bind group 0 textura+sampler, bind group 1
  opacity, um draw(0..3) por layer visivel)
- `kdb/adr/layer-system.md:36-40` (performance steady state: 1 draw call no
  composite pass, ~8mb por layer em 1920x1080)
- `kdb/caranguejovermelho/livro/cap-amostra-dirty-tracking.md` (dirty tracking
  per-layer, `compute_hash`/`resolve_dirty`, referenciado e nao reescrito aqui)

codigo: estrutura e ordenacao de layers
- `crates/engine/src/compositor/layer/mod.rs:15-20` (`LayerId`, `DEFAULT = LayerId(0)`)
- `crates/engine/src/compositor/layer/mod.rs:22-26` (`LayerEffect::Blur/Shadow`)
- `crates/engine/src/compositor/layer/mod.rs:44` (campo `dirty: bool`)
- `crates/engine/src/compositor/mod.rs:55-66` (`Compositor::new`, push da layer
  default)
- `crates/engine/src/compositor/mod.rs:68-72` (`begin_frame` limpa nos)
- `crates/engine/src/compositor/mod.rs:76` (`invalidate`)
- `crates/engine/src/compositor/mod.rs:106-138` (`resolve`: ensure_texture,
  resolve_scene, upload so pras dirty)
- `crates/engine/src/compositor/mod.rs:145-151` (`resolve_scene` ordena por
  z_order so quando `!sorted`)
- `crates/engine/src/compositor/layer_ops.rs:4-9` (`create_layer`, z_order,
  `sorted = false`)
- `crates/engine/src/compositor/layer_ops.rs:74-78` (`mark_layer_clean` desliga
  `dirty`)
- `crates/engine/src/compositor/drawing.rs:49-63` (`push` vai pra `DEFAULT`,
  `draw_rect`)

codigo: textura offscreen
- `crates/engine/src/compositor/layer/texture.rs:5-17` (`ensure_texture`, early
  return quando dimensoes batem, reescreve so a opacidade)
- `crates/engine/src/compositor/layer/texture.rs:27-42` (textura MSAA opcional,
  `RENDER_ATTACHMENT` apenas)
- `crates/engine/src/compositor/layer/texture.rs:44-54` (textura da layer,
  usage `RENDER_ATTACHMENT | TEXTURE_BINDING`)
- `crates/engine/src/compositor/layer/texture.rs:71-78` (opacity buffer, 4 bytes,
  uniform)
- `crates/engine/src/compositor/layer/texture.rs:98` (`self.dirty = true` apos
  recriar textura)
- `crates/engine/src/compositor/layer/mod.rs:307-313` (`render_attachment`, MSAA
  resolve target vs direto)

codigo: premultiplied alpha
- `crates/engine/src/gpu/utils.rs:3-16` (`premultiplied_blend`, src One, dst
  OneMinusSrcAlpha, Add)
- `crates/engine/src/gpu/shaders/quad.wgsl:29-40` (`srgb_to_linear` + `return
  vec4(rgb * in.color.a, in.color.a)`)
- `crates/engine/src/gpu/shaders/rect_sdf.wgsl:85-89` (lineariza, depois
  `return vec4(rgb * a, a)`)
- `crates/engine/src/gpu/shaders/text.wgsl:44-46`,
  `crates/engine/src/gpu/shaders/image.wgsl:69`,
  `crates/engine/src/gpu/shaders/shadow.wgsl:41-43` (mesmo padrao premultiplicado)

codigo: composite pass
- `crates/engine/src/gpu/shaders/composite.wgsl:18-27` (`vs_main`, full-screen
  triangle por `vertex_index`, UV com Y invertido)
- `crates/engine/src/gpu/shaders/composite.wgsl:30-34` (`fs_main`, sample + `*
  opacity`, cor ja premultiplicada)
- `crates/engine/src/gpu/shaders/composite.wgsl:4-10` (bind groups: g0 textura +
  sampler, g1 opacity uniform)
- `crates/engine/src/gpu/pipelines.rs:504-519` (`create_composite_pipeline`,
  `buffers: &[]`, blend premultiplicado, target = surface_format)
- `crates/engine/src/gpu/pipelines.rs:520-528` (`TriangleList`, `cull_mode: None`)
- `crates/engine/src/window/render_passes.rs:463-509` (`encode_composite_pass`,
  loop por layer visivel, `draw(0..3, 0..1)`, um por folha)

codigo: orquestracao do frame
- `crates/engine/src/window/render.rs:15` (`begin_frame`)
- `crates/engine/src/window/render.rs:74-85` (`resolve` com `ResolveResources`)
- `crates/engine/src/window/render.rs:135-141` (coleta `dirty_layer_ids`: visivel
  e dirty)
- `crates/engine/src/window/render.rs:145-156` (clear color linearizado,
  `to_linear_array`)
- `crates/engine/src/window/render.rs:158-167` (`encode_layer_passes`, fase
  offscreen)
- `crates/engine/src/window/render.rs:177-187` (`mark_layer_clean` antes do
  `encode_composite_pass`)
- `crates/engine/src/window/render_passes.rs:110-132` (`begin_layer_pass`)
- `crates/engine/src/window/render_passes.rs:268-284` (`encode_layer_passes`,
  clear da textura pra `(0,0,0,0)` transparente)
- `crates/engine/src/gpu/surface.rs:134-139` (`surface_render_view`, forca formato
  sRGB pra reencodar gamma)

versoes (conferidas contra o Cargo.toml)
- `Cargo.toml:50` wgpu 28
- `Cargo.toml:23` edition 2024
- `Cargo.toml:68` rustc-hash 2.1
- `Cargo.toml:70` web-time 1.1
- `Cargo.toml:51` winit 0.30, `Cargo.toml:54` cosmic-text 0.18, `Cargo.toml:55`
  taffy 0.9

nao confirmado
- nao ha benchmark isolado do custo em microssegundo do composite pass nem do
  premultiplied alpha. as afirmacoes de custo aqui sao estruturais (um draw por
  layer visivel, ~8mb por layer), tiradas do adr, nao de medicao independente.
- "~8mb por layer em 1920x1080" e afirmacao do `kdb/adr/layer-system.md:40`,
  conferida so por estimativa de 1920x1080x4 bytes, nao medida de VRAM real.
- a explicacao da franja de borda como motivo do premultiplied e o raciocinio
  padrao do tema, consistente com o codigo; o adr documenta a necessidade pro
  operador `over`, nao a franja em detalhe.
- comentarios reproduzidos nos blocos de codigo foram aparados quando continham
  travessao longo ou setas, pra manter o arquivo sem em dash; as linhas
  executaveis estao verbatim e compilam como no repo.
