---
title: "workspace virtual: a crate como fronteira"
parte: 1
status: rascunho
rastros:
  - kdb/adr/workspace-engine-at-root-libs-in-crates-demos-in-examples.md
  - Cargo.toml
  - crates/engine/Cargo.toml
  - crates/rope/Cargo.toml
  - crates/git/Cargo.toml
  - crates/engine/src/lib.rs
  - crates/rope/src/lib.rs
  - crates/git/src/lib.rs
  - crates/engine/examples/snake/main.rs
  - crates/engine/examples/counter/main.rs
  - crates/rope/benches/edit.rs
  - "commit e6f7091 (demos viram exemplos)"
---

# workspace virtual: a crate como fronteira

imagina uma oficina com varias salas. cada sala tem a sua porta, a sua chave, a
sua bancada. elas dividem o predio, a fiacao, o endereco na rua, mas o que entra
e o que sai de cada sala passa por uma porta so. voce nao atravessa a parede pra
pegar uma ferramenta da sala ao lado. voce bate, a porta abre, alguem te entrega
o que esta no balcao. o resto da sala continua fechado, e voce nem sabe como ele
e por dentro.

uma crate em rust e essa sala. o predio inteiro e o workspace. e a parte que
quase ninguem para pra reparar e que essa divisao nao e organizacao de pasta, e
fronteira de verdade, com tres significados ao mesmo tempo: fronteira de api
(o que a sala deixa no balcao), fronteira de compilacao (cada sala e reformada
sozinha) e fronteira de dependencia (a sala A pode chamar a B, mas se a B chamar
a A de volta o predio nao fica de pe). este capitulo abre nessa imagem da
oficina e desce ate o `Cargo.toml` que nao tem pacote, ate as crates `rope` e
`git` que nao sabem o que e uma GPU, e ate o commit que apagou quase oito mil
linhas movendo dois demos pra dentro de uma pasta de exemplos.

## o manifesto que nao tem pacote

a primeira surpresa, pra quem abre o `Cargo.toml` da raiz esperando ver um
projeto, e que nao tem projeto nenhum ali. tem isto:

```toml
[workspace]
resolver = "2"
# Todos os crates do projeto sao members (aparecem como targets na IDE).
members = [
    "crates/engine",
    "crates/macros",
    "crates/git",
    "crates/ide",
    "crates/lot",
    "crates/monster",
    "crates/narrate",
    "crates/narrate-macro",
    "crates/parser",
    "crates/prime",
    "crates/rope",
    "crates/showcase",
]
```

repara no que nao esta la. nao tem `[package]`. nao tem `name`, nao tem
`version`, nao tem um `src/main.rs` ou `src/lib.rs` que esse arquivo descreva.
isso e um manifesto de workspace virtual: um `Cargo.toml` cuja unica funcao e
declarar que aquela pasta e a raiz de um conjunto de crates e listar quem sao os
membros. ele nao compila nada por si. ele coordena.

o detalhe que muda tudo e o contraste com como o projeto comecou. o adr que
registrou a reorganizacao, `workspace-engine-at-root-libs-in-crates-demos-in-examples`,
abre com a engine como crate raiz: "the engine is the root crate `plev`
(`[package]` at the workspace root, src is the engine)". ou seja, na decisao
original o `Cargo.toml` da raiz tinha sim um `[package]`, e a engine morava no
`src/` da raiz. o que voce ve hoje na branch de reestruturacao e o passo
seguinte, que o adr ainda nao reescreveu: a engine desceu pra `crates/engine`,
virou mais uma sala da oficina, e a raiz ficou puramente virtual. marco isso
explicito porque o adr e o codigo divergem nesse ponto, e a fonte de verdade pra
este capitulo e o `Cargo.toml` que esta no disco, nao o paragrafo do adr que
descreve um estado anterior.

por que descer a engine pra ser uma crate como as outras, em vez de deixar ela
reinando na raiz? porque uma raiz virtual deixa todas as salas simetricas. nao
existe mais a sala principal e as salas acessorias. existe `crates/engine`,
`crates/rope`, `crates/git`, e cada uma e olhada pela mesma regua. a engine
continua sendo a coisa de que todo mundo depende, mas isso e uma propriedade do
grafo de dependencia, nao do lugar dela no sistema de arquivos. a hierarquia
fica nos `[dependencies]`, onde ela e checada pelo compilador, e nao na
profundidade da pasta, onde nao e checada por ninguem.

