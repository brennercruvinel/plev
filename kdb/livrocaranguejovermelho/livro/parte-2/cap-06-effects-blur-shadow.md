---
title: "effects: blur, shadow e o composite pass"
parte: 2
status: rascunho
rastros:
  - crates/engine/src/effects/processor.rs:9
  - crates/engine/src/effects/types.rs:6
  - crates/engine/src/effects/types.rs:52
  - crates/engine/src/effects/pipelines.rs:60
  - crates/engine/src/effects/apply.rs:71
  - crates/engine/src/gpu/texture_pool.rs:32
  - crates/engine/src/gpu/shaders/blur.wgsl:47
  - crates/engine/src/gpu/shaders/shadow.wgsl:39
  - crates/engine/src/gpu/shaders/composite.wgsl:30
  - crates/engine/src/compositor/layer/mod.rs:23
  - crates/engine/src/window/render_passes.rs:371
  - kdb/adr/effects-architecture.md
  - kdb/mission/steps/checked/task-08-done.md
---

# effects: blur, shadow e o composite pass

pega um card de produto qualquer, daqueles que ficam flutuando sobre um fundo.
o card tem uma sombra macia embaixo, que sugere que ele esta um pouco mais perto
de voce do que o resto da tela. atras dele, quando o fundo escorrega, o vidro
fica fosco, borrado, com a luz vazando difusa. ninguem olha pra isso e pensa
"que belo gaussian blur de 13 taps". a pessoa so sente que a interface tem
profundidade. e o trabalho de quem escreve a engine e fazer essa sensacao
acontecer sem rasgar o orcamento de tempo de um frame.

esse capitulo e sobre o subsistema que produz esses dois efeitos no plev: o
blur e a sombra. os dois sao pos-processamento. a camada ja foi desenhada numa
textura, e o efeito opera sobre essa textura, nao sobre cada retangulo ou cada
glifo que entrou nela. e isso muda tudo na conta de custo. um botao com sombra
nao re-desenha o botao para fazer a sombra. ele pega a silhueta da textura que
ja existe, borra, e compoe por baixo. o desenho original aconteceu uma vez.

abro pela imagem do card porque e o jeito honesto de comecar. mas a pergunta
que move o capitulo e mais seca: como voce borra uma textura inteira na GPU,
todo frame, sem alocar memoria nova e sem usar compute shader? a resposta tem
tres pecas: um processador que segura os pipelines, um par de shaders de
fragmento que fazem o trabalho pesado, e um pool de texturas que se recusa a
liberar memoria. vou descer nas tres.

## o EffectProcessor segura os pipelines

a primeira decisao registrada no adr de efeitos e a mais importante: fragment
shader only. sem compute. o `kdb/mission/steps/checked/task-08-done.md` marca
isso como decisao explicita, alinhada com `rules.md`. o motivo e portabilidade.
compute shader funciona bem no desktop, mas o alvo do plev inclui o browser via
WebGPU e WASM, e fragment shader e o caminho que roda igual em todo lugar sem
caminho condicional. um blur de 13 taps cabe folgado num fragment shader. nao
precisa de compute para isso.

o objeto que carrega esse mundo e o `EffectProcessor`. ele e dono dos tres
pipelines de render e dos layouts de bind group que eles compartilham:

```rust
pub struct EffectProcessor {
    pub blur_pipeline: wgpu::RenderPipeline,
    pub shadow_pipeline: wgpu::RenderPipeline,
    pub composite_pipeline: wgpu::RenderPipeline,
    pub effect_texture_bgl: wgpu::BindGroupLayout,
    pub blur_uniform_bgl: wgpu::BindGroupLayout,
    pub shadow_uniform_bgl: wgpu::BindGroupLayout,
    pub composite_uniform_bgl: wgpu::BindGroupLayout,
    pub linear_sampler: wgpu::Sampler,
    pub(super) composite_uniform_buffer: wgpu::Buffer,
    pub(super) surface_format: wgpu::TextureFormat,
}
```

tres pipelines, um por efeito de base: blur, shadow, composite. quatro layouts
de bind group, um sampler linear, um buffer de uniform para o composite e o
formato da surface guardado para criar texturas compativeis. o `EffectProcessor::new`
constroi tudo isso uma vez, na inicializacao do device, e nunca mais. criar
pipeline na GPU e caro, entao isso fica fora do hot path por construcao.

