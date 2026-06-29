---
title: GpuVec e buffers persistentes
parte: 2
status: rascunho
rastros:
  - crates/engine/src/gpu/vec.rs
  - crates/engine/src/gpu/texture_pool.rs
  - crates/engine/src/compositor/layer/mod.rs
  - crates/engine/src/compositor/layer/geometry.rs
  - crates/engine/src/compositor/memory.rs
  - crates/engine/src/window/render_passes.rs
  - crates/engine/src/window/render.rs
  - kdb/adr/render-on-demand-requires-explicit-invalidation.md
  - crates/engine/benches/scene_build.rs
---

# GpuVec e buffers persistentes

cozinha de restaurante numa sexta a noite. o pedido entra, sai um prato, entra
outro. o cozinheiro de linha tem uma frigideira na mao e uma fila de comandas na
frente. ninguem que ja trabalhou numa cozinha cheia pega uma frigideira nova a
cada prato, usa, joga no lixo e abre o armario pra pegar outra. voce reusa a
mesma panela, no maximo passa um pano, e se chega um pedido grande demais pra ela
voce sobe pra uma maior e fica com a maior. a frigideira nao volta pro armario no
meio do servico. ela fica na boca do fogao, quente, pronta pro proximo.

quase todo software grafico cozinha do jeito errado. todo frame ele aloca buffer
novo, despeja os vertices, manda pra placa, descarta, e no frame seguinte repete
o ritual inteiro. funciona. a GPU moderna engole esse desperdicio sem reclamar
alto. mas tem dois custos que nao aparecem no comeco e que cobram juros quando a
cena fica pesada: alocar e liberar memoria de GPU repetidamente pressiona o
alocador da placa, e o trafego CPU para GPU enche um barramento que voce vai
querer livre justo na hora em que a tela tem mais coisa pra desenhar.

o plev cozinha como o cozinheiro de linha. as duas pecas deste capitulo, o
`GpuVec` e a texture pool, sao a frigideira que nao volta pro armario. uma guarda
bytes lineares, geometria, o vertice e o indice de cada layer. a outra guarda
textura 2D, os alvos intermediarios que os efeitos precisam pra borrar e somar.
formas diferentes de memoria, mesma regra de cozinha: cresce, nunca encolhe,
reusa entre um prato e o outro, e em regime estavel nao abre o armario nenhuma
vez. este capitulo abre nessa imagem e desce ate as linhas onde a regra vira
codigo, ate o `(self.capacity * 2).max(needed)`, ate o `in_use` que decide
reusar ou criar, e ate a fronteira onde o benchmark de CPU mede o que mede e nao
mede o que nao da pra medir sem placa.

vale uma nota de rota antes. a parte 1 ja abriu o `GpuVec` pela porta do
ownership: quem possui o buffer, quem empresta pra desenhar, por que o borrow
checker impede um use after free de VRAM. aqui eu entro pela outra porta, a da
estrategia de memoria. nao "quem pode escrever no buffer", mas "quando o buffer
cresce, quando ele e reusado, e quanto de placa ele prende fazendo isso". o
ownership e a mecanica. o reuso e a politica. as duas pecas guardam a mesma
memoria por angulos diferentes.

## duas pecas, uma familia

antes de descer no codigo, vale fixar a regra que as duas obedecem, porque ela e
identica e e o que torna o capitulo um capitulo so e nao dois.

a regra tem tres partes. cresce sob demanda: quando o que voce precisa nao cabe
no que ja existe, aloca mais. nunca encolhe: quando a demanda cai, a memoria fica
do tamanho do pico. reusa entre frames: a peca sobrevive de um frame pro outro e
o trabalho do proximo frame e reescrever por cima do que ja esta alocado, nao
alocar de novo. some as tres e voce tem a propriedade que o codigo persegue:
estado estavel sem alocacao. uma tela parada, ou uma animacao que repinta a mesma
quantidade de coisa, nao toca o alocador da placa depois que aqueceu.

o `GpuVec` aplica isso a um buffer contiguo de bytes. ele e o `Vec` da std com a
memoria morando na placa em vez da heap: dobra de tamanho quando estoura, e o
amortizado e o mesmo. a texture pool aplica a mesma regra a textura, que nao e um
array linear, e uma grade 2D com largura, altura e formato. voce nao pode "dobrar"
uma textura do mesmo jeito que dobra um buffer, entao a pool faz a versao 2D da
ideia: guarda um conjunto de texturas indexadas pelo tamanho e pelo formato, e
reusa a que tem a forma certa em vez de criar uma nova. quando nenhuma serve, ai
sim cria, e guarda pra proxima.

repara que as duas resolvem o mesmo problema com a mesma filosofia porque o
gargalo e o mesmo: criar e destruir recurso de GPU e caro, e fazer isso todo
frame e o tipo de custo que nao machuca no protótipo e arruina o app cheio. a
escolha de design foi pagar memoria parada pra nao pagar alocacao recorrente. e
uma aposta, e como toda aposta de engenharia ela tem um preco que o codigo
assume de olho aberto e mede. vou chegar no preco.

## o GpuVec, o buffer que cresce e nunca encolhe

o `GpuVec` mora em `crates/engine/src/gpu/vec.rs` e o struct cabe na palma da
mao:

```rust
pub struct GpuVec {
    buffer: wgpu::Buffer,
    capacity: u64,
    usage: wgpu::BufferUsages,
    label: &'static str,
}
```

o `buffer` e o handle dono da memoria de placa. `capacity` e quantos bytes a
placa reservou. `usage` guarda pra que serve o buffer pra poder recriar com a
mesma intencao quando ele crescer. `label` e o nome de debug que aparece no
profiler de GPU, um `&'static str` que aponta pra dentro do binario e nao custa
heap nenhuma. a parte 1 desmontou esses campos pelo lado da posse. aqui o campo
que interessa e o `capacity`, porque ele e a memoria da estrategia de
crescimento.