## uma versao num lugar so

a oficina divide a fiacao. no workspace, a fiacao sao as versoes. abaixo da
declaracao dos membros, o mesmo `Cargo.toml` carrega o bloco que faz as crates
nao brigarem entre si:

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
authors = ["Brenner Cruvinel <brennertalks@gmail.com>"]
repository = "https://github.com/brennercruvinel/plevdev"
```

e logo depois, a fonte unica das versoes de dependencia:

```toml
[workspace.dependencies]
# -- internos (path)
engine = { path = "crates/engine" }
git = { path = "crates/git" }
rope = { path = "crates/rope" }
# -- gpu / janela
wgpu = "28"
winit = { version = "0.30", features = ["rwh_06"] }
# -- texto / layout / vetor
cosmic-text = "0.18"
taffy = "0.9"
ropey = "1.6"
# -- dados / util
gix = { version = "0.84", default-features = false, features = ["sha1", "revision", "parallel"] }
```

cada crate membro, em vez de escrever `wgpu = "28"` no proprio manifesto,
escreve `wgpu.workspace = true`. a versao mora num lugar so. quando o wgpu sai do
28 pro 29, voce edita uma linha na raiz e o predio inteiro anda junto. ninguem
fica pra tras numa versao antiga por esquecimento, e mais importante, ninguem
fica numa versao incompativel sem querer. duas crates do mesmo workspace puxando
duas minor diferentes do mesmo wgpu seria uma fonte de bug surda, do tipo que so
aparece quando os dois tipos `wgpu::Device` que parecem iguais nao sao o mesmo
tipo pro compilador. a regra do projeto fecha esse buraco antes dele existir.

o adr chama isso de higiene de cargo, e lista o conjunto: `[workspace.package]`
pra versao, edition, rust-version, authors e repository compartilhados;
`[workspace.dependencies]` como a fonte unica de verdade de versao;
`[workspace.lints]` herdado por toda crate; `[profile]` ajustado. eu acrescento
um detalhe que o adr nao destaca e que vale reparar na lista de membros versus a
lista de path deps. nem toda crate vira dependencia interna. `engine`, `git`,
`rope` e as outras libs aparecem em `[workspace.dependencies]` com `path`, porque
alguem depende delas. `ide`, `showcase`, `parser`, `prime` nao aparecem ali. sao
folhas. sao apps e leitores, ninguem importa eles. a propria tabela de
dependencia conta quem e biblioteca e quem e produto final, sem precisar de
comentario.

a fiacao tem ainda os lints e os profiles, que tambem descem da raiz:

```toml
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "warn"

[workspace.lints.clippy]
uninlined_format_args = "warn"
```

```toml
[profile.release]
lto = "thin"
codegen-units = 1

[profile.dev.package."*"]
opt-level = 2
```

cada crate herda os lints com um `[lints] workspace = true` de tres linhas, e
ganha a mesma barra: clippy limpo sob `-D warnings`, do jeito que o portao do
projeto exige. o `profile.dev.package."*"` com `opt-level = 2` e o tipo de ajuste
que so faz sentido num workspace com dependencia pesada: ele compila as
dependencias externas otimizadas mesmo em debug, pra que o wgpu nao arraste o
runtime do build de desenvolvimento, enquanto o seu proprio codigo segue
incremental e rapido. uma config, todo membro. e a vantagem concreta da raiz
virtual: o lugar comum existe e e barato de manter.

## a crate como fronteira de api

ate aqui a raiz coordena. a parte boa mora nas salas. abre a porta da `rope` e
le o que esta escrito logo na entrada, no topo do `lib.rs`:

```rust
//! Core text editing model: rope buffer, multi-cursor selections,
//! transactional edits and undo history.
//!
//! This crate has no UI or GPU dependencies -- everything is pure data
//! manipulation, fully testable headless. The design follows Helix:
//! `Document = Rope + Selections + History`, with edits expressed as
//! `Transaction`s that can be inverted, composed and used to map
//! positions (and therefore selections) across edits.
```

"no UI or GPU dependencies". esse comentario nao e decoracao, e uma promessa que
o `Cargo.toml` da `rope` cumpre na unha:

```toml
[package]
name = "rope"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish = false
description = "Rope-based text editing core: document, transactions, multi-cursor, history (pure, no UI)"

