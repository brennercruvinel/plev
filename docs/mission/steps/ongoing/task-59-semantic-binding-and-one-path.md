---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
task: task-59
wave: 3
depends: [task-57, task-58]
branch: task/59-sem-binding
---

# task-59: ligação semântica com o render e um caminho operacional

## objetivo

ligar, a cada frame, a identidade lógica (`SemId`) à identidade temporária
de render (`ViewId`, quando existe) e à posição atual na cena (bounds,
layer, clip), sem colocar semântica nas primitivas de GPU. e fazer o input
humano chegar na capacidade pelo mesmo `Dispatcher` que agente,
accessibility e teste usam.

## ligação com a tese

seção 6 ("a mesma estrutura não é a mesma árvore literal": o botão gera
fundo, sombra, texto e ícone, e continua sendo um botão), seção 7 (as três
identidades: lógica persistente, temporária de render, posição atual),
seção 10 (um caminho só). goal 4 do programa: não criar quatro interfaces
paralelas para a mesma coisa.

## estado atual no código

- `ViewId` volta a zero a cada frame (`crates/engine/src/input/mod.rs:50-61`)
  e `HitRegion` não tem significado (`crates/engine/src/input/types.rs:137-146`).
- os widgets de comps fazem hit test próprio por `Rect::contains`
  (exemplo: `CommitForm::hit_test_click` em
  `crates/ide/src/views/commit_form.rs`); `InputState` não é o ponto de
  verdade do hit test nos apps que shipam.
- o `Compositor` é o acumulador por frame que todo render recebe
  (`crates/engine/src/compositor/mod.rs:56-68`, `begin_frame`), e as layers
  têm hash de conteúdo para dirty tracking
  (`crates/engine/src/compositor/layer/mod.rs:43`).
- o clique dos apps chama a lógica do app direto (no ide:
  `CommitFormAction::Commit` em
  `crates/ide/src/views/workspace/input.rs:47`); no showcase, o todo app em
  `crates/showcase/src/view/apps/app.rs` e `app/` sobre o modelo
  `crates/showcase/src/model`.
- modal e overlay vivem em `crates/comps/src/feedback` e
  `crates/comps/src/overlay` (`OverlayManager`).

## fase a fechada (2026-09-23)

integrada na `main` em `f71dc20` (seis commits), validada: diff lido, gate
verde (1636 testes), o modelo do todo só muda dentro de `execute`
(conferido no código), a view mudou só onde a spec permitia
(`overlay_holds_pointer`, reuso da condição de modal que já existia). ADR:
`docs/ADR/semantic-binding-beside-the-scene.md`.

decisões aceitas: a layer da ligação é explícita (`bind` na default,
`bind_to_layer` na dada), porque inferir pela última escrita erra quando o
widget liga antes de desenhar; o clip é lido dos nós da própria layer;
inerte por setter chamado pela view; a linha inteira é
`todo.item[<id>]` e o checkbox da linha fica sem id para não duplicar;
só itens visíveis pelo filtro são projetados. custo medido: 50 a 60 ns por
ligação, projeção do todo com 103 itens em cerca de 50 µs (M4, release).

limitações que seguem: `hit` só enxerga ids ligados (quem recusa página
coberta é o inert); `hit` não lê o `clip_rect` físico da layer;
`Binding.view` fica `None` nos widgets do comps; contador e
`todo.filter` sem rect ligado.

falta a fase b: os outros widgets interativos do comps com `SemId`, helper
de projeção e matriz.

## revisão do orquestrador depois das tasks 57 e 58 (2026-09-23)

esta seção vale sobre o resto do arquivo onde os dois divergirem.

- o `sem` está na `main` com o contrato (57) e o dispatcher (58). leia
  `docs/mission/steps/checked/task-57-semantic-contract-done.md`,
  `docs/mission/steps/checked/task-58-actions-preconditions-receipts-done.md`
  e os dois ADRs antes de começar. `Operable::execute` recebe o
  `RequestId`; `Actor::Assistive` confirma como uma pessoa.
