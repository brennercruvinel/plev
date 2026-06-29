---
title: "async sem runtime pesado: init da gpu e o eventloopproxy"
parte: 1
capitulo: 7
status: rascunho
idioma: pt-br
ancoras:
  - crates/engine/src/window/mod.rs
  - crates/engine/src/gpu/context.rs
  - crates/engine/src/window/state.rs
  - crates/engine/src/window/lifecycle.rs
  - crates/showcase/src/app.rs
  - kdb/adr/async-gpu-init-and-single-wasm-entry.md
  - kdb/adr/wasm-webgpu-validation.md
rastros: ver bloco final
---

# async sem runtime pesado: init da gpu e o eventloopproxy

a primeira vez que rodei o showcase no navegador, a tela ficou preta e ficou
preta. nada de erro vermelho no console, nada de panic, nada. so o canvas
parado, esperando um frame que nunca vinha. no desktop a mesma build subia em
menos de um segundo e desenhava a galeria inteira. no chrome, preto.

esse capitulo e sobre o porque desse preto, e sobre a peca de codigo que
resolve ele sem arrastar um runtime async inteiro pra dentro do binario. a
peca tem nome: `EventLoopProxy`. ela e pequena, cabe em um struct field e em
uma chamada de `spawn_local`, e ela existe por um motivo que vale a pena
entender de verdade, porque ele encosta no jeito que o navegador roda codigo
e no jeito que o rust modela async.

vou abrir pelo lado humano (o que era o preto), descer pro codigo real que
esta no repo hoje, e depois cavar ate o porque arquitetural. o numero de
benchmark que eu tenho pra ancorar isso aqui e modesto e nao e de wasm, entao
vou ser honesto sobre o que esta medido e o que nao esta.

## a gpu nasce assincrona

antes de qualquer truque de event loop, tem um fato que define tudo: pegar
acesso a gpu e uma operacao assincrona. nao por capricho de api, por natureza.
voce pede um adapter (a abstracao de uma placa de video ou de um backend de
software), o sistema vai conversar com o driver, e isso leva tempo. voce pede
um device e uma queue a partir daquele adapter, e isso tambem leva tempo. no
wgpu, a crate que o plev usa pra falar com vulkan, metal, dx12 e webgpu, essas
duas operacoes sao `async`.

da pra ver isso na assinatura do construtor do nosso contexto de gpu. ele e
`async fn`, e o corpo dele tem dois `await` que sao o coracao da coisa:

```rust
impl GpuContext {
    pub async fn new(window: Arc<Window>) -> Self {
        Self::new_with_config(window, RenderConfig::default()).await
    }

    pub async fn new_with_config(window: Arc<Window>, mut config: RenderConfig) -> Self {
        // ...
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

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

repare no `.await` depois do `request_adapter`, e logo abaixo tem outro depois
do `request_device`. esses dois pontos sao onde a funcao para, devolve o
controle, e espera o mundo externo (o driver, o browser, o compositor do SO)
responder. tudo entre o primeiro e o ultimo `.await` e o que o codigo chama de
inicializacao da gpu: criar a instance, achar o adapter, abrir o device e a
queue, descobrir o formato de surface, montar os buffers de projecao, compilar
cada pipeline (quad, rect sdf, sombra, texto, imagem, composite, backdrop),
criar os samplers. e bastante coisa. e tudo isso pendura no resultado de um
future.

aqui ja aparece a primeira escolha de design do projeto: nao tem um segundo
caminho sincrono. existe um unico construtor, `async`, e os dois targets
(desktop e wasm) consomem o mesmo future de jeitos diferentes. isso e
deliberado, e e o que o adr de init chama de "render path com zero branches de
plataforma": a diferenca de plataforma mora na inicializacao, nao no desenho.

## no desktop, a gente simplesmente bloqueia

um `async fn` em rust nao roda sozinho. ele devolve um future, um valor preguicoso
que so anda quando alguem o conduz (poll). no desktop a maneira mais direta de
conduzir um future ate o fim e bloquear a thread atual ate ele resolver. e
exatamente o que o codigo faz, com a crate `pollster`, que e um executor
minimo de uma funcao so:

```rust
#[cfg(not(target_arch = "wasm32"))]
{
    let gpu = pollster::block_on(GpuContext::new(window));
    let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
    let effect_processor = EffectProcessor::new(&gpu.device, gpu.surface_format());
    let texture_pool = TexturePool::new();
    self.state = GpuState::Ready {
        gpu,
        text_system,
        effect_processor,
        texture_pool,
    };
}
```

`pollster::block_on` faz o nome que tem: ele para a thread, fica em loop dando
poll no future ate ele terminar, e devolve o valor. simples, honesto, e
funciona porque no desktop a thread do event loop pode dormir um pouco durante
o boot sem quebrar nada. o usuario clica no app, a janela aparece, leva uns
milissegundos a mais pra primeira tela, ninguem nota. `pollster` aparece no
`Cargo.toml` so pra esse target, junto com o `env_logger`: as duas sao
dependencias native-only, ficam fora do build de wasm de proposito.

`pollster` nao e o tokio, nao e o async-std, nao e um runtime de verdade. ele
e um executor de uma tarefa, sem scheduler, sem thread pool, sem timer wheel,
sem reactor. e essa e a tese inteira do capitulo escrita em uma crate: pra
inicializar a gpu eu nao preciso de um runtime async. preciso de uma forma de
levar um future ate o fim, e cada plataforma ja tem a sua. no desktop e
bloquear. no navegador e outra coisa, e e ai que mora o problema.

## por que o navegador nao pode bloquear

agora o preto. no navegador, `pollster::block_on(GpuContext::new(...))`
dentro do handler de `resumed` nao funciona. o adr e direto sobre isso: o
browser nao pode bloquear, porque os pedidos de adapter e device sao promises
de verdade, e bloquear a thread principal pra esperar uma promise no
javascript ou trava ou da panic.

vale entender o mecanismo, nao so a regra. o navegador roda o seu wasm na
thread principal, a mesma thread que processa a fila de eventos da pagina,
desenha o DOM, e resolve promises. essa thread tem um modelo cooperativo: o
codigo roda ate devolver o controle, e so quando devolve o controle o
navegador consegue avancar a fila de microtasks onde as promises sao
resolvidas. uma promise de webgpu (o `requestAdapter`, o `requestDevice`) so
completa quando o seu codigo solta a thread e deixa o event loop girar.

`block_on` faz o oposto. ele agarra a thread e fica em loop esperando o future
resolver. so que o future depende de uma promise. e a promise depende da
thread ser solta. deadlock circular: o seu loop espera a promise, a promise
espera o seu loop terminar. no melhor caso o browser detecta e mata; no pior,
trava a aba. era esse o preto. a `resumed` chamava o caminho de bloqueio, a
thread principal nunca era devolvida, o `requestAdapter` nunca resolvia, o
`GpuContext` nunca nascia, e o estado ficava parado pra sempre esperando um
device que nunca chegava.

o documento de validacao de wasm guarda a memoria mais crua desse bug, antes
mesmo do deadlock: a versao que tentava ser esperta com `spawn_local` mas
errava o destino do resultado. o async no wasm roda via
`wasm_bindgen_futures::spawn_local`, e o closure que voce passa pra ele precisa
ser `'static`. isso quer dizer que `self` nao pode ser movido pra dentro do
closure (o `self` do `ApplicationHandler` e emprestado, nao e `'static`). o
codigo original criava o `GpuContext` la dentro do future e depois o
descartava, porque nao tinha como devolver ele pro `self`. resultado: a gpu
nascia, vivia o tempo do future, e morria sem nunca ser instalada no estado da
app. o estado ficava preso pra sempre. mesmo sintoma, causa diferente: num
caso o future nunca completa, no outro ele completa e joga o resultado fora.

os dois desenham a mesma forma do problema. a inicializacao da gpu no
navegador tem que rodar fora da pilha do handler, de forma assincrona, e o
resultado tem que voltar pra app por um canal que sobreviva ao closure
`'static`. esse canal e o `EventLoopProxy`.

## o eventloopproxy, em quatro passos

o winit (a crate de janela e event loop, na versao 0.30 no nosso lock) tem um
recurso pensado pra exatamente isso: eventos de usuario. voce parametriza o
event loop com um tipo seu, e ganha um proxy que pode injetar valores desse
tipo na fila de eventos de qualquer lugar, inclusive de dentro de um future.
quando o valor chega, o winit te chama de volta no metodo `user_event` do seu
`ApplicationHandler`, na thread certa, no momento certo do loop.

o documento de validacao descreve o padrao em quatro passos, e o codigo do
repo segue eles a risca:

1. `EventLoop::<AppEvent>::with_user_event().build()` cria um event loop com
   um tipo de evento custom.
2. `event_loop.create_proxy()` devolve um `EventLoopProxy<AppEvent>`, que e
   `Send + Sync`.
3. `spawn_local` recebe uma copia do proxy e manda `AppEvent::GpuReady { ... }`
   quando o async termina.