o sampler é linear, com `ClampToEdge` nos dois eixos. linear porque o blur
amostra entre texels e quer interpolacao suave; clamp porque quando o kernel
puxa amostra de fora da borda, voce quer repetir a borda, nao envolver para o
outro lado da textura. detalhe pequeno, mas se fosse `Repeat` a sombra de um
card no canto vazaria pixel do canto oposto.

os bind groups seguem um arranjo fixo que o adr descreve: o group 0 carrega
textura mais sampler e e compartilhado por blur, shadow e composite; o group 1
carrega o uniform especifico de cada efeito. dois bind groups no maximo, o que
fica bem dentro do limite de 4 que o WebGPU garante em WASM. essa folga nao e
acidente. quem mira o browser conta os bind groups.

um detalhe que vale parar: dos quatro buffers de uniform possiveis, so o do
composite é persistente. blur e shadow criam o buffer de uniform na hora, por
pass, e jogam fora. o comentario no proprio `new` explica o porque:

```rust
// Blur/shadow uniforms are transient per pass (see `apply.rs`:
// staged `write_buffer`s all land before the next submit, so a
// shared buffer would leak the last write into every pass). Only
// the composite alpha buffer persists.
```

guarda essa frase. ela é a parte mais sutil do subsistema inteiro e vou voltar
nela quando o blur entrar.

## os pesos do gaussian saem no CPU

antes de borrar qualquer coisa, alguem precisa decidir o peso de cada amostra.
um blur gaussiano pondera o pixel central mais que os vizinhos, e os vizinhos
mais proximos mais que os distantes, seguindo a curva do sino. no plev esses
pesos saem no CPU, uma vez por sigma, na funcao `gaussian_weights`:

```rust
/// Compute 13-tap symmetric Gaussian weights for the given sigma.
/// Returns [center, w1, w2, ..., w6, 0, 0, ...] (16 floats).
pub fn gaussian_weights(sigma: f32) -> [f32; 16] {
    let mut weights = [0.0f32; 16];
    if sigma <= 0.0 {
        weights[0] = 1.0;
        return weights;
    }

    let s2 = 2.0 * sigma * sigma;
    let mut sum = 0.0f32;

    weights[0] = 1.0;
    sum += weights[0];

    for (i, weight) in weights.iter_mut().enumerate().skip(1).take(6) {
        let w = (-((i * i) as f32) / s2).exp();
        *weight = w;
        sum += 2.0 * w;
    }

    let inv = 1.0 / sum;
    for w in weights.iter_mut().take(7) {
        *w *= inv;
    }

    weights
}
```

treze taps: o centro mais seis offsets simetricos para cada lado. o array tem 16
floats, nao 13. os tres ultimos sao padding. esse padding nao e descuido, e o
preco de alinhamento do WGSL: do lado do shader os pesos chegam como
`array<vec4<f32>, 4>`, e um vec4 ocupa 16 bytes alinhados. sete pesos uteis
viram dois vec4 mais um terco do terceiro, e o resto fica zero. o
`BlurUniforms` do lado Rust carrega esse layout literal:

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurUniforms {
    pub direction: [f32; 2],
    pub texel_size: [f32; 2],
    pub weights: [f32; 16], // 13 weights + 3 padding (vec4 aligned)
}
```

o `#[repr(C)]` mais `bytemuck::Pod` é o que garante que esses bytes vao para a
GPU exatamente na ordem que o WGSL espera ler. se o layout Rust e o layout WGSL
divergem um byte que seja, o blur sai torto e o erro nao aparece num panic, ele
aparece na tela, o que é pior de caçar.

o loop normaliza os pesos para somar 1.0. isso importa: se a soma nao for 1.0, o
blur ou clareia ou escurece a imagem, porque a energia total muda. os testes do
crate trancam isso. tem um teste que varre sigmas de 0.5 a 6.0 e exige que a
soma fique a menos de `1e-5` de 1.0, outro que checa o caso `sigma = 0` (peso
central 1.0, resto zero, ou seja, sem blur), outro que checa o decaimento
monotonico, e um que checa que o padding e realmente zero. quatro testes para
uma funcao de vinte linhas. e a proporcao certa, porque essa funcao alimenta um
shader que roda em todo pixel da tela e um bug aqui é difuso de diagnosticar.

por que no CPU e nao no shader? porque o sigma muda raramente, e o peso so
depende do sigma. calcular `exp` para sete amostras uma vez por frame no CPU é
de graça comparado com recalcular dentro de um fragment shader que roda milhoes
de vezes por frame. o trabalho mais rapido continua sendo o que nao acontece, e
aqui ele simplesmente nao acontece na GPU.

