---
id: task-69/phase-2
title: "task-69, phase 2: transformation by SemId"
project: plev
type: section
parent: task-69
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [generated-surfaces, animation, performance]
relations: [adr-0051, task-75]
evidence: [56d8146, d9dc076, 0215217, ef2574a, 6a2cb3e, 3f510b3, 4593a4c]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 2: the surface transforms without losing identity

Records phase 2: an open surface transforms into another shape and reconciles the nodes by `SemId`, with geometry by `SemId`, enter, exit and move transitions, and fade-out dismissal. The result is that the field keeps its text, caret and drag when the widget is the same; a button that turns into a `SubmitButton` loses hover and press, because `Press` is private in comps.


branch `task/69b-transform`, worktree próprio, sobre a `main` @ 69b65ed.

## sonda da fase 2 (antes do primeiro tipo, 2026-09-24)

na base não existe transformação: `SurfaceHost::open` recusa um id
aberto, então o único caminho de uma forma verificada para outra é
fechar a superfície e abrir a outra. o teste
`closing_and_reopening_makes_every_widget_new`
(`rust/screen/tests/transform.rs`) faz isso sobre o app de teste com o
golden do todo (`rust/plevir/tests/fixtures/uigallery-todo.snapshot.json`):

- a forma compacta é a fixture da task-68 (`showcase-todo.ir.json`). a
  outra é nova, `rust/screen/tests/fixtures/uigallery-todo-columns.ir.json`,
  verificada contra o mesmo golden: duas colunas, à esquerda o filtro e
  as contagens (`todo_panel.side`), à direita a entrada
  (`todo_panel.main`). os ids dos controles são os mesmos; o grid
  `todo_panel.counts` sai, `todo_panel.side` e `todo_panel.main` entram;
- uma pessoa clica no campo, digita "buy milk", volta o caret três
  posições e digita "X": o app tem "buy mXilk" (a tecla caiu onde o caret
  estava, e o caret mora no widget);
- `close`, depois `open` da forma de duas colunas, verificada contra o
  app como ele está agora. numa área de 900 de largura, o campo estava
  em x 60, 779 de largura; agora está em x 430, 409 de largura: o mesmo
  `SemId` em outro lugar.

o que volta e o que se perde:

- o texto volta. ele é do app (`todo.input:text`), e o widget novo o lê
  pela ligação no `sync` da materialização;
- o foco se perde: `focused()` é `None`, e a tecla seguinte não vai a
  campo nenhum;
- o caret se perde: focado de novo de fora (`Surface::focus`), a tecla cai
  no fim, "buy mXilkY", e não três posições para trás;
- a instância é outra: `close` soltou a `Surface` inteira, e `open`
  criou um widget novo para cada nó. hover, press e o arraste em curso
  iriam junto, pelo mesmo motivo.

onde o estado mora hoje, componente a componente do vocabulário. o que é
do app é relido pela ligação no `sync`. o que é do widget vive no
`Surface.widgets: BTreeMap<SemId, Widget>`. a geometria vive em
`Surface.rects: Vec<Rect>` e `Surface.drawn: Vec<bool>`, pela posição na
árvore, zerados quando a árvore muda.

| componente | do app (ligação) | do widget (retido por `SemId`) | o comps expõe |
|---|---|---|---|
| `text_field` | `text`, o `enabled` do `set_value` | foco, caret, seleção, piscar, hover | `TextField::input` é pub: `focused` e `placeholder` pub, `buffer.cursor()` e `selection()` só para ler; o piscar e o hover são privados |
| `slider` | `value`, faixa e passo do nó alvo, `enabled` | arraste, hover, foco | `is_dragging()`, `is_hovered()`, `is_focused()`, só para ler |
| `switch` | `on`, `enabled` | a mola do knob, hover, press, foco | `on` pub; o resto privado |
| `checkbox` | `checked`, `enabled` | hover, press, foco | `label` pub; o resto privado |
| `button`, `icon_button`, `chip` | label, `enabled`, `active`, `selected` | hover, press, foco (o `Press` de cada um) | `Press` privado |
| `button` que liga `busy` | `busy`, `enabled` | o `Press` do `SubmitButton` | privado |
| `select` | `options`, `selected`, `enabled` | aberto, opção destacada, scroll do painel, foco | `is_open()`; o resto privado |
| `tabs` | `options`, `selected` | hover, foco | `labels`, `active` pub |
| `table` | `rows` | o scroll das linhas | `columns` e `rows` pub |
| `dialog` | `open`, os labels e o `enabled` das ações | hover e press dos dois botões | privado |
| container, `card`, `list`, `text`, `stat`, `badge`, `separator`, `empty_state`, `progress`, `chart`, `image` | tudo | nada (`Widget::Static`) | não se aplica |