[dependencies]
ropey.workspace = true
unicode-segmentation.workspace = true

[dev-dependencies]
proptest.workspace = true
criterion.workspace = true
```

duas dependencias de runtime. um rope de texto e segmentacao de grafema unicode.
mais nada. a `rope` nao conhece `wgpu`, nao conhece `winit`, nao conhece a engine.
ela nao tem como desenhar um pixel mesmo que quisesse, porque nao tem o tipo pra
isso no escopo. e essa impossibilidade que faz dela uma fronteira de api de
verdade, e nao so um modulo bem nomeado. um modulo dentro da engine teria o
universo da engine ao alcance da mao, e mais cedo ou mais tarde alguem chamaria
uma coisa de render no meio da logica de edicao "so dessa vez". a crate tira essa
opcao da mesa. o que a `rope` exporta esta no fim do `lib.rs`, e e tudo o que o
mundo de fora ve dela:

```rust
pub use document::Document;
pub use history::{CommitKind, History, UndoStep};
pub use movement::GoalColumn;
pub use ropey::{self, Rope};
pub use selection::{Selection, SelectionSet};
pub use transaction::{Bias, Edit, Transaction};
```

esse bloco de `pub use` e o balcao da sala. `Document`, `Transaction`,
`Selection`, `History`. um app que edita texto pega esses tipos e monta o resto.
o que esta dentro de `document.rs`, como o rope guarda os bytes, como a
transaction inverte um edit, fica do lado de dentro, livre pra mudar sem avisar
ninguem, contanto que o balcao continue o mesmo. essa e a diferenca operacional
entre `pub` e `pub(crate)`: a crate decide, item por item, o que e contrato
publico e o que e detalhe interno, e o compilador faz valer.

a `git` conta a mesma historia com outro sotaque. o `lib.rs` dela:

```rust
//! Git backend for plev applications.
//!
//! Two layers:
//! - `GitRepo`: clean synchronous API (gix for reads, git CLI for
//!   status/diff/mutations -- see ADR notes in `repo.rs`).
//! - `GitClient`: runs a `GitRepo` on a worker thread; commands in via
//!   channel, results out via callback, so a UI thread never blocks.
//!
//! No UI dependencies -- this crate is testable against real temporary
//! repositories (see `tests/real_repo.rs`).
```

e o manifesto, mais enxuto ainda que o da `rope`:

```toml
[dependencies]
gix.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

uma dependencia de runtime, `gix`, com um conjunto minimo de features ligado na
raiz. de novo, nenhuma UI. o backend de git fala com o repositorio e devolve
`Branch`, `Commit`, `Hunk`, `FileStatus`, `DiffLine`. quem desenha esses dados na
tela e o app `ide`, que e uma sala diferente. a `git` e testavel contra um
repositorio temporario de verdade, headless, justamente porque ela nao arrasta o
mundo grafico junto. "testavel" aqui e consequencia direta da fronteira: uma
crate que so depende de `gix` e `tempfile` voce roda no CI sem janela, sem
device de GPU, sem display server. o teste e barato porque a sala e pequena.

vale notar a direcao das setas, porque ela e o terceiro sentido de fronteira. a
engine depende da `rope` (esta la, `rope.workspace = true`, no `Cargo.toml` da
engine). a `rope` nao depende da engine. a `git` nao depende de nenhuma das duas,
e nem a engine depende da `git`. sao folhas independentes que os apps consomem por
cima. cargo nao deixa esse grafo ter ciclo: se a `rope` tentasse importar a engine
que ja importa a `rope`, o build para com erro de dependencia ciclica. a fronteira
de crate transforma "evite acoplamento circular", que num projeto de modulo unico
e so uma boa intencao, numa regra que o compilador recusa quebrar.

## a crate como fronteira de compilacao

a engine e a sala maior, e o manifesto dela mostra o outro lado da fronteira, o
que tem a ver com como o codigo vira binario. o cabecalho:

```toml
[lib]
crate-type = ["cdylib", "staticlib", "rlib"]
```

uma crate e a unidade que o cargo compila e reusa. a engine declara tres formatos
de saida ao mesmo tempo: `rlib` pra ser linkada por outras crates rust do
workspace, `cdylib` pra virar uma biblioteca dinamica que o shell android carrega,
`staticlib` pra ser embutida no app ios. o mesmo codigo fonte, tres artefatos,
porque tres plataformas pedem formatos diferentes. isso e decisao de crate, mora
no `[lib]`, e nao teria onde existir se a engine fosse so um modulo solto.

e tem as features, que sao o jeito da crate ligar e desligar pedaco de si na hora
de compilar:

```toml
[features]
default = ["accessibility"]
accessibility = ["dep:accesskit", "dep:accesskit_winit"]
web-entry = []
android-entry = []
hot-reload = ["dep:notify", "dep:notify-debouncer-full"]
```

`web-entry` e `android-entry` ficam desligadas por padrao de proposito. o `lib.rs`
da engine guarda, atras delas, os pontos de entrada da propria engine:

```rust
#[cfg(all(target_arch = "wasm32", feature = "web-entry"))]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    // ... cria o event loop e roda o App embutido da engine
}
```

o comentario no codigo explica o porque, e o porque e exatamente uma questao de
fronteira: um modulo wasm so pode ter um `#[wasm_bindgen(start)]`, e um cdylib so
exporta um `android_main`. se a engine exportasse o dela sempre, um app como o
`showcase`, que traz a propria entrada, colidiria de simbolo. a feature deixa o
app dizer "a entrada e minha" e desligar a da engine. a crate e a unidade onde
essa escolha cabe, porque feature e propriedade de crate. de novo, uma coisa que
nao existiria sem a sala ter parede.

a fronteira de compilacao tem um efeito pratico que se sente no dia a dia: quando
voce mexe na `rope`, o cargo recompila a `rope` e quem depende dela, nao o
workspace inteiro do zero. a sala e reformada, o resto do predio fica de pe. num
codigo de crate unica, qualquer toque num arquivo central convida o compilador a
reconsiderar tudo. a divisao em crate da ao cargo uma juncao natural pra cortar o
trabalho, e o ganho aparece toda vez que voce salva um arquivo e espera o build.

um detalhe do manifesto da engine fecha o raciocinio sobre direcao de dependencia,
agora no eixo de teste:

```toml
[dev-dependencies]
narrate.workspace = true
criterion.workspace = true
lot.workspace = true
monster.workspace = true
```

essas crates entram so como `dev-dependencies`. elas existem pra exemplo,
benchmark e teste da engine, e nao pesam no artefato que vai pro android ou pro
ios. um `dev-dependency` nao polui o grafo de runtime. a fronteira separa "o que a
engine precisa pra rodar" de "o que a engine precisa pra se testar", e essa
separacao tambem e cargo enforcando, nao convencao no papel.

## mover demo para example reduz acoplamento

agora a parte que da o titulo do capitulo um sentido concreto, com numero. dois
dos demos do projeto, um cubo 3d girando e um jogo da cobrinha que joga sozinho,
eram crates de verdade. `crates/scene-3d` e `crates/snake-game`, cada um com o seu
`Cargo.toml`, cada um membro do workspace, cada um aparecendo como target na IDE.
o commit `e6f7091`, de mensagem seca "org: demos scene-3d e snake-game viram
exemplos", desfez isso. o `git show --stat` dele conta a historia inteira num
numero:

```
23 files changed, 1 insertion(+), 7864 deletions(-)
```

uma insercao. quase oito mil delecoes. de onde vem tanta linha apagada movendo um
demo? do `Cargo.lock`. cada demo-crate carregava o proprio lockfile resolvido:
`crates/scene-3d/Cargo.lock` com 3904 linhas, `crates/snake-game/Cargo.lock` com
3900. duas resolucoes de dependencia completas e redundantes, versionadas no
repo, pra dois brinquedos que so existem pra mostrar a engine funcionando. o
commit apagou os dois lockfiles, apagou os dois `Cargo.toml`, tirou os dois nomes
da lista de membros da raiz, e moveu o `src/` de cada um pra uma pasta de
exemplos. o codigo dos demos quase nao mudou, as linhas que mudaram de verdade
foram as de plumbing: manifesto, lockfile, membro do workspace.