## blur separavel, dois passes, treze amostras

o shader de blur é um fragment shader que amostra a textura ao longo de uma
direcao. o `vs_main` nao recebe vertex buffer nenhum. ele desenha um triangulo
de tela cheia a partir do `vertex_index`, o truque dos tres vertices que cobrem
todo o clip space. tres `draw(0..3)` e a tela inteira esta coberta, sem geometria
para subir. o `fs_main` é onde o blur mora:

```rust
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let step = blur.direction * blur.texel_size;

    // Center sample (weight index 0)
    var color = textureSample(source_texture, source_sampler, in.uv) * get_weight(0);

    // Symmetric taps: indices 1..6 correspond to offsets +/-1..+/-6
    for (var i: i32 = 1; i < 7; i = i + 1) {
        let offset = step * f32(i);
        let w = get_weight(i);
        color += textureSample(source_texture, source_sampler, in.uv + offset) * w;
        color += textureSample(source_texture, source_sampler, in.uv - offset) * w;
    }

    return color;
}
```

uma amostra central mais seis pares simetricos, treze no total. o `direction`
diz se o passe e horizontal `(1,0)` ou vertical `(0,1)`, e o `texel_size` é
`1/largura, 1/altura`, o tamanho de um texel em coordenada UV. o mesmo shader
serve aos dois passes; só muda o uniform.

aqui esta a decisao de arquitetura que importa, e ela é puramente matematica. um
blur gaussiano 2D de raio r, feito direto, amostra um quadrado de r por r para
cada pixel. isso é O(n^2) em numero de amostras. mas o gaussiano é separavel:
borrar na horizontal e depois na vertical da o mesmo resultado que borrar nas
duas dimensoes de uma vez. dois passes 1D de 13 amostras custam 26 amostras por
pixel. um passe 2D do mesmo raio custaria 13 por 13, 169 amostras. o
`task-08-done.md` registra isso de forma seca: "blur separável (h+v) é o(n) vs.
o(n²) para kernel direto". 26 contra 169 é a diferenca entre um efeito viavel em
tempo real e um que engasga.

o passe horizontal le a textura source e escreve numa textura temporaria. o
passe vertical le essa temporaria e escreve numa segunda. o resultado é o blur
completo. o `apply_blur` orquestra os dois:

```rust
pub(crate) fn apply_blur(&self, ctx: &mut EffectContext<'_>, sigma: f32) -> TextureHandle {
    let weights = gaussian_weights(sigma);
    let texel_size = [1.0 / ctx.width as f32, 1.0 / ctx.height as f32];

    let temp_a = ctx
        .pool
        .acquire(ctx.device, ctx.width, ctx.height, self.surface_format);
    let temp_b = ctx
        .pool
        .acquire(ctx.device, ctx.width, ctx.height, self.surface_format);

    // Horizontal pass: source -> temp_a
    // ... begin_render_pass em temp_a, set direction [1.0, 0.0], draw(0..3)

    // Vertical pass: temp_a -> temp_b
    // ... begin_render_pass em temp_b, set direction [0.0, 1.0], draw(0..3)

    ctx.pool.release(temp_a);
    temp_b
}
```

repara no fim: `temp_a` volta para o pool, `temp_b` é devolvido como handle. a
textura intermediaria do passe horizontal nao serve para mais nada depois que o
vertical leu dela, entao ela é liberada na hora. o `temp_b` carrega o resultado
e vira a entrada do proximo efeito na cadeia, ou do composite.

agora a parte sutil que prometi. cada passe de blur cria seu proprio buffer de
uniform, na hora, e nao reusa um buffer compartilhado. o comentario em `apply.rs`
explica por que, e é o tipo de defeito que esse repo já pagou uma vez:

```rust
/// One-shot uniform buffer + bind group for a blur pass. Transient on
/// purpose: `queue.write_buffer` stages ALL writes before the next
/// submit executes any pass, so reusing one buffer across the H and V
/// passes (or across several blurs in one frame -- e.g. backdrop
/// nodes) would make every pass read the LAST write.
```

o `queue.write_buffer` no wgpu nao escreve na memoria da GPU na hora que voce
chama. ele agenda a escrita, e todas as escritas agendadas pousam antes do
proximo submit executar qualquer pass. se voce reusasse um buffer entre o passe
horizontal e o vertical, escrevendo `direction = [1,0]` e depois `[0,1]` no
mesmo buffer, os dois passes leriam `[0,1]`, porque a ultima escrita venceu. os
dois passes virariam dois passes verticais. a imagem ficaria borrada só na
vertical, e voce passaria uma tarde olhando o shader procurando bug onde nao tem.
a correcao é criar um buffer por passe. transiente de proposito.

