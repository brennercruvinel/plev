---
title: ownership e borrow pelo GpuVec e pelos buffers persistentes
parte: 1
status: rascunho
rastros:
  - crates/engine/src/gpu/vec.rs
  - crates/engine/src/compositor/layer/mod.rs
  - crates/engine/src/compositor/layer/geometry.rs
  - crates/engine/src/compositor/memory.rs
  - crates/engine/src/window/render_passes.rs
  - crates/engine/src/compositor/mod.rs
  - kdb/adr/render-on-demand-requires-explicit-invalidation.md
  - crates/engine/benches/scene_build.rs
---

# ownership e borrow pelo GpuVec e pelos buffers persistentes

todo tutorial de rust te ensina ownership com uma `String`. voce cria, voce move,
voce empresta, o compilador reclama, voce entende. funciona como primeira mordida,
mas e um exemplo sem consequencia. a `String` mora na heap do seu processo, e se
voce errar o borrow o pior que acontece e o programa nao compilar. nada se quebra
no mundo. eu quero contar a mesma historia com uma peca onde o erro nao seria so
um warning vermelho: um pedaco de memoria que nao esta nem na heap nem na stack do
seu programa, que vive na placa de video, que sobrevive de um frame pro outro, e
que a GPU pode estar lendo no exato instante em que voce pensa em sobrescrever.

essa peca, no plev, e o `GpuVec`. e o capitulo abre nele porque ele e o lugar onde
ownership, borrow e lifetime param de ser regra de prova de compilador e viram
regra de correcao visual. se voce emprestar esse buffer errado, ou esquecer de
avisar que ele mudou, a tela mente. mostra o frame de ontem como se fosse o de
agora. e o compilador, na maior parte do caminho, te impede de chegar la. esse e o
trato que eu quero desmontar: como o borrow checker, uma ferramenta que parece
existir so pra evitar segfault em CPU, acaba ancorando a invariante mais importante
do render on demand.

vou abrir no caderno em cima da mesa, descer ate as quatro linhas do `struct`
real, e ir ate o ponto onde o `&mut` do upload encosta no `&` do render pass e o
compilador decide quem espera quem.

## o caderno que voce nao arranca

pensa num caderno na mesa, daqueles de espiral. voce anota a lista de compras numa
pagina. amanha a lista e quase a mesma, muda um item. voce tem duas escolhas. pode
arrancar a folha, pegar uma em branco e reescrever tudo do zero, todo dia. ou pode
manter a folha, apagar so o que mudou e escrever por cima.

quase todo software grafico ingenuo arranca a folha. todo frame ele aloca buffer
novo, joga os vertices, manda pra GPU, descarta, e no frame seguinte faz tudo de
novo. funciona. a GPU moderna engole isso sem reclamar muito. mas e desperdicio, e
desperdicio tem dois custos que ninguem ve no comeco: a alocacao e o liberacao
repetidos pressionam o alocador da placa, e o trafego CPU para GPU enche um
barramento que voce vai querer livre quando a cena ficar pesada de verdade.

o plev mantem a folha. cada buffer de geometria de cada layer e alocado uma vez,
cresce quando precisa, nunca encolhe, e e reescrito por cima quando a cena muda. o
nome dessa folha no codigo e `GpuVec`. e a primeira coisa que importa entender e
que ele nao guarda os dados: ele guarda a posse de um pedaco de VRAM e um punhado
de metadados sobre ele. a folha, nao o que esta escrito nela.

## o codigo: quatro campos e a posse de um buffer

o `GpuVec` inteiro cabe na cabeca. ele vive em `crates/engine/src/gpu/vec.rs`, e o
struct e isto:

```rust
pub struct GpuVec {
    buffer: wgpu::Buffer,
    capacity: u64,
    usage: wgpu::BufferUsages,
    label: &'static str,
}
```

quatro campos. o primeiro, `buffer`, e onde mora a posse. um `wgpu::Buffer` e um
handle dono de um recurso de GPU: por baixo dele tem memoria alocada na placa (ou
no que o backend usar pra emular, no caso do wasm). a regra de ownership do rust se
aplica a esse handle do mesmo jeito que se aplicaria a uma `String`, com uma
diferenca de peso: quando o `GpuVec` for dropado, o `Drop` do `wgpu::Buffer` libera
a memoria de GPU. a posse aqui nao e uma metafora de stack, e a vida de um recurso
fora do seu processo amarrada a vida de um valor dentro dele. e isso que o rust faz
de melhor e que C te deixa errar: o recurso externo morre exatamente quando o dono
sai de escopo, sem free manual, sem double free, sem esquecer.

