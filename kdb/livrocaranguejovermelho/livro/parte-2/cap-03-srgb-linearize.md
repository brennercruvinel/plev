---
title: cor, srgb e linearizar uma vez na entrada
parte: 2
status: rascunho
rastros:
  - crates/engine/src/color.rs:70
  - crates/engine/src/color.rs:61
  - crates/engine/src/color.rs:185
  - crates/engine/src/theme/color_space.rs:22
  - crates/engine/src/theme/color_space.rs:40
  - crates/engine/src/gpu/surface.rs:73
  - crates/engine/src/gpu/surface.rs:122
  - crates/engine/src/gpu/surface.rs:134
  - crates/engine/src/window/render.rs:145
  - crates/engine/src/gpu/shaders/quad.wgsl:29
  - crates/showcase/src/renderer.rs:73
  - crates/ide/src/renderer.rs:84
  - kdb/adr/linearize-colors-before-the-gpu.md
  - kdb/adr/render-into-an-srgb-view-format.md
---

# cor, srgb e linearizar uma vez na entrada

eu escolhi um cinza. `#303030`, o fundo de pagina do tema. coloquei na tela,
rodei, e o que apareceu nao era esse cinza. era um cinza bem mais claro, lavado,
sem contraste com o que estava por cima. minha primeira reacao foi a errada, e e a
reacao que quase todo mundo tem: o token esta claro demais, escurece o token. fui
la, troquei pra um cinza mais escuro, rodei de novo, e aquela tela ficou aceitavel.
fui pra proxima tela. errada de novo. outro fundo, outro cinza lavado.

esse e o momento em que voce precisa parar e desconfiar de que o problema nao e a
cor. e o caminho que a cor percorre ate virar pixel. porque se cada cinza que voce
poe sai claro demais por um fator parecido, nao tem token errado, tem uma conta
errada acontecendo entre o valor que voce escreveu e o valor que a GPU pintou. e a
correcao por token, escurecer um a um, e exatamente o tipo de band-aid que esse
projeto aprendeu a punir: conserta a tela da frente e deixa toda cor futura errada.

o numero que fechou o caso foi medido, nao olhado. o token `#303030` tem valor 48
por canal. na tela, medido pixel a pixel, ele saia 118. quase duas vezes e meia
mais claro. depois da correcao, o mesmo token de 48 mediu 50. a validacao foi a
regua no pixel, nao a inspecao visual, e isso importa porque o olho perdoa, a
medicao nao. esses dois numeros estao registrados no adr da decisao
(`kdb/adr/linearize-colors-before-the-gpu.md`).

este capitulo desmonta esse caminho. abre na ideia humana de que existem dois jeitos
de guardar um cinza, desce ate a funcao real que faz a conversao, e termina no ponto
em que o formato do alvo de render decide se voce precisa fazer a conta ou nao. a
regra inteira cabe numa frase: a cor entra em srgb e e linearizada exatamente uma
vez, no ponto em que ela entra na memoria da GPU. o resto do capitulo e por que essa
frase tem cada uma das suas palavras.

## dois jeitos de guardar o mesmo cinza

quando voce escreve `#303030` num CSS, num token de tema, num literal hex, voce esta
falando em srgb. srgb e o espaco em que quase toda cor do mundo digital vive: os
arquivos de imagem, os valores de design, o que o monitor espera receber. e ele tem
uma propriedade que parece detalhe e e o centro de tudo: ele nao e linear.

o motivo e o olho. a gente enxerga mais degraus de diferenca na parte escura do que
na parte clara. se voce gastasse os 256 niveis de um canal de 8 bits de forma linear,
do preto ao branco em passos iguais de luz fisica, voce desperdicaria quase todos os
codigos nas regioes claras, onde o olho mal distingue, e ficaria com poucos degraus
no escuro, onde o olho enxerga banda. entao srgb dobra a escala. ele guarda mais
resolucao no escuro com uma curva, a tal funcao de transferencia, que comprime a
parte de baixo e estica a parte de cima. o valor 48 num canal srgb nao significa "48
de 255 de luz". significa "o codigo 48 nessa curva", que em luz fisica de verdade da
por volta de 0.0296 de 1.0. bem mais escuro do que a fracao 48/255 sugere.