vale uma nota de honestidade aqui. o `kdb/adr/effects-architecture.md`, na lista
de cuidados, diz que "blur uniform buffer é compartilhado entre h e v pass via
write_buffer entre passes". isso contradiz o codigo de hoje em `apply.rs`, que
cria buffer por passe exatamente para nao compartilhar. o adr esta defasado em
relacao ao codigo. nao confirmado qual veio primeiro, mas o codigo é a verdade
corrente e o comentario dele documenta a razao. registro a divergencia em vez de
escolher um dos dois calado.

## a sombra é uma silhueta borrada

a sombra reusa quase tudo do blur. ela tem só um passe a mais na frente: a
extracao de silhueta. o `shadow.wgsl` pega o canal alpha da textura source,
multiplica pela cor e pelo alpha da sombra, e escreve uma silhueta tingida:

```rust
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let src = textureSample(source_texture, source_sampler, in.uv);
    let alpha = src.a * shadow.color.a;
    // Linearize the sRGB shadow tint, then premultiplied output.
    return vec4<f32>(srgb_to_linear(shadow.color.rgb) * alpha, alpha);
}
```

a forma da sombra vem do alpha da textura, nao da cor. onde o botao é opaco, a
sombra é cheia; onde é transparente, a sombra some. a cor vem do uniform
`shadow.color`, que entra em sRGB e é linearizada ali no shader antes de virar
output. isso fecha com a regra que o capitulo de cor já estabeleceu: cor entra em
sRGB e é linearizada uma vez ao cruzar para a GPU. a sombra obedece a mesma
regra. e o output já sai premultiplicado, `rgb * alpha`, que é o que o composite
espera receber.

depois da extracao, a sombra simplesmente reusa o `apply_blur`. o `apply_shadow`
extrai a silhueta numa textura do pool, e se o sigma é maior que zero, borra
essa silhueta com o mesmo caminho de dois passes do blur comum:

```rust
// Step 2: Blur the silhouette
if sigma > 0.0 {
    let sil_view = silhouette.view().clone();
    let blurred = self.apply_blur(
        &mut EffectContext {
            device: ctx.device,
            queue: ctx.queue,
            encoder: ctx.encoder,
            pool: ctx.pool,
            source_view: &sil_view,
            width: ctx.width,
            height: ctx.height,
        },
        sigma,
    );
    ctx.pool.release(silhouette);
    blurred
} else {
    silhouette
}
```

no total a sombra usa tres passes: um de extracao e dois de blur. e o ponto de
performance é o mesmo do começo do capitulo. a sombra nao re-renderiza o botao.
ela pega o alpha da textura que já existe, tinge, borra. o desenho da cena
aconteceu uma vez, e a sombra é derivada barata daquele unico desenho.

um detalhe de Rust aparece nesse trecho e merece nome. o `silhouette.view().clone()`.
o `apply_blur` precisa de uma referencia emprestada da view source, mas tambem
precisa do `ctx.pool` mutavel para pegar as duas texturas temporarias. a
silhueta vive no pool. se voce passasse `silhouette.view()` direto, estaria
emprestando do pool ao mesmo tempo que precisa dele mutavel, e o borrow checker
te para. clonar a `TextureView` resolve o conflito de borrow, e em wgpu 28 esse
clone é barato porque a view é reference-counted, o clone só incrementa um
contador. o adr registra isso na lista de cuidados, e é uma daquelas linhas que
parece detalhe e na verdade é a unica forma do codigo compilar.

## o composite pass desenha por cima

o ultimo pipeline pega o resultado do efeito e o desenha na surface. o
`composite.wgsl` é o mais simples dos tres: amostra a textura da camada e
multiplica pela opacidade.

```rust
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(layer_texture, layer_sampler, in.uv);
    // color is already premultiplied -- just scale by opacity
    return color * opacity;
}
```

a cor já chega premultiplicada, entao a opacidade é um escalar que multiplica
todos os canais de uma vez, RGB e alpha juntos. nada de tratar alpha separado. é
aqui que o blending entra, e o blending é a outra metade da historia. o pipeline
de composite é o unico dos tres construido com `BlendState` explicito:

```rust
Some(wgpu::BlendState {
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
}),
```

