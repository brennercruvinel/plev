---
title: erro como valor, Result e o caminho da limpeza de lint
parte: 1
status: rascunho
rastros:
  - crates/engine/src/error.rs
  - crates/engine/src/window/lifecycle.rs
  - kdb/adr/clippy-zero-warnings.md
  - kdb/adr/index.md
  - Cargo.toml
---

# erro como valor, Result e o caminho da limpeza de lint

toda funcao que faz algo de verdade pode falhar. abrir um arquivo, criar uma
janela, pedir um adaptador de GPU pro sistema operacional. a pergunta nao e se
vai falhar, e o que voce faz com a falha quando ela chega. tem duas escolas
sobre isso e elas brigam ha decadas.

a primeira diz: quando algo der errado, jogue uma excecao, deixe ela subir pela
pilha, e que alguem la em cima resolva. o erro vira um evento, uma coisa que
acontece com o programa por fora do fluxo normal. a segunda diz: o erro nao e um
evento, e um valor. a funcao que pode falhar devolve um valor que ou e o
resultado que voce queria, ou e a explicacao do que deu errado. nada some por
baixo do pano. o erro fica na sua mao, do mesmo jeito que o sucesso ficaria.

rust escolheu a segunda escola e levou ela a serio. nao tem `try/catch` na
linguagem. o que tem e `Result<T, E>`, um enum com duas variantes: `Ok(T)`
carrega o sucesso, `Err(E)` carrega o erro. quem chama a funcao recebe o enum e
e obrigado pelo compilador a decidir o que fazer com cada caso. esse capitulo
comeca nessa escolha, mostra o tipo de erro real que a engine carrega, e desce
ate um lugar que parece nao ter nada a ver: a faxina de 107 warnings do clippy.
porque no fundo e a mesma ideia em duas escalas. a de dar um tipo pro que antes
era vago.

## o erro como um valor que voce segura

deixa eu ser concreto antes de filosofar. quando eu abro um arquivo em python, o
codigo feliz e uma linha. se o arquivo nao existe, uma excecao sobe, e se eu nao
tiver um `try` em volta, o programa morre com um traceback. o caminho do erro e
invisivel no codigo ate o momento em que ele explode. eu posso passar meses sem
nunca escrever o tratamento, e o programa "funciona", ate o dia que o arquivo
some.

em rust eu nao tenho esse luxo, e isso e de proposito. a funcao que abre o
arquivo devolve um `Result`. eu nao consigo usar o conteudo do arquivo sem antes
abrir a caixa e olhar o que tem dentro. ou eu trato o `Err`, ou eu propago ele
pra cima de forma explicita, ou eu mando o programa entrar em panico de forma
explicita. o que eu nao consigo e fingir que a falha nao existe. o compilador
nao deixa. o erro deixou de ser um evento que acontece comigo e virou um dado que
eu seguro na mao.

isso muda o jeito de pensar. quando o erro e valor, voce desenha o tipo dele. e
um tipo seu, com as variantes que fazem sentido pro seu dominio, com a mensagem
que voce escreveu, com a cadeia de causa que voce escolheu preservar. o erro
deixa de ser uma string solta ou uma excecao generica e passa a ser parte do
contrato da sua API. e ai que a engine entra.

## o codigo real, PlevError

a plev tem um unico tipo de erro pra coisa toda. ele vive em
`crates/engine/src/error.rs`, um arquivo de 59 linhas, e e a primeira coisa
declarada na lib, antes de rendering, antes de texto, antes de qualquer
subsistema. olha o enum:

```rust
use std::fmt;

/// Unified error type for the plev engine.
#[derive(Debug)]
pub enum PlevError {
    /// Window creation or platform error.
    Window(winit::error::OsError),

    /// GPU initialization or resource error.
    Gpu(String),

    /// WASM-specific: web-sys API unavailable or returned an unexpected value.
    #[cfg(target_arch = "wasm32")]
    Wasm(&'static str),

    /// File watcher error (hot-reload only).
    #[cfg(feature = "hot-reload")]
    Watcher(notify::Error),
}
```

