---
title: "compositor e SceneNode: a arvore que vira frame"
parte: 2
status: rascunho
rastros:
  - crates/engine/src/compositor/mod.rs
  - crates/engine/src/compositor/scene.rs
  - crates/engine/src/compositor/vertex.rs
  - crates/engine/src/compositor/drawing.rs
  - crates/engine/src/compositor/clip.rs
  - crates/engine/src/compositor/sequence.rs
  - crates/engine/src/compositor/layer/geometry.rs
  - crates/engine/src/compositor/layer/mod.rs
  - crates/engine/src/window/render_passes.rs
  - kdb/adr/layer-system.md
  - kdb/adr/benchmark-results.md
---

# compositor e SceneNode: a arvore que vira frame

uma interface, olhada de cima, e uma lista de coisas pra desenhar. um retangulo
aqui, um texto ali, uma sombra embaixo daquele cartao, uma imagem com canto
arredondado no topo. voce descreve, a tela aparece. parece direto ate voce
lembrar que a GPU nao sabe o que e "cartao" nem o que e "texto". ela sabe
triangulo. ela sabe vertice com posicao e cor, sabe preencher o espaco entre
tres vertices, e e mais ou menos isso. todo o resto, o vocabulario inteiro de
interface, e traducao.

o capitulo passado fechou na pergunta "preciso refazer isso?", o dirty tracking,
que decide se um frame vale a pena. este aqui assume que vale e responde a
pergunta seguinte, que e mais concreta: dado que eu vou desenhar, como uma lista
de retangulo e texto vira triangulo de verdade, e quantas chamadas de desenho
isso custa. abre na lista que voce escreve e desce ate o `draw_indexed` que a
engine emite pra GPU, passando pelo recorte, pela montagem da geometria e pela
ordem de empilhamento. o dirty tracking fica de fora de proposito, ele e o
capitulo-amostra.

## o vocabulario: SceneNode

a unidade de desenho do plev e o `SceneNode`, um enum em
`crates/engine/src/compositor/scene.rs`. cada variante e uma coisa que da pra
desenhar. um `Rect`, um `RoundedRect`, um `GradientRect`, um `Text`, um `Path`,
uma `Image`, uma `Shadow`, um `BackdropBlur`, e mais dois nos que nao desenham
nada sozinhos, `PushClip` e `PopClip`, que volto neles na secao de recorte.

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum SceneNode {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
    },
    RoundedRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        corner_radius: f32,
        border_width: f32,
        border_color: [f32; 4],
    },
    // GradientRect, Text, Path, Image, PushClip, PopClip,
    // BackdropBlur, Shadow ...
}
```

repara que o no e dado puro. posicao, tamanho, cor. nenhum handle de GPU,
nenhuma referencia a buffer, nenhum ponteiro pra textura. um `Rect` e quatro
numeros e uma cor. isso e deliberado e e o primeiro fio que vale puxar: o
`SceneNode` e o unico contrato entre quem usa a engine e o compositor. um widget,
uma tela inteira, um app, tudo que eles produzem e uma lista desses nos. nada
mais atravessa essa fronteira. quem desenha nunca toca em vertice, pipeline ou
fila de comando da GPU.

a superficie que voce chama pra produzir esses nos mora em
`crates/engine/src/compositor/drawing.rs`, e ela parece imperativa, immediate
mode no jeitao:

```rust
impl Compositor {
    pub fn push(&mut self, node: SceneNode) {
        self.push_to_layer(LayerId::DEFAULT, node);
    }