a condição de parada não disparou. um nó que continua com o mesmo `SemId`
e o mesmo widget não precisa passar estado de uma instância para outra:
a transformação mantém a instância, e o estado vai junto, exposto ou não.
os props que a forma nova pode mudar num widget mantido (o placeholder do
campo, o label do checkbox, o título da coluna da tabela, o label do
botão e do chip) são campos pub.

o estado só teria de atravessar instâncias quando o widget muda sob o
mesmo id. isso acontece com a troca de componente, que o briefing me
deixa decidir, e num caso dentro do mesmo componente: um `button` que
passa a ligar `busy` (ou deixa de ligar) é outro widget do comps,
`SubmitButton` em vez de `Button`. o `Press` dos dois (hover e press em
curso) é privado e não passa. registro como achado, sem parar. o que se
perde é transitório: o hover volta no movimento seguinte do ponteiro, e
um press em curso no instante da troca não completa. nenhum estado do app
e nenhum estado semântico se perde. se isso tiver de passar, é mudança no
comps (expor o `Press` dos dois) e pede aprovação.

outro achado, fora do comps: o engine não expõe reduced motion. não há
símbolo de preferência de movimento em `rust/engine` nem em
`rust/engine/src/widgets`, e só a task-75 (pendente) traz a preferência do sistema.
a fase 2 não pede mudança no engine para isso: a superfície recebe a
preferência de quem a tiver (`SurfaceHost::set_reduced_motion`), e hoje
ninguém a tem. a task-75 liga a fonte.

correção da sonda (commit 5): a fixture de duas colunas descrita acima
era a primeira versão, duas colunas flex. na metade da página Surface
(420 de largura em 1200) ela empurrava o botão add para fora do card, e
num celular ficaria pior. ela virou um grid sobre `field_min_w`: duas
colunas onde a largura comporta (`todo_panel.side` com o filtro, os chips
um sob o outro, e as contagens; depois `todo_panel.entry`, o campo sobre
o botão), uma coluna onde não. `todo_panel.main` saiu; `todo_panel.side`
continua sendo o nó que nasce e `todo_panel.counts` o que sai. o que a
sonda mostrou (o texto volta, o foco e o caret se perdem) vale igual com
a fixture nova, e o teste da sonda passa com ela.

## handoff da fase 2 (2026-09-24)

goal: task-69 fase 2, a superfície se transforma sem perder identidade.

status: implementado. `cargo test -p screen -p uigallery` verdes,
`./tool/tasks/fullcheck.sh` verde com a perna urnaui, sobre a `main` @ 69b65ed.

### commits

branch `task/69b-transform`, todos assinados, na ordem do briefing:

- 56d8146 task-69b: a sonda da fase 2 no arquivo da task
- d9dc076 task-69b: the geometry of a surface by SemId, not by place in the tree
- 0215217 task-69b: transform an open surface and reconcile it by SemId
- ef2574a task-69b: the transitions of a change of form, reduced motion, a dismissal that fades
- 6a2cb3e task-69b: the Surface page switches its surface between two verified forms
- 3f510b3 task-69b: a form the Surface page cannot take leaves its surface as it was
- e o commit do adendo do ADR e deste handoff

### a sonda