o coracao da estrategia e o `ensure_capacity`:

```rust
pub fn ensure_capacity(&mut self, device: &wgpu::Device, needed: u64) {
    if needed <= self.capacity {
        return;
    }
    let new_cap = (self.capacity * 2).max(needed);
    self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(self.label),
        size: new_cap,
        usage: self.usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    self.capacity = new_cap;
    log::debug!("{} grew to {} bytes", self.label, new_cap);
}
```

a logica e a de qualquer vetor que dobra. se o que voce precisa cabe na
capacidade atual, sai na primeira linha sem fazer nada. esse `if needed <=
self.capacity { return; }` e o caminho quente, o que roda na imensa maioria dos
frames depois que o app aquece. ele e o "a frigideira ja serve, continua
cozinhando". so quando o pedido estoura a panela e que a linha do meio dispara:
`new_cap` vira o dobro da capacidade atual, ou o exato que voce pediu, o que for
maior. e um buffer novo desse tamanho substitui o antigo.

o `(self.capacity * 2).max(needed)` merece um paragrafo, porque a escolha de
dobrar nao e estetica, e a unica que da o amortizado certo. se em vez de dobrar
voce crescesse de pouquinho, digamos somando um tamanho fixo a cada estouro, voce
realocaria a cada poucos inserts e o custo total de encher o buffer viraria
quadratico no numero de itens. dobrando, voce realoca em pontos cada vez mais
espacados: 4k, 8k, 16k, 32k. o numero de realocacoes pra chegar em N bytes e
logaritmico, e o custo somado de todas elas e linear em N, o que diluido por
item da tempo constante amortizado. e exatamente o mesmo raciocinio do `Vec` da
biblioteca padrao do rust, e nao por acaso: o `GpuVec` e o `Vec` com a memoria
do outro lado do barramento. o `.max(needed)` cobre o caso em que dobrar ainda
nao basta, um salto de geometria que mais que dobra de um frame pro outro, e ai
ele pula direto pro tamanho pedido em vez de dobrar duas vezes.

o `needed` que entra nessa conta vem do `upload`, que e o unico jeito de botar
dado no buffer:

```rust
pub fn upload<T: bytemuck::Pod>(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    data: &[T],
) {
    let bytes = bytemuck::cast_slice(data);
    self.ensure_capacity(device, bytes.len() as u64);
    queue.write_buffer(&self.buffer, 0, bytes);
}
```

duas coisas dessa funcao importam pra estrategia de reuso. a primeira e o
`ensure_capacity` no meio: todo upload primeiro garante que cabe, e na maioria
dos frames esse garantir e um `return` imediato porque ja coube no frame
passado. a segunda e o `0` no `queue.write_buffer(&self.buffer, 0, bytes)`. todo
upload escreve a partir do inicio do buffer, no offset zero, sobrescrevendo o que
estava la. o comentario no topo do arquivo promete "partial writes", escritas
parciais, e na pratica o que existe hoje no metodo `upload` e a reescrita do
buffer inteiro a partir do zero a cada chamada. nao tem append, nao tem escrever
no meio: o caminho exposto reescreve a folha do comeco. marco essa diferenca
entre o comentario e o codigo porque ela e o tipo de coisa que vale conferir na
fonte e nao na legenda, e porque o reuso aqui e do buffer, da panela, nao dos
bytes velhos que estavam nele.

esse `upload` nao acontece solto. ele acontece dentro da layer, que guarda os
buffers como posse opcional e persistente, em
`crates/engine/src/compositor/layer/mod.rs`:

```rust
pub(crate) quad_vb: Option<GpuVec>,
pub(crate) quad_ib: Option<GpuVec>,
```

e tem um par desses pra cada tipo de geometria que a layer desenha: quad, sdf,
shadow, image, backdrop, text. o `Option` e o que deixa a layer existir sem
buffer nenhum ate precisar de um. a alocacao e preguicosa e mora em
`crates/engine/src/compositor/layer/geometry.rs`:

```rust
pub(crate) fn upload_quad_geometry(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    if !self.quad_vertices.is_empty() {
        let vb = self.quad_vb.get_or_insert_with(|| {
            GpuVec::new(
                device,
                "layer_quad_vb",
                wgpu::BufferUsages::VERTEX,
                INITIAL_VB_SIZE,
            )
        });
        vb.upload(device, queue, &self.quad_vertices);

        let ib = self.quad_ib.get_or_insert_with(|| {
            GpuVec::new(
                device,
                "layer_quad_ib",
                wgpu::BufferUsages::INDEX,
                INITIAL_IB_SIZE,
            )
        });
        ib.upload(device, queue, &self.quad_indices);
    }
}
```

o `get_or_insert_with` e o reuso entre frames inteiro numa chamada. na primeira
vez que a layer tem quad pra desenhar, ele cria o `GpuVec` com o tamanho inicial.
da segunda em diante, ele acha o `Some` que ja esta la e devolve o mesmo buffer,
que o `upload` reescreve por cima. o `INITIAL_VB_SIZE` e 4096 bytes e o
`INITIAL_IB_SIZE` e 2048, os dois fixados em `layer/mod.rs`. esses tamanhos
iniciais sao um chute calibrado pra cena pequena caber sem realocar logo de cara:
uma layer com poucos retangulos ja nasce com folga e nunca cresce. uma layer
pesada cresce nas primeiras frames, ate o `capacity` estabilizar no pico, e dali
pra frente todo upload e reescrita pura, zero `create_buffer`.

