---
title: a layer limpa nao desenha
parte: 2
status: amostra
rastros:
  - kdb/adr/layer-system.md
  - kdb/mission/rules.md#performance
  - kdb/adr/benchmark-results.md
  - crates/engine/src/compositor/layer/mod.rs
  - crates/engine/src/compositor/scene.rs
  - crates/engine/benches/scene_build.rs
---

# a layer limpa nao desenha

imagina uma tela cheia de janela, botao, um paragrafo de texto. voce mexe o
mouse e nada muda na imagem. o cursor anda pela tela, mas a janela continua no
mesmo lugar, com a mesma cor, o mesmo texto. quantas vezes o computador deveria
redesenhar uma tela que nao mudou?

a resposta honesta e nenhuma. e quase todo software desenha mesmo assim, frame
depois de frame, repintando pixel que ja estava certo. a maioria so nao percebe
porque a GPU e rapida o bastante pra esconder o desperdicio. esconder nao e o
mesmo que resolver.

o plev parte de uma ideia simples, quase preguicosa: o trabalho mais rapido e o
que nao acontece. se uma camada da tela esta igual ao frame anterior, ela nao
desenha. nem um pixel. e o nome disso, na engine, e dirty tracking por hash.
este capitulo abre nessa imagem do quadro que nao mudou e desce ate o `FxHasher`
e o numero de microssegundo que aparece no benchmark.

## o quadro que nao mudou

pensa num pintor que recebe a tarefa de manter um quadro na parede sempre
atualizado. toda manha ele chega, olha o quadro, e tem duas opcoes. pode apagar
tudo e pintar de novo do zero, todo dia, mesmo que o desenho seja exatamente o
de ontem. ou pode tirar uma foto do que pintou ontem, comparar com o que deveria
estar la hoje, e so pegar o pincel se algo for diferente.

o segundo pintor faz menos trabalho. parece obvio dito assim. o detalhe e que
comparar dois quadros olhando pincelada por pincelada tambem da trabalho. se a
comparacao for cara demais, nao compensa, e melhor repintar. entao o truque real
nao e "comparar antes de pintar", e "achar um jeito de comparar que seja muito
mais barato que pintar".

e ai que entra o hash. em vez de guardar o quadro inteiro de ontem e comparar
imagem com imagem, o pintor anota um unico numero que resume o quadro. um numero
que muda se qualquer coisa no desenho mudar, e que continua igual se nada mudou.
de manha ele calcula o numero do quadro de hoje e compara com o de ontem. dois
numeros. se forem iguais, ele vai tomar cafe. a engine faz exatamente isso, e o
quadro e o que a gente chama de layer.

## a ideia: cada camada lembra o que desenhou

no plev a tela nao e uma coisa so. ela e empilhada em camadas, as layers. cada
layer tem a sua propria textura offscreen, os seus proprios buffers, e desenha a
sua propria lista de coisas. o fundo pode ser uma layer, um painel flutuante
pode ser outra, um menu que abre por cima pode ser uma terceira. isso esta
descrito no adr do sistema de layers (`kdb/adr/layer-system.md`), e o motivo de
existir camada separada e justamente esse: quando o menu anima, so a layer do
menu precisa ser redesenhada, o fundo fica parado.

cada layer guarda um numero, o hash da cena que ela desenhou da ultima vez. a
cada frame, antes de mandar qualquer coisa pra GPU, a engine reconstroi a
descricao da cena (a lista de retangulo, texto, sombra, o que for), calcula o
hash dessa lista nova, e compara com o numero antigo. igual quer dizer cena
identica. cena identica quer dizer: nao toca em nada.

a regra esta escrita seca em `kdb/mission/rules.md`, na secao de performance:

> dirty tracking via fxhasher per layer, no steady state: layer limpa = zero
> render pass, zero geometry, zero shaping

"steady state" e o nome chique pro pintor tomando cafe. e o estado em que nada
muda na tela. e a promessa dessa linha e forte: uma layer limpa custa zero
render pass, zero reconstrucao de geometria, zero shaping de texto. zero nao e
figura de linguagem aqui. e o numero de operacoes de GPU que acontecem. agora,
como uma comparacao de dois numeros vira esse zero.

## o codigo: um numero por no, um numero por camada

