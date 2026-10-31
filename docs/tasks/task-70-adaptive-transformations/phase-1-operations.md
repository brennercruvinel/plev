---
id: task-70/phase-1-operations
title: "task-70: phase 1, the operations, the inverse, the verification and the host"
project: plev
type: section
status: reference
parent: task-70
date: 2026-09-24
date_basis: text
topics: [ir, generated-surfaces, semantics]
relations: [adr-0067, task-68, task-69]
source: docs/log/steps/task-70-adaptive-transformations.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [872ccafe, a35131bf, 34993190, 35078c72, ce755a67, b3c66abd, ec2ab5db]
---

# phase 1: the operations, the inverse, the verification and the host

Records phase 1 of task-70: the nine transformation operations over the `VerifiedIr` of an open surface (`Hide`, `Show`, `Group`, `Reorder`, `Emphasize`, `Density`, `Restyle`, `SwapView` and `Promote`), each verified again and with its inverse, plus the host that applies them with history and undo. The result preserves the identity of the ids, and the inverses return the surface byte for byte; the demo ended up without `SwapView`, because the durations chart does not fit in Silences (the app's only `List` is the list of cut ids). The phase is the base that the proposer of phase 2 and the composed capabilities of phase 3 reuse.

branch `task/70a-transforms`, sobre a `main` em `d64b4b85`. a fase 1 é
só isto: as nove operações sobre o IR de uma superfície aberta, cada uma
verificada de novo, com a inversa, preservando identidade, e o host que
as aplica com histórico e desfazer. o proponente (fase 2) e as
capacidades compostas (fase 3) ficam fora.

## sonda (antes do primeiro tipo, 2026-09-24)

o que existe hoje: a troca de forma inteira. o host recebe um
`VerifiedIr` novo e reconcilia por `SemId` (`rust/screen/src/host/mod.rs:107`
a `122`, `SurfaceHost::transform`; `rust/screen/src/surface.rs:141` a
`190`: o widget de um id que continua com o mesmo `Kind` fica, e com ele
foco, caret e arraste). o `VerifiedIr` só sai do `verify`
(`rust/plevir/src/verify/mod.rs:25` a `50` e `54` a `107`). nenhuma
operação pequena existe.

### (a) o IR: o que cada operação mexe e o que o verifier checa depois

o nó é `IrNode` (`rust/plevir/src/ir.rs:62` a `83`): `sem`, `component`,
`label`, `description`, `props`, `layout`, `size`, `bind`, `children`,
`repeat`. o verifier anda a árvore uma vez e aplica as oito regras
(`rust/plevir/src/error.rs:13` a `31`): 1 schema e versão, 2 identidade,
3 vocabulário, 4 estado, 5 ação, 6 política, 7 acesso, 8 limites.

- **`Hide(id)`** tira o nó dos `children` do pai. não pode esconder a
  raiz (ela é a superfície; fechar é `SurfaceHost::close`) nem o item de
  um `repeat` (`ir.rs:149` a `153`: um repeat sempre tem o seu item). a
  verificação só falha pela regra 3 quando o pai é um `split`, que tem
  exatamente dois filhos (`verify/mod.rs:333` a `341`).
- **`Show(id)`** põe o nó de volta nos `children` do pai, na posição de
  onde saiu. falha pela regra 3 (o pai virou um `split` com dois de novo),
  pela regra 7 quando o pai é um `overlay` e o nó leva controles para uma
  segunda camada (`verify/access.rs:129` a `138`), e pela regra 2 se o id
  foi tomado nesse meio tempo (conferido antes, com o motivo).
- **`Group(ids, into)`** cria `IrNode::new(into)` (um contêiner sem
  componente, `vocab.rs:176` a `179`: papel `Group`, rótulo opcional) no
  lugar do primeiro irmão, com os irmãos dentro, na ordem em que estavam,
  e o `layout` do pai quando ele é um `stack`. falha pela regra 2 (o
  `into` repetido, `verify/mod.rs:256` a `264`; dentro de um template sem
  `[{item}]`, `:233` a `238`; um `into` que é também o que um template faz
  para algum item, `:286` a `300`), pela regra 3 (tirar um dos dois
  filhos de um `split`), e pela regra 8 (um nível a mais, `:172` a `176`;
  um nó a mais, `:72` a `80`).
- **`Reorder(parent, ids)`** permuta `children`. nenhuma regra olha a
  ordem; o que muda junto é a ordem de foco, que o verifier recalcula na
  ordem da árvore (`verify/mod.rs:39` a `45`), então o foco segue o que
  a pessoa vê. um `repeat` não tem `children`: a ordem das instâncias é a
  da lista do app (`rust/screen/src/expand.rs:93` a `95`), e o
  `Reorder` recusa.
- **`Emphasize(id)`** muda só `props`: um degrau acima na escala que o
  componente desenha. o texto sobe a `typography` na rampa do tema
  (`rust/engine/src/styling/theme/tokens.rs:79` a `183`: small, caption,
  base-2, base, title, headline, h4; sem valor é `body`,
  `rust/screen/src/draw.rs:77` a `81`); o botão e a imagem sobem o
  `control_size` (`draw.rs:149` a `155` e `:182` a `189`); o card, a
  lista e o contêiner sobem a `elevation` (`draw.rs:199` a `220` e `:264`
  a `270`). stat, badge, tabela, gráfico e os outros controles não
  desenham nenhum token de ênfase: recusa, sem sentido para aquele nó.
  nenhuma regra do verifier olha `props` (só o tipo, fechado), então a
  ênfase nunca falha na verificação; ela falha antes, no topo da escala.
- **`Density(level)`** muda o `gap` de `stack` e `grid` e o
  `props.padding`, um degrau na escala `xs` a `xxl`
  (`rust/plevir/src/token.rs:36` a `46`), só onde há um token (sem token é
  o padrão do componente, e ele fica). nunca falha na verificação
  (nenhuma regra olha gap nem padding); recusa quando nada se move (tudo
  já na ponta).
- **`Restyle(id, role)`** muda `props.color` de um texto ou
  `props.background` de um contêiner ou lista, os dois papéis que a
  superfície desenha (`draw.rs:291` a `302` e `:212` a `220`); o papel
  tem que ser de primeiro plano para o texto e de fundo para o contêiner.
  nunca falha na verificação.
- **`SwapView(id, component)`** muda `component` e o nome da prop em
  `bind.state`. é aqui que os dados ligados decidem. o `Chart` mostra só
  `data`, e `data` só aceita `List`; ele não é contêiner e precisa de
  rótulo (`rust/plevir/src/vocabulary.rs:253` a `259`). a superfície desenha
  os números em que cada item da lista se converte
  (`rust/screen/src/draw.rs:443` a `459`, sobre o `Value::List` de
  texto, `widget.rs:120` a `126`), e o `List` do `sem` não tem tipo de
  item. a `Table` mostra `rows`, `List` ou `Set`, rótulo opcional
  (`vocab.rs:238` a `244`), cada item uma linha de texto (`draw.rs:334` a
  `344`). então a troca que cabe é entre componentes que mostram o mesmo
  dado com outro nome, sem largar nenhuma ligação: tabela e gráfico (a
  lista), os gráficos entre si, texto, stat e badge (um valor), checkbox,
  switch e chip de alternar (um bool com `toggle`), select e tabs. falha
  pela regra 4 quando o tipo não serve (um stat de `num` para badge, que
  aceita `text` ou `int`, `vocab.rs:276` a `282`; uma tabela sobre um
  `Set` para gráfico) e pela regra 7 quando o destino exige rótulo que o
  nó não tem (gráfico e stat, `verify/access.rs:103` a `107`). de uma
  `List` com `repeat` para tabela ou gráfico não cabe: os ids do template
  sairiam da árvore sem `Hide`, e a identidade proíbe; e os itens de um
  `repeat` são as chaves das instâncias (`ir.rs:141` a `148`; as regras
  montam a lista sobre um estado cujos valores são as chaves dos filhos,
  `rust/offers/src/list.rs:1` a `7`), não medidas.
  **na Silences não cabe o gráfico das durações.** o único `List` que o
  app declara é `silences:cuts`, com os ids dos cortes (`["1", "2"]` na
  projeção fixada, `rust/plevir/tests/fixtures/uigallery-silences.snapshot.json`);
  a duração é um `num` por corte (`silences.cut[1]:duration`), não uma
  lista. um gráfico ligado a `silences:cuts` passaria no verifier e
  desenharia barras de altura 1 e 2, os ids: mentira. para caber, o app
  teria que declarar uma lista de durações, e isso é o app, fora desta
  fase. a demo fica sem o `SwapView` (o briefing previa isso).
- **`Promote(capability)`** acrescenta nos `children` de um contêiner
  dito um `button` com `activate` ligado à capacidade e o rótulo que o
  registro dá (`rust/plevir/src/capabilities.rs:31` a `33`). recusa antes quando o
  registro não declara a ação, quando a política da ação deixa de fora o
  ator que pediu (`sem::Actors`), quando a superfície já mostra a ação
  (ou a guarda escondida: aí é `Show`), e quando a ação leva parâmetro (o
  `activate` do botão não manda nenhum, `verify/bind.rs:86` a `107`).
  falha pela regra 6 quando quem escreveu a superfície não pode ligar a
  ação (`verify/access.rs:63` a `69`) ou quando ela é destrutiva sem
  confirmação sempre (`:70` a `78`), e pelas regras 2 e 8.

### (b) onde o módulo mora

no `uir`, em `rust/plevir/src/transform/`. motivos: a operação é dado que
viaja pelo gravador e, na fase 2, sai do proponente, e o `propose`
depende do `uir` e do `sem`, não do engine (`rust/offers/Cargo.toml:19`
a `24`); aplicar é função pura de `Ir` para `Ir` mais o `verify`, que já
está no `uir`; e o teste de propriedade pede `proptest`, que é
dev-dependency só do `uir` entre os dois (`rust/plevir/Cargo.toml:22` a
`25`; o `surface` não tem, e o `Cargo.toml` está fora). o host (histórico,
quem pediu, desfazer, reconciliação) fica no `surface`, em `host.rs`,
onde a troca de forma já está. o `uir` continua puro: o módulo não chama
engine nem comps.

### (c) a inversa

cada operação devolve a inversa junto, e as duas saem exatas por
construção. onde o vocabulário tem a operação contrária exata, a inversa
é ela; onde não tem, a inversa guarda o pedaço de antes.

- naturalmente invertíveis, a inversa é uma operação: `Hide` e `Show`
  (o nó volta ao pai e à posição de onde saiu; a inversa de um `Show`
  guarda a posição de antes, para o registro de escondidos voltar igual
  mesmo quando a posição foi limitada ao tamanho do pai); `Reorder` (a
  inversa é o `Reorder` com a ordem inteira de antes); `SwapView` (dentro
  de uma família a troca de nome da prop é uma bijeção, então a troca de
  volta dá o mesmo nó; o `BTreeMap` do `bind` não guarda ordem de
  inserção).
- guardam o que substituíram: `Emphasize`, `Restyle` e `Density` (os
  `props` e o `layout` de antes de cada nó que mudou: a escala limita nas
  pontas e "sem token" não é um degrau, então o passo contrário não volta
  ao mesmo lugar); `Group` e `Promote` (o nó pai inteiro de antes, porque
  as nove não têm "desagrupar" nem "tirar o promovido").

a escolha: exatidão primeiro, e o menor pedaço que dá. voltar byte a byte
é o critério, e cada uma delas volta; a operação contrária é o que o
gravador e a fase 2 leem melhor, e fica onde é exata. desfazer também
passa pelo `verify`: nada sai sem ser `VerifiedIr`.

**como o `Hide` aparece no IR sem mudar o vocabulário.** o nó sai da
árvore do `Ir`, que é o que o host materializa, desenha e projeta, e fica
guardado ao lado, no estado da transformação (`uir::transform`): o nó
inteiro, o pai e a posição. o `Show` e a inversa tiram de lá. o
vocabulário não tem "escondido": o `IrNode` não tem campo para isso
(`ir.rs:62` a `83`), o decodificador recusa campo desconhecido
(`rust/plevir/src/decode.rs:23` a `34`), e o único nó que não desenha é o
dialog fechado, que precisa de um estado `open` do app
(`rust/screen/src/draw.rs:70` a `75`). nenhum campo novo; não há
parada. a identidade vale sobre os dois juntos: todo id de antes está na
árvore ou guardado, e só o `Hide` guarda.

### (d) o template de um `repeat`

o IR tem o template uma vez, com `[{item}]` no id (`ir.rs:25` a `27`,
`:141` a `153`); as instâncias só existem na árvore expandida da
superfície (`rust/screen/src/expand.rs:93` a `95` e `:129` a `146`).
então uma operação sobre `review.cut_item[{item}]` vale para todos os
itens, os de agora e os que vierem. uma operação sobre o id de uma
instância (`review.cut_apply[1]`) não acha nó no IR: recusa, dizendo o
template que faz aquele id (`rust/plevir/src/item.rs:38` a `71`,
`Pattern::matches`) e que a operação é sobre ele. mexer em um item só
exigiria no IR uma sobreposição por item, que o vocabulário não tem: fica
fora da fase 1. dentro de um template, o `Group` precisa de um `into`
com `[{item}]`, e o verifier cobra (`verify/mod.rs:233` a `238`).

### (e) telas fixas

a tela fixa da Silences e a do todo não são IR: a Silences calcula a
geometria em código (`apps/uigallery/src/views/apps/silences/fixed_page/layout.rs:153`,
`compute`) e desenha a linha do tempo com as marcas dos cortes
(`layout.rs:355` a `371`), que nenhum componente do vocabulário
desenha (`vocab.rs:48` a `93`; um componente novo pede o widget no comps
e um ADR, `vocab.rs:10` a `11`).

- **opção 1, exprimir a tela fixa em IR.** a fixture do formulário de
  commit da 68 prova que dá: é a tela fixa do ide, com os ids do próprio
  ide (`rust/plevir/tests/fixtures/idestudio-commit-form.ir.json`;
  `rust/plevir/tests/golden.rs:4` a `5`), o verifier deixa um nó com id
  do app ligar só as ações daquele nó (`verify/mod.rs:265` a `281`), e a
  projeção trata esse nó como do app (`rust/screen/src/project.rs:19` a
  `22`). a tela passa a ser uma superfície e todas as nove operações valem
  sobre ela, verificadas, com inversa e histórico, pelo mesmo caminho. o
  custo: cada tela reescrita em IR, e a da Silences perde a linha do
  tempo até existir o componente.
- **opção 2, transformação no nível da projeção.** a tela desenha como
  quer e a transformação mexe no que ela projeta. o que a pessoa vê e o
  que o agente lê se separam: um `Reorder` na projeção não move nada na
  tela, e um `Hide` só some da tela se cada tela implementar o
  esconder no próprio layout. não há verifier nem inversa por
  construção, e cada tela vira um caso.

**recomendação: a opção 1**, tela por tela, quando a tela couber no
vocabulário (o todo cabe: `rust/plevir/tests/fixtures/uigallery-todo.ir.json`; o
formulário de commit cabe), e a tela que não couber fica fixa e sem
transformação até o componente existir. a opção 2 quebra a promessa de
que o que se vê é o que se projeta. não é motivo de parada nesta fase: a
fase 1 trabalha só sobre IR.

## handoff da fase 1 (2026-09-24)

```text
Goal: task-70 fase 1: as nove operações sobre o IR de uma superfície
      aberta, cada uma verificada de novo, com a inversa exata,
      preservando identidade, e o host que as aplica com histórico e
      desfazer
Status: pronto para o orquestrador; ADR proposto
```

### commits

- 872ccafe task-70: the probe of phase 1, the transformations over the IR
- a35131bf task-70: hide, show, group and reorder over a verified IR, each with its inverse
- 34993190 task-70: emphasize, density and restyle, by token, each with the looks before
- 35078c72 task-70: swap a view within its family and promote an action the registry declares
- ce755a67 task-70: identity as a property, over random surfaces and the real fixtures
- b3c66abd task-70: the host applies an operation with history and undoes the last
- ec2ab5db task-70: the Claude's review adapted, in a test and in the regress
- o commit deste texto e do ADR

### a sonda

está na seção "sonda" acima, com (a) a (e). o resumo: nenhuma operação
exigiu mudar o vocabulário nem o verifier; o `Hide` guarda o nó ao lado
da árvore, sem campo novo; toda inversa volta byte a byte; o estado
retido ficou pelo host, sem mexer na reconciliação da 69. nenhuma
condição de parada aconteceu. o que a sonda achou e mudou a demo: o
gráfico das durações não cabe na Silences (o único `List` do app são os
ids dos cortes), então a demo ficou com o `Reorder` e o `Hide`.

### cada operação, a inversa e onde pode falhar

o código: `rust/plevir/src/transform/` (`mod.rs:55` o `Op`, `:197` o
`Adapted`, `:241` o `apply`, `:268` o `undo`, `:355` a verificação de
todo resultado; `shape.rs`, `look.rs`, `view.rs` as operações; `tree.rs`
o caminho até um nó).

| operação | inversa | recusa antes | o verifier recusa |
| --- | --- | --- | --- |
| `hide` (`shape.rs:37`) | `show` | raiz, item de um repeat, já escondido, id de instância | `split` com um filho |
| `show` (`shape.rs:64`) | o nó de lado de novo, na posição guardada | não escondido, dentro de um escondido, lugar escondido ou sumido | `split` com três, controles em duas camadas de um `overlay` |
| `group` (`shape.rs:136`) | o pai de antes | não irmãos, `into` já usado (na árvore ou de lado), nome repetido | `into` sem `[{item}]` num template, filho de `split`, profundidade e contagem |
| `reorder` (`shape.rs:156`) | `reorder` com a ordem inteira de antes | repeat (a ordem é do app), estranho, ordem igual | nada |
| `emphasize` (`look.rs:82`) | props e layout de antes | topo da escala, componente sem token de ênfase | nada |
| `density` (`look.rs:131`) | props e layout de cada nó que mexeu | nada se move | nada |
| `restyle` (`look.rs:204`) | props de antes | papel de outro tipo (tinta num texto, fundo num contêiner), componente com cores do tema, papel igual | nada |
| `swap_view` (`view.rs:58`) | `swap_view` com o componente de antes | fora da família (`view.rs:38`), lista que repete template, chip que manda `activate` | tipo do dado (regra 4), rótulo exigido (regra 7) |
| `promote` (`view.rs:147`) | o contêiner de antes | ação não declarada, política que deixa de fora quem pediu, ação com parâmetro, já mostrada (ou escondida), lugar sem filhos | autor da superfície sem permissão, destrutiva sem confirmação (regra 6) |

a escada de ênfase do texto segue a rampa do tema por tamanho
(`look.rs:27` e `:49`): sem estilo é `body` (16) e vai para `title` (20),
depois `headline` e `h4`.

### onde o módulo mora e por quê

no `uir` (`rust/plevir/src/transform/`), puro: a operação é dado que o
gravador guarda e que o proponente da fase 2 escreve, e o `propose` só
depende de `uir` e `sem`; aplicar é função de IR para IR seguida do
`verify`, que está no `uir`; e o `proptest` só é dev-dependency do
`uir`. o histórico, quem pediu e a reconciliação ficam no `surface`.

### o host e o histórico

`rust/screen/src/host/mod.rs`: `apply` (`:201`) confere e verifica a
operação contra o registro do app sob a política dada, e a superfície
toma a forma pelo mesmo caminho da troca inteira (`take_form`, `:166`:
a colisão com as outras superfícies e o `Surface::transform`,
reconciliado por `SemId`); guarda um `Change` (`:88`: a operação, a
inversa, o ator). `undo` (`:232`) aplica a última inversa, verificada de
novo, do mesmo jeito. `changes` (`:263`) e `adapted` (`:269`) leem o
histórico e o que está de lado. uma operação recusada deixa superfície
e histórico como estavam; `transform` (`:151`) e `close` (`:308`)
encerram o histórico. o `apply` recebe a política porque o `VerifiedIr`
não guarda a sua. a reconciliação da 69 não mudou.

o estado retido, testado em `rust/screen/tests/adapt.rs`: um slider
no meio de um arraste continua o arraste depois de um `hide`, de um
`reorder` e do desfazer do `reorder`; um campo mantém o teclado e o caret
(a tecla cai três posições atrás) depois de `hide`, `emphasize` e
`reorder`; um clique começado no aplicar do primeiro corte completa
depois que a lista sobe acima do vídeo, e o app aplica o corte pelo
dispatcher (a instância do corte manteve o widget).

`PERSON_ONLY`, no mesmo arquivo: uma superfície escrita por uma pessoa
sobre um cofre cujo `toggle` e `open` o app guarda de agentes; depois de
`group`, `reorder`, `swap_view` (checkbox para switch) e `promote` do
`open`, a página projeta as ações com a política do app (agente
fora); o `promote` pedido por um agente é recusado. no `uir`
(`rust/plevir/tests/transform.rs`), o mesmo com a ação `admin reset`, e o
autor agente não liga a ação nem para uma pessoa (regra 6).

### a identidade

`rust/plevir/tests/identity.rs`: superfícies aleatórias que verificam por
construção e sequências de até sete operações escolhidas por índice no
que a superfície tem quando cada uma se aplica (256 casos): toda
operação que se aplica verifica, nenhum id sai da superfície, só o
`hide` tira ids da árvore (os do nó escondido), a inversa volta byte a
byte; a sequência inteira desfeita na ordem do histórico volta ao
começo. e toda operação em todo nó das quatro fixtures reais (o
formulário de commit e a revisão do ide, o todo, a revisão do Claude da
Silences), com o conjunto de tipos aplicados fixado por fixture.

### a demo

sobre a fixture do Claude da primeira rodada, pelo gancho novo
`ShowcaseView::stage_silences_op`
(`apps/uigallery/src/views/apps/silences/generative/page.rs:389`), como o
host aplica: `reorder review.root: review.title, review.cuts_list` e
`hide review.controls`. o teste
(`apps/uigallery/src/views/apps/silences/tests/generative/transform.rs`):
a 1280x800 os dois cortes ficam dentro da janela, acima do vídeo; a
detecção sai da superfície e continua na tela fixa; o histórico tem as
duas mudanças pelo `test:regress`; desfazer as duas devolve a fixture
byte a byte; e o `swap_view` da lista para gráfico é recusado ("repeats
a template"), com o app declarando uma lista só, `silences:cuts`.

o regress: `State` ganhou `ops` (operações em JSON aplicadas depois da
proposta, `examples/regress/state.rs:211`), e o estado novo "Silences,
review by the Claude (fixture), adapted" (`:72`), abençoado a 1280x800
em core com a imagem.

| | imagem |
| --- | --- |
| antes (a fixture do Claude, baseline da 69) | `tool/bench/regress/baselines/silences-review-by-the-claude-fixture@core@1280x800/frame.png` |
| depois (o `reorder` e o `hide`) | `tool/bench/regress/baselines/silences-review-by-the-claude-fixture-adapted@core@1280x800/frame.png` |

no antes, a superfície mostra o título, o arquivo, a duração, os dois
frames e os sliders repetidos, e os cortes ficam abaixo da dobra. no
depois, logo abaixo do título vêm os dois cortes, cada um com faixa,
estado, duração, frame, prévia, aplicar e rejeitar; depois o arquivo e
os frames do vídeo; os sliders saíram da superfície e seguem na tela
fixa à esquerda.

### telas fixas

a resposta está na sonda (e) e no ADR: exprimir a tela fixa em IR, tela
por tela, quando ela cabe no vocabulário (o todo e o formulário de
commit cabem); a Silences não cabe por causa da linha do tempo, e fica
fixa e sem transformação até o componente existir. transformar só a
projeção não é o caminho: o que se vê e o que se projeta se separam.

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ d64b4b85 (a contagem da 69, sem código mudado desde 25633cba) | 2747 passam, 25 ignorados (139 suítes) | 85 passam, 1 ignorado (6 suítes) | 2832 |
| depois, esta branch @ ec2ab5db (`./tool/tasks/fullcheck.sh` offline, saída 0, 21:44:23 a 21:46:38) | 2776 passam, 0 falhas, 25 ignorados (142 suítes) | 85 passam, 0 falhas, 1 ignorado (6 suítes) | 2861 |

29 testes novos (2 do `tree`, 17 de `uir/tests/transform.rs`, 2 de
`uir/tests/identity.rs`, 5 de `surface/tests/adapt.rs`, 3 da demo) e três
suítes; nenhum ignorado a mais. o gate inteiro verde, com a perna
urnaui e o check wasm com `-D warnings`. `df -h /`: 19 GB livres antes
e depois. `cargo run --offline -p regress --
check` (com o `main.rs` tocado): 170 baselines, 170 iguais, nenhum frame
desenhado, nenhuma GPU aberta; os 169 de antes iguais, um novo.

### linha de ADR para o índice (o [`adr/`](../../adr/README.md) não está na minha lista)

| 2026-09-24 | [adaptive-transforms-preserve-identity](../../adr/generated/0067-adaptive-transforms-preserve-identity.md) | proposed | módulo `uir::transform`, puro: nove operações como dado (`Op`, serde com `op`) sobre um `Adapted` (o `VerifiedIr` e os nós guardados de lado); cada resultado verificado de novo, com a inversa exata (a operação contrária onde ela é exata: hide e show, reorder, swap_view; o pedaço de antes onde não: props e layout de emphasize, restyle e density, o pai de group e promote); identidade: todo id na árvore ou guardado, só o hide tira da árvore, sem campo novo no `IrNode`; operação sobre template vale para todo item, id de instância recusado com o template; swap_view só dentro de uma família (valor, bool, escolha, lista) e nunca de uma lista que repete; promote só de ação que o registro declara para quem pediu; `SurfaceHost::apply` e `undo` com histórico (`Change`: operação, inversa, ator), reconciliado por `SemId` como a troca inteira; telas fixas: exprimir em IR, não transformar a projeção (task-70 fase 1) |

### o que as fases 2 e 3 precisam desta

- fase 2 escreve `Op` no JSON acima (a regra de campo desconhecido já
  recusa um erro de digitação do modelo) e aplica por
  `SurfaceHost::apply` com o ator que pediu; falta o schema do `Op` para
  o prompt, a ação de transformar pelo port da página e a linha de
  status. as preferências de accessibility são `emphasize` e `density`
  pedidas em nome da pessoa, com o mesmo histórico.
- fase 3 lê `SurfaceHost::changes` (com o ator) ao lado dos receipts da
  63; instalar uma composta é um `promote` depois que o app a declarar.

### o que ficou aberto

- o gráfico das durações: o app precisaria declarar uma lista de
  durações (a Silences, fora da fase), e trocar a lista pelo gráfico
  tiraria da árvore os ids do template; se isso deve virar um "hide
  dentro do swap", dito no resultado, é decisão do orquestrador. hoje a
  regra da task vale como está: nenhum id sai sem `hide`.
- mexer num item só de um template pede uma sobreposição por item no
  vocabulário (ADR).
- `emphasize` num botão depois de `md` não muda nada no hoff core: a
  escada do botão para em 44 px (`rust/screen/src/token.rs:142` a
  `154`). stat e badge não desenham token de ênfase
  (`rust/screen/src/draw.rs:304` e `305`), então "números maiores"
  num stat pede o materializador ler `typography` ali (fase 2, no
  `surface`).
- não há refazer; o `Inverse` é só `Serialize` (o `Restore` guarda um
  `IrNode`, que o `uir` só lê dentro de um documento inteiro).
- o host recebe a política a cada `apply`; guardar a política de cada
  superfície ao abrir mudaria o `open`.
- o estado novo do regress depende do `claude` no PATH de quem abençoa,
  como os dois de antes (o seletor do proponente).
- o ADR está como proposto; o índice é do orquestrador.