acima, antes do código, com a correção da fixture logo depois. em uma
linha: fechar e abrir era o único caminho, o texto voltava (é do app) e
o foco e o caret se perdiam (eram do widget). a condição de parada não
disparou; os dois achados (o `Press` privado entre `Button` e
`SubmitButton`, e o engine sem reduced motion) estão lá.

### a reconciliação

`SurfaceHost::transform(VerifiedIr, &Snapshot)` transforma a superfície
aberta que o IR nomeia; por baixo, `Surface::transform`. as regras:

- o id da superfície vem do IR. sem superfície aberta com esse id,
  `SurfaceError::NotOpen`; com um nó que tomaria o id de outra superfície
  aberta, `SurfaceError::Collision`. nos dois casos a superfície fica com
  a forma que tinha. só `VerifiedIr` transforma, como só ele materializa;
- a chave é o `SemId` e o widget do comps que o nó pede (`Kind`: o
  componente, e num `button` se ele liga `busy`). mesmo id e mesmo
  widget: o widget fica, e com ele foco, caret, seleção, hover e o
  arraste em curso. o que a forma nova pode mudar nele (label,
  placeholder, título da coluna) é posto de novo no `sync`;
- mesmo id com outro widget (outro componente, ou o botão que passa a
  ligar `busy`): widget novo, e nada do widget velho passa. o que ele
  mostra é do app e é relido pela ligação. decidi assim porque estado de
  widget não tem tradução entre tipos (caret de campo num checkbox), e o
  que precisa sobreviver a uma troca de representação é do app. a imagem
  do widget velho sai enquanto a do novo entra;
- id novo ganha widget; id que sumiu solta o seu. um nó que volta antes
  de a imagem dele sumir é a mesma instância de novo (teste
  `a_node_back_before_its_picture_is_gone_is_the_same_instance`: o campo
  sai no meio da edição, volta, e o caret está onde estava);
- a geometria é por `SemId`: o rect em que cada nó foi desenhado fica sob
  o id dele. um nó que continua mantém o seu até o frame seguinte, então
  um clique nesse meio cai no que a pessoa viu; um nó que saiu perde o
  seu na hora. antes, uma árvore que mudava zerava tudo, e nada era
  atingido até o frame seguinte;
- um `sync` que muda a árvore (a lista de um repeat andou) passa pela
  mesma reconciliação e mostra as mesmas transições.

### as transições e o custo em frames

a semântica não espera os frames. o nó que nasce entra na projeção na
hora, é ligado e atingido, e aparece com fade. o nó que sai deixa a
projeção na hora. um ato de agente nele é recusado como qualquer alvo
que não existe: o `route` o deixa passar como está, e o dispatcher do app
responde `NotFound`. a imagem dele (um ghost com o nó, o widget, o
snapshot que mostrava e o rect) some com fade onde estava, desenhada
por cima dos nós vivos, sem ligação e sem hit. o nó que mudou de lugar
desliza do rect desenhado até o rect do layout.

nenhum motor de animação novo:

- entrada e saída: a mola de progresso do overlay, `Spring<f32>` sobre
  `theme.motion`, como o `OverlayManager` a roda. vira opacidade: o nó
  desenha num compositor próprio, e cada scene node que empurrou vai para
  o frame com o alfa escalado. num path, os vértices, com o hash movido
  junto; imagem e backdrop blur não têm cor e aparecem inteiros até a
  metade;
- movimento: `Tween` do engine sobre `theme.duration.slow`
  (`EaseInOutCubic`), do rect desenhado ao rect do layout de cada frame
  (um resize no meio aterrissa);
- começam no primeiro frame que desenha a forma nova, com o theme e o
  layout desse frame;
- um fade termina a um degrau de 8 bits do fim (1/255). o limiar de
  repouso da mola do engine pedia metade dos frames a mais, todos com a
  mesma imagem: 56 contra 37 no mixer.

render on demand: `Surface::wake` e `SurfaceHost::wake` pedem um frame
quando uma mudança de forma espera ser desenhada, todo frame enquanto
uma transição roda, e nada depois. contado como a task-82 contou (dois
segundos de turnos de 60 Hz, headless, frame só no turno que pede):