esse e o acoplamento ficando visivel no momento em que voce o remove. um demo
como crate e um membro: ele aparece no grafo, ele fixa as proprias versoes, ele
tem lockfile proprio, ele pode ser importado por engano por outra crate, ele
entra no `cargo build --workspace`. um demo como exemplo nao e nada disso. cargo
descobre exemplo sozinho, sem `Cargo.toml`, pela convencao de pasta. o adr resume
em uma linha o que isso significa: "cargo discovers them with no Cargo.toml; they
are not products". o exemplo nao e um produto. ele e um arquivo que consome a api
publica da crate de fora, do mesmo lugar de onde um usuario real consumiria, e
prova que a api fecha.

olha o `main.rs` da cobrinha hoje, depois da mudanca, vivendo em
`crates/engine/examples/snake/`:

```rust
//! Snake Game -- auto-playing AI snake built entirely on plev primitives.
//!
//! Run: `cargo run --example snake`
//!
//! Demonstrates: Rect, RoundedRect (SDF), Text, per-frame scene rebuild,
//! timer-based game tick, keyboard input, dirty tracking.
#![allow(dead_code)]

mod rendering;
mod state;
mod ui;

use engine::winit::event_loop::EventLoop;
use rendering::SnakeApp;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = SnakeApp::new();
    event_loop.run_app(&mut app).unwrap();
}
```

a linha que importa e `use engine::winit::event_loop::EventLoop`. o exemplo trata
a engine como dependencia externa, importa pelo nome publico, usa o reexport de
`winit` que a engine oferece (`pub use winit;` no `lib.rs` dela). ele nao alcanca
o interior da engine, so o balcao. o `counter`, outro exemplo, faz o mesmo com a
camada de componente:

```rust
use engine::component::Component;
use engine::compositor::Compositor;
use engine::gpu::GpuContext;
use engine::text::TextSystem;
use engine::winit::application::ApplicationHandler;
```

cada `use engine::...` ali e um teste implicito de que aquele caminho publico
existe e e usavel de fora. quando voce escreve um exemplo, voce esta exercitando a
fronteira de api da crate da posicao de quem consome, que e a unica posicao que
revela se a api e boa. um teste de unidade dentro da crate enxerga `pub(crate)` e
campos privados, e por isso nao sente o atrito que o usuario sente. o exemplo
sente, porque ele esta do lado de fora da parede.

houve um segundo passo que o commit `e6f7091` sozinho nao mostra. naquele commit
os exemplos foram pra `examples/` na raiz, porque a engine ainda era a crate raiz,
como o adr descreve. na reestruturacao seguinte, quando a engine desceu pra
`crates/engine`, os exemplos desceram junto, pra `crates/engine/examples/`. e a
razao e bonita: o exemplo demonstra a engine, entao ele mora dentro da crate que
ele demonstra. o comando pra rodar acompanhou o endereco. o comentario dentro do
arquivo ainda diz `cargo run --example snake`, do tempo da raiz, mas a forma atual
e a que o contrato do projeto registra, `cargo run -p engine --example snake`,
qualquer `crates/engine/examples/<name>/main.rs`. marco a defasagem do comentario
como nao confirmada de propósito, e fica anotada pra quem for ler o arquivo e
estranhar.

## o que so o cargo test pega

tem uma cicatriz nesse processo que o adr teve a honestidade de registrar, e ela
ensina mais que o acerto. a passada mecanica que renomeou as referencias `crate::X`
durante a reorganizacao tocou, por engano, um modulo de dentro de um exemplo: o
proprio `lifecycle` do `counter`. o erro passou batido pelo build comum. nas
palavras do adr: "caught only by `cargo test` because `cargo build --workspace`
does not compile examples".