a unidade de desenho do plev e o `SceneNode`. um enum em
`crates/engine/src/compositor/scene.rs`. cada variante e uma coisa que da pra
desenhar: um `Rect`, um `RoundedRect`, um `Text`, uma `Shadow`, um `Path`. quando
voce manda a engine desenhar um retangulo, voce esta empurrando um
`SceneNode::Rect` pra dentro de uma layer.

o primeiro nivel do hash mora no proprio no. cada `SceneNode` sabe se reduzir a
um `u64`:

```rust
impl SceneNode {
    pub(crate) fn hash_u64(&self) -> u64 {
        let mut h = FxHasher::default();
        match self {
            SceneNode::Rect {
                x,
                y,
                w,
                h: rh,
                color,
            } => {
                0u8.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                w.to_bits().hash(&mut h);
                rh.to_bits().hash(&mut h);
                for c in color {
                    c.to_bits().hash(&mut h);
                }
            }
            // ... uma arm por variante, mesma forma: tag + campos
            SceneNode::Text { key, x, y, color } => {
                1u8.hash(&mut h);
                key.hash(&mut h);
                x.to_bits().hash(&mut h);
                y.to_bits().hash(&mut h);
                for c in color {
                    c.to_bits().hash(&mut h);
                }
            }
            // ...
        }
        h.finish()
    }
}
```

tres detalhes valem a leitura devagar.

o `0u8` e o `1u8` no comeco de cada arm sao a tag da variante. eles existem pra
que um `Rect` em certa posicao e um `Text` na mesma posicao nunca colidam por
acaso, mesmo que os campos numericos batam. a forma do no entra no hash.

o `to_bits()` em cada `f32` e o ponto que so faz sentido depois que voce ja
sofreu com float. em rust, `f32` nao implementa `Hash` nem `Eq`, e por um bom
motivo: `NaN != NaN`, e dois zeros (`+0.0` e `-0.0`) sao iguais por comparacao
mas tem bits diferentes. pra hash de cena a gente nao quer a semantica de
numero, quer a semantica de bytes. `to_bits()` pega o padrao de bits cru do
float e devolve um `u32`, que ai sim tem `Hash`. dois retangulos sao "a mesma
coisa" pro dirty tracking quando os bits batem, nao quando os valores sao
matematicamente iguais. e a escolha certa: se um `x` mudou de 100.0 pra
100.00001, o desenho mudou, e o hash tem que mudar junto.

o `FxHasher` e o terceiro detalhe, e ele vem de `rustc-hash 2.1` (a versao esta
no `Cargo.toml`, linha 68). nao e o hasher padrao da std. a std usa SipHash, que
e resistente a ataque de colisao, otimo pra `HashMap` que recebe chave de fora
pela rede. aqui a chave nao vem de fora, vem da nossa propria cena, e o que
importa e velocidade bruta sobre inteiro. a regra do projeto e explicita sobre
isso, ainda em `rules.md`: fxhashmap pra qualquer cache em hot path, nunca
hashmap padrao, porque siphash e 2 a 3x mais lento pra chave de inteiro. o
dirty tracking roda todo frame. todo frame e hot path.

o `TextNodeKey` que aparece dentro do `SceneNode::Text` merece uma nota. ele nao
guarda o texto e o resultado do shaping juntos, guarda a chave que identifica
unicamente um pedaco de texto pra ser desenhado: a string, o tamanho da fonte em
bits, a altura de linha em bits, a largura maxima, o peso, o letter spacing, a
familia. de novo tudo em bits pelo mesmo motivo. essa chave e o mesmo objeto que
o cache de shaping usa. e por isso que um texto que nao mudou nao re-shapeia: a
chave e identica, o hash da layer e identico, e o caminho que faria o
harfbuzz trabalhar nem e tocado.

o segundo nivel junta os nos numa layer. e quase anticlimaxico de tao curto, em
`crates/engine/src/compositor/layer/mod.rs`:

```rust
pub(crate) fn compute_hash(&self) -> u64 {
    let mut scene_hasher = FxHasher::default();
    for node in &self.nodes {
        node.hash_u64().hash(&mut scene_hasher);
    }
    scene_hasher.finish()
}

pub(crate) fn resolve_dirty(&mut self) {
    let hash = self.compute_hash();
    if hash != self.prev_hash {
        self.dirty = true;
        self.prev_hash = hash;
    }
}
```