quatro variantes, e cada uma carrega uma coisa diferente, o que ja conta uma
historia. `Window` embrulha um `winit::error::OsError`, o erro que o sistema
operacional devolve quando a criacao de janela falha. ele nao vira string, fica
o tipo original do winit dentro. `Gpu` carrega um `String`, porque erro de GPU
no wgpu vem de muitos lugares e nem sempre tem um tipo unico bom de embrulhar,
entao aqui a engine aceita a mensagem ja formatada. `Wasm` carrega um
`&'static str`, uma string literal estatica, porque os erros de browser que a
engine produz sao poucos e fixos, escritos a mao no codigo, nao precisam de
alocacao. `Watcher` embrulha o erro do `notify`, o crate de file watching usado
no hot reload.

repara nos dois atributos de compilacao condicional. `Wasm` so existe quando o
alvo e `wasm32`. `Watcher` so existe quando a feature `hot-reload` esta ligada.
isso quer dizer que o proprio formato do tipo de erro muda conforme a plataforma
e as features. um build de desktop sem hot reload nem tem a variante `Watcher`
compilada. o erro nao e um saco generico de tudo que poderia dar errado em
qualquer lugar, e exatamente o conjunto de falhas possiveis pra aquele build
especifico. essa e a primeira decisao de tipo que vale marcar: a forma do erro
acompanha a forma do programa.

## escolher o verbo, "is" e "has" em vez de excecao

um tipo de erro nao serve pra nada se voce nao puder imprimir ele, encadear a
causa e converter pra ele de forma barata. as tres coisas estao no mesmo
arquivo, escritas a mao. primeiro o `Display`:

```rust
impl fmt::Display for PlevError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Window(e) => write!(f, "window: {e}"),
            Self::Gpu(msg) => write!(f, "gpu: {msg}"),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(msg) => write!(f, "wasm: {msg}"),
            #[cfg(feature = "hot-reload")]
            Self::Watcher(e) => write!(f, "file watcher: {e}"),
        }
    }
}
```

cada variante imprime com um prefixo curto que diz de onde veio. um detalhe que
parece bobo e nao e: o `{e}` e o `{msg}` dentro do `write!` sao argumentos de
formato inline, a variavel direto dentro das chaves, nao `write!(f, "window:
{}", e)`. isso nao foi gosto. o workspace liga o lint `uninlined_format_args =
"warn"` no `Cargo.toml`, e como a porta de qualidade roda clippy com `-D
warnings`, a versao com o argumento posicional nem compilaria limpa. o estilo do
`Display` aqui e uma consequencia direta da politica de lint. eu volto nesse fio
mais pra frente, porque ele e a costura entre as duas metades desse capitulo.

depois o `Error`, com `source`:

```rust
impl std::error::Error for PlevError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Window(e) => Some(e),
            #[cfg(feature = "hot-reload")]
            Self::Watcher(e) => Some(e),
            _ => None,
        }
    }
}
```

isso e a cadeia de causa. quando a falha veio de outro erro com tipo proprio (o
`OsError` do winit, o `Error` do notify), o `source` devolve ele, e quem estiver
imprimindo a cadeia consegue subir ate a origem. `Window` e `Watcher` tem uma
fonte de verdade, entao devolvem `Some`. `Gpu` carrega so uma string e `Wasm`
carrega um literal, nenhum dos dois tem um erro pai com tipo, entao caem no `_ =>
None`. o erro nao mente sobre ter uma causa que ele nao tem.

por fim as duas conversoes:

```rust
impl From<winit::error::OsError> for PlevError {
    fn from(e: winit::error::OsError) -> Self {
        Self::Window(e)
    }
}

#[cfg(feature = "hot-reload")]
impl From<notify::Error> for PlevError {
    fn from(e: notify::Error) -> Self {
        Self::Watcher(e)
    }
}
```

esses dois `From` sao o que faz o operador `?` funcionar sem cerimonia. quando
uma funcao devolve `PlevResult` e voce escreve `algo_que_da_OsError()?`, o `?`
olha se existe um `From<OsError> for PlevError`, acha esse aqui, e converte
sozinho antes de retornar o `Err`. sem o `From`, voce teria que escrever
`.map_err(PlevError::Window)?` na mao em todo ponto de propagacao. o `From`
transforma a conversao de erro em algo que some no `?`. e a ergonomia inteira do
erro como valor depende disso: propagar tem que ser mais facil que engolir.

no fim do arquivo, o alias:

```rust
/// Convenience alias for `Result<T, PlevError>`.
pub type PlevResult<T> = Result<T, PlevError>;
```

`PlevResult<T>` e so um apelido pra `Result<T, PlevError>`. economiza repetir o
tipo de erro em toda assinatura e da um nome que a leitura reconhece. quando voce
ve `-> PlevResult<()>` numa funcao da engine, voce ja sabe: isso pode falhar, e a
falha vem na lingua do `PlevError`.

### uma digressao sobre thiserror

vale uma parada aqui, porque tem uma escolha interessante escondida. o
`Cargo.toml` do workspace declara `thiserror = "2"` nas dependencias
compartilhadas. o thiserror e o crate que a comunidade rust usa pra gerar tudo
isso, o `Display`, o `Error`, os `From`, a partir de uns atributos de derive.
voce escreveria `#[derive(Error)]` e `#[error("window: {0}")]` e o macro
cuspiria as impls. seria menos linha.

so que o `crates/engine/Cargo.toml` nao lista thiserror nas dependencias do
crate. a engine escreve as impls a mao. o thiserror esta disponivel no
workspace, mas o tipo de erro central nao usa ele. eu nao tenho no repo a justi
ficativa escrita dessa escolha (marco como nao confirmado o porque exato), entao
nao vou inventar a motivacao. o que eu posso dizer pelo codigo e o efeito: as
impls a mao deixam a compilacao condicional explicita no proprio `match`, com os
`#[cfg(...)]` dentro de cada arm, e nao escondem nada atras de um macro. pra um
tipo de 59 linhas, com quatro variantes e dois `cfg`, o custo de escrever na mao
e baixo e o que voce le e exatamente o que roda. menos magica, menos uma camada
de macro pra entender quando algo der errado no proprio tratamento de erro.

## o ? na pratica, Option virando Result

a teoria fica abstrata sem um ponto de uso. o mais limpo na engine esta em
`crates/engine/src/window/lifecycle.rs`, na funcao `setup_wasm_canvas`, que so
existe no build de browser. a assinatura ja anuncia a intencao:

```rust
#[cfg(target_arch = "wasm32")]
pub fn setup_wasm_canvas(window: &winit::window::Window) -> crate::error::PlevResult<()> {
    use crate::error::PlevError;
    use winit::platform::web::WindowExtWebSys;

    let canvas = window
        .canvas()
        .ok_or(PlevError::Wasm("canvas not available on window"))?;

    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or(PlevError::Wasm("no document available"))?;

    let body = doc
        .body()
        .ok_or(PlevError::Wasm("no <body> element in document"))?;

    body.append_child(&canvas)
        .map_err(|_| PlevError::Wasm("failed to append canvas to <body>"))?;
```

essa funcao e uma aula pequena de como `Option` e `Result` conversam. quase tudo
que vem do mundo do browser via `web-sys` devolve `Option`, porque uma API do
DOM pode simplesmente nao estar la. `window.canvas()` devolve
`Option<HtmlCanvasElement>`, o canvas pode nao existir. `web_sys::window()`
devolve `Option`, pode nao ter objeto window. e assim por diante.

o problema e que `Option` e `Result` sao tipos diferentes. o `Option` sabe que
algo esta ausente, mas nao sabe por que. o `?` numa funcao que devolve `Result`
nao engole um `Option` direto, porque um `None` nao carrega erro nenhum pra
propagar. e ai que entra o `.ok_or(...)`. ele pega o `Option` e o transforma em
`Result`: `Some(x)` vira `Ok(x)`, e `None` vira `Err(PlevError::Wasm("..."))`,
com a mensagem especifica daquele ponto. so depois disso o `?` pode agir. cada
`ok_or` ali e o lugar onde uma ausencia anonima ganha um nome de erro concreto.

a ultima linha usa `.map_err(...)` em vez de `.ok_or(...)` por um motivo simples
de tipo: `append_child` ja devolve um `Result`, nao um `Option`. ele falha com
um erro do tipo do web-sys (um `JsValue`), que aqui nao interessa preservar.
entao `.map_err(|_| PlevError::Wasm("failed to append canvas to <body>"))`
descarta o erro original e poe a mensagem da engine no lugar. a funcao inteira
nao tem um `if err`, nao tem um `match`. ela tem quatro pontos de falha, cada um
com uma frase que diz o que faltou, e o `?` costura tudo. no fim, se nada falhou,
ela devolve `Ok(())`. erro como valor, propagado por valor, sem um unico ramo de
controle de fluxo escrito a mao.