4. `ApplicationHandler::user_event` recebe o evento e faz a transicao de
   estado.

o tipo de evento e um enum simples. no engine ele se chama `AppEvent` e carrega
exatamente o que a inicializacao produziu, pronto pra ser instalado:

```rust
pub enum AppEvent {
    GpuReady {
        gpu: GpuContext,
        text_system: TextSystem,
        effect_processor: EffectProcessor,
        texture_pool: TexturePool,
    },
}
```

repare que o evento nao carrega um "ok" ou um codigo de status. ele carrega o
objeto inteiro, o `GpuContext` por valor, junto com o sistema de texto, o
processador de efeitos e o pool de texturas. o future construiu tudo isso e
agora entrega tudo de uma vez, por movimento, atravessando a fronteira entre o
mundo async e o event loop. e por isso que o enum tem que viver junto do tipo
do event loop: `EventLoop::<AppEvent>`. o winit vai guardar esse valor na fila
e te devolver ele intacto.

o proxy fica guardado no struct da app, atras de um cfg, porque so existe nos
targets que precisam dele:

```rust
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub(crate) event_loop_proxy: Option<EventLoopProxy<AppEvent>>,
```

esse `Option` e o detalhe que faz o resto funcionar. ele comeca preenchido, e
na hora de inicializar a gpu o codigo faz `.take()` nele. tomar o proxy
(deixando `None` no lugar) move o proxy pra dentro do closure sem precisar de
clone e, de quebra, garante que a inicializacao async so dispara uma vez: se
`resumed` for chamado de novo, o `take` ja devolveu `None` e nada acontece. e o
mesmo cfg vale pra android, porque android tambem tem um ciclo de vida onde a
superficie pode ser recriada e onde bloquear nem sempre e a escolha certa. o
campo so nao existe no desktop puro, onde o caminho e `block_on` direto.

## o codigo de init, dividido por target

agora da pra ver a peca central inteira. e a `init_gpu`, e ela e literalmente
o mesmo metodo com dois corpos selecionados por `cfg`:

```rust
pub(crate) fn init_gpu(&mut self, window: Arc<Window>) {
    self.scale_factor = window.scale_factor();
    self.safe_area = crate::platform::SafeAreaInsets::from_window(&window);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effect_processor = EffectProcessor::new(&gpu.device, gpu.surface_format());
        let texture_pool = TexturePool::new();
        self.state = GpuState::Ready {
            gpu,
            text_system,
            effect_processor,
            texture_pool,
        };
    }

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(proxy) = self.event_loop_proxy.take() {
            let window_clone = window;
            wasm_bindgen_futures::spawn_local(async move {
                let gpu = GpuContext::new(window_clone).await;
                let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
                let effect_processor = EffectProcessor::new(&gpu.device, gpu.surface_format());
                let texture_pool = TexturePool::new();
                let _ = proxy.send_event(AppEvent::GpuReady {
                    gpu,
                    text_system,
                    effect_processor,
                    texture_pool,
                });
            });
        }
    }
}
```

leia os dois lados como se fossem a mesma pessoa contando a mesma historia em
dois idiomas. as primeiras linhas, scale factor e safe area, valem pros dois.
depois o caminho de desktop bloqueia, monta o estado, e ja deixa `GpuState::Ready`
no lugar antes de a funcao retornar. quando `init_gpu` termina no desktop, a
gpu ja existe.

o caminho de wasm e o oposto: quando `init_gpu` termina, a gpu ainda nao
existe. o que aconteceu foi so o disparo. `self.event_loop_proxy.take()` tirou
o proxy do `Option`, o `window` foi movido pro `window_clone` (que e
`'static`, porque e um `Arc<Window>` proprio, nao um emprestimo de `self`), e o
`spawn_local` agendou o future no microtask queue do navegador. a funcao
retorna na hora. o `await` la dentro vai rodar depois, quando a thread for
solta e o browser girar o loop. e quando o `await` final resolver, a ultima
linha do future faz `proxy.send_event(AppEvent::GpuReady { ... })`, empurrando
o contexto pronto de volta pra fila do event loop.

o `let _ =` no `send_event` nao e desleixo. `send_event` devolve um
`Result<(), EventLoopClosed<T>>`: ele falha so se o event loop ja tiver
fechado, o que aqui significaria que a aba foi descartada antes da gpu ficar
pronta. nesse caso nao tem o que fazer, o destino do evento nao existe mais,
entao descartar o erro e a coisa certa.

o outro lado do canal e o `user_event`. o `ApplicationHandler` do engine so
repassa pro handler real:

```rust
fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
    self.handle_user_event(event_loop, event);
}
```

e o handler de verdade casa o `GpuReady`, instala tudo no estado e pede um
redraw:

```rust
pub(crate) fn handle_user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
    match event {
        AppEvent::GpuReady {
            gpu,
            text_system,
            effect_processor,
            texture_pool,
        } => {
            log::info!("GPU context ready (async)");
            self.state = GpuState::Ready {
                gpu,
                text_system,
                effect_processor,
                texture_pool,
            };
            if let Some(ref window) = self.window {
                window.request_redraw();
            }
        }
    }
}
```

aqui o `gpu` que vinha viajando dentro do `AppEvent` finalmente pousa em
`self.state`. nao tem clone, nao tem `Arc<Mutex<...>>`, nao tem canal de
mensagem improvisado. o objeto inteiro foi movido do future pro evento, do
evento pro estado. o sistema de tipos do rust garante que ele atravessou essa
viagem sem ninguem mais segurando uma referencia pra ele, e o `match` por
movimento desmonta o evento e remonta o estado em um passo. e o `request_redraw`
no fim e o que tira a tela do preto: agora que tem gpu, da pra desenhar o
primeiro frame.

aqui mora uma das regras de ouro do engine, a de invalidacao: qualquer handler
que muda estado visivel precisa invalidar, pedir um frame. o `handle_user_event`
acabou de mudar o estado de "sem gpu" pra "pronto pra desenhar". se ele nao
pedisse o redraw, a app voltaria a dormir e a tela continuaria preta mesmo com
a gpu pronta. o render acontece sob demanda, e a chegada da gpu e exatamente o
tipo de demanda que tem que acordar o loop.

## o mesmo padrao na app, e por que cada plataforma escolhe um final

o engine expoe o padrao, mas cada app monta o seu event loop. o showcase tem o
seu proprio `UserEvent` e o seu proprio `ApplicationHandler`, com a mesma forma.
o que muda, e o que vale notar, e como cada plataforma constroi e roda o loop.
no desktop:

```rust
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
```

`run_app` toma a thread e roda o loop ate o app sair. e o final natural no
desktop: a funcao `run` so retorna quando a janela fecha. no navegador esse
final nao serve, e o motivo de novo encosta no modelo do browser:

```rust
#[cfg(target_arch = "wasm32")]
pub fn run_web() {
    use winit::platform::web::EventLoopExtWebSys;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("failed to init console_log");

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    app.proxy = Some(event_loop.create_proxy());
    event_loop.spawn_app(app);
}
```

tres diferencas, todas falando a mesma coisa. primeira: aqui o proxy e criado
com `event_loop.create_proxy()` e guardado em `app.proxy` antes de o loop
comecar. o desktop nem cria proxy, porque nao vai precisar. segunda: o setup de
log e de panic hook usa `console_log` e `console_error_panic_hook` em vez do
`env_logger`, porque no browser o destino do log e o console do dev tools.
terceira, e a mais importante: `spawn_app` no lugar de `run_app`.

a diferenca entre `spawn_app` e `run_app` no winit web e o resumo de tudo que
esse capitulo vem dizendo. `run_app` no browser teria que "nunca retornar", e
no javascript a unica forma de uma funcao nao retornar e lancar uma excecao
pra escapar do `main`. isso e feio e fragil. `spawn_app` faz o certo: ele
entrega o app pro event loop interno do proprio navegador e devolve o controle.
o loop nao e o seu, e o do browser. ele chama o seu `resumed`, o seu
`window_event`, o seu `user_event` quando for a hora, cooperando com a thread
principal em vez de sequestrar ela. e por isso que a app no browser pode mandar
um `GpuReady` pelo proxy e confiar que o `user_event` vai ser chamado: porque o
loop e cooperativo e continua girando depois que `run_web` retorna.

android fecha o trio com um terceiro final ainda diferente. ele usa
`run_app`, como o desktop, mas constroi o loop com o backend de GameActivity:

```rust
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    // ...
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .with_android_app(android_app)
        .build()
        .expect("android event loop");
    let mut app = App::new();
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("event loop error: {e:?}");
    }
}
```

quatro entry points, quatro finais (`run`, `run_web`, `android_main`,
`showcase_ios_main`), uma forma de evento. o `EventLoop::<UserEvent>::with_user_event()`
e a coluna que aguenta os quatro. e o `UserEvent::GpuReady` so e construido de
fato no wasm; nos outros targets o enum existe mas o caminho async nao dispara,
por isso ele leva um `#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]`
no codigo do showcase, pra silenciar o aviso sem ligar `allow` no crate
inteiro.

