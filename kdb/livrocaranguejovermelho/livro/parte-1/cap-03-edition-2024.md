---
title: "edition 2024 na pratica: #[unsafe(no_mangle)] e as entradas unicas"
parte: 1
status: rascunho
rastros:
  - crates/showcase/src/app.rs
  - kdb/adr/workspace-engine-at-root-libs-in-crates-demos-in-examples.md
  - kdb/adr/index.md
---

# edition 2024 na pratica: #[unsafe(no_mangle)] e as entradas unicas

voce escreveu `#[no_mangle]` mil vezes. sempre funcionou. ai voce migra o
projeto pra edition 2024, roda o build de android, e o compilador para voce
no meio do caminho com um erro que parece picuinha: `unsafe attribute used
without unsafe`. a sua primeira reacao e a minha tambem: por que diabos um
atributo que so diz "nao renomeie esse simbolo" virou unsafe? eu nao to
desreferenciando ponteiro nenhum aqui.

esse capitulo e sobre esse atrito. nao o atrito sintatico, que se resolve
trocando `#[no_mangle]` por `#[unsafe(no_mangle)]` e seguir a vida. o atrito
de verdade e o que esse `unsafe` esta tentando te dizer, e ele aparece
inteiro quando voce olha pra superficie de ffi de um app real que tem que
existir como quatro binarios diferentes ao mesmo tempo. no plev esse app e o
showcase: uma galeria de widgets que roda em desktop, no browser via
webgpu/wasm, no android e no ios, a partir do mesmo `crates/showcase/src/
app.rs`. quatro plataformas, quatro entradas, e duas delas exigem que voce
assine o tal contrato unsafe na cara.

vou usar o showcase como demo justamente porque ele tensiona a regra ate o
osso. ele nao e um exemplo de tutorial que so tem um `fn main`. ele tem
quatro pontos de entrada que precisam coexistir no mesmo arquivo, e dois
deles tocam a borda do mundo rust, o ponto onde o seu codigo deixa de ser
chamado por outro codigo rust e passa a ser chamado pelo gameactivity do
android ou pelo `main.m` em objective-c do ios. e ali, na borda, que a edition
2024 resolveu te obrigar a parar e escrever a palavra.

## o que a edition 2024 mudou, e o que ela nao mudou

primeiro o fato seco, conferido no `Cargo.toml` do workspace. o projeto roda
edition 2024 e fixa a toolchain minima:

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
```

todo crate herda isso com `edition.workspace = true`, inclusive o showcase. e
o que a edition 2024 trouxe pra essa conversa especifica: alguns atributos que
antes voce escrevia direto agora exigem o wrapper `unsafe(...)`. `no_mangle` e
o caso classico. o que era

```rust
#[no_mangle]
pub extern "C" fn minha_funcao() {}
```

passou a ser

```rust
#[unsafe(no_mangle)]
pub extern "C" fn minha_funcao() {}
```

a nota disso ja esta registrada no proprio kdb, na linha de cross-compile de
android do index de adr: "NDK 27.2, cargo-ndk v4.1.2, #[unsafe(no_mangle)]
requerido em edition 2024, game-activity feature necessaria". ou seja, isso
nao e teoria de release notes, e uma coisa que a frota tropeçou, anotou e
seguiu. (nao confirmado: o numero exato do RFC que estabilizou os "unsafe
attributes". o que esta confirmado, e e o que importa pro app, e a versao da
edition, a toolchain 1.85, e o fato de que todo `no_mangle` do repo aparece
ja na forma envelopada.)

o que a edition 2024 nao mudou e a semantica. `#[unsafe(no_mangle)]` faz
exatamente o que `#[no_mangle]` fazia: desliga o name mangling do simbolo, de
forma que a funcao apareça na tabela de simbolos do binario com o nome literal
que voce deu, e nao com o hash que o compilador normalmente gera. o
comportamento e identico. a mudanca e so que agora voce precisa dizer
explicitamente que sabe o que esta fazendo. menos magia, mais contrato.

## por que esse atributo e unsafe