## a tensao honesta, quando o ? nao cabe

eu seria desonesto se parasse aqui e deixasse parecer que a engine inteira
respira `Result` por todo lado. ela nao. no mesmo arquivo, poucas dezenas de
linhas acima do `setup_wasm_canvas`, a criacao de janela faz isso:

```rust
let window = Arc::new(
    event_loop
        .create_window(attrs)
        .expect("plev: failed to create window"),
);
```

`create_window` devolve um `Result<Window, OsError>`. e a engine tem um
`From<OsError> for PlevError`, exatamente o que precisaria pra escrever
`create_window(attrs)?` e propagar. mas aqui o codigo usa `.expect(...)`, que
entra em panico se a janela nao nascer. por que o tipo de erro foi desenhado pra
suportar `?` num ponto que usa `expect`?

a resposta esta na assinatura de onde esse codigo mora. isso roda dentro do
callback `resumed` do `ApplicationHandler` do winit, um metodo que o framework
chama e cuja assinatura nao devolve `Result`. eu nao posso usar `?` ali, porque
nao tem pra onde o erro subir, o winit nao espera um `Result` de volta. e se a
janela nao pode ser criada nesse momento do ciclo de vida, nao tem recuperacao
sensata, o app nao existe sem janela. entao a escolha e panico com uma mensagem
clara, e nao um erro propagado pra lugar nenhum.

essa e a parte que os tutoriais de "erro como valor" costumam pular. o ideal e
lindo no papel: toda falha vira valor, tudo propaga, nada entra em panico. a
realidade e que voce esbarra em fronteiras onde a assinatura nao e sua, callbacks
de plataforma, traits de framework, pontos de entrada que o sistema operacional
chama. nesses pontos o `Result` nao tem pra onde ir, e a decisao certa as vezes e
o panico honesto com mensagem, nao a contorcao pra fingir que aquilo era
recuperavel. o `From<OsError>` continua valendo a pena: ele esta la pros pontos
de propagacao que existem hoje e pros que vao existir quando esse codigo sair de
dentro do callback. o tipo de erro foi desenhado pra ergonomia, mesmo onde o
ponto de uso atual nao colhe ela.

## o mesmo instinto, uma escala acima, o lint

agora a virada do capitulo. tudo que eu descrevi ate aqui e a mesma decisao,
repetida: quando algo e vago (uma ausencia, uma falha, uma string solta), dar um
tipo pra ele. `PlevError` da tipo pra falha. `ok_or` da tipo pra ausencia. e
quando voce treina esse instinto no nivel do erro, ele aparece de novo num lugar
que parece pura cosmetica: a limpeza de lint.

a plev passou por uma faxina registrada no adr `kdb/adr/clippy-zero-warnings.md`.
o projeto tinha acumulado 107 warnings do clippy rodando com `-D warnings`,
distribuidos em 12 categorias. a porta de qualidade do repo exige clippy limpo
(`cargo clippy --workspace --all-targets -- -D warnings` e um dos quatro
comandos que todo commit tem que passar), entao 107 warnings era 107 razoes pro
build vermelho. o commit de origem registrado no adr e `2d95911`.

a leitura preguicosa dessa historia e "limparam uns warnings, mudaram umas
linhas pra deixar o linter quieto". e parte foi isso mesmo, vou ser honesto sobre
a parte cosmetica. mas uma fatia boa do trabalho nao foi maquiagem, foi mudanca
de tipo guiada pelo lint. o clippy apontou lugares onde o tipo estava errado, e o
conserto certo nao era silenciar, era criar um tipo. e essa a tese desse
capitulo, e ela so e verdade se eu separar as duas fatias com numero.

## as duas faixas dos 107

das 12 categorias, duas concentram a maior parte e sao de fato cosmetica de
legibilidade. float excessive precision teve 36 ocorrencias, a maior delas:
literais `f32` escritos com mais digitos do que um float de 32 bits consegue
representar, uns 7 significativos. ficavam concentrados nas matrizes de espaco de
cor OKLCH e na constante kappa das curvas de bezier. o conserto foi truncar pro
que cabe e usar separador `_` pra legibilidade, tipo `0.412_221_5`. isso nao muda
tipo nenhum, e higiene de numero.