## o resync de viewport: o detalhe que so o async expoe

tem uma sutileza que so aparece porque a init e assincrona, e que e facil de
nao perceber ate a tela sair torta. enquanto o `requestAdapter` e o
`requestDevice` estavam rodando, a thread estava solta. com a thread solta, o
navegador pode ter feito de tudo, inclusive redimensionar o canvas. o usuario
girou o celular, arrastou a borda da janela, abriu o dev tools e empurrou o
layout. esses resizes chegam como `WindowEvent::Resized` normais, mas podem ter
chegado antes da gpu existir, quando ainda nao tinha surface pra reconfigurar.

por isso o handler de `GpuReady` no showcase nao so instala o estado, ele
re-sincroniza o viewport inteiro:

```rust
fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserEvent) {
    match event {
        UserEvent::GpuReady { gpu, text_system, effects, texture_pool } => {
            log::info!("GPU context ready (async)");
            self.state = GpuState::Ready { gpu, text_system, effects, texture_pool };
            // The canvas may have been resized while the adapter and
            // device were being requested; re-sync everything.
            self.configure_viewport();
        }
    }
}
```

o `configure_viewport` le o `inner_size` atual da janela, reconfigura a
surface, reescreve a projecao e refaz o layout da view com o tamanho de agora,
nao com o tamanho que existia quando `resumed` rodou. o adr coloca isso como
parte da decisao: o handler instala o contexto e re-sincroniza o viewport, o
que tambem cobre os resizes que aconteceram durante a inicializacao. e um
exemplo limpo de uma regra do engine, a de que geometria de container deriva do
espaco disponivel, nunca de constante: o tamanho certo e o que a janela reporta
no momento, e o codigo vai buscar esse momento na hora em que a gpu nasce.

esse detalhe so existe porque a init e async. no desktop, com `block_on`, nao
tem janela entre o create_window e o estado pronto, entao nao tem resize pra
perder. no navegador tem essa janela, ela pode ser longa, e ignorar ela
significa primeiro frame com a surface no tamanho errado. e o tipo de bug que
nao da panic e nao aparece no teste de viewport unico, so na vida real quando
alguem mexe na janela durante o boot.

## o segundo problema: um entry point de wasm por modulo

o adr de async junta a inicializacao com um problema vizinho, e os dois andam
juntos por um motivo. um modulo wasm aceita exatamente um
`#[wasm_bindgen(start)]`. o engine, no comeco, exportava um start
incondicional. isso fazia o entry de toda app downstream colidir com o do
engine na hora de linkar. duas funcoes brigando pra ser o ponto de entrada do
mesmo modulo wasm.

a decisao foi por o start do proprio engine atras de uma feature de cargo,
`web-entry`, desligada por padrao. o `Cargo.toml` do engine deixa isso
explicito no comentario da feature: ela exporta o `#[wasm_bindgen(start)]` do
engine no wasm32, e fica off por padrao pra que as apps downstream definam o
proprio entry de browser sem colidir. e foi por isso que o showcase ganhou o
seu proprio `run_web`, com o seu `spawn_app`, o seu `console_log` e o seu
`console_error_panic_hook`, reusando so o que e seguro reusar do engine, como o
`setup_wasm_canvas`.

um aparte de naming, porque o repo esta no meio de uma transicao. o adr foi
escrito quando o crate do engine ainda se chamava `plev`, e cita o helper como
`plev::window::setup_wasm_canvas`. no codigo de hoje o crate se chama `engine`
e o showcase importa ele como `engine::window::setup_wasm_canvas`. mesma
funcao, mesmo lugar no modulo (`crates/engine/src/window/mod.rs` reexporta ela
sob `#[cfg(target_arch = "wasm32")]`), so o prefixo do crate mudou. fica a nota
pra quem cruzar o adr com o codigo e estranhar o `plev::`.

a conexao entre os dois problemas e que ambos sao sobre a mesma fronteira: como
um app entra no navegador. um e sobre quando a gpu fica pronta (async, via
proxy), o outro e sobre quem chama o codigo primeiro (o entry point, via
feature gate). resolver os dois junto e o que deixa "a mesma fonte serve os
dois targets com duas regioes cfg-gated, sem fork de app", que e o que o adr
declara como consequencia.

## limits, e por que webgpu nao e webgl

um ultimo pedaco que a inicializacao no navegador decide, e que e facil errar.
quando voce pede o device, voce passa `required_limits`. no codigo o valor e
`wgpu::Limits::default()` nos dois targets:

```rust
let (device, queue) = adapter
    .request_device(&wgpu::DeviceDescriptor {
        label: Some("plev_device"),
        required_features: wgpu::Features::empty(),
        #[cfg(not(target_arch = "wasm32"))]
        required_limits: wgpu::Limits::default(),
        #[cfg(target_arch = "wasm32")]
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: Default::default(),
        experimental_features: Default::default(),
    })
    .await
    .expect("Failed to create device");
```

o documento de validacao explica a escolha. tem um perfil alternativo,
`Limits::downlevel_webgl2_defaults()`, que limita o `max_texture_dimension_2d`
a 2048. isso e pequeno demais pro nosso atlas de glifos e imagens. o
`Limits::default()` e o baseline garantido pelo spec do webgpu e suporta
texturas ate 8192 por 8192. como o engine mira webgpu de verdade no browser
(repare no `Backends::BROWSER_WEBGPU` la no `Instance::new`), e nao o caminho
de compatibilidade via webgl, usar `Limits::default()` e o certo. pedir o
perfil de webgl seria assumir uma plataforma mais pobre do que a que a gente de
fato suporta, e cortar o atlas pela metade.

esse detalhe parece desconexo do async, mas ele vive no mesmo `await` do
`request_device`. a inicializacao e o lugar onde todas as decisoes de
plataforma se concentram: backend, limits, formato de surface, encode de srgb.
o render que vem depois nao tem branch de plataforma nenhum, justamente porque
todas elas foram pagas aqui, durante o boot, dentro do future.

## o porque arquitetural: o browser ja e o runtime

da pra resumir a tese assim. async em rust e dois pedacos separados: a
linguagem, que da o `async`/`await` e o tipo `Future`, e o runtime, que conduz
os futures e cuida de IO, timers e scheduling. tokio e async-std sao runtimes.
eles sao otimos quando o seu programa e cheio de IO concorrente, milhares de
conexoes, tarefas que esperam disco e rede o tempo todo. eles trazem um
scheduler, um reactor, threads, e um custo de binario e de complexidade que
acompanha tudo isso.

o plev nao precisa disso pra subir a gpu. ele precisa de uma operacao
assincrona, uma so, levada ate o fim, uma vez, na inicializacao. e as duas
plataformas que importam ja tem o executor que faz isso. no desktop, a thread e
o executor, via `pollster::block_on`, que e uma crate de uma funcao. no
navegador, o proprio browser e o runtime: ele ja tem um event loop cooperativo,
ja resolve promises, ja tem um microtask queue. o `wasm_bindgen_futures::spawn_local`
nao traz um runtime novo, ele so pendura o seu future no runtime que ja esta
rodando, o do navegador. e o `EventLoopProxy` e a ponte de volta desse runtime
pro seu codigo.

isso e o "async sem runtime pesado" do titulo. nao tem tokio no `Cargo.toml` do
engine. nao tem async-std. tem `pollster` (native-only) e
`wasm-bindgen-futures` (web-only), as duas minusculas, cada uma so no target em
que faz sentido, separadas por `[target.'cfg(...)']` no manifesto. o resto do
trabalho assincrono e feito pela plataforma, e o codigo so se conecta nela pelo
ponto certo.

tem um beneficio de tipo que cai de graca dessa escolha, e que vale marcar. o
`EventLoopProxy<AppEvent>` e `Send + Sync`. isso quer dizer que mandar o evento
de volta e uma operacao thread-safe por construcao, checada pelo compilador, e
nao por convencao. no wasm a thread e uma so, entao o `Send + Sync` parece
ocioso, mas ele e o que deixa o mesmo padrao valer no android, onde o ciclo de
vida e mais agressivo e a superficie pode ir e voltar. o mesmo `AppEvent`, o
mesmo `user_event`, o mesmo proxy. a forma nao muda, so o final do loop muda por
plataforma.

vale insistir num ponto que parece pedante mas e o nucleo da escolha:
`pollster` nao e um runtime, e por isso ele cabe aqui. um runtime de verdade
mantem um conjunto de tarefas vivas, agenda elas entre si, e fica acordando pra
ver quem pode progredir. `pollster::block_on` nao mantem nada: ele pega um
future, fica dando poll nele na thread atual, e quando ele resolve, acabou. nao
tem fila de outras tarefas, nao tem reactor escutando file descriptors, nao tem
thread de fundo. e essa pobreza que e a virtude. eu nao quero que a
inicializacao da gpu traga junto uma maquinaria de concorrencia que o resto do
engine nunca vai usar, porque o engine nao e um servidor, e um compositor que
desenha frames sob demanda. trazer um runtime pra subir uma gpu seria pagar o
custo de uma usina pra acender uma lampada. o future existe, e real, mas ele e
conduzido pela coisa mais simples que da conta: a thread no desktop, o browser
no web.

