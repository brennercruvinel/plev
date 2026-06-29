---
title: "gpu: device, surface, pipelines"
parte: 2
status: rascunho
rastros:
  - crates/engine/src/gpu/context.rs
  - crates/engine/src/gpu/surface.rs
  - crates/engine/src/gpu/pipelines.rs
  - crates/engine/src/gpu/utils.rs
  - crates/engine/src/gpu/config.rs
  - crates/engine/src/window/render.rs
  - kdb/adr/render-into-an-srgb-view-format.md
  - kdb/adr/benchmark-results.md
  - crates/engine/benches/scene_build.rs
  - Cargo.toml
---

# gpu: device, surface, pipelines

quando voce abre uma janela e ela mostra um retangulo cinza, parece que nao
aconteceu quase nada. abriu, pintou, pronto. mas entre o `cargo run` e o
primeiro pixel na tela tem uma conversa inteira acontecendo, e ela e mais
parecida com passar numa alfandega do que com chamar uma funcao. o seu
programa chega na fronteira da placa de video sem saber nada sobre ela: nao
sabe qual placa e, nao sabe que formatos de cor ela aceita, nao sabe se da pra
sincronizar com o monitor ou nao. ele precisa perguntar tudo, aceitar o que a
placa oferece, e so entao montar as ferramentas de desenho. essa conversa toda
mora numa camada do plev de tres palavras: device, surface, pipelines.

esse capitulo desmonta essa camada. e ele importa porque e o unico lugar do
engine onde o codigo conversa direto com o hardware. tudo que vem depois, o
compositor, o texto, o layout, desenha por cima de abstracoes. aqui em baixo
ainda nao tem abstracao, tem negociacao. o engine pede uma placa, a placa diz
o que tem, o engine configura uma superficie de desenho com o que a placa
permitiu, e por fim cria os pipelines, que sao os programas de desenho ja
compilados e prontos pra rodar milhares de vezes por segundo. eu vou abrir
isso no acessivel, descer pro codigo real de `crates/engine/src/gpu/`, e parar
de proposito num detalhe que parece bobo e e o defeito mais caro que essa
camada ja pagou: como criar o render target. no fim eu trago o numero de capa
do projeto, os 159 a 222 milhoes de rects por segundo, e sou honesto sobre o
que esse numero mede e o que ele nao mede nesta camada.

a versao da biblioteca grafica e a wgpu 28, conferida no `Cargo.toml`. a winit
0.30 cuida da janela, mas quem fala com a GPU e a wgpu, e e a API dela que
aparece em quase toda linha daqui pra frente.

## quatro substantivos antes de qualquer codigo

vale firmar quatro palavras antes, porque elas se repetem o capitulo inteiro e
e facil confundir.

o `Instance` e o ponto de entrada da wgpu. ele e quem enumera os backends
disponiveis (metal no mac, vulkan no linux, webgpu no browser) e de onde sai
tudo o mais. voce cria um, e a partir dele pede uma superficie e um adaptador.

o `Adapter` e o handle de uma GPU fisica vista por um backend. e ele que
carrega as capacidades: quais formatos de cor, quais modos de apresentacao,
qual o nome da placa. pedir um adaptador e perguntar "qual GPU eu vou usar".

o `Device` e a GPU ja aberta pra trabalho, mais a `Queue`, que e a fila por
onde voce manda comandos e uploads pra ela. device e queue andam juntos, vem
do mesmo pedido, e sao recursos globais: um por aplicacao, emprestados pra quem
precisar deles. nenhuma parte do engine possui o device, todas pegam ele por
referencia na hora que vao desenhar.

a `Surface` e a ponte entre a GPU e a janela concreta da winit. e nela que a
imagem final aparece. a surface tem um formato de cor, um tamanho, um modo de
apresentacao, e ela e a unica peca dessa lista que pode morrer e renascer no
meio da vida do programa, porque a janela embaixo dela pode sumir (android
suspende o app) e voltar. guarda essa, ela explica metade das decisoes de
design da camada.

com os quatro firmados, da pra olhar a estrutura que segura todos eles.

## o GpuContext, tudo num lugar so

o engine junta device, queue, surface, a configuracao da surface e todos os
pipelines num struct unico, o `GpuContext`, em
`crates/engine/src/gpu/context.rs`. ele e grande, mas o grande dele e
honesto: e uma lista de recursos de GPU que vivem o programa inteiro. os
campos do topo sao estes:

```rust
pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Option<wgpu::Surface<'static>>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub config: RenderConfig,
    pub projection_buffer: wgpu::Buffer,
    pub projection_bind_group_layout: wgpu::BindGroupLayout,
    pub projection_bind_group: wgpu::BindGroup,
    pub quad_pipeline: wgpu::RenderPipeline,
    pub rect_sdf_pipeline: wgpu::RenderPipeline,
    pub shadow_analytic_pipeline: wgpu::RenderPipeline,
    pub text_pipeline: wgpu::RenderPipeline,
    // ...
}
```

duas coisas saltam logo. a primeira: a `surface` e o unico recurso embrulhado
em `Option`. device, queue, os pipelines, todos sao posse direta, sempre
presente. a surface e `Option<wgpu::Surface<'static>>`, e o `None` nao e um
estado de erro, e um estado de vida. quando o app suspende, o engine larga a
surface e ela vira `None`. quando volta, recria. todo o resto sobrevive, so a
ponte com a janela e que vai e volta. esse `Option` e uma decisao registrada
no tipo: o engine declara, na assinatura do campo, que a superficie e mortal e
o device nao.