a parte que incomoda e justamente a palavra. desreferenciar ponteiro cru e
unsafe porque pode crashar. transmutar tipo e unsafe porque pode corromper
memoria. mas `no_mangle`? ele nao toca memoria nenhuma. e ai que ta o ponto
arquitetural: o `unsafe` aqui nao protege a memoria, ele protege o espaço de
nomes do linker.

quando voce desliga o mangling, voce assume a responsabilidade pela unicidade
do simbolo. o compilador mangla os nomes exatamente pra garantir que duas
funcoes chamadas `new` em modulos diferentes nunca colidam no binario final.
ao escrever `no_mangle`, voce diz: confia em mim, esse nome `android_main` e
unico, ninguem mais vai exportar um simbolo com esse nome, e se alguem
exportar o linker vai te dar um erro de simbolo duplicado, ou pior, vai pegar
o errado em silencio. isso e uma invariante que o compilador nao consegue
checar sozinho, porque ela atravessa a fronteira do crate, atravessa o ffi,
chega no `main.m` ou no manifesto do android. o `unsafe` e o reconhecimento de
que voce, e nao o borrow checker, e o responsavel por essa garantia.

isso casa com a postura do workspace inteiro. os lints herdados deixam claro
que unsafe aqui e levado a serio:

```toml
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "warn"
```

o repo nao trata unsafe como detalhe sintatico. ele avisa quando voce faz
operacao unsafe dentro de fn unsafe sem bloco explicito, justamente pra que o
escopo do perigo fique sempre delimitado. `#[unsafe(no_mangle)]` e a mesma
filosofia aplicada a atributo: o perigo tem que estar visivel no ponto onde
ele acontece.

## o crate que vira tres coisas ao mesmo tempo

antes de olhar as entradas, vale entender por que o showcase precisa de mais
de uma. um `fn main` resolveria desktop. mas android nao chama `main`, e ios
nao chama `main` da forma que voce pensa. cada plataforma tem o proprio jeito
de pegar o seu codigo e dar partida nele, e o jeito muda o formato do binario
que voce tem que produzir. o `Cargo.toml` do showcase declara os tres
formatos de uma vez:

```toml
[lib]
crate-type = ["lib", "cdylib", "staticlib"]

[[bin]]
name = "showcase"
path = "src/main.rs"
```

o comentario logo acima dessa secao, no arquivo real, explica cada um sem
rodeio: o `lib` (rlib) e o que o binario de desktop e de web linkam; o
`cdylib` e o `.so` do android, que o gameactivity carrega e de onde ele chama
`android_main`; o `staticlib` e o `.a` do ios, linkado pelo app do xcode, que
chama `showcase_ios_main`. um arquivo de codigo, tres artefatos de saida, mais
o binario do desktop. e exatamente por isso que as entradas precisam coexistir
no mesmo `app.rs` separadas por `#[cfg(...)]`: o mesmo source compila pra
quatro alvos, e cada alvo so enxerga a entrada que faz sentido pra ele.

o cabecalho do modulo `app.rs` ja anuncia esse desenho logo na primeira
linha: ele e o `ApplicationHandler` do winit que possui a janela, o estado de
gpu, o compositor e a `ShowcaseView`, "plus the per-platform entry points
(desktop `run`, web `run_web`, android `android_main`, iOS
`showcase_ios_main`)". a view fica winit-free; esse modulo e o unico que toca
o event loop. essa frase e o contrato do capitulo inteiro: quatro entradas,
um lugar so.

## android_main: o simbolo que o gameactivity procura pelo nome

vamos ao caso mais cabeludo primeiro, porque e o que motivou a anotacao no
kdb. essa e a entrada de android do showcase, copiada do arquivo:

```rust
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("plev-showcase"),
    );

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

repara em duas coisas. a funcao e `pub fn`, sem `extern "C"`. e ela carrega o
`#[unsafe(no_mangle)]`. por que essa combinacao? porque quem chama
`android_main` aqui nao e codigo C escrito por voce, e a cola do backend
`android-game-activity` do winit, que e rust compilado na mesma toolchain. a
chamada acontece dentro do mundo rust, entao a abi nao precisa virar C. mas o
backend acha a sua funcao pelo nome de simbolo, nao pelo caminho de modulo
rust, e e por isso que o `no_mangle` continua obrigatorio: se o nome fosse
manglado, a cola procuraria `android_main` e nao acharia.