    pub fn draw_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        self.push(SceneNode::Rect { x, y, w, h, color });
    }

    pub fn draw_rounded_rect(&mut self, p: RoundedRectParams) {
        self.push(SceneNode::RoundedRect {
            x: p.x,
            y: p.y,
            w: p.w,
            h: p.h,
            color: p.color,
            corner_radius: p.corner_radius,
            border_width: p.border_width,
            border_color: p.border_color,
        });
    }
}
```

`draw_rect` nao desenha nada. ele empurra um `SceneNode::Rect` pro fim de uma
lista. `push_to_layer` acha a layer certa e faz `layer.nodes.push(node)`, e
quando a layer pedida nao existe ele loga um aviso e segue, em vez de quebrar:

```rust
pub fn push_to_layer(&mut self, layer_id: LayerId, node: SceneNode) {
    if let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) {
        layer.nodes.push(node);
    } else {
        log::warn!("push_to_layer: layer {layer_id:?} not found");
    }
}
```

entao "desenhar" no plev e, na pratica, construir uma lista. quando o frame
comeca, `Compositor::begin_frame` chama `layer.begin_frame()` em cada layer, que
faz um `self.nodes.clear()`. a lista zera, voce descreve a cena toda de novo, do
zero, todo frame. esse e o modelo mental do consumidor: a cada frame voce diz
tudo que quer ver agora, sem gerenciar handle de objeto retido, sem dizer "mova
aquele retangulo que criei tres frames atras". voce so redescreve.

essa lista plana e a "arvore" do titulo, e vou ser honesto sobre o nome. o
`SceneNode` aqui nao e uma arvore de ponteiro filho-pai como um DOM. e um vetor
linear, em ordem de empilhamento. a hierarquia visual (cartao dentro de painel
dentro de tela) ja foi achatada pelo layout antes de chegar aqui, e o que sobra
e a sequencia de pincelada: desenhe isto, depois isto por cima, depois aquilo
por cima. a estrutura de arvore que importa pro compositor e a pilha de recorte,
que se abre e fecha com `PushClip`/`PopClip` e e a unica coisa com profundidade
real nessa lista. fora isso, e uma fila. uma fila que vira frame.

## uma volta so pela cena

o coracao deste capitulo e uma funcao: `Layer::build_geometry`, em
`crates/engine/src/compositor/layer/geometry.rs`. ela pega a lista de
`SceneNode` da layer e cospe vertice, indice e uma sequencia de comando de
desenho. e ela faz isso numa volta so pela lista.

esse "uma volta so" e a primeira decisao de arquitetura que nao e obvia. seria
mais simples, de codigo, percorrer a cena varias vezes: uma passada juntando
todos os retangulos, outra todas as sombras, outra todos os textos, cada tipo no
seu balde. dava menos `match`. o problema e que isso destroi a ordem de
empilhamento entre tipos diferentes. se voce desenha um icone de path por cima
de uma pilula de SDF que esta por cima de um cartao, e a engine agrupa por tipo,
ela vai desenhar todos os cartoes, depois todas as pilulas, depois todos os
icones, e a sobreposicao sai errada. o icone que devia tampar so a sua pilula
acaba tampando o cartao do vizinho.

entao a volta e uma so, e a ordem da lista vira a ordem do desenho:

```rust
pub(crate) fn build_geometry(&mut self, viewport: (f32, f32)) -> u32 {
    self.quad_vertices.clear();
    self.quad_indices.clear();
    self.quad_ranges.clear();
    // ... limpa sdf, shadow, image, backdrop, sequence, text_groups
    self.sequence.clear();
    self.text_groups.clear();

    let mut clips = ClipStack::default();
    let mut culled = 0u32;

    for node in &self.nodes {
        match node {
            SceneNode::PushClip { x, y, w, h } => clips.push([*x, *y, *w, *h]),
            SceneNode::PopClip => clips.pop(),

            SceneNode::Rect { x, y, w, h, color } => {
                // emite 4 vertices, 6 indices, registra o range e o comando
            }
            // ... uma arm por variante
        }
    }

    self.quad_index_count = self.quad_indices.len() as u32;
    // ... fecha os outros contadores
    culled
}
```

cada arm faz a mesma coreografia: testa se o no esta visivel, emite os vertices
no buffer do tipo dele, registra um intervalo de indices, e empurra um comando na
sequencia. o retorno e quantos nos foram descartados (`culled`), que vira
estatistica de frame. note que os buffers sao por tipo (`quad_vertices`,
`sdf_vertices`, `shadow_vertices`...) mas a `sequence` e unica e atravessa todos
eles. essa e a chave da coisa toda, e volto nela na secao da sequencia.

## de no a vertice: o caso do retangulo

vale ver o caso mais simples inteiro, porque ele estabelece o padrao que todos os
outros repetem. um retangulo:

```rust
SceneNode::Rect { x, y, w, h, color } => {
    if outside_viewport(viewport, *x, *y, *w, *h) || clips.is_empty_clip() {
        culled += 1;
        continue;
    }
    let first_index = self.quad_indices.len() as u32;
    let base = self.quad_vertices.len() as u32;
    self.quad_vertices.extend_from_slice(&[
        QuadVertex { position: [*x, *y], color: *color },
        QuadVertex { position: [x + w, *y], color: *color },
        QuadVertex { position: [x + w, y + h], color: *color },
        QuadVertex { position: [*x, y + h], color: *color },
    ]);
    self.quad_indices.extend_from_slice(&[
        base, base + 1, base + 2,
        base + 2, base + 3, base,
    ]);
    record_range(&mut self.quad_ranges, first_index, 6, clips.current());
    push_geometry(&mut self.sequence, DrawKind::Quad, first_index, 6, clips.current());
}
```

quatro vertices, um por canto, em ordem horaria a partir do topo-esquerda:
topo-esquerda, topo-direita, baixo-direita, baixo-esquerda. seis indices, que sao
dois triangulos. o primeiro triangulo usa os vertices `base, base+1, base+2`
(topo-esquerda, topo-direita, baixo-direita). o segundo usa
`base+2, base+3, base` (baixo-direita, baixo-esquerda, topo-esquerda). os dois
triangulos compartilham a diagonal entre topo-esquerda e baixo-direita, e juntos
cobrem o retangulo. e o jeito mais velho do mundo de desenhar um quad numa GPU, e
nao tem nada de errado em ser velho.

o `base` existe porque todos os retangulos da layer dividem o mesmo buffer de
vertice. o primeiro retangulo ocupa os indices 0 a 3, o segundo 4 a 7, e por ai
vai. `base` e onde os vertices deste retangulo comecam, entao os indices sao
sempre relativos a ele. `first_index` e a mesma ideia do lado dos indices: onde,
no buffer de indice, comeca este desenho. esses dois numeros sao o que permite
empacotar mil retangulos num par de buffers e ainda saber onde cada um esta.

o `QuadVertex` em si e magro, em `crates/engine/src/compositor/vertex.rs`:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}
```

duas coordenadas, quatro canais de cor. o `#[repr(C)]` garante que o layout em
memoria e previsivel, sem o rust reordenar campo. o `bytemuck::Pod` (plain old
data) e o que deixa a engine pegar um `&[QuadVertex]` e reinterpretar como
`&[u8]` cru pra mandar pra GPU sem copia campo a campo. o `layout()` que
acompanha a struct descreve pra wgpu onde cada atributo cai: `position` no offset
0, `color` no offset 8 (depois de dois `f32`), com os formatos `Float32x2` e
`Float32x4`. isso e o contrato entre o vertice em rust e o shader que vai ler ele
do outro lado.

## as familias de vertice, e por que sao varias

se todo desenho fosse retangulo de cor solida, um tipo de vertice bastava. mas
canto arredondado, sombra com blur, imagem do atlas e backdrop frosted pedem
coisas diferentes do shader, e cada um ganha sua propria struct de vertice e seu
proprio pipeline. sao cinco familias em `vertex.rs`: `QuadVertex`,
`RectSdfVertex`, `ShadowVertex`, `ImageVertex` e `BackdropVertex`.

