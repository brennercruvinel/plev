---
id: task-79/phase-79a-probe
title: "phase 1 (task-79a): the probe"
project: plev
type: section
parent: task-79
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [agents, recording, performance]
relations: [task-76, adr-0062, adr-0065]
evidence: [91be2cd6, 73f08fbc, 7728da47, 5009b657, ac0f9c22, f45bc235, 853b885b, aef62a07, ccb6dc31, 697ff1f2, 22489539, 575bc605, d210a7c2, 0bdef89f]
source: docs/log/steps/task-79-scene-over-socket.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 1 (task-79a): the probe

The phase 1 probe measured the cost of sending each page's scene through the socket before choosing the design: the channel fits in version 1 of the protocol as a new request, the per-dirty-layer codec stays in `rec`, and the fixed image gains pixel reading in the engine, read-only. The phase delivered the mirror in-process and between two processes, with the bindings on top, the bandwidth measured per page, and a privacy review that took the camera out of the channel and made the scene available only on the person's request. The point is to keep the scene as data, not pixel streaming or widget JSON, which is what the task's goal calls for.

branch `task/79a-scene-socket`, base `8c4ad908`. worktree
`.claude/worktrees/agent`, target `target/agents/one`. a medição está em
`apps/uigallery/tests/scene_probe.rs` (ignorada; `cargo test -p record --test
scene_probe -- --ignored --nocapture`), build de debug: os crates do
workspace em opt-level 0, as dependências em 2, então os tempos são teto.
164 frames: as 82 páginas da galeria nos temas core e hoff, a 1280x800
@1 (o viewport dos baselines do regress), cada uma encenada por
`showcase::app::stage_page`.

## (a) o protocolo

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

## (b) o codec

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

## (c) as imagens

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

## (d) as ligações

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

## parada: os pixels de uma imagem

problema: sem ler os pixels de um `ImageHandle` no app, a imagem não
viaja, e o espelho desenha um buraco onde o app desenha a imagem (o
Card, nos dois temas; o Capture com a câmera ligada). ler exige mudar
`rust/engine/src/drawing/gpu/image.rs`, que está fora da lista.

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

## handoff da fase 1 (task-79a)