e quem amarra esse nome ao app de verdade? o `AndroidManifest.xml`. o
gameactivity precisa saber qual biblioteca nativa carregar, e ele descobre por
um meta-data:

```xml
<meta-data
    android:name="android.app.lib_name"
    android:value="showcase" />
```

o comentario do proprio manifesto diz: "android.app.lib_name selects the
native lib (libshowcase.so); winit drives the surface from android_main". o
caminho completo do contrato fica visivel quando voce junta as pecas: o
manifesto aponta pra `showcase`, o sistema carrega `libshowcase.so`, e dentro
dele procura o simbolo `android_main`, que so existe com esse nome literal
porque voce desligou o mangling. tire o `#[unsafe(no_mangle)]` e o `.so`
compila, empacota, instala, e o app abre numa tela preta porque o simbolo que
deveria dar a partida virou um hash que ninguem procura.

o script de build fecha o argumento com um detalhe que eu acho bonito de
honesto. o `build_android.sh` compila o cdylib do showcase via cargo-ndk e
depois faz isso:

```bash
find "$JNILIBS" -name 'libengine.so' -delete
```

ou seja: o cargo-ndk gera tambem um `libengine.so`, porque o engine tambem e
um cdylib. mas esse `.so` e peso morto. o `libshowcase.so` ja linka o engine
estaticamente, e o gameactivity so carrega `showcase`. entao o script apaga o
`libengine.so` do diretorio de jniLibs antes de montar o apk. esse delete e a
evidencia fisica da regra de entrada unica: existe exatamente um `.so` que o
android carrega, com exatamente um `android_main`, e qualquer outro simbolo de
entrada que vaze pra dentro do pacote e lixo na melhor hipotese, colisao na
pior. (o NDK usado e o 27.2.12479018, fixado no proprio script via
`ANDROID_NDK_HOME`; bate com a anotacao de "NDK 27.2" do index.)

## showcase_ios_main: o extern "C" que o main.m chama

agora o contraste que deixa a logica do `extern "C"` cristalina. essa e a
entrada de ios:

```rust
#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn showcase_ios_main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("ios event loop");
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
```

aqui tem `extern "C"`, e a android nao tinha. a diferenca nao e estilo, e
quem esta do outro lado da chamada. no android o chamador e rust. no ios o
chamador e objective-c. o app do xcode tem um `main.m` minusculo, e ele e
literalmente isto:

```c
extern void showcase_ios_main(void);

int main(int argc, char *argv[]) {
    @autoreleasepool {
        showcase_ios_main();
    }
    return 0;
}
```

esse `main.m` declara `showcase_ios_main` como uma funcao C externa e a chama
de dentro do `@autoreleasepool`. pra essa chamada atravessar a fronteira de
linguagem sem corromper a pilha, os dois lados tem que concordar na abi, e a
abi comum entre rust e objective-c e a do C. dai o `extern "C"`: ele fixa a
convencao de chamada. e o `#[unsafe(no_mangle)]` de novo cumpre o outro papel:
o `main.m` referencia a funcao pelo nome `showcase_ios_main`, entao o nome no
binario rust tem que ser exatamente esse, sem hash. o comentario do `main.m`
resume o desenho: "winit owns the iOS application lifecycle: showcase_ios_main
(exported by the Rust staticlib) builds the winit event loop and calls
UIApplicationMain, which never returns. This main is just the C entry the
linker expects".

junta os dois casos e o `unsafe(no_mangle)` revela as duas garantias que ele
empacota numa palavra so. a primeira e estabilidade de nome: alguem fora do
rust (o manifesto, o `main.m`, o linker) vai te chamar por um nome literal, e
voce promete manter esse nome. a segunda e unicidade: voce promete que esse
nome nao colide com nenhum outro no binario final. o android estressa a
segunda (por isso o delete do `libengine.so`). o ios estressa a primeira (por
isso o `main.m` que so existe pra ter um simbolo C pra chamar). nos dois, o
compilador nao tem como verificar a promessa, porque ela vive do lado de fora.
o `unsafe` e a sua assinatura embaixo dela.

