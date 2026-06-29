---
title: modularizar por responsabilidade, o limite das 300 linhas
parte: 1
status: rascunho
rastros:
  - kdb/adr/adr-003-srp-modularization.md
  - kdb/adr/srp-modularization.md
  - crates/engine/src/compositor/mod.rs
  - crates/engine/src/window/render.rs
  - crates/engine/src/narrate_runtime/mod.rs
  - crates/engine/src/component/lifecycle_impl.rs
  - .contracts/.mantras/.code/.lang/.rust/rust-conventions.md
  - crates/engine/src/lib.rs
---

# modularizar por responsabilidade, o limite das 300 linhas

abre um arquivo de 1219 linhas e tenta achar onde a funcao que voce quer mexer
comeca. voce rola. passa por uns structs no topo, depois um bloco de impl, depois
um segundo impl da mesma struct trezentas linhas abaixo, depois um modulo de teste
gigante coladinho no fim. voce perde o fio. quando acha a funcao, ja esqueceu o
nome do campo que viu no struct la em cima e precisa rolar de volta. esse arquivo
existiu de verdade no plev, chamava `narrate_runtime.rs`, e era o maior do
codebase antes da refatoracao que esse capitulo conta.

a dor nao e estetica. arquivo grande nao e feio, e caro. caro de ler, caro de
revisar, caro de compilar incrementalmente, e, pra mim, caro de cocriar. quando eu
peco pro claude mexer numa funcao no meio de um arquivo de mil linhas, ele tem que
carregar o arquivo inteiro pra contexto pra nao quebrar o que esta fora da janela
que ele ve. o arquivo grande e um imposto que voce paga toda vez que toca nele, e
voce paga em silencio, porque a GPU rapida esconde a lentidao do mesmo jeito que o
editor rapido esconde o tamanho do arquivo. esconder nao e resolver.

a regra que saiu disso e simples de enunciar e chata de aplicar: um arquivo de
codigo de producao tem um tamanho maximo, e quando passa, voce divide por
responsabilidade, nao por linha. esse capitulo abre nessa imagem do arquivo que
nao cabe na cabeca e desce ate o `pub use` que mantem a API publica intacta
enquanto o arquivo por baixo vira um diretorio inteiro.

## o limite, e o numero que andou

o numero comecou pessoal. eu apliquei a regra de modularizar por responsabilidade
quando os meus arquivos passavam de umas 333 linhas, muito antes do plev virar
projeto serio, e melhorou minha leitura, minha revisao e a cocriacao com o claude
e os demais modelos. era heuristica de bolso, nao contrato.

quando virou contrato no plev, o numero foi fixado em 300. o adr-003 registra a
decisao assim, seca: "adotar limite maximo de 300 linhas por arquivo .rs,
dividindo modulos monoliticos em submodulos por responsabilidade unica"
(`kdb/adr/adr-003-srp-modularization.md:29`). e o titulo desse capitulo carrega
esse 300 de proposito, porque foi com 300 na cabeca que a refatoracao grande
aconteceu.

so que o numero andou depois. a convencao viva do projeto hoje nao diz 300, diz
369. esta escrito em `.contracts/.mantras/.code/.lang/.rust/rust-conventions.md:48`:
"hard limit 369 lines per source file, target ~220 for new files. split oversize
production code by single responsibility". e a linha seguinte traz o detalhe que o
adr original nao tinha e que importa muito na pratica: blocos de teste sao isentos
da conta. a convencao e explicita que `#[cfg(test)]` nao conta, "in-file unit
tests are idiomatic rust and must never force a split", e fecha com "the limit is a
tax on big production units, never a tax on testing well" (linhas 49 a 53).

vale parar nessa isencao, porque ela e a parte madura da regra. se voce conta o
bloco de teste no limite, voce empurra os testes pra fora do arquivo so pra caber
no numero, e teste in-file e idiomatico em rust, fica ao lado do que ele testa.
o `compositor/tests.rs` hoje tem 1431 linhas e nao viola nada, porque e teste. a
regra mira a unidade de producao grande, que e onde mora a confusao de
responsabilidade, e nao pune quem testa bem. tres numeros entao orbitam a mesma
ideia: 333 da origem pessoal, 300 da decisao registrada no adr-003, 369 da
convencao atual com isencao de teste. eu marco isso explicito porque o livro se
sustenta em ler a fonte, nao a legenda: o titulo fala em 300, a regra hoje vale 369,
e a diferenca e historia real do projeto, nao erro de digitacao.

o que nao mudou foi o criterio de corte. nunca foi "passou da linha, parte no
meio". sempre foi "passou da linha, isso e sinal de que tem mais de uma
responsabilidade aqui dentro, ache as costuras e separe". o numero e um alarme,
nao a regra. a regra e single responsibility.

## o monolito, e a conta que nao bate