- a task roda em duas fases, cada uma um briefing e uma validação:
  - fase a (primeira): a tabela de ligação no compositor, o `engine`
    dependendo de `sem`, o `SemId` opcional e o helper de projeção só nos
    widgets que o todo usa (`TextField`, `Checkbox`), mais `Button` (o ide
    usa na 62), o todo do showcase de ponta a ponta pelo dispatcher, e o
    fundo inerte com modal aberto.
  - fase b (depois da validação da fase a): os outros widgets interativos
    do comps com `SemId` e helper, cada um na matriz.
- o todo não tem botão de adicionar: o Enter adiciona. o modelo semântico
  do todo fica assim: `todo` (Region), `todo.input` (TextInput, state
  `text`, ações `set_value` com parâmetro texto e `submit` com reads
  `todo.input:text`), `todo.filter` (TabList, state `selected`) com
  `todo.filter[all]`, `todo.filter[active]`, `todo.filter[completed]`
  (Tab, ação `select`), `todo.item[<id>]` (ListItem, label = texto, state
  `completed`, ações `toggle` com reads `todo.item[<id>]:completed` e
  `remove`), `todo.counter` (Status, state `active` e `done`). as pílulas
  de filtro são desenhadas pela página, não por um widget: a página liga
  os rects delas com `Compositor::bind` diretamente.
- o dispatcher mora na página (`TodoPage` é `Semantics + Operable` e dona
  de um `Dispatcher`); o clique projeta a página naquele instante, monta o
  `Request::observe` com `Actor::Human` e despacha. tempo: `Millis` vindo
  do relógio que o showcase já usa para o tick.
- `Constraints.editable` é `false` por padrão; o `todo.input` declara
  `editable: true`, senão `set_value` com `constrained` é recusado.

## decisão de arquitetura (recomendada; vira ADR)

1. o `Compositor` ganha uma tabela lateral por frame:
   `bind(&SemId, rect)` registra `SemId -> Binding { rect, layer, clip,
   order, view: Option<ViewId> }` em coordenadas lógicas de janela, com a
   layer e o clip correntes. a tabela zera em `begin_frame`. ela não entra
   no hash de layer: ligar ou não um id nunca suja uma layer nem muda um
   pixel.
2. `crates/engine` passa a depender de `sem` (puro, compila em wasm).
3. comps: cada widget interativo aceita um `SemId` opcional
   (`Button::sem(id)`, e o equivalente em `IconButton`, `SubmitButton`,
   `Chip`, `TextField`, `Checkbox`, `Switch`, `Slider`, `Select`, `Tabs`,
   `NavLink`, `Tree`, `Table` quando a linha é selecionável,
   `ContextMenu`, `Modal`) e, no render, chama `compositor.bind(id, rect)`.
4. comps oferece helpers de projeção por tipo de widget, para cada app não
   reinventar o mapeamento: `Checkbox::project(&self, b, id, parent)`
   declara `Role::Checkbox`, label, state `checked`, ação `toggle`; o
   mesmo para os outros. o app continua dono do estado; o helper só lê o
   widget que o app possui.
5. um caminho só: a interação de um widget com `SemId` vira
   `Request::observe(&frame_snapshot, Actor::Human, id, action)` e o app
   chama `dispatcher.dispatch`. o handler de cada capacidade existe uma
   vez, dentro de `Operable::execute`; o clique deixa de chamar a lógica
   direto. agente, accessibility e teste pedem a mesma ação e caem no
   mesmo `execute`.
6. modal aberto: o app (ou o `AppShell`, se for genérico) projeta o fundo
   como `inert`; o dispatcher rejeita `Inert`, igual ao clique que já não
   passa.
7. ponte com o keymap (`engine::actions`): o binding de tecla pode nomear
   uma capacidade (`"commit.submit/submit"`). o desenho entra no ADR; a
   implementação só se for pequena, senão fica registrada para a task-62.

## escopo

- tabela de ligação no compositor, com API de leitura
  (`bindings()`, `binding(&SemId)`, `hit(&self, x, y) -> Option<&SemId>`
  pelo topo da ordem de desenho, respeitando clip e layer visível).
- `ViewId` continua como está; `Binding.view` guarda o `ViewId` quando o
  widget registra hit region.
- `SemId` opcional e helper de projeção nos widgets interativos de comps,
  com a matriz de testes (`crates/comps/src/tests/matrix.rs`) cobrindo a
  projeção em todo tema, em phone e desktop.