a segunda: o `'static` no tipo da surface. `wgpu::Surface<'static>` quer dizer
que a surface nao empresta a janela por um lifetime curto, ela segura a janela
por um `Arc<Window>` que dura o programa todo. isso e o que permite guardar a
surface dentro de um struct de vida arbitraria sem briga de lifetime, e e a
razao de o construtor receber `Arc<Window>` em vez de `&Window`.

o resto dos campos sao os pipelines (um por tipo de coisa que o engine
desenha: quad, rect com borda arredondada via SDF, sombra, texto, imagem,
backdrop, composite) e os recursos compartilhados que esses pipelines usam: o
buffer de projecao, os bind group layouts, o sampler de composicao. tudo
criado uma vez, no `new`, e reusado todo frame. essa e a aposta central da
camada, e eu volto nela no fim: o caro acontece uma vez, no startup. o loop de
desenho so amarra recursos ja prontos.

## a alfandega: instance, adapter, device

a construcao inteira mora em `GpuContext::new_with_config`. o `new` publico so
chama ele com a config padrao:

```rust
pub async fn new(window: Arc<Window>) -> Self {
    Self::new_with_config(window, RenderConfig::default()).await
}
```

repara no `async`. abrir uma GPU e uma operacao assincrona, porque pedir
adaptador e pedir device sao chamadas que o driver pode demorar pra responder,
e a wgpu modela isso com `.await`. no desktop o engine resolve isso com um
executor minimo (a pollster, 0.4 no `Cargo.toml`); no browser o `.await` cai
direto no event loop do JS. mas o ponto pra ja: a primeira coisa que o
construtor faz nao e tocar a GPU, e ajustar a config.

```rust
config.msaa_samples = config.effective_msaa_samples();
crate::path::set_default_tolerance(config.path_tolerance);
```

`effective_msaa_samples` esta em `crates/engine/src/gpu/config.rs` e e um
porteiro pequeno mas importante: a wgpu garante 1 ou 4 amostras de
multisampling, e qualquer outro valor que o app peca cai pra 4 com um warn. o
default e 4 amostras, vsync ligado, tolerancia de tesselacao 0.1. essas tres
escolhas sao o comportamento historico do engine, congelado no `Default` pra
ninguem ter que adivinhar. o engine clampa o valor antes de qualquer alocacao
porque o numero de amostras vai entrar na criacao de todos os pipelines la na
frente, e um pipeline criado com 2 amostras simplesmente nao seria valido.
melhor consertar na entrada do que descobrir no meio.

so depois disso o codigo passa a fronteira. ele cria o `Instance` e ja faz a
primeira bifurcacao por plataforma:

```rust
let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
    #[cfg(not(target_arch = "wasm32"))]
    backends: wgpu::Backends::PRIMARY,
    #[cfg(target_arch = "wasm32")]
    backends: wgpu::Backends::BROWSER_WEBGPU,
    ..Default::default()
});
```

essa e uma das pouquissimas bifurcacoes `#[cfg]` por plataforma na camada, e
ela e inevitavel: no desktop o engine quer os backends primarios (metal,
vulkan, dx12), no browser so existe um caminho, o WebGPU do navegador. tudo o
que da pra fazer uma vez so, o engine faz uma vez so, e essa e uma das que nao
da. guarda o contraste, porque a decisao do sRGB la na frente e exatamente o
oposto: um problema que parecia exigir um `#[cfg]` e que o engine resolveu sem
nenhum.

com o instance na mao, ele cria a surface e pede o adaptador:

```rust
let surface = instance.create_surface(window).unwrap();
let adapter = instance
    .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    })
    .await
    .expect("Failed to find a suitable GPU adapter");
```

tem ordem aqui que parece estranha na primeira leitura: a surface nasce antes
do adaptador. por que pedir a ponte com a janela antes de saber qual placa vai
desenhar nela? porque o pedido de adaptador usa a surface como filtro.
`compatible_surface: Some(&surface)` diz pra wgpu "so me de uma GPU que
consiga desenhar nessa superficie aqui". em maquina com duas placas, isso
elimina a que nao fala com a janela. o `power_preference` pede a de alta
performance (a dedicada, num laptop com placa integrada e dedicada), e o
`force_fallback_adapter: false` recusa o adaptador de software, aquele
emulado em CPU que existe so como ultimo recurso. o engine prefere falhar a
desenhar numa GPU emulada, e o `.expect` deixa isso explicito: se nao tem
placa de verdade compativel, o programa para aqui com uma mensagem clara, nao
arrasta um renderer fantasma.

de posse do adaptador, o pedido do device:

```rust
let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor {
        label: Some("plev_device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: Default::default(),
        experimental_features: Default::default(),
    })
    .await
    .expect("Failed to create device");
```

(no codigo real os `required_limits` aparecem duas vezes, atras de `#[cfg]`,
mas hoje as duas pontas pedem `wgpu::Limits::default()`; deixei uma so aqui pra
nao distrair.)

esse descriptor e uma declaracao de modestia, e ela e proposital. `Features::empty()`
quer dizer que o engine nao pede nenhuma feature opcional de GPU, nada de
recurso exotico que so algumas placas tem. `Limits::default()` pede os limites
mais conservadores que a wgpu garante em qualquer lugar, inclusive no WebGL de
um celular antigo. esse par, zero features e limites default, e a base de toda
a promessa de portabilidade do projeto: um codigo que so usa o que a placa mais
fraca da lista oferece roda na placa mais forte sem mudar nada. o
`MemoryHints::Performance` e a unica dica de luxo, e e so uma dica, diz pro
alocador da wgpu priorizar velocidade sobre uso de memoria. o `label`
`plev_device` e o nome que aparece nas ferramentas de profiling de GPU, do
mesmo jeito que os labels dos buffers e dos pipelines: quando voce abre um
xcode ou um renderdoc e ve a arvore de recursos, esse nome esta la.