esse e o tipo de detalhe que so morde quem ja foi mordido. `cargo build
--workspace` nao compila exemplo. exemplo so e compilado por `cargo test`, ou por
`cargo build --examples`. entao um exemplo quebrado fica verde num pipeline que so
roda build, e vermelho num que roda test. a licao virou regra no proprio adr, na
secao de coisas a evitar: nao confie em `cargo build --workspace` pra provar que o
codigo de exemplo compila, so `cargo test` constroi os exemplos. e por isso que o
portao do projeto abre com `cargo test --workspace`, e nao com um simples build.
o exemplo so cumpre o papel de guardiao da fronteira de api se ele de fato for
compilado, e a unica garantia disso e o teste.

ha um eco aqui da fronteira de compilacao da secao anterior. exemplo nao entra no
artefato de runtime, nao e produto, nao pesa no binario final. mas ele precisa
entrar no portao, senao a prova que ele oferece nunca e cobrada. o projeto resolve
isso colocando o exemplo no caminho do `cargo test`, que e o mesmo caminho dos
testes de unidade e dos benchmarks com `harness = false`. a engine declara o
bench assim:

```toml
[[bench]]
name = "scene_build"
harness = false
```

e a `rope` declara o dela, `edit`, do mesmo jeito. o bench da `rope` consome a api
publica exatamente como o exemplo, importando de fora:

```rust
use rope::{Document, Transaction};

fn doc_with(lines: usize) -> Document {
    let mut d = Document::new();
    d.apply(Transaction::insert(0, &big_body(lines)));
    d
}
```

`use rope::{Document, Transaction}`. o benchmark e mais um consumidor externo do
balcao, mais uma prova de que a fronteira fecha. ele mede `rope_build_5k_lines` e
um roundtrip de insert e delete que mantem o tamanho do rope estavel. nao tenho um
numero medido pra citar aqui: a tabela de resultados publicada,
`benchmark-results.md` na versao 0.2, traz scene construction, dirty tracking,
tessellation, signals e text hashing, e nao traz a `rope`. entao o bench existe,
roda, e ancora a api, mas o tempo dele eu deixo em aberto em vez de inventar um
valor. o numero deste capitulo nao e de tempo de execucao, e o `7864` de linhas
que sumiram quando dois demos pararam de ser crate.

## por que assim, e nao tudo num crate so

da pra escrever um projeto inteiro numa crate so. muita gente escreve, e funciona
ate certo tamanho. a pergunta que vale e o que voce ganha quebrando em crates, e a
resposta nao e "organizacao", porque pasta tambem organiza. a resposta e que a
crate move tres garantias do campo da disciplina pessoal pro campo do que o
compilador recusa.

a primeira e a privacidade. dentro de uma crate, `pub(crate)` deixa qualquer
modulo enxergar qualquer outro, e a tentacao de atravessar a parede pra resolver
rapido esta sempre la. entre crates, so o que e `pub` no balcao atravessa. a
`rope` nao tem como chamar render porque o tipo de render nao existe no mundo dela.
a fronteira nao depende de eu ser disciplinado as duas da manha, ela e estrutural.

a segunda e a aciclicidade. a engine depende da `rope`, a `rope` nao depende da
engine, e o cargo nao deixa essa seta inverter sem um erro de build. num codigo de
modulo unico, "nao crie dependencia circular" e conselho. entre crates, e lei
fisica do grafo.

a terceira e a unidade de compilacao e de versao. a crate e o que o cargo
recompila e reusa, e e onde feature, `crate-type` e profile se aplicam. uma raiz
virtual com `[workspace.dependencies]` faz toda crate compartilhar uma versao de
cada dependencia, e ainda assim cada crate escolher as proprias features. a engine
liga `cdylib`, `staticlib`, `rlib` e cinco features. a `rope` nao liga nada disso,
porque nao precisa. a mesma fiacao, bancadas diferentes.

o demo virando exemplo e o caso limpo de onde colocar a fronteira. um demo nao e
biblioteca: ninguem importa o jogo da cobrinha como dependencia. e nao e produto
de verdade: ele existe pra provar a engine. ele e um terceiro tipo, o consumidor
de demonstracao, e o lugar dele e `examples/`, onde o cargo o descobre sem
manifesto, onde ele nao polui o grafo, onde ele exercita a api publica de fora e
entra no `cargo test` como guardiao da fronteira. transformar dois demos em
exemplos apagou dois lockfiles, dois manifestos e duas entradas de membro, e nao
custou quase nenhuma linha de logica. o acoplamento que sumiu estava todo no
plumbing, e era invisivel ate o momento de cortar.