esse e o reuso entre frames do lado dos bytes. o buffer e alocado uma vez por
layer por tipo de geometria, cresce ate o pico nas primeiras frames, e vira uma
folha que voce reescreve sem nunca arrancar. a layer segura a posse pelo
`Option<GpuVec>` de um frame pro outro, e o `get_or_insert_with` e a dobradica
que liga a primeira passada (criar) a todas as seguintes (reusar).

## nunca encolher e uma aposta com preco medido

"nunca encolhe" e a parte da regra que parece desperdicio e nao e. se uma layer
teve um pico de mil retangulos num frame e despencou pra dez no seguinte, o
buffer continua do tamanho de mil. a aposta e direta: o pico tende a voltar.
interface real oscila, mas oscila em torno de um teto. um painel que abriu cheio
vai abrir cheio de novo. realocar pra baixo no vale so pra realocar pra cima
quando o pico volta e churn puro, exatamente o custo que a peca inteira existe
pra evitar. entao o codigo escolhe ficar com a panela maior.

o preco dessa aposta e memoria de GPU presa que nao reflete o uso logico do
momento. e o codigo sabe disso de forma explicita, ao ponto de ter um metodo so
pra contar quanto esta preso:

```rust
/// Allocated GPU bytes (capacity, not the live byte count: the buffer
/// grows and never shrinks). Feeds the perf monitor's memory stats.
pub fn capacity_bytes(&self) -> u64 {
    self.capacity
}
```

o doc comment ja entrega a sutileza no parenteses: capacidade alocada, nao
contagem de bytes vivos. um monitor de memoria quer saber quanto de VRAM esta
amarrado, nao quanto esta em uso util naquele instante, porque o que estoura uma
placa e o reservado, nao o logicamente ocupado. o `compositor/memory.rs` soma
esse numero por todos os buffers de todas as layers:

```rust
let buffers = [
    &self.quad_vb,
    &self.quad_ib,
    &self.sdf_vb,
    &self.sdf_ib,
    // ... todos os pares vb/ib da layer
]
.into_iter()
.map(|b| b.as_ref().map_or(0, GpuVec::capacity_bytes))
.sum::<u64>();
```

o `map_or(0, GpuVec::capacity_bytes)` e onde o `Option<GpuVec>` paga a conta:
buffer que ainda nao foi alocado conta zero, buffer alocado conta a capacidade
cheia. o estilo aqui e o do crate inteiro, o engine nao pergunta nada pro driver
sobre memoria. ele estima a partir do que ele mesmo controla, as capacidades dos
buffers e as dimensoes das texturas. tem um teste que crava essa propriedade: um
`Compositor::new()` headless, sem nenhum device tocado, reporta zero bytes de
GPU, porque nenhum buffer chegou a ser criado. essa propriedade, headless igual a
zero, vai voltar quando eu falar do benchmark, porque e ela que explica o que o
`scene_build.rs` consegue e o que ele nao consegue medir.

a honestidade aqui e que "nunca encolhe" troca um custo que voce sente (churn de
alocacao no hot path) por um custo que voce monitora (VRAM parada). os dois sao
reais. a escolha foi mover o custo pra onde da pra medir e pra onde ele e
constante e previsivel, em vez de deixar ele variavel e escondido na frequencia
de frame.

## a texture pool, o mesmo instinto aplicado a textura

a segunda peca da familia mora em `crates/engine/src/gpu/texture_pool.rs`, e o
comentario de abertura ja declara a regra com as mesmas palavras da frigideira:

```rust
/// Pool of reusable GPU textures keyed by (width, height, format).
/// Grow-only: never destroys textures. In steady state, zero allocations.
```

"grow-only, never destroys textures, in steady state zero allocations" e
literalmente a mesma politica do `GpuVec` dita pra textura. a diferenca e a
forma do recurso. um buffer e uma fita de bytes que voce dobra. uma textura e
uma grade 2D, e duas texturas so sao intercambiaveis se tiverem a mesma largura,
a mesma altura e o mesmo formato de pixel. entao a pool nao "cresce" uma textura
existente. ela indexa as texturas pela forma e reusa a que casa.

a chave e isso, um struct de tres campos que e Hash e Eq:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TextureKey {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}
```

e a pool e um mapa dessa chave pra uma lista de entradas:

```rust
pub struct TexturePool {
    entries: FxHashMap<TextureKey, Vec<PoolEntry>>,
}
```

o `FxHashMap` vem do `rustc-hash 2.1`, conferido no `Cargo.toml`, e nao e o
`HashMap` padrao da std de proposito. a chave aqui nasce dentro do processo, e o
mapa e consultado no caminho de render, entao o que importa e velocidade bruta de
hash sobre inteiro, nao a resistencia a ataque de colisao que o SipHash da std
oferece e que aqui nao serve pra nada. e o mesmo criterio que o dirty tracking
usa pro hash de cena.

cada entrada da lista e um `PoolEntry`, e ele guarda um detalhe que e facil ler
rapido demais:

```rust
struct PoolEntry {
    #[allow(dead_code)] // Kept alive to own the GPU allocation; view references it
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    in_use: bool,
}
```

o `#[allow(dead_code)]` no campo `texture` nao e descuido, e uma decisao com
comentario explicando a invariante, do jeito que o projeto exige (allow por item,
com motivo, nunca crate inteiro). o codigo nunca le `texture` de novo depois de
criada. mas a `TextureView`, que e o que de fato entra nos render passes, e uma
vista pra dentro daquela textura. se a `texture` fosse dropada, a `view`
apontaria pro nada. entao a entrada segura a `texture` viva so pra manter a
alocacao de GPU enquanto a `view` a referencia. o `dead_code` e morto pra leitura
e vivissimo pra posse. e o `in_use` e o flag que faz a pool funcionar: marca se
aquela textura especifica esta emprestada agora ou disponivel pra reuso.

