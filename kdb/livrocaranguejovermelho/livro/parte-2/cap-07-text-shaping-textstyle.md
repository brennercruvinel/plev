---
title: "text: shaping e uma TextStyle por run"
parte: 2
status: rascunho
rastros:
  - crates/engine/src/text/backend.rs
  - crates/engine/src/text/measure.rs
  - crates/engine/src/text/system.rs
  - crates/engine/src/text/fonts.rs
  - crates/engine/src/text/cache.rs
  - crates/engine/src/compositor/scene.rs
  - crates/engine/src/text/tests_measure.rs
  - kdb/adr/one-text-style-for-measurement-and-drawing.md
---

# text: shaping e uma TextStyle por run

tem um bug de interface que e tao comum que virou quase um cheiro de codigo, e
quando voce aprende a sentir o cheiro nao consegue mais nao ver. e a etiqueta
maior que a forma. o botao que tem um rotulo certo, "Follow", e uma pilula
arredondada bonita em volta dele, so que o texto encosta na borda direita, ou
pior, vaza um pixel pra fora, ou ainda pior, a pilula ficou larga demais e
sobra um vazio esquisito de um lado so. voce olha e sabe que tem algo errado,
mas nao consegue dizer o que. parece detalhe de designer chato. nao e. e quase
sempre o mesmo defeito, com o mesmo mecanismo por baixo, e ele tem nome.

o mecanismo e este: alguem calculou o tamanho da forma usando um modelo do
texto, e desenhou o texto usando outro modelo. dois modelos do mesmo texto que
discordam. um diz que "Follow" tem 47 pixels de largura, o outro desenha 51. a
forma nasce com 47 mais o padding, o glifo ocupa 51, e a conta nao fecha por 4
pixels que ninguem pediu. o defeito nao esta no shaping nem na medida tomados
sozinhos. cada um, isolado, esta certo. o defeito esta no fato de existirem
dois.

esse capitulo e sobre uma regra que fecha essa classe inteira de bug de uma
vez, em vez de remendar pilula por pilula: um run de texto tem exatamente uma
`TextStyle`, e essa mesma `TextStyle` e a entrada da medida e a entrada do
desenho. nao duas que por acaso batem. uma. construir as duas separadas e um
defeito por definicao, mesmo nos dias em que os numeros coincidem. vou abrir no
botao em cima da mesa, descer ate o `struct` real de quatro campos mais um, mostrar
o caminho de shaping pelo cosmic-text dos dois lados da fronteira, e chegar no
ponto exato onde a divergencia nasce quando voce deixa.

## a forma que mente

antes do codigo, vale olhar de perto os tres casos que pagaram por essa regra,
porque eles nao sao hipoteticos. estao registrados no adr
`kdb/adr/one-text-style-for-measurement-and-drawing.md`, e os tres tem o mesmo
esqueleto com roupas diferentes.

o primeiro foi o ide, o cliente git do projeto. ele dimensionava cada forma com
uma heuristica por caractere, `chars * font_size * 0.58`. conta o numero de
caracteres, multiplica pela fonte, multiplica por um fator magico de 0.58 que
alguem chutou como "largura media de um glifo". e na hora de desenhar, desenhava
com shaping de verdade, na fonte Rubik no peso 600. o erro medido ia de -10% a
+21%. dez por cento pra menos quer dizer texto vazando da forma. vinte e um por
cento pra mais quer dizer pilula larga demais, com aquele vazio de um lado. o
mesmo botao, dependendo da palavra, errava pros dois lados.

repara no problema de fundo da formula `chars * 0.58`. ela trata todo glifo como
se tivesse a mesma largura. mas "Follow" tem dois `l` finos e um `o` largo e um
`w` larguissimo, e a heuristica nao sabe disso. nem sabe que o peso 600 engorda
cada traco em relacao ao 400. nem sabe que digito tem largura propria. o `0.58`
e uma media que nao bate com palavra nenhuma especifica, so com a media de um
corpus que ninguem mediu. ela esta sempre errada, a duvida e so de quanto e pra
que lado.

o segundo caso e mais sutil e por isso mais perigoso. o pipeline do builder
media o texto com `letter_spacing: 0.0` cravado no codigo, enquanto expunha um
modificador `.tracking()` no lado do desenho. ou seja: dava pra voce pedir
espacamento entre letras na hora de desenhar, e a medida ignorava esse pedido,
media sempre como se o espacamento fosse zero. o detalhe que salvou por um tempo
e que o `.tracking()` estava inerte naquele momento, nao fazia nada de fato.
entao a divergencia existia no codigo mas nao aparecia na tela. ficou latente.
uma divergencia latente e uma bomba com o timer parado: no dia que alguem ligar
o `.tracking()`, todas as formas que usam tracking passam a medir sem ele e
ficam apertadas, e o bug nasce ja velho, ja espalhado, sem ninguem ter mexido na
medida.

o terceiro caso e o mais antigo, de uma versao do engine anterior a integracao
do cosmic-text, que media `chars * 0.6` antes de existir shaping de verdade.
sintoma identico. tres implementacoes diferentes, tres epocas, um unico
mecanismo: o tamanho saiu de um modelo, o pixel saiu de outro.