## o que isso me ensinou

eu demorei pra entender que a crate nao e uma pasta com nome bonito. a pasta voce
reorganiza num sabado e nada muda no comportamento do build. a crate muda o que o
compilador permite. quando a `rope` nao consegue chamar a GPU, nao e porque eu
prometi nao chamar, e porque o tipo nao esta no escopo. quando a engine nao pode
fechar um ciclo com a `rope`, nao e porque eu tomei cuidado, e porque o cargo
para. a fronteira que se sustenta sozinha vale dez fronteiras que dependem de eu
lembrar.

e a raiz virtual e a peca que deixa tudo isso simetrico e barato. uma versao num
lugar, um conjunto de lints num lugar, um perfil de build num lugar, e cada sala
da oficina com a propria porta. a engine virou crate como as outras justamente pra
caber nessa simetria, e o predio ficou mais facil de raciocinar por causa disso,
nao mais dificil.

se eu fosse deixar uma frase pra aurora ler disso aqui um dia: organize por pasta
quando quiser arrumar, mas quebre em crate quando quiser que a arrumacao deixe de
ser opcional. o `7864` de linhas apagadas num commit que so moveu dois demos e a
prova mais concreta que eu tenho de que a fronteira certa, no lugar certo, paga o
proprio custo no dia em que voce a desenha.

## rastros

manifesto e raiz virtual
- `Cargo.toml:1-17` (workspace virtual: `[workspace]`, `resolver = "2"`, lista de
  members, e a ausencia de `[package]` na raiz)
- `Cargo.toml:21-26` (`[workspace.package]`: version 0.1.0, edition 2024,
  rust-version 1.85, authors, repository)
- `Cargo.toml:30-35` (`[workspace.lints]`: `unsafe_op_in_unsafe_fn = "warn"`,
  `uninlined_format_args = "warn"`)
- `Cargo.toml:39-48` (`[workspace.dependencies]` internas por path: engine, git,
  rope e as outras libs)
- `Cargo.toml:50-73` (versoes externas unicas: wgpu 28, winit 0.30, cosmic-text
  0.18, taffy 0.9, ropey 1.6, gix 0.84)
- `Cargo.toml:106-114` (`[profile.release]` lto thin + codegen-units 1;
  `[profile.dev.package."*"]` opt-level 2)

crate engine (fronteira de compilacao)
- `crates/engine/Cargo.toml:4-12` (name "engine", publish = false, description
  "plev: a gpu-first compositing engine ...")
- `crates/engine/Cargo.toml:14-15` (`[lib] crate-type = ["cdylib","staticlib","rlib"]`)
- `crates/engine/Cargo.toml:20-31` (features: default = ["accessibility"],
  web-entry, android-entry, hot-reload)
- `crates/engine/Cargo.toml:52` (`rope.workspace = true`: engine depende de rope)
- `crates/engine/Cargo.toml:59-63` (dev-dependencies: narrate, criterion, lot,
  monster)
- `crates/engine/Cargo.toml:97-99` (`[[bench]] name = "scene_build"`,
  `harness = false`)
- `crates/engine/src/lib.rs:52-55` (`pub use macros::component;`, `pub use wgpu;`,
  `pub use winit;`, reexports do balcao)
- `crates/engine/src/lib.rs:123-145` (`wasm_main` atras de `feature = "web-entry"`,
  motivo: um `#[wasm_bindgen(start)]` por modulo)
- `crates/engine/src/lib.rs:98-121` (`android_main` atras de
  `feature = "android-entry"`, motivo: um `android_main` por cdylib)