collapsible if teve 23 ocorrencias, a segunda maior: `if` aninhado que dava pra
fundir com `&&`. o conserto usou `&&` e let-chains, que a edition 2024 suporta
(`if let ... && condition`). de novo, mais legibilidade que arquitetura, ainda
que o let-chain seja um recurso de linguagem que so existe porque o workspace
roda na edition 2024. entre essas duas categorias ja sao 59 dos 107 warnings, e
eu chamaria a maior parte disso de cosmetica honesta. um exemplo generico de
collapsible if, so pra fixar o padrao, fora do repo:

```rust
// antes (clippy reclama de collapsible_if)
if a {
    if b {
        faz();
    }
}

// depois
if a && b {
    faz();
}
```

ate aqui o cetico tem razao. a virada esta nas categorias do meio.

## quando o lint te obriga a nomear um tipo

too many arguments teve 16 ocorrencias: funcoes com mais de 7 parametros. o
clippy bate nessas porque uma funcao com 9 ou 10 argumentos posicionais e um
campo minado, e facil trocar a ordem de dois `f32` e nao perceber. o adr conta
que o conserto usou duas estrategias. pra funcoes chamadas com frequencia, a
correcao foi agrupar os argumentos em structs nomeados: `ResolveResources`,
`CardColors`, `CardLayout`. pra umas poucas funcoes de showcase que recebem cores
individuais do tema, onde agrupar nao melhorava a leitura, a escolha foi um
`#[allow(clippy::too_many_arguments)]` por item.

para nessa primeira estrategia um segundo. o clippy nao pediu "crie uma struct".
ele pediu "essa funcao tem argumentos demais". o conserto certo foi olhar pros
argumentos e perceber que varios deles andavam juntos, eram na verdade um
conceito so que nunca tinha ganhado nome. `CardColors` nao e um truque pra
satisfazer o linter, e o reconhecimento de que aquele punhado de cores e uma
coisa: a paleta de um card. o lint foi o sintoma, o tipo faltante era a doenca. e
exatamente o mesmo movimento do `PlevError`: tinha uma informacao solta (varios
argumentos, varias falhas possiveis) e a resposta foi embrulhar num tipo com
nome.

complex types teve 7 ocorrencias e o adr diz que foram resolvidas junto com as de
too many arguments. eram tipos longos demais aparecendo cru nas assinaturas, o
exemplo do adr e `Option<(&wgpu::Buffer, &wgpu::Buffer, u32)>`. um tipo desses
numa assinatura e a mesma coisa que um erro como string: tecnicamente carrega a
informacao, mas nao tem nome, ninguem sabe o que aquela tupla de dois buffers e
um `u32` significa sem ir caçar. a saida foi a mesma, struct nomeado ou reducao
de visibilidade dos metodos afetados. de novo o lint apontando um lugar onde
faltava tipo.

default implementations foi a terceira faixa que mexeu em tipo: 14 structs que
tinham um `new()` sem argumentos mas nao implementavam `Default`. o clippy
reclama disso porque, no ecossistema rust, um tipo construivel sem argumento
deveria oferecer `Default`, e codigo generico que pede `T: Default` espera isso.
o conserto, nas palavras do adr:

```rust
impl Default for X {
    fn default() -> Self {
        Self::new()
    }
}
```

isso e decisao de tipo pura. implementar `Default` muda o que aquele tipo
promete, faz ele caber em todo lugar que pede a trait, abre a porta pra `..
Default::default()` e pra coleções que constroem o valor padrao sozinhas. pra
`App` e `AccessibilityState` a impl foi gated com `#[cfg(...)]` pra manter a
compilacao condicional, o mesmo cuidado que o `PlevError` tem com as variantes
`Wasm` e `Watcher`. e uma das 11 ocorrencias da categoria "outras" foi o oposto
exato, um `impl Default` escrito a mao que o clippy mostrou ser derivavel, trocado
por `#[derive(Default)]`. nos dois sentidos, o lint estava falando sobre o
contrato de tipo, nao sobre estetica.