| caso | turnos desenhados de 120 |
|---|---|
| mixer parado | 0 |
| mixer, mudança de forma (slider muda de painel, dica nasce, reset sai, switch vira checkbox) | 37, numa sequência só a partir da mudança, nenhum depois |
| a mesma mudança com reduced motion | 1 |

reduced motion: o engine não expõe preferência de movimento, e a do
sistema chega com a task-75. a superfície recebe a preferência de quem a
tiver (`SurfaceHost::set_reduced_motion`, `Surface::set_reduced_motion`).
com ela, toda mudança é instantânea, inclusive as que estão em curso, e
a dispensa solta tudo na hora. a página do showcase não tem fonte para
ela e não a liga.

dispensar: `SurfaceHost::close(&SemId) -> bool`. a superfície deixa a
projeção, o route e o hit na hora, a imagem dela some com fade onde
estava (`SurfaceHost::draw_closing`) e depois tudo é solto.

o arraste: um slider em arraste continua em arraste quando muda de
painel. no primeiro frame da forma nova ele ainda está onde estava, e o
ponteiro age pelo que a pessoa vê; enquanto desliza e depois de chegar,
age pelo rect novo; soltar termina o arraste (teste
`a_drag_goes_on_after_the_slider_changes_panel`, no mixer, porque
nenhuma fixture da task-68 tem slider).

### a página

`apps/uigallery/src/views/apps/surface/mod.rs`: um controle segmentado
(`comps::nav::Tabs`, "compact" e "two columns") acima da metade gerada
troca a forma da superfície. as duas formas:

- compact: a fixture da task-68, `rust/plevir/tests/fixtures/uigallery-todo.ir.json`;
- two columns: `rust/screen/tests/fixtures/uigallery-todo-columns.ir.json`,
  fixture local do crate `surface`, que a página inclui por `include_str!`
  como já incluía a da task-68. fica ali porque `rust/plevir/**` é
  proibido nesta fase, e os testes do `surface` a usam.

cada forma é verificada contra o que o todo projeta no momento do
clique (a projeção real). se não verificar ou não transformar, a
superfície fica como estava e o controle volta (teste
`a_form_that_does_not_transform_leaves_the_surface_as_it_was`). o texto
digitado, o caret e o filtro continuam (teste
`the_control_switches_the_form_and_the_typed_text_and_the_filter_go_on`).
o clique no controle tira o foco do campo, como qualquer clique fora dele,
mas o caret fica no widget. a legenda nomeia o IR da forma mostrada.

render headless, 1200x800, core, escala 1, pelo teste
`surface_page::tests::forms::the_page_draws_headless_in_both_forms`, que
com `SURFACE_PAGE_OUT=<dir>` escreve o png e a projeção de cada forma. a
forma compacta também sai de `cargo run -p uigallery -- --render Surface`,
com a mesma revisão da fase 1 (6ad867dc925c2c1f, 22 nós):

- compact: à esquerda o todo escrito à mão; à direita, sob a legenda
  "materialized from showcase-todo.ir.json", o controle com "compact"
  ativo e o card da superfície (campo "new todo" e add desabilitado na
  mesma linha, os chips All, Active, Completed, e os stats 2 active e 1
  done empilhados);
- two columns: a legenda nomeia `showcase-todo-columns.ir.json`, "two
  columns" ativo, e o card em duas colunas: à esquerda os três chips um
  sob o outro e os stats; à direita o campo e o add embaixo dele. nada sai
  do card. num celular, o grid vira uma coluna (teste
  `the_two_column_form_fits_a_phone_width`).

a projeção da forma de duas colunas (os 10 nós do todo como na fase 1;
os 12 da superfície, forma lean). `todo_panel.side` entrou,
`todo_panel.counts` saiu, os controles e o estado são os mesmos:

```json
[
  { "id": "todo_panel", "role": "region", "label": "quick todo" },
  { "id": "todo_panel.card", "role": "group", "label": "todo", "parent": "todo_panel" },
  { "id": "todo_panel.side", "role": "group", "label": "filter and counts", "parent": "todo_panel.card" },
  { "id": "todo_panel.filter", "role": "group", "label": "filter", "parent": "todo_panel.side" },
  { "id": "todo_panel.filter.all", "role": "button", "label": "All", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ] },
  { "id": "todo_panel.filter.active", "role": "button", "label": "Active", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ] },
  { "id": "todo_panel.filter.completed", "role": "button", "label": "Completed", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ] },
  { "id": "todo_panel.active", "role": "label", "label": "active", "parent": "todo_panel.side",
    "state": { "value": { "kind": "int", "value": 2 } } },
  { "id": "todo_panel.done", "role": "label", "label": "done", "parent": "todo_panel.side",
    "state": { "value": { "kind": "int", "value": 1 } } },
  { "id": "todo_panel.entry", "role": "group", "label": "", "parent": "todo_panel.card" },
  { "id": "todo_panel.input", "role": "text_input", "label": "new todo", "parent": "todo_panel.entry",
    "state": { "text": { "kind": "text", "value": "" } },
    "actions": [
      { "name": "set_value", "reads": ["todo_panel.input:text"], "param": { "kind": "text", "constrained": true } },
      { "name": "submit", "enabled": false, "reads": ["todo_panel.input:text"] }
    ] },
  { "id": "todo_panel.add", "role": "button", "label": "add", "parent": "todo_panel.entry",
    "actions": [ { "name": "activate", "enabled": false, "reads": ["todo_panel.input:text"] } ],
    "enabled": false }
]
```

### interfaces adicionadas e alteradas

no crate `surface` (todas próprias):

- nova: `SurfaceHost::transform(VerifiedIr, &Snapshot) -> Result<&mut Surface, SurfaceError>`;
- nova variante: `SurfaceError::NotOpen(SemId)`;
- alterada: `SurfaceHost::close(&SemId)` devolve `bool` (era
  `Option<Surface>`): a superfície não sai inteira, ela some com fade.
  nenhum consumidor chamava `close`;
- novas: `SurfaceHost::draw_closing(&mut Compositor, &Theme)`,
  `SurfaceHost::set_reduced_motion(bool)`, `SurfaceHost::wake(f32) -> Wake`,
  `Surface::transform(VerifiedIr, &Snapshot)`, `Surface::set_reduced_motion(bool)`;
- semântica estendida: `Surface::wake` passa a incluir as transições;
  depois de uma mudança de árvore, o hit usa o rect desenhado de cada id
  até o frame seguinte.

no showcase, a API pública da `SurfacePage` (a que `view/dispatch.rs` e
`view/input.rs` chamam) não mudou. `switch` e `FORMS` são privados da
página.

### invariantes

- a projeção, o route e o hit seguem a forma atual. uma imagem que ainda
  some nunca é capacidade;
- um widget só é mantido para o mesmo `SemId` e o mesmo `Kind`;
- parada, a superfície não pede frame. uma transição pede até assentar
  e depois nada;
- com reduced motion, nenhuma transição existe.

### testes

18 testes novos:

- `rust/screen/tests/transform.rs` (5): a sonda (fechar e abrir
  perde foco e caret); transformar mantém widget, foco e caret, projeta a
  forma nova na hora e volta; outro componente no mesmo id é widget novo
  sobre o mesmo estado (switch para checkbox); transformação recusada
  (`NotOpen`, `Collision`); o arraste continua depois de mudar de painel;
- `rust/screen/tests/transition.rs` (6): o que nasce entra na projeção na
  hora, o que sai deixa a projeção na hora e é desenhado até a transição
  acabar, sem ligação e sem hit, e o ato de agente nele é `NotFound`; a
  transição pede frames até assentar e depois nenhum (37 de 120);
  reduced motion faz a troca instantânea (1 de 120); reduced motion no
  meio da transição a termina; um nó que volta antes de a imagem sumir é
  a mesma instância; a dispensa some com fade e depois solta tudo (e na
  hora, com reduced motion);