os outros tres campos sao contabilidade. `capacity` e quantos bytes a placa
reservou, em `u64`. `usage` guarda pra que serve esse buffer (vertice, indice, e
sempre com `COPY_DST` ligado, ja volto nisso) pra poder recriar com a mesma flag
quando crescer. e `label` e uma string de debug que aparece nas ferramentas de
profiling de GPU.

o `label` merece uma parada, porque ele e a primeira aula de lifetime do capitulo,
escondida num detalhe que e facil ler rapido demais: o tipo dele e `&'static str`,
nao `String`. isso e uma decisao, nao um acaso. `&'static str` e uma referencia
emprestada com o maior lifetime que existe, o programa inteiro. na pratica significa
que esse campo so aceita literais de string, aqueles `"layer_quad_vb"` que vivem no
binario desde a compilacao e nunca sao liberados. o `GpuVec` empresta esse texto sem
nunca possuir uma copia dele. nao tem alocacao de `String`, nao tem clone, nao tem
free. e um borrow que dura tanto quanto o processo, entao guardar dentro de um
struct de vida arbitraria nao gera nenhum conflito de lifetime. o compilador aceita
de boa porque `'static` sobrevive a qualquer coisa. e a forma mais barata de carregar
um nome: zero bytes de heap, so um ponteiro pra dentro do proprio executavel.

## a construcao: alocar uma vez, com COPY_DST

o construtor e direto, mas tem uma flag que e o eixo de tudo que vem depois:

```rust
pub fn new(
    device: &wgpu::Device,
    label: &'static str,
    usage: wgpu::BufferUsages,
    initial_cap: u64,
) -> Self {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: initial_cap,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    Self {
        buffer,
        capacity: initial_cap,
        usage,
        label,
    }
}
```

repara no `usage | wgpu::BufferUsages::COPY_DST`. o chamador diz "isso e um buffer
de vertice" passando `wgpu::BufferUsages::VERTEX`, e o `GpuVec` adiciona `COPY_DST`
por conta propria, sempre. `COPY_DST` quer dizer "esse buffer pode ser destino de
uma copia". sem essa flag a placa rejeita qualquer escrita posterior. e como o
projeto inteiro do `GpuVec` e ser escrito por cima frame apos frame, sem ela a peca
nao faria sentido. a flag e o jeito de declarar pra GPU, na hora da alocacao, "vou
reescrever isso aqui muitas vezes". guardar tambem o `usage` original, sem o
`COPY_DST`, e o que permite recriar o buffer com a mesma intencao quando ele
crescer, sem precisar lembrar a flag de novo.

`device.create_buffer` recebe `&wgpu::Device` por referencia compartilhada, um `&`.
o `device` nao e movido pra dentro do `GpuVec`, ele e so emprestado pelo tempo da
chamada. isso e proposital e e o padrao do crate inteiro: o `Device` e a `Queue`
sao recursos globais, um por aplicacao, emprestados pra quem precisar deles na hora
que precisar. nenhum `GpuVec` possui o device. centenas de `GpuVec` compartilham o
mesmo `&Device` em chamadas separadas, nunca ao mesmo tempo de forma exclusiva. e
exatamente o caso de uso do borrow compartilhado: muitos leitores, nenhum dono novo.

`mapped_at_creation: false` significa que a memoria nao nasce mapeada na CPU. a
gente nao vai escrever direto no ponteiro, vai usar `queue.write_buffer`, que e o
caminho de upload que o `COPY_DST` habilita. detalhe pequeno que fecha a coerencia
da flag.

## o upload: borrow compartilhado entra, bytes saem

a unica forma de botar dado nesse buffer e o `upload`:

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

tem ownership e borrow em quase toda linha dessa funcao, vale ler devagar.

a assinatura comeca com `&mut self`. escrever no `GpuVec` exige posse exclusiva dele
naquele instante. nao da pra ter duas escritas concorrentes, nem da pra escrever
enquanto outra parte do codigo le. o compilador garante isso de graca: `&mut self`
e, por definicao, a unica referencia viva pra aquele valor enquanto dura. essa
unica palavra, `mut`, e metade do capitulo. ela e a razao de o upload nao poder
acontecer no meio de um render pass, e eu chego la.