o que esses tres ensinam juntos e que o defeito nao e de calibragem. nao adianta
trocar 0.58 por 0.61 e achar a constante perfeita. nao existe constante perfeita,
porque a largura de um texto nao e uma funcao do numero de caracteres. e uma
funcao da fonte, do peso, do espacamento, da forma exata de cada glifo, e da
maneira como o shaper junta tudo isso. a unica coisa que sabe a largura de
verdade e a coisa que de fato faz o shaping. entao a medida tem que ser feita
pelo mesmo shaper que desenha, com a mesma fonte, com o mesmo estilo. nao parecida.
a mesma.

## o objeto que os dois lados compartilham

a peca que carrega esse "mesmo estilo" e a `TextStyle`. ela mora em
`crates/engine/src/text/backend.rs`, e e deliberadamente pequena:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    /// Extra advance per glyph in px (CSS `letter-spacing`). The HOFF
    /// reference uses `0.025em` on its Inter body styles.
    pub letter_spacing: f32,
    pub font_family: Option<String>,
}
```

cinco campos. tamanho da fonte, altura de linha, peso, espacamento entre letras,
e a familia (com `None` querendo dizer "a familia default do engine"). nao tem
mais nada, e isso e proposital. cada um desses cinco campos e algo que muda a
largura ou a altura do texto quando voce desenha. `font_size` obvio. `font_weight`
porque o peso 700 ocupa mais que o 400 na mesma palavra. `letter_spacing` porque
ele adiciona avanco por glifo. `font_family` porque Inter e JetBrains Mono medem
diferente. e `line_height` porque ele define a altura da caixa de linha. se um
campo afeta o pixel, ele tem que estar aqui, pra entrar tanto na medida quanto
no desenho.

o construtor preenche os defaults de um jeito que vale anotar:

```rust
impl TextStyle {
    pub fn new(font_size: f32) -> Self {
        Self {
            font_size,
            line_height: font_size * DEFAULT_LINE_HEIGHT_FACTOR,
            font_weight: 400,
            letter_spacing: 0.0,
            font_family: None,
        }
    }
```

`DEFAULT_LINE_HEIGHT_FACTOR` e `1.3`, definido no topo do mesmo arquivo. entao um
`TextStyle::new(16.0)` nasce com altura de linha de 20.8, peso 400, sem
espacamento, familia default. os modificadores `with_weight`, `with_letter_spacing`,
`with_family`, `with_line_height` sao builder methods que devolvem `Self` por
valor, no estilo que o crate inteiro usa: voce encadeia
`TextStyle::new(14.0).with_family("Inter").with_weight(600)` e tem o estilo
montado numa expressao so.

repara no `#[derive(... PartialEq)]` no topo do struct. esse `PartialEq` nao e
decoracao. ele e o que permite, mais tarde, usar a `TextStyle` (e os bits dos
campos dela) como chave de cache nos dois lados. dois estilos iguais sao `==`, e
isso e o que faz o cache de medida e o cache de shaping reconhecerem que "ja vi
esse exato pedido antes". se a `TextStyle` nao fosse comparavel campo a campo, o
cache nao teria como funcionar, e a regra de "um estilo, dois consumidores"
perderia a metade que torna ela barata.

## um por run, e o que e um run

o titulo do capitulo diz "uma TextStyle por run", entao vale parar no que e um
run. ele aparece logo abaixo da `TextStyle`, no `StyleRun`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct StyleRun {
    pub range: Range<usize>,
    pub style: TextStyle,
}
```

um `StyleRun` e um pedaco do texto (um intervalo de bytes, `range`) com um estilo
(`style`). a ideia, no horizonte, e rich text: voce quer um trecho em negrito no
meio de um paragrafo, ou syntax highlighting onde cada token tem cor e peso
proprios. cada um desses trechos e um run, e cada run carrega uma `TextStyle`.
um por run. nao dois. o trecho em negrito e medido e desenhado pelo mesmo estilo
negrito, o trecho normal pelo mesmo estilo normal.

agora, honestidade sobre o estado atual. o comentario no codigo e claro: o
backend de cosmic-text hoje so honra um estilo por texto, o do primeiro run. o
metodo interno que escolhe o estilo e literalmente isto:

```rust
fn run_style(runs: &[StyleRun]) -> TextStyle {
    runs.first()
        .map(|run| run.style.clone())
        .unwrap_or_default()
}
```

pega o primeiro run, clona o estilo dele, e se a lista vier vazia usa o default.
os atributos por span, com cada run no seu estilo, chegam com o rich text (a
nota no codigo marca como WS-A.3, ainda nao confirmado quando). entao na pratica,
agora, e um estilo por texto. mas a estrutura ja esta certa: o tipo e
`&[StyleRun]`, plural, e o dia que cada run for shapeado com o seu estilo, a
regra "um por run" passa de aspiracao a implementacao sem mudar o contrato. o
importante pro capitulo nao muda: seja um run ou doze, cada run tem uma
`TextStyle`, e essa `TextStyle` e a unica fonte de verdade pro tamanho e pro
pixel daquele run.

## o shaping, ou: o que cosmic-text faz com a sua palavra

shaping e a palavra tecnica pro trabalho que transforma uma sequencia de
caracteres numa sequencia de glifos posicionados. nao e trivial e nao e linear.
"fi" pode virar uma ligadura, um glifo so. um caractere arabe muda de forma
dependendo dos vizinhos. um acento pode se combinar com a letra anterior. o
avanco de cada glifo (quanto o cursor anda depois de desenhar ele) depende da
fonte, do tamanho, do peso, do glifo seguinte (kerning). fazer isso na mao e
loucura, e por isso o engine delega pro cosmic-text 0.18, conferido no
`Cargo.toml` do workspace.

o caminho de shaping do lado da medida esta em
`crates/engine/src/text/measure.rs`, no metodo `prepare` do contexto de medida:

```rust
fn prepare(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) {
    let fs = &mut self.font_system;
    self.scratch
        .set_metrics(fs, Metrics::new(style.font_size, style.line_height));
    self.scratch.set_size(fs, max_width, None);
    self.scratch
        .set_text(fs, text, &attrs_for(style), Shaping::Advanced, None);
    self.scratch.shape_until_scroll(fs, false);
}
```

quatro chamadas, em ordem. `set_metrics` planta o tamanho da fonte e a altura de
linha (`Metrics::new(font_size, line_height)`, os dois saidos da `TextStyle`).
`set_size` define a largura maxima pra quebra de linha, `None` quando o texto nao
quebra. `set_text` e onde o texto entra com os atributos resolvidos por
`attrs_for(style)`, e repara no `Shaping::Advanced`: e o modo de shaping completo
do cosmic-text, com kerning, ligadura, reordenacao bidirecional, o pacote inteiro.
nao e o modo basico. e `shape_until_scroll` executa o shaping de fato.

o `attrs_for` e a ponte entre a `TextStyle` do engine e os atributos do
cosmic-text, e ele merece uma lida devagar porque tem uma conversao escondida:

```rust
fn attrs_for(style: &TextStyle) -> Attrs<'_> {
    let mut attrs = Attrs::new().weight(Weight(style.font_weight));
    if style.letter_spacing != 0.0 && style.font_size > 0.0 {
        // cosmic-text tracking is in EM; the engine API is px.
        attrs = attrs.letter_spacing(style.letter_spacing / style.font_size);
    }
    match style.font_family {
        Some(ref family) => attrs.family(Family::Name(family)),
        None => attrs,
    }
}
```

o peso vai direto, embrulhado no `Weight` do cosmic-text. o espacamento entre
letras tem a pegadinha: a API do engine fala em pixels, e a do cosmic-text fala
em EM (fracao do tamanho da fonte). entao o engine divide: `letter_spacing /
font_size` converte px pra EM na hora de passar. e a familia: se a `TextStyle`
nomeia uma familia, vira `Family::Name`, senao deixa a default. guarda essa
funcao na cabeca, ela vai aparecer de novo, identica em espirito, do outro lado
da fronteira, e e ai que a regra do capitulo vai ficar concreta.

uma palavra sobre o `FontSystem`. o cosmic-text precisa de um banco de fontes pra
fazer shaping, o `FontSystem`. o detalhe arquitetural fino aqui e que existem
dois, um de cada lado, e os dois registram exatamente as mesmas faces. o lado da
medida constroi o seu num `new_font_system`, o lado do desenho constroi o dele no
`TextSystem::new`, e os dois chamam a mesma funcao, `register_embedded_fonts`, do
arquivo `fonts.rs`:

```rust
pub(super) fn register_embedded_fonts(db: &mut Database) {
    // Rubik (SIL OFL, ...): HOFF UI sans.
    db.load_font_data(include_bytes!("../../assets/fonts/Rubik-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Rubik-Medium.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Rubik-SemiBold.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Rubik-Bold.ttf").to_vec());
    // Inter (...): named-family fallback.
    // ... Inter 400/500/600/700, JetBrains Mono, codicons ...
    db.set_sans_serif_family("Rubik");
    db.set_monospace_family("JetBrains Mono");
}
```

por que isso importa pro capitulo. se o lado da medida tivesse so a Inter-Regular
e o lado do desenho tivesse a Inter-SemiBold, eles fariam shaping de uma palavra
em peso 600 com faces diferentes, e os avancos divergiriam mesmo com a mesma
`TextStyle`. compartilhar o objeto de estilo nao basta se cada lado interpretar
esse estilo com um conjunto de fontes diferente. por isso a regra "uma TextStyle
por run" tem uma irma silenciosa: os dois lados registram as mesmas faces. o
comentario no proprio `fonts.rs` diz isso com todas as letras, uma face presente
de um lado so faz os avancos divergirem dos pixels. esse e o assunto inteiro do
proximo capitulo (embed de cada peso em uso), entao aqui eu so deixo o fio
amarrado: mesmo estilo, mesmas fontes, e a medida fica honesta.

## medir sem GPU, porque o layout vem antes da placa

uma das primeiras coisas que confunde quando voce olha o modulo de texto e que
tem dois lugares que parecem fazer a mesma coisa. tem o `measure.rs`, com o
`TextMeasurer`, e tem o `system.rs`, com o `TextSystem`. os dois fazem shaping
com cosmic-text. por que dois?

a resposta esta na ordem das coisas num frame. o layout (calcular onde cada caixa
fica) e o tratamento de input (descobrir onde voce clicou no texto) acontecem
antes de existir qualquer recurso de GPU. o `TextSystem` do lado do desenho e
casado com o atlas de glifos, que precisa de `device` e `queue` da placa pra
existir. mas o layout precisa saber a largura de um botao muito antes de a placa
entrar na historia. se a unica forma de medir texto dependesse da GPU, voce nao
conseguiria fazer layout sem ela, e isso quebraria o teste headless, o wasm antes
do contexto, tudo. o doc no topo do `measure.rs` explica exatamente isso: o lado
do render acopla o `FontSystem` ao atlas, entao a medida precisa do seu proprio
`FontSystem`, sem GPU.

por isso o `TextMeasurer` guarda o `FontSystem` num `thread_local`:

```rust
thread_local! {
    static MEASURE_CTX: RefCell<MeasureContext> = RefCell::new(MeasureContext::new());
}
```

o engine e single-threaded, e um `FontSystem` e caro de construir, caro demais pra
criar a cada chamada de medida. entao ele vive uma vez por thread, escondido atras
de um `RefCell`, e cada `measure` pega ele emprestado. e o `TextMeasurer` em si e
um struct vazio, uma fachada sem estado:

```rust
pub struct TextMeasurer;
```

todos os metodos sao associados, batem no `thread_local`, fazem o trabalho,
devolvem. a entrada principal e a `measure`, a ancora que o adr aponta como a
unica fonte sancionada de largura:

```rust
pub fn measure(text: &str, font_size: f32, max_width: Option<f32>) -> (f32, f32) {
    Self::measure_styled(text, &TextStyle::new(font_size), max_width)
}
```

repara que `measure` e so um atalho. ela monta um `TextStyle::new(font_size)` e
delega pra `measure_styled`. ou seja, mesmo a versao "simples" da medida, a que so
recebe um tamanho, constroi uma `TextStyle` por baixo. nao existe caminho de
medida que nao passe por uma `TextStyle`. esse e o ponto. a `measure_styled` e o
coracao:

```rust
pub fn measure_styled(text: &str, style: &TextStyle, max_width: Option<f32>) -> (f32, f32) {
    if text.is_empty() {
        return (0.0, 0.0);
    }
    let key = MeasureKey::new(text, style, max_width);
    MEASURE_CTX.with(|ctx| {
        let ctx = &mut *ctx.borrow_mut();
        if let Some(&size) = ctx.cache.get(&key) {
            return size;
        }
        ctx.prepare(text, style, max_width);
        let size = measure_runs(&ctx.scratch);
        ctx.cache.put(key, size);
        size
    })
}
```

texto vazio mede zero, atalho honesto. dai monta a chave de cache, consulta, e se
nao tem, faz o `prepare` (o shaping que vimos), extrai o tamanho com
`measure_runs`, guarda no cache e devolve. o `measure_runs` e a parte que de fato
le o resultado do shaping:

```rust
fn measure_runs(buffer: &Buffer) -> (f32, f32) {
    let mut width = 0.0_f32;
    let mut height = 0.0_f32;
    for run in buffer.layout_runs() {
        width = width.max(run.line_w);
        height = run.line_top + run.line_height;
    }
    (width, height)
}
```

a largura e o maximo das larguras de linha (`line_w`), a altura e o topo da
ultima linha mais a altura dela. e a largura real do texto shapeado, glifo por
glifo, com kerning e tudo. nao tem `0.58` aqui. nao tem `chars`. tem o que o
cosmic-text computou.

vale olhar a chave de cache, a `MeasureKey`, porque ela e onde a regra do
capitulo aparece como estrutura de dados:

```rust
struct MeasureKey {
    text: String,
    font_size_bits: u32,
    line_height_bits: u32,
    font_weight: u16,
    letter_spacing_bits: u32,
    font_family: Option<String>,
    width_bucket: Option<i32>,
}
```

a chave inclui peso e espacamento entre letras. isso e o adr falando: o
`TextMeasurer` cacheia por uma chave que inclui peso e letter spacing. por que
isso importa? porque se a chave ignorasse o espacamento, dois pedidos com
espacamentos diferentes colidiriam no cache e o segundo receberia a largura do
primeiro. e a versao em cache do bug do builder, a pilula media sem o tracking.
os `_bits` (os campos guardados como `u32` via `f32::to_bits`) sao por causa de
`f32` nao ser `Eq` nem `Hash`, entao o codigo guarda a representacao de bits, que
e. tem um teste que crava essa propriedade,
`letter_spacing_distinguishes_measure_cache`: mesmo texto, mesmo estilo, so o
espacamento muda, e a largura em cache tem que diferir.

um detalhe do `width_bucket`: a largura de quebra entra arredondada pro pixel
inteiro mais proximo (`max_width.map(|w| w.round() as i32)`). isso e uma escolha
de cache, agrupar larguras quase iguais no mesmo balde pra nao explodir o cache
com 0.1px de diferenca. o tamanho do cache de medida e 2048 entradas, e o de
metricas verticais 256, os dois conferidos no topo do `measure.rs`.

## a outra ponta: desenhar com o mesmo estilo

ate aqui falei da medida. a outra metade da regra e o desenho, e e onde ela ou se
cumpre ou se quebra. o desenho mora no `TextSystem`, em
`crates/engine/src/text/system.rs`, no `resolve_for_layer`. ele recebe os nos de
texto da cena e, na primeira fase, garante que cada um esta shapeado:

```rust
let mut buffer =
    Buffer::new(&mut self.font_system, Metrics::new(font_size, line_height));