- o todo app do showcase ligado de ponta a ponta como prova: `todo.input`
  (TextInput, ação `set_value`), `todo.add` (Button, `activate`, reads
  `todo.input:text`), `todo.item[<id>]` (ListItem, state `done`, ações
  `toggle` e `remove`), `todo.filter` (TabList/Select). o app implementa
  `Semantics` e `Operable` e o clique vai pelo dispatcher.

## fora de escopo

AccessKit (task-60), socket (task-61), ide e git (task-62), gravação
(task-63), cmd+click (task-65), IR (task-68). ligar todos os apps: só o
todo app do showcase aqui.

## dependências

task-57 e task-58 integradas.

## superfície sob responsabilidade

- `crates/engine/src/compositor/` (só a tabela de ligação e a API de
  leitura; nada no caminho de desenho)
- `crates/engine/Cargo.toml` (dep `sem`)
- `crates/comps/src/{action,form,nav,content,feedback}` (o `SemId`
  opcional e os helpers de projeção), `crates/comps/src/tests`
- `crates/showcase/src/view/apps/app.rs`, `crates/showcase/src/view/apps/app/**`,
  `crates/showcase/src/model/**` (o todo app)
- proposta de ADR `docs/ADR/semantic-binding-beside-the-scene.md`

proibido: `crates/engine/src/{accessibility,window}` (task-60),
`crates/ide`, `crates/git` (task-62), `crates/engine/src/text` e
`crates/engine/src/text/probe.rs` (task-66 roda em paralelo),
`crates/showcase/src/app.rs` e o entry point do showcase (task-66).

## interfaces que consome

`sem` inteiro (57 e 58).

## interfaces que pode alterar

`Compositor` (adição, sem mudar assinatura existente), widgets de comps
(adição de builder opcional e helpers). mudança de assinatura existente em
comps ou engine precisa de aprovação.

## contrato exposto

- `Compositor::bind`, `bindings`, `binding`, `hit`; `Binding`.
- invariantes: a tabela é por frame; não afeta hash de layer; um `SemId`
  aparece no máximo uma vez por frame (segunda ligação é erro logado uma
  vez e ignorado, nunca panic); o `rect` é o mesmo que o widget usa para
  desenhar e para hit test (uma geometria só).
- helpers de projeção por widget: role, label, state e ações
  consistentes em todo app.
- o padrão "clique vira request" documentado no ADR, com o todo app como
  referência.

## critérios de aceitação

- o todo app: adicionar, marcar e remover item por clique e por
  `Actor::Test` produzem o mesmo estado e o mesmo snapshot (teste com os
  dois caminhos).
- mover um controle (viewport estreito contra largo) muda o `rect` da
  ligação e mantém o `SemId`; o `ViewId` pode mudar.
- ligar ids em nós com pixels idênticos não suja layer (teste de hash).
- modal aberto: request no fundo volta `Inert`.
- a matriz de comps projeta role e label certos para todo widget com
  `SemId`.
- custo: medir, no showcase, o tempo de projeção e de ligação por frame
  na página do todo e registrar no ADR (sem meta inventada).
- `./script/gate` verde (o wasm check do showcase agora compila `sem`).

## testes esperados

unitários da tabela (hit com clip, ordem, layer invisível), de cada
helper de projeção, do todo app pelos dois caminhos, do modal inerte, e o
teste de que a ligação não altera hash.

## riscos e perguntas abertas

- widgets com estado interno (texto do `TextField`, valor do `Slider`):
  o helper lê do widget, o app continua dono do widget. se aparecer estado
  que só o widget conhece e o app não consegue projetar, reportar.
- listas virtualizadas: item fora da tela não é ligado (não tem rect), mas
  continua no snapshot se o app o projeta. a accessibility (60) trata
  offscreen.

## commits esperados

1. `task-59: tabela de ligação SemId no compositor`
2. `task-59: engine depende de sem`
3. `task-59: SemId opcional e projeção nos widgets de ação e formulário`
4. `task-59: SemId opcional e projeção em nav, content e feedback`
5. `task-59: todo app do showcase pelo dispatcher`
6. `task-59: modal deixa o fundo inerte`

## pare e reporte se

- a tabela no compositor precisar tocar o caminho de desenho;
- algum widget exigir mudança de assinatura pública;
- o padrão "clique vira request" quebrar invalidação ou foco.