a mais gorda e a `RectSdfVertex`, que carrega retangulo arredondado e gradiente:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectSdfVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub rect_params: [f32; 4],
    pub border_color: [f32; 4],
    /// Second gradient stop. Equal to `color` for solid fills.
    pub color2: [f32; 4],
    /// Linear-gradient brush: (dir_x, dir_y, enabled, unused).
    pub gradient: [f32; 4],
}
```

a sigla SDF e o ponto. signed distance field, campo de distancia com sinal. em
vez de tesselar o canto arredondado em dezenas de triangulinhos pra aproximar a
curva, o plev desenha um quad reto comum e deixa o shader calcular, pra cada
pixel, a distancia ate a borda do retangulo arredondado, descartando o pixel que
cai fora. o canto fica perfeito em qualquer escala porque e calculado, nao
aproximado. o `rect_params` carrega o que o shader precisa pra essa conta:
meia-largura, meia-altura, raio do canto e largura da borda. o `uv` vai de
`[-1, -1]` no canto topo-esquerda a `[1, 1]` no baixo-direita, dando ao shader a
coordenada local dentro do retangulo. o nome do pipeline confirma a leitura: em
`render_passes.rs`, o `DrawKind::SdfRect` liga o `gpu.rect_sdf_pipeline`.

a parte de gradiente reusa o mesmo vertice e o mesmo pipeline. um retangulo de
cor solida tem `color2 == color` e `gradient` com o flag de habilitado zerado.
um gradiente de verdade preenche `color2` com a segunda parada e `gradient` com a
direcao. a direcao vem de um helper:

```rust
pub fn gradient_direction(angle_deg: f32) -> [f32; 2] {
    let rad = angle_deg.to_radians();
    [rad.sin(), -rad.cos()]
}
```

o cosseno entra negado porque o espaco de tela tem o y pra baixo, ao contrario do
y pra cima da convencao matematica. um angulo de 0 grau aponta pra cima e poe a
primeira parada embaixo. ha uma divergencia nos comentarios sobre o que 90 graus
significa: o doc do enum em `scene.rs` diz "first stop at the left" e o do helper
em `vertex.rs` diz "90 points right". o vetor que a funcao devolve pra 90 graus e
`[1, 0]`, que aponta pra direita no espaco de tela. nao vou resolver a frase do
comentario aqui, so registro que os dois docs discordam e marco como nao
confirmado qual descricao verbal e a canonica.

a `ShadowVertex` desenha sombra sem passe de blur. o comentario do
`SceneNode::Shadow` chama de "Evan Wallace approximation", uma aproximacao
analitica da sombra gaussiana de um retangulo arredondado, calculada direto no
shader em vez de borrar uma textura. ela tem dois modos. drop shadow, a sombra
que cai pra fora e atras do retangulo, e inset shadow, a que cai pra dentro,
clipada na borda arredondada, o `box-shadow: inset` do CSS. dois helpers
amarram a matematica do blur:

```rust
pub fn shadow_sigma(blur_radius: f32) -> f32 {
    blur_radius / 2.0
}

pub fn shadow_padding(blur_radius: f32) -> f32 {
    3.0 * shadow_sigma(blur_radius)
}
```

o sigma e o raio de blur do CSS dividido por dois, a convencao de box-shadow. o
padding e tres sigmas, porque tres desvios padrao cobrem mais de 99,7% de uma
gaussiana, e e ai que a sombra "acaba" pra fins praticos. isso aparece na arm de
sombra do `build_geometry`: o drop shadow expande o quad pelo padding e o desloca
pelo offset, enquanto o inset mantem o quad do tamanho exato do retangulo e move
so a mascara dentro do shader, pelo `params2`. detalhe que so faz sentido quando
voce ja errou: o offset do drop ja vai assado na posicao do quad, mas o do inset
nao pode ir, senao o recorte duro contra a borda escorrega junto. por isso o
inset guarda o offset separado e aplica so na avaliacao da mascara borrada.

`ImageVertex` e `BackdropVertex` fecham o conjunto. a imagem sai do atlas de
textura compartilhado e carrega, alem da posicao, a coordenada de amostragem em
pixel do atlas e um retangulo de clamp pra que a filtragem linear nunca sangre o
vizinho do atlas. o backdrop e o vertice mais economico do grupo, ele so carrega
a mascara do retangulo arredondado, porque a textura borrada do fundo e amostrada
pela posicao no framebuffer la no shader, nao viaja por vertice. cada familia
existe porque um shader diferente le ela do outro lado, e cada shader diferente e
um pipeline diferente que a montagem final vai ter que ligar na hora certa.

## recorte: o corte que nao abre render pass

agora os dois nos que nao desenham nada. `PushClip` e `PopClip` definem uma
regiao retangular, e tudo que for empilhado entre eles fica restrito a essa
regiao. um painel que rola, uma lista com overflow, um cartao que corta o que
passa da borda, tudo isso e recorte.

o jeito ingenuo de implementar isso numa GPU e abrir um render pass novo por
regiao de recorte, ou usar stencil buffer. os dois custam. o plev faz por scissor
rect, o retangulo de tesoura que a GPU aplica de graca no rasterizador, e o
truque pra isso funcionar com recorte aninhado mora na `ClipStack`, em
`crates/engine/src/compositor/clip.rs`:

```rust
#[derive(Default)]
pub(crate) struct ClipStack {
    stack: Vec<ClipRect>,
}