antes da refatoracao o codebase tinha varios desses arquivos. e aqui aparece a
primeira coisa que eu nao consigo reconciliar entre as fontes, entao marco como
divergencia honesta. o adr-003 diz que o codebase "acumulou 27 arquivos .rs acima
de 300 linhas" (`kdb/adr/adr-003-srp-modularization.md:17`, repetido na linha 23).
o segundo documento, `srp-modularization.md`, diz "44 arquivos rust acima de 300
linhas" (`kdb/adr/srp-modularization.md:14`). o escopo desse capitulo trabalha com
44, que e o numero do segundo doc e da tabela de metricas dele
(`kdb/adr/srp-modularization.md:106`, linha "arquivos .rs > 300 linhas, 44, 0").
nao tenho como verificar qual contagem e a certa hoje, porque o estado pre-refator
nao esta mais no working tree e o historico git anterior foi parcialmente perdido
(o forgejo self-hosted que guardava parte da historia foi deletado, mesma nota que
aparece no capitulo de dirty tracking). entao registro os dois numeros: 27 no
adr-003, 44 no srp-modularization, e sigo o escopo usando 44.

o que as duas fontes concordam e no campeao. o maior arquivo era o
`narrate_runtime.rs`, com 1219 linhas (`kdb/adr/adr-003-srp-modularization.md:24`,
`kdb/adr/srp-modularization.md:14`). dentro dele coexistiam, segundo o adr,
"definicoes de tipos, logica de execucao, API publica e testes"
(`kdb/adr/adr-003-srp-modularization.md:19-20`). isso e a definicao de violacao de
single responsibility: quatro motivos diferentes pra mexer no mesmo arquivo. mudou
o formato de um tipo, voce abre. mudou a logica de execucao, voce abre o mesmo
arquivo. mudou a API, mesmo arquivo. cada uma dessas mudancas tem ritmo proprio,
autor proprio, risco proprio, e estavam todas amarradas no mesmo blob de 1219
linhas.

hoje `narrate_runtime` nao e mais um arquivo, e um diretorio, e da pra ver a
costura por onde ele rasgou:

```
crates/engine/src/narrate_runtime/
  mod.rs          51 linhas   (declaracoes + re-export)
  tokenizer.rs   126 linhas   (lexing)
  parser.rs      170 linhas   (parsing)
  keywords.rs     62 linhas   (tabela de palavras)
  modifiers.rs   269 linhas   (logica de modificadores)
  extraction.rs   97 linhas   (extracao)
  tests/                      (testes isolados)
```

1219 linhas viraram seis arquivos de producao, nenhum perto do limite, mais um
diretorio de teste. e a divisao nao foi arbitraria. tokenizer, parser, keywords,
modifiers, extraction sao as fases reais do runtime, cada uma com sua propria razao
pra mudar. quem mexe na tabela de keyword nao precisa nem abrir o parser. esse e o
ganho, e ele e estrutural, nao cosmetico.

## o padrao, submodulo com mod.rs e pub use

o jeito de dividir e sempre o mesmo, e o `compositor` e o melhor exemplo vivo
porque ele ainda esta inteiro no working tree e compila contra o `Cargo.toml` que
voce tem na mao. o topo do `crates/engine/src/compositor/mod.rs` e quase so isso,
declaracao e re-export:

```rust
mod clip;
pub(crate) mod drawing;
mod layer;
mod layer_ops;
mod memory;
mod scene;
mod sequence;
mod stats;
mod vertex;

#[cfg(test)]
mod tests;

pub use clip::{
    ClipRect, DrawRange, clip_to_scissor, intersect_rects, intersect_scissors, merge_text_groups,
};
pub use drawing::{GradientRectParams, RoundedRectParams, ShadowParams};
pub use layer::{Layer, LayerEffect, LayerId};
pub use scene::{SceneNode, TextNodeKey};
pub use sequence::{DrawCommand, DrawKind};
pub use stats::RenderStats;
pub use vertex::{
    BackdropVertex, ImageVertex, QuadVertex, RectSdfVertex, ShadowVertex, gradient_direction,
    shadow_padding, shadow_sigma,
};
```

leia isso devagar porque tem mais decisao aqui do que parece. cada `mod` declara
um submodulo que e um arquivo separado: `clip.rs`, `scene.rs`, `vertex.rs` e por
ai. a visibilidade de cada `mod` e escolhida na mao. `mod clip;` sem `pub` quer
dizer que o modulo `clip` em si e privado ao compositor, mas os tipos que ele
exporta sobem pela linha `pub use clip::{...}`. ja `pub(crate) mod drawing;` deixa
o modulo `drawing` visivel pro crate engine inteiro, nao so pro compositor, porque
outras partes da engine desenham direto por ali. essa diferenca entre `mod`,
`pub(crate) mod` e o `pub use` e o coracao do padrao. o arquivo virou diretorio,
mas o que sai pela porta da frente continua sendo escolhido a dedo.

