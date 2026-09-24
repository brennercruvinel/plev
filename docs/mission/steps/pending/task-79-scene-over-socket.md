---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-79
wave: trilha
depends: [task-61, task-66]
depends-optional: [task-63]
branch: task/79-scene-socket
---

# task-79: cena por socket

## objetivo

a UI roda num lugar e desenha noutro. o app no android manda a cena para o
desktop, que renderiza com o inspector por cima; ou o contrário, lógica
num servidor e cliente fino renderizando (server-driven UI no nível de
cena, não de JSON de widget). o urnaui explorando um corpus remoto sem
baixar nada é o caso óbvio.

## ligação com a tese

seção 27 (cena por socket), seção 2 (a cena é um vetor de nós puros,
hasheado por layer, gravável com delta).

## estado atual no código

- `SceneNode` é dado puro, `Clone + Debug + PartialEq`, sem serde
  (`crates/engine/src/compositor/scene.rs:6-7`).
- layers com hash: só as sujas precisariam viajar.
- o monster já sabe codificar cena com delta descoberto (modo B).
- depois da 61: transporte local e protocolo versionado para o agente.
  depois da 66: render sem janela.

## escopo

1. um transporte só: cena e semântica viajam pelo mesmo transporte da
   task-61, em canais separados, com o protocolo versionado. nenhum segundo
   protocolo paralelo.
2. codificação por frame: só as layers sujas, pelo hash; texto como
   `TextStyle` mais string (o receptor rasteriza com as mesmas fontes
   embutidas, determinismo da 77); imagem enviada uma vez por hash de
   conteúdo. avaliar reusar o codec de cena do monster em vez de serde.
3. visualizador no desktop que recebe e renderiza, com as ligações da
   task-59 por cima (inspector).
4. fase 2: input de volta (o visualizador manda eventos ou, melhor,
   requests semânticas pelo canal do agente).
5. rede: fora do local só com pareamento explícito e nunca execução
   remota; o ADR decide se esta task sai do localhost.

## fora de escopo

streaming de vídeo de pixels; multiusuário.

## dependências

61 e 66 integradas; 63 se o codec de cena do monster for o escolhido.

## superfície sob responsabilidade

o canal de cena no crate de protocolo (com aprovação, porque o protocolo é
da 61), o visualizador (exemplo ou app), serde ou codec para `SceneNode`
(mudança no engine, com aprovação), ADR
`docs/ADR/scene-travels-as-data.md`.

## interfaces que consome

protocolo e transporte (61), headless e renderer (66), ligações (59),
codec do monster (63).

## interfaces que pode alterar

`SceneNode` por derive de serialização, com aprovação.

## critérios de aceitação

- o showcase rodando num processo (ou no simulador iOS) espelhado num
  visualizador desktop com pixels iguais (hash de frame renderizado nos
  dois lados).
- banda medida por página, com e sem mudança.
- `./script/gate` verde.

## testes esperados

roundtrip da codificação de cena, delta só com layers sujas, espelho em
processo com dois compositores.

## riscos e perguntas abertas

- segurança: cena pode conter dado do usuário; nada sai da máquina sem
  pareamento explícito.

## commits esperados

1. `task-79: serialização de cena por layer suja`
2. `task-79: canal de cena no transporte da 61`
3. `task-79: visualizador com ligações por cima`
4. `task-79: input de volta como request semântica`

## pare e reporte se

- o protocolo da 61 não comportar um segundo canal sem versão nova;
- `SceneNode` precisar mudar além de derive.

## fase 1 (task-79a): a sonda

branch `task/79a-scene-socket`, base `8c4ad908`. worktree
`.claude/worktrees/agent`, target `target/agents/one`. a medição está em
`crates/rec/tests/scene_probe.rs` (ignorada; `cargo test -p rec --test
scene_probe -- --ignored --nocapture`), build de debug: os crates do
workspace em opt-level 0, as dependências em 2, então os tempos são teto.
164 frames: as 82 páginas da galeria nos temas core e hoff, a 1280x800
@1 (o viewport dos baselines do regress), cada uma encenada por
`showcase::app::stage_page`.

### (a) o protocolo

um servidor v1 de hoje recusa um pedido de cena como forma desconhecida,
com o id, e a conexão continua aberta (medido pelo `decode_client` e por
um `Client` num `AgentPort` de verdade, antes de qualquer mudança):

```text
-> {"v":1,"id":"7","type":"scene","window":"main"}
<- {"v":1,"id":"7","type":"error","error":"bad_message","reason":"unknown variant `scene`, expected one of `hello`, `observe`, `act`, `confirm`, `decline`"}
```

é o que `codec.rs:84-87` faz com um `type` que o `Ask` não conhece, e o
teste `a_broken_line_is_a_bad_message_with_the_reason` já fixa
(`tests/codec.rs:391-395`, o `teleport`). então o canal cabe na versão 1
como um pedido novo: nenhum cliente de hoje o manda (`plev observe`,
`plev act` e `--ask` não mudam), um cliente novo diante de um app de
antes lê a recusa e sabe que ali não há cena. o `act` já responde várias
linhas com o mesmo id (uma por estágio, `proto.rs:212-217`); a cena
responde do mesmo jeito, um frame por linha, até a conexão acabar. sem
versão 2, sem parada por aqui.

