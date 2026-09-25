---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-23
domain: generative-runtime
task: task-70
wave: 7
depends: [task-68, task-69, task-63]
branch: task/70-adaptive
---

# task-70: transformações adaptativas

## objetivo

a interface muda sem ser reconstruída do zero: a partir da interface
atual, do estado, do objetivo e do contexto, o sistema aplica uma
transformação segura e verificada que preserva identidade. e, a partir do
histórico de receipts, o sistema oferece capacidades compostas quando
percebe uma sequência que o usuário repete.

## ligação com a tese

seção 16 inteira (interface adaptativa, não apenas generativa: "tem
informação demais aqui", "não consigo comparar esses números", "tenho
dificuldade para enxergar esses números", a sequência review, approve,
publish virando "[aprovar e publicar]"; adaptação contextual real, não
responsive design). seção 4 (o espectro: parametrizada, adaptativa,
recomposta).

## estado atual no código

- depois de 68 e 69: `VerifiedIr`, host de superfícies com reconciliação
  por `SemId`.
- depois de 63: histórico de receipts gravado.
- telas fixas (showcase, ide) não são IR; para serem adaptadas, precisam
  ser expressas em IR (a fixture do formulário de commit da task-68 prova
  que dá) ou receber transformações no nível de projeção. a escolha é
  parte desta task.

## escopo

### operações de transformação

sobre um `VerifiedIr`: `Hide(SemId)`, `Show(SemId)`, `Group(ids, into)`,
`Reorder(parent, ids)`, `Emphasize(SemId)`, `Density(level)` dentro dos
tokens, `Restyle(SemId, token role)`, `SwapView(SemId, component)` quando
os dados ligados são compatíveis (tabela para gráfico de barras),
`Promote(capability)`.

toda transformação:

- produz IR que passa no verifier de novo;
- preserva identidade: nenhum `SemId` some sem um `Hide` explícito, e o
  estado dos que continuam é mantido pela reconciliação da task-69;
- é reversível (a transformação inversa é gerada junto) e registrada.

### proponentes de transformação

- pedidos do usuário ("tem informação demais", "não consigo comparar",
  "números maiores") viram transformações pelo mesmo `Proposer` da task-69
  (saída em transformações em vez de IR inteiro).
- preferências de accessibility (escala de fonte, contraste, densidade,
  reduced motion) como transformações dentro dos tokens.

### capacidades compostas

- detector sobre o histórico de receipts (63): a mesma sequência de ações,
  do mesmo ator, na mesma ordem, repetida N vezes.