buffer.set_size(&mut self.font_system, max_width, None);
let mut attrs = Attrs::new().weight(Weight(key.font_weight));
if letter_spacing != 0.0 && font_size > 0.0 {
    // cosmic-text tracking is in EM; the key stores px.
    attrs = attrs.letter_spacing(letter_spacing / font_size);
}
if let Some(ref family) = key.font_family {
    attrs = attrs.family(Family::Name(family));
}
buffer.set_text(&mut self.font_system, &key.text, &attrs, Shaping::Advanced, None);
buffer.shape_until_scroll(&mut self.font_system, false);
```

para e compara isso com o `prepare` e o `attrs_for` do lado da medida. e a mesma
sequencia. `Metrics::new(font_size, line_height)`. `set_size` com a largura de
quebra. os atributos montados com `Weight`, com a mesma conversao de letter
spacing px pra EM (`letter_spacing / font_size`, o comentario ate repete a frase),
com `Family::Name` quando tem familia. `set_text` com `Shaping::Advanced`.
`shape_until_scroll`. linha por linha, o desenho faz o que a medida faz. essa
repeticao quase literal nao e copia preguicosa, e a condicao de a regra valer:
medir e desenhar tem que produzir o mesmo shaping, entao os dois caminhos tem que
chamar o cosmic-text com os mesmos parametros derivados do mesmo estilo.

so que tem uma diferenca importante na superficie. o lado da medida recebe uma
`TextStyle`. o lado do desenho recebe uma `key.font_size`, `key.font_weight`,
`key.letter_spacing` soltos, de uma estrutura chamada `TextNodeKey`. e aqui que a
regra do capitulo poderia escorregar, e e aqui que o engine planta o segundo
metodo critico, em `crates/engine/src/compositor/scene.rs`:

```rust
pub fn from_style(text: &str, style: &crate::text::TextStyle, max_width: Option<f32>) -> Self {
    Self {
        text: text.to_string(),
        font_size_bits: style.font_size.to_bits(),
        line_height_bits: style.line_height.to_bits(),
        max_width_bits: max_width.map(|w| w.to_bits()),
        font_weight: style.font_weight,
        letter_spacing_bits: style.letter_spacing.to_bits(),
        font_family: style.font_family.clone(),
    }
}
```

`TextNodeKey::from_style` recebe uma `&TextStyle` e copia campo por campo pra
dentro da chave do no de desenho. ou seja, a `TextNodeKey` que o desenho usa nao e
montada na mao, com numeros digitados a parte. ela e derivada da mesma `TextStyle`
que a medida usou. o adr e explicito sobre isso: a `TextStyle` e a entrada de
`TextMeasurer::measure_styled` (dimensao) e de `TextNodeKey::from_style` (desenho).
um objeto, dois consumidores. e o adr tambem diz o que nunca fazer: nunca
construir um campo da `TextNodeKey` a mao ao lado de um spec de medida construido
a parte. use `from_style` no objeto compartilhado.

tem dois testes que guardam essa ponte. `text_node_key_from_style_carries_every_field`
monta um estilo com tamanho, altura de linha, peso, espacamento e familia, passa
por `from_style`, e verifica que cada um dos cinco campos chegou intacto na chave.
e `text_node_key_distinguishes_letter_spacing` verifica que duas chaves que so
diferem no espacamento sao `!=` e tem hash diferente, pra que o cache de shaping
do `TextSystem` (um `FxHashMap<TextNodeKey, ShapedEntry>`, com o `ShapedEntry`
guardando o `Buffer` shapeado, definido no `cache.rs`) nao confunda uma pilula
com tracking com uma sem.

## onde a divergencia nasce, e por que ela e defeito por definicao

junta as duas pontas. de um lado, `measure_styled(text, &style, max_width)`. do
outro, `TextNodeKey::from_style(text, &style, max_width)` alimentando o shaping do
desenho. se as duas recebem a mesma `&style`, o mesmo `text`, a mesma `max_width`,
elas fazem o mesmo shaping com a mesma fonte e produzem o mesmo resultado. a
largura que a forma reserva e a largura que o glifo ocupa. a conta fecha por
construcao.

agora pensa no que precisa acontecer pra ela nao fechar. alguem teria que chamar a
medida com um estilo e o desenho com outro. ou construir a `TextNodeKey` na mao,
com um peso diferente do que a medida usou. ou medir com espacamento zero e
desenhar com tracking. todos os tres casos do adr sao variacoes disso: duas fontes
de verdade pro mesmo texto. e o ponto mais afiado do adr, o que eu acho que e o
coracao da decisao, e este: construir os dois lados separadamente e um defeito por
definicao, independente de os valores por acaso baterem agora.

isso parece forte demais na primeira leitura. se os numeros batem, qual o
problema? o problema e o "por acaso". o caso do builder mostra o mecanismo: a
medida com `letter_spacing: 0.0` e o desenho com `.tracking()` batiam, porque o
tracking estava inerte. os valores coincidiam. e o codigo estava errado mesmo
assim, porque a coincidencia dependia de uma terceira coisa (o tracking nao estar
fazendo nada) que ninguem garantiu que ia continuar verdadeira. no dia que o
tracking ligou, a coincidencia evaporou e o bug apareceu em toda forma com
tracking de uma vez. dois caminhos que produzem o mesmo numero por motivos
independentes nao estao corretos, estao sortudos. e sorte nao e invariante.

por isso o adr define correcao no nivel da estrutura, nao do valor. nao e "a
medida e o desenho devem produzir a mesma largura". e "a medida e o desenho devem
sair do mesmo objeto". a primeira formulacao voce so consegue testar comparando
numeros, depois do fato, caso a caso, pra sempre. a segunda voce garante no tipo:
se a unica forma de chegar numa `TextNodeKey` for `from_style`, e a unica forma de
medir for `measure_styled`, e as duas pedem `&TextStyle`, entao nao existe lugar
no codigo pra plantar a divergencia. ela deixa de ser um bug que voce caca e vira
um estado que o codigo nao representa.

## o numero, e a honestidade sobre o que tem e o que nao tem

eu gosto de fechar com numero. nesse capitulo o numero exige cuidado, porque tem
dois tipos de numero rondando e eles nao sao a mesma coisa.

o primeiro tipo sao os numeros do defeito, e esses sao reais e estao no adr. o
erro do ide ia de -10% a +21% entre overflow e pilula larga. a heuristica velha
era `chars * font_size * 0.58`, e a versao mais antiga `chars * 0.6`. esses
numeros nao sao benchmark, sao a magnitude do problema que a regra fechou. o teste
`round_trip_narrow_chars_proportional` ate deixa um comentario sobre o `0.6`: ele
shapeia `"illiilli"`, oito glifos finos, e o comentario diz que a razao velha de
0.6 falhava exatamente ali, porque `i` e `l` na Inter sao estreitissimos e a media
chutada estourava a conta. e ha os limiares dos testes de regressao de peso:
faces irmas de uma familia diferem ate uns 10% em avanco (a Rubik Bold roda uns
8.5% mais larga que a Rubik Regular), enquanto uma queda pra familia errada
desvia 35% ou mais. o teste usa um limiar de 13% pra pegar o bug de fallback sem
acusar a variacao real entre faces. e o teste de letter spacing crava a mecanica:
0.025em a 14px da 0.35px de tracking, e o delta da largura tem que bater
`spacing * (n_glifos - 1)`, dentro de 1px.

o segundo tipo de numero seria um microbenchmark do shaping, "medir um label X
custa Y microssegundos". e aqui eu preciso ser direto: nao confirmado. o
`SUMARIO.md` lista a secao de texto com `bench: n/a`, e eu nao encontrei nas
ancoras deste capitulo um benchmark que isole o custo de `measure_styled` ou do
shaping do `TextSystem`. existe um grupo `text_hashing` no `scene_build.rs` (eu o
vi citado no capitulo do GpuVec), mas ele mede o hashing de nos de texto, nao o
shaping em si, e eu nao confirmei o numero dele aqui. entao nao vou te vender um
"tempo de shaping". o que da pra dizer com fundamento, e isso e arquitetura e nao
chute, e que o custo de shapear o mesmo texto duas vezes (uma na medida, uma no
desenho) e amortizado por dois caches independentes: o `cache` do `MeasureContext`
(2048 entradas, chaveado por `MeasureKey`) e o `shaping_cache` do `TextSystem`
(`FxHashMap` chaveado por `TextNodeKey`, com despejo por frame). um label que nao
muda e shapeado uma vez de cada lado e depois sai do cache nos frames seguintes. o
preco de ter dois `FontSystem` e dois caches e o preco de poder medir antes da GPU
e desenhar com a GPU sem que um caminho dependa do outro. eu acho um preco justo,
mas e uma opiniao sobre o trade, nao um numero medido.

## por que assim, e nao mais simples

da pra perguntar se nao seria mais limpo ter um lugar so. um `TextSystem` unico
que mede e desenha, sem o `TextMeasurer` separado. menos codigo, um `FontSystem`
so, um cache so. e seria, ate o momento em que voce precisa medir um botao pra
fazer layout antes de a placa de video existir, e ai voce descobre que acoplou a
medida ao atlas de glifos e nao consegue mais rodar layout headless, nem em teste,
nem no wasm antes do contexto. o split em dois nao e duplicacao por desleixo, e a
fronteira certa: a medida e GPU-free de proposito, e o desenho e GPU-bound de
proposito, e a regra "uma TextStyle por run" e o fio que costura os dois pra que,
apesar de separados, eles nunca discordem.

e da pra perguntar o contrario tambem: por que nao deixar cada widget medir do seu
jeito, com a heuristica que quiser? porque foi exatamente isso que aconteceu no
ide, doze lugares medindo cada um a sua maneira, e o adr conta o desfecho: um unico
ponto de substituicao, `hoff::measure_text` (a nota do adr; eu nao abri esse
arquivo, marco como nao confirmado o caminho exato), consertou os doze de uma vez.
quando a medida e uma funcao centralizada e sancionada, corrigir a medida e
corrigir todo mundo que mede. quando cada um tem a sua, voce conserta um e os
outros onze continuam errados, cada um do seu jeito. centralizar a verdade e o que
torna o conserto atomico.

a parte que eu acho mais bonita, e que vale levar pra fora do texto, e que a regra
move o problema de um regime onde voce testa valores pra um regime onde voce
garante estrutura. testar valor e infinito: pra cada palavra nova, cada peso novo,
cada tamanho novo, voce teria que conferir se a medida bate com o desenho. garantir
estrutura e finito: voce prova uma vez que `measure_styled` e `from_style` saem do
mesmo `&TextStyle`, e a partir dali a coincidencia de largura nao e mais algo que
voce torce pra acontecer, e algo que nao tem como nao acontecer. o teste
`backend_measure_matches_measurer` e `shaped_text_size_matches_measure` cravam
isso na ponta: o backend mede igual ao measurer, e o `ShapedText.size()` bate com
`measure_styled`, byte a byte, com `assert_eq!`, nao com tolerancia. quando voce
consegue usar `assert_eq!` em vez de "dentro de 0.5px" entre medida e desenho, e
porque os dois sao a mesma computacao, nao duas parecidas.

## o que isso me ensinou

a licao que eu levei daqui nao foi sobre tipografia. foi sobre uma forma de
fechar bug que eu subestimava. a tentacao, quando a pilula vaza, e medir melhor.
achar a constante certa, calibrar o fator, somar um padding de seguranca. e isso
funciona pro caso que voce esta olhando e falha pro proximo, porque voce esta
tratando o sintoma (os 4 pixels) e nao a causa (os dois modelos). a regra de uma
`TextStyle` por run, compartilhada por medida e desenho, nao mede melhor. ela
remove a possibilidade de medir e desenhar diferente. e uma classe inteira de bug
que para de existir porque o codigo deixa de ter como expressar ela.

e a segunda licao, a que demorou mais a cair a ficha: a melhor regra nao e a que
te diz qual valor esta certo, e a que faz o valor errado nao ter onde morar. o
adr nao manda "confira se a largura medida bate com a desenhada". ele manda
"meca e desenhe a partir do mesmo objeto", e a largura bater vira consequencia, nao
obrigacao. dado um pedaco de codigo que cuida de texto, eu aprendi a perguntar
primeiro nao "esse calculo esta certo?", e sim "quantas fontes de verdade existem
pra esse numero?". se for mais de uma, o resto e questao de tempo ate elas
discordarem. uma `TextStyle`, um shaping, dois consumidores que leem o mesmo
resultado. a forma para de mentir porque so existe uma verdade pra ela contar.

## rastros

codigo (crate engine, conferido contra a arvore atual e contra o Cargo.toml:
cosmic-text 0.18)

- `crates/engine/src/text/backend.rs:15` (`DEFAULT_LINE_HEIGHT_FACTOR = 1.3`)
- `crates/engine/src/text/backend.rs:19` (`struct TextStyle`, cinco campos:
  `font_size`, `line_height`, `font_weight`, `letter_spacing`, `font_family`,
  com `#[derive(... PartialEq)]`)