## o que da pra medir, e o que nao da

vou ser honesto sobre numeros, porque o briefing pede ancora em dado e eu nao
vou inventar um. nas ancoras desse capitulo nao tem um benchmark de quanto
tempo a inicializacao async da gpu leva no navegador. o adr fala que o caminho
de runtime e exercitado pelo build do trunk mais um screenshot scriptado de
browser, mas isso e um teste de "funciona", nao um numero de "quao rapido".
considere o tempo de init no wasm como nao confirmado por aqui.

o unico numero de init que eu encontrei medido no kdb e de outro contexto: no
adr de deploy do emulador de android, com gpu host via gfxstream e o adapter
aparecendo como apple m4, a init completa em cerca de 700ms. e um numero util
pra ter uma ordem de grandeza do custo de subir adapter, device e pipelines,
mas e do emulador de android com gfxstream, nao do navegador e nao de hardware
real. nao use ele como proxy do tempo de boot no browser; use ele so como
lembrete de que a inicializacao da gpu nao e instantanea, e que por isso ela
nao pode ficar no meio do caminho do primeiro frame, bloqueando.

o que da pra afirmar com seguranca, porque esta no codigo e no comportamento, e
mais qualitativo e mais importante: no desktop o `init_gpu` retorna com a gpu
pronta, e no wasm ele retorna com a gpu ainda a caminho. essa diferenca e a
unica coisa que o async precisa modelar, e o `EventLoopProxy` e como o codigo
modela ela sem um runtime no meio.

## fechando

o preto da tela era um deadlock disfarcado de silencio. a thread principal do
navegador estava presa esperando uma promise que so resolveria se a thread
fosse solta. a correcao nao foi um patch no ponto onde a tela ficava preta, foi
mudar quem conduz o future: parar de bloquear a thread e devolver ela pro
browser, deixar a gpu nascer no microtask queue, e trazer o resultado de volta
pelo unico canal que sobrevive ao closure `'static`, o `EventLoopProxy`.

o codigo que faz isso e pequeno. um enum `AppEvent` com um `GpuReady` que
carrega o contexto inteiro. um `Option<EventLoopProxy<AppEvent>>` no struct,
tomado com `.take()`. um `spawn_local` que faz `await` na init e `send_event`
no fim. um `user_event` que instala o estado, re-sincroniza o viewport e pede
um redraw. e no desktop, ao lado, um `pollster::block_on` que faz a mesma
historia de um jeito sincrono. duas regioes `cfg`, um construtor `async`, zero
runtime pesado.

e por isso que vale entender a peca em vez de so copiar ela. o
`EventLoopProxy` nao e um truque do plev, e o jeito que o winit te deixa
costurar codigo async de qualquer fonte de volta no event loop. a gpu e o caso
que forcou ele aqui, mas o mesmo padrao serve pra qualquer coisa que nasce fora
da pilha do handler e precisa voltar pra app: download que terminou, arquivo
que abriu, resposta de rede. async, no fim, e so isso: levar um future ate o
fim com o executor que a plataforma ja te deu, e ter um canal de volta. o resto
e peso que voce nao precisa carregar.

## rastros

cada afirmacao tecnica deste capitulo, com file:line da fonte conferida no
repo.

- `GpuContext::new` e `async`, com `.await` no adapter e no device:
  `crates/engine/src/gpu/context.rs:58` (assinatura `pub async fn new`),
  `:69` (`pub async fn new_with_config`), `:83-90` (`request_adapter(...).await`),
  `:95-108` (`request_device(...).await`).
- backends por target (`PRIMARY` no native, `BROWSER_WEBGPU` no wasm):
  `crates/engine/src/gpu/context.rs:74-80`.
- `required_limits: wgpu::Limits::default()` nos dois targets:
  `crates/engine/src/gpu/context.rs:99-102`.
- formato de surface e encode srgb decididos na init:
  `crates/engine/src/gpu/context.rs:115-134`.
- `surface_format()` retorna o view format srgb quando a surface e nao-srgb:
  `crates/engine/src/gpu/surface.rs:122`.
- caminho desktop com `pollster::block_on` e estado `Ready` imediato:
  `crates/engine/src/window/state.rs:34-46`.