o parametro `data: &[T]` e um slice emprestado. o `upload` nao toma posse dos seus
vertices. ele le um pedaco de memoria que continua sendo do chamador, copia os
bytes pra GPU, e devolve o controle. o `Vec<TextVertex>` que mora na layer continua
inteiro e dono de si depois do upload. isso importa porque o mesmo `Vec` vai ser
relido no proximo frame pra recalcular o hash de dirty tracking. se o upload tomasse
posse, a layer perderia os dados que ela precisa manter pra comparar. borrow, nao
move, e o que mantem a fonte viva dos dois lados.

o bound `T: bytemuck::Pod` e o contrato que torna a proxima linha segura. `Pod`
quer dizer "plain old data", um tipo que e so bytes, sem ponteiro escondido, sem
padding indefinido, sem `Drop` que faca algo. `bytemuck::cast_slice(data)` pega o
`&[T]` e reinterpreta como `&[u8]`, sem copiar, sem alocar. e uma operacao que
seria `unsafe` na mao, transmutar um slice de um tipo pra slice de byte, e o
`bytemuck` a embrulha numa API segura cobrando o preco no sistema de tipos: so
compila se `T` provar que e `Pod`. de novo um borrow, `cast_slice` empresta `data`
e devolve outra vista emprestada dos mesmos bytes, ninguem vira dono de nada novo.
a versao do bytemuck no projeto e a 1 com a feature `derive` ligada, conferida no
`Cargo.toml`, o que deixa os vertices do engine derivarem `Pod` com um
`#[derive(Pod)]` em vez de implementar na unha.

so na ultima linha o dado de fato sai do seu processo. `queue.write_buffer(&self.buffer, 0, bytes)`
agenda uma copia dos `bytes` pro inicio (`offset 0`) do buffer. repara no `&self.buffer`:
o upload empresta o proprio buffer por `&` compartilhado pra entregar pro wgpu, mesmo
estando dentro de um metodo `&mut self`. nao tem conflito ai porque e tudo uma
sequencia, uma coisa de cada vez, o mesmo thread. o `&mut self` garante que ninguem
de fora esta olhando esse `GpuVec` enquanto a funcao roda.

um ponto honesto sobre o que esse `write_buffer` faz e o que ele nao faz. ele nao
escreve na hora, no bare metal. ele agenda a escrita na fila da GPU, o wgpu cuida da
sincronizacao do lado da placa pra que a copia aconteca em ordem em relacao aos
draws que usam o buffer. ou seja, a corrida de dados no relogio da GPU quem evita e
o wgpu, nao o borrow checker. o borrow checker resolve um problema diferente, do
lado da CPU, e e ai que mora a parte bonita.

## a folha cresce, nunca encolhe

antes de chegar no borrow, falta a peca que torna o borrow perigoso de verdade:
`ensure_capacity`.

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

a logica e a de qualquer vetor que dobra. se o que voce precisa cabe na capacidade
atual, nao faz nada. se nao cabe, calcula uma capacidade nova: o dobro do atual, ou
o exato que voce pediu, o que for maior. e aloca um buffer novo desse tamanho. o
crescimento amortizado e o mesmo do `Vec` da std, voce paga a realocacao de vez em
quando e na media insere em tempo constante. o comentario no topo do arquivo resume
a politica em tres palavras: cresce, nunca encolhe, escritas parciais.

"nunca encolhe" e uma escolha, nao uma limitacao. se uma layer teve um pico de mil
retangulos num frame e caiu pra dez no seguinte, o buffer continua do tamanho do
pico. a aposta e que o pico volta, e realocar pra baixo so pra realocar pra cima de
novo e churn puro. o preco dessa aposta e memoria de GPU presa, e o engine sabe
disso: tem uma funcao so pra contar quanto.

agora a linha que e o centro de gravidade do capitulo inteiro:

```rust
self.buffer = device.create_buffer(/* ... */);
```

essa atribuicao substitui o buffer. o velho `wgpu::Buffer`, o que estava em
`self.buffer` ate aqui, e dropado nesse exato ponto. o `Drop` dele roda, a memoria
de GPU antiga e marcada pra liberacao. apos essa linha, qualquer referencia que
existisse apontando pro buffer velho estaria apontando pra um recurso morto.

segura essa imagem. `ensure_capacity` pode, a qualquer upload, trocar o buffer por
baixo e matar o anterior. e isso que transforma o borrow de "boa pratica" em
"questao de vida ou morte de um ponteiro".

## o borrow que sai pra desenhar

do outro lado da peca tem o `buffer()`, o jeito de pegar o handle pra desenhar:

```rust
pub fn buffer(&self) -> &wgpu::Buffer {
    &self.buffer
}
```