- `crates/engine/src/text/backend.rs:20` (`pub font_size: f32`, primeiro campo da
  `TextStyle`)
- `crates/engine/src/text/backend.rs:31` (`TextStyle::new`, defaults: line_height
  = font_size * 1.3, weight 400, letter_spacing 0.0, family None)
- `crates/engine/src/text/backend.rs:73` (`struct StyleRun`, `range` + `style`, a
  unidade de rich text)
- `crates/engine/src/text/backend.rs:111` (`run_style`, hoje usa
  `runs.first()`/`unwrap_or_default`: um estilo por texto ate o rich text)
- `crates/engine/src/text/backend.rs:121` (`shape` delega pra
  `TextMeasurer::shape`)
- `crates/engine/src/text/backend.rs:125` (`measure` delega pra
  `TextMeasurer::measure_styled`)
- `crates/engine/src/text/measure.rs:26` (`thread_local! MEASURE_CTX`, FontSystem
  GPU-free vivo uma vez por thread)
- `crates/engine/src/text/measure.rs:23` (`MEASURE_CACHE_CAPACITY = 2048`,
  `VMETRICS_CACHE_CAPACITY = 256`)
- `crates/engine/src/text/measure.rs:40` (`struct MeasureKey`, inclui
  `font_weight` e `letter_spacing_bits`; campos f32 como `_bits` via `to_bits`;
  `width_bucket` arredondado ao pixel)