quando essas tres chamadas voltam, o engine tem o que precisava da fronteira:
uma placa real, aberta, com uma fila de comandos. a alfandega passou. agora
falta configurar onde a imagem vai aparecer.

## a surface e a escolha do formato

o adaptador sabe quais formatos de cor a surface aceita, e a primeira pergunta
que o engine faz a ele e essa lista:

```rust
let surface_caps = surface.get_capabilities(&adapter);
let surface_format = surface_caps
    .formats
    .iter()
    .find(|f| f.is_srgb())
    .copied()
    .unwrap_or(surface_caps.formats[0]);
```

ele varre os formatos oferecidos, pega o primeiro que e sRGB, e cai no primeiro
da lista se nenhum for. esse `find(is_srgb)` parece um detalhe e e o coracao de
todo o pipeline de cor do engine. eu vou voltar ao porque na proxima secao,
mas ja adianto o problema: cor no plev e sRGB na entrada e linear dentro da
GPU, e a conversao de uma pra outra acontece de graca na hora de escrever na
tela, mas so se o formato da escrita for um formato sRGB. se o engine escrever
numa superficie que nao e sRGB, a conversao some, e a tela inteira sai escura.

no desktop esse `find` quase sempre acha um formato sRGB e acabou. no browser
nao acha, e e ai que entra a parte mais sutil da camada inteira:

```rust
let render_format = surface_format.add_srgb_suffix();
let view_formats = if render_format != surface_format {
    vec![render_format]
} else {
    vec![]
};
```

`add_srgb_suffix()` pega um formato como `bgra8unorm` e devolve a variante sRGB
dele, `bgra8unorm-srgb`. no desktop, onde `surface_format` ja e sRGB, essa
funcao e a identidade: o formato volta igual, `render_format == surface_format`,
e `view_formats` fica vazio. no browser, onde `surface_format` e a versao crua,
`render_format` vira a versao sRGB, e `view_formats` passa a carregar essa
variante. esse vetor `view_formats` e o mecanismo: ele diz pra wgpu "essa
surface vai ser desenhada nao so pelo formato base, mas tambem por esse formato
sRGB que eu estou registrando aqui". registrar nao e usar ainda, e habilitar.

repara que esse trecho nao tem `#[cfg]`. nao tem "se for web faca assim, se for
desktop faca assado". tem uma unica expressao que, alimentada pelos formatos
reais que a placa ofereceu, faz a coisa certa em qualquer plataforma sozinha.
no desktop ela colapsa pra um no-op, no browser ela liga o caminho extra. essa
e a forma que o engine prefere e que aparece o livro inteiro: a diferenca de
plataforma expressa uma vez, no dado de configuracao, em vez de espalhada em
ramos por todo lugar.

## o render target so via surface_render_view

agora o defeito. e ele e tao instrutivo que tem um ADR so pra ele,
`kdb/adr/render-into-an-srgb-view-format.md`, com data de 2026-06-10.

o problema, contado pelo ADR: a API de canvas do WebGPU so aceita formatos de
surface que nao sao sRGB, `bgra8unorm` ou `rgba8unorm`. no desktop a surface ja
e sRGB, entao a conversao de linear pra sRGB que o pipeline inteiro assume
acontece sozinha, na escrita, sem ninguem pedir. no browser essa conversao foi
silenciosamente pulada: os valores linearizados eram escritos crus, e o fundo
da pagina, que deveria medir (48,48,48), saiu medindo (8,8,8). cor cerca de
duas vezes e meia mais escura do que o esperado. e o ADR e cirurgico ao
nomear: isso e o inverso exato do bug de gamma do desktop, produzido pela
mesma raiz, a suposicao de que a escrita encoda sozinha.

a decisao tem tres partes, e elas amarram a secao anterior com o codigo de
desenho. a surface e configurada com o formato base mais a variante sRGB no
`view_formats`. todos os pipelines, todas as texturas de layer do compositor, e
a view da surface usam o formato sRGB. e o jeito de garantir que a view da
surface usa o formato certo nao pode ser o jeito obvio. esse e o no.

primeiro, o engine expoe qual formato as render passes miram:

```rust
pub fn surface_format(&self) -> wgpu::TextureFormat {
    self.surface_config
        .view_formats
        .first()
        .copied()
        .unwrap_or(self.surface_config.format)
}
```

essa funcao devolve o formato da view (o sRGB) quando ele existe, e cai no
formato base da surface quando nao. no desktop, `view_formats` esta vazio,
entao ela devolve o formato base, que ja e sRGB. no browser, ela devolve a
variante sRGB que foi registrada. um metodo, uma resposta consistente pra todo
o resto do engine: os pipelines sao criados com esse formato, as texturas
intermediarias do compositor usam esse formato, e a view final da surface usa
esse formato. ninguem precisa saber em que plataforma esta.

segundo, e aqui mora a linha que o ADR mais protege, a criacao da view da
surface, em `crates/engine/src/gpu/surface.rs`:

```rust
pub fn surface_render_view(&self, output: &wgpu::SurfaceTexture) -> wgpu::TextureView {
    output.texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(self.surface_format()),
        ..Default::default()
    })
}
```