se eu somar as faixas que mexeram em tipo, too many arguments (16), complex types
(7) e default implementations (14), dao 37 dos 107. mais de um terço da faxina
nao foi maquiagem, foi o linter apontando tipos que faltavam ser nomeados ou
traits que faltavam ser implementadas. e a frase do escopo desse capitulo, que o
lint guiou decisao de tipo e nao so cosmetica, para de pe sobre esses 37, nao
sobre os 107. a honestidade do numero e que sustenta a tese.

## a armadilha que prova que visibilidade e decisao de tipo

tem um detalhe na secao de armadilhas do adr que e quase uma parabola. um dos
warnings sugeria deixar `Compositor::resolve` mais privado, marcar `pub(crate)`,
porque o tipo do retorno era mais privado que o metodo publico (o classico "type
more private than item"). parece uma sugestao inofensiva de visibilidade. a
correcao compilou limpa na lib. e quebrou 14 exemplos, porque os exemplos em
`crates/engine/examples/` chamavam `resolve` de fora do crate.

a regra que nasceu disso esta escrita no adr: rodar `cargo check --examples`
depois de qualquer mudanca de visibilidade em API publica. mas o que me
interessa aqui e o que o episodio revela. visibilidade nao e cosmetica, e
fronteira de tipo. quando o clippy sugeriu apertar a visibilidade de `resolve`,
ele estava propondo encolher a superficie publica do `Compositor`, e essa
superficie e um contrato com 14 consumidores que o linter nao via. o lint
acertou o sintoma local (o tipo do retorno era mais privado que o metodo) e
errou a consequencia global, porque o lint nao enxerga o crate de exemplos. a
licao nao e ignorar o clippy, e que decisao de tipo, ate uma tao pequena quanto
um `pub(crate)`, tem alcance que o linter sozinho nao mede. quem decide o tipo e
voce, com o repo inteiro na cabeca. o clippy aponta, nao decide.

## a costura, por que o Display usa {e}

agora eu fecho o fio que deixei solto la atras. o `Display` do `PlevError` usa
`{e}` e `{msg}` inline, e eu disse que isso era consequencia do lint
`uninlined_format_args` que o workspace liga no `Cargo.toml`. aqui esta a costura
inteira do capitulo num detalhe de cinco caracteres.

o `error.rs` foi escrito (ou reescrito) ja sob a regra. a politica de lint do
projeto nao e um passo de limpeza que acontece depois e some, ela e uma pressao
constante que molda como cada linha nova nasce. o tipo de erro central da engine
imprime do jeito que imprime porque a porta de qualidade exige. nao foi alguem
escolhendo `{e}` por gosto e o clippy concordando depois, foi o clippy definindo
o gosto possivel. as duas metades desse capitulo, o erro como valor e a faxina de
lint, nao sao dois assuntos que eu colei. sao a mesma forca aparecendo no tipo
(`PlevError` da nome a falha) e na linha (`{e}` em vez de `{}`, `Default` em vez
de `new` solto, struct em vez de 9 argumentos). o lint nao e o juiz que vem
depois da escrita. ele e parte da gramatica em que a escrita acontece.

vale registrar tambem por que o numero e 107 e nao 7, e por que ir a zero. o
`-D warnings` transforma todo warning em erro de build. nao tem warning "que da
pra conviver". ou e zero, ou o build e vermelho. isso e duro de proposito: um
projeto que tolera 3 warnings tolera 30, porque o olho para de ver o ruido. zerar
e o que mantem o sinal alto, todo warning novo aparece sozinho num fundo limpo, e
voce sabe na hora que foi a sua mudanca que o trouxe. a outra armadilha do adr
reforça isso de um angulo de processo: quatro agentes rodando correcoes de clippy
no mesmo working tree ao mesmo tempo se sobrescreveram, correcao de um apagada
por outro, e a regra que ficou foi um unico agente pra correcoes que tocam
arquivos sobrepostos. zerar lint e um estado, e estado compartilhado nao
sobrevive a quatro mãos escrevendo em cima.

## por que assim, e nao de outro jeito

junta as duas pontas e o desenho fica claro. rust escolheu erro como valor, e a
plev levou isso pro tipo central `PlevError`, com variantes que acompanham
plataforma e feature, com `From` pra propagacao barata via `?`, com `source` pra
cadeia de causa, escrito a mao em 59 linhas porque pra esse tamanho a clareza
ganha do macro. esse e o primeiro nivel: a falha tem tipo.

o segundo nivel e a politica de lint, e ela e a mesma ideia operando sobre o
codigo como um todo. o clippy roda com `-D warnings` e foi de 107 a zero. a maior
parte (os 59 de float precision e collapsible if) foi higiene de legibilidade,
sem vergonha nenhuma de ser so isso. mas 37 deles, too many arguments, complex
types e default implementations, foram o linter apontando tipos que faltavam: um
punhado de argumentos que era uma struct sem nome, uma tupla longa que era um
conceito sem nome, um `new()` que pedia a trait `Default`. nesses, o conserto foi
criar tipo, nao silenciar warning. e a armadilha do `pub(crate)` que quebrou 14
exemplos prova o limite: o lint aponta o sintoma local com precisao e nao ve a
consequencia global, entao a decisao de tipo continua sendo humana, com o repo
inteiro em vista.

a engine podia ter ido pelo caminho da string. erro como `String`, argumentos
soltos, `#[allow]` espalhado pra calar o clippy, warnings tolerados. funcionaria,
no sentido de compilar e rodar. o custo apareceria depois, quando alguem
precisasse saber por que a janela nao abriu, ou o que aquela tupla de dois
buffers significava, ou qual das 9 cores de um card estava trocada. o caminho que
o repo seguiu paga esse custo na frente, no momento da escrita, dando nome cedo.
`less, but better` nao e so sobre cortar. e sobre o que sobra ter nome.

## o que isso me ensinou

eu cheguei nesse trabalho achando que lint era chato. ruido do compilador, regra
de gente metida, coisa pra silenciar com um `allow` e seguir a vida. o que mudou
minha cabeca foi separar os 107 em faixas e ver que um terço deles nao era opiniao
de estilo, era o clippy enxergando um tipo que eu nao tinha desenhado direito.
quando o linter diz "argumentos demais", ele as vezes esta dizendo "voce tem um
conceito sem nome aqui". quando diz "tipo complexo", esta dizendo a mesma coisa
de outro jeito. ele nao sabe qual e o nome certo, isso e meu, mas ele aponta o
lugar onde falta.

e a ponte com o erro como valor foi a parte que eu nao esperava. sao o mesmo
reflexo. o `?` que propaga `PlevError` e o struct `CardColors` que nasceu de um
warning de argumentos demais vem do mesmo lugar: a recusa de deixar informacao
solta sem tipo. um trata falha, o outro trata um agrupamento de cores, mas os
dois respondem a mesma pergunta, qual e o tipo que carrega isso direito.

se eu fosse deixar uma frase pra aurora ler disso um dia: o compilador e o linter
nao sao a policia do seu codigo, sao o primeiro leitor honesto dele. quando eles
reclamam, na maioria das vezes nao e que o codigo esta feio, e que tem um tipo
pedindo pra nascer e voce ainda nao deu nome. o `PlevError` e um tipo que nasceu
porque falha precisa de nome. uns 37 daqueles warnings foram outros tipos
pedindo a mesma coisa. o resto, os 59, foi so manter a casa limpa pra dar pra
ouvir os 37.

## rastros

tipo de erro (crate engine)
- `crates/engine/src/error.rs:4-19` (`enum PlevError`, variantes `Window`,
  `Gpu(String)`, `Wasm(&'static str)` sob `#[cfg(target_arch = "wasm32")]`,
  `Watcher(notify::Error)` sob `#[cfg(feature = "hot-reload")]`)
- `crates/engine/src/error.rs:21-32` (`impl Display`, prefixo por variante,
  `{e}`/`{msg}` inline)
- `crates/engine/src/error.rs:34-43` (`impl Error` com `source`, `Window` e
  `Watcher` devolvem `Some`, resto `None`)
- `crates/engine/src/error.rs:45-49` (`From<winit::error::OsError>`)
- `crates/engine/src/error.rs:51-56` (`From<notify::Error>` sob feature
  `hot-reload`)
- `crates/engine/src/error.rs:58-59` (`pub type PlevResult<T> = Result<T,
  PlevError>`)
- `crates/engine/src/lib.rs:1-2` (`pub mod error`, primeiro modulo da lib)

uso real de Result/Option/?
- `crates/engine/src/window/lifecycle.rs:207-245` (`setup_wasm_canvas` devolve
  `PlevResult<()>`, usa `.ok_or(PlevError::Wasm(...))?` pra converter `Option`
  em `Result` e `.map_err(...)?` no `append_child`)
- `crates/engine/src/window/lifecycle.rs:39` e `:86`
  (`create_window(attrs).expect("plev: failed to create window")`, ponto onde
  `?` nao cabe porque roda dentro do callback `resumed` do `ApplicationHandler`)

faxina de lint (adr)
- `kdb/adr/clippy-zero-warnings.md:10-17` (107 warnings em 12 categorias sob `-D
  warnings`)
- `kdb/adr/clippy-zero-warnings.md:7` (commit de origem `2d95911`)
- `kdb/adr/clippy-zero-warnings.md:20-28` (float excessive precision, 36, OKLCH
  e kappa de bezier, separador `_`, `0.412_221_5`)
- `kdb/adr/clippy-zero-warnings.md:29-35` (collapsible if, 23, `&&` e let-chains
  da edition 2024)
- `kdb/adr/clippy-zero-warnings.md:37-42` (default implementations, 14 structs,
  `impl Default { fn default() -> Self { Self::new() } }`, `App` e
  `AccessibilityState` gated)
- `kdb/adr/clippy-zero-warnings.md:44-50` (too many arguments, 16, structs
  `ResolveResources`/`CardColors`/`CardLayout` ou `#[allow]` por item)
- `kdb/adr/clippy-zero-warnings.md:52-56` (complex types, 7, ex.
  `Option<(&wgpu::Buffer, &wgpu::Buffer, u32)>`, resolvidos com too many args)
- `kdb/adr/clippy-zero-warnings.md:58-67` (outras, 11, incluindo `clamp`,
  `clone on copy`, derivable impl trocada por `#[derive(Default)]`)
- `kdb/adr/clippy-zero-warnings.md:69-77` (armadilha "type more private than
  item": `pub(crate)` em `Compositor::resolve` compilou na lib mas quebrou 14
  exemplos; regra: `cargo check --examples` apos mudanca de visibilidade)
- `kdb/adr/clippy-zero-warnings.md:78-83` (armadilha: agentes paralelos se
  sobrescrevem, regra de um unico agente por arquivos sobrepostos)
- `kdb/adr/index.md:65` (linha-resumo: 107 warnings em 12 categorias)

politica e versoes (conferidas contra o Cargo.toml)
- `Cargo.toml:33-34` (`[workspace.lints.clippy]` `uninlined_format_args =
  "warn"`, o que explica o `{e}`/`{msg}` inline no `Display`)
- `Cargo.toml:30-31` (`[workspace.lints.rust]` `unsafe_op_in_unsafe_fn = "warn"`)
- `Cargo.toml:64` (`thiserror = "2"` nas deps do workspace)
- `Cargo.toml:51` (`winit = "0.30"`, fonte do `OsError`)
- `Cargo.toml:86` (`notify = "7"`, fonte do `Watcher`, feature `hot-reload`)
- `Cargo.toml:50` (`wgpu = "28"`)
- `Cargo.toml:23` (edition 2024, requisito do let-chain do collapsible if)
- `Cargo.toml:24` (rust-version 1.85)
- `.contracts/.agents/AGENTS.md` (a porta: `cargo clippy --workspace
  --all-targets -- -D warnings` entre os quatro comandos)

nao confirmado
- a motivacao escrita pra escrever as impls de `PlevError` a mao em vez de usar
  `thiserror` (declarado em `Cargo.toml:64` mas ausente de
  `crates/engine/Cargo.toml`) nao esta documentada no repo; descrevo o efeito
  observavel, nao a intencao.
- o adr `clippy-zero-warnings.md` nao lista `error.rs` entre os arquivos
  tocados pelas 107 correcoes; a ligacao entre o `Display` e o lint
  `uninlined_format_args` e inferida do estilo do codigo somado a politica do
  `Cargo.toml`, nao de uma linha de commit que ligue os dois.
- a soma "37 warnings de tipo" (16 too many args + 7 complex types + 14 default
  impls) e contagem minha sobre os numeros do adr, nao um numero que o adr
  apresenta agrupado.