o padrao de divisao que o segundo doc registra e essa receita
(`kdb/adr/srp-modularization.md:113-122`): `types.rs` pra structs, enums e
constantes; `engine.rs`, `processor.rs` ou `execution.rs` pra logica central;
`api.rs` pra interface publica; `tests.rs` pros testes; e o `mod.rs` so com
declaracao de submodulo e `pub use`. pra pipeline de GPU ele acrescenta
`pipelines.rs`, `render.rs`, `gpu.rs`. o compositor segue o espirito disso com
nomes proprios do dominio, que e o certo: `clip`, `scene`, `vertex`, `layer`,
`sequence`, `stats`, `memory` sao as responsabilidades reais de um compositor, e
nao um molde generico colado por cima.

## a API que nao muda, e por que isso e o ponto inteiro

aqui esta a propriedade que torna toda a refatoracao segura, e ela e a coisa que
mais importa do capitulo. quando voce transforma `signal.rs` em `signal/mod.rs`, ou
`narrate_runtime.rs` em `narrate_runtime/mod.rs`, o resto do codebase nao muda uma
linha. o `lib.rs` continua igual. o motivo e uma regra de resolucao de modulo do
proprio rust: `pub mod compositor;` resolve tanto pra `compositor.rs` quanto pra
`compositor/mod.rs`, sem distincao. o segundo doc registra isso direto: "rust
resolve `pub mod signal;` tanto para `signal.rs` quanto para `signal/mod.rs`
automaticamente, portanto `lib.rs` nao requer alteracao"
(`kdb/adr/srp-modularization.md:25-28`).

confere no `lib.rs` real: a unica linha que declara o compositor e
`pub mod compositor;` (`crates/engine/src/lib.rs:5`). nao tem nada ali dizendo se
`compositor` e um arquivo ou um diretorio. pra quem consome, e indiferente. e essa
indiferenca e exatamente o que faz `pub use` valer ouro. quando o `mod.rs` faz
`pub use scene::SceneNode;`, o caminho publico do tipo continua sendo
`crate::compositor::SceneNode`, mesmo que o tipo viva fisicamente em
`compositor/scene.rs`. quem importava antes da divisao continua importando depois,
byte por byte igual. o adr-003 fixa isso como invariante: "API publica identica
(zero breaking changes)" e "470 testes continuam passando sem modificacao"
(`kdb/adr/adr-003-srp-modularization.md:57-58`).

essa e a parte que separa refatoracao de reescrita. uma reescrita muda o contrato e
te obriga a cacar todos os call sites. uma modularizacao por `pub use` mantem o
contrato congelado e move so a implementacao por baixo. e por isso que da pra fazer
isso em 44 arquivos sem virar um pesadelo de migracao: cada arquivo dividido e um
no-op pra quem o usa de fora. a refatoracao acontece inteira no espaco privado do
modulo. o mundo la fora nao percebe.

a digressao que vale: isso so funciona porque rust separa de verdade visibilidade
de localizacao. em muita linguagem, onde voce poe um simbolo determina quem o ve. em
rust, `pub use` desacopla as duas coisas. voce pode espalhar a implementacao por
sete arquivos e reagrupar a superficie publica num unico ponto de re-export. o
arquivo vira detalhe de organizacao interna, e a API vira uma decisao explicita,
escrita num lugar so. o `mod.rs` do compositor e literalmente o mapa do que e
publico, do que e `pub(crate)`, e do que fica privado. da pra auditar a superficie
inteira do modulo lendo trinta linhas no topo de um arquivo.

## ResolveResources, quando o argumento vira struct

modularizar nao foi so cortar arquivo. teve um caso onde a divisao por
responsabilidade esbarrou numa assinatura de funcao ruim, e a correcao certa nao
era um novo arquivo, era um novo tipo. o metodo `Compositor::resolve` recebia oito
argumentos posicionais. o segundo doc lista eles: "device, queue, format, width,
height, composite_bgl, opacity_bgl, sampler"
(`kdb/adr/srp-modularization.md:50-51`). chamar uma funcao assim e um campo minado:
troca a ordem de dois `&BindGroupLayout` e compila do mesmo jeito, com o
comportamento errado.

a solucao foi agrupar tudo num struct com campos nomeados. olha o tipo real hoje em
`crates/engine/src/compositor/mod.rs:27`:

```rust
/// GPU resources needed for layer texture resolution and compositing.
pub struct ResolveResources<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub format: wgpu::TextureFormat,
    pub width: u32,
    pub height: u32,
    /// MSAA sample count for layer textures (1 = render directly, no resolve).
    pub msaa_samples: u32,
    pub composite_bgl: &'a wgpu::BindGroupLayout,
    pub opacity_bgl: &'a wgpu::BindGroupLayout,
    pub sampler: &'a wgpu::Sampler,
}
```