`&self` entra, `&wgpu::Buffer` sai. um borrow compartilhado, somente leitura, com o
lifetime amarrado ao `&self`: a referencia devolvida nao pode viver mais que o
`GpuVec` de onde saiu. isso e o elytron do lifetime trabalhando sem voce escrever
nada, o compilador entende que `&wgpu::Buffer` empresta de `self` e nao deixa o
emprestimo sobreviver ao emprestado.

quem chama esse `buffer()` la em cima e o caminho de render. a layer expoe os
buffers de cada tipo de geometria assim, em `crates/engine/src/compositor/layer/mod.rs`:

```rust
pub fn quad_buffers(&self) -> Option<(&wgpu::Buffer, &wgpu::Buffer, u32)> {
    let vb = self.quad_vb.as_ref()?;
    let ib = self.quad_ib.as_ref()?;
    if self.quad_index_count == 0 {
        return None;
    }
    Some((vb.buffer(), ib.buffer(), self.quad_index_count))
}
```

`quad_buffers` toma `&self`, um borrow compartilhado da layer inteira, e devolve
referencias compartilhadas pros dois buffers (vertice e indice) mais a contagem de
indices. note o `Option`: se a layer nunca alocou o buffer (`quad_vb` e
`Option<GpuVec>` e ainda esta `None`) ou se nao tem nada pra desenhar
(`quad_index_count == 0`), ele devolve `None` e o render pula esse tipo de
geometria. a posse opcional aqui nao e detalhe, e o que deixa uma layer headless,
sem device, existir sem nenhum buffer alocado.

e no render pass, em `crates/engine/src/window/render_passes.rs`, as referencias
viram comandos de desenho:

```rust
pass.set_vertex_buffer(0, vb.slice(..));
pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
```

`vb.slice(..)` cria uma `BufferSlice` que empresta o buffer pelo tempo que o
`pass` durar. enquanto esse render pass esta sendo gravado, ele segura uma
referencia viva pro buffer. e ai esta a colisao que o capitulo vinha montando.

## onde o &mut encosta no &

junta as duas pontas. de um lado, `upload` precisa de `&mut self` no `GpuVec`, e por
dentro pode chamar `ensure_capacity`, que substitui e dropa o buffer. do outro lado,
o render pass segura um `&wgpu::Buffer` emprestado desse mesmo `GpuVec` pelo tempo
inteiro do pass.

o que o borrow checker faz com isso e simples e absoluto: voce nao pode ter um
`&mut` e um `&` pro mesmo valor vivos ao mesmo tempo. se o render pass esta segurando
a referencia pro buffer, voce nao consegue nem chamar `upload`, porque `upload` pede
exclusividade que a referencia compartilhada esta negando. o codigo abaixo, escrito
de proposito pra falhar, e a forma minima do erro:

```rust
// exemplo pedagogico: isto NAO compila
let mut vb = GpuVec::new(&device, "vb", wgpu::BufferUsages::VERTEX, 4096);
vb.upload(&device, &queue, &vertices);   // ok: &mut vb, comeca e termina aqui

let buf: &wgpu::Buffer = vb.buffer();    // borrow compartilhado de vb comeca aqui
vb.upload(&device, &queue, &novos);      // ERRO[E0502]: &mut vb com buf ainda vivo
pass.set_vertex_buffer(0, buf.slice(..)); // buf usado aqui, mantem o borrow vivo
```

o compilador para no segundo `upload` com o classico `cannot borrow vb as mutable
because it is also borrowed as immutable`. e ele esta certo de um jeito que vai
muito alem da burocracia. lembra que `upload` pode chamar `ensure_capacity`, que
pode fazer `self.buffer = device.create_buffer(...)` e dropar o buffer velho. se o
compilador deixasse essa linha passar, o `buf` que o render pass usa duas linhas
abaixo apontaria pra um `wgpu::Buffer` ja dropado. seria um use after free de um
recurso de GPU, a classe de bug que assombra C e C++ no codigo grafico ha decadas,
e que aqui simplesmente nao tem como ser escrita.

e isso que eu quis dizer la no comeco com "o compilador ancora a invariante". o
borrow checker nao sabe nada sobre GPU, sobre frame, sobre VRAM. ele so sabe que um
`&mut` e um `&` pro mesmo dado nao convivem. mas como o `GpuVec` foi modelado pra que
a unica escrita seja `&mut self` e a unica leitura pro desenho seja `&`, essa regra
generica de aliasing vira, sem custo extra, a garantia de que voce nunca sobrescreve
um buffer que a GPU esta lendo no mesmo trecho de codigo. a "escrita concorrente" que
o checker captura nao e uma race de threads, e a tentativa de mutar a folha enquanto
alguem ainda esta lendo a folha. e ele captura no tempo de compilacao, antes de o
programa existir.