essa funcao, na linha 134 do arquivo, e a unica forma sancionada de criar um
render target a partir da textura da surface. ela cria a view passando
`format: Some(self.surface_format())` de proposito, forcando a view a usar o
formato sRGB mesmo quando a textura por baixo e crua. o doc comment dela e um
aviso direto a quem for mexer no codigo: um `texture.create_view(&Default::default())`
herda o formato proprio da textura (possivelmente nao-sRGB) e pula a codificacao
de gama em silencio, sempre passe por aqui pra render targets de surface.

e por que isso e tao perigoso a ponto de virar ADR? porque o caminho errado e
invisivel onde a maioria dos desenvolvedores testa. o ADR cristaliza isso na
secao "avoid": nunca chame `create_view(&Default::default())` numa textura de
surface em codigo novo. a falha e invisivel no desktop e catastrofica na web, o
que faz ela sobreviver ao code review. um revisor roda no mac, ve cor certa,
aprova. o bug so aparece no browser, longe de quem escreveu. e a outra regra do
"avoid" e a que amarra a filosofia da camada: nao faca um ramo por plataforma
pra consertar cor na web, o mecanismo de formato ja expressa a diferenca uma
vez, na configuracao. as consequencias do ADR registram o resultado: o fundo na
web mediu (8,8,8) antes e (48,48,48) depois, identico ao desktop, e os sete
sitios de chamada de render (a janela da engine, o showcase, o ide, o scene3d,
o snake game e os dois exemplos) foram migrados pra `surface_render_view`. um
caminho de codigo, todas as plataformas encodam igual.

da pra ver esse contrato fechando no codigo de desenho real, em
`crates/engine/src/window/render.rs`. todo frame, o engine pega a textura
corrente da surface e cria a view pelo metodo certo:

```rust
let output = match surface.get_current_texture() {
    Ok(t) => t,
    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
        gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
        return;
    }
    // ... Timeout, outros erros
};
let surface_view = gpu.surface_render_view(&output);
```

e quando o compositor vai resolver a cena, ele recebe `gpu.surface_format()`
como formato alvo, o mesmo formato que a view usa. a consistencia nao e mantida
por disciplina, e mantida por todo mundo perguntando a mesma fonte. repara
tambem no tratamento de `Lost` e `Outdated`: se a surface ficou invalida (a
janela mudou de tamanho, o sistema reconfigurou o display), o engine
reconfigura a surface chamando `resize` com o tamanho atual e desiste do frame.
nao tenta desenhar numa surface morta, reconfigura e espera o proximo frame.
essa graca com a surface invalida e a mesma postura do `Option` la no struct: a
surface e a peca mortal, e o codigo trata ela como mortal em todo ponto de
contato.

## a surface que morre e renasce

falei que a surface e o unico recurso que vai e volta. o arquivo
`crates/engine/src/gpu/surface.rs` e quase todo sobre esse ciclo de vida.

o `resize` e o caso comum, a janela mudou de tamanho:

```rust
pub fn resize(&mut self, width: u32, height: u32) {
    let width = width.max(1);
    let height = height.max(1);
    self.surface_config.width = width;
    self.surface_config.height = height;
    if let Some(ref surface) = self.surface {
        surface.configure(&self.device, &self.surface_config);
    }
    // Back to physical coordinates until the app re-applies its logical
    // projection (apps call `set_projection` right after `resize`).
    self.logical_size = None;
    let projection_data = ortho_projection(width as f32, height as f32);
    self.queue.write_buffer(
        &self.projection_buffer,
        0,
        bytemuck::cast_slice(&projection_data),
    );
}
```

duas coisas valem nota. o `.max(1)` em largura e altura: uma surface de
dimensao zero e invalida, e janelas minimizadas reportam zero. clampar pra 1 e
o jeito barato de nunca configurar uma surface degenerada. e o `if let Some` em
volta do `configure`: se a surface esta `None` (app suspenso), o resize so
atualiza os numeros guardados na config e nao toca em nada de GPU. quando a
surface voltar, ela ja nasce com o tamanho certo. de novo o `Option`
trabalhando, o codigo continua valido mesmo sem a ponte.

o resize tambem reescreve a matriz de projecao. a `ortho_projection`, em
`crates/engine/src/gpu/utils.rs`, monta uma projecao ortografica que mapeia
`[0, largura]` pra `[-1, 1]` no X e `[0, altura]` pra `[1, -1]` no Y, o Y pra
baixo que e a convencao de tela. e ela e escrita num buffer uniforme que todos
os vertex shaders leem. mudou o tamanho da surface, muda a projecao, e por isso
o resize toca os dois juntos. o detalhe do `logical_size = None` e sobre HiDPI
e fica pro capitulo de tokens e projecao, basta saber que o resize volta pra
coordenadas fisicas e o app reaplica a logica logo depois.

o caso interessante e o par suspende e resume, que so existe no mobile:

```rust
pub fn drop_surface(&mut self) {
    self.surface = None;
    log::info!("Surface dropped");
}
```

quando o android suspende o app, a janela nativa some, e segurar uma surface
apontando pra uma janela morta e exatamente o tipo de erro que trava o app no
resume. entao o engine larga a surface, ela vira `None`, e o device, a queue, e
todos os pipelines continuam vivos. nenhum recurso de desenho e perdido, so a
ponte. no resume, o `recreate_surface` reconstroi a ponte a partir de uma
janela nova, e o detalhe que importa e que ele repete a logica do sRGB:

```rust
let render_format = format.add_srgb_suffix();
self.surface_config.view_formats = if render_format != format {
    vec![render_format]
} else {
    vec![]
};
```