e a assinatura ficou `pub fn resolve(&mut self, res: &ResolveResources<'_>)`
(`crates/engine/src/compositor/mod.rs:106`). um detalhe pra marcar: o doc fala em
oito argumentos, o struct hoje tem nove campos. ele ganhou `msaa_samples` depois que
o adr foi escrito, o que e o tipo de coisa que esse struct foi feito pra absorver.
adicionar um recurso de GPU agora e acrescentar um campo nomeado, nao quebrar a
ordem de uma lista posicional de todo mundo que chama. esse e o beneficio composto:
o call site fica auto-documentado e a assinatura para de ser fragil a crescimento.

o call site real, no loop de render da janela, mostra como fica
(`crates/engine/src/window/render.rs:74`):

```rust
self.compositor
    .resolve(&crate::compositor::ResolveResources {
        device: &gpu.device,
        queue: &gpu.queue,
        format: gpu.surface_format(),
        width: gpu.surface_config.width,
        height: gpu.surface_config.height,
        msaa_samples: gpu.config.msaa_samples,
        composite_bgl: &gpu.composite_bind_group_layout,
        opacity_bgl: &gpu.opacity_bind_group_layout,
        sampler: &gpu.composite_sampler,
    });
```

cada campo se nomeia. ninguem mais troca `composite_bgl` com `opacity_bgl` por
descuido de ordem. e aqui vem a armadilha que esse struct gerou, que e a parte mais
instrutiva do capitulo inteiro, entao ela tem secao propria.

## armadilha 1, pub(crate) quebrou 14 exemplos

depois de criar o `ResolveResources`, o clippy reclamou. o warning era "type is
more private than item", e a sugestao implicita era reduzir a visibilidade do
metodo `resolve()` pra `pub(crate)`, porque ele expunha um tipo que o lint achava
menos publico do que devia. seguir o clippy de olho fechado compilou a lib
limpa. e quebrou todos os exemplos.

o segundo doc registra o estrago: "isso compilou na lib mas quebrou todos os
exemplos que chamam `compositor.resolve(...)` de fora do crate"
(`kdb/adr/srp-modularization.md:78-80`). a regra que saiu disso esta escrita logo
embaixo: "antes de reduzir visibilidade de metodos publicos, verificar se exemplos
(`cargo check --examples`) e crates dependentes os utilizam. a solucao correta foi
tornar o struct `ResolveResources` `pub` em vez de esconder o metodo"
(`kdb/adr/srp-modularization.md:81-85`).

isso e mais fundo do que parece. exemplo em rust mora em `examples/` e compila como
se fosse um crate de fora. ele so enxerga a API publica, igual a um consumidor de
verdade. entao exemplo nao e enfeite, e teste de superficie publica. quando o
clippy disse "esse metodo poderia ser mais privado", ele estava certo do ponto de
vista da lib isolada e errado do ponto de vista do contrato real, porque a lib nao
vive isolada, ela vive com 14 exemplos pendurados nela. confirmo que o
`ResolveResources` continua `pub` no codigo de hoje (`crates/engine/src/compositor/mod.rs:28`)
e que ele e usado de fora em varios exemplos: `monster_player`, `layers`, `input`,
`sdf_shapes`, `todo`, `lottie_player`, `charts`, `text`, `text_input`, `visual`,
entre outros, todos construindo o struct direto.

a licao que eu tiro: o clippy mede o crate, voce tem que medir o produto. o gate do
projeto inclui `cargo check --examples` justamente porque a lib passar sozinha nao
prova nada sobre quem depende dela. reduzir visibilidade e uma mudanca de contrato
disfarcada de limpeza de lint, e contrato so se mede com todos os consumidores no
loop.

## armadilha 2, worktree nao ve arquivo untracked

a refatoracao foi paralelizada com worktrees git. a ideia e boa: cada agente de
refatoracao roda numa worktree isolada, mexe num modulo independente, e ninguem
pisa no arquivo do outro. o doc registra: "agentes de refatoracao foram executados
em worktrees git isoladas para evitar conflitos de escrita simultanea"
(`kdb/adr/srp-modularization.md:38-40`), com paralelismo de "3-4 modulos refatorados
simultaneamente" (`kdb/adr/srp-modularization.md:43`).