- `crates/engine/src/text/measure.rs:100` (`prepare`: `set_metrics`/`set_size`/
  `set_text` com `Shaping::Advanced`/`shape_until_scroll`)
- `crates/engine/src/text/measure.rs:132` (`attrs_for`: `Weight`, conversao
  letter_spacing px -> em via `/ font_size`, `Family::Name`)
- `crates/engine/src/text/measure.rs:174` (`measure_runs`: largura = max
  `line_w`, altura = `line_top + line_height`)
- `crates/engine/src/text/measure.rs:266` (`struct TextMeasurer`, fachada sem
  estado)
- `crates/engine/src/text/measure.rs:271` (`measure` monta `TextStyle::new` e
  delega: nao ha caminho de medida sem `TextStyle`)
- `crates/engine/src/text/measure.rs:277` (`measure_styled`: vazio -> (0,0),
  cache get/put por `MeasureKey`, shaping, `measure_runs`)
- `crates/engine/src/text/measure.rs:416` (`shape` -> `ShapedText`)
- `crates/engine/src/text/system.rs:43` (`TextSystem::new`, casado com o atlas,
  precisa de device/queue; chama `register_embedded_fonts`)
- `crates/engine/src/text/system.rs:40` (`GLYPH_CACHE_CAPACITY = 4096`)
- `crates/engine/src/text/system.rs:120` (`resolve_for_layer`, shaping do lado do
  desenho)