e e por isso que o engine separa as duas coisas no tempo de proposito. em
`Compositor::resolve`, primeiro vem a fase de `&mut`, os uploads:

```rust
for layer in &mut self.layers {
    if layer.dirty {
        layer.upload_quad_geometry(res.device, res.queue);
        layer.upload_sdf_geometry(res.device, res.queue);
        layer.upload_shadow_geometry(res.device, res.queue);
        layer.upload_image_geometry(res.device, res.queue);
        layer.upload_backdrop_geometry(res.device, res.queue);
        // ...
    }
}
```

esse loop pega `&mut self.layers` e escreve nos buffers. so depois que ele termina,
quando esse `&mut` morre, e que o render pass entra com seus borrows compartilhados
pra ler. a ordem upload primeiro, desenho depois nao e mantida por disciplina ou
comentario, ela e forcada pela assinatura: voce nao consegue intercalar as duas
fases nem que tente, porque o `&mut` de uma e o `&` da outra se recusam a coexistir.
o design transformou uma regra de correcao temporal numa regra de tipo.

## o que isso tem a ver com nao desenhar

aqui o capitulo cruza com o assunto que parece de outro departamento: invalidacao. e
o cruzamento e a parte que eu mais demorei pra ver inteira.

o `GpuVec` e persistente. ele sobrevive entre frames. as layers guardam os buffers
como `Option<GpuVec>`, e uma vez alocados eles ficam la, com a geometria do ultimo
frame, esperando. a alocacao acontece preguicosa, com `get_or_insert_with`, em
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

`get_or_insert_with` cria o `GpuVec` na primeira vez e reusa nas seguintes. o
`INITIAL_VB_SIZE` e 4096 bytes e o `INITIAL_IB_SIZE` e 2048, os dois conferidos no
`layer/mod.rs`. dali pra frente o mesmo buffer e reescrito por cima a cada upload, e
so cresce se a geometria passar do que cabe. a folha que voce nao arranca.

agora repara em quem chama esse upload. la no `resolve`, o loop estava todo dentro de
um `if layer.dirty`. se a layer nao esta suja, nenhum `upload_*` roda. e ai esta o
fio que conecta tudo: se ninguem reescreve o buffer, ele continua exatamente com a
geometria do frame passado. e quando o render pass for desenhar, ele desenha o que
estiver no buffer. o buffer persistente nao tem memoria de "isso e velho". pra ele,
o conteudo de ontem e tao valido quanto o de hoje. o default, na ausencia de qualquer
sinal, e redesenhar o frame antigo.

e por isso que o render on demand precisa de invalidacao explicita, e nao por
elegancia. o adr que registra essa decisao,
`kdb/adr/render-on-demand-requires-explicit-invalidation.md`, conta o bug que pagou
por essa regra. o engine so renderiza quando alguem pede um frame, disciplina de
bateria e CPU copiada de editor de producao. sob esse modelo, um handler de evento
que muda o estado visual mas falha em sinalizar nao produz um frame velho na tela:
produz um aplicativo congelado. e foi exatamente o que aconteceu. eventos de scroll
foram consumidos, o estado mudou, os handlers retornaram `false`, nada invalidou, e o
showcase parecia completamente travado. o defeito foi lido primeiro como "scroll nao
implementado", quando era invalidacao faltando.

o adr cristaliza a consequencia numa frase que vale colar na parede: returning false
is a statement that nothing visible changed, and the scheduler believes it. retornar
`false` e uma afirmacao de que nada visivel mudou, e o escalonador acredita. o buffer
persistente e o motivo de o escalonador poder acreditar com seguranca: ele sabe que o
conteudo de ontem ainda esta la, intacto, pronto pra redesenhar. a invalidacao e o
unico jeito de dizer "nao, dessa vez tem coisa nova, pegue o caminho do `&mut` e
reescreva". o adr fecha o contrato: uma feature de interacao nova nao esta pronta
quando o estado muda, esta pronta quando a mudanca de estado provadamente agenda um
frame. os testes de regressao verificam o booleano e a invalidacao, nao so o estado
mutado.