ai veio o buraco. worktree git so contem arquivo que o git rastreia. teve um diretorio
de trabalho em progresso, `examples-wip/`, que estava fora do git, e os agentes
mandados refatorar ele "reportaram sucesso mas as alteracoes nao existiam (diretorio
ausente da worktree)" (`kdb/adr/srp-modularization.md:68-70`). o agente escreveu num
caminho que, dentro da worktree, simplesmente nao existia, e ainda assim achou que
tinha terminado. trabalho fantasma. a regra que ficou: "verificar `git ls-files
<path>` antes de usar worktree para arquivos que podem estar em `.gitignore`"
(`kdb/adr/srp-modularization.md:46`).

confirmei a forma dessa armadilha no tree de hoje. `git ls-files | grep examples-wip`
nao retorna nada, e o diretorio `examples-wip` nao existe mais no working tree.
entao nao consigo reconstruir o episodio exato, mas a mecanica e verificavel e a
regra e solida: worktree e uma janela pro estado rastreado do repo, e o que o git
nao conhece, a worktree nao mostra. um comando antes de comecar, `git ls-files no
caminho`, e voce sabe se vai trabalhar no vazio.

essa e do tipo de falha que so doi quando voce automatiza. uma pessoa abrindo a
worktree veria na hora que a pasta nao esta la. um agente que confia no caminho que
recebeu escreve, fecha, e diz pronto. quanto mais voce delega execucao, mais o
pre-flight importa, porque ninguem mais esta olhando a tela no meio do caminho.

## armadilha 3, agentes paralelos sem isolamento corrompem estado

a worktree existe por causa dessa terceira armadilha, que e o motivo de todo o
cuidado. quando os agentes nao foram isolados, deu ruim. o doc: "quando 4 agentes de
clippy operaram no mesmo working tree sem worktree, alteracoes de um agente foram
sobrescritas por outro. resultado: build quebrado com metades de refatoracoes
aplicadas" (`kdb/adr/srp-modularization.md:88-91`).

esse e o classico de concorrencia, so que com arquivo no lugar de memoria. dois
escritores no mesmo recurso sem coordenacao e race condition, nao importa se o
recurso e um `u64` ou um `mod.rs`. o agente A le o arquivo, planeja a mudanca, e
escreve. entre o ler e o escrever, o agente B tambem escreveu. a escrita de A apaga
a de B, ou pior, as duas se intercalam e voce fica com meia refatoracao de cada,
que e o estado que o doc chama de "build quebrado com metades aplicadas". e o pior
estado possivel, porque nao e nem o antigo nem o novo, e um Frankenstein que nao
compila e nao da pra reverter limpo.

a regra: "para alteracoes nos mesmos arquivos, usar agentes sequenciais ou um unico
agente. paralelizar apenas quando os conjuntos de arquivos sao disjuntos"
(`kdb/adr/srp-modularization.md:93-95`). disjunto e a palavra chave. paralelismo
seguro exige particao, conjuntos de arquivos que nao se tocam. quando da pra
garantir isso, a worktree por modulo independente realiza a particao no nivel do
filesystem e o paralelismo voa. quando nao da pra garantir, a resposta certa e
serializar, mesmo que doa na velocidade. e a mesma logica de lock que voce usaria
em codigo concorrente, so que aplicada a uma frota de agentes mexendo num repo.
disciplina de fronteira no lugar de mutex.

isso conversa direto com uma regra do agents.md do projeto: agentes em paralelo so
quando os arquivos sao disjuntos, senao um unico agente ou sequencial. a frota nao e
magica, ela obedece as mesmas leis de coordenacao de qualquer sistema concorrente.

## armadilha 4, module_inception

a quarta e a menor, e e quase comica de tao especifica de rust. quando voce divide
`component.rs` num diretorio `component/`, a tentacao e por o miolo num arquivo
`component/component.rs`. o clippy odeia isso. ele dispara o lint `module_inception`
quando um modulo tem o mesmo nome do diretorio pai. o doc registra o caso e a
correcao: "clippy reporta warning quando um modulo tem o mesmo nome que seu diretorio
pai. renomeado para `component/lifecycle_impl.rs`"
(`kdb/adr/srp-modularization.md:97-100`).

confirmei no tree de hoje: existe `crates/engine/src/component/lifecycle_impl.rs`, e
nao existe nenhum `component/component.rs`. o nome novo e melhor por acidente feliz:
`lifecycle_impl` diz o que tem dentro, enquanto `component/component` so repetia o
nome do pai sem informacao nova. o lint te empurrou pra um nome que carrega
significado. as vezes a regra mecanica acerta o ponto humano de tabela.

## o numero, 44 para 0, e o estado honesto de hoje

a tabela de metricas do segundo doc fecha a conta da refatoracao
(`kdb/adr/srp-modularization.md:104-111`):

| metrica | antes | depois |
|---------|-------|--------|
| arquivos .rs > 300 linhas | 44 | 0 |
| maior arquivo .rs | 1219 | 300 |
| clippy warnings (-D warnings) | 107 | 0 |
| testes passando | 470 | 470 |
| exemplos compilaveis | 0 | 15 |
| total arquivos .rs | ~60 | 271 |

eu leio essa tabela com as duas linhas que mais me importam grifadas. testes
passando, 470 antes, 470 depois. e exemplos compilaveis, 0 antes, 15 depois. a
primeira prova que a API publica nao mudou: se 470 testes passavam e continuaram
passando sem edicao, o contrato ficou congelado, que era a promessa inteira do
`pub use`. a segunda prova que a superficie publica voltou a ser exercitada de fora:
os exemplos que nao compilavam voltaram a compilar, e e isso que pega regressao de
visibilidade tipo a do `pub(crate)`. o adr-003 traz a contraparte de custo: o numero
de arquivos subiu "de ~60 para 271" (`kdb/adr/adr-003-srp-modularization.md:52`), e
a distribuicao final ficou "45% (0-100 linhas), 31% (101-200), 24% (201-300)"
(`kdb/adr/adr-003-srp-modularization.md:65`). mais arquivos, cada um menor, e essa e
a troca que a regra compra de proposito.

agora a parte honesta, porque o livro le a fonte e nao a legenda. esses numeros sao
um retrato de um momento. o doc carrega no frontmatter um `commit: 2d95911`
(`kdb/adr/srp-modularization.md:7`) que eu nao acho no historico do branch atual,
provavelmente pela mesma perda parcial de historia git que ja citei. marco isso como
nao confirmado: nao consigo amarrar a tabela a um commit verificavel. e o estado de
hoje, no branch `refactor/workspace-restructure`, nao e mais o "0 arquivos acima de
300" da coluna depois. contei 14 arquivos de producao acima de 369 linhas agora,
fora os testes, com o `ui/widgets/card.rs` em 1119 linhas e o `ui/icons.rs` em 858
liderando a lista. isso nao desmente o adr, contextualiza ele: o branch atual e uma
reestruturacao do workspace em andamento, o codigo cresceu de novo desde a
refatoracao registrada, e a regra continua valendo como alvo, agora em 369 com
isencao de teste. dois arquivos do adr tambem nao batem com o tree de hoje: o
`message_dock/ui.rs` que o adr-003 cita como o maior arquivo pos-refator de 300
linhas (`kdb/adr/adr-003-srp-modularization.md:64`) nao existe mais, e os structs
`CardColors` e `CardLayout` que o segundo doc descreve pro showcase
(`kdb/adr/srp-modularization.md:59-62`) nao aparecem no codigo atual. marco os tres
como nao confirmados no tree de hoje. a foto e real, o objeto fotografado andou.

## por que assim, e nao de outro jeito

a pergunta funda nao e "qual o numero magico de linhas". numero magico nao existe,
333, 300 e 369 sao todos chutes calibrados pra mesma intuicao. a pergunta e por que
o tamanho de arquivo merece virar invariante de qualidade num projeto, ao lado de
zero warning e teste passando. e a resposta tem quatro pernas, e nenhuma e estetica.

a primeira e a unica razao pra mudar. single responsibility, no fundo, e isso: um
arquivo deveria ter uma unica razao pra mudar. o adr-003 poe nesses termos, "cada
arquivo tem uma unica razao para mudar" (`kdb/adr/adr-003-srp-modularization.md:46`).
quando tipo, execucao, API e teste moram juntos, o arquivo tem quatro razoes pra
mudar, e cada commit que toca uma das quatro arrisca as outras tres. separar por
responsabilidade reduz o raio de explosao de cada mudanca. mexer no tokenizer do
narrate nao chega perto do parser, porque sao arquivos diferentes com fronteira
real entre eles.

a segunda e compilacao incremental. o adr lista "compilacao incremental mais
eficiente (unidades menores)" como consequencia positiva
(`kdb/adr/adr-003-srp-modularization.md:48`). rust recompila por unidade. arquivo
menor e unidade menor, e mudanca local reconstroi menos. nao tenho um numero de
benchmark de compilacao pra colar aqui, entao nao invento um, marco como afirmacao
do adr e nao medicao independente. mas a direcao e conhecida: um arquivo de 1219
linhas que muda toda hora e um gargalo de rebuild que voce paga em cada iteracao.

a terceira e revisao e navegacao, e essa e a que eu sinto na pele todo dia. o doc
poe como "navegacao por responsabilidade em vez de por scroll"
(`kdb/adr/srp-modularization.md:31`). e literal. quando o arquivo e `clip.rs`, eu sei
que clip mora ali e so ali. nao rolo mil linhas cacando, abro o arquivo certo pelo
nome. code review melhora pela mesma razao: um diff que toca `vertex.rs` esta falando
de vertice, e o revisor carrega so esse contexto na cabeca. arquivo pequeno e um
contrato com o leitor sobre o que ele vai encontrar la dentro.

a quarta e a que liga isso ao motivo do plev existir do jeito que existe: cocriacao
com modelo de linguagem. o arquivo grande estoura a janela util de atencao do
modelo, do meu e do dele. quando eu peco uma mudanca cirurgica num arquivo de mil
linhas, o modelo tem que segurar mil linhas de contexto pra nao quebrar o que esta
fora da edicao, e a chance de ele perder um detalhe trinta linhas acima cresce com o
tamanho. arquivo por responsabilidade entrega ao modelo a unidade certa pra raciocinar:
o `mod.rs` pra entender a superficie, o `scene.rs` pra mexer em cena, sem arrastar o
resto junto. a regra de 300, ou 369, ou 333, e em boa parte uma regra de ergonomia de
contexto, pra humano e pra agente ao mesmo tempo.

e tem o custo, que seria desonesto esconder. o adr lista os negativos sem suavizar:
"maior numero de arquivos (de ~60 para 271)", "necessidade de pub(crate)/pub(super)
para visibilidade entre submodulos", "impl blocks distribuidos entre arquivos (rust
permite, mas exige disciplina)" (`kdb/adr/adr-003-srp-modularization.md:50-54`). cada
um e real. mais arquivo e mais ponto de navegacao no editor. `impl` espalhado quer
dizer que os metodos de um tipo podem viver em arquivos diferentes, o que rust
permite mas exige que voce saiba onde procurar. e a visibilidade entre submodulos
deixa de ser de graca: o que antes era um item privado no mesmo arquivo agora precisa
de `pub(crate)` ou `pub(super)` explicito pra um submodulo irmao enxergar. isso e
trabalho. a aposta do projeto e que o trabalho de declarar visibilidade explicita
paga, porque ele torna a fronteira de cada modulo visivel no codigo em vez de
implicita no fato de tudo estar no mesmo arquivo. fronteira explicita e mais chata de
escrever e muito mais barata de manter.

a digressao que fecha o porque: a regra das 300 linhas e prima da regra de pipe curto
do unix e do mnemonico de tres letras do assembly. nenhuma delas e sobre o numero. a
do unix e sobre fazer uma coisa bem, a do assembly e sobre nomear um espaco de
operacoes de forma que caiba na memoria muscular. a das 300 e sobre fazer um arquivo
caber numa unica razao de mudar e numa unica passada de leitura. less, but better. o
numero e so o alarme que dispara quando voce parou de fazer uma coisa so.

## o que isso me ensinou

a parte que demorei pra internalizar nao foi o `pub use`, foi que modularizar bem e
quase todo sobre o que voce nao mexe. a refatoracao de 44 arquivos so foi viavel
porque a API publica ficou congelada o tempo inteiro. 470 testes passando antes e
depois nao e detalhe de rodape, e a prova de que voce moveu so o que era pra mover. o
`pub use` no `mod.rs` e o instrumento que deixa voce reorganizar o interior sem
tocar no contrato, e e por isso que da pra fazer isso em escala sem virar um mar de
breaking changes.

as armadilhas todas tem o mesmo formato, agora que olho de longe. a worktree que nao
ve untracked, o `pub(crate)` que quebrou os exemplos, os agentes paralelos que se
sobrescreveram: todas sao falha de fronteira. a worktree e uma fronteira entre o que
o git rastreia e o que nao rastreia, e o erro foi assumir que o caminho estava do
lado de dentro. o `pub(crate)` e uma fronteira entre o crate e seus consumidores, e o
erro foi medir so um lado. o paralelismo de agentes e uma fronteira entre conjuntos de
arquivos, e o erro foi deixar os conjuntos se tocarem. modularizar bem e desenhar
fronteira boa, e cada armadilha foi uma fronteira que eu achei que estava num lugar e
estava em outro.

se eu fosse deixar uma coisa pra aurora ler disso aqui um dia: o limite de linha nunca
foi sobre linha. e um alarme barato que dispara antes de o arquivo ficar caro demais
pra ler, pra revisar, pra compilar e pra cocriar. quando ele toca, a pergunta certa
nao e "como corto isso pra caber", e "quantas razoes de mudar eu enfiei aqui dentro
sem perceber". o `pub use` cuida pra que responder essa pergunta nao custe nada pra
quem usa o codigo de fora. menos arquivo grande, mais fronteira clara, mesma API. o
resto e encanamento.

## rastros

adr e convencao

- `kdb/adr/adr-003-srp-modularization.md:29` (decisao: limite de 300 linhas por
  arquivo .rs, divisao por single responsibility)
- `kdb/adr/adr-003-srp-modularization.md:17` e `:23` (27 arquivos acima de 300 linhas
  pre-refator, segundo este doc)
- `kdb/adr/adr-003-srp-modularization.md:24` (maior arquivo: 1219 linhas,
  narrate_runtime.rs)
- `kdb/adr/adr-003-srp-modularization.md:19-20` (responsabilidades misturadas: tipos,
  execucao, API, testes)
- `kdb/adr/adr-003-srp-modularization.md:46-48` (consequencias positivas: unica razao
  pra mudar, compilacao incremental)
- `kdb/adr/adr-003-srp-modularization.md:50-54` (consequencias negativas: ~60 -> 271
  arquivos, pub(crate)/pub(super), impl distribuido)
- `kdb/adr/adr-003-srp-modularization.md:57-58` (API inalterada, 470 testes passando
  sem modificacao)
- `kdb/adr/adr-003-srp-modularization.md:64-65` (maior pos-refator: 300 linhas
  message_dock/ui.rs; distribuicao 45/31/24)
- `kdb/adr/srp-modularization.md:14` (44 arquivos acima de 300 linhas, maior 1219 em
  narrate_runtime.rs)
- `kdb/adr/srp-modularization.md:25-28` (rust resolve `pub mod` para .rs e /mod.rs,
  lib.rs nao muda)
- `kdb/adr/srp-modularization.md:31` (navegacao por responsabilidade em vez de scroll)
- `kdb/adr/srp-modularization.md:38-46` (worktrees isoladas; regra git ls-files antes
  de worktree)
- `kdb/adr/srp-modularization.md:48-51` (resolve com 8 args posicionais, lista dos
  argumentos)
- `kdb/adr/srp-modularization.md:59-62` (CardColors/CardLayout para showcase scene)
- `kdb/adr/srp-modularization.md:68-70` (armadilha worktree untracked, examples-wip
  ausente)
- `kdb/adr/srp-modularization.md:78-85` (pub(crate) em resolve quebrou 14 exemplos,
  solucao: ResolveResources pub)
- `kdb/adr/srp-modularization.md:88-95` (agentes paralelos sem isolamento corrompem
  estado; regra de conjuntos disjuntos)
- `kdb/adr/srp-modularization.md:97-100` (module_inception, component/component.rs ->
  lifecycle_impl.rs)
- `kdb/adr/srp-modularization.md:104-111` (tabela de metricas: 44->0, 1219->300,
  107->0 warnings, 470->470 testes, 0->15 exemplos, ~60->271 arquivos)
- `kdb/adr/srp-modularization.md:113-122` (padrao de divisao: types/engine/api/tests/
  mod, pipelines/render/gpu)
- `.contracts/.mantras/.code/.lang/.rust/rust-conventions.md:48-53` (limite vivo: 369
  linhas, target ~220, isencao de blocos #[cfg(test)])

codigo (crate engine, conferido contra o Cargo.toml)

- `crates/engine/src/compositor/mod.rs:1-25` (declaracoes de submodulo + pub use
  re-export; mod vs pub(crate) mod vs pub use)
- `crates/engine/src/compositor/mod.rs:27-39` (struct ResolveResources<'a>, pub, 9
  campos nomeados incluindo msaa_samples)
- `crates/engine/src/compositor/mod.rs:106` (assinatura `pub fn resolve(&mut self,
  res: &ResolveResources<'_>)`)
- `crates/engine/src/window/render.rs:74-85` (call site real de resolve com struct
  nomeado)
- `crates/engine/src/lib.rs:5` (`pub mod compositor;`, indiferente a arquivo vs
  diretorio)
- `crates/engine/src/narrate_runtime/mod.rs` (diretorio: tokenizer, parser, keywords,
  modifiers, extraction, tests; nenhum arquivo de producao perto do limite)
- `crates/engine/src/component/lifecycle_impl.rs` (renomeacao do module_inception,
  confirmada no tree)

versoes (conferidas contra o Cargo.toml)

- `Cargo.toml:50` wgpu 28 (tipos wgpu::Device, Queue, TextureFormat, BindGroupLayout,
  Sampler em ResolveResources)
- `Cargo.toml:23` edition 2024

nao confirmado

- divergencia de contagem pre-refator: adr-003 diz 27 arquivos acima de 300 linhas
  (`adr-003-srp-modularization.md:17`), srp-modularization diz 44
  (`srp-modularization.md:14`). o capitulo segue 44 conforme o escopo, mas registra
  os dois.
- `commit: 2d95911` no frontmatter de `srp-modularization.md:7` nao foi encontrado no
  historico do branch atual; provavel perda parcial de historia git (forgejo
  self-hosted deletado). a tabela de metricas nao foi amarrada a um commit verificavel.
- `message_dock/ui.rs` (`adr-003-srp-modularization.md:64`) nao existe no working tree
  atual; nao verificavel.
- `CardColors`/`CardLayout` (`srp-modularization.md:59-62`) nao aparecem no codigo
  atual; nao verificaveis.
- estado atual do branch `refactor/workspace-restructure`: contei 14 arquivos de
  producao acima de 369 linhas (excluindo testes), liderados por
  `ui/widgets/card.rs` (1119) e `ui/icons.rs` (858). isso e estado de reestruturacao
  em andamento, nao o "0 acima de 300" da coluna pos-refator do adr; a contagem foi
  feita por wc -l no tree de hoje, nao por ferramenta do projeto.
- ganho de compilacao incremental e afirmacao do adr
  (`adr-003-srp-modularization.md:48`), sem benchmark de tempo de build medido neste
  capitulo.
- "470 testes passando" e "0 -> 15 exemplos compilaveis" sao numeros do
  `srp-modularization.md:104-111`, nao reexecutados aqui (cargo indisponivel no
  ambiente de escrita).