- `crates/engine/src/text/system.rs:147` (fase 1: `Buffer::new`/`set_size`/
  `Attrs` com `Weight`, mesma conversao letter_spacing px->em, `Family::Name`,
  `set_text` `Shaping::Advanced`, `shape_until_scroll`)
- `crates/engine/src/text/system.rs:30` (`shaping_cache: FxHashMap<TextNodeKey,
  ShapedEntry>`, despejo por frame em `finish_frame`)
- `crates/engine/src/text/fonts.rs:23` (`register_embedded_fonts`: Rubik e Inter
  400/500/600/700, JetBrains Mono, codicons; `set_sans_serif_family("Rubik")`;
  o mesmo registro nos dois FontSystem)
- `crates/engine/src/text/cache.rs:43` (`struct ShapedEntry { buffer }`, o Buffer
  shapeado guardado no cache de desenho)
- `crates/engine/src/compositor/scene.rs:111` (`struct TextNodeKey`, campos em
  `_bits`, mesma forma da `MeasureKey`)
- `crates/engine/src/compositor/scene.rs:140` (`TextNodeKey::from_style`, deriva a
  chave de desenho da mesma `&TextStyle`, campo por campo)

testes de regressao

- `crates/engine/src/text/tests_measure.rs:266`
  (`backend_measure_matches_measurer`: backend mede `assert_eq!` ao measurer)