`compute_hash` passa por cada no, pega o `u64` dele e joga no hasher da layer. a
ordem entra no resultado, porque ordem de desenho importa: um retangulo azul por
cima de um vermelho e diferente do vermelho por cima do azul. `resolve_dirty`
compara o numero de agora com o `prev_hash` guardado. se mudou, marca a layer
como `dirty` e atualiza o numero. se nao mudou, nao faz nada, e `dirty`
continua `false`.

note que `resolve_dirty` so liga o `dirty`, nunca desliga. quem zera o flag e o
comeco do proximo frame, depois que a layer ja foi (ou nao) redesenhada. essa
separacao e o que deixa a coisa correta sob resize e sob invalidacao externa,
volto nisso mais pra frente.

no topo de tudo, o `Compositor` decide se vale acordar o render loop:

```rust
pub fn needs_render(&self) -> bool {
    if self.invalidated || !self.sorted {
        return true;
    }
    self.layers
        .iter()
        .any(|l| l.dirty || l.compute_hash() != l.prev_hash)
}
```

essa funcao e o coracao do render on demand. se ninguem invalidou de fora, se a
ordem das layers nao mudou, e se nenhuma layer tem hash diferente do anterior,
ela devolve `false`. `false` quer dizer: nao redesenha esse frame. o app dorme.
nao tem busy loop queimando bateria pra repintar a mesma tela 120 vezes por
segundo.

## o que "limpa" economiza de verdade

vale rastrear o que acontece quando o hash bate, porque "zero" e uma afirmacao
que precisa de prova. o caminho esta em `Compositor::resolve_scene`, no
`mod.rs` do compositor:

```rust
for layer in &mut self.layers {
    layer.resolve_dirty();
    if layer.dirty {
        let culled = layer.build_geometry(viewport);
        self.stats.layers_redrawn += 1;
        self.stats.nodes_culled += culled;
    }
    // ...
}
```

o `build_geometry` so e chamado dentro do `if layer.dirty`. e ele que transforma
a lista de `SceneNode` em vertice e indice de verdade, faz culling do que esta
fora do viewport, monta os buffers. uma layer limpa pula esse bloco inteiro. a
geometria que ela ja tinha continua valida, ninguem reconstroi.

um passo acima, em `Compositor::resolve`, o upload pra GPU tem a mesma guarda:

```rust
for layer in &mut self.layers {
    if layer.dirty {
        layer.upload_quad_geometry(res.device, res.queue);
        layer.upload_sdf_geometry(res.device, res.queue);
        layer.upload_shadow_geometry(res.device, res.queue);
        // ...
    }
}
```

layer limpa, nenhum `upload_*`. os buffers na VRAM ficam intocados. e na hora de
compor a imagem final, o adr descreve o desfecho: a layer ja tem a textura
offscreen do frame passado pronta, entao a composicao e um unico draw call, um
triangulo de tela cheia que estampa a textura da layer. um draw por layer
visivel, e mais nada (`layer-system.md`, secao do composite pass).

ai esta o "zero" inteiro, desmontado em tres ausencias concretas. nenhum
`build_geometry`, e a geometria nao e reconstruida. nenhum `upload_*`, e a GPU
nao recebe byte novo. nenhum no de texto novo passando pela chave, e o shaping
nao roda. o que sobra e a textura que ja existia e um triangulo pra desenhar.
o adr ainda registra o custo de memoria desse desenho: a textura offscreen
ocupa cerca de 8mb por layer em 1920x1080 (`layer-system.md`, secao de
performance em steady state). e o preco de manter o quadro de ontem pendurado
pra poder nao repintar.

## o numero: 3.31 microssegundos para mil retangulos

eu nao gosto de afirmacao de performance sem um numero atras. o benchmark vive
em `crates/engine/benches/scene_build.rs`, grupo `dirty_tracking`, e a medicao
foi feita num macbook pro m4, rust 1.94.0, criterion 0.5
(`kdb/adr/benchmark-results.md`).

| benchmark | tempo | maquina |
|-----------|-------|---------|
| static_1000_rects (steady state) | 3.31 us | m4, macos |

3.31 microssegundos pra mil retangulos. pra ter escala: um frame a 60hz tem
16.667 microssegundos de orcamento, mil vezes esse valor. o custo de manter mil
retangulos estaveis na tela e uma fracao de meio por cento de um frame. e por
isso que o takeaway do doc fecha assim: cenas estaticas sao basicamente de graca
depois do frame 1.

agora a parte honesta, porque o livro inteiro se sustenta em ler a fonte e nao a
legenda. o `benchmark-results.md` rotula essa linha como "hash comparison only".
o codigo do benchmark conta uma historia um pouco diferente:

```rust
b.iter(|| {
    comp.begin_frame();
    for i in 0..1_000 {
        let f = i as f32;
        comp.draw_rect(f, f, 100.0, 50.0, [0.5, 0.5, 0.5, 1.0]);
    }
    black_box(&comp);
});
```

o loop medido faz `begin_frame` (que limpa os nos de todas as layers) e empurra
os mil retangulos de novo, identicos. ele nao chama `resolve_scene` nem
`compute_hash` dentro da medicao. ou seja, os 3.31us sao o custo de redescrever
a cena inteira do zero a cada frame, nao o custo de comparar dois hashes. a
comparacao em si e mais barata ainda. o rotulo "hash comparison only" no doc esta
impreciso, e marco isso explicitamente como nao confirmado.

mas o resultado conta a favor do desenho, nao contra. mesmo um app que joga
fora a cena toda e remonta tudo de novo todo frame, no estilo immediate mode,
gasta 3.31us pra mil retangulos. e quando o `compute_hash` roda em cima dessa
cena remontada e bate com o anterior, todo o resto (geometria, upload, shaping,
render pass) e cortado. o custo de descrever a cena fica, o custo de
materializar a cena some. e a descricao e quase de graca.

## por que assim, e nao de outro jeito

dirty tracking nao e ideia nova. retained mode existe ha decadas, e a
reconciliacao de arvore que o react popularizou e prima dessa logica: descrever
a UI de novo a cada mudanca, comparar com a anterior, aplicar so o diff. flutter
faz o seu, gpui faz o seu. o plev nao inventou o conceito. a pergunta
interessante e onde colocar a fronteira da comparacao, e e ai que a escolha
arquitetural aparece.

o plev compara no nivel da layer, com um hash da lista de nos. isso da uma
propriedade boa: a superficie da API parece immediate mode (voce chama
`draw_rect` todo frame, sem gerenciar handle de objeto retido), mas por baixo o
comportamento e retained (a layer guarda a textura e so refaz quando o hash
muda). voce escreve codigo simples de raciocinar, "desenho tudo que quero ver
agora", e a engine recupera a eficiencia do retained sem te cobrar a contabilidade
manual. essa dualidade e o ponto. a maioria das engines te faz escolher entre os
dois.

a fronteira da comparacao tambem e deliberada, e o `rules.md` registra os dois
lados dela. o compositor compara no nivel de layer, via `FxHasher`. mas existe um
segundo cache, por componente, que vive no `Component<L>` e nao no compositor:
campos `cached_nodes` e `needs_render`, com `invalidate()`. sao duas camadas de
"nao refaca o que nao mudou" em escalas diferentes. o componente decide se
re-roda o `render()` dele; o compositor decide se re-materializa a layer. nenhum
dos dois conhece o outro. um consumidor do plev nunca toca no compositor direto,
o `SceneNode` e o unico contrato. essa separacao e o que permite trocar o
mecanismo de dirty tracking de um lado sem quebrar o outro.

tem trade-off, e seria desonesto esconder. hash colide, em teoria. dois `u64`
diferentes de cena podem bater no mesmo numero e a engine acharia que nada mudou
quando algo mudou, deixando um frame velho na tela. na pratica a chance e
remota pra qualquer cena real e o custo de uma comparacao perfeita (guardar a
cena toda e comparar campo a campo) nao paga. e a velha troca de seguranca
absoluta por velocidade, feita de olho aberto. o `to_bits` traz o outro
asterisco: animar o `font_size` muda os bits do `TextNodeKey`, muda o hash, e
dispara re-shaping a cada frame. por isso uma das armadilhas em `rules.md` e
manter o `font_size` fixo durante animacao e animar posicao e opacity, nunca o
tamanho. o dirty tracking e tao bom quanto a estabilidade dos bits que voce
alimenta nele.

e tem o caso que o hash sozinho nao cobre, que e por que `needs_render` checa
`self.invalidated` antes de qualquer layer. nem toda mudanca visivel esta na
lista de nos. input chegou, a janela mudou de tamanho, o relogio da animacao
avancou: nada disso altera necessariamente a cena que voce descreve, mas todos
exigem um frame novo. o `invalidate()` e a valvula manual pra isso, e a
invariante do projeto e dura nesse ponto: toda mudanca de estado visivel tem que
invalidar. o hash automatiza o caso comum (a cena descreveu a si mesma diferente)
e o `invalidate` cobre o caso que a cena nao consegue ver sozinha. os dois juntos
e que fecham o render on demand sem deixar buraco.