o comentario no codigo e literal: espelha `GpuContext::new_with_config`,
renderiza numa view sRGB mesmo quando o formato da surface nao pode ser sRGB.
ou seja, a regra de cor que custou um ADR nao mora so no construtor, ela mora
em todo lugar que cria ou recria surface. se o recreate esquecesse o
`add_srgb_suffix`, um app android que suspende e volta sairia com cor escura
depois do primeiro resume, e seria um bug de reproducao horrivel, "as cores
ficam erradas as vezes". por isso o caminho de recriacao carrega a mesma logica
em vez de confiar que alguem lembre.

## os pipelines, e por que sao quase todos iguais

device e surface prontos, falta o que de fato desenha: os pipelines. um
pipeline de render, na wgpu, e um objeto que junta um shader compilado, o
formato dos vertices que entram, o formato da cor que sai, o modo de mistura
(blend), o tipo de primitiva (triangulos), e a config de multisampling, tudo
validado e congelado num objeto que voce so amarra e usa. criar um pipeline e
caro, compila shader, valida layout. usar um pipeline e barato. por isso eles
nascem todos no `new`, uma vez, e vivem o programa inteiro.

o arquivo `crates/engine/src/gpu/pipelines.rs` tem uma funcao de criacao por
pipeline, e elas sao quase identicas. olha a do quad inteira:

```rust
pub(super) fn create_quad_pipeline(
    device: &wgpu::Device,
    shader_source: &str,
    projection_bgl: &wgpu::BindGroupLayout,
    surface_format: wgpu::TextureFormat,
    msaa_samples: u32,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("quad_shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("quad_pipeline_layout"),
        bind_group_layouts: &[projection_bgl],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("quad_pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[QuadVertex::layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(premultiplied_blend()),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: msaa_samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    })
}
```

parece muita coisa, mas a maior parte e a mesma em todos os pipelines, e isso e
o ponto. o `format: surface_format` recebe aquele `render_format` sRGB que
discutimos, fechando o circulo: o pipeline mira o mesmo formato que a view da
surface usa, entao a codificacao de gama vale do mesmo jeito quando ele
desenha. o `blend: Some(premultiplied_blend())` aponta pra funcao em
`utils.rs` que define a mistura de alfa pre-multiplicado, e ela vale pra cada
pipeline visivel do engine:

```rust
pub(crate) fn premultiplied_blend() -> wgpu::BlendState {
    wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        // alpha: o mesmo
    }
}
```

o `src_factor: One` em vez de `SrcAlpha` e a marca do alfa pre-multiplicado: a
cor que chega ja foi multiplicada pelo alfa dela, entao o blend nao multiplica
de novo, so soma. isso e um assunto do capitulo do compositor, mas vale ver
que a decisao vive aqui, no estado de blend de cada pipeline, e que ela e a
mesma em todos. e o `multisample.count: msaa_samples` e onde aquele clamp do
inicio do capitulo desemboca: o numero de amostras que o `effective_msaa_samples`
garantiu entra em cada pipeline, e e por isso que ele tinha que ser valido
antes de chegar aqui.

o `immediate_size: 0` no layout e um campo da wgpu 28 (o tamanho de constantes
imediatas, que o engine nao usa). o `cache: None` e a ausencia de um cache de
pipeline em disco. detalhes da versao, mas e bom registrar que estao ali e
zerados de proposito: o engine nao depende de nenhum dos dois.

agora a parte que parece desperdicio e nao e. tem `create_quad_pipeline`,
`create_rect_sdf_pipeline`, `create_shadow_analytic_pipeline`,
`create_text_pipeline`, `create_image_pipeline`, `create_backdrop_pipeline`,
`create_composite_pipeline`, e elas sao em boa parte copia uma da outra. por
que nao uma funcao generica com parametros? porque o que muda entre elas e
pouco e especifico, e abstrair esse pouco custaria mais clareza do que repetir.
o que muda de verdade e o shader, o formato dos vertices, e os bind group
layouts. o quad usa so o layout de projecao (`bind_group_layouts: &[projection_bgl]`).
o texto usa projecao mais o atlas de glifos, dois layouts. a imagem usa
projecao mais o atlas de imagem. o backdrop usa projecao mais o layout de
composicao. cada combinacao e uma decisao de quais recursos aquele shader le, e
ler isso direto na funcao e mais honesto do que decifrar uma tabela de
parametros.

os bind group layouts em si tambem sao compartilhados quando tem a mesma forma.
o atlas de texto e o atlas de imagem usam o mesmo `texture_sampler_bgl` de
`utils.rs`, uma textura 2D amostravel mais um sampler de filtragem, so o label
muda. a projecao e a opacidade usam o mesmo `uniform_bgl`, um unico buffer
uniforme, mudando so o estagio de shader que ve ele (vertex pra projecao,
fragment pra opacidade). reusar a forma do layout e o mesmo principio do reuso
dos buffers: descrever uma vez, instanciar varias.

o pipeline que foge do molde e o de composicao, e vale ver onde ele difere:

```rust
device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("composite_pipeline"),
    layout: Some(&layout),
    vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[],
        compilation_options: Default::default(),
    },
    // ...
    multisample: wgpu::MultisampleState::default(),
    // ...
})
```

dois detalhes. `buffers: &[]`, sem nenhum buffer de vertice. o pipeline de
composicao nao recebe geometria de fora: os vertices sao sintetizados dentro do
proprio vertex shader a partir do indice, o triangulo que cobre a tela inteira
do passe de composicao. e `MultisampleState::default()`, que e uma amostra,
sem MSAA, porque a composicao roda depois que as layers ja foram resolvidas com
o multisampling que tinham que ter; compor as layers ja resolvidas nao precisa
de mais amostras. e a unica peca da camada que nao usa `msaa_samples`, e a
diferenca diz exatamente o que ela faz.