a GPU nao quer essa curva. ela quer luz linear. todo calculo que mistura cor, e a
composicao e isso o tempo todo, soma de camadas, alpha por cima de alpha, blur que e
media de vizinhos, antialias que e media na borda, so da o resultado fisicamente
certo se os numeros forem proporcionais a luz. somar dois cinzas em srgb e somar dois
codigos numa regua torta: o meio dos dois nao e o cinza do meio. somar os mesmos dois
cinzas em linear da o meio certo. entao existe um lugar, e so um, onde a cor precisa
sair da curva srgb e virar luz linear. esse lugar e a fronteira de entrada da GPU.

vale separar aqui uma confusao que custa caro. existe mais de uma linearizacao no
codigo, e elas servem pra coisas diferentes. essa do capitulo e a da fronteira de
desenho. tem outra, em `crates/engine/src/theme/color_space.rs`, que linearzia srgb
pra entrar no OKLCH e derivar variantes de token de forma perceptualmente uniforme
(srgb vira linear, vira LMS, vira OKLab). e a mesma curva matematica, a funcao
`srgb_to_linear` em `color_space.rs:22` e identica em forma a da fronteira, mas o
proposito e outro: aquela conta acontece no CPU, na hora de derivar cor de design,
e o resultado volta pra srgb antes de qualquer coisa tocar a tela. juntar as duas na
cabeca leva voce a linearizar duas vezes ou no lugar errado. sao duas portas
diferentes pro mesmo predio.

## a funcao: linearizar uma vez, no CPU, pro valor que vira clear ou uniform

a fronteira do CPU tem um nome no codigo. e `Color::to_linear_array`, em
`crates/engine/src/color.rs:70`. o `Color` e um tipo simples, quatro `f32` RGBA no
intervalo de 0.0 a 1.0, e a conversao e isto:

```rust
/// The color with its RGB channels converted from sRGB (the space hex/CSS
/// values live in) to linear, alpha untouched.
pub fn to_linear_array(self) -> [f32; 4] {
    let lin = |c: f32| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    [lin(self.0[0]), lin(self.0[1]), lin(self.0[2]), self.0[3]]
}
```

repara em duas coisas que parecem pequenas e nao sao. a primeira: tem um galho. o
trecho de baixo da curva, ate 0.04045, e uma reta, `c / 12.92`. de la pra cima e a
potencia `((c + 0.055) / 1.055).powf(2.4)`. srgb nao e uma gamma pura de 2.4, ele e
uma reta curta perto do preto colada numa potencia no resto. esse pedaco reto existe
pra evitar derivada infinita no zero e pra dar comportamento bom no escuro profundo.
quem implementa "gamma" como um `pow(2.4)` solto erra justo o canto onde o olho mais
repara. o codigo nao corta esse canto.

a segunda: o alpha nao passa pela conta. `self.0[3]` entra e sai igual. alpha nao e
cor, e cobertura, quanto daquele pixel pertence aquela camada. cobertura ja vive em
linear, ela e uma fracao de area, nao um codigo de curva. linearizar o alpha seria
inventar uma transformacao que nao existe e bagunçar a composicao. o comentario da
funcao deixa isso explicito ja na assinatura, "alpha untouched", e o teste de
regressao crava: a verificacao `assert_eq!(g[3], 1.0, "alpha must stay linear")` em
`color.rs:185` reprova qualquer mudanca que comece a mexer no quarto canal.

o uso disso num app e direto. o showcase pega o fundo do tema e linearzia antes de
entregar pro clear, em `crates/showcase/src/renderer.rs:73`:

```rust
let [cr, cg, cb, ca] = view.theme.colors.bg.to_linear_array();
```