junta as duas metades do capitulo e a forma fica inteira. o ownership unico do
`GpuVec` sobre o buffer, mais o borrow compartilhado que sai pra desenhar, mais o
`&mut` exclusivo que e o unico caminho de escrita, formam um sistema onde a ausencia
de upload e o estado de repouso normal, nao um erro. e quando o estado de repouso e
"redesenhe o que ja esta no buffer", a correcao passa a depender de um sinal explicito
que diga quando sair do repouso. o borrow checker garante que voce nunca escreve no
buffer errado na hora errada. a invalidacao garante que voce escreve no buffer certo
na hora certa. uma e mecanica de compilador, a outra e contrato de arquitetura, e as
duas guardam a mesma folha por angulos diferentes.

## contar a memoria que voce nao devolve

falei que "nunca encolhe" tem um preco e que o engine sabe medir esse preco. a peca
que mede e o `capacity_bytes`, o ultimo metodo do `GpuVec`:

```rust
/// Allocated GPU bytes (capacity, not the live byte count: the buffer
/// grows and never shrinks). Feeds the perf monitor's memory stats.
pub fn capacity_bytes(&self) -> u64 {
    self.capacity
}
```

o doc comment ja avisa a sutileza: isso e a capacidade alocada, nao a contagem de
bytes vivos. se o buffer cresceu pra caber mil retangulos e a cena caiu pra dez, esse
numero ainda reporta o tamanho de mil. e o que voce quer pra um monitor de memoria,
voce quer saber quanto de VRAM esta preso, nao quanto esta logicamente em uso. o
`compositor/memory.rs` soma esses numeros por todos os buffers de todas as layers
pra estimar a memoria residente de GPU:

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

o `map_or(0, GpuVec::capacity_bytes)` e onde o `Option<GpuVec>` paga: buffer nao
alocado conta zero, buffer alocado conta a capacidade. e o estilo aqui e o do crate
inteiro, o engine nao pergunta nada pro driver. ele estima a memoria a partir do que
ele mesmo controla, as capacidades dos buffers e as dimensoes das texturas. tem um
teste que crava isso: um `Compositor::new()` headless, sem device tocado, reporta
zero bytes de GPU, porque nenhum buffer foi criado. essa propriedade vai ser
importante quando eu falar do benchmark, ja chego la.

repara que `capacity_bytes` toma `&self`, mais um leitor compartilhado. somar a
memoria nao mexe em nada, entao nao precisa de `&mut`. de novo o sistema de tipos
documentando a intencao: voce le a contabilidade da folha sem direito de escrever
nela. o tipo da assinatura ja te conta o que a funcao pode e nao pode fazer, antes
de voce ler uma linha do corpo.

## o numero, e a honestidade sobre o que ele mede

eu nao gosto de fechar um capitulo de performance sem um numero, mas dessa vez o
numero exige uma ressalva que e mais interessante que o numero em si.

a ancora de benchmark do capitulo e `crates/engine/benches/scene_build.rs`. ela tem
seis grupos: `push_rects`, `push_paths`, `dirty_tracking`, `tessellation`,
`signals`, `text_hashing`. o que importa pro `GpuVec` e olhar o que esses
benchmarks de fato exercitam, e a resposta honesta e: nenhum deles toca o `GpuVec`
direto. todos constroem um `Compositor::new()`, que e headless. e ja vimos, pelo
teste de memoria, que compositor headless nao aloca buffer nenhum. ou seja, o
`scene_build.rs` mede o lado CPU que alimenta o `GpuVec`, a construcao dos
`Vec<vertice>` e `Vec<indice>` que mais tarde, num app com device de verdade, seriam
passados pro `upload`. ele nao mede o `queue.write_buffer`, nem o `create_buffer`, nem
o crescimento do buffer.

isso nao e um defeito do benchmark, e a fronteira certa pra medir num bench de CPU,
que precisa rodar sem GPU pra ser reproduzivel em CI. mas seria desonesto eu te
vender um "tempo de upload do GpuVec" tirado dali, porque esse tempo nao esta sendo
medido. marco explicitamente como nao confirmado: nao existe, na ancora dada, um
numero que isole o custo do `upload`, do `ensure_capacity` ou do `write_buffer`. o
que da pra dizer com fundamento e o formato do trabalho que precede o upload. o
grupo `push_rects`, por exemplo, mede empurrar 100, 1000 e 10000 retangulos pra
dentro do compositor:

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

esse e o trabalho que constroi a cena que, depois de virar geometria, seria
escrita no `GpuVec`. medir isso separado do upload e coerente com a arquitetura, que
separa a fase CPU (descrever a cena, calcular dirty, montar vertices) da fase GPU
(upload mais render pass) justamente porque o borrow checker forca essa separacao,
como vimos. o benchmark mede o lado de ca da fronteira. o lado de la, o
`queue.write_buffer` real, depende de driver e de placa e nao cabe num bench de CPU
deterministico. se um dia eu quiser o numero do upload, ele vai ter que sair de um
profile com device real, nao do `scene_build.rs`. ate la, o numero honesto e: o
custo de upload no `GpuVec` nao esta medido em nenhuma ancora deste capitulo.