tem ainda o hot reload, atras da feature `hot-reload`, que e o pagamento que
essa extracao de funcoes habilitou. como cada pipeline nasce de uma funcao
nomeada, da pra recompilar um shader em tempo de execucao e recriar so o
pipeline daquele shader, com um escopo de erro em volta pra que um shader
quebrado nao derrube o app:

```rust
let guard = self.device.push_error_scope(wgpu::ErrorFilter::Validation);
let pipeline = Self::create_quad_pipeline(/* ... */);
if let Some(err) = pollster::block_on(guard.pop()) {
    log::error!("Shader reload failed for quad.wgsl: {}", err);
    return false;
}
self.quad_pipeline = pipeline;
```

isso e tema do capitulo de hot reload, mas a semente esta aqui: as funcoes de
criacao foram extraidas, na frase do proprio arquivo, "for hot-reload support".
a repeticao que parece preguica e o que permite trocar uma peca sem mexer nas
outras.

## o numero de capa, e o que ele de fato mede

eu nao fecho um capitulo de camada de GPU sem o numero, mas esse numero precisa
de uma honestidade que e mais util do que o numero solto.

o numero de capa do projeto, o que aparece no outline do paper e no briefing de
benchmarks, sao 159 a 222 milhoes de rects por segundo. ele vem de
`kdb/adr/benchmark-results.md`, medido num macbook pro m4, rust 1.94.0,
criterion 0.5. a tabela e essa:

| benchmark | tempo | throughput |
|-----------|-------|------------|
| push_rects/100 | 629 ns | 159m rects/s |
| push_rects/1000 | 5.48 us | 183m rects/s |
| push_rects/10000 | 45.0 us | 222m rects/s |

o throughput sobe com o tamanho da cena porque o custo fixo por chamada se
dilui: cem rects pagam relativamente mais overhead do que dez mil, e por isso
os dez mil chegam aos 222 milhoes. e um numero de capa justo, e e enorme.

mas aqui vem a parte que importa pra este capitulo, e eu prefiro dizer do que
deixar voce achar que esse numero mede a camada de device, surface e pipelines.
ele nao mede. o benchmark esta em `crates/engine/benches/scene_build.rs`, e o
corpo do `bench_push_rects` e este:

```rust
b.iter(|| {
    let mut comp = Compositor::new();
    for i in 0..n {
        let f = i as f32;
        comp.draw_rect(f, f, 100.0, 50.0, [0.5, 0.5, 0.5, 1.0]);
    }
    black_box(&comp);
});
```

repara: ele cria um `Compositor::new()` e empilha retangulos. nao tem device,
nao tem surface, nao tem pipeline, nao tem `get_current_texture`, nao tem
nenhum dos objetos deste capitulo. o `scene_build.rs` mede o lado CPU que
constroi a descricao da cena, antes de qualquer GPU entrar na conversa. os 159
a 222 milhoes de rects por segundo sao a velocidade de descrever rects na CPU,
nao a velocidade de a GPU desenhar rects.

e isso, longe de ser um problema, e o argumento arquitetural do capitulo
inteiro virado do avesso. lembra a aposta que eu plantei la no comeco: o caro
acontece uma vez, no startup. pedir adaptador, abrir device, configurar
surface, compilar os sete pipelines, tudo isso roda uma vez quando a janela
abre, e some do caminho quente. o que sobra no loop de cada frame e amarrar
pipelines que ja existem e descrever a cena, e a parte que domina o tempo de um
frame e justamente essa descricao de cena na CPU, que e o que o benchmark
isola. o engine empurrou o custo da GPU pro startup de proposito pra que o
custo por frame ficasse na CPU, mensuravel, deterministico, sem driver no meio.
o numero de capa mede o lado certo da fronteira porque a arquitetura colocou o
trabalho recorrente nesse lado.

o que esse numero nao mede, eu marco explicitamente como nao confirmado nesta
camada: nao existe, nas ancoras deste capitulo, um numero que isole o tempo de
`request_adapter`, de `request_device`, do `surface.configure`, da criacao de
um pipeline, ou da execucao de um draw na GPU. esses tempos dependem de driver
e de placa, e um bench de CPU deterministico (que e o que o `scene_build.rs` e,
de proposito, pra rodar em CI sem GPU) nao os captura. se um dia eu quiser o
custo de abrir o device ou de compilar os pipelines, vai ter que sair de um
profile com device real, com xcode ou renderdoc, nao do `scene_build.rs`. ate
la, o honesto e: a camada de device, surface e pipelines nao tem numero medido
nas ancoras deste capitulo, e o numero de capa que existe mede a fase de CPU
que essa camada deliberadamente isolou.

## por que assim, e nao de outro jeito

da pra perguntar se tanta cerimonia valia a pena. podia ser uma funcao que abre
a GPU, configura a surface com o primeiro formato que aparecer, cria os
pipelines inline, e desenha. da pra fazer, muitos engines fazem. o ganho da
forma do plev nao e ter inventado device, surface ou pipeline, esses sao
conceitos da wgpu. o ganho e onde cada decisao foi colocada.

a escolha do formato sRGB vive na configuracao, alimentada pelos formatos reais
da placa, sem ramo por plataforma. isso significa que a diferenca entre desktop
e web sobre cor se resolve uma vez, no `view_formats`, em vez de espalhar `if
web` por todo sitio de desenho. o `surface_render_view` ser o unico caminho
sancionado pro render target significa que o defeito mais caro da camada, a cor
escura na web, so pode reaparecer se alguem contornar uma funcao documentada de
proposito, e o ADR esta la pra que esse contorno nao passe no review. a surface
ser o unico recurso em `Option` significa que o ciclo de suspende e resume do
mobile nao precisa reconstruir o engine inteiro, so a ponte com a janela, e o
device, a queue e os pipelines sobrevivem intactos. e os pipelines nascerem de
funcoes nomeadas e separadas significa que o caro acontece uma vez no startup e
que da pra trocar um shader sozinho com hot reload sem tocar nos outros.