o ide faz igual, em `crates/ide/src/renderer.rs:84`, com `theme.bg_body`. e a janela
do proprio engine monta o `wgpu::Color` de clear a partir do mesmo `to_linear_array`,
em `crates/engine/src/window/render.rs:145`:

```rust
// wgpu clear values are linear; the sRGB surface re-encodes on
// write. Linearize the sRGB theme color so the bg shows its true
// tone instead of a washed-out ~2.5x lighter gray.
let bg = self.theme.colors.bg.to_linear_array();
wgpu::Color {
    r: bg[0] as f64,
    g: bg[1] as f64,
    b: bg[2] as f64,
    a: 1.0,
}
```

o detalhe que esse comentario carrega e a chave do capitulo inteiro: os valores de
clear do wgpu sao tratados como lineares. a GPU vai re-encodar linear para srgb na
hora da escrita, porque a superficie e um formato srgb. se voce entregar o srgb cru
ali, sem linearizar, a GPU assume que aquilo ja era linear e encoda de novo. dois
encodes empilhados. e o fator de 2.5 que lavou a tela.

## por que "uma vez" e nao zero, e nao duas

a palavra mais importante da regra e "uma". nao e que linearizar seja bom e quanto
mais melhor. linearizar de menos e linearizar de mais dao bug, e os dois bugs sao o
mesmo erro de raiz visto de dois lados.

o lado de zero conversao e o que eu abri o capitulo contando. no desktop, a
superficie da janela e um formato srgb. isso quer dizer que a GPU faz um servico
automatico: toda vez que ela escreve um pixel nessa superficie, ela aplica a curva
linear para srgb sozinha. esse encode-na-escrita e a suposicao que o pipeline inteiro
faz. entao o que voce poe na memoria tem que estar em linear, porque a GPU vai botar
de volta na curva. por meses o engine entregou srgb cru pra GPU como se fosse linear.
a GPU re-encodou. o `#303030` de codigo 48, que em linear seria 0.0296, foi tratado
como se 0.188 ja fosse luz linear e re-encodado pra cerca de 0.46, que e `#767676`.
midtone subiu por volta de 2.5 vezes. medido: 48 virou 118. o sintoma chegou varias
vezes como "cinza lavado, sem contraste", e foi atribuido errado a escolha de token
mais de uma vez. o adr registra isso e a recusa explicita do band-aid de escurecer
token.

o lado de conversao demais aparece na web, e e o inverso exato pela mesma raiz. a API
de canvas do WebGPU so aceita formato de superficie nao-srgb, `bgra8unorm` ou
`rgba8unorm`. quer dizer: na web nao existe o encode-na-escrita automatico que o
desktop tem. se voce linearizar a cor pra 0.0296 e escrever numa superficie que nao
re-encoda, o 0.0296 vai cru pra tela e e mostrado como escuro demais. o adr
`render-into-an-srgb-view-format.md` mediu isso: o fundo de pagina deu (8,8,8) em vez
de (48,48,48). e o mesmo `#303030`, agora escuro demais por falta do encode em vez de
claro demais por excesso. um pipeline, uma suposicao de raiz, dois sintomas opostos.

a saida nao foi um galho por plataforma. botar um `if web` que decide linearizar ou
nao seria espalhar a decisao por todo call site e garantir que um dia alguem esquece
de um. a saida foi fazer a web se comportar como o desktop no ponto que importa: dar
a ela uma superficie que tambem encoda na escrita.

## o alvo srgb: a outra metade da conta

a decisao mora em `crates/engine/src/gpu/surface.rs`. na hora de configurar a
superficie, o codigo pega o formato base e pede a variante srgb dele pra lista de
`view_formats`, em `surface.rs:73`:

```rust
let render_format = format.add_srgb_suffix();
self.surface_config.view_formats = if render_format != format {
    vec![render_format]
} else {
    vec![]
};
```