- caminho wasm com `event_loop_proxy.take()`, `spawn_local`, `await`,
  `send_event(AppEvent::GpuReady{...})`:
  `crates/engine/src/window/state.rs:48-65`.
- assinatura de `init_gpu` e scale factor / safe area comuns aos dois targets:
  `crates/engine/src/window/state.rs:30-33`.
- enum `AppEvent::GpuReady` carregando gpu, text_system, effect_processor,
  texture_pool: `crates/engine/src/window/mod.rs:38-45`.
- campo `event_loop_proxy: Option<EventLoopProxy<AppEvent>>` atras de
  `cfg(any(wasm32, android))`: `crates/engine/src/window/mod.rs:71-72`.
- construtor `new_with_proxy` cfg-gated: `crates/engine/src/window/mod.rs:180-181`.
- `ApplicationHandler::user_event` repassando pro handler:
  `crates/engine/src/window/mod.rs:266-268`.
- `handle_user_event` instalando `GpuState::Ready` e pedindo redraw:
  `crates/engine/src/window/lifecycle.rs:138-158`.
- `resumed` chamando `init_gpu` e `setup_wasm_canvas` no wasm:
  `crates/engine/src/window/lifecycle.rs:42-45`, `:62`.
- `setup_wasm_canvas` definicao e reexport sob `cfg(wasm32)`:
  `crates/engine/src/window/lifecycle.rs:206-244`,
  `crates/engine/src/window/mod.rs:10-11`.
- showcase: `UserEvent::GpuReady` com `allow(dead_code)` fora do wasm:
  `crates/showcase/src/app.rs:30-40`.
- showcase: caminho desktop `block_on` e wasm `spawn_local`/`send_event`:
  `crates/showcase/src/app.rs:146-158`, `:162-176`.
- showcase: `user_event` re-sincronizando viewport via `configure_viewport`:
  `crates/showcase/src/app.rs:179-199`, `:102-115`.
- showcase: `run` (desktop, `run_app`) vs `run_web` (wasm, `create_proxy` +
  `spawn_app`) vs `android_main` (`with_android_app` + `run_app`):
  `crates/showcase/src/app.rs:341-345`, `:352-362`, `:368-387`.
- adr: browser nao pode bloquear; adapter/device sao promises:
  `kdb/adr/async-gpu-init-and-single-wasm-entry.md:18-21`.
- adr: decisao do `spawn_local` + `EventLoopProxy` `GpuReady` + re-sync de
  viewport: `kdb/adr/async-gpu-init-and-single-wasm-entry.md:29-32`.
- adr: um `#[wasm_bindgen(start)]` por modulo e feature `web-entry` off por
  padrao: `kdb/adr/async-gpu-init-and-single-wasm-entry.md:22-23`, `:33-36`.
- adr: canvas 100vw/100vh, resizes como `WindowEvent::Resized`:
  `kdb/adr/async-gpu-init-and-single-wasm-entry.md:24`, `:37-39`.
- adr: deps por target (pollster/env_logger native, wasm-bindgen/console web):
  `kdb/adr/async-gpu-init-and-single-wasm-entry.md:40-41`.
- adr: consequencia "mesma fonte, duas regioes cfg-gated, sem fork":
  `kdb/adr/async-gpu-init-and-single-wasm-entry.md:46-48`.
- validacao: closure `'static`, `self` nao move, gpucontext descartado, estado
  preso: `kdb/adr/wasm-webgpu-validation.md:13-14`.
- validacao: os quatro passos do padrao eventloopproxy:
  `kdb/adr/wasm-webgpu-validation.md:17-23`.
- validacao: api winit 0.30 (`with_user_event`, `create_proxy`, `send_event`
  retorna `Result<(), EventLoopClosed<T>>`, `user_event`):
  `kdb/adr/wasm-webgpu-validation.md:24-28`.
- validacao: limits webgpu vs webgl (`downlevel_webgl2_defaults` limita a 2048,
  `default` ate 8192): `kdb/adr/wasm-webgpu-validation.md:30-34`.
- feature `web-entry` off por padrao no manifesto do engine:
  `crates/engine/Cargo.toml` (bloco `[features]`, `web-entry = []` com
  comentario).
- versoes do lock: wgpu 28, winit 0.30, pollster 0.4, wasm-bindgen-futures 0.4:
  `Cargo.toml:50`, `:51`, `:83`, `:91`.
- numero de init (nao-wasm, contexto de emulador android com gfxstream, apple
  m4, init em ~700ms): `kdb/adr/android-emulator-deploy.md:31`.
- nao confirmado: nenhuma das ancoras registra um benchmark de tempo de init
  async da gpu no navegador.