## por que assim, e nao de outro jeito

da pra perguntar se toda essa cerimonia de ownership valia a pena pra um buffer.
podia ser um `wgpu::Buffer` solto, recriado quando desse na telha, com a sincronizacao
na mao. da pra fazer. C faz, e paga em bug de use after free e em double free e em
buffer escrito enquanto e lido. o ganho do `GpuVec` nao e ter inventado o conceito de
buffer que cresce, isso e um `Vec` com outra memoria. o ganho e ter desenhado a API de
forma que o sistema de tipos do rust carregue, de graca, as invariantes que em outra
linguagem seriam comentario e reza.

a posse unica do buffer dentro do `GpuVec` significa que a memoria de GPU tem um dono
claro e e liberada exatamente quando esse dono morre. a escrita so por `&mut self`
significa que mutacao e exclusiva por construcao, sem precisar de lock, porque o
unico thread que mexe na cena ja e serializado pelo borrow. a leitura pro desenho so
por `&` significa que cem draws podem referenciar o buffer ao mesmo tempo, sem copia,
contanto que ninguem esteja escrevendo. e o `ensure_capacity` poder trocar o buffer
por baixo significa que segurar uma referencia velha atravessando um upload e um erro
de compilacao, nao um crash em producao. cada uma dessas e uma decisao de design que
escolheu deixar o compilador trabalhar em vez de confiar no programador.

e a fronteira entre essa mecanica e a invalidacao explicita e o que eu acho mais
bonito de toda a peca. o borrow checker resolve "nao escreva enquanto le". ele nao
resolve, e nem teria como, "lembre de escrever quando algo mudou". esse e um problema
de fora do programa, mora na intencao do usuario que clicou, arrastou, animou. nenhuma
analise estatica ve isso. entao o engine corta o problema em dois pedacos que cabem em
ferramentas diferentes: a metade que o compilador ve, ele entrega pro compilador, via
ownership e borrow do `GpuVec`. a metade que o compilador nao ve, ele entrega pra um
contrato humano testavel, a invalidacao do adr, com teste de regressao em cima do
booleano. menos coisa pra confiar no programador, mais coisa provada pela maquina, e o
resto isolado num lugar so onde da pra olhar com lupa.

## o que isso me ensinou

a licao que eu levei daqui nao foi sobre GPU. foi sobre o que o ownership do rust
serve de verdade quando para de ser exercicio de `String`. nos exemplos de tutorial,
o borrow checker parece um porteiro chato que te faz provar coisas obvias. no
`GpuVec` ele e a unica razao de eu poder dormir tranquilo sabendo que nenhum frame vai
desenhar a partir de um buffer que foi liberado no meio do caminho. a mesma regra,
`&mut` e `&` nao convivem, que num exemplo de livro impede um bug que nao machucaria
ninguem, aqui impede um use after free de VRAM que num app de producao apareceria como
glitch intermitente impossivel de reproduzir.

e a segunda licao, a que demorou mais: o sistema de tipos cobre o que ele consegue
ver, e a engenharia boa e em boa parte saber onde ele para de ver e botar um contrato
explicito exatamente nessa borda. o `GpuVec` deixa o compilador garantir a seguranca
da escrita. a invalidacao garante a presenca da escrita. nenhuma das duas sozinha
fecha o render on demand. juntas, fecham. e a folha continua na mesa, com o que foi
escrito ontem, esperando alguem avisar que mudou.

se eu fosse deixar uma frase disso pra alguem que esta aprendendo rust com a `String`
e achando teoria demais: o borrow checker nao esta te impedindo de fazer o que voce
quer, ele esta te impedindo de fazer o que voce nao percebeu que estava prestes a
fazer. no `GpuVec`, o que voce nao percebeu e que ia desenhar a partir de um buffer
morto. ele percebeu por voce, na compilacao, de graca.

## rastros

codigo (crate engine, conferido contra a arvore atual)
- `crates/engine/src/gpu/vec.rs:5` (`struct GpuVec`, quatro campos: `buffer`,
  `capacity`, `usage`, `label: &'static str`)
- `crates/engine/src/gpu/vec.rs:13` (`GpuVec::new`, `usage | COPY_DST`,
  `mapped_at_creation: false`)