`add_srgb_suffix()` e a peca esperta. no desktop, onde o formato base ja e srgb, ela
e a identidade: nao tem sufixo pra adicionar, `render_format == format`, a lista de
`view_formats` fica vazia, e o mecanismo inteiro vira um no-op. na web, onde o base e
`bgra8unorm`, ela devolve `bgra8unorm-srgb`, a lista ganha esse formato, e agora
existe uma view srgb pra escrever. a diferenca entre as plataformas fica expressa uma
vez, na configuracao, e nao espalhada em condicional de runtime.

a partir dai, todo o resto do engine consulta o formato de view, nao o da textura.
`surface_format()`, em `surface.rs:122`, devolve o formato de view quando ele existe:

```rust
pub fn surface_format(&self) -> wgpu::TextureFormat {
    self.surface_config
        .view_formats
        .first()
        .copied()
        .unwrap_or(self.surface_config.format)
}
```

e e por isso que os pipelines, as texturas de camada do compositor e a view da
superficie ficam todos consistentes: todos perguntam pra mesma funcao qual e o
formato que encoda. se o pipeline fosse criado com um formato e a view com outro, o
encode aconteceria num lugar e nao no outro, e voce teria metade da tela certa.

a parte que mais pega gente desavisada e como a view da superficie e criada.
`surface_render_view`, em `surface.rs:134`, e o unico jeito sancionado:

```rust
pub fn surface_render_view(&self, output: &wgpu::SurfaceTexture) -> wgpu::TextureView {
    output.texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(self.surface_format()),
        ..Default::default()
    })
}
```

o ponto e o `format: Some(self.surface_format())`. se voce chamar o caminho
preguicoso, `texture.create_view(&Default::default())`, a view herda o formato da
propria textura, que na web e o nao-srgb, e pula o encode de gamma calado. e o tipo
de falha que sobrevive a revisao, porque ela e invisivel no desktop, onde o formato
base ja e srgb e o `Default` calha de dar certo, e catastrofica na web, onde ela
escurece tudo. o adr conta que os sete call sites de render foram migrados pra
`surface_render_view` de uma vez por isso, e marca a regra: nunca criar view de
superficie com `Default` em codigo novo. a regra geral do engine, "render targets de
superficie so via gpu.surface_render_view", e exatamente esse cuidado virado lei.

agora da pra ver a conta inteira fechando. a cor entra srgb. `to_linear_array` tira
ela da curva no CPU, uma vez, e poe o valor linear no clear. a superficie e srgb, por
formato no desktop ou por view-format na web, entao a GPU re-encoda na escrita e
devolve o pixel pro lugar certo da curva. 48 entra, 48 sai. o que estava quebrado era
sempre uma das duas metades faltando: ou a linearizacao no CPU, ou o encode na
escrita.

## a outra porta: cores de vertice linearizadas no shader

o clear e os uniforms montados no CPU passam por `to_linear_array`. mas nem toda cor
chega por ai. as cores que vao em cada vertice de um quad, de um retangulo, de uma
sombra, viajam pra GPU dentro do buffer de geometria e sao linearizadas la dentro, no
shader, em WGSL. abrir os dois caminhos e ver que sao o mesmo galho matematico ajuda
a confiar que nao tem duas verdades brigando.

o shader do quad, `crates/engine/src/gpu/shaders/quad.wgsl:29`, carrega a mesma curva:

```wgsl
// sRGB (the space theme/hex colors live in) -> linear. The surface is an sRGB
// format, so the GPU re-encodes linear->sRGB on write; without this an #303030
// fill would be treated as linear and shown ~2.5x too light.
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Linearize, then premultiplied alpha output.
    let rgb = srgb_to_linear(in.color.rgb);
    return vec4<f32>(rgb * in.color.a, in.color.a);
}
```

e a `to_linear_array` do CPU escrita em vetor. o galho em 0.04045, a reta `/ 12.92`,
a potencia `2.4`, tudo igual. so muda a forma: o `select(hi, lo, cond)` do WGSL faz o
papel do `if` do Rust por canal, em paralelo nos tres. esse mesmo `srgb_to_linear`
aparece nos shaders de `rect_sdf`, `text`, `shadow` e `shadow_analytic`. a regra do
engine de uma `TextStyle` por run de texto, compartilhada entre medicao e desenho,
encosta aqui: a cor do texto tambem e srgb e tambem e linearizada no shader, no mesmo
ponto, com a mesma curva.