impl ClipStack {
    pub(crate) fn push(&mut self, rect: ClipRect) {
        let combined = match self.current() {
            Some(cur) => intersect_rects(cur, rect),
            None => rect,
        };
        self.stack.push(combined);
    }

    pub(crate) fn current(&self) -> Option<ClipRect> {
        self.stack.last().copied()
    }

    pub(crate) fn is_empty_clip(&self) -> bool {
        matches!(self.current(), Some([_, _, w, h]) if w <= 0.0 || h <= 0.0)
    }
}
```

a sacada e que cada `push` ja guarda a intersecao com o recorte de cima, nao o
retangulo cru. entao o topo da pilha sempre tem o recorte efetivo pronto, e
`current()` e O(1), e so olhar o ultimo. recorte dentro de recorte e a intersecao
dos dois, calculada uma vez no push, nao toda hora que um no precisa saber onde
esta clipado. e se a intersecao deu vazia, largura ou altura menor ou igual a
zero, o `is_empty_clip` avisa, e qualquer no dentro dessa regiao morta e
descartado antes de virar um unico vertice. isso aparece na guarda de toda arm:
`outside_viewport(...) || clips.is_empty_clip()`.

o `pop` e tolerante de proposito. um `PopClip` sem o `PushClip` correspondente nao
quebra, ele loga e ignora, porque cena malformada nao deve derrubar o render:

```rust
pub(crate) fn pop(&mut self) {
    if self.stack.pop().is_none() {
        log::warn!("PopClip without matching PushClip -- ignored");
    }
}
```

cada desenho leva junto o recorte que estava ativo quando ele foi emitido, e e
ai que entra o `DrawRange`:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawRange {
    pub first_index: u32,
    pub index_count: u32,
    /// Intersection of the clip stack when these primitives were emitted.
    /// `None` = unclipped (full viewport scissor).
    pub clip: Option<ClipRect>,
}
```

um range e um pedaco contiguo do buffer de indice mais o recorte dele. `None`
quer dizer sem recorte, tesoura no viewport inteiro. e `record_range` faz uma
coisa esperta quando empilha esses ranges: se o desenho novo tem o mesmo recorte
do anterior e e contiguo no buffer, ele funde os dois num range so:

```rust
pub(crate) fn record_range(
    ranges: &mut Vec<DrawRange>,
    first_index: u32,
    index_count: u32,
    clip: Option<ClipRect>,
) {
    if index_count == 0 {
        return;
    }
    if let Some(last) = ranges.last_mut()
        && last.clip == clip
        && last.first_index + last.index_count == first_index
    {
        last.index_count += index_count;
        return;
    }
    ranges.push(DrawRange { first_index, index_count, clip });
}
```

essa fusao e o que faz mil retangulos sem recorte virarem um range so, de seis
mil indices. e isso, no fim, vira uma chamada de desenho so. guarde esse fato,
porque ele e o numero que importa quando a gente chegar nas draw calls.

a conversao final, de recorte logico pra scissor de pixel, fica em
`clip_to_scissor`, que arredonda pra fora, recorta no viewport e devolve `None`
quando a area visivel zerou (a hora de pular o desenho de vez). e tem
`intersect_scissors`, que cruza o scissor do range com o scissor proprio da
layer. recorte aninhado vira aritmetica de retangulo, e a GPU nunca precisa de um
render pass extra pra respeitar a borda de um painel.

## a sequencia: preservar a ordem de empilhamento

voltando ao fio que deixei solto la atras. os buffers de vertice sao separados por
tipo, mas a ordem de empilhamento atravessa os tipos. como o plev junta as duas
coisas? com uma segunda lista, a `sequence`, em
`crates/engine/src/compositor/sequence.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawKind {
    Quad,
    Shadow,
    SdfRect,
    Image,
    Text,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawCommand {
    Geometry { kind: DrawKind, range: DrawRange },
    BackdropBlur {
        first_index: u32,
        sigma: f32,
        clip: Option<ClipRect>,
    },
}
```

enquanto os `quad_ranges`, `sdf_ranges` e companhia agrupam tudo de um pipeline
junto, a `sequence` guarda a ordem real de empilhamento entre pipelines. cada
comando diz "desenhe este intervalo destes buffers" e os comandos estao na mesma
ordem em que a cena empilhou. e isso que deixa a montagem final intercalar um
quad, depois uma sombra, depois um SDF, depois um texto, depois outro quad,
exatamente como o autor da cena pediu. o comentario do modulo poe nessas
palavras: trocar de pipeline dentro de um render pass de UI e barato, e comandos
consecutivos do mesmo tipo e recorte sao fundidos pra que cena em regime estavel
ainda emita poucos desenhos.

a fusao acontece em `push_geometry`, que e o irmao do `record_range` do lado da
sequencia:

```rust
pub(crate) fn push_geometry(
    sequence: &mut Vec<DrawCommand>,
    kind: DrawKind,
    first_index: u32,
    index_count: u32,
    clip: Option<ClipRect>,
) {
    if index_count == 0 && kind != DrawKind::Text {
        return;
    }
    if kind != DrawKind::Text
        && let Some(DrawCommand::Geometry { kind: last_kind, range }) = sequence.last_mut()
        && *last_kind == kind
        && range.clip == clip
        && range.first_index + range.index_count == first_index
    {
        range.index_count += index_count;
        return;
    }
    sequence.push(DrawCommand::Geometry {
        kind,
        range: DrawRange { first_index, index_count, clip },
    });
}
```

mesmo tipo, mesmo recorte, indices contiguos: funde. tipo diferente ou recorte
diferente: comando novo. e ali esta a economia de novo. dez retangulos seguidos,
sem recorte, viram um comando de sessenta indices. cinco textos seguidos no mesmo
recorte viram... nao, texto e a excecao, e a excecao tem motivo.