- `rust/screen/tests/events.rs` (+1): antes do frame seguinte, o
  clique atinge o que a pessoa viu (falha na base);
- `rust/screen/src/widget.rs` (+1, unitário): o botão que liga `busy`
  é outro widget;
- `apps/uigallery/src/views/apps/surface/tests/mod.rs` (+4): as duas
  formas verificam contra o todo como uma superfície só (mesmo id, mesmos
  controles); o controle troca a forma, e o texto, o caret e o filtro
  continuam; a forma de duas colunas cabe num celular; uma forma que não
  transforma deixa a superfície como estava;
- `.../surface_page/tests/forms.rs` (+1): a página headless nas duas
  formas, cena e projeção (e pixels com GPU).

fixtures novas em `rust/screen/tests/fixtures/`:
`showcase-todo-columns.ir.json`, `mixer-left.ir.json`,
`mixer-right.ir.json`. o app do mixer (`support::mixer()`) é construído
no teste.

### validação executada

- `cargo test -p screen`, `cargo test -p uigallery --lib surface_page`;
- `cargo clippy -p screen -p uigallery --all-targets -- -D warnings`;
- `RUSTFLAGS="-D warnings" cargo check --target wasm32-unknown-unknown -p uigallery`;
- `./tool/tasks/fullcheck.sh`, saída 0, com a perna urnaui (`df -h /` antes: 15 GB
  livres);
- `cargo run -p uigallery -- --render Surface --out ... --semantics ...` e
  `SURFACE_PAGE_OUT=<dir> cargo test -p uigallery --lib surface_page::tests::forms -- --nocapture`
  para os pngs e as projeções.

### contagem do gate

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 69b65ed (`cargo test --workspace` neste worktree; urnaui como no fechamento da fase 1) | 2297 passam, 12 ignorados | 82 passam, 1 ignorado | 2379 |
| depois, esta branch @ 3f510b3 (`./tool/tasks/fullcheck.sh`, saída 0) | 2315 passam, 0 falhas, 12 ignorados | 82 passam, 0 falhas, 1 ignorado | 2397 |

18 testes novos; a contagem não cai.

### linha de ADR

o adendo datado "a surface changes form (task-69 phase 2, proposed)" em
[`0051-materialized-surfaces-from-verified-ir.md`](../../adr/Generative/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md). linha do índice
para o orquestrador (o [`adr/`](../../adr/README.md) é proibido aqui):

`| 2026-09-24 | [materialized-surfaces-from-verified-ir](../../adr/Generative/0051-materialized-surfaces-from-verified-ir/0051-materialized-surfaces-from-verified-ir.md) | accepted, adendo da fase 2 proposto | fase 2: SurfaceHost::transform reconcilia por SemId e pelo widget do comps (o mesmo mantém foco, caret e arraste; outro é widget novo sobre o estado do app); geometria por SemId; a semântica muda na hora (o que sai é NotFound) e os frames depois: entrada e saída na mola do overlay, movimento num Tween de theme.duration.slow, fade até um degrau de 8 bits; render on demand (37 de 120 turnos no mixer); reduced motion recebido de quem o tem (o engine não expõe, task-75); close some com fade e solta; a página Surface alterna compacta e duas colunas (task-69) |`

### decisões

- widget por `Kind`, não só por componente. contexto: o `button` com
  `busy` é `SubmitButton`, sem o `busy` é `Button`. decisão: a chave é o
  widget. razão: um widget só pode ser mantido se for o mesmo tipo.
  consequência: hover e press não passam nessa troca (`Press` privado no
  comps);
- nada passa entre widgets de tipos diferentes. razão: o que precisa
  sobreviver é do app, e a ligação o relê;
- as transições começam no primeiro frame da forma nova. razão: a mola
  e o tween precisam do theme, e o movimento precisa do rect de chegada,
  que só o layout desse frame conhece;
- o fade é feito por scene node, num compositor próprio do nó. razão: a
  superfície desenha na camada que recebe e não tem camada própria para
  usar a opacidade de camada;