a ordem da ultima linha tem um detalhe que e facil inverter e quebra a composicao.
primeiro linearzia, depois multiplica pelo alpha: `rgb * in.color.a`. isso e
premultiplied alpha, e ele tem que acontecer em luz linear pra a borda misturar
certo. se voce multiplicasse pelo alpha ainda em srgb, a transparencia ficaria com
halo, porque estaria fazendo media na regua torta de novo, so que na cobertura. o
comentario do shader marca a ordem de proposito: "Linearize, then premultiplied alpha
output".

## o que voce nao converte: textura que ja chegou decodificada

ter a regra "linearize na entrada" nao significa linearizar tudo que entra. tem cor
que ja chega linear, e converter de novo seria o bug de conversao-demais por outra
porta. o adr lista o que fica de fora: textura de imagem do atlas, backdrop,
composite e blur. essas chegam ja decodificadas, e quando uma textura criada de uma
imagem decodificada e amostrada atraves de um formato de textura srgb, o proprio
sample ja devolve linear. linearizar uma segunda vez escureceria a imagem do mesmo
jeito que a web escurecia o fundo.

entao o mapa mental e por porta, nao por cor. tem tres portas pra memoria da GPU e
cada uma tem sua regra. clear e uniform montado no CPU: `to_linear_array`. cor de
vertice no buffer de geometria: `srgb_to_linear` no shader. textura ja decodificada:
nada, ela ja esta linear na hora do sample. a unica forma de errar e mandar uma cor
pela porta errada, e e por isso que a fronteira e desenhada com nome de funcao e nao
deixada pra disciplina de quem escreve cada call site.

uma digressao que vale, porque eu ja me enrolei com ela. "formato de textura srgb"
faz duas coisas dependendo da direcao. na escrita, encoda linear pra srgb. na
leitura, no sample, decoda srgb pra linear. e o mesmo formato fazendo o trabalho
simetrico nas duas pontas. e por isso que uma textura de imagem amostrada por formato
srgb ja sai linear no shader sem voce pedir, e por isso que a superficie srgb encoda
sozinha na escrita. o formato e que carrega a conta. entender que o formato e um par
de conversoes, e nao um rotulo de cor, fecha quase toda a confusao que sobra.

## o pino: um teste que mede o pixel, nao olha a tela

a parte que transforma tudo isso de "decisao boa" em "decisao que nao volta a
quebrar" e o teste de regressao. ele esta em `crates/engine/src/color.rs:185`,
chamado `to_linear_array_darkens_srgb_midtones`:

```rust
#[test]
fn to_linear_array_darkens_srgb_midtones() {
    // #303030 (the page bg) is 0.188 sRGB; linear is ~0.029. Feeding the
    // raw sRGB value to an sRGB surface would re-encode it to ~0.46 (the
    // washed-out bug); the linear value round-trips back to ~0.188 on write.
    let g = Color::hex(0x303030).to_linear_array();
    assert!((g[0] - 0.0296).abs() < 0.001, "got {}", g[0]);
    assert_eq!(g[0], g[1]);
    assert_eq!(g[3], 1.0, "alpha must stay linear");
    // Pure black/white are fixed points of the transfer function.
    assert_eq!(Color::hex(0x000000).to_linear_array()[0], 0.0);
    assert!((Color::hex(0xFFFFFF).to_linear_array()[0] - 1.0).abs() < 1e-6);
}
```

esse teste e pequeno e e a coisa mais importante do arquivo. ele crava o ponto exato
da curva no valor exato do bug original. `#303030` tem que dar 0.0296 em linear, com
tolerancia de 0.001. se alguem um dia mexer na funcao de transferencia, trocar a
gamma, cortar o galho da reta, esse numero muda e o teste reprova antes de chegar na
tela de alguem. ele tambem fixa os dois pontos que nao podem se mover: preto puro vira
0.0 e branco puro vira 1.0, os extremos sao fixos da transformacao. e cuida do alpha,
de novo, com a mensagem "alpha must stay linear".