texto nunca funde. os ranges de texto sao placeholder, zerados, porque na hora do
`build_geometry` os glifos ainda nao foram resolvidos. o shaping (transformar a
string em posicao de glifo) acontece depois, no sistema de texto, e so ai as
posicoes reais aparecem. entao a montagem da geometria so reserva um comando
`Text` vazio por grupo de texto e agrupa os nos de texto em `text_groups`:

```rust
SceneNode::Text { .. } => {
    let clip = clips.current();
    let joins_last = matches!(
        self.sequence.last(),
        Some(DrawCommand::Geometry { kind: DrawKind::Text, range }) if range.clip == clip
    );
    if joins_last {
        if let Some((nodes, _)) = self.text_groups.last_mut() {
            nodes.push(node.clone());
        }
    } else {
        self.text_groups.push((vec![node.clone()], clip));
        push_geometry(&mut self.sequence, DrawKind::Text, 0, 0, clip);
    }
}
```

a logica de agrupamento e o que mantem a ordem honesta. um texto junta no grupo
anterior so quando nada foi desenhado no meio, ou seja, quando o ultimo comando
da sequencia ja e um `Text` com o mesmo recorte. se um retangulo foi empilhado
entre dois textos, o segundo texto comeca um grupo novo e um comando novo, pra
que o retangulo desenhe de verdade entre os dois. depois que o sistema de texto
resolve os glifos, `assign_text_ranges` volta na sequencia e preenche os
placeholders, um pra um, em ordem de pintura: o range `i` pertence ao grupo de
texto `i`. se a contabilidade nao bater, ele loga, e degrada zerando o comando em
vez de corromper a sequencia inteira.

## culling: o que esta fora nao vira vertice

a guarda no comeco de cada arm faz uma coisa simples e barata. antes de emitir
qualquer vertice, a engine pergunta se o no esta visivel. duas condicoes
descartam: estar inteiro fora do viewport, ou estar dentro de um recorte vazio.

```rust
fn outside_viewport(viewport: (f32, f32), x: f32, y: f32, w: f32, h: f32) -> bool {
    x + w <= 0.0 || y + h <= 0.0 || x >= viewport.0 || y >= viewport.1
}
```

quatro comparacoes. se a caixa do no esta toda a esquerda, toda em cima, toda a
direita ou toda embaixo da tela, ela nao vira geometria, e o contador `culled`
sobe. pro path, que nao tem caixa explicita, a engine calcula o bounding box dos
vertices tesselados antes de testar. cada no descartado e um que nao gasta
vertice, nao gasta indice, nao gasta upload e nao gasta tempo de shader. o numero
de nos cortados vira `nodes_culled` no `RenderStats`, ao lado de `layers_redrawn`
e dos contadores de vertice por tipo, o painel de instrumentacao do frame.

esse culling e grosso de proposito. ele e por caixa alinhada aos eixos, sem
rotacao, sem teste pixel a pixel. a meta nao e descartar com precisao cirurgica,
e descartar rapido o caso comum: a lista de cem itens onde noventa estao fora da
janela rolada. esses noventa somem por quatro comparacoes cada, antes de tocar a
GPU.

## a montagem final: vertice vira draw call

ate aqui tudo foi CPU. a lista de no virou vertice em buffer e uma sequencia de
comando. falta a parte que de fato fala com a GPU, e ela mora em
`crates/engine/src/window/render_passes.rs`. o laco que importa percorre a
`sequence` da layer e, pra cada comando, garante o pipeline certo, poe o scissor e
emite o desenho:

```rust
let mut bound: Option<DrawKind> = None;
for cmd in layer.sequence() {
    match *cmd {
        DrawCommand::Geometry { kind, range } => {
            if range.index_count == 0 {
                continue;
            }
            let Some((sx, sy, sw, sh)) =
                command_scissor(range.clip, base_scissor, clip_scale, vw, vh)
            else {
                continue;
            };
            if bound != Some(kind) {
                if !bind_draw_kind(&mut pass, kind, layer, gpu, text_system) {
                    bound = None;
                    continue;
                }
                bound = Some(kind);
            }
            pass.set_scissor_rect(sx, sy, sw, sh);
            pass.draw_indexed(
                range.first_index..range.first_index + range.index_count,
                0,
                0..1,
            );
            draw_calls += 1;
        }
        DrawCommand::BackdropBlur { .. } => { /* suspende o pass, borra, retoma */ }
    }
}
```

tres coisas acontecem por comando. primeiro, o recorte do range vira scissor de
pixel via `command_scissor`, que escala o recorte logico pra pixel fisico, recorta
no viewport e cruza com o scissor proprio da layer. se a area zerou, o comando e
pulado. segundo, `bound` lembra qual pipeline esta ligado no pass agora, e
`bind_draw_kind` so e chamado quando o tipo do comando muda. e ali que a fusao da
sequencia paga: dez quads fundidos num comando so significam um bind e um
`draw_indexed`, nao dez. terceiro, `set_scissor_rect` e `draw_indexed` emitem o
desenho de verdade, e `draw_calls` sobe um.

o `bind_draw_kind` e o mapa de tipo pra pipeline. ele escolhe os buffers e o
pipeline conforme o `DrawKind`:

```rust
match kind {
    DrawKind::Quad => pass.set_pipeline(&gpu.quad_pipeline),
    DrawKind::Shadow => pass.set_pipeline(&gpu.shadow_analytic_pipeline),
    DrawKind::SdfRect => pass.set_pipeline(&gpu.rect_sdf_pipeline),
    DrawKind::Image => {
        let Some(image_bg) = gpu.image_atlas.bind_group() else {
            return false;
        };
        pass.set_pipeline(&gpu.image_pipeline);
        pass.set_bind_group(1, image_bg, &[]);
    }
    DrawKind::Text => {
        pass.set_pipeline(&gpu.text_pipeline);
        pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
    }
}
```