## o que isso me ensinou

a parte que eu levei mais tempo pra internalizar nao foi o hash. foi que a
otimizacao de verdade aqui e estrutural, nao esperta. nao tem SIMD nem truque de
bit obscuro. tem um `u64` por layer e um `if`.
o ganho vem de ter desenhado o sistema pra que a pergunta "preciso refazer isso?"
seja barata de responder, e de ter colocado a fronteira da pergunta no lugar
certo. `less, but better` parece slogan ate voce ver virar 3.31us pra uma cena
que um motor ingenuo repintaria mil vezes por segundo sem pensar.

se eu fosse deixar uma coisa pra aurora ler disso aqui um dia: o codigo mais
rapido continua sendo o que voce nao executa, e a engenharia boa e em boa parte
arranjar as pecas pra poder nao executar. o resto desse capitulo, o `FxHasher`,
o `to_bits`, o `prev_hash`, e so o encanamento que torna essa preguica segura.

## rastros

adr e regras
- `kdb/adr/layer-system.md` (dirty tracking per-layer, premultiplied alpha,
  composite pass de triangulo de tela cheia; "performance em steady state",
  linhas 36-40; custo de ~8mb por layer em 1920x1080, linha 40)
- `kdb/adr/index.md:19` (linha-resumo do layer system: per-layer dirty tracking,
  offscreen textures, gpuvec compartilhado)
- `kdb/mission/rules.md:21` (regra de performance: dirty tracking via fxhasher
  per layer, steady state = zero render pass, zero geometry, zero shaping)
- `kdb/mission/rules.md:18` (fxhashmap em hot path, siphash 2-3x mais lento)
- `kdb/mission/rules.md:40` (armadilha: nao animar font_size, re-shaping jitter)

codigo (crate engine)
- `crates/engine/src/compositor/scene.rs:168-325` (`SceneNode::hash_u64`, arms
  por variante com tag + `to_bits`)
- `crates/engine/src/compositor/scene.rs:110-122` (`TextNodeKey`, chave do
  shaping em bits, compartilhada com o cache de texto)
- `crates/engine/src/compositor/layer/mod.rs:425-439` (`compute_hash`,
  `resolve_dirty`)
- `crates/engine/src/compositor/mod.rs:83-90` (`Compositor::needs_render`)
- `crates/engine/src/compositor/mod.rs:145-185` (`resolve_scene`, guarda
  `if layer.dirty` em torno de `build_geometry`)
- `crates/engine/src/compositor/mod.rs:106-138` (`resolve`, guarda em torno dos
  `upload_*`)
- `crates/engine/src/compositor/drawing.rs:61` (`draw_rect`, usado pelo bench)

benchmark
- `kdb/adr/benchmark-results.md:30` (static_1000_rects, 3.31 us, steady state)
- `kdb/adr/benchmark-results.md:56` (takeaway: cenas estaticas de graca apos o
  frame 1)
- `crates/engine/benches/scene_build.rs:68-89` (`bench_dirty_tracking`, o loop
  medido faz begin_frame + repush, nao chama resolve_scene)

versoes (conferidas contra o Cargo.toml)
- `Cargo.toml:68` rustc-hash 2.1 (`FxHasher`)
- `Cargo.toml:50` wgpu 28
- `Cargo.toml:70` web-time 1.1 (`Instant` usado em `resolve`)
- `Cargo.toml:23` edition 2024

nao confirmado
- o rotulo "hash comparison only" em `benchmark-results.md:30` nao bate com o
  loop medido em `scene_build.rs:78-85`, que mede begin_frame + repush de 1000
  rects e nao chama `resolve_scene`/`compute_hash` dentro da medicao. os 3.31us
  sao o custo de redescrever a cena, nao da comparacao de hash em si.
- commit de origem do dirty tracking nao rastreavel com seguranca: o historico
  git anterior foi parcialmente perdido (forgejo self-hosted deletado, ver
  `kdb/briefing/00-visao.md`). os arquivos do compositor estao hoje em `d29e80a`;
  o adr `layer-system.md` foi tocado por ultimo em `b08bce8` (rebrand phi -> plev).
- "~8mb por layer em 1920x1080" e afirmacao do adr, nao medida de forma
  independente neste capitulo.