uma digressao de campo que vale anotar, porque ja custou tempo na frota: no
ios o `cargo check` passa e o `cargo build` falha se voce nao tiver o
Xcode.app completo, so as command line tools. o index de adr registra isso
direto: "cargo check ok mas cargo build falha sem xcode.app (uikit framework
nao encontrado com cli tools only)". e o tipo de coisa que parece bug de
codigo e e ambiente: o `showcase_ios_main` esta perfeito, o que falta e o
framework do sistema pra linkar contra. se voce bater nisso, nao mexe no
`app.rs`, instala o xcode.

## run e run_web: as entradas que nao precisam assinar nada

pra fechar o quadrante, as duas entradas que nao carregam `no_mangle`, porque
nenhuma das duas atravessa a borda do mundo rust pela tabela de simbolos. a de
desktop:

```rust
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
```

`run` e uma `pub fn` rust comum. quem chama ela e o `src/main.rs` do binario,
que e rust, pelo caminho de modulo normal. o compilador resolve a chamada em
tempo de compilacao, o nome pode ser manglado a vontade, ninguem de fora
procura por `run` na tabela de simbolos. zero ffi, zero contrato com o linker,
zero unsafe.

a de browser tem mais cerimonia, mas pela mesma razao continua sem
`no_mangle`:

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

`run_web` usa `spawn_app` em vez de `run_app` por um motivo que esta no
comentario do arquivo: no browser, `run_app` jogaria uma excecao pra escapar
do `main`, entao voce entrega o proxy do event loop pro app e deixa o loop do
proprio browser manter as coisas vivas. e tambem por isso que a inicializacao
de gpu no wasm e assincrona: o `resumed` dispara a criacao do `GpuContext`
numa task e o resultado volta por um `UserEvent::GpuReady` no proxy, enquanto
no desktop e no mobile o `resumed` bloqueia em `GpuContext::new` ali mesmo.
mas nada disso e ffi por simbolo. no wasm a ponte com o javascript e o
`wasm-bindgen`, que tem o proprio mecanismo de export, e a entrada do engine
no browser usa `#[wasm_bindgen(start)]`, nao `no_mangle`. a fronteira existe,
mas ela e mediada por outra ferramenta, entao o contrato muda de forma.

o mapa fica simples quando voce o organiza pela pergunta "quem me chama de
fora do rust, e por nome de simbolo?". desktop nao, web nao (o bindgen
intermedia), android sim, ios sim. as duas que respondem sim sao exatamente as
duas com `#[unsafe(no_mangle)]`. a edition 2024 nao inventou essa divisao, ela
so passou a exigir que voce a torne explicita nas duas onde ela ja existia.

## a regra da entrada unica: por que o engine esconde a propria android_main

ate aqui olhei o showcase, que e um app. mas a regra de entrada unica tem uma
consequencia que so aparece quando voce tem mais de um crate capaz de exportar
a mesma entrada. o engine, em `crates/engine`, tambem sabe rodar sozinho. ele
tem a propria `android_main` e a propria entrada de web. se as duas estivessem
sempre ligadas, qualquer app que linkasse o engine herdaria um segundo
`android_main`, e o cdylib teria dois simbolos com o mesmo nome literal. o
linker reclamaria, ou pegaria um dos dois ao acaso.

a solucao no repo e feature-gating. o `Cargo.toml` do engine declara as
entradas como features desligadas por padrao:

```toml
# Exports the engine's own `#[wasm_bindgen(start)]` entry point on wasm32.
# Off by default so downstream apps (showcase, ide, ...) can define
# their own browser entry without colliding with the engine's.
web-entry = []
android-game-activity = ["winit/android-game-activity"]
# Exports the engine's own `android_main` entry on android. Off by default so
# downstream apps can ship their own GameActivity entry without symbol clash.
android-entry = []
```

e a `android_main` do engine, em `crates/engine/src/lib.rs`, fica atras desse
gate:

```rust
#[cfg(all(target_os = "android", feature = "android-entry"))]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    // ...
    log::info!("plev android_main started");
    // ...
}
```

o comentario logo acima dela diz a regra inteira, sem que eu precise
parafrasear: "a cdylib exports exactly one android_main symbol, so a
downstream app (showcase, ide, ...) that ships its own GameActivity entry must
be able to opt out and define its own". esse e o porque arquitetural por tras
do delete do `libengine.so` no script de build. nao e que o engine seja
incapaz de rodar, e que so pode existir um `android_main` por `.so`, entao o
engine entrega o dele desarmado por padrao, e quem quiser uma entrada e quem a
arma. o showcase nunca liga `android-entry`; ele liga `android-game-activity`
(o backend do winit) e exporta a propria `android_main` no `app.rs`. olha o
`Cargo.toml` do showcase pra confirmar que ele puxa o backend mas nao a
entrada do engine:

```toml
[target.'cfg(target_os = "android")'.dependencies]
engine = { workspace = true, features = ["android-game-activity"] }
android_logger.workspace = true
```

`android-game-activity` esta na lista. `android-entry` nao. e exatamente o que
o desenho pede: o app pega a glue do winit, mas guarda pra si o direito de ser
a unica entrada. o engine faz o mesmo movimento no web com `web-entry`,
gateando seu `#[wasm_bindgen(start)]`, porque um modulo wasm so pode ter um
ponto de start. a simetria entre as duas plataformas nao e coincidencia, e a
mesma invariante (uma entrada por artefato) aplicada a dois mecanismos
diferentes de export.

esse e o tipo de decisao que parece exagero ate o dia em que voce adiciona o
segundo app. o ide, tambem em `crates/`, e um cliente de git de verdade, e
roda na mesma engine. no dia em que ele ganhar um shell de android, ele vai
exportar o proprio `android_main`, e vai poder, porque o engine nunca impos o
dele. a entrada unica deixou de ser uma regra sobre o showcase e virou uma
propriedade do engine: ele e linkavel por qualquer numero de apps sem nunca
brigar por simbolo de entrada com nenhum deles.

## digressao: name mangling, a tabela de simbolos e o custo do no_mangle

vale a pena descer um nivel pra entender por que o mangling existe, porque ai
o `unsafe` para de parecer arbitrario. rust permite duas funcoes `new` em
modulos diferentes, generics monomorfizados, metodos de trait com o mesmo nome
em tipos diferentes. tudo isso, no fim, vira simbolo no binario. se os nomes
fossem literais, `new` colidiria com `new` na primeira oportunidade. o
compilador entao mangla: ele codifica o caminho completo, os tipos, a
assinatura, num nome unico e feio que ninguem precisa ler. o efeito colateral
bom e que voce nunca pensa em colisao de simbolo escrevendo rust normal,
porque o mangling te isola disso.

`no_mangle` desliga esse isolamento pra um simbolo especifico. voce ganha um
nome literal, estavel, chamavel de fora. voce perde a rede de protecao. a
partir dali, garantir que `android_main` e unico no binario e trabalho seu, e
e um trabalho que o compilador nao tem informacao pra fazer, porque a outra
ponta do contrato (o manifesto, o `main.m`, o backend do winit) nao esta no
codigo que ele compila. e literalmente uma promessa sobre algo fora do alcance
do type system. essa e a definicao operacional de por que e `unsafe`: nao
porque crasha agora, mas porque move uma garantia que era do compilador pra
sua responsabilidade, e essa garantia, se quebrada, quebra de um jeito que so
o linker ou o runtime descobre.

e o custo de runtime disso? aqui e onde eu seria desonesto se inventasse um
numero. nao existe benchmark de superficie de ffi no kdb, e nao deveria
existir, porque essas entradas sao chamadas uma vez por processo. `android_
main` roda na partida e o que ela faz de caro e construir o event loop e a
gpu, nao o overhead de simbolo, que e zero apos o link. os benchmarks que o
repo de fato mantem sao de caminho quente de render (push de retangulos,
tessellation, dirty tracking, ciclos de signal), e nenhum deles toca a borda
de ffi. (nao confirmado como relevante aqui: os numeros de
`benchmark-results.md`; eles existem e sao reais, mas medem o render, nao a
entrada.) o custo real do `no_mangle` nao e tempo, e disciplina: e o
`libengine.so` que vira peso morto e precisa ser apagado, e o segundo
`android_main` que precisa ficar atras de uma feature. o preco se paga na
tabela de simbolos e no tamanho do pacote, nao no relogio.