o `acquire` e onde reusar ou criar e decidido:

```rust
pub fn acquire(
    &mut self,
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> TextureHandle {
    let key = TextureKey {
        width,
        height,
        format,
    };
    let entries = self.entries.entry(key).or_default();

    for (i, entry) in entries.iter_mut().enumerate() {
        if !entry.in_use {
            entry.in_use = true;
            return TextureHandle {
                key,
                index: i,
                view: entry.view.clone(),
            };
        }
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pool_texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    // ...
}
```

o caminho quente e o `for` no meio. dada a chave (largura, altura, formato), a
pool varre a lista de entradas daquela forma e na primeira que estiver livre
(`!entry.in_use`) ela marca como em uso, clona a view, e devolve um handle. nesse
caminho nao tem `create_texture`. nenhum byte novo de placa. e o "a frigideira do
tamanho certo ja esta no fogao, pega ela". so quando o `for` termina sem achar
nenhuma livre e que o codigo cai pro `create_texture`, aloca uma textura nova
daquela forma exata, e empurra como entrada nova na lista. a textura nova ja
nasce `in_use: true` e vira mais uma panela no conjunto, disponivel pra reuso a
partir do proximo release.

a `view` e clonada no acquire (`entry.view.clone()`), e o `TextureHandle`
carrega essa view clonada em vez de uma referencia pra dentro da pool. isso e de
proposito: clonar a view desacopla o handle do emprestimo da pool, entao quem
segura o handle nao prende a pool inteira num borrow enquanto desenha. a parte 1
contou a mesma historia do `GpuVec.buffer()` devolvendo `&wgpu::Buffer`; aqui a
solucao e clonar a view (que e barato, e um handle com contagem de referencia por
baixo) em vez de emprestar, justamente pra esquivar o conflito de borrow que
trancaria a pool.

o `release` e o oposto exato do acquire, e e curtissimo:

```rust
pub fn release(&mut self, handle: TextureHandle) {
    if let Some(entries) = self.entries.get_mut(&handle.key)
        && handle.index < entries.len()
    {
        entries[handle.index].in_use = false;
    }
}
```

ele toma posse do handle (note que e por valor, `handle`, nao por referencia, o
que consome o handle e impede usar a textura depois de devolvida), acha a entrada
pela chave e pelo indice, e vira o `in_use` pra `false`. nada e destruido. a
textura continua alocada, a view continua valida, so o flag muda. devolver e
liberar pra reuso, nao liberar memoria. a panela volta pra boca do fogao, nao pro
armario. esse `if let ... && ...` com o `let`-chain numa condicao so e edition
2024, conferida no `Cargo.toml`, e e a forma que o repo usa pra checar a chave e
o limite do indice numa expressao so.

## o ciclo de vida dentro de um frame

a parte mais bonita da pool nao e o acquire nem o release isolados, e o
bailado dos dois dentro de um frame, porque e ali que o reuso aparece em
movimento. o caso mais ilustrativo esta no backdrop blur, em
`crates/engine/src/window/render_passes.rs`. o efeito de borrar o que esta atras
de uma layer precisa de um alvo temporario pra compor o fundo, e outro pra
guardar o resultado borrado. os dois saem da pool:

```rust
let compose = texture_pool.acquire(&gpu.device, vw, vh, format);
```

esse `compose` recebe a composicao de tudo que esta abaixo da layer atual, com a
cor de fundo e as layers de baixo desenhadas em ordem. depois ele e borrado, e o
resultado vai pra outra textura que o proprio `apply_blur` pega da pool por
dentro. assim que o blur termina, o `compose` ja cumpriu o papel e e devolvido na
hora:

```rust
let blurred = effects.apply_blur(
    &mut crate::effects::EffectContext {
        device: &gpu.device,
        queue: &gpu.queue,
        encoder,
        pool: texture_pool,
        source_view: &compose_view,
        width: vw,
        height: vh,
    },
    sigma,
);
draw_calls += 2;
texture_pool.release(compose);
```

repara na sequencia. acquire do `compose`, blur (que faz acquire de mais uma
textura por dentro), release do `compose` assim que ele nao serve mais. esse
release no meio do frame e o que deixa o `compose` voltar pra fila e ser reusado
pelo proximo efeito do mesmo frame, ou pela proxima layer com backdrop. uma tela
com tres paineis de vidro fosco nao aloca tres `compose`, ela aloca um e reusa
ele tres vezes, contanto que os releases caiam entre os acquires.

tem um comentario nesse arquivo que e a coisa mais sutil da peca inteira, e vale
ler com calma:

```rust
// Releasing before the sampling draw is recorded is safe: the bind
// group keeps the texture alive, and any reuse of the pooled texture
// is recorded -- and therefore executes -- after this read.
texture_pool.release(blurred);
```

a textura borrada e devolvida pra pool antes mesmo de o draw que a le ter sido
gravado no encoder. parece bug. parece que voce devolveu a frigideira antes de
servir o prato que estava nela. a razao de ser seguro tem dois fios. o primeiro:
o bind group criado com a view daquela textura segura uma referencia que mantem a
textura viva, entao o release nao destroi nada (a pool nunca destroi, ja vimos),
so marca como reusavel. o segundo, e esse e o pulo do gato: a GPU executa os
comandos na ordem em que foram gravados no encoder. se algum acquire posterior
pegar essa mesma textura e gravar uma escrita nela, essa escrita vai estar depois
no encoder do que a leitura que estamos gravando agora, e portanto vai executar
depois. a ordem de gravacao no encoder e a ordem de execucao na placa, e isso e o
que torna devolver cedo correto. e um raciocinio de sincronizacao temporal que
mora na semantica do wgpu 28, nao no borrow checker, e o comentario esta ali
exatamente porque sem ele a linha parece errada.