`One / OneMinusSrcAlpha` é o blending premultiplicado classico. o fator de source
é `One` porque a cor já vem multiplicada pelo seu proprio alpha, entao voce nao
precisa multiplicar de novo no blend. o fator de destino é `OneMinusSrcAlpha`, o
que sobrou de transparencia da camada de cima. blur e shadow nao tem blend
proprio, eles escrevem direto na textura temporaria com `LoadOp::Clear` em
transparente. só o composite mistura com o que já esta na tela. essa separacao é
limpa: os efeitos produzem texturas isoladas, e o unico ponto que toca a surface
de verdade é o composite.

os tres pipelines compartilham o mesmo construtor de tela cheia,
`create_fullscreen_pipeline`. blur e shadow passam `None` para o blend, o
composite passa o `BlendState` acima. um vertex state sem buffer, topologia de
triangulo, sem depth, sem multisample. é a forma mais enxuta de pipeline que da
para ter no wgpu, e os tres efeitos sao variacoes em cima dela.

## o pool que se recusa a liberar memoria

cada blur pega duas texturas temporarias. cada sombra pega tres. numa tela com
varias camadas com efeito, isso é dezenas de texturas por frame. se a engine
alocasse e destruisse textura de GPU toda vez, o custo de alocacao sozinho
mataria o frame rate. alocar memoria de GPU nao é como dar um `malloc`, envolve o
driver, e fazer isso dezenas de vezes por frame, sessenta frames por segundo, é
um caminho garantido para stutter.

a resposta é o `TexturePool`. grow-only. ele cresce conforme a demanda e nunca
destroi textura em regime permanente. a estrutura é um mapa de chave para uma
lista de entradas:

```rust
pub struct TexturePool {
    entries: FxHashMap<TextureKey, Vec<PoolEntry>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TextureKey {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}
```

a chave é `(largura, altura, formato)`. duas texturas com a mesma chave sao
intercambiaveis, entao o pool guarda uma lista delas por chave e entrega a
primeira que estiver livre. o `acquire` é direto:

```rust
pub fn acquire(
    &mut self,
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> TextureHandle {
    let key = TextureKey { width, height, format };
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

    // nenhuma livre: cria uma nova, marca in_use, devolve
    // usage: RENDER_ATTACHMENT | TEXTURE_BINDING
    // ...
}
```

procura uma entrada livre com aquela chave. achou, marca como em uso e devolve.
nao achou, cria uma textura nova, com `RENDER_ATTACHMENT | TEXTURE_BINDING` no
usage, porque toda textura de efeito é alvo de render num passe e fonte de
amostra no proximo. o `release` é mais simples ainda: marca a entrada como livre
de novo, sem destruir nada. a textura fica residente, esperando o proximo
`acquire` com a mesma chave.

esse é o ponto do grow-only. no primeiro frame que usa um blur, o pool aloca as
texturas e loga a criacao. nos frames seguintes, com a mesma geometria de tela,
o `acquire` sempre encontra uma textura livre da chave certa e a reusa. zero
alocacao em regime permanente. o adr diz isso de forma direta: "grow-only: nunca
destrói texturas". o comentario no topo do arquivo é ainda mais seco: "In steady
state, zero allocations".

o `TextureHandle` guarda um clone da `TextureView`, nao uma referencia para
dentro do pool. de novo o motivo é o borrow checker. se o handle emprestasse a
view do pool, voce nao conseguiria pegar uma segunda textura sem soltar a
primeira, porque o segundo `acquire` precisa do pool mutavel. clonando a view, o
handle fica independente do pool, e o `apply_blur` consegue segurar `temp_a` e
`temp_b` ao mesmo tempo. o clone barato de view do wgpu 28 é o que faz esse
desenho funcionar.

grow-only tem um custo, e o codigo é honesto sobre ele. o pool nunca encolhe
sozinho. se a janela for redimensionada, todas as texturas com o tamanho antigo
viram lixo residente, ocupando memoria que ninguem vai pedir de novo. por isso
existe o `invalidate_size`, chamado no resize:

```rust
/// Drop textures that don't match the given surface dimensions.
pub fn invalidate_size(&mut self, width: u32, height: u32) {
    self.entries.retain(|key, entries| {
        if key.width != width || key.height != height {
            let any_in_use = entries.iter().any(|e| e.in_use);
            if any_in_use {
                log::warn!(
                    "TexturePool: cannot invalidate {}x{} -- texture still in use",
                    key.width, key.height
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

no resize, ele retem só as texturas que batem com o novo tamanho e descarta o
resto, com um cuidado: se uma textura de tamanho antigo ainda esta em uso, ele
nao descarta e loga um warning. melhor vazar uma textura por um frame do que
puxar o tapete de baixo de um pass que ainda esta lendo dela. grow-only durante o
frame, encolhe só na fronteira do resize, e só o que da para encolher com
seguranca.

tem ainda um `memory_bytes` que soma os bytes residentes de todas as texturas do
pool, em uso ou nao, e alimenta o monitor de performance. ele multiplica largura
por altura por bytes-por-pixel do formato pela contagem de entradas. é a forma da
engine saber quanto o pool esta segurando, e o numero aparece no overlay de
debug junto com o resto das stats de memoria.

## o render loop e o enum LayerEffect

falta amarrar isso ao loop de render. quem decide aplicar um efeito numa camada é
o `apply_layer_effects`, em `window/render_passes.rs`. ele varre as camadas
visiveis que tem efeito, e para cada uma encadeia os efeitos:

```rust
for effect in effects {
    let sv = current_view_owner
        .as_ref()
        .map(|h| h.view())
        .unwrap_or(source_view);
    let handle = match effect {
        LayerEffect::Blur { sigma } => effect_processor.apply_blur(
            &mut crate::effects::EffectContext { /* device, queue, encoder, pool, sv, sw, sh */ },
            *sigma,
        ),
        LayerEffect::Shadow { sigma, color } => effect_processor.apply_shadow(
            &mut crate::effects::EffectContext { /* ... */ },
            *sigma,
            *color,
        ),
    };
    if let Some(prev) = current_view_owner.take() {
        texture_pool.release(prev);
    }
    current_view_owner = Some(handle);
}
```

o `current_view_owner` é o truque de encadeamento. o primeiro efeito le da
textura da camada. o segundo le da saida do primeiro. cada efeito devolve um
handle que vira a entrada do proximo, e o handle anterior volta para o pool assim
que o proximo ja foi computado. uma camada com blur seguido de outro efeito flui
por essa cadeia sem alocacao extra, reusando o pool a cada elo. no fim, o ultimo
handle vira um bind group que entra na lista de resultados de efeito, e o
desenho final na surface acontece depois, no `encode_composite_pass`.

aqui preciso ser exato sobre o enum, porque tem uma armadilha de nomeacao no
repo. o `match` acima casa com `LayerEffect::Blur` e `LayerEffect::Shadow`. esse
`LayerEffect` é o do compositor, definido em `compositor/layer/mod.rs`:

```rust
#[derive(Clone, Debug)]
pub enum LayerEffect {
    Blur { sigma: f32 },
    Shadow { sigma: f32, color: [f32; 4] },
}
```

dois braços: blur e shadow. é esse o enum que o render loop consome, é esse que
o `set_layer_effects` recebe, e é esse que os testes do compositor exercitam.

mas existe um segundo `LayerEffect`, no modulo de efeitos, em `effects/types.rs`,
e ele é diferente:

```rust
#[derive(Clone, Debug)]
pub enum LayerEffect {
    Blur { sigma: f32 },
    DropShadow { offset_x: f32, offset_y: f32, sigma: f32, color: [f32; 4] },
    Opacity { alpha: f32 },
}
```

tres braços: blur, drop shadow com offset, e opacity. esse enum tem mais
expressividade, offset de sombra e opacidade como efeito de primeira classe. mas
ele nao é o que o render loop usa. uma busca pelo repo nao acha nenhuma
referencia a esse `effects::LayerEffect` fora do proprio arquivo onde ele é
declarado. ele é exportado pelo `mod.rs` do modulo de efeitos via `pub use
types::*`, e fica ali, definido e nao consumido pelo caminho de render.

nao confirmado o motivo da divergencia. a leitura mais provavel, olhando o
`task-08-done.md`, é que o enum de `types.rs` é o desenho original da task-08,
que listava blur, drop shadow e opacity como os tres efeitos planejados, e o
`LayerEffect` do compositor é a versao que de fato entrou no render loop, com
shadow sem offset separado e com opacity tratada por outro caminho, o
`layer.opacity` que alimenta o composite. os dois enums coexistem hoje, um usado
e um nao. registro isso porque o capitulo prometeu explicar o enum do render loop,
e a verdade é que tem dois com o mesmo nome, e só um deles esta no loop. quem for
limpar isso depois precisa saber que sao dois.

## o numero que ainda nao existe

todo capitulo desse livro tenta fechar num numero de benchmark. esse nao
consegue, e prefiro dizer do que inventar. nao existe benchmark de efeitos no
repo. os benches que existem cobrem rope, parser, lot, codec e o scene_build do
compositor. nenhum mede blur, shadow ou composite.

o `task-08-done.md` tem uma meta no checklist, "performance: blur em resolução
1080p < 2ms por frame (fase f)", e essa linha esta marcada com `[ ]`, nao feita.
é alvo, nao medida. entao o numero que esse subsistema deveria ter, o custo real
de um blur de tela cheia em 1080p, ainda nao foi capturado neste repo. nao
confirmado se fica abaixo de 2ms. o desenho aponta para sim: dois passes de 13
amostras, pool sem alocacao em regime permanente, pesos prontos do CPU. mas
aponta nao é mede, e seria desonesto trocar um pelo outro.

o que da para afirmar com ancora é a estrutura, e a estrutura é coerente. o
trabalho caro, criar pipeline e alocar textura, acontece uma vez. o trabalho por
frame é encadear passes de tela cheia que reusam texturas que ja existem. a parte
do subsistema que ainda falta nao é codigo, é medicao. quando alguem escrever o
bench da fase f, esse capitulo ganha o paragrafo que esta faltando aqui.

uma ultima nota de versao, conferida contra o `Cargo.toml` do workspace: wgpu 28,
winit 0.30, cosmic-text 0.18, taffy 0.9 sao as quatro deps de peso da engine. o
`bytemuck 1` é o que move os uniforms para a GPU com layout estavel, o `rustc-hash
2.1` da o `FxHashMap` que indexa o pool, e o `pollster 0.4` aparece só no caminho
de hot-reload de shader, para bloquear no `pop` do error scope quando um shader
recarregado falha de validar. nenhum desses numeros foi suposto, todos saem do
manifesto.

---

## rastros

cada afirmacao tecnica deste capitulo, com file:line. conferido contra a arvore
em que escrevi (branch `refactor/workspace-restructure`).

processador e pipelines (crate engine)
- `crates/engine/src/effects/processor.rs:9` (`EffectProcessor`, campos: tres
  pipelines, quatro BGLs, sampler linear, buffer de composite, surface_format)
- `crates/engine/src/effects/processor.rs:23` (`new`, constroi sampler
  ClampToEdge + linear, BGLs group 0 textura+sampler e group 1 uniform)
- `crates/engine/src/effects/processor.rs:126` (comentario: blur/shadow uniforms
  transientes por causa do staged `write_buffer`; só composite persiste)
- `crates/engine/src/effects/pipelines.rs:8` (`create_blur_pipeline`)
- `crates/engine/src/effects/pipelines.rs:34` (`create_shadow_pipeline`)
- `crates/engine/src/effects/pipelines.rs:60` (`create_effect_composite_pipeline`,
  blend `One / OneMinusSrcAlpha` em color e alpha)
- `crates/engine/src/effects/pipelines.rs:170` (`create_fullscreen_pipeline`, vs
  sem buffer, triangle list, sem depth/multisample)

tipos e pesos
- `crates/engine/src/effects/types.rs:6` (`LayerEffect` do modulo de efeitos:
  Blur, DropShadow, Opacity; nao referenciado fora do arquivo)
- `crates/engine/src/effects/types.rs:27` (`BlurUniforms`, direction, texel_size,
  `[f32; 16]` = 13 pesos + 3 padding)
- `crates/engine/src/effects/types.rs:52` (`gaussian_weights`, 13-tap, centro + 6
  simetricos, normalizado para somar 1.0)
- `crates/engine/src/effects/tests.rs:4` (soma 1.0 para sigma 0.5..6.0), `:16`
  (sigma 0 -> centro 1.0), `:25` (decaimento monotonico), `:41` (padding zero)

aplicacao dos passes
- `crates/engine/src/effects/apply.rs:50` (comentario: buffer de blur transiente
  de proposito; reuso vazaria a ultima escrita para todos os passes)
- `crates/engine/src/effects/apply.rs:71` (`apply_blur`, dois passes H e V, pega
  temp_a e temp_b do pool, libera temp_a, devolve temp_b)
- `crates/engine/src/effects/apply.rs:155` (`apply_shadow`, extracao de silhueta +
  blur; `silhouette.view().clone()` para evitar borrow conflict, linha 209)
- `crates/engine/src/effects/apply.rs:230` (`composite_pass`, escreve alpha no
  buffer persistente, draw 0..3)

shaders WGSL
- `crates/engine/src/gpu/shaders/blur.wgsl:8` (`weights: array<vec4<f32>, 4>`)
- `crates/engine/src/gpu/shaders/blur.wgsl:25` (`vs_main`, triangulo de tela cheia
  via `vertex_index`, sem vbo)
- `crates/engine/src/gpu/shaders/blur.wgsl:47` (`fs_main`, centro + loop 1..6,
  pares simetricos, 13 amostras)
- `crates/engine/src/gpu/shaders/shadow.wgsl:32` (`srgb_to_linear`), `:39`
  (`fs_main`, alpha = src.a * shadow.color.a, output premultiplicado linearizado)
- `crates/engine/src/gpu/shaders/composite.wgsl:30` (`fs_main`, `color * opacity`,
  cor já premultiplicada)

pool de texturas
- `crates/engine/src/gpu/texture_pool.rs:1` (comentario: grow-only, steady state
  zero allocations)
- `crates/engine/src/gpu/texture_pool.rs:32` (`TexturePool`, `FxHashMap<TextureKey,
  Vec<PoolEntry>>`; `TextureKey` = width, height, format, linha 5)
- `crates/engine/src/gpu/texture_pool.rs:49` (`acquire`, reusa livre ou cria),
  `:85` (usage `RENDER_ATTACHMENT | TEXTURE_BINDING`)
- `crates/engine/src/gpu/texture_pool.rs:112` (`release`, marca in_use=false sem
  destruir)
- `crates/engine/src/gpu/texture_pool.rs:122` (`memory_bytes`, alimenta o monitor)
- `crates/engine/src/gpu/texture_pool.rs:135` (`invalidate_size` no resize, retem
  só o tamanho novo, warning se ainda em uso)

render loop e o enum
- `crates/engine/src/compositor/layer/mod.rs:23` (`LayerEffect` do compositor:
  Blur { sigma }, Shadow { sigma, color }; é o que o render loop casa)
- `crates/engine/src/window/render_passes.rs:371` (`apply_layer_effects`), `:402`
  (`match effect`), `:403` (Blur -> apply_blur), `:415` (Shadow -> apply_shadow),
  `:429` (encadeia via current_view_owner, libera o anterior)
- `crates/engine/src/compositor/layer_ops.rs:43` (`set_layer_effects`)
- `crates/engine/src/compositor/tests.rs:147` (teste de set_layer_effects com
  Blur sigma 8.0)

adr e tracking
- `kdb/adr/effects-architecture.md` (pipeline fragment-only, blur separavel 2
  passes, shadow 3 passes, composite premultiplicado, texturepool grow-only,
  gaussian 13-tap, bind groups group 0/1, max 2 dentro do limite WASM de 4)
- `kdb/mission/steps/checked/task-08-done.md` (decisao fragment shader only; o(n)
  vs o(n^2); meta `< 2ms` em 1080p marcada `[ ]`, nao feita)

versoes (conferidas contra o Cargo.toml)
- `Cargo.toml:50` wgpu 28, `:51` winit 0.30, `:54` cosmic-text 0.18, `:55` taffy
  0.9, `:52` bytemuck 1, `:68` rustc-hash 2.1, `:83` pollster 0.4

nao confirmado
- divergencia de enum: `effects/types.rs:6` define `LayerEffect` com Blur,
  DropShadow e Opacity, mas o render loop em `render_passes.rs:402` casa com o
  `LayerEffect` do compositor (`compositor/layer/mod.rs:23`), que tem só Blur e
  Shadow. nenhuma referencia a `effects::LayerEffect` fora de `types.rs`. os dois
  coexistem; o motivo da divergencia nao foi confirmado.
- o adr `effects-architecture.md:49` afirma que o buffer de uniform do blur é
  compartilhado entre os passes H e V. o codigo em `apply.rs:50-68` cria buffer
  transiente por passe exatamente para nao compartilhar. o adr esta defasado em
  relacao ao codigo; qual veio primeiro nao foi confirmado.
- nao existe benchmark de efeitos no repo (os benches sao rope, parser, lot,
  codec, scene_build). o custo real de um blur 1080p por frame nao foi medido
  aqui; o `< 2ms` é alvo nao cumprido da fase f, nao medida.
- o task-08-done.md lista os arquivos originais como `src/effects.rs` e
  `src/texture_pool.rs`; hoje o modulo esta dividido em
  `effects/{processor,types,pipelines,apply,tests,mod}.rs` e o pool em
  `gpu/texture_pool.rs`. a modularizacao aconteceu depois da task; o commit exato
  da divisao nao foi rastreado neste capitulo.