## o que a doc ainda nao conta: o engine saiu da raiz

uma honestidade de rastro, porque o pedido foi conferir tudo contra a fonte e
marcar o que diverge. uma das ancoras desse capitulo e o adr "engine at root,
libraries and apps in crates, demos in examples". esse adr, com status
accepted e data 2026-06-12, descreve o engine como o crate raiz `plev`, com o
`[package]` no topo do workspace e o `src` sendo o engine. ele diz, com todas
as letras: "the engine is the root crate `plev` ([package] at the workspace
root, src is the engine)".

so que o `Cargo.toml` que eu li nesta branch (`refactor/workspace-
restructure`) nao tem `[package]` na raiz. ele tem so `[workspace]`, e o engine
aparece como um member em `crates/engine`, com `name = "engine"` no proprio
manifesto. ou seja: a reestruturacao ja moveu o engine pra dentro de
`crates/`, e o adr ainda descreve o layout anterior, em que ele morava na
raiz. nao e contradicao de uma estar errada, e o adr documentando um estado e
a branch ja estando no meio do proximo. quem ler o adr e o codigo no mesmo dia
vai notar o descompasso. registro aqui pra que o rastro seja honesto: a
decisao de "tres tiers, uma regra cada" (engine, libs/apps em `crates/`, demos
em `examples/`) vale; o detalhe de "engine na raiz" foi superado nesta branch
e o adr ainda nao foi atualizado pra refletir isso.

tem um segundo rastro de renomeacao que ainda mora no codigo, e ele toca
direto o tema do capitulo. o comentario de doc da `android_main` do showcase,
no `app.rs`, fala do backend "enabled via the `plev/android-game-activity`
feature". mas o `Cargo.toml` do showcase ativa a feature via o crate `engine`,
nao `plev`: `engine = { workspace = true, features = ["android-game-
activity"] }`. o historico de commits explica: teve um rebrand do naming
inicial (a letra grega φ virou plev) e depois a reorganizacao das pastas por
crates. o comentario ficou para tras citando `plev/` enquanto o crate hoje se
chama `engine`. nao muda o build (a feature existe e e a mesma), mas e o tipo
de tell que confunde quem chega depois, e por isso fica marcado: o comentario
do `app.rs` diz `plev/...`, o manifesto diz `engine`, e o que vale e o
manifesto.

## fechando: o gate e a checklist real

o que eu levaria desse capitulo pra dentro do meu proprio codigo, sem o
floreio:

a edition 2024 nao te complicou a vida de graca. ela pegou a unica linha do
seu app que faz uma promessa que o compilador nao consegue verificar, a linha
que exporta um simbolo por nome pra alguem de fora do rust, e te obrigou a
escrever `unsafe` em cima dela. no showcase, essas linhas sao duas, e exatas:
a `android_main`, chamada pelo gameactivity via `android.app.lib_name` no
manifesto, e a `showcase_ios_main`, chamada pelo `main.m` em objective-c por
abi C. as outras duas entradas, `run` no desktop e `run_web` no browser, nao
exportam simbolo por nome pra ninguem de fora, entao nao carregam `no_mangle`,
e a edition 2024 nao tem nada a cobrar delas.

a regra que sustenta isso tudo e uma so: um artefato, uma entrada. um `.so`
com um `android_main`, um modulo wasm com um `start`, um `.a` com um
`showcase_ios_main`. o engine respeita a regra deixando as proprias entradas
atras de features desligadas (`android-entry`, `web-entry`), o app respeita
exportando as suas e nao ligando as do engine, e o build respeita apagando o
`libengine.so` que sobra. quando voce mexer em qualquer coisa perto dessa
borda, o gate do projeto continua valendo: `cargo test --workspace`, `cargo
clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, e o
mais barato pra pegar regressao cross-platform, `cargo check --target
wasm32-unknown-unknown -p showcase`. esse ultimo nao compila android nem ios,
mas e o guarda que mais cedo te avisa quando uma das quatro entradas saiu da
linha.

e se um dia voce abrir o `app.rs` e quiser adicionar uma quinta plataforma, o
arquivo ja te ensina a forma: um `#[cfg(...)]` que isola o alvo, uma `fn` que
constroi o event loop e chama `run_app`, e, so se alguem de fora do rust for
te chamar por nome, o `#[unsafe(no_mangle)]` com a assinatura que voce agora
sabe ler como um contrato, e nao como uma chatice do compilador.