o caminho geral dos efeitos de layer, no mesmo arquivo, mostra o reuso em cadeia
com um ping-pong de handles:

```rust
let mut current_view_owner: Option<crate::gpu::texture_pool::TextureHandle> = None;

for effect in effects {
    let sv = current_view_owner
        .as_ref()
        .map(|h| h.view())
        .unwrap_or(source_view);
    let handle = match effect {
        LayerEffect::Blur { sigma } => effect_processor.apply_blur(/* ... */),
        LayerEffect::Shadow { sigma, color } => effect_processor.apply_shadow(/* ... */),
    };
    if let Some(prev) = current_view_owner.take() {
        texture_pool.release(prev);
    }
    current_view_owner = Some(handle);
}
```

uma layer pode ter uma pilha de efeitos, blur depois de shadow depois de outro
blur. cada efeito le a saida do anterior e produz uma nova. o `current_view_owner`
guarda o handle da textura atual; quando o proximo efeito produz a sua, o anterior
e devolvido pra pool na hora (`texture_pool.release(prev)`) e vira candidato a
reuso pelo efeito seguinte. uma cadeia de cinco efeitos nao precisa de cinco
texturas vivas ao mesmo tempo, ela vai reciclando: a textura que o efeito 1 usou e
liberada e pode ser a mesma que o efeito 3 pega. e o mesmo instinto de cozinha
levado pro limite, lavando a panela entre os passos do mesmo prato em vez de
empilhar panela suja.

## o reuso entre frames, e o resize que invalida

dentro de um frame a pool recicla. entre frames ela persiste. a pool e criada uma
vez, no setup da janela em `crates/engine/src/window/state.rs` (um
`TexturePool::new()`), e vive na estrutura de estado da aplicacao de frame em
frame. no fim de um frame, todos os handles ja foram devolvidos via release, o que
quer dizer que todas as entradas estao com `in_use: false`. quando o proximo frame
chama acquire pra mesma forma de textura, o `for` do acquire acha aquelas entradas
livres e devolve elas. nenhum `create_texture`. e por isso que o comentario diz
"in steady state, zero allocations": numa tela que repinta o mesmo tipo de efeito
todo frame, a pool aquece nas primeiras frames criando as texturas que precisa, e
dali pra frente so vira flags de `in_use` pra la e pra ca.

o `memory_bytes` conta o custo desse reuso, exatamente como o `capacity_bytes`
contava pro `GpuVec`:

```rust
pub fn memory_bytes(&self) -> u64 {
    self.entries
        .iter()
        .map(|(key, entries)| {
            u64::from(key.width)
                * u64::from(key.height)
                * bytes_per_pixel(key.format)
                * entries.len() as u64
        })
        .sum()
}
```

a conta e largura vezes altura vezes bytes por pixel vezes o numero de entradas
daquela forma. note o `entries.len()`: ele conta todas as texturas daquela chave,
em uso ou nao, porque a pool e grow-only e o que esta parado tambem ocupa placa.
mesma honestidade do `capacity_bytes`, o numero reporta o reservado, nao o em uso.
o `bytes_per_pixel` no fim do arquivo mapeia os formatos que a pool de fato ve
(formato de surface e de efeito) pro custo por pixel, com um fallback de 4 bytes
pro formato desconhecido, que e o caso comum. e esse `memory_bytes` alimenta o
monitor de performance, em `crates/engine/src/window/render.rs`, junto com a
memoria dos buffers:

```rust
texture_pool_bytes: texture_pool.memory_bytes(),
```

o reuso grow-only tem um caso que ele precisa quebrar de proposito, e e o resize
da janela. se a janela muda de 1920x1080 pra 1280x720, todas as texturas da pool
guardadas na forma antiga viraram lixo: ninguem mais vai pedir uma textura de
1920x1080 enquanto a surface for 1280x720, e elas ficariam presas pra sempre
ocupando placa, porque grow-only nunca destroi sozinho. pra esse caso existe o
`invalidate_size`:

```rust
pub fn invalidate_size(&mut self, width: u32, height: u32) {
    self.entries.retain(|key, entries| {
        if key.width != width || key.height != height {
            let any_in_use = entries.iter().any(|e| e.in_use);
            if any_in_use {
                log::warn!(
                    "TexturePool: cannot invalidate {}x{} -- texture still in use",
                    key.width,
                    key.height
                );
                return true;
            }
            false
        } else {
            true
        }
    });
}
```

ele e a unica porta de saida da regra "nunca destroi". no resize, ele varre as
entradas e descarta as que nao casam com o novo tamanho da surface, com uma
guarda: se alguma textura daquela forma ainda esta `in_use`, ela e mantida e o
codigo loga um aviso, porque destruir uma textura que um render pass do frame
corrente ainda esta usando seria o tipo de erro que essa familia inteira existe
pra impedir. entao a poda so acontece pra texturas ociosas de tamanho velho. o
grow-only continua valendo dentro de cada tamanho de surface; o `invalidate_size`
so apaga a memoria de um tamanho que nao existe mais. e a contrapartida exata da
aposta do `GpuVec`: o buffer nunca encolhe porque o pico volta, mas a textura de
um tamanho de janela que nao existe mais nunca volta, entao essa a pool deixa ir.

## por que isso casa com o render on demand

aqui as duas pecas cruzam com um assunto que parece de outro departamento e nao
e: invalidacao. e o cruzamento e o que faz o reuso entre frames ser mais que uma
otimizacao, ser um pre-requisito de correcao.