branch `task/79a-scene-socket`, base `8c4ad908`. worktree
`.claude/worktrees/agent`, target `target/agents/one`. a rodada parou uma
vez, depois da sonda (a seção "parada: os pixels de uma imagem" acima); o
orquestrador liberou a opção 1 (`gpu/image.rs` e a linha de `gpu/mod.rs`)
com cinco condições, e a rodada seguiu. a revisão achou depois um
problema de privacidade (o canal mostrava a um agente o que o contrato
proíbe: os frames da câmera, e tudo o que a redação não vê); a correção
são os cinco commits a partir de `697ff1f2` (seção "a revisão de
privacidade"). commits, em ordem:

- `91be2cd6` task-79: the scene channel probe
- `73f08fbc` task-79: the image store reads its pixels back
- `7728da47` task-79: the scene codec, layer by dirty layer
- `5009b657` task-79: the scene channel on the agent transport
- `ac0f9c22` task-79: the showcase offers its windows' scenes
- `f45bc235` task-79: the mirror, headless or in a window, with the bindings over it
- `853b885b` task-79: the mirror's pixels, in process and between two processes, and the band
- `aef62a07` task-79: base64 a quad at a time, and the offer's cost measured
- `ccb6dc31` task-79: ADR and the phase 1 handoff
- `697ff1f2` task-79: the image store tells a stream from a fixed image
- `22489539` task-79: the scene is shared only when the person asks, under a name
- `575bc605` task-79: a camera's frames never cross
- `d210a7c2` task-79: the shell says who watches
- `0bdef89f` task-79: the windowed mirror says a refusal
- (este) task-79: the privacy section of the ADR, and the handoff after the review

nenhuma janela foi aberta: o app do teste de socket é um processo filho
headless, o visualizador também; o `--show` do exemplo compila e não foi
rodado. nenhum processo ficou para trás (o filho sai pelo `quit`, e o
`Drop` do teste mata só o próprio filho pelo handle dele).

### a revisão de privacidade

todo mundo que conecta no port é `Actor::Agent`, pelo `hello`, e o
contrato decide o que um agente vê: o snapshot guarda só o comprimento
de um segredo (`sem::redact`), e o ADR
`capture-is-a-job-the-person-turns-on` tira os frames da câmera e as
amostras do microfone de todo agente. a cena carrega tudo o que a janela
desenha, que a redação nunca vê. o que mudou:

1. opt-in separado, da pessoa: o canal de cena só existe se quem lançou
   o app pedir, `PLEV_MIRROR=1` além do `PLEV_AGENT=1` (build de debug;
   `AgentPort::from_env` lê, `wire::mirror_enabled`), ou se um app que
   abre o próprio port chamar `share_scenes`. sem isso, um `scene` é
   recusado com um código próprio, `scene_not_shared` (um
   `ProtoError::SceneNotShared` novo, não `bad_message`), e a razão diz o
   que a pessoa faz: "this app's person did not share its windows: a
   scene is shown to a watcher only when the person who launches the app
   asks for it, with PLEV_MIRROR=1 beside PLEV_AGENT=1 (a debug build);
   observe and act work without it". `plev observe`, `plev act` e
   `--ask` não mudam.
2. imagem de fluxo nunca cruza: uma imagem de fluxo (a câmera) viaja
   como tamanho e lugar, sem pixels e sem id, com ou sem `PLEV_MIRROR`,
   sob uma chave que os frames dela dividem, então um frame novo da
   câmera não move layer nenhuma e não manda nada. o visualizador desenha
   no lugar uma caixa escura com a borda e "live image, not mirrored"
   (`rec::scene::LIVE_LABEL`). sem override nesta fase. a imagem fixa
   (a da Card) segue cruzando. para o codec saber o que é fluxo, o store
   ganhou `ImageStore::is_stream` e `image_is_stream` (commit
   `697ff1f2`): só leitura, no `gpu/image.rs` liberado e na mesma linha
   de reexportação do `gpu/mod.rs`, testado no `image.rs`. **é uma
   função a mais que a API liberada (que eram `pixels` e
   `image_pixels`)**: sem ela o primeiro frame da câmera cruzaria antes
   de o fluxo ser reconhecido pelo segundo id no mesmo lugar. o
   orquestrador decide se a mantém.
3. quem assiste fica registrado: o watch vem depois do `hello` (antes,
   `not_enabled`); o servidor guarda quem começou e quem terminou de
   assistir qual janela (`take_scene_watches`, um watch cortado entre
   eles), e o shell loga "scene: claude watches main" e "scene: claude no
   longer watches main". o watch que começa chega ao host como observação
   do agente (`Host::observed`, na revisão que o app projeta), então a
   gravação (`PLEV_RECORD=1`) guarda o `observed` daquele agente, sem
   mudar nada em `src/view`.
4. testes: sem `PLEV_MIRROR` o pedido é recusado com o código novo (em
   processo, pelo socket, e com o ambiente lido de verdade num processo
   filho: sem a variável, `0` e `true` recusam, `1` assiste); com a
   câmera falsa ligada, nenhum byte dos frames do `FakeCapture` está na
   saída inteira do watch (as partes binárias decodificadas e as linhas
   em base64, nas quatro fases de um pixel de 4 bytes nos grupos de 3), e
   o espelho difere do app só dentro da caixa da câmera; a Card segue com
   hash igual.
5. o ADR ganhou a seção de privacidade (abaixo, na linha do índice
   também).
6. o roteiro do brenner usa `PLEV_AGENT=1 PLEV_MIRROR=1`.

a condição 3 da liberação (o receptor transforma a imagem de fluxo num
slot de fluxo) ficou sem objeto: nenhum pixel de fluxo chega mais, e o
`Mirror` não faz slot nenhum. o código dela está no `7728da47`, para
quando uma fase com override quiser.

### o acessor de pixels e as cinco condições

`ImageStore::pixels(&ImageHandle) -> Option<&[u8]>` e
`image_pixels(&ImageHandle) -> Option<Vec<u8>>` (`gpu/image.rs`, 604
linhas de produção com o `is_stream`; a reexportação em `gpu/mod.rs`).

1. nada muda no store: nenhum campo, nenhum comportamento, nenhuma
   memória; o `git diff` do `image.rs` só acrescenta, e nenhum teste que
   existia mudou.
2. testes no `image.rs`: `a_fixed_image_reads_back_its_pixels` (e o store
   igual depois de ler), `a_stream_reads_back_its_latest_frame` (o último
   frame por qualquer handle do fluxo, transparente antes do primeiro
   `replace`), `a_handle_the_store_does_not_hold_reads_nothing` (outro
   tamanho, outro lugar, store vazio) e, da revisão,
   `a_stream_is_told_from_a_fixed_image`.
3. sem objeto depois da revisão (acima).
4. a prova é pelo socket entre dois processos (abaixo), com a Card
   (imagens) e páginas sem imagem, hash de frame igual dos dois lados; a
   câmera pelo `FakeCapture`, que agora prova o contrário: nada cruza.
5. o mesmo acessor fecha a lacuna do gravador: `rec/src/payload.rs`
   escreve a imagem com `has_pixels = 0` porque não havia como ler os
   pixels de um handle; com `image_pixels` há, e o replay desenharia as
   imagens (as fixas: a regra da câmera vale para ele também). não foi
   fechada aqui (dito no ADR).

### o canal

um pedido novo da versão 1, na conexão do agente, sem segundo protocolo:

```text
-> {"v":1,"id":"1","type":"hello","name":"mirror"}
-> {"v":1,"id":"9","type":"scene","window":"main"}
<- {"v":1,"id":"9","type":"scene_blob","key":"8a1f...","at":0,"total":24012,"bytes":"..."}
<- {"v":1,"id":"9","type":"scene","seq":1,"head":"...","layers":[{"key":0,"hash":"3c9e...","body":"..."},...],"bounds":[...]}
<- {"v":1,"id":"9","type":"scene","seq":2,"layers":[{"key":0,"hash":"77d0...","patch":"..."},{"key":1,"hash":"51aa..."}]}

sem PLEV_MIRROR=1:
<- {"v":1,"id":"9","type":"error","error":"scene_not_shared","reason":"this app's person did not share its windows: ..."}
```

- `wire::scene` (puro, compila no wasm): o `Ask::Scene`, os `Reply::Scene`
  e `Reply::SceneBlob`, o `Offer` que o app entrega e o estado de cada
  watch no `Server` (o hash de cada layer, os assets e os bindings que
  mandou). um watch por conexão, depois do `hello`, janela nomeada por 1 a
  32 de `[A-Za-z0-9_-]`, `main` por padrão. o que não mudou não vai: layer
  com hash igual vai sem corpo, o patch quando o watch tem a base, o
  corpo senão, o asset que o watch não tem, head e bindings quando mudam;
  frame sem mudança não manda nada.
- base64 nas partes binárias (`wire::b64`, próprio, sem dependência). um
  asset, e um corpo que não caberia na linha do frame, vão antes em
  linhas `scene_blob` de até `SCENE_PART` (384 KiB), toda linha abaixo de
  `MAX_LINE`. o `MAX_LINE` mudou de casa (`port.rs` para `scene.rs`) e
  segue exportado como `wire::MAX_LINE`; o `Traffic` foi para um módulo
  próprio (`traffic.rs`), o `server.rs` fica com o protocolo.
- o port (`port.rs`, 623 linhas de produção): `offer_scene`, `watching`,
  `share_scenes` e `take_scene_watches` no fio do app, sem esperar
  socket; cada conexão conta os bytes na fila, e um watch que passa de
  `SCENE_QUEUE` (16 MiB) é cortado. as respostas de sempre não olham esse
  teto: o contrato dos clientes de hoje não muda.
- `SceneReader` junta as linhas do lado de quem assiste.
- o showcase (`src/app/mirror.rs`, módulo novo; `app.rs` 603, `pace.rs`
  e `access.rs` com uma chamada cada): com o port ligado e a cena
  compartilhada, o shell escreve o frame de uma janela (`main`, ou `page`
  para a janela própria da página) só se alguém assiste, depois de cada
  frame desenhado nela e depois de cada drain do port, então um watch que
  chega recebe o frame que está na tela sem esperar o próximo; e loga
  quem começa e quem termina de assistir.

### o codec

`rec::scene` (o `rec` já dependia do `engine` e do `monster`, e testa
contra a galeria):

- `Sender` escreve cada layer em ordem de composição: propriedades e
  todo nó como é (`f32` como bits, texto como `TextNodeKey`, clip, vidro
  e sombra como nó), FNV-1a 64 como hash, e um patch contra o frame
  anterior (quantos nós do começo e do fim ficam, os do meio escritos)
  quando é menor. imagem e path viajam como asset com chave; cada asset
  é montado uma vez enquanto frames seguidos o referenciam; a imagem de
  fluxo, só como tamanho e lugar.
- `Mirror` lê estrito (entrada cortada ou quebrada é `SceneError`, nunca
  pânico), confere o corpo pelo hash e o patch pelo que ele produz, solta
  o asset que o frame não referencia (a mesma regra do servidor, então os
  dois lados concordam sem dizer), e desenha num compositor próprio com as
  mesmas layers, ordem e propriedades; no lugar da imagem de fluxo, a
  caixa.
- o `SceneNode` não mudou; a `TextNodeKey` volta pelo literal, nunca por
  `TextNodeKey::new` (hot reload).

### o visualizador

`showcase::app::mirror::Viewer` (junta, aplica, desenha, e com `bindings`
contorna e nomeia cada controle numa layer acima de todas) e
`apps/mirroring/src` (diz `hello` como "mirror"; headless
por padrão, uma linha por frame com bytes, layers mandadas e o hash dos
pixels; `--show` numa janela). rodado headless contra o app filho do
teste: o hash do frame do exemplo foi o do app (`258ff798e84965ad` na
primeira página, `26849f9186861eb2` no Todo sem ligações), e o png com
ligações mostra o contorno e o nome de `todo`, `todo.input`,
`todo.item[1..3]` e dos três filtros.

### os testes e as páginas espelhadas

- `engine`: 4 no `image.rs` (os 3 do acessor e o `is_stream`).
- `rec`: 8 no `scene/tests.rs` (todo tipo de nó de volta, texto com toda
  a chave, delta de uma layer suja como patch, asset montado uma vez, o
  receptor soltando o que não desenha, recusas, entrada cortada e
  virada byte a byte, e a imagem de fluxo: cinco frames viajam uma vez,
  como a caixa, sem nenhum byte deles em asset ou corpo, sem mover a
  layer) e `tests/scene.rs` (as 82 páginas nos dois temas, nó por nó,
  sem GPU).
- `wire`: 3 do base64, 12 no `tests/scene.rs` (as formas fixadas, bytes
  quebrados recusados, inteiro e depois só o que mudou, asset uma vez,
  partes abaixo do limite, corpo grande como blob, um watch por conexão
  e o nome da janela, o leitor recusando fora de ordem, pelo socket ao
  lado de um `observe`, o watch que não lê cortado sem o app esperar e
  com o fim dito, a recusa `scene_not_shared` em processo e pelo socket,
  o watch antes do `hello` recusado e depois dele dito ao app e ao host)
  e 2 no `tests/env.rs` (`PLEV_MIRROR` lido de verdade num filho); o
  roundtrip do `codec.rs` ganhou as mensagens novas e o erro novo.
- `showcase`: `app::mirror::tests` (a linha do log; o shell com janela
  nua e uma gravação oferece o frame da janela principal ao watch depois
  do drain, nada num frame igual, o patch do hover, e a gravação guarda o
  watch como observação do agente) e `tests/mirror.rs`:
  - em processo, dois compositores: as 82 páginas nos temas core e hoff
    (164 frames), uma view percorrendo a galeria e um visualizador
    seguindo por um `Server`, cada página um delta da anterior: os bytes
    do frame espelhado iguais aos do app. porque todas: é barato (1,9 s)
    e pega todo tipo de nó que a galeria desenha. as imagens aqui não
    provam nada (o store é um só no processo).
  - entre dois processos, pelo socket: o binário do teste roda a si mesmo
    como o app (`app_child`: `ShowcaseView` desenhado por um
    `WindowHost::headless`, o port aberto e a cena compartilhada, as
    ofertas como o shell faz) e o teste é o visualizador, com `hello`.
    hash igual no frame que estava na tela quando o watch chegou (Theme),
    na Card/hoff (as duas imagens cruzaram de um store para o outro), no
    Todo/core (e no hover, uma layer, patch), no Effects/hoff (vidro) e
    no Capture/core com a câmera desligada. com a câmera falsa ligada:
    nenhum byte dos frames do fake na saída do watch, os frames da câmera
    não mandam nada (300 ms de silêncio cada), e o espelho difere do app
    só dentro da caixa `capture.frame`. porque essas: imagem fixa, texto
    com ligações e um delta, vidro, e fluxo. um frame redesenhado sem
    mudança não manda nada.
  - as ligações por cima: o frame com elas só difere do do app perto dos
    controles, e o contorno aparece em todo controle ligado.
  - `band` (ignorado): a banda por página, abaixo.

### a banda

bytes no fio de um watch (cada linha como o port escreve, com o `\n`),
1280x800 @1. por página (`cargo test -p uigallery --test mirror band --
--ignored --nocapture`, 164 frames):

```text
primeiro frame a um watch que chega: mínimo 41 899, mediana 43 811, média 58 863, máximo 561 420
  (os paths dos ícones do sidebar são uns 37 KB em base64 de cada página simples;
   Card 561 420, dos quais 506 880 são os pixels das duas imagens; Icons 183 687)
um frame com uma mudança (o ponteiro sobre um controle, senão um scroll de 40 px):
  mínimo 243, mediana 347, média 1 740, máximo 32 288 (Chart, o scroll move os paths) em 100 frames;
  64 frames sem mudança possível por hover nem scroll
um frame sem mudança: 0

amostras:              primeiro   uma mudança
Todo / core              48 328   319 (hover todo.filter[completed])
Card / hoff             561 384   423 (hover)
Effects / hoff           57 565   15 873 (scroll 40 px, o vidro redesenha)
Typography / core        47 119   5 059 (scroll 40 px)
Button / core           115 890   623 (hover)
```

pelo socket entre os dois processos, com o custo no fio do app (build de
debug, os crates do workspace em opt-level 0):

```text
Card / hoff, a página chega:          561 269 bytes em 19 linhas, oferecido em 9,2 ms
Todo / core, a página chega:           48 225 bytes em 19 linhas, 2,1 ms
Effects / hoff, a página chega:        57 462 bytes em 19 linhas, 2,5 ms
Todo / core, um hover:                    318 bytes em 1 linha, 0,16 ms
redesenho sem mudança:                      0 bytes, 0,09 a 0,14 ms
Capture / core, a câmera liga:          1 433 bytes em 2 linhas (a caixa no lugar), 0,13 ms
Capture / core, um frame da câmera:         0 bytes, 0,07 a 0,27 ms
```

antes da revisão o frame da câmera custava 1 229 410 bytes em 4 linhas e
19 ms do fio do app (38 ms com o primeiro base64, que o `aef62a07`
reescreveu); agora a câmera não cruza e não custa nada no fio. o base64
mais rápido segue valendo para as imagens fixas (as duas da Card, 9 ms
da oferta em debug).

### o gate e o regress

- antes (base `8c4ad908`, `target/agents/gate-79a-before.txt`): verde,
  2 781 passados, 0 falhas, 20 ignorados.
- depois da primeira entrega (`target/agents/gate-79a.txt`): verde,
  2 810 passados, 0 falhas, 24 ignorados.
- depois da revisão de privacidade (`target/agents/gate-79a-privacy.txt`):
  verde, 2 816 passados, 0 falhas, 24 ignorados. +35 testes sobre a base:
  4 do `image.rs`, 9 do `rec`, 17 do `wire` (base64, `scene.rs`, `env.rs`)
  e 5 do showcase; +4 ignorados: as duas medições da sonda, o
  `app_child` (o lado do app no teste de socket, rodado por ele) e o
  `band`. pernas urnaui e wasm rodaram. `df -h /`: 15 GB livres.
- `regress check` (`target/agents/regress-79a-privacy.txt`): 163
  baselines, 163 iguais.
- linhas de produção dos arquivos tocados: `server.rs` 627, `port.rs`
  623, `image.rs` 604, `app.rs` 603, `rec/scene/node.rs` 502, os outros
  abaixo de 430; o guard de higiene verde.

### a linha do ADR para o índice

```text
| 2026-09-24 | [scene-travels-as-data](scene-travels-as-data.md) | proposed | task-79 fase 1: a cena de uma janela viaja pela conexão do agente como um pedido novo da v1 (`{"type":"scene","window":"main"}` depois do `hello`, recusado como `bad_message` por um app de antes), só quando a pessoa que lança o app pede (`PLEV_MIRROR=1` além do `PLEV_AGENT=1`; sem isso, `scene_not_shared`, que diz como ligar); frames como respostas ao mesmo id: o primeiro inteiro, depois só as layers com hash novo (o patch quando o watch tem a base), os assets que o watch não tem, head e bindings quando mudam, nada num frame igual; linhas JSON com base64, asset e corpo grande em `scene_blob` de até 384 KiB, toda linha abaixo do `MAX_LINE`; o app nunca espera, watch com mais de 16 MiB na fila é cortado; codec próprio por layer suja no `rec` (todo nó como é, `f32` em bits, texto como `TextNodeKey`, clip, vidro e sombra, que o monster não carrega), imagem fixa uma vez por conteúdo com os pixels (`image_pixels`, só leitura, liberado no engine), imagem de fluxo nunca (a câmera vai como tamanho e lugar, o visualizador desenha uma caixa que diz que não espelha); o app loga quem assiste e a gravação guarda o watch como observação do agente; a lacuna da redação registrada (um campo que desenhasse um segredo chegaria a quem assiste; nenhum desenha, a fase 2 fecha antes de sair do localhost); visualizador headless ou em janela com as ligações por cima; pixel igual nas 164 páginas em processo e pelo socket entre dois processos (Card, Todo, Effects), a câmera diferindo só na caixa; local; fase 2 é input como request semântica |
```

### o roteiro para o brenner ver o espelho numa janela

num checkout desta branch, dois terminais:

```text
# 1. o app, com o port ligado e a cena compartilhada (build de debug)
PLEV_AGENT=1 PLEV_MIRROR=1 cargo run -p uigallery

# 2. o espelho numa janela, a janela principal do app, com as ligações
cargo run -p mirroring -- --show
```

- o app loga `scene: mirror watches main` quando o espelho começa, e
  `scene: mirror no longer watches main` quando ele fecha.
- `b` liga e desliga as ligações; Escape ou fechar a janela encerra o
  espelho (o app segue).
- a janela própria da página: `D` no app abre, e
  `cargo run -p mirroring -- --show --window page`.
- com mais de um app no ar, `--app showcase` (o nome) ou o caminho do
  socket em `$TMPDIR/plev/`.
- sem janela: `cargo run -p mirroring -- --frames 3 --png
  /tmp/espelho.png` imprime bytes e hash de três frames e grava o último.
- sem o `PLEV_MIRROR=1` o espelho sai com a recusa:
  `mirror: scene not shared: this app's person did not share its windows: ...`.
- o que olhar: navegue, passe o ponteiro, troque o tema (`T`); na página
  Capture, ligue a câmera: no espelho a caixa diz "live image, not
  mirrored". a janela do espelho abre no tamanho lógico do app; se a
  escala da tela for outra, o desenho é o mesmo em pixels lógicos.

### o que fica para a fase 2

- input de volta como request semântica pelo canal do agente: o
  visualizador acha o controle sob o clique pelas ligações que já recebe
  e manda um `act` sobre ele, apoiado numa observação, como qualquer
  agente; nunca evento cru nem coordenada.
- a lacuna da redação: um nó que desenhe um segredo marcado, e viajando
  mascarado, antes de qualquer coisa sair do localhost.
- fora do localhost: pareamento explícito, transporte cifrado, nunca
  execução remota (o ADR decide que esta fase não sai).
- o gravador guardar os pixels das imagens fixas pelo mesmo acessor
  (`rec/src/payload.rs`).
- um override para a câmera, se algum dia houver, é decisão da pessoa e
  de um ADR, não desta fase.
- o custo de uma imagem fixa grande no fio do app: codificar as linhas no
  fio do escritor.
- vídeo de pixels, multiusuário, iOS e android seguem fora.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0062: capture is a job the person turns on](../../adr/Interface/0062-capture-is-a-job-the-person-turns-on/0062-capture-is-a-job-the-person-turns-on.md)
- [ADR 0065: the scene travels as data](../../adr/Recording/0065-scene-travels-as-data/0065-scene-travels-as-data.md)
- [task-76: platform integration](../task-76-platform-integration/task-76-platform-integration.md)
- [task-79: scene over socket](task-79-scene-over-socket.md)
- [closure (orchestrator, 2026-09-25)](closing.md)
- [phase 2 (task-79b): the probe](phase-79b-probe.md)
<!-- /related -->