## rastros

- edition 2024 e toolchain minima: `Cargo.toml:22-24` (version 0.1.0, edition
  2024, rust-version 1.85)
- `#[unsafe(no_mangle)]` requerido em edition 2024 (anotacao da frota):
  `kdb/adr/index.md:27`
- lint de unsafe explicito no workspace: `Cargo.toml:30-31`
  (`unsafe_op_in_unsafe_fn = "warn"`)
- crate-type triplo do showcase (lib, cdylib, staticlib) e bin:
  `crates/showcase/Cargo.toml:12-17`
- as quatro entradas declaradas no cabecalho do modulo:
  `crates/showcase/src/app.rs:1-11`
- `android_main` do showcase (`#[unsafe(no_mangle)]`, `pub fn`, sem
  extern "C"): `crates/showcase/src/app.rs:368-387`
- `showcase_ios_main` do showcase (`#[unsafe(no_mangle)]`, `pub extern "C"`):
  `crates/showcase/src/app.rs:391-399`
- `run` (desktop, sem no_mangle): `crates/showcase/src/app.rs:339-346`
- `run_web` (wasm, spawn_app, sem no_mangle): `crates/showcase/src/app.rs:351-362`
- init de gpu assincrona no wasm vs bloqueante no nativo:
  `crates/showcase/src/app.rs:7-11`, `crates/showcase/src/app.rs:146-176`
- `android.app.lib_name` = showcase no manifesto e libshowcase.so:
  `android/app/src/main/AndroidManifest.xml:35-37` (meta-data) e o comentario
  adjacente sobre android_main
- delete do `libengine.so` como peso morto no build de android, e NDK 27.2:
  `android/build_android.sh` (bloco do find -delete e `ANDROID_NDK_HOME`)
- `main.m` declara e chama `showcase_ios_main` por abi C:
  `ios/showcase/Sources/main.m:8-13`
- staticlib libshowcase.a como origem do simbolo de ios:
  `ios/showcase/project.yml:3-6`
- entrada de ios falha no `cargo build` sem Xcode.app (uikit nao encontrado):
  `kdb/adr/index.md:28`
- features de entrada do engine desligadas por padrao (web-entry,
  android-entry) e o porque do symbol clash: `crates/engine/Cargo.toml:23-30`
- `android_main` do engine atras do gate `android-entry`, com o comentario
  "exactly one android_main symbol": `crates/engine/src/lib.rs:92-121`
- `#[wasm_bindgen(start)]` do engine atras do gate `web-entry`:
  `crates/engine/src/lib.rs:123-145`
- showcase puxa `android-game-activity` mas nao `android-entry`:
  `crates/showcase/Cargo.toml:38-40`
- adr "engine at root" descreve o engine como crate raiz `plev`:
  `kdb/adr/workspace-engine-at-root-libs-in-crates-demos-in-examples.md:22-26`
- divergencia: nesta branch o engine esta em `crates/engine`, sem `[package]`
  na raiz: `Cargo.toml:1-17` (workspace members) e `crates/engine/Cargo.toml:5`
  (`name = "engine"`)
- comentario de doc do showcase ainda cita `plev/android-game-activity`
  enquanto o manifesto usa o crate `engine`:
  `crates/showcase/src/app.rs:366-367` vs `crates/showcase/Cargo.toml:39`
- benchmarks do repo medem render, nao a borda de ffi (contexto, nao numero
  aplicado): `kdb/adr/index.md:53`
- o gate de quatro passos do projeto (referencia operacional, AGENTS.md)