cada familia de vertice que vimos la atras reaparece aqui como um pipeline
nomeado: `quad_pipeline`, `rect_sdf_pipeline`, `shadow_analytic_pipeline`,
`image_pipeline`, `text_pipeline`. o vertice e o pipeline sao as duas pontas do
mesmo contrato, e o `bind_draw_kind` e onde as pontas se encontram em tempo de
execucao.

esse laco roda dentro do render pass de uma layer e desenha na textura offscreen
dela, nao na tela. cada layer pinta a sua propria textura. quando todas as layers
terminaram a geometria, vem o segundo ato, a composicao, em
`encode_composite_pass`. e aqui a coisa fica quase ridicula de simples:

```rust
pass.set_pipeline(&gpu.composite_pipeline);

let mut draw_calls = 0u32;
for layer in compositor.layers() {
    if !layer.visible {
        continue;
    }
    // acha o bind group da textura da layer + o uniform de opacity
    if let (Some(bg), Some(opacity_bg)) = (final_bg, layer.opacity_bind_group()) {
        pass.set_bind_group(0, bg, &[]);
        pass.set_bind_group(1, opacity_bg, &[]);
        pass.draw(0..3, 0..1);
        draw_calls += 1;
    }
}
```

um `draw(0..3)` por layer visivel. tres vertices, sem buffer de vertice nenhum. e
o triangulo de tela cheia, um triangulo grande o suficiente pra cobrir a tela
inteira, gerado dentro do shader a partir do `vertex_index` (0, 1, 2), que estampa
a textura da layer no framebuffer final com a opacity dela. o ADR do sistema de
layers descreve exatamente isso: full-screen triangle via vertex_index, sem
vertex buffer, tres verts, um draw por layer visivel. a arvore inteira, mil
retangulos, textos, sombras, colapsa nisso: uma textura por layer, e um triangulo
por layer pra carimbar a textura na tela.

vale uma nota sobre a cor que esse carimbo usa, porque ela tem um detalhe que o
ADR registra. a composicao roda com premultiplied alpha, o blend
`One/OneMinusSrcAlpha` em vez do `SrcAlpha/OneMinusSrcAlpha` mais comum, e os
shaders escrevem `rgb * a, a`. isso e necessario pro operador `over` compor as
layers corretamente, empilhar uma layer translucida sobre outra sem escurecer a
borda. pra cor opaca o resultado e visualmente identico, entao e o tipo de
decisao que voce so percebe quando a borda de um menu translucido fica com uma
auria suja, e ai descobre que o blend mode estava errado.

## o numero, e o que ele de fato mede

eu nao gosto de afirmar performance sem numero atras, e tambem nao gosto de
pendurar um numero na afirmacao errada. os benchmarks vivem em
`crates/engine/benches/scene_build.rs`, medidos num macbook pro m4, rust 1.94.0,
criterion 0.5, registrados em `kdb/adr/benchmark-results.md`.

| benchmark | tempo | throughput |
|-----------|-------|------------|
| push_rects/100 | 629 ns | 159m rects/s |
| push_rects/1000 | 5.48 us | 183m rects/s |
| push_rects/10000 | 45.0 us | 222m rects/s |
| push_paths/circles_1000 | 106 us | 9.4m paths/s |

cinco microssegundos e meio pra empurrar mil retangulos. de 159 a 222 milhoes de
retangulos por segundo do lado da CPU. o takeaway do doc fecha assim: a construcao
da cena e desprezivel perto do trabalho de GPU. e e verdade, com uma ressalva que
eu faco questao de marcar, porque o livro inteiro se sustenta em ler a fonte, nao
a legenda.

o `push_rects` mede `comp.draw_rect` num laco. ou seja, ele mede a construcao da
lista de `SceneNode`, o `push` que vimos no comeco do capitulo. ele nao chama
`build_geometry`. o codigo do bench e literalmente um laco de `draw_rect` cercado
por `black_box`, sem `resolve_scene` no meio. entao esses 5,48us sao o custo de
montar o vetor de nos, nao o custo de transformar esse vetor em vertice, indice e
sequencia, que e o assunto deste capitulo. a parte de virar geometria, o
`build_geometry`, nao tem benchmark isolado na suite. eu nao tenho um numero
medido pra ela, e nao vou inventar um. marco isso explicitamente como nao
confirmado: o custo de `build_geometry` em si nao esta separado no bench atual.

o que da pra dizer com a fonte na mao e mais modesto e ainda util. descrever a
cena (o push) e quase de graca, dezenas de milhoes de retangulos por segundo.
materializar a cena (o build) faz trabalho proporcional ao numero de nos visiveis,
quatro vertices e seis indices por retangulo, mais a fusao de range e a fusao de
sequencia, que sao O(1) por no porque so olham o ultimo. e o resultado, depois da
fusao, e que mil retangulos contiguos sem recorte saem como um range, um comando e
uma draw call. o caminho de path conta a historia oposta e o doc registra: empurrar
path custa cerca de dez vezes mais que retangulo (106us pra mil circulos contra
5,48us pra mil rects), porque o path clona buffers de vertice ja tesselados. por
isso a tesselacao e do lado do usuario, feita uma vez, e nao por frame. o
`circle(r=50)` tesselado custa 3,70us de uma vez, nao a cada frame.

## por que assim, e nao de outro jeito

dois fios amarram a arquitetura desta parte, e vale puxar os dois.