o comentario do teste e quase um resumo do capitulo: o `#303030` e 0.188 em srgb, em
linear e ~0.029, entregar o srgb cru pra uma superficie srgb re-encodaria pra ~0.46,
que e o bug lavado, e o valor linear faz o round-trip de volta pra ~0.188 na escrita.
e essa frase, "round-trip de volta", que diz por que a conta toda existe: voce tira
da curva no CPU exatamente pra GPU poder botar de volta na curva na escrita e o numero
fechar no mesmo lugar que entrou.

vale insistir num metodo, nao so num teste. a validacao da decisao foi pixel medido,
118 antes e 50 depois contra token de 48, e nao "ficou melhor a olho". o olho nao
serve aqui porque o erro e justo do tamanho que o olho aceita como "deve ser assim".
um cinza 2.5 vezes mais claro ainda parece um cinza plausivel. e foi por isso que o
bug sobreviveu meses e foi confundido com escolha de token. a regua no pixel e o que
separa "consertei" de "achei que consertei". o adr poe isso como instrucao direta:
antes de "consertar" cor lavada, meca o pixel; se um token de 48 renderiza 118, o bug
esta na transferencia, nao no token.

## a regra, e por que ela e tao facil de furar em silencio

`Color` tem dois metodos que devolvem `[f32; 4]` e parecem intercambiaveis. `to_array`
devolve os quatro canais como estao, em srgb. `to_linear_array` linearzia os tres de
cor. a assinatura e quase igual, o nome difere por uma palavra, e o compilador aceita
os dois em qualquer lugar que espera um `[f32; 4]`. usar `to_array` num valor de clear
compila, roda, e reintroduz o bug original calado. o adr marca isso com todas as
letras: usar `to_array` pra um clear value traz o bug de volta em silencio.

e por isso que o comentario de doc da funcao, em `crates/engine/src/color.rs:61`,
gasta seis linhas explicando quando aplicar, em vez de so descrever o que ela faz:

```rust
/// The window surface is an sRGB format: the GPU encodes linear->sRGB on
/// write. Values handed to the GPU must therefore be linear, or an `#303030`
/// fill (0.188 sRGB) would be treated as linear and re-encoded to ~0.46
/// (#767676) - washing every surface out and crushing contrast. Apply this
/// when feeding a color into the clear value or any CPU-built uniform.
/// (Vertex colors are linearized in-shader via `srgb_to_linear`.)
```

o comentario faz o trabalho que o tipo nao consegue fazer sozinho. ele diz a regra
inteira: aplica em clear value e em qualquer uniform montado no CPU, e lembra que cor
de vertice ja e linearizada no shader, pra ninguem linearizar duas vezes a mesma cor
por nao saber qual porta ela usa. um sistema de tipos que distinguisse `SrgbColor` de
`LinearColor` pegaria isso em compile time. o engine ainda nao tem essa distincao no
tipo, entao a fronteira mora no nome do metodo, no comentario, e no teste de
regressao. e uma fronteira de disciplina apoiada por um pino medido, nao uma fronteira
de tipo. registrar isso aqui e honesto: a protecao e boa, e nao e a prova de
compilador que ownership da no `GpuVec`.

se eu fosse condensar o capitulo numa lista pra colar do lado do monitor, seria esta.
cor entra srgb, sempre, e o que voce escreve no token. linear acontece uma vez, na
entrada da GPU, e em lugar nenhum antes. clear e uniform de CPU passam por
`to_linear_array`. cor de vertice passa por `srgb_to_linear` no shader. textura
decodificada nao passa por nada, ela ja esta linear no sample. a superficie e srgb por
formato no desktop e por view-format na web, e e ela que re-encoda na escrita pra o
numero voltar pro lugar. e a view da superficie sai de `surface_render_view`, nunca de
`create_view(&Default::default())`. cada item dessa lista existe porque a falta dele
ja apareceu medido como um cinza errado na tela.

a conta nao e elegante por gosto. ela e o minimo de trabalho que faz 48 entrar e 48
sair, em duas plataformas que decodam cor de jeitos opostos, com uma unica
linearizacao no caminho. quando o fundo mediu 50 contra um token de 48, no desktop, e
(48,48,48) na web onde antes era (8,8,8), nao foi uma cor que ficou bonita. foi a
mesma cor saindo igual nos dois lugares, que e a unica coisa que esse engine promete
quando diz "rendering identico em toda plataforma".

## rastros

mapeamento de cada afirmacao pro arquivo e linha que a sustenta. o que nao consegui
confirmar esta marcado.

- `Color` e quatro `f32` RGBA em 0.0..=1.0, com `to_array` (srgb cru) e
  `to_linear_array` (linear): `crates/engine/src/color.rs:1`, `:57`, `:70`
- a funcao de transferencia srgb para linear, com o galho em 0.04045, a reta
  `/ 12.92` e a potencia `2.4`, e o alpha intocado: `crates/engine/src/color.rs:70`
- o comentario de doc com a regra de quando aplicar (clear e uniform de CPU; cor de
  vertice no shader) e o exemplo do `#303030` re-encodado pra ~0.46 / `#767676`:
  `crates/engine/src/color.rs:61`
