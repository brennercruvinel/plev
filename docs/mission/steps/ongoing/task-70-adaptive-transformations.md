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