- `crates/engine/src/text/tests_measure.rs:306`
  (`shaped_text_size_matches_measure`: `ShapedText.size()` == `measure_styled`)
- `crates/engine/src/text/tests_measure.rs:316`
  (`letter_spacing_increases_advance_per_glyph`: 0.025em a 14px = 0.35px, delta ~
  `spacing*(n-1)`)
- `crates/engine/src/text/tests_measure.rs:339`
  (`letter_spacing_distinguishes_measure_cache`: espacamento diferente, largura em
  cache diferente)
- `crates/engine/src/text/tests_measure.rs:348`
  (`text_node_key_distinguishes_letter_spacing`: chaves `!=` e hash distinto)
- `crates/engine/src/text/tests_measure.rs:367`
  (`text_node_key_from_style_carries_every_field`: os cinco campos chegam intactos
  via `from_style`)
- `crates/engine/src/text/tests_measure.rs:108`
  (`assert_advance_close_to_regular`: faces irmas ate ~10%, fallback errado ~35%+,
  limiar 13%; Rubik Bold ~8.5% mais larga que Regular)
- `crates/engine/src/text/tests_measure.rs:213`
  (`round_trip_narrow_chars_proportional`: `"illiilli"`, onde a razao velha de 0.6
  falhava)

adr

- `kdb/adr/one-text-style-for-measurement-and-drawing.md:13` (defeito recorrente:
  etiqueta maior que a forma, texto vaza; um mecanismo em toda instancia)