o engine so renderiza quando alguem pede um frame. e disciplina de bateria e CPU
copiada de editor de producao, e esta registrada no adr
`kdb/adr/render-on-demand-requires-explicit-invalidation.md`. sob esse modelo, o
estado de repouso normal do app e nao desenhar. e a unica razao de o repouso ser
seguro e que tudo do ultimo frame ainda esta resident: os `GpuVec` das layers
seguem alocados com a geometria de ontem, e as texturas offscreen das layers
seguem na placa com o pixel de ontem. a persistencia grow-only e o substrato que
torna "nao fazer nada" um estado valido. se cada frame jogasse fora seus buffers
e suas texturas, nao existiria "o frame de ontem ainda esta la pra reaproveitar",
e render on demand desabaria, porque toda frame teria que reconstruir tudo do
zero, que e justo o que o modelo evita.

o adr conta o preco de esquecer isso. ele descreve um bug que foi shipado:
eventos de scroll foram consumidos, o estado mudou, os handlers retornaram
`false`, nada invalidou, e o showcase parecia completamente travado. o defeito foi
lido primeiro como "scroll nao implementado", quando era invalidacao faltando. a
frase que o adr cristaliza vale colar na parede: returning false is a statement
that nothing visible changed, and the scheduler believes it. retornar `false` e
uma afirmacao de que nada visivel mudou, e o escalonador acredita. as duas pecas
deste capitulo sao o motivo de o escalonador poder acreditar com seguranca: ele
sabe que os buffers e as texturas do ultimo frame estao intactos, prontos pra
redesenhar identico. a invalidacao e o unico jeito de dizer "dessa vez tem coisa
nova, reescreve o buffer e refaz o efeito".

o adr ainda amarra o caso da animacao, que e onde o reuso e a invalidacao se
tocam mais de perto: as animacoes mantem o fluxo de frames so enquanto ativas,
pela condicao `is_animating || compositor.needs_render()`. enquanto a animacao
roda, os buffers sao reescritos e as texturas refeitas todo frame, e e ai que a
folha grow-only paga: a animacao nao realoca nada, ela reescreve por cima de
buffer ja do tamanho do pico e reusa textura ja do tamanho da surface. a
animacao mais pesada que o app aguenta e exatamente a que reusa melhor, porque
ela aqueceu a pool e os buffers na primeira frame e nunca mais tocou o alocador.
e o "avoid" do adr fecha o contrato pelo lado do que nao fazer: nunca debugar
"UI congelada" enfiando redraw em loop, ache o handler que mentiu sobre nao mudar
estado. o reuso entre frames e o que deixa o repouso barato; a invalidacao e o
que tira do repouso na hora certa. uma sem a outra nao fecha o render on demand.

## o numero, e a honestidade sobre o que o scene_build mede

eu nao fecho um capitulo de memoria e performance sem encarar o benchmark, mas
dessa vez a parte interessante e o que ele nao mede, e por que.

a ancora de bench do capitulo e `crates/engine/benches/scene_build.rs`. ela tem
seis grupos: `push_rects`, `push_paths`, `dirty_tracking`, `tessellation`,
`signals`, `text_hashing`, todos amarrados no `criterion_group!` do fim do
arquivo. a versao do criterion no projeto e a 0.5, conferida no `Cargo.toml`. o
que importa pras pecas deste capitulo e olhar o que esses grupos de fato
exercitam, e a resposta honesta e: nenhum deles toca o `GpuVec` nem a texture
pool direto. todos constroem um `Compositor::new()`, que e headless. e ja vimos,
pelo teste de memoria, que compositor headless nao aloca buffer nenhum (reporta
zero bytes de GPU). sem device, nao tem `create_buffer`, nao tem `create_texture`,
nao tem `queue.write_buffer`. o que o `scene_build.rs` mede e o lado CPU que
alimenta as duas pecas: a construcao dos `Vec` de vertice e indice que mais
tarde, num app com placa de verdade, seriam passados pro `upload`, e que
disparariam o `ensure_capacity` e o `acquire`.

o grupo mais direto e o `push_rects`:

```rust
fn bench_push_rects(c: &mut Criterion) {
    let mut group = c.benchmark_group("push_rects");
    for count in [100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &n| {
            b.iter(|| {
                let mut comp = Compositor::new();
                for i in 0..n {
                    let f = i as f32;
                    comp.draw_rect(f, f, 100.0, 50.0, [0.5, 0.5, 0.5, 1.0]);
                }
                black_box(&comp);
            });
        });
    }
    group.finish();
}
```

ele mede empurrar 100, 1000 e 10000 retangulos pra dentro de um compositor novo.
esse e o trabalho que constroi a cena que, depois de virar geometria, seria
escrita no `GpuVec`. medir isso separado do upload e a fronteira certa pra um
bench que precisa rodar sem GPU pra ser reproduzivel em CI, e e coerente com a
arquitetura, que separa a fase CPU (descrever a cena, montar vertice) da fase GPU
(upload mais render pass) de proposito.

mas seria desonesto eu te vender daqui um "custo de upload do GpuVec" ou um
"custo de acquire da pool", porque esses tempos nao estao sendo medidos nesta
ancora. marco explicito como nao confirmado: o `scene_build.rs` nao isola o custo
do `upload`, do `ensure_capacity`, do `queue.write_buffer`, do `create_texture`
nem do `acquire`. ele roda headless e nao materializa nenhuma das duas pecas. um
numero de upload ou de alocacao de textura exigiria um profile com device real,
fora deste arquivo, que dependeria de driver e de placa e por isso nao caberia num
bench de CPU deterministico. o que da pra afirmar com fundamento e o formato do
trabalho que precede as duas pecas (o `push_rects`, o `push_paths`, o
`text_hashing`), e a propriedade estrutural de que o caminho quente delas, em
regime estavel, nao aloca nada. o custo do reuso em si, o `if needed <=
self.capacity { return; }` e o `for` que acha a entrada livre, e tao barato que o
ganho dele nao esta em ser rapido de medir, esta em ser uma ausencia de alocacao,
que e justo o que nao da pra cronometrar com proveito num bench de cena.