- `crates/engine/src/gpu/vec.rs:33` (`ensure_capacity`, dobra e nunca encolhe,
  `self.buffer = device.create_buffer(...)` dropa o buffer velho)
- `crates/engine/src/gpu/vec.rs:48` (`upload<T: bytemuck::Pod>`, `&mut self`,
  `cast_slice`, `queue.write_buffer(&self.buffer, 0, bytes)`)
- `crates/engine/src/gpu/vec.rs:59` (`buffer(&self) -> &wgpu::Buffer`, borrow
  compartilhado pro desenho)
- `crates/engine/src/gpu/vec.rs:63` (`capacity_bytes`, capacidade alocada, nao bytes
  vivos; alimenta o perf monitor)
- `crates/engine/src/compositor/layer/mod.rs:59` (campos `quad_vb`/`quad_ib` e os
  outros pares como `Option<GpuVec>`, posse opcional e persistente)
- `crates/engine/src/compositor/layer/mod.rs:95` (`INITIAL_VB_SIZE = 4096`,
  `INITIAL_IB_SIZE = 2048`)
- `crates/engine/src/compositor/layer/mod.rs:218` (`quad_buffers(&self)`, devolve
  `&wgpu::Buffer` via `vb.buffer()` mais a contagem de indices)
- `crates/engine/src/compositor/layer/geometry.rs:488` (`upload_quad_geometry`,
  `get_or_insert_with` aloca uma vez e reusa o `GpuVec`)
- `crates/engine/src/compositor/mod.rs:120` (`resolve`, loop de `upload_*` gateado por
  `if layer.dirty`, fase `&mut` antes do render pass)
- `crates/engine/src/window/render_passes.rs:104` (`pass.set_vertex_buffer(0, vb.slice(..))`
  e `set_index_buffer`, o borrow do buffer que dura o render pass)
- `crates/engine/src/compositor/memory.rs:16` (`gpu_memory_bytes`, soma
  `capacity_bytes` por buffer via `map_or(0, GpuVec::capacity_bytes)`)
- `crates/engine/src/compositor/memory.rs:58` (`headless_compositor_reports_zero_gpu_memory`,
  compositor sem device aloca zero buffer)

adr e contrato de invalidacao
- `kdb/adr/render-on-demand-requires-explicit-invalidation.md:13` (bug de scroll:
  estado mudou, handler retornou false, nada invalidou, app pareceu congelado)
- `kdb/adr/render-on-demand-requires-explicit-invalidation.md:22` (todo handler que
  muda algo visivel deve retornar true ou chamar o caminho de invalidacao; returning
  false is a statement that nothing visible changed)
- `kdb/adr/render-on-demand-requires-explicit-invalidation.md:29` (feature pronta
  quando a mudanca de estado provadamente agenda um frame; testes verificam o booleano)

benchmark
- `crates/engine/benches/scene_build.rs:10` (`bench_push_rects`, 100/1000/10000 rects
  num `Compositor::new()`)
- `crates/engine/benches/scene_build.rs:186` (grupos do bench: push_rects, push_paths,
  dirty_tracking, tessellation, signals, text_hashing)

versoes (conferidas contra o Cargo.toml)
- `Cargo.toml:50` wgpu 28 (`wgpu::Buffer`, `BufferUsages`, `queue.write_buffer`)
- `Cargo.toml:52` bytemuck 1 com feature `derive` (`Pod`, `cast_slice`)
- `Cargo.toml:99` criterion 0.5 (o bench de scene_build)
- `Cargo.toml:23` edition 2024

nao confirmado
- nenhuma ancora deste capitulo mede o custo do `upload`, do `ensure_capacity` ou do
  `queue.write_buffer`. o `scene_build.rs` roda headless (`Compositor::new()`, sem
  device), entao nao materializa nenhum `GpuVec`; ele mede a construcao CPU da cena que
  precede o upload, nao o upload em si. um numero de upload exigiria profile com device
  real, fora desta ancora.
- o `bench_push_rects` constroi a cena mas nao chama `resolve`/`upload`, entao mesmo o
  caminho de geometria que alimenta o `GpuVec` nao esta inteiramente coberto pela
  medicao; o que esta medido e o `draw_rect` empilhando `SceneNode`.
- a afirmacao de que o `wgpu::queue.write_buffer` cuida da sincronizacao no relogio da
  GPU e o comportamento documentado da API do wgpu, nao uma medicao feita aqui; o foco
  deste capitulo e a barreira de borrow no lado da CPU, que e o que o repo demonstra
  diretamente no codigo.