o primeiro e a dualidade da API. por fora, o compositor parece immediate mode.
voce chama `draw_rect`, `draw_text`, `draw_shadow` todo frame, redescreve a cena
do zero, nao gerencia handle de objeto. e o jeito mais simples de raciocinar
sobre UI: desenho tudo que quero ver agora. por dentro, o `SceneNode` e a unica
coisa que cruza a fronteira, e o que acontece com ele (virar vertice, virar
textura, ser comparado por hash, ser pulado quando nada mudou) e problema do
compositor, nao seu. essa separacao e o que deixa a engine trocar todo o
maquinario de materializacao, mudar o layout de vertice, mexer na fusao de range,
sem que um widget perceba. o contrato e o no, nao o vertice.

o segundo e a escolha de fazer o trabalho no shader em vez de na geometria. canto
arredondado via SDF, sombra analitica via aproximacao de Wallace, recorte via
scissor. os tres tem a mesma forma de pensamento: emitir o minimo de geometria,
um quad reto, e empurrar a complexidade pro shader, que roda em paralelo massivo
e e barato por pixel. a alternativa, tesselar cada canto arredondado em
triangulinhos, gera muito mais vertice, escala mal com o raio e fica serrilhada
quando voce dá zoom. o caminho do SDF gera quatro vertices e um canto perfeito em
qualquer escala. e o recorte por scissor evita o custo mais caro de todos, abrir
um render pass por regiao clipada, trocando ele por aritmetica de retangulo na
pilha de clip.

tem trade-off, e seria desonesto esconder. fazer o trabalho no shader significa
que cada pixel do quad paga o calculo do SDF, mesmo o pixel bem no meio do
retangulo que obviamente esta dentro. pra retangulo grande e cheio, isso e
overdraw de computacao que a tesselacao nao teria. a aposta do plev e que GPU
moderna engole essa conta com folga e que a economia de geometria e de draw call
compensa, ainda mais com a fusao que transforma cena estavel em poucos desenhos.
e uma aposta de olho aberto, nao um descuido.

e fica a fronteira limpa que o capitulo todo desenhou, do jeito que eu gosto de
ver fronteira. o app produz `SceneNode`. o `build_geometry` faz uma volta so e
produz vertice, range e sequencia, respeitando ordem de empilhamento e recorte. o
render pass por layer percorre a sequencia, liga pipeline conforme o tipo, poe o
scissor e emite `draw_indexed`. o composite pass estampa cada textura de layer com
um triangulo de tela cheia. quatro estagios, cada um com uma responsabilidade, e
o `SceneNode` atravessando o primeiro sem saber que os outros tres existem.

## o que isso me ensinou

a parte que demorei pra internalizar nao foi o SDF nem o blend premultiplicado.
foi a `sequence`. minha primeira intuicao foi agrupar tudo por pipeline, porque
parecia obvio que juntar todos os retangulos num desenho e todas as sombras em
outro seria mais rapido. e seria, pra um draw call. mas estaria errado pra
imagem, porque a ordem de empilhamento e parte da cena, nao um detalhe de
performance que da pra otimizar a revelia. a sequencia preserva a ordem e ainda
funde os vizinhos compativeis, e e exatamente o equilibrio que eu nao tinha
pensado: respeite a intencao primeiro, agrupe o que sobrar depois.

se eu fosse deixar uma frase disso pra aurora ler um dia: o trabalho de uma
engine de UI nao e desenhar, e traduzir. ela recebe um vocabulario humano,
retangulo, texto, sombra, canto, e o devolve no unico idioma que a GPU fala,
triangulo com posicao e cor. o `SceneNode` e a palavra de entrada, o
`draw_indexed` e a de saida, e os quatro estagios no meio sao o dicionario. o
resto, o `to_bits`, o hash, a decisao de nem traduzir quando nada mudou, e o
capitulo-amostra, e e so o encanamento que torna essa traducao barata o
bastante pra rodar sessenta vezes por segundo.

## rastros

contrato e API de desenho
- `crates/engine/src/compositor/scene.rs:6-108` (enum `SceneNode`, as variantes
  Rect, RoundedRect, GradientRect, Text, Path, Image, PushClip, PopClip,
  BackdropBlur, Shadow)
- `crates/engine/src/compositor/scene.rs:96-107` (`Shadow` com `inset`, comentario
  "Evan Wallace approximation", drop vs inset, linhas 87-95)
- `crates/engine/src/compositor/drawing.rs:49-59` (`push`, `push_to_layer` com o
  warn de layer ausente)
- `crates/engine/src/compositor/drawing.rs:61-94` (`draw_rect`, `draw_rounded_rect`)
- `crates/engine/src/compositor/mod.rs:14-25` (`pub use` exportando SceneNode,
  DrawCommand, DrawKind, vertices, clip helpers)
- `crates/engine/src/compositor/mod.rs:68-72` (`begin_frame`) e
  `crates/engine/src/compositor/layer/mod.rs:421-423` (`Layer::begin_frame`, o
  `nodes.clear()`)

montagem da geometria (uma volta so)
- `crates/engine/src/compositor/layer/geometry.rs:106-486` (`build_geometry`, a
  volta unica pela cena com `match` por variante)
- `crates/engine/src/compositor/layer/geometry.rs:132-173` (arm `Rect`: 4 vertices,
  6 indices, padrao `base..base+3`, `record_range` + `push_geometry`)
- `crates/engine/src/compositor/layer/geometry.rs:26-28` (`outside_viewport`, 4
  comparacoes) e `:77-88` (`path_bounds` pro culling de path)
- `crates/engine/src/compositor/layer/geometry.rs:275-352` (arm `Shadow`: drop
  expande pelo padding e desloca pelo offset, inset mantem o quad e move a mascara
  via `params2`)