cada uma dessas e a mesma postura: empurrar a diferenca pro dado e pro tipo, em
vez de pro fluxo de controle. o `Option` na surface e o tipo dizendo "essa peca
e mortal". o `view_formats` e o dado dizendo "essa surface tem um formato extra
de escrita". o `effective_msaa_samples` clampando na entrada e o tipo
garantindo que o numero que chega no pipeline ja e valido. menos coisa pra
confiar no programador no momento quente, mais coisa decidida e congelada no
startup.

## o que isso me ensinou

a licao que eu levei dessa camada nao foi sobre GPU, foi sobre onde colocar a
diferenca. quando eu olhei a primeira vez, achei que a parte dificil ia ser
falar com o hardware, o adaptador, o device, os limites. e essa parte e quase
mecanica: pede com modestia (zero features, limites default), aceita o que a
placa da, e pronto. a parte dificil de verdade foi a cor, e ela ensinou uma
coisa que eu nao esperava: o bug mais caro de uma camada de baixo nivel pode
ser o mais invisivel, justamente porque so aparece onde voce nao testa. um
fundo (8,8,8) em vez de (48,48,48) no browser, com tudo certo no mac, e o tipo
de defeito que passa por review sorrindo. a defesa contra ele nao foi mais
codigo, foi concentrar o caminho num metodo so e escrever um ADR explicando por
que o caminho obvio e uma armadilha.

e a segunda licao, a que amarra o capitulo no numero: a melhor coisa que essa
camada faz e sair do caminho. ela paga o caro uma vez, no startup, e deixa o
loop de frame com o trabalho que da pra medir na CPU, sem driver no meio. o
numero de capa, os 159 a 222 milhoes de rects por segundo, e a prova disso pelo
avesso: ele mede a fase de CPU porque e a fase de CPU que sobrou no caminho
quente, e ela sobrou porque a camada de device, surface e pipelines fez o
trabalho dela no momento em que a janela abriu, e depois ficou quieta amarrando
pipelines que ja estavam prontos.

se eu fosse deixar uma frase pra quem esta abrindo a primeira janela com wgpu e
achando que o dificil e a GPU: o dificil nao e abrir a placa, isso a wgpu
resolve com tres `.await`. o dificil e decidir onde cada diferenca de
plataforma vai morar, e a resposta boa, quase sempre, e no dado da configuracao
e no tipo do campo, nao num `if` espalhado pelo desenho.

## rastros

codigo (crate engine, conferido contra a arvore atual)

- `crates/engine/src/gpu/context.rs:23` (`struct GpuContext`, surface em
  `Option`, device/queue/pipelines em posse direta)
- `crates/engine/src/gpu/context.rs:26` (`surface: Option<wgpu::Surface<'static>>`,
  o `'static` que segura a janela por `Arc`)
- `crates/engine/src/gpu/context.rs:58` (`new` chama `new_with_config` com
  `RenderConfig::default()`, e `async`)
- `crates/engine/src/gpu/context.rs:69` (`new_with_config`, `effective_msaa_samples`
  e `set_default_tolerance` antes de tocar a GPU)
- `crates/engine/src/gpu/context.rs:74` (`Instance::new`, `Backends::PRIMARY` vs
  `Backends::BROWSER_WEBGPU` atras de `#[cfg]`)
- `crates/engine/src/gpu/context.rs:82` (`create_surface`, depois `request_adapter`
  com `HighPerformance`, `compatible_surface: Some(&surface)`, `force_fallback_adapter: false`)
- `crates/engine/src/gpu/context.rs:95` (`request_device`, `Features::empty()`,
  `Limits::default()`, `MemoryHints::Performance`, label `plev_device`)
- `crates/engine/src/gpu/context.rs:115` (`get_capabilities`, `find(is_srgb)`,
  `unwrap_or(formats[0])`)
- `crates/engine/src/gpu/context.rs:129` (`render_format = surface_format.add_srgb_suffix()`,
  `view_formats` so quando difere, sem `#[cfg]`)
- `crates/engine/src/gpu/context.rs:139` (fallback de `present_mode` pra `AutoVsync`)
- `crates/engine/src/gpu/context.rs:153` (`SurfaceConfiguration`, `usage: RENDER_ATTACHMENT`,
  `format` base, `view_formats`, `configure`)
- `crates/engine/src/gpu/context.rs:166` (buffer de projecao, `UNIFORM | COPY_DST`,
  `uniform_bgl` e bind group)
- `crates/engine/src/gpu/context.rs:190` (pipelines criados no `new` com `render_format`,
  uma vez)
- `crates/engine/src/gpu/surface.rs:14` (`resize`, `.max(1)`, `if let Some` no
  `configure`, reescreve a projecao)
- `crates/engine/src/gpu/surface.rs:35` (`drop_surface`, surface vira `None`,
  resto sobrevive)
- `crates/engine/src/gpu/surface.rs:42` (`recreate_surface`, android resume,
  repete o `add_srgb_suffix` e o `view_formats`)
- `crates/engine/src/gpu/surface.rs:122` (`surface_format()`, devolve o formato da
  view, cai no base)
- `crates/engine/src/gpu/surface.rs:134` (`surface_render_view`, unico caminho
  sancionado pro render target, forca `format: Some(self.surface_format())`)