- o fade termina a 1/255 do fim. razão: frames que não mudam pixel são o
  que o render on demand existe para cortar;
- reduced motion como entrada da superfície. razão: o engine não o
  expõe, e a mudança no engine é da task-75.

### limitações

- o fade é por primitiva, não por grupo: as primitivas de um nó se
  misturam entre si enquanto somem;
- o ghost é desenhado sobre os nós vivos. um container que sai com fundo
  cobriria a forma nova enquanto some (nenhuma fixture tem);
- uma superfície aparece de uma vez ao abrir. não há entrada animada no
  `open`;
- o tempo por frame de uma transição não foi medido. o custo conhecido é
  desenhar duas vezes o nó que está em fade;
- o controle da página é da página, não uma capacidade: um agente não
  troca a forma pela porta;
- a página não tem fonte de reduced motion.

### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima), o
  [`changelog.md`](../../CHANGELOG.md) (a página Surface troca de forma) e, se quiser, a
  entrada do `surface` no `docs/ATLAS.toml`, que descreve a fase 1;
- a página inclui `rust/screen/tests/fixtures/uigallery-todo-columns.ir.json`
  por `include_str!`. mudar essa fixture muda a página, e o teste
  `both_forms_verify_against_the_todo_as_one_surface` a verifica contra o
  todo real;
- o `Press` privado entre `Button` e `SubmitButton`, se for preciso que
  passe, é mudança no comps e pede aprovação.

### o que a fase 3 (proponentes) precisa

- o proponente devolve IR. verificado, ele vai para
  `SurfaceHost::transform` quando a superfície está aberta (o mesmo id de
  superfície) e para `open` quando não está. a superfície atual que o
  `Proposer` recebe é `Surface::verified()` (o IR) com `Surface::shown()`
  (o snapshot que ela mostra);
- continuidade depende de ids estáveis. a reconciliação só casa por
  `SemId`, então o prompt do proponente LLM precisa pedir que as mesmas
  capacidades mantenham os ids da superfície atual. um id renomeado é um
  nó que sai e outro que nasce, e o estado do widget (caret, arraste) se
  perde. o proponente por regras pode garantir isso por construção;
- propostas seguidas durante uma transição já funcionam: o movimento
  recomeça do rect desenhado, e um nó que volta recupera a instância;
- a medida "intenção até o primeiro frame" soma o transform, que não foi
  medido à parte. na fase 1, a cena de uma superfície custava frações de
  milissegundo;
- a entrada da intenção na página (um campo de texto) substitui ou
  acompanha o controle segmentado, e pode ser uma capacidade projetada;
- continua aberta a pergunta da fase 1: o verifier aceita props que o
  widget ignora, e um modelo que escreve `typography` num `text_field` não
  aprende com o erro.

# fechamento da fase 2 (orquestrador, 2026-09-24)

integrada na `main` em `4593a4c` (sete commits, rebaseados sem
conflito). o adendo do ADR foi aceito.

validação do orquestrador:
- diff lido (geometria por `SemId`, `transform`, `motion.rs`, a página);
- gate verde no diretório da task: 2429 passaram, 0 falharam, 13
  ignorados, com a perna urnaui;
- o PNG da forma em duas colunas, conferido: o mesmo todo reorganizado
  (os filtros e as contagens à esquerda, o campo e o add à direita), o
  controle "compact | two columns" em cima.

severidade do que ficou:
- P2: o engine não expõe a preferência de reduced motion do sistema (a
  superfície recebe a flag de fora; a fonte do sistema vem com a
  task-75); um `button` que vira `SubmitButton` perde hover e press (o
  `Press` é privado no comps); o controle de forma da página não é
  capacidade projetada, então um agente não troca a forma pela porta;
- P3: fade por primitiva, não por grupo; a imagem que sai é desenhada por
  cima dos nós vivos; `open` sem entrada animada.

próximo: fase 3 (proponentes). o provedor do LLM é decisão do brenner.