o limite de linha: `MAX_LINE` (1 MiB, `port.rs:48`) vale para o que o
cliente manda (`port.rs:470-487`); o que o app escreve não tem limite
hoje (`port.rs:446-454`, e o `Client` lê com `read_line`,
`client.rs:72`). o frame medido, codificado em binário e levado como
string base64 numa linha JSON:

```text
frame inteiro, sem os pixels de imagem (164 frames):
  mínimo 40 372 bytes, média 50 829, máximo 178 112 (Icons / core) = 17,0% de MAX_LINE
imagens: 2 nós em toda a galeria (Card, nos dois temas), 380 160 bytes de RGBA
  = 506 880 em base64 (48% de MAX_LINE)
um frame de câmera (task-76 item 10, 640x360 RGBA a 15 fps): 921 600 bytes,
  1 228 800 em base64 = 1,17 x MAX_LINE; 13,8 MB/s cru, 18,4 MB/s em base64
```

proposta de framing: a mesma conexão, linhas JSON v1, o frame binário
em base64 numa linha `scene`. uma imagem vai uma vez por conteúdo, em
linhas próprias antes do frame que a usa, cortada em pedaços que deixam
cada linha abaixo de `MAX_LINE`, então o mesmo limite vale nos dois
sentidos e o cliente pode ler com teto. o custo é o base64 (33% a mais,
só no que viaja). as outras duas saídas ficam de fora: um segundo socket
é um segundo caminho, com outra regra de acesso, o segundo protocolo que
a tarefa proíbe; trocar a conexão para binário depois do `hello` mistura
na mesma conexão respostas JSON e frames binários, o que pede um
envelope binário para tudo depois da troca, que é framing de versão 2, e
tira o `plev`/`socat` da leitura. com frames de até 178 KB e deltas de
centenas de bytes, 33% não pagam essa conta.

### (b) o codec

o monster não serve para o espelho. o IR v0 não tem nó para
`PushClip`, `PopClip`, `BackdropBlur` e `Shadow` (`raise.rs:163-177`,
`raise.rs:286-289`) e o `raise_scene` pula cada um (`raise.rs:307-310`):
4 a 12 clips por frame em toda página, até 63 sombras (Button / core), o
vidro das páginas Effects, Bubble, Message, Message Scroller e Dock. e o
que ele carrega volta quantizado (twips e rgba8, `quant.rs`; o path com a
cor do primeiro vértice, `rec/src/payload.rs:15-16`). pixel igual pelo
monster é impossível por construção.

o codec próprio escreve cada nó como ele é: uma tag e os campos, `f32`
como bits, o texto como a `TextNodeKey` (string e tipografia) e o lugar,
a imagem como o lugar no atlas e a chave de conteúdo, o path como a
chave de conteúdo (a geometria vai uma vez), clip, vidro e sombra como
os outros. por frame inteiro:

```text
                         mínimo    média   máximo
nós por frame                52       79      295
layers por frame              4     4,01        5 (Git)
próprio, bytes           30 277   38 120  133 582
  dos quais paths        28 076   34 312  123 776  (ícones do sidebar; uma vez por conteúdo)
  sem os paths            2 201    3 809   15 689
próprio, ms por frame      0,02     0,03     0,11
monster + assets, bytes  13 253   16 846   56 623
monster, ms por frame      0,11     0,20     1,08  (raise, discovery, encode)
```

e um frame com uma mudança, numa página com texto, uma com imagem e uma
com vidro (o ponteiro sobre um controle; no Effects nenhum hover muda o
frame, então um scroll de 40 px):

```text
                          layer suja inteira   patch da layer suja      monster (2o frame)
Todo / core, hover        1 798 (1 de 4)       195 (1 nó escrito)       63
Card / hoff, hover        8 140 (1 de 4)       272 (2 nós)              83
Effects / hoff, scroll   11 756 (1 de 4)     4 812 (78 nós)          4 047
frame sem mudança            96 bytes de cabeçalho; nada precisa ir
```

o patch é a layer suja escrita contra a versão que o receptor já tem:
quantos nós do começo e do fim ficam iguais, e os do meio escritos. com
ele o próprio fica no tamanho do monster (3x no pior caso medido), sem
perder nó nem bit, e a um décimo do custo de CPU. escolha: codec próprio
por layer suja, no `rec`, que já depende do `engine` e do `monster`, já
guarda asset uma vez por conteúdo (`visual.rs`) e já testa contra a
galeria (o `showcase` é dev-dependency). o `wire` não pode depender do
`engine` sem mexer no `Cargo.lock`, e nem deve (o `plev` e o wasm ficam
leves): o transporte leva o frame como bytes opacos, com o recorte por
layer e por asset que o codec entrega.