- `crates/engine/src/gpu/pipelines.rs:12` (`create_quad_pipeline`, layout `[projection_bgl]`,
  `format: surface_format`, `premultiplied_blend`, `count: msaa_samples`)
- `crates/engine/src/gpu/pipelines.rs:177` (`create_text_pipeline`, dois bind group
  layouts: projecao + atlas de glifos)
- `crates/engine/src/gpu/pipelines.rs:352` (`reload_shader`, `push_error_scope`,
  recria so o pipeline do shader trocado)
- `crates/engine/src/gpu/pipelines.rs:485` (`create_composite_pipeline`, `buffers: &[]`,
  `MultisampleState::default()`)
- `crates/engine/src/gpu/utils.rs:3` (`premultiplied_blend`, `src_factor: One`,
  `dst_factor: OneMinusSrcAlpha`)
- `crates/engine/src/gpu/utils.rs:19` (`ortho_projection`, `[0,w] -> [-1,1]`,
  Y pra baixo)
- `crates/engine/src/gpu/utils.rs:33` (`texture_sampler_bgl`, forma compartilhada
  por atlas de texto e de imagem)
- `crates/engine/src/gpu/utils.rs:59` (`uniform_bgl`, forma compartilhada por
  projecao e opacidade)
- `crates/engine/src/gpu/config.rs:32` (`RenderConfig::default`, msaa 4, `AutoVsync`,
  tolerancia 0.1)
- `crates/engine/src/gpu/config.rs:47` (`effective_msaa_samples`, clampa pra 1 ou 4)
- `crates/engine/src/window/render.rs:37` (`get_current_texture`, `Lost`/`Outdated`
  reconfigura via `resize` e desiste do frame)
- `crates/engine/src/window/render.rs:53` (`surface_view = gpu.surface_render_view(&output)`,
  todo frame)
- `crates/engine/src/window/render.rs:78` (resolve recebe `gpu.surface_format()`,
  mesmo formato da view)

adr (cor e render target)

- `kdb/adr/render-into-an-srgb-view-format.md:9` (titulo: render into an sRGB view
  when the surface format cannot be sRGB; data 2026-06-10)
- `kdb/adr/render-into-an-srgb-view-format.md:13` (contexto: canvas WebGPU so aceita
  formato nao-sRGB; fundo mediu (8,8,8) em vez de (48,48,48))
- `kdb/adr/render-into-an-srgb-view-format.md:22` (decisao: base + variante sRGB em
  `view_formats`, render passes miram a view sRGB)
- `kdb/adr/render-into-an-srgb-view-format.md:36` (consequencia: web (8,8,8) antes,
  (48,48,48) depois; sete sitios migrados pra `surface_render_view`)
- `kdb/adr/render-into-an-srgb-view-format.md:43` (avoid: nunca
  `create_view(&Default::default())` em surface; nao ramificar por plataforma)

benchmark e numero de capa

- `kdb/adr/benchmark-results.md:11` (maquina m4, rust 1.94.0, criterion 0.5)
- `kdb/adr/benchmark-results.md:18` (push_rects/100: 629 ns, 159m rects/s)
- `kdb/adr/benchmark-results.md:19` (push_rects/1000: 5.48 us, 183m rects/s)
- `kdb/adr/benchmark-results.md:20` (push_rects/10000: 45.0 us, 222m rects/s)
- `crates/engine/benches/scene_build.rs:10` (`bench_push_rects`, `Compositor::new()`
  headless, sem device/surface/pipeline)
- `crates/engine/benches/scene_build.rs:12` (tamanhos 100/1000/10000)

versoes (conferidas contra o Cargo.toml)

- `Cargo.toml:23` edition 2024
- `Cargo.toml:50` wgpu 28 (`Instance`, `Adapter`, `Device`, `Surface`,
  `RenderPipeline`, `add_srgb_suffix`, `immediate_size`)
- `Cargo.toml:51` winit 0.30 (a janela, `Arc<Window>`)
- `Cargo.toml:52` bytemuck 1 com `derive` (`cast_slice` na projecao)
- `Cargo.toml:83` pollster 0.4 (executor do `.await` no desktop e no `recreate_surface`)
- `Cargo.toml:99` criterion 0.5 (o `scene_build.rs`)

nao confirmado

- nenhuma ancora deste capitulo isola o tempo de `request_adapter`,
  `request_device`, `surface.configure`, criacao de pipeline, ou execucao de
  draw na GPU. o `scene_build.rs` roda headless (`Compositor::new()`, sem
  device), entao mede a construcao CPU da cena, nao a camada de device/surface/
  pipelines. um numero dessa camada exigiria profile com device real, fora desta
  ancora.
- o numero de capa (159 a 222m rects/s) e throughput de descricao de cena na
  CPU, nao throughput de desenho de rects na GPU. atribui-lo a velocidade da GPU
  seria desonesto; ele mede a fase que a arquitetura colocou no caminho quente, e
  e por isso que ele e o numero certo pro caminho quente, nao pra esta camada.
- a descricao do pipeline de composicao como "triangulo que cobre a tela
  inteira" se apoia em `buffers: &[]` (sem buffer de vertice, vertices vindos do
  shader por indice) e na nota de arquitetura do SUMARIO (2.4, full-screen
  triangle no composite pass); eu nao li o `composite.wgsl` pra confirmar a
  geometria exata sintetizada no vertex shader.
- a afirmacao de que `queue.write_buffer` e `surface.configure` sincronizam do
  lado da GPU e comportamento documentado da API da wgpu, nao uma medicao feita
  aqui.