- `kdb/adr/one-text-style-for-measurement-and-drawing.md:22` (ide:
  `chars * font_size * 0.58`, desenho em Rubik 600, erro -10% a +21%)
- `kdb/adr/one-text-style-for-measurement-and-drawing.md:24` (builder: medida com
  `letter_spacing: 0.0` cravado vs `.tracking()` no desenho, inerte, divergencia
  latente)
- `kdb/adr/one-text-style-for-measurement-and-drawing.md:27` (engine antigo:
  `chars * 0.6`, sintoma identico)
- `kdb/adr/one-text-style-for-measurement-and-drawing.md:31` (decisao: um
  `TextStyle` por run, entrada de `measure_styled` e de `from_style`; construir os
  dois separados e defeito por definicao)
- `kdb/adr/one-text-style-for-measurement-and-drawing.md:37` (`TextMeasurer` e a
  unica fonte sancionada de largura, GPU-free, cacheia por chave com peso e letter
  spacing)
- `kdb/adr/one-text-style-for-measurement-and-drawing.md:43` (consequencia: largura
  da forma = texto medido + padding por construcao)
- `kdb/adr/one-text-style-for-measurement-and-drawing.md:45` (um ponto de
  substituicao, `hoff::measure_text`, consertou doze call sites do ide de uma vez)
- `kdb/adr/one-text-style-for-measurement-and-drawing.md:51` (evitar: nunca estimar
  largura aritmeticamente; nunca construir campo da TextNodeKey ao lado de spec de
  medida separado; plumb de novo atributo nos dois lados na mesma mudanca)

nao confirmado

- nao existe, nas ancoras deste capitulo, um microbenchmark que isole o custo de
  `measure_styled` ou do shaping do `TextSystem`. o `SUMARIO.md` marca a secao de
  texto como `bench: n/a`. o grupo `text_hashing` do `scene_build.rs` mede hashing
  de nos, nao shaping, e seu numero nao foi conferido aqui. as afirmacoes de custo
  amortizado por cache sao arquitetura (os dois caches existem e estao chaveados),
  nao tempo medido.
- o caminho exato `hoff::measure_text` vem do texto do adr; eu nao abri esse
  arquivo, entao cito como a nota do adr o registra, sem confirmar o file:line do
  simbolo.
- a chegada do rich text com um estilo de fato por run (per-span) e citada no
  codigo como WS-A.3, futura; nao confirmei a data nem o estado dessa workstream.