- `crates/engine/src/compositor/layer/geometry.rs:454-476` (arm `Text`: agrupamento
  em `text_groups`, comando placeholder, `joins_last`)
- `crates/engine/src/compositor/layer/mod.rs:390-419` (`assign_text_ranges`,
  patch dos placeholders 1:1 em ordem de pintura)

familias de vertice e pipelines
- `crates/engine/src/compositor/vertex.rs:1-27` (`QuadVertex`, `#[repr(C)]`,
  `bytemuck::Pod`, `layout()` com offsets 0 e 8)
- `crates/engine/src/compositor/vertex.rs:29-88` (`RectSdfVertex`, `rect_params`,
  `gradient`, `color2`)
- `crates/engine/src/compositor/vertex.rs:90-142` (`ShadowVertex`, `params`/`params2`)
- `crates/engine/src/compositor/vertex.rs:144-232` (`ImageVertex` com `uv_bounds`
  de clamp, `BackdropVertex` amostrado por posicao de framebuffer)
- `crates/engine/src/compositor/vertex.rs:235-243` (`shadow_sigma` = blur/2,
  `shadow_padding` = 3*sigma)
- `crates/engine/src/compositor/vertex.rs:245-251` (`gradient_direction`,
  `[sin, -cos]`, y pra baixo)

recorte
- `crates/engine/src/compositor/clip.rs:60-90` (`ClipStack`, `push` guarda a
  intersecao, `current` O(1), `is_empty_clip`)
- `crates/engine/src/compositor/clip.rs:75-79` (`pop` tolerante, loga e ignora)
- `crates/engine/src/compositor/clip.rs:10-17` (`DrawRange` com `clip: Option`)
- `crates/engine/src/compositor/clip.rs:94-115` (`record_range`, fusao de range
  contiguo com mesmo clip)
- `crates/engine/src/compositor/clip.rs:21-40` (`intersect_rects`,
  `clip_to_scissor`) e `:43-55` (`intersect_scissors`)

sequencia (ordem de empilhamento)
- `crates/engine/src/compositor/sequence.rs:1-50` (doc do modulo, `DrawKind`,
  `DrawCommand` com `Geometry` e `BackdropBlur`)
- `crates/engine/src/compositor/sequence.rs:56-86` (`push_geometry`, fusao por
  tipo+clip+contiguidade, texto nunca funde)

montagem final (vertice vira draw call)
- `crates/engine/src/window/render_passes.rs:294-318` (laco sobre `sequence()`,
  `command_scissor`, `bind_draw_kind` so quando muda `bound`, `set_scissor_rect`,
  `draw_indexed`, `draw_calls += 1`)
- `crates/engine/src/window/render_passes.rs:70-107` (`bind_draw_kind`, mapa
  DrawKind -> pipeline: quad, shadow_analytic, rect_sdf, image, text)
- `crates/engine/src/window/render_passes.rs:9-28` (`command_scissor`, escala
  logico->fisico, recorta no viewport, cruza com o scissor da layer)
- `crates/engine/src/window/render_passes.rs:463-511` (`encode_composite_pass`,
  `draw(0..3)` por layer visivel, triangulo de tela cheia)

ADR
- `kdb/adr/layer-system.md:23-28` (composite pass: full-screen triangle via
  vertex_index, sem vb, 3 verts, um draw por layer visivel)
- `kdb/adr/layer-system.md:11-15` (premultiplied alpha, `One/OneMinusSrcAlpha`,
  shaders escrevem `rgb*a, a`, necessario pro operador `over`)
- `kdb/adr/layer-system.md:17-22` (arquitetura de layers: textura offscreen por
  layer, buffers proprios) e `:41-43` (gpuvec compartilhado)

benchmark
- `kdb/adr/benchmark-results.md:11-18` (push_rects/100 629ns, /1000 5.48us,
  /10000 45.0us; 159m-222m rects/s) e `:19` (push_paths/circles_1000 106us)
- `kdb/adr/benchmark-results.md:30-34` (lyon: circle r50 3.70us, tesselacao de
  uma vez)
- `kdb/adr/benchmark-results.md:42-47` (takeaways: construcao de cena desprezivel
  vs GPU; path push ~10x mais que rect, pre-tesselado e o design certo)
- `crates/engine/benches/scene_build.rs:10-25` (`bench_push_rects`, laco de
  `draw_rect` sem `resolve_scene`)

nao confirmado
- o `push_rects` (`scene_build.rs:10-25`) mede a construcao da lista de
  `SceneNode` (o `push`), nao `build_geometry`. os 5.48us pra mil rects sao o
  custo de montar o vetor de nos, nao o de virar vertice/indice/sequencia. nao ha
  benchmark isolado pra `build_geometry` na suite atual, entao nao afirmo um numero
  medido pra a materializacao da geometria.
- os comentarios de `gradient_direction` discordam sobre 90 graus: `scene.rs:36`
  diz "first stop at the left" e `vertex.rs:247` diz "90 points right". o vetor
  retornado pra 90 graus e `[1, 0]` (aponta pra direita no espaco de tela); qual
  descricao verbal e a canonica fica em aberto.
- a leitura "o shader calcula o SDF do canto" se apoia no nome do vertice
  (`RectSdfVertex`), no `rect_sdf_pipeline` (`render_passes.rs:90`) e nos campos
  `rect_params`; os arquivos `.wgsl` dos shaders nao foram lidos neste capitulo,
  entao a matematica exata do shader nao esta verificada aqui.
- "tres sigmas cobrem >99,7% da gaussiana" e a justificativa do
  `shadow_padding`; a constante `3.0` esta no codigo (`vertex.rs:241-243`), o
  percentual e a regra estatistica conhecida, nao uma medida do repo.