## por que assim, e nao de outro jeito

da pra perguntar se valia construir duas abstracoes pra isso. podia ser
`wgpu::Buffer` e `wgpu::Texture` soltos, criados e dropados quando desse na
telha. da pra fazer. e o custo aparece exatamente onde voce nao olha: no alocador
da placa, na hora em que a cena fica pesada e o app comeca a alocar e liberar
recurso de GPU dezenas de vezes por frame. o ganho das duas pecas nao e ter
inventado o conceito de pool nem de buffer que cresce, isso e velho. o ganho e
ter desenhado as duas em torno de uma propriedade so, "em regime estavel, zero
alocacao", e ter pago por ela com memoria parada que o codigo mede em vez de
ignorar.

a simetria entre as duas e o que eu acho mais limpo. o `GpuVec` e a texture pool
resolvem o mesmo problema (criar e destruir recurso de GPU e caro) com a mesma
politica (cresce, nunca encolhe sozinho, reusa entre frames) adaptada a duas
formas de memoria (fita de bytes contigua, grade 2D indexada por dimensao). a
fita voce dobra; a grade voce indexa e reusa por forma. as duas tem um metodo de
contagem (`capacity_bytes`, `memory_bytes`) que reporta o reservado e nao o usado,
porque e o reservado que estoura placa. as duas tem uma valvula de escape da regra
de nunca encolher, e as duas valvulas sao deliberadas: o `GpuVec` so realoca pra
cima, nunca pra baixo, apostando que o pico volta; a pool so apaga textura no
`invalidate_size`, quando o tamanho de janela que ela servia deixou de existir e o
pico daquela forma nunca mais volta. uma aposta diz "fica grande que vai precisar
de novo", a outra diz "esse tamanho morreu, pode ir", e as duas estao certas pelo
mesmo raciocinio sobre quando o pico reaparece.

e o casamento com o render on demand e o que tira as duas do balde "otimizacao" e
poe no balde "correcao". o reuso entre frames nao e so mais rapido, ele e o que
faz "nao desenhar" ser um estado valido, porque o frame de ontem so esta la pra
ser reaproveitado se os buffers e as texturas dele sobreviveram. a parte 1 mostrou
o borrow checker garantindo que voce nunca escreve num buffer enquanto a GPU le
dele. esta parte mostra a outra metade: o buffer e a textura sobrevivem de
proposito justo pra que, na ausencia de invalidacao, o engine possa redesenhar o
mesmo frame sem reconstruir nada. uma metade e mecanica de compilador, a outra e
estrategia de memoria, e as duas servem o mesmo contrato de invalidacao.

## o que isso me ensinou

a licao que eu levei daqui nao foi sobre GPU, foi sobre custo invisivel. o
desperdicio de alocar e liberar recurso todo frame nao aparece no protótipo,
porque a placa esconde, e e exatamente por isso que ele e perigoso: voce so
descobre que estava pagando quando a cena fica pesada o bastante pra a conta vir,
e ai o custo ja esta espalhado por todo lugar. as duas pecas deste capitulo sao a
decisao de pagar esse custo uma vez, no aquecimento, em vez de pagar um
pouquinho toda frame pra sempre. e a forma de pagar uma vez foi reusar: a
frigideira que fica no fogao, o buffer que vira folha reescrita, a textura que
volta pra pool com um flag em vez de pro armario.

a segunda licao, a que demorou mais, foi que reuso bom exige medir o que voce nao
devolve. nunca encolher e uma aposta, e aposta sem placar e fe. o que faz a
estrategia ser engenharia e nao otimismo e o `capacity_bytes` e o `memory_bytes`
do lado, contando a VRAM parada, pra que se a aposta de "o pico volta" estiver
errada pra algum app o numero apareca no monitor antes de virar problema. o reuso
e barato; saber quanto ele custa em memoria parada e o que mantem ele honesto.

se eu fosse deixar uma frase pra alguem que esta projetando o cache de recurso de
GPU de um motor proprio: a parte facil e reusar, qualquer pool faz; a parte que
separa um cache que aguenta producao de um que vaza placa em silencio e ter uma
porta de saida da regra de nunca encolher (`invalidate_size` quando o tamanho
morre) e um numero que conta o que voce esta segurando (`memory_bytes`). reusar
sem medir e como deixar todas as panelas no fogao e nunca olhar o gas.

## rastros

codigo (crate engine, conferido contra a arvore atual)
- `crates/engine/src/gpu/vec.rs:5` (`struct GpuVec`, quatro campos: `buffer`,
  `capacity`, `usage`, `label`)
- `crates/engine/src/gpu/vec.rs:33` (`ensure_capacity`, `(self.capacity *
  2).max(needed)`, dobra amortizada, `self.buffer = device.create_buffer(...)`
  substitui o antigo)
- `crates/engine/src/gpu/vec.rs:48` (`upload<T: bytemuck::Pod>`, chama
  `ensure_capacity` e escreve `queue.write_buffer(&self.buffer, 0, bytes)` no
  offset 0)