o `SceneNode` não muda: todo campo é público (`scene.rs:7-108`), a
`TextNodeKey` também (`scene.rs:111-122`), o `ImageHandle`
(`gpu/image.rs:43-49`) e o `TessellatedPath` (`path/types.rs:14-18`), e o
`PartialEq` que o roundtrip compara já existe (`scene.rs:6`). o receptor
monta a `TextNodeKey` pelo literal, nunca por `TextNodeKey::new`, que
troca o texto pelo do fonte quando o hot reload está ligado
(`scene.rs:124-131`). o hash de layer do engine é `pub(crate)`
(`layer/mod.rs:426`); o codec usa o seu, FNV-1a sobre os bytes da layer,
como o regress (`examples/regress/hash.rs`).

### (c) as imagens

aqui a sonda para. o `ImageHandle` é lugar no atlas, tamanho e um id
(`gpu/image.rs:43-49`); os pixels ficam no `ImageStore`, que desde a
task-76 guarda os de cada upload (`gpu/image.rs:98-108`, `126-129`), mas
nada fora do engine os lê: o store é um `static` privado
(`gpu/image.rs:368`), `uploads` é campo privado, e nenhuma função pública
devolve pixels (as públicas são `gpu/image.rs:143, 162, 189, 209, 223,
344, 350, 355` e as globais `371, 376, 386, 392`). é a mesma parede que o
gravador achou (`rec/src/payload.rs:17-19`, "a recorder cannot read them
back from a handle today") e que o svg marca (`headless/svg/mod.rs:314`).
a imagem de fluxo da câmera é a mesma coisa: a página troca os pixels do
slot com `replace_image_rgba` (`view/apps/capture/mod.rs:318-326`), o
handle volta com o mesmo lugar e um id novo (`gpu/image.rs:217-251`), e
os pixels ficam só no store.

### (d) as ligações

o visualizador precisa de onde cada controle aparece no frame: o
`SemId` e o rect visível (o rect dentro do clip), que é
`Compositor::visible_bindings()` (`compositor/binding.rs:189-192`), a
mesma lista que o port já serve como `Bound` (`proto.rs:292-298`, o
`Host::bounds` do showcase em `view/dispatch/agent.rs`). a ligação mora
ao lado da cena, fora de todo hash de layer (`binding.rs:1-14`), então
viaja no cabeçalho do frame, só quando muda. medido: nenhuma na maior
parte da galeria; Todo 267 bytes, Toggle Group 267, Surface 1 324,
Silences 615, Files 147, Capture 199, Git 77 (id e quatro `f32` por
controle). o visualizador desenha as ligações numa layer própria, acima
das espelhadas; o hash comparado com o do app é o do frame sem ela.

### parada: os pixels de uma imagem

problema: sem ler os pixels de um `ImageHandle` no app, a imagem não
viaja, e o espelho desenha um buraco onde o app desenha a imagem (o
Card, nos dois temas; o Capture com a câmera ligada). ler exige mudar
`crates/engine/src/gpu/image.rs`, que está fora da lista.

opções:

1. um leitor no store, só leitura: `ImageStore::pixels(&self, &ImageHandle)
   -> Option<&[u8]>` (a entrada no lugar do handle, com o tamanho dele)
   e a global `image_pixels(&ImageHandle) -> Option<Vec<u8>>` (cópia sob
   o lock), mais a reexportação em `gpu/mod.rs`. umas 25 linhas, nenhum
   comportamento muda, nenhuma memória a mais (os pixels já estão lá). o
   app manda cada imagem uma vez por hash dos pixels; uma imagem de fluxo
   (mesmo lugar, id novo) vira, no receptor, um slot de fluxo
   (`stream_image_rgba` uma vez, `replace_image_rgba` depois), para o
   atlas dele não encher como na sonda da task-76 (244 frames enchem
   8192x8192). o mesmo leitor fecha depois o buraco do gravador.
2. sem mudar o engine: a imagem viaja com o tamanho só e o espelho não
   a desenha (como o gravador hoje); pixel igual só nas páginas sem
   imagem, que são todas menos o Card (e o Capture com câmera).
3. o app registra os pixels onde carrega: exige `src/view`
   (`components/card.rs:34, 51`, `apps/silences/worker.rs:274, 477`,
   `apps/capture/mod.rs:318-326`) e o `comps`, proibidos, e perde o que
   outros crates carregam (`urnaui/src/view/explorer.rs:331`).

impacto: com a 2, a fase 1 entrega o canal, o codec, o visualizador e a
banda, mas não "imagem uma vez por conteúdo, com os pixels"; o espelho
em processo ainda bateria no Card, porque os dois compositores dividem o
store global, e isso não prova que o pixel viajou.

recomendação: a 1, com as condições que o orquestrador quiser (como na
task-76 item 10): só leitura, testada no `image.rs` (imagem fixa, fluxo
depois de um `replace`, handle de outro tamanho ou lugar dá `None`).

decisão necessária: liberar `gpu/image.rs` e a linha de `gpu/mod.rs`
para o leitor (opção 1), ou seguir sem pixels (opção 2).