- o teste de regressao que crava `#303030` em 0.0296, fixa preto em 0.0 e branco em
  1.0, e exige alpha linear: `crates/engine/src/color.rs:185`
- a outra linearizacao, perceptual, pro OKLCH (srgb -> linear -> LMS -> OKLab), com
  sua propria `srgb_to_linear` no CPU: `crates/engine/src/theme/color_space.rs:22`,
  `:40`
- a configuracao da superficie com `add_srgb_suffix()` e `view_formats`, identidade no
  desktop e variante srgb na web: `crates/engine/src/gpu/surface.rs:73`
- `surface_format()` devolvendo o formato de view pra manter pipelines, texturas de
  camada e a view da superficie consistentes: `crates/engine/src/gpu/surface.rs:122`
- `surface_render_view()` forcando `format: Some(self.surface_format())`, o unico
  caminho sancionado pra view de superficie: `crates/engine/src/gpu/surface.rs:134`
- o clear color da janela montado a partir de `to_linear_array`, com o comentario de
  que valores de clear do wgpu sao lineares e a superficie srgb re-encoda na escrita:
  `crates/engine/src/window/render.rs:145`
- a `srgb_to_linear` em WGSL, identica em forma a do CPU, e a ordem linearizar-antes-
  de-premultiply no fragment: `crates/engine/src/gpu/shaders/quad.wgsl:29`
- uso real em app: `crates/showcase/src/renderer.rs:73`, `crates/ide/src/renderer.rs:84`
- decisao, o bug de dois encodes no desktop, a medicao 118 antes / 50 depois contra
  token 48, a recusa de escurecer token como band-aid, e o que nao converter duas
  vezes (atlas, backdrop, composite, blur): `kdb/adr/linearize-colors-before-the-gpu.md`
- decisao do alvo srgb, o inverso na web com (8,8,8) antes / (48,48,48) depois, os
  sete call sites migrados pra `surface_render_view`, e o aviso contra
  `create_view(&Default::default())`: `kdb/adr/render-into-an-srgb-view-format.md`
- versoes conferidas no `Cargo.toml` da raiz: wgpu 28, winit 0.30, cosmic-text 0.18,
  taffy 0.9
- nao confirmado: o numero "~2.5x" e descrito nos comentarios e no adr como a razao
  aproximada do midtone (118/48 da ~2.46; 0.46/0.188 da ~2.45), coerente com as
  medicoes, mas o fator nao tem um benchmark dedicado, so as medicoes pontuais de
  pixel. o sumario marca este capitulo como bench n/a, entao nao ha numero de
  benchmark, so as medicoes de pixel dos dois adrs