- `crates/engine/src/gpu/vec.rs:1` (comentario de topo: "grows, never shrinks,
  partial writes")
- `crates/engine/src/gpu/vec.rs:63` (`capacity_bytes`, capacidade alocada e nao
  bytes vivos; alimenta o perf monitor)
- `crates/engine/src/compositor/layer/mod.rs:59` (campos `quad_vb`/`quad_ib` e os
  outros pares como `Option<GpuVec>`, posse opcional e persistente entre frames)
- `crates/engine/src/compositor/layer/mod.rs:95` (`INITIAL_VB_SIZE = 4096`,
  `INITIAL_IB_SIZE = 2048`)
- `crates/engine/src/compositor/layer/geometry.rs:488` (`upload_quad_geometry`,
  `get_or_insert_with` cria uma vez e reusa o `GpuVec` nos frames seguintes)
- `crates/engine/src/compositor/memory.rs:16` (`gpu_memory_bytes`, soma
  `capacity_bytes` por buffer via `map_or(0, GpuVec::capacity_bytes)`)
- `crates/engine/src/compositor/memory.rs:58`
  (`headless_compositor_reports_zero_gpu_memory`, compositor sem device aloca zero
  buffer)
- `crates/engine/src/gpu/texture_pool.rs:1` (doc de topo: grow-only, never
  destroys textures, in steady state zero allocations)
- `crates/engine/src/gpu/texture_pool.rs:6` (`TextureKey`, indexa por width,
  height, format; `derive(Hash, Eq)`)
- `crates/engine/src/gpu/texture_pool.rs:12` (`PoolEntry`, `texture` com
  `#[allow(dead_code)]` mantida viva pra a `view` referenciar, mais `in_use`)
- `crates/engine/src/gpu/texture_pool.rs:32` (`TexturePool`, `FxHashMap<TextureKey,
  Vec<PoolEntry>>`)
- `crates/engine/src/gpu/texture_pool.rs:49` (`acquire`, `for` que reusa entrada
  `!in_use` ou cai pro `device.create_texture`)
- `crates/engine/src/gpu/texture_pool.rs:112` (`release`, consome o handle e seta
  `in_use = false`, nao destroi)
- `crates/engine/src/gpu/texture_pool.rs:122` (`memory_bytes`, soma width*height*
  bpp*len por chave; conta entradas em uso e ociosas)
- `crates/engine/src/gpu/texture_pool.rs:135` (`invalidate_size`, unica poda da
  regra grow-only; descarta forma de tamanho velho, mantem se ainda `in_use`)
- `crates/engine/src/gpu/texture_pool.rs:157` (`bytes_per_pixel`, formatos vistos
  pela pool, fallback de 4 bytes)
- `crates/engine/src/window/render_passes.rs:164` (`texture_pool.acquire` do
  `compose` no backdrop blur)
- `crates/engine/src/window/render_passes.rs:216` (`texture_pool.release(compose)`
  no meio do frame, devolve pra reuso)
- `crates/engine/src/window/render_passes.rs:232` (comentario: release antes do
  draw de leitura e seguro porque o bind group mantem a textura viva e o reuso e
  gravado, e executa, depois da leitura)
- `crates/engine/src/window/render_passes.rs:395` (cadeia de efeitos com
  `current_view_owner`, ping-pong de handle: release do anterior, acquire do
  proximo)
- `crates/engine/src/window/render.rs:209` (`texture_pool_bytes:
  texture_pool.memory_bytes()`, a pool alimenta o monitor)

adr e contrato de invalidacao
- `kdb/adr/render-on-demand-requires-explicit-invalidation.md:11` (context:
  renderiza so quando alguem pede frame; bug de scroll que pareceu app congelado)
- `kdb/adr/render-on-demand-requires-explicit-invalidation.md:25` (returning false
  is a statement that nothing visible changed, and the scheduler believes it)
- `kdb/adr/render-on-demand-requires-explicit-invalidation.md:28` (animacoes
  mantem frames so enquanto ativas, `is_animating || compositor.needs_render()`)
- `kdb/adr/render-on-demand-requires-explicit-invalidation.md:33` (avoid: nunca
  debugar UI congelada com redraw em loop; ache o handler que mentiu)

benchmark
- `crates/engine/benches/scene_build.rs:10` (`bench_push_rects`, 100/1000/10000
  rects num `Compositor::new()` headless)
- `crates/engine/benches/scene_build.rs:186` (`criterion_group!` com os seis
  grupos: push_rects, push_paths, dirty_tracking, tessellation, signals,
  text_hashing)

versoes (conferidas contra o Cargo.toml)
- `Cargo.toml:50` wgpu 28 (`wgpu::Buffer`, `wgpu::Texture`, `BufferUsages`,
  `queue.write_buffer`, ordem de gravacao no encoder)
- `Cargo.toml:52` bytemuck 1 com feature `derive` (`Pod`, `cast_slice` no upload)
- `Cargo.toml:68` rustc-hash 2.1 (`FxHashMap` da texture pool)
- `Cargo.toml:99` criterion 0.5 (o bench scene_build)
- `Cargo.toml:23` edition 2024 (o `let`-chain `if let ... && ...` no `release`)

nao confirmado
- nenhuma ancora deste capitulo mede o custo do `upload`, do `ensure_capacity`,
  do `queue.write_buffer`, do `create_texture` ou do `acquire`. o `scene_build.rs`
  roda headless (`Compositor::new()`, sem device), entao nao materializa nenhum
  `GpuVec` nem nenhuma textura de pool; ele mede a construcao CPU da cena que
  precede as duas pecas, nao o reuso nem a alocacao de GPU em si. um numero real
  exigiria profile com device, fora desta ancora.
- o comentario de topo do `vec.rs` diz "partial writes", mas o metodo `upload`
  exposto hoje reescreve o buffer inteiro a partir do offset 0; nao ha, no caminho
  lido, escrita parcial no meio do buffer. anotei a diferenca entre o comentario e
  o codigo em vez de assumir o comentario.
- a afirmacao de que devolver a textura pra pool antes de gravar o draw de leitura
  e seguro porque a ordem de gravacao no encoder e a ordem de execucao na GPU e o
  comportamento documentado do wgpu (e o que o comentario no codigo afirma), nao
  uma medicao feita aqui.