crate rope (fronteira de api, pura)
- `crates/rope/src/lib.rs:5-8` ("This crate has no UI or GPU dependencies ... pure
  data manipulation, fully testable headless")
- `crates/rope/src/lib.rs:16-21` (balcao publico: `pub use Document, History,
  Selection, Transaction` etc.)
- `crates/rope/Cargo.toml:9-15` (deps de runtime so ropey + unicode-segmentation;
  dev-deps proptest + criterion)
- `crates/rope/Cargo.toml:20-22` (`[[bench]] name = "edit"`, `harness = false`)
- `crates/rope/benches/edit.rs:5-18` (`use rope::{Document, Transaction}`, consumo
  externo da api; `rope_build_5k_lines`, `rope_insert_delete_roundtrip`)

crate git (fronteira de api, sem UI)
- `crates/git/src/lib.rs:1-22` (duas camadas GitRepo/GitClient, "No UI
  dependencies ... testable against real temporary repositories"; balcao:
  `pub use GitClient, GitRepo, Branch, Commit, Hunk, FileStatus` etc.)
- `crates/git/Cargo.toml:9-18` (dep de runtime so `gix`; dev-dep so `tempfile`)

exemplos (demo consome a api de fora)
- `crates/engine/examples/snake/main.rs:1-21` (`use engine::winit::event_loop::EventLoop`,
  exemplo trata a engine como dependencia externa)
- `crates/engine/examples/counter/main.rs:14-24` (`use engine::component::Component`,
  `engine::compositor::Compositor`, `engine::gpu::GpuContext`, etc.)

commit (demos viram exemplos, o numero do acoplamento removido)
- `e6f7091` "org: demos scene-3d e snake-game viram exemplos": 23 files changed,
  1 insertion(+), 7864 deletions(-); deletou `crates/scene-3d/Cargo.toml`,
  `crates/snake-game/Cargo.toml`, `crates/scene-3d/Cargo.lock` (3904 linhas),
  `crates/snake-game/Cargo.lock` (3900 linhas), tirou os 2 membros do
  `Cargo.toml` da raiz, e moveu o `src/` de cada demo pra `examples/` (na raiz,
  no estado daquele commit). ancestral de HEAD, presente em `main` e em
  `refactor/workspace-restructure` (`git show --stat e6f7091`,
  `git merge-base --is-ancestor`)

adr (decisao e cicatriz)
- `kdb/adr/workspace-engine-at-root-libs-in-crates-demos-in-examples.md:20-30`
  (tres tiers: engine, libs/apps em crates, demos em examples; "cargo discovers
  them with no Cargo.toml; they are not products")
- `.../workspace-...md:32-40` (higiene: `[workspace.package]`,
  `[workspace.dependencies]` como fonte unica, `[workspace.lints]`, profile,
  `publish = false`, shaders movidos)
- `.../workspace-...md:44-51` (consequencia: a passada de rename tocou o
  `lifecycle` do counter; "caught only by `cargo test` because `cargo build
  --workspace` does not compile examples")
- `.../workspace-...md:53-59` (avoid: nao por demo em crates/, nao fixar versao
  numa crate quando ela e do workspace, nao confiar em `cargo build --workspace`
  pra provar exemplo)
- `contrato do projeto (AGENTS .contracts/.agents/AGENTS.md, secao running)`:
  `cargo run -p engine --example <name>`, qualquer
  `crates/engine/examples/<name>/main.rs`

benchmark
- `kdb/adr/benchmark-results.md` (v0.2, m4, 2026-03-11): traz scene construction,
  dirty tracking, tessellation, signals, text hashing; nao traz numero da `rope`,
  por isso este capitulo nao cita tempo de execucao da `rope`

nao confirmado
- o adr `workspace-engine-at-root-libs-in-crates-demos-in-examples.md:25-26`
  descreve a engine como crate raiz `plev` com `[package]` na raiz e `src` na
  raiz. o tree atual (branch `refactor/workspace-restructure`) tem raiz virtual
  sem `[package]` (`Cargo.toml:1-17`), engine em `crates/engine` com
  `name = "engine"` (`crates/engine/Cargo.toml:5`) e exemplos em
  `crates/engine/examples/`. o adr nao foi reescrito pra esse passo; trato o
  `Cargo.toml` no disco como fonte de verdade.
- o comentario `Run: cargo run --example snake` em
  `crates/engine/examples/snake/main.rs:3` (e o equivalente no counter) reflete a
  forma antiga, de quando a engine era raiz. a forma atual e
  `cargo run -p engine --example snake`. defasagem de comentario, nao de
  comportamento.
- `benchmark-results.md` nao publica numero pro bench `rope/edit`, entao nenhum
  tempo de execucao da `rope` foi citado.