- proposta de composta (`review -> approve -> publish` vira "aprovar e
  publicar"); instalar a composta exige aceite humano.
- executar a composta é uma sequência de requests encadeados: cada passo
  confere as próprias precondições contra o estado depois do passo
  anterior; o primeiro rejeitado para a sequência e o receipt da composta
  diz em qual passo e por quê. não há promessa de rollback, a menos que as
  capacidades ofereçam desfazer (declarado no ADR).

## fora de escopo

aprendizado de preferência entre sessões; transformação de tela fixa que
não passa por IR, se a análise da task concluir que não cabe.

## dependências

68, 69 e 63 integradas.

## superfície sob responsabilidade

módulo de transformação junto do IR (68) ou do host (69), conforme o ADR;
o detector de sequências junto do leitor de sessão (63); proposta de ADR
`docs/ADR/adaptive-transforms-preserve-identity.md`.

## interfaces que consome

`VerifiedIr`, `verify` (68), host e reconciliação (69), sessão e receipts
(63), `Dispatcher` (58).

## interfaces que pode alterar

as próprias. mudança no IR ou no host passa pelo orquestrador.

## critérios de aceitação

- testes: cada operação preserva identidade e passa no verifier; a inversa
  volta ao IR original.
- demo: na revisão de silêncios, "tem informação demais" reduz densidade e
  agrupa; "não consigo comparar" troca a lista por gráfico; o estado do que
  continua fica.
- composta: sequência repetida detectada, proposta, aceita, executada; um
  passo que envelhece no meio para a sequência com o motivo.
- `./script/gate` verde.

## testes esperados

unitários por operação, propriedade de preservação de identidade
(proptest), detector de sequência com histórico sintético, execução
encadeada com rejeição no meio.

## riscos e perguntas abertas

- interface que muda sozinha atrapalha memória muscular. transformação só
  por pedido ou com aceite; nunca automática sem o usuário ver.

## commits esperados

1. `task-70: operações de transformação com verificação e inversa`
2. `task-70: proponente de transformação`
3. `task-70: preferências de accessibility como transformação`
4. `task-70: detector de sequência sobre receipts`
5. `task-70: capacidade composta encadeada`

## pare e reporte se

- telas fixas não couberem sem virar IR;
- a composta exigir transação que as capacidades não oferecem.

## fase 1: as operações, a inversa, a verificação e o host

branch `task/70a-transforms`, sobre a `main` em `d64b4b85`. a fase 1 é
só isto: as nove operações sobre o IR de uma superfície aberta, cada uma
verificada de novo, com a inversa, preservando identidade, e o host que
as aplica com histórico e desfazer. o proponente (fase 2) e as
capacidades compostas (fase 3) ficam fora.

### sonda (antes do primeiro tipo, 2026-09-24)

o que existe hoje: a troca de forma inteira. o host recebe um
`VerifiedIr` novo e reconcilia por `SemId` (`crates/surface/src/host.rs:107`
a `122`, `SurfaceHost::transform`; `crates/surface/src/surface.rs:141` a
`190`: o widget de um id que continua com o mesmo `Kind` fica, e com ele
foco, caret e arraste). o `VerifiedIr` só sai do `verify`
(`crates/uir/src/verify/mod.rs:25` a `50` e `54` a `107`). nenhuma
operação pequena existe.

#### (a) o IR: o que cada operação mexe e o que o verifier checa depois

o nó é `IrNode` (`crates/uir/src/ir.rs:62` a `83`): `sem`, `component`,
`label`, `description`, `props`, `layout`, `size`, `bind`, `children`,
`repeat`. o verifier anda a árvore uma vez e aplica as oito regras
(`crates/uir/src/error.rs:13` a `31`): 1 schema e versão, 2 identidade,
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
  da lista do app (`crates/surface/src/expand.rs:93` a `95`), e o
  `Reorder` recusa.
- **`Emphasize(id)`** muda só `props`: um degrau acima na escala que o
  componente desenha. o texto sobe a `typography` na rampa do tema
  (`crates/engine/src/theme/tokens.rs:79` a `183`: small, caption,
  base-2, base, title, headline, h4; sem valor é `body`,
  `crates/surface/src/draw.rs:77` a `81`); o botão e a imagem sobem o
  `control_size` (`draw.rs:149` a `155` e `:182` a `189`); o card, a
  lista e o contêiner sobem a `elevation` (`draw.rs:199` a `220` e `:264`
  a `270`). stat, badge, tabela, gráfico e os outros controles não
  desenham nenhum token de ênfase: recusa, sem sentido para aquele nó.
  nenhuma regra do verifier olha `props` (só o tipo, fechado), então a
  ênfase nunca falha na verificação; ela falha antes, no topo da escala.
- **`Density(level)`** muda o `gap` de `stack` e `grid` e o
  `props.padding`, um degrau na escala `xs` a `xxl`
  (`crates/uir/src/token.rs:36` a `46`), só onde há um token (sem token é
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
  rótulo (`crates/uir/src/vocab.rs:253` a `259`). a superfície desenha
  os números em que cada item da lista se converte
  (`crates/surface/src/draw.rs:443` a `459`, sobre o `Value::List` de
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
  `crates/propose/src/list.rs:1` a `7`), não medidas.
  **na Silences não cabe o gráfico das durações.** o único `List` que o
  app declara é `silences:cuts`, com os ids dos cortes (`["1", "2"]` na
  projeção fixada, `crates/uir/fixtures/showcase-silences.snapshot.json`);
  a duração é um `num` por corte (`silences.cut[1]:duration`), não uma
  lista. um gráfico ligado a `silences:cuts` passaria no verifier e
  desenharia barras de altura 1 e 2, os ids: mentira. para caber, o app
  teria que declarar uma lista de durações, e isso é o app, fora desta
  fase. a demo fica sem o `SwapView` (o briefing previa isso).
- **`Promote(capability)`** acrescenta nos `children` de um contêiner
  dito um `button` com `activate` ligado à capacidade e o rótulo que o
  registro dá (`crates/uir/src/caps.rs:31` a `33`). recusa antes quando o
  registro não declara a ação, quando a política da ação deixa de fora o
  ator que pediu (`sem::Actors`), quando a superfície já mostra a ação
  (ou a guarda escondida: aí é `Show`), e quando a ação leva parâmetro (o
  `activate` do botão não manda nenhum, `verify/bind.rs:86` a `107`).
  falha pela regra 6 quando quem escreveu a superfície não pode ligar a
  ação (`verify/access.rs:63` a `69`) ou quando ela é destrutiva sem
  confirmação sempre (`:70` a `78`), e pelas regras 2 e 8.

#### (b) onde o módulo mora

no `uir`, em `crates/uir/src/transform/`. motivos: a operação é dado que
viaja pelo gravador e, na fase 2, sai do proponente, e o `propose`
depende do `uir` e do `sem`, não do engine (`crates/propose/Cargo.toml:19`
a `24`); aplicar é função pura de `Ir` para `Ir` mais o `verify`, que já
está no `uir`; e o teste de propriedade pede `proptest`, que é
dev-dependency só do `uir` entre os dois (`crates/uir/Cargo.toml:22` a
`25`; o `surface` não tem, e o `Cargo.toml` está fora). o host (histórico,
quem pediu, desfazer, reconciliação) fica no `surface`, em `host.rs`,
onde a troca de forma já está. o `uir` continua puro: o módulo não chama
engine nem comps.

#### (c) a inversa

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
(`crates/uir/src/decode.rs:23` a `34`), e o único nó que não desenha é o
dialog fechado, que precisa de um estado `open` do app
(`crates/surface/src/draw.rs:70` a `75`). nenhum campo novo; não há
parada. a identidade vale sobre os dois juntos: todo id de antes está na
árvore ou guardado, e só o `Hide` guarda.

#### (d) o template de um `repeat`

o IR tem o template uma vez, com `[{item}]` no id (`ir.rs:25` a `27`,
`:141` a `153`); as instâncias só existem na árvore expandida da
superfície (`crates/surface/src/expand.rs:93` a `95` e `:129` a `146`).
então uma operação sobre `review.cut_item[{item}]` vale para todos os
itens, os de agora e os que vierem. uma operação sobre o id de uma
instância (`review.cut_apply[1]`) não acha nó no IR: recusa, dizendo o
template que faz aquele id (`crates/uir/src/item.rs:38` a `71`,
`Pattern::matches`) e que a operação é sobre ele. mexer em um item só
exigiria no IR uma sobreposição por item, que o vocabulário não tem: fica
fora da fase 1. dentro de um template, o `Group` precisa de um `into`
com `[{item}]`, e o verifier cobra (`verify/mod.rs:233` a `238`).

#### (e) telas fixas

a tela fixa da Silences e a do todo não são IR: a Silences calcula a
geometria em código (`crates/showcase/src/view/apps/silences/layout.rs:153`,
`compute`) e desenha a linha do tempo com as marcas dos cortes
(`layout.rs:355` a `371`), que nenhum componente do vocabulário
desenha (`vocab.rs:48` a `93`; um componente novo pede o widget no comps
e um ADR, `vocab.rs:10` a `11`).

- **opção 1, exprimir a tela fixa em IR.** a fixture do formulário de
  commit da 68 prova que dá: é a tela fixa do ide, com os ids do próprio
  ide (`crates/uir/fixtures/ide-commit-form.ir.json`;
  `crates/uir/tests/fixtures.rs:4` a `5`), o verifier deixa um nó com id
  do app ligar só as ações daquele nó (`verify/mod.rs:265` a `281`), e a
  projeção trata esse nó como do app (`crates/surface/src/project.rs:19` a
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
vocabulário (o todo cabe: `crates/uir/fixtures/showcase-todo.ir.json`; o
formulário de commit cabe), e a tela que não couber fica fixa e sem
transformação até o componente existir. a opção 2 quebra a promessa de
que o que se vê é o que se projeta. não é motivo de parada nesta fase: a
fase 1 trabalha só sobre IR.

### handoff da fase 1 (2026-09-24)

```text
Goal: task-70 fase 1: as nove operações sobre o IR de uma superfície
      aberta, cada uma verificada de novo, com a inversa exata,
      preservando identidade, e o host que as aplica com histórico e
      desfazer
Status: pronto para o orquestrador; ADR proposto
```

#### commits

- 872ccafe task-70: the probe of phase 1, the transformations over the IR
- a35131bf task-70: hide, show, group and reorder over a verified IR, each with its inverse
- 34993190 task-70: emphasize, density and restyle, by token, each with the looks before
- 35078c72 task-70: swap a view within its family and promote an action the registry declares
- ce755a67 task-70: identity as a property, over random surfaces and the real fixtures
- b3c66abd task-70: the host applies an operation with history and undoes the last
- ec2ab5db task-70: the Claude's review adapted, in a test and in the regress
- o commit deste texto e do ADR

#### a sonda

está na seção "sonda" acima, com (a) a (e). o resumo: nenhuma operação
exigiu mudar o vocabulário nem o verifier; o `Hide` guarda o nó ao lado
da árvore, sem campo novo; toda inversa volta byte a byte; o estado
retido ficou pelo host, sem mexer na reconciliação da 69. nenhuma
condição de parada aconteceu. o que a sonda achou e mudou a demo: o
gráfico das durações não cabe na Silences (o único `List` do app são os
ids dos cortes), então a demo ficou com o `Reorder` e o `Hide`.

#### cada operação, a inversa e onde pode falhar

o código: `crates/uir/src/transform/` (`mod.rs:55` o `Op`, `:197` o
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

#### onde o módulo mora e por quê

no `uir` (`crates/uir/src/transform/`), puro: a operação é dado que o
gravador guarda e que o proponente da fase 2 escreve, e o `propose` só
depende de `uir` e `sem`; aplicar é função de IR para IR seguida do
`verify`, que está no `uir`; e o `proptest` só é dev-dependency do
`uir`. o histórico, quem pediu e a reconciliação ficam no `surface`.

#### o host e o histórico

`crates/surface/src/host.rs`: `apply` (`:201`) confere e verifica a
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

o estado retido, testado em `crates/surface/tests/adapt.rs`: um slider
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
(`crates/uir/tests/transform.rs`), o mesmo com a ação `admin reset`, e o
autor agente não liga a ação nem para uma pessoa (regra 6).

#### a identidade

`crates/uir/tests/identity.rs`: superfícies aleatórias que verificam por
construção e sequências de até sete operações escolhidas por índice no
que a superfície tem quando cada uma se aplica (256 casos): toda
operação que se aplica verifica, nenhum id sai da superfície, só o
`hide` tira ids da árvore (os do nó escondido), a inversa volta byte a
byte; a sequência inteira desfeita na ordem do histórico volta ao
começo. e toda operação em todo nó das quatro fixtures reais (o
formulário de commit e a revisão do ide, o todo, a revisão do Claude da
Silences), com o conjunto de tipos aplicados fixado por fixture.

#### a demo

sobre a fixture do Claude da primeira rodada, pelo gancho novo
`ShowcaseView::stage_silences_op`
(`crates/showcase/src/view/apps/silences/generated/page.rs:389`), como o
host aplica: `reorder review.root: review.title, review.cuts_list` e
`hide review.controls`. o teste
(`crates/showcase/src/view/apps/silences/tests/generated/transform.rs`):
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
| antes (a fixture do Claude, baseline da 69) | `crates/showcase/fixtures/regress/silences-review-by-the-claude-fixture@core@1280x800/frame.png` |
| depois (o `reorder` e o `hide`) | `crates/showcase/fixtures/regress/silences-review-by-the-claude-fixture-adapted@core@1280x800/frame.png` |

no antes, a superfície mostra o título, o arquivo, a duração, os dois
frames e os sliders repetidos, e os cortes ficam abaixo da dobra. no
depois, logo abaixo do título vêm os dois cortes, cada um com faixa,
estado, duração, frame, prévia, aplicar e rejeitar; depois o arquivo e
os frames do vídeo; os sliders saíram da superfície e seguem na tela
fixa à esquerda.

#### telas fixas

a resposta está na sonda (e) e no ADR: exprimir a tela fixa em IR, tela
por tela, quando ela cabe no vocabulário (o todo e o formulário de
commit cabem); a Silences não cabe por causa da linha do tempo, e fica
fixa e sem transformação até o componente existir. transformar só a
projeção não é o caminho: o que se vê e o que se projeta se separam.

#### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ d64b4b85 (a contagem da 69, sem código mudado desde 25633cba) | 2747 passam, 25 ignorados (139 suítes) | 85 passam, 1 ignorado (6 suítes) | 2832 |
| depois, esta branch @ ec2ab5db (`./script/gate` offline, saída 0, 21:44:23 a 21:46:38) | 2776 passam, 0 falhas, 25 ignorados (142 suítes) | 85 passam, 0 falhas, 1 ignorado (6 suítes) | 2861 |

29 testes novos (2 do `tree`, 17 de `uir/tests/transform.rs`, 2 de
`uir/tests/identity.rs`, 5 de `surface/tests/adapt.rs`, 3 da demo) e três
suítes; nenhum ignorado a mais. o gate inteiro verde, com a perna
urnaui e o check wasm com `-D warnings`. `df -h /`: 19 GB livres antes
e depois. `cargo run --offline -p showcase --example regress --
check` (com o `main.rs` tocado): 170 baselines, 170 iguais, nenhum frame
desenhado, nenhuma GPU aberta; os 169 de antes iguais, um novo.

#### linha de ADR para o índice (o `docs/ADR/README.md` não está na minha lista)

| 2026-09-24 | [adaptive-transforms-preserve-identity](adaptive-transforms-preserve-identity.md) | proposed | módulo `uir::transform`, puro: nove operações como dado (`Op`, serde com `op`) sobre um `Adapted` (o `VerifiedIr` e os nós guardados de lado); cada resultado verificado de novo, com a inversa exata (a operação contrária onde ela é exata: hide e show, reorder, swap_view; o pedaço de antes onde não: props e layout de emphasize, restyle e density, o pai de group e promote); identidade: todo id na árvore ou guardado, só o hide tira da árvore, sem campo novo no `IrNode`; operação sobre template vale para todo item, id de instância recusado com o template; swap_view só dentro de uma família (valor, bool, escolha, lista) e nunca de uma lista que repete; promote só de ação que o registro declara para quem pediu; `SurfaceHost::apply` e `undo` com histórico (`Change`: operação, inversa, ator), reconciliado por `SemId` como a troca inteira; telas fixas: exprimir em IR, não transformar a projeção (task-70 fase 1) |

#### o que as fases 2 e 3 precisam desta

- fase 2 escreve `Op` no JSON acima (a regra de campo desconhecido já
  recusa um erro de digitação do modelo) e aplica por
  `SurfaceHost::apply` com o ator que pediu; falta o schema do `Op` para
  o prompt, a ação de transformar pelo port da página e a linha de
  status. as preferências de accessibility são `emphasize` e `density`
  pedidas em nome da pessoa, com o mesmo histórico.
- fase 3 lê `SurfaceHost::changes` (com o ator) ao lado dos receipts da
  63; instalar uma composta é um `promote` depois que o app a declarar.

#### o que ficou aberto

- o gráfico das durações: o app precisaria declarar uma lista de
  durações (a Silences, fora da fase), e trocar a lista pelo gráfico
  tiraria da árvore os ids do template; se isso deve virar um "hide
  dentro do swap", dito no resultado, é decisão do orquestrador. hoje a
  regra da task vale como está: nenhum id sai sem `hide`.
- mexer num item só de um template pede uma sobreposição por item no
  vocabulário (ADR).
- `emphasize` num botão depois de `md` não muda nada no hoff core: a
  escada do botão para em 44 px (`crates/surface/src/token.rs:142` a
  `154`). stat e badge não desenham token de ênfase
  (`crates/surface/src/draw.rs:304` e `305`), então "números maiores"
  num stat pede o materializador ler `typography` ali (fase 2, no
  `surface`).
- não há refazer; o `Inverse` é só `Serialize` (o `Restore` guarda um
  `IrNode`, que o `uir` só lê dentro de um documento inteiro).
- o host recebe a política a cada `apply`; guardar a política de cada
  superfície ao abrir mudaria o `open`.
- o estado novo do regress depende do `claude` no PATH de quem abençoa,
  como os dois de antes (o seletor do proponente).
- o ADR está como proposto; o índice é do orquestrador.

## fase 2: o proponente de transformação

branch `task/70b-transform-proposer`, sobre a `main` em `3dde4976`. a
fase 2 é isto: uma intenção em palavras, com uma superfície aberta, vira
operações da fase 1 (pelas regras, sem modelo, ou pelo Claude), cada uma
verificada e aplicada como um `Request` pelo dispatcher do app, com o
ator no histórico e desfazer pelo mesmo caminho; a política de cada
superfície guardada pelo host; ênfase em `stat` e `badge`; e as três
lacunas do checkpoint b na revisão gerada.

### sonda (antes do primeiro tipo, 2026-09-24)

#### (a) o proponente de transformação

o `Proposer` (`crates/propose/src/lib.rs:74` a `76`) devolve um `Ir`, e
a saída é `propose_verified` (`:82` a `88`), que só deixa sair
`VerifiedIr`. a task-69 usa os dois pelas páginas
(`crates/showcase/src/view/apps/silences/generated/page.rs:218`,
`crates/showcase/src/view/apps/surface_page/page.rs:244`, e o worker do
Claude em `surface_page/asking.rs:260`). mexer no `Proposer` mexeria em
tudo isso.

o desenho: um trait irmão no mesmo crate, sem tocar no `Proposer`.

- `Adapt<'a>`: o que se pede a um transformador. a intenção, o registro
  (`Capabilities`), o snapshot, a superfície como está (`&Adapted`, o
  `VerifiedIr` e o que está de lado, `crates/uir/src/transform/mod.rs:197`),
  a política com que ela foi verificada e quem pediu (`&Actor`, que o
  `promote` lê, `mod.rs:257` a `261`).
- `Transformer::transform(&mut self, &Adapt) -> Result<Adaptation,
  ProposeError>`, onde `Adaptation` é `Ops(Vec<Op>)` ou `Undo` (desfazer
  não é uma operação: é o host que sabe o que foi a última mudança).
- `transform_verified(t, ask) -> Result<Transformed, ProposeError>`:
  pede ao transformador e aplica cada operação, em ordem, sobre o
  `Adapted` da pergunta (`Adapted::apply`, que verifica,
  `mod.rs:241` a `264`); devolve `Changed { ops, adapted }` só quando
  todas aplicaram e o resultado é um `Adapted` verificado, ou `Undo`.
  lista vazia é erro (nada a mudar). é o par do `propose_verified`:
  nada sai sem passar pelo verifier.
- `Rules` e `Llm<M>` implementam os dois traits. o `Model` é o mesmo.

#### (b) a gramática das regras

`words.rs` lê as intenções de superfície (`Part`, `Form`, "compare",
"everything", `crates/propose/src/words.rs:17` a `230`). as de mudança
entram num leitor irmão, com o mesmo `fold` (caixa, acento, plural,
`:132`) e as mesmas duas línguas:

- **menos** ("too much", "cluttered", "busy", "simpler", "less";
  "demais", "poluído", "mais simples", "menos"): esconde o que não é da
  tarefa, agrupa, e aperta o espaçamento (`density denser`).
- **comparar** ("compare", "comparar") só com uma palavra de dificuldade
  ("can't", "cannot", "hard"; "não consigo", "difícil", "dificuldade"):
  "compare the original with the cuts" continua sendo a comparação de
  vistas da mídia da 69 (`words.rs:106` a `116`, `rules.rs:85` a `99`).
- **maior** ("bigger", "larger", "grow"; "maior", "maiores", "grande",
  "aumentar"), e "não consigo ler/enxergar" ("can't see", "can't read";
  "enxergar", "ler" com dificuldade): `emphasize`.
- **desfazer** ("undo", "revert"; "desfazer", "voltar"): a última
  mudança pedida, inteira.

o resto da intenção: palavras de enchimento das duas línguas, ou
palavras do app (as mesmas de `Parts::words`, `parts.rs:257`), que
escolhem o alvo de "maior" e "comparar". uma palavra que não é nada
disso, ou nenhuma palavra de mudança, e a intenção não é mudança: segue
pelo caminho da 69 (uma superfície nova, `Rules::propose`).

como as regras escolhem o alvo, pela estrutura e pelo registro, nunca
por nome de app:

- **a tarefa** é o que liga ação (um nó com `bind.actions`, ou que tem
  um assim abaixo, um `repeat` inteiro). **contexto** é um bloco que
  liga estado e nenhuma ação na sua subárvore (um vídeo grande, um
  cartão de contagens). "menos" esconde cada bloco de contexto,
  descendo da raiz só pelos contêineres que guardam a tarefa, nunca
  dentro de um `repeat` (o item é a tarefa inteira), nunca filho de
  `split`; nada se a superfície não liga ação nenhuma. depois agrupa:
  num contêiner que mistura leituras e controles em fila (dois controles
  ou mais e uma leitura ou mais), os controles vão para um grupo novo
  (`<pai>.actions`, que no template carrega o `[{item}]` do pai). por
  fim `density denser`. cada operação é tentada em sequência sobre o
  `Adapted`; a recusada é pulada; nenhuma aplicada é erro com o motivo.
- **comparar**: um nó ligado a um `List`/`Set` mostrado como `table` vira
  gráfico de barras quando todo valor da lista lê como número e a lista
  não é a chave dos filhos do seu nó (a estrutura de `list.rs:1` a `15`:
  uma lista de itens é de ids, não de medidas); um gráfico vira tabela.
  uma lista com `repeat` não troca (a sonda da fase 1: os ids do
  template sairiam sem `hide`, `view.rs:70` a `75`). nada que caiba:
  recusa, dizendo o que há e por que não troca. **na Silences não cabe**,
  como a fase 1 achou (o único `List` do app são os ids dos cortes; a
  duração é um `num` por corte): a demo mostra a recusa com o motivo, e
  a troca é provada numa superfície com uma lista de números.
- **maior**: `emphasize` em cada `stat` e em cada `text` ligado a um
  `int` ou `num` (os que as palavras do app nomeiam, quando nomeiam
  algum); num template vale para todo item; recusa no topo da escada.

#### (c) o Claude

`Llm<M>` (`crates/propose/src/llm.rs:36` a `118`) implementa o
`Transformer` com um prompt próprio, num arquivo novo ao lado de
`prompt.rs` (que já tem 341 linhas), reaproveitando o que é igual: as
regras do verifier (`prompt.rs:21` a `67`), os componentes (`:119` a
`173`) e o registro (`:186` a `229`, onde um valor secreto já sai como o
tamanho que o `sem` guarda, `crates/sem/src/redact.rs:1` a `15`, e uma
imagem é um texto `<id>@<segundos>`, nunca pixel). o que é novo: a
superfície aberta (o JSON do IR e os nós de lado, por id e pai), as
nove operações no JSON da fase 1 com o que cada uma faz e recusa, as
regras de identidade (todo id na árvore ou de lado, só `hide` tira, o
`into` novo, `[{item}]` no template, id de instância recusado), as
famílias do `swap_view`, e a resposta: um objeto `{"ops": [...]}`, sem
prosa.

a resposta é lida do primeiro `{` ao último `}` (como `llm.rs:78` a
`85`), e cada operação é tentada em ordem sobre o `Adapted`, verificada;
a primeira recusa, com o índice e o motivo, volta ao modelo junto com a
resposta ("attempt N was refused", a mesma frase que o `claude` falso
lê, `crates/claude-model/tests/fake-claude`), no máximo
`Llm::ATTEMPTS` vezes; esgotado, o erro traz os motivos da última e a
superfície não muda. o `SYSTEM` (`crates/claude-model/src/lib.rs:55`)
já diz "responda exatamente o que ele pede, na forma que ele pede":
serve às operações sem mudança.

#### (d) o caminho pela página

o desenho que a pergunta sugeria era uma ação nova (`silences_surface.adapt`
e `undo`). não cabe sem mudar baseline fora do que o item 7 deixa: o
`regress` compara a projeção antes dos pixels (`crates/showcase/examples/regress/compare.rs:74`,
`Snapshot::diff`), e um nó novo na linha, ou uma ação nova num nó dela,
muda o `semantics.json` de toda baseline que mostra a linha. a linha é
projetada em `surface_page/proposing.rs:340` a `394`: com a superfície
aberta, as baselines da revisão do Claude (fixture), a adaptada da fase
1 e a revisão pelas regras mudariam, e sem superfície as duas da tela
fixa (`silences@core`, `silences@hoff`) também, se o nó aparecesse
sempre.

o caminho que não muda nada disso é o que o ADR da fase 1 já escreveu
("through the same `Proposer` path"): o mesmo `propose`
(`proposing.rs:396` a `415`, `generated/page.rs:194` a `248`,
`surface_page/page.rs:221` a `273`). com uma superfície aberta, a
intenção é lida primeiro como mudança:

- pelas regras: `transform_verified(Rules)`; `Changed` aplica as
  operações de uma vez pelo host, com o ator da request; `Undo` desfaz a
  última mudança pedida; "não é mudança" segue para `propose_verified`
  (a 69, uma forma nova no lugar); outra recusa deixa a superfície como
  está e a linha diz por quê.
- pelo Claude: se o leitor da (b) diz que é mudança, o worker do Claude
  (`surface_page/asking.rs:159` a `276`) roda `transform_verified(Llm)`
  fora da thread do app, o `propose` responde `Execution::Started`, a
  linha mostra a tentativa e os segundos, e o `settle` aplica as
  operações e encerra a request com as tentativas, os segundos e o custo
  (como a 69). "desfazer" não chama o Claude. se não é mudança, é a 69.
- um agente pede pelo port como pede o `propose`: `set_value` na
  intenção e `propose`. o protocolo não muda; o ator da request vai no
  `Change.by`. o desfazer é a intenção "undo"/"desfazer", pelo mesmo
  `propose`.

#### (e) a política por superfície

o host recebe a política a cada `apply` e `undo`
(`crates/surface/src/host.rs:201` a `259`) porque o `VerifiedIr` não
guarda a sua, e o `VerifiedIr` é do verifier (proibido). o host guarda
a política de cada superfície ao abrir e ao trocar de forma:
`open_under(ir, app, policy)` e `transform_under(ir, app, policy)`;
`open` e `transform` (`host.rs:123` e `:143`) ficam com a assinatura e
guardam `GenerationPolicy::default()`, a política com que a página
Surface verifica as suas formas (`surface_page.rs:190` e `:268`, fora
da minha lista), então ela não muda. `apply(id, op, by, app)` e
`undo(id, app)` usam a da superfície; `policy(id)` a lê. as páginas
abrem uma proposta com a política da proposta (`GenerationPolicy::by`
do autor).

e o histórico por pedido: `apply_all(id, ops, by, app)` aplica a lista
inteira sobre o `Adapted` antes de tocar na superfície (tudo ou nada),
toma a forma uma vez (uma reconciliação, uma transição) e guarda cada
`Change` com a fronteira do pedido; `undo_asked(id, app)` desfaz o
último pedido inteiro. `apply` é um pedido de uma operação; `undo`
continua desfazendo uma.

#### (f) `stat` e `badge`

o materializador desenha o `stat` com `Stat::new(valor, rótulo)` sem
tamanho (`crates/surface/src/draw.rs:83` a `86`, `:139` e `:305`), e o
`badge` com `Badge::tag(texto)` (`:138` e `:304`); o `emphasize` recusa
os dois (`crates/uir/src/transform/look.rs:108` a `112`). o comps já tem
o que falta, sem mudança:

- `Stat` tem `StatSize` `Sm`, `Md`, `Lg`, com o valor em `base_m`,
  `title` e `headline` (`crates/comps/src/content/stat.rs:15` a `23` e
  `:61` a `68`). o token é o `typography` do nó, o mesmo da rampa do
  texto: até `base` é `Sm`, `title` (o de hoje, sem token) é `Md`,
  `headline` e acima é `Lg`. a escada do `emphasize` num stat: `base_m`,
  `title`, `headline`, topo.
- `Badge` tem dois formatos, `Tag` (o vidro) e `Count` (preenchido com
  a cor da intenção, texto em contraste,
  `crates/comps/src/action/badge.rs:17` a `24` e `:88` a `101`). o token
  é o `background` do nó: sem ele é `Tag`; com um papel de intenção
  (`accent`, `success`, `danger`, `info`) é o `Count` com essa intenção.
  a ênfase de um badge é ele preenchido (`accent`); o topo é ter fundo.
  maior de tamanho o comps não faz, e mudar o comps é parada: a ênfase
  do badge é de peso, não de tamanho.

o `emphasize` ganha as duas linhas em `look.rs`: é a falta que a sonda
mostra no `crates/uir/src/transform/**`. o vocabulário e o verifier não
mudam (nenhuma regra olha `props`, só o tipo).

#### as três lacunas do checkpoint b

1. o `READS` (`crates/propose/src/prompt.rs:73` a `83`) ganha: revisar
   itens é ver o que cada um é (a imagem do item, quando o registro tem
   uma, e o seu estado), e "só o que a intenção precisa" não tira isso.
   sem nome de app.
2. a linha do item das regras (`crates/propose/src/rules.rs:381` a
   `440`, sobre `list.rs:153` a `242`) põe todo texto e número numa fila
   só, sem rótulo. passa a separar por sentido: o que o item é (imagem,
   texto que não é número, selo), os números (`stat` pequeno, com a
   chave do estado como rótulo, `typography: base_m`, como os `Count` já
   fazem com a chave, `parts.rs:225`), as ações (checkbox, botões). cada
   sentido com dois nós ou mais vira um grupo, com id do template
   (`[{item}]`); uma linha com um nó por sentido (o todo: texto e
   checkbox) fica como está.
3. o estado de poucos valores: um estado de texto que duas ações ou mais
   sem parâmetro do mesmo item leem, e só ele (as `reads` da declaração,
   como o `flip` do checkbox em `list.rs:185` a `194`), é uma decisão:
   sai como `badge`. na Silences, `decision`, que `apply_cut` e
   `reject_cut` leem; o `status` do ide não (o `stage` lê dois estados).

as baselines que mudam por isso são só as da revisão pelas regras
(`silences-review-by-the-rules@*`). nada disso toca a fixture do
Claude.

#### nenhuma parada

nada pede mudar o protocolo v1, o vocabulário ou o verifier do IR, nem o
comps. a mudança em `crates/uir/src/transform/look.rs` é a da (f).

### handoff da fase 2 (2026-09-24)

```text
Goal: task-70 fase 2: uma intenção em palavras, com uma superfície
      aberta, vira operações (pelas regras ou pelo Claude), verificadas e
      aplicadas pelo dispatcher do app como request, com o ator no
      histórico e desfazer pelo mesmo caminho; a política por superfície;
      ênfase em stat e badge; as três lacunas do checkpoint b
Status: pronto para o orquestrador; adição ao ADR proposta
```

#### commits

- 793adf98 task-70: the probe of phase 2, the transformation proposer
- c01021a4 task-70: a transformer answers operations, and only a verified surface comes out
- bdb828ea task-70: the rules read a change of the surface shown, in english and portuguese
- 66b5abd5 task-70: the Claude answers a change as operations, and a refused one goes back with why
- 38426a12 task-70: a change asked through the row's propose, by the rules or the Claude, and the policy kept per surface
- e80d33dd task-70: a stat grows by its typography and a badge fills by its background
- 19fea13d task-70: the review by the rules shows what each item is, its numbers named and its decision as a badge
- 5d1ca12f task-70: the demo, too much information and can't compare over the open review, in a test and in the regress
- o commit deste texto e do ADR

#### a sonda

está na seção "sonda" da fase 2, acima, de (a) a (f), com as lacunas.
nenhuma condição de parada aconteceu: nada pediu mudar o protocolo v1,
o vocabulário ou o verifier do IR, nem o comps; nenhum baseline fora da
revisão gerada mudou. a única mudança em `crates/uir/src/transform/**`
é a da (f): `look.rs` ganhou as escadas de stat e badge.

um desvio do desenho que a pergunta sugeria, na (d): não há ação nova
(`silences_surface.adapt`, `undo`). um nó ou uma ação nova na linha muda
o `semantics.json` de toda baseline que mostra a linha (o `regress`
compara a projeção antes dos pixels), inclusive as da revisão do Claude
e as duas da tela fixa. o pedido é o mesmo `propose` (o que o ADR da
fase 1 já dizia: "through the same `Proposer` path"), e o desfazer é a
intenção "undo"/"desfazer" por ele.

#### o proponente, a gramática e o prompt

- `crates/propose/src/adapt.rs`: `Adapt` (`:28`), `Adaptation`
  (`:57`), `Transformer` (`:66`), `Transformed` (`:72`) e
  `transform_verified` (`:98`), que aplica cada operação em ordem pelo
  `Adapted::apply` e só devolve as operações com o `Adapted` verificado
  que elas fazem; a primeira recusa sai como `ProposeError::Unapplied`
  com o lugar; nenhuma operação é `NoChange`. erros novos em
  `crates/propose/src/error.rs:55` a `80`.
- a gramática: `crates/propose/src/change.rs` (menos, comparar com
  dificuldade, maior, desfazer, em inglês e português;
  `reads_as_change` para a página). as regras por estrutura e registro:
  `crates/propose/src/reshape.rs` (`less`, `compare`, `bigger`, e o
  `impl Transformer for Rules`). nenhuma cita app.
- o prompt: `crates/propose/src/ops.rs` (o prompt da mudança, a leitura
  de `{"ops": [...]}` e o reparo com "attempt N was refused"), e o
  `impl Transformer for Llm` em `crates/propose/src/llm.rs:129`. o
  `SYSTEM` do `claude-model` serve como está (não mudou).

testes: `crates/propose/tests/adapt.rs` (a saída verificada, a recusa
com o lugar, o desfazer, a promoção conferida por quem pede),
`tests/change.rs` (as regras em inglês e português: menos sobre a
revisão do Claude e sobre o todo, comparar que troca uma tabela de
números por gráfico e que recusa na Silences com o motivo, maior em
textos de número e em stats, desfazer, e o que não é mudança),
`tests/llm_change.rs` (resposta boa aplicada, resposta recusada volta
com o motivo e o modelo responde de novo, três recusas devolvem os
motivos e nada muda, o prompt com as operações, o registro com o valor
secreto como tamanho e nunca o texto, a superfície e o que está de
lado, e um agente que não promove ação só da pessoa).

#### o caminho pela página e pelo port

`crates/showcase/src/view/apps/surface_page/proposing/changing.rs`,
compartilhado pela página Surface e pela Silences: com uma superfície
aberta, o `propose` lê a intenção como mudança primeiro. pelas regras,
aplica na hora pelo `SurfaceHost::apply_all` com o ator da request, e a
request termina com `steps` e `ops` (as operações em linha); "desfazer"
é `undo_asked`. pelo Claude, quando a intenção lê como mudança, o worker
(`surface_page/asking.rs`, `Job::Change`) pede operações fora da thread
do app, a request fica `Started`, a linha mostra "asking the Claude for
...: attempt n of 3, s", e o `settle` aplica quando chegam, com
`attempts`, `seconds` e `cost_usd`; "undo" não chama modelo. o que não é
mudança segue para a proposta da 69. a linha diz o que houve ("changed
by the rules for "...", 3 steps", "changed by the Claude for "...", 1
step, in 2.1 s, 1 attempt, 0.050 USD", "undone: the last change, 3
steps"), e o status projeta `steps` ou `undone`.

testes: `crates/showcase/src/view/apps/silences/tests/generated/change.rs`
(uma pessoa pede menos e desfaz, byte a byte; um agente pede pelo
`act`, as mudanças são dele; comparar recusa e nada muda; uma intenção
de superfície ainda propõe uma; e com o `claude` falso: a mudança fora
da thread, com custo; uma resposta recusada pedida de novo; três
recusas não mudam nada; "undo" sem prompt novo) e
`crates/showcase/src/view/apps/surface_page/tests/agent.rs` (um agente
pelo port de verdade escreve "too much", propõe, e depois "undo": as
requests são dele, o protocolo é o do `propose`).

#### a política

`crates/surface/src/host.rs`: `open_under` e `transform_under` guardam a
política com que a superfície foi verificada; `open` e `transform`
guardam a padrão (a das formas da página Surface, que não é minha).
`apply(id, op, by, app)` e `undo(id, app)` não recebem mais política:
usam a da superfície. `apply_all` (tudo ou nada, uma forma, um pedido),
`undo_asked`, `last_ask`, `current` e `policy`. as páginas mostram uma
proposta com a política dela (`GenerationPolicy::by` do autor).
testes: `crates/surface/tests/policy.rs` (a operação de um agente
verificada sob a política da pessoa que escreveu a superfície; os
pedidos tudo ou nada e desfeitos inteiros) e os de `tests/adapt.rs`,
no desenho novo.

#### a ênfase

`crates/surface/src/draw.rs` e `token.rs`: o stat lê `typography` como
o tamanho do valor (`StatSize` `Sm`, `Md`, `Lg`), o badge lê
`background` como a intenção do formato preenchido (`Badge::count`);
`crates/uir/src/transform/look.rs`: a escada do stat (`base_m`, `title`,
`headline`) e a do badge (preenchido com o accent). o badge ganha peso,
não tamanho: o comps não tem badge maior, e mudá-lo seria parada.
testes: `crates/surface/tests/emphasis.rs` e o teste de `look.rs`.

#### as três lacunas, antes e depois

1. o `READS` (`crates/propose/src/prompt.rs:75` a `89`) diz que revisar
   itens é ver o que cada um é (a imagem, o nome, os números com nome,
   o estado) e que "só o que a intenção precisa" não tira isso; teste
   de texto em `crates/propose/tests/llm.rs`, sem nome de app.
2. e 3. a linha do item das regras por sentido
   (`crates/propose/src/row.rs`): o que o item é (frame, faixa, e a
   decisão como badge, porque `apply_cut` e `reject_cut` leem só ela,
   `list.rs`), os números como stat pequeno com o nome do estado
   ("start", "end", "duration"), as ações em botões pequenos numa linha.

| | antes (`main` @ 3dde4976) | depois |
| --- | --- | --- |
| core 1280x800 | `git show 3dde4976:crates/showcase/fixtures/regress/silences-review-by-the-rules@core@1280x800/frame.png` | `crates/showcase/fixtures/regress/silences-review-by-the-rules@core@1280x800/frame.png` |
| hoff 1280x800 | idem, `@hoff@1280x800` | `crates/showcase/fixtures/regress/silences-review-by-the-rules@hoff@1280x800/frame.png` |
| core 1200x800 | idem, `@core@1200x800` | `crates/showcase/fixtures/regress/silences-review-by-the-rules@core@1200x800/frame.png` |

no antes, cada corte é "3.02 5.02 2 0:03.0 to 0:05.0 pending" sem nome,
e em 1200 "reject cut" desce sozinho. no depois, cada corte tem o frame,
a faixa e o selo "pending"; embaixo "3.02 5.02 2" com "start end
duration"; embaixo os três botões numa linha, também em 1200. os dois
cortes cabem na janela. aceitei as três depois de olhar cada uma.

#### a demo

pelas regras, com a revisão aberta, em teste
(`crates/showcase/src/view/apps/silences/tests/generated/demo.rs`) e no
regress (três estados novos em `examples/regress/state.rs`, pedidos pela
linha como um agente pede):

| estado | antes | depois |
| --- | --- | --- |
| "tem informação demais" sobre a revisão do Claude | `crates/showcase/fixtures/regress/silences-review-by-the-claude-fixture@core@1280x800/frame.png` | `crates/showcase/fixtures/regress/silences-review-by-the-claude-fixture-too-much-in-portuguese@core@1280x800/frame.png` |
| "too much information" sobre a revisão das regras | `crates/showcase/fixtures/regress/silences-review-by-the-rules@core@1280x800/frame.png` | `crates/showcase/fixtures/regress/silences-review-by-the-rules-too-much-information@core@1280x800/frame.png` |
| "I can't compare these" sobre a revisão das regras | o mesmo antes | `crates/showcase/fixtures/regress/silences-review-by-the-rules-can-t-compare@core@1280x800/frame.png` |

- "tem informação demais" na revisão do Claude: saem o cartão do vídeo
  e o das contagens (contexto, nenhuma ação), os três botões de cada
  corte viram um grupo, o espaçamento aperta; os dois cortes, que
  estavam abaixo da dobra, ficam dentro de 1280x800, cada um com faixa,
  selo, duração e frame. 4 passos, do harness.
- "too much information" na revisão das regras: saem o vídeo grande e
  as contagens, aperta; a linha já vem agrupada. 3 passos.
- "I can't compare these": recusado, a linha diz por quê em vermelho
  (`proposed.silences.cuts` repete os itens sobre `silences:cuts`, e
  nenhuma lista de números aparece), a revisão fica como estava. a troca
  para gráfico é provada numa superfície com uma lista de números
  (`crates/propose/tests/change.rs`).
- o estado do que continua (`demo.rs`): um corte aplicado antes continua
  aplicado, no app e no selo da gerada; um slider do Claude no meio do
  arraste continua o arraste no lugar novo depois que um agente pede
  "too much information" (-45, -36, e -12 dB depois da mudança).

#### baselines

- novos (3): `silences-review-by-the-rules-too-much-information@core@1280x800`,
  `silences-review-by-the-claude-fixture-too-much-in-portuguese@core@1280x800`,
  `silences-review-by-the-rules-can-t-compare@core@1280x800`, com imagem.
- mudaram (3), pela regra do item 7: `silences-review-by-the-rules@core@1280x800`,
  `@hoff@1280x800` e `@core@1200x800`. nenhum outro.

#### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 3dde4976 (a contagem da fase 1) | 2776 passam, 25 ignorados (142 suítes) | 85 passam, 1 ignorado (6 suítes) | 2861 |
| depois, esta branch @ 5d1ca12f (`./script/gate` offline, saída 0, 22:50:45 a 22:52:58) | 2809 passam, 0 falhas, 25 ignorados (147 suítes) | 85 passam, 0 falhas, 1 ignorado (6 suítes) | 2894 |

33 testes novos e cinco suítes (`propose`: `adapt`, `change`,
`llm_change`; `surface`: `policy`, `emphasis`); o gate inteiro verde,
com a perna urnaui e o check wasm com `-D warnings`. `df -h /`: 19 GB
livres antes e depois. `cargo run --offline -p showcase --example
regress -- check` (com o `main.rs` tocado): 173 baselines, 173 iguais,
nenhum frame desenhado, nenhuma GPU aberta (167 iguais aos da `main`,
3 abençoados de novo pela regra do item 7, 3 novos).

#### um incidente

na primeira bênção do estado da fixture em português, o harness não
escolhia o proponente antes de pedir, e o seletor do regress escolhe o
`claude` que estiver no PATH: o `propose` da mudança começou pelo
Claude, e o processo do regress saiu logo depois (a request ficou
`Executing` e o estado falhou). não achei nenhum `claude -p` rodando
logo depois (`ps`), mas não consigo garantir que nenhuma chamada tenha
começado: no máximo uma, com o teto de 0,5 USD do `claude-model`. o
harness agora escolhe "rules" antes de todo pedido
(`examples/regress/state.rs`, `ask`), e nenhum teste do gate alcança o
programa de verdade (`Claude::find` não procura nos testes do crate).

#### roteiro para o orquestrador provar com o Claude de verdade

na janela de 1200x800 do checkpoint b, com o binário desta branch (ou
da `main` depois de integrar):

```text
cargo build -p showcase                     # e copiar o binário, como no checkpoint
PLEV_AGENT=1 PLEV_MIRROR=1 PLEV_BACKGROUND=1 PLEV_RECORD=1 <showcase> Silences

# a revisão pelo Claude (a cota é do orquestrador)
plev act silences_surface.proposer select --param '"claude"' --app showcase --name orquestrador
plev act silences_surface.intent set_value --param '"cut the silences of this video, then I want to review the cuts"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message   # até a resposta
cargo run -p showcase --example mirror -- --app showcase --frames 1 --png 70b-01-review.png

# a mudança pelo Claude
plev act silences_surface.intent set_value --param '"tem informação demais"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message,steps,attempt,cost_usd
cargo run -p showcase --example mirror -- --app showcase --frames 1 --png 70b-02-less-claude.png

# comparar e números maiores pelo Claude
plev act silences_surface.intent set_value --param '"I can'"'"'t compare these"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message
plev act silences_surface.intent set_value --param '"números maiores"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev observe --app showcase --name orquestrador --scope silences_surface.status --keys busy,message
cargo run -p showcase --example mirror -- --app showcase --frames 1 --png 70b-03-bigger-claude.png

# desfazer (sem modelo) e as mesmas frases pelas regras
plev act silences_surface.intent set_value --param '"desfazer"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
plev act silences_surface.proposer select --param '"rules"' --app showcase --name orquestrador
plev act silences_surface.intent set_value --param '"too much information"' --app showcase --name orquestrador
plev act silences_surface.propose propose --app showcase --name orquestrador
cargo run -p showcase --example mirror -- --app showcase --frames 1 --png 70b-04-less-rules.png
```

o que olhar: na 01, se a revisão nova mantém o frame de cada corte (a
lacuna 1 do prompt); na 02, as operações que o Claude escolhe (o
`steps` e o `ops` do receipt, o custo e as tentativas na linha), se os
cortes ficam dentro da janela e se nada do que a pessoa decide saiu; em
"I can't compare these", se o Claude recusa com `{"ops": []}` ou tenta
um `swap_view` que o host recusa (três tentativas no máximo, a
superfície intacta); em "números maiores", se sobe os stats; "desfazer"
não gasta chamada; pelas regras, o mesmo resultado da baseline
`...-too-much-information`. e pelo port, que cada receipt é do
`orquestrador`.

#### linha de ADR para o índice (o `docs/ADR/README.md` não está na minha lista)

| 2026-09-24 | [adaptive-transforms-preserve-identity](adaptive-transforms-preserve-identity.md) | adição proposta | fase 2: `propose::Transformer` ao lado do `Proposer`, `transform_verified` só devolve operações com o `Adapted` verificado; a gramática de mudanças das regras (menos, comparar com dificuldade, maior, desfazer, en e pt) por estrutura e registro; o prompt de mudança com `{"ops": [...]}` e reparo; o mesmo `propose` da linha com uma superfície aberta, sem ação nova (a projeção das baselines não muda); o host guarda a política de cada superfície e os pedidos (`apply_all`, `undo_asked`); ênfase de stat por `typography` e de badge por `background`; a linha do item das regras por sentido, a decisão como badge (task-70 fase 2) |

#### o que a fase 3 precisa desta

- `SurfaceHost::changes` com o ator e `last_ask` (o que um pedido fez),
  ao lado dos receipts da 63; um pedido é uma request `propose` com
  `steps` e `ops` no outcome, então o detector de sequência também lê
  os receipts.
- instalar uma composta é um `promote` depois que o app a declarar; o
  prompt de mudança já descreve o `promote` e a recusa por ator.

#### o que ficou aberto

- comparar na Silences: recusado com o motivo. para trocar por gráfico,
  o app teria que declarar uma lista de durações (a Silences, fora
  daqui).
- o Claude só recebe uma mudança quando a gramática das regras lê a
  intenção como mudança ("put the cuts first" iria para a proposta de
  superfície); crescer a gramática, ou deixar o modelo escolher entre
  mudança e superfície numa chamada só, é decisão do orquestrador.
- `PERSON_ONLY` pelo port: nem a Silences nem o todo têm ação só da
  pessoa, então a promoção recusada para um agente está provada no
  `propose` (o transformador e o `Llm`) e no host (fase 1), não pelo
  port de uma página.
- não há refazer; "undo" desfaz o último pedido inteiro, e o `undo` do
  host ainda desfaz uma operação.
- a ênfase do badge é de peso (preenchido), não de tamanho.
- a legenda da página Surface (`surface_page.rs`, fora da minha lista)
  não fala de mudança: depois de uma mudança ela continua dizendo o que
  dizia.
- "reduz densidade": segui o briefing, `density denser` (o espaçamento
  aperta); se a intenção deveria afrouxar, é uma linha em `reshape.rs`.
- a adição ao ADR está como proposta; o índice é do orquestrador.
