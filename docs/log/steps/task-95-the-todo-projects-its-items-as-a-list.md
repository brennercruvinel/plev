---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-24
domain: generative-surfaces
task: task-95
depends: [task-68, task-69]
branch: task/95-todo-items-as-a-list
origin: handoffs das fases 1 e 3a da task-69 (a superfície gerada não repete sobre os itens do todo)
severity: P2 (completude da onda 6; nada que existe fica inválido)
---

# task-95: o todo projeta os itens como lista

## por que

o IR repete um nó por item de um estado `List` ou `Set` do app
(`uir::Repeat`, `each`). o todo do showcase projeta cada item visível como
um nó `todo.item[<id>]`, mas nenhum estado lista esses ids, então uma
superfície gerada não tem sobre o que repetir: a página Surface mostra o
campo, os chips e a contagem, nunca os itens. o proponente por regras da
task-69 fase 3a bateu nisso: "os itens ativos" não sai do registro. é o
que falta para o checkpoint b comparar a tela gerada com a fixa pelo
mesmo conteúdo.

## escopo

- o todo projeta um estado de lista com os ids dos itens, na ordem em
  que aparecem, e os nós `todo.item[<id>]` continuam como são; o nome, o
  tipo (`List` ou `Set`) e se a lista segue o filtro do todo são decisão
  da task, com o motivo;
- o golden da projeção do todo em `rust/plevir/tests/fixtures` acompanha;
- a superfície da página Surface ganha os itens por `repeat` (as duas
  formas verificadas e a proposta), marcar e remover pela superfície
  pelo dispatcher do todo;
- o proponente por regras gera um `repeat` quando a intenção pede os
  itens, pela estrutura do registro (um estado de lista cujos itens são
  nós do app), nunca pelo nome do todo; as mesmas regras continuam
  lendo o workbench do ide.

## aceite

a superfície gerada mostra os itens e opera sobre eles pelo mesmo
dispatcher; o proponente gera a lista para a intenção "os itens" no todo
e no ide; o golden atualizado e conferido contra o app; gate verde.

## sonda (antes do primeiro tipo, 2026-09-24)

na base, `main` @ bd5f107, por um teste descartável
(`apps/uigallery/src/views/apps/todo/tests/probe_95.rs`, rodado com
`cargo test -p showcase --lib probe_95 -- --nocapture` e jogado fora
depois). ele monta o `TodoPage` real, lista os nós `todo.item[...]` e todo
estado `List` ou `Set` da projeção, pede o filtro active pelo dispatcher
do todo (`Actor::Test`) e lista de novo; verifica à mão um IR que repete
sobre `todo:items`; e lê o golden do workbench do ide
(`rust/plevir/tests/fixtures/idestudio-workbench.snapshot.json`), verifica e
materializa um IR que repete sobre `changes:files`. a saída:

```text
(a) the todo, filter all:
  todo.item[1] ListItem "Absorb the plev demos into the showcase" [completed=Bool(false)]
  todo.item[2] ListItem "Measure text, never estimate it" [completed=Bool(false)]
  todo.item[3] ListItem "Port the todo domain (tested first)" [completed=Bool(true)]
  lists and sets: []
  select active: Some("completed")
(a) the todo, filter active:
  todo.item[1] ListItem "Absorb the plev demos into the showcase" [completed=Bool(false)]
  todo.item[2] ListItem "Measure text, never estimate it" [completed=Bool(false)]
  lists and sets: []
(b) an IR that repeats over todo:items, against the todo's registry:
  /surface/root/repeat/each (rule State): the app declares no state `todo:items`
(c) the ide workbench:
  lists and sets: ["changes:files list List([\"src/lexer.rs\", \"notes.md\", \"src/parser.rs\"])", "changes:staged set Set({\"src/lexer.rs\"})", "diff:lines list List([])", "stacks.commit[48948ff99b2bab50ad30b69c2a71ef75edae1dd9]:files list List([\"README\"])"]
  the repeat over changes:files verifies; focus order [SemId("review.file[{item}].stage")]
  materialized: ["review.files", "review.file[src/lexer.rs]", "review.file[src/lexer.rs].status", "review.file[src/lexer.rs].stage", "review.file[notes.md]", "review.file[notes.md].status", "review.file[notes.md].stage", "review.file[src/parser.rs]", "review.file[src/parser.rs].status", "review.file[src/parser.rs].stage"]
```

o IR de (b), um checkbox por item ligado a `todo.item[{item}]:completed`
e `toggle`:

```json
{ "plev_ir": 1, "surface": { "id": "todo_panel", "title": "quick todo",
  "root": { "sem": "todo_panel.items", "component": "list", "label": "items",
    "repeat": { "each": "todo:items", "item": { "sem": "todo_panel.item[{item}]",
      "children": [ { "sem": "todo_panel.item[{item}].done", "component": "checkbox", "label": "done",
        "bind": { "state": { "checked": "todo.item[{item}]:completed" },
                  "actions": { "toggle": { "target": "todo.item[{item}]", "action": "toggle" } } } } ] } } } } }
```

o que a sonda mostra:

- (a) o todo projeta um nó `todo.item[<id>]` por item visível (list_item,
  label com o texto, estado `completed`), e só os visíveis: com o filtro
  active, `todo.item[3]` sai. nenhum estado da projeção é `List` ou
  `Set`. o texto do item só está no label, não num estado;
- (b) o verifier recusa o repeat pela regra 4, no ponteiro do `each`: o
  estado não existe. o resto do molde nem chega a ser lido contra o
  registro;
- (c) o workbench já tem a forma que falta ao todo: `changes:files`, uma
  `List` num nó (`changes`) cujos filhos são `changes.file[<item>]`, um
  por valor. o repeat sobre ela verifica e materializa uma instância por
  arquivo, com o id chaveado pelo item. o workbench tem outras três
  coleções: `changes:staged` (um `Set` que é parte dos mesmos arquivos),
  `diff:lines` (vazia) e `stacks.commit[...]:files` (valores sem nó
  próprio).

a condição de parada (o todo já projetar uma lista) não disparou.

## a forma da lista (decidida, código pronto, não commitado)

- `todo:items`, uma `List` no nó `todo` (a região que já é o parent de
  cada `todo.item[<id>]`), com os ids dos itens visíveis na ordem em que
  a página os mostra. `List` e não `Set`: a ordem é a da tela, e um `Set`
  se desenrola em ordem de texto ("10" antes de "2"). no nó `todo` e não
  num nó novo: os itens já são filhos dele, e um nó de lista novo pediria
  mudar o parent de cada item (a mudança só cresce). segue o filtro,
  como os nós: a projeção só mostra o que existe agora, então cada id da
  lista é um nó da página e cada nó de item está na lista. trazer todos
  pediria projetar nós para itens que a tela esconde (capacidades que a
  pessoa não vê e que a página não liga).
- `todo.item[<id>]:text`, o texto do item como estado. o IR nomeia uma
  instância pelo item (o id, "1") e não interpola label, então sem esse
  estado uma superfície gerada mostra o checkbox de cada item e nenhum
  texto: o checkpoint b não teria o mesmo conteúdo para comparar. é o que
  o handoff da fase 1 da task-69 já apontava.
- aditivo: o diff do golden `rust/plevir/tests/fixtures/uigallery-todo.snapshot.json`
  só acrescenta (`todo.state.items` e um `text` em cada item); nenhum id,
  role, estado ou ação que existia muda. a fixture compacta de hoje
  continua verificando contra ele.
- teste novo `the_items_are_projected_as_a_list_in_the_order_shown`
  (`app/tests/semantics.rs`): a lista é exatamente os nós de item, na
  ordem, sob `todo`; o `text` de cada um é o label; adicionar, marcar sob
  um filtro, remover e filtrar movem a lista; sem itens, lista vazia.
- `regress`: as cinco baselines que mostram o todo (`todo@core@1280x800`,
  `todo@core@390x844`, `todo@hoff@1280x800`, `toggle-group@core@1280x800`,
  `toggle-group@hoff@1280x800`, a página Toggle Group mostra o todo)
  mudaram só de sentido (`items` e os três `text`), "visual unchanged";
  `bless` nelas, o `frame.png` intacto, e o `check` passa nas 157.

## parada: um teste do `regress` fora da lista quebra com o `text` do item

problema: `cargo test --workspace --no-fail-fast` com a mudança acima dá
2372 passam, 1 falha, 13 ignorados. a falha é
`tests::another_label_in_the_baseline_is_a_semantic_change_of_that_control_only`
em `tool/bench/regress/src/tests.rs` (o briefing não lista
`apps/uigallery/examples/**`; é "o resto do showcase"). o `relabel` do
teste troca o texto "Measure text, never estimate it" no JSON inteiro da
baseline, e agora o texto também está no estado `text` do item, então o
`check` vê duas mudanças no mesmo controle e o teste espera uma:

```text
[LabelChanged { node: SemId("todo.item[2]"), before: "Measure text, then estimate it", after: "Measure text, never estimate it" },
 StateChanged { at: StateRef { node: SemId("todo.item[2]"), key: StateKey("text") }, before: Some(Text("Measure text, then estimate it")), after: Some(Text("Measure text, never estimate it")), added: [], removed: [] }]
```

opções:

1. o `relabel` do teste troca só o campo `label` do nó (lê o JSON, muda
   o label de `todo.item[2]`, escreve), e o teste fica como está: "outro
   label é mudança de sentido só daquele controle". umas dez linhas num
   arquivo fora da lista.
2. o teste aceita as duas mudanças, as duas em `todo.item[2]` (o label e
   o `text`): é o que acontece de verdade quando o texto de um item muda,
   e "só daquele controle" continua valendo. umas cinco linhas no mesmo
   arquivo.
3. o todo não projeta o `text` do item. nada fora da lista muda, e a
   superfície gerada mostra os itens sem o texto (só o checkbox de cada
   um, a instância nomeada pelo id): o repeat existe, o checkpoint b não
   tem o que comparar.

recomendação: 2, e a autorização para `tool/bench/regress/src/tests.rs`
(só esse teste). a 1 também serve; a 3 esvazia a task.

decisão necessária: qual opção; na 1 ou na 2, a autorização para o
arquivo.

### o plano do resto (não bloqueia; confirmar junto)

- as duas fixtures do todo ganham os itens por `repeat` sobre
  `todo:items`, o mesmo molde nas duas (trocar de forma mantém cada
  instância): uma linha com o texto (`text` ligado a
  `todo.item[{item}]:text`) e um checkbox "done" (`checked` de
  `completed`, `toggle`). a compacta põe a lista entre a entrada e o
  filtro; a de duas colunas ganha `todo_panel.main` com a entrada e a
  lista, ao lado de `todo_panel.side`.
- remover pela superfície: o `remove` do todo é destrutivo e não pede
  confirmação a ninguém (`Confirm::Never`), e a regra 6 do verifier só
  deixa ligar uma ação assim ao `confirm` de um dialog. o molde ganha um
  dialog por item, `open` ligado a `completed` (a confirmação aparece sob
  um item feito) e `confirm` ligado ao `remove`. o `cancel` fica sem
  ligação: a superfície desenha o botão e ele não pede nada (limite da
  materialização do dialog, vai para o handoff). alternativas: o dialog
  sempre aberto em cada item, ou nenhuma remoção na superfície (só o
  todo fixo remove, a superfície acompanha). e um dialog fechado continua
  projetado com o `confirm` (regra da fase 1): para um agente, o
  `remove` de um item ativo continua ao alcance pela superfície.
- `Rules`: uma parte nova, "items" (item, items, itens, list, lista),
  que acha por estrutura um estado `List` ou `Set` de um nó cujos valores
  são exatamente as chaves dos filhos dele de um mesmo prefixo
  (`P[<valor>]`): no todo, `todo:items`; no ide, `changes:files` (o
  `changes:staged` é só parte dos arquivos e fica de fora). por item: um
  estado text, int ou num vira `text`; um bool com uma ação sem parâmetro
  que lê só ele vira checkbox (o `toggle` do todo; o `stage` do ide lê
  dois estados e não vira); as outras ações sem parâmetro, não
  destrutivas e permitidas ao autor, viram botões. "everything" passa a
  incluir a lista. os ids do molde que continua são mantidos (hoje o
  `Namer` nunca guarda ids de molde).

## resolução da parada (orquestrador, 2026-09-24)

opção 2: só o teste `another_label_in_the_baseline_is_a_semantic_change_of_that_control_only`
de `tool/bench/regress/src/tests.rs` muda, e aceita as duas
mudanças em `todo.item[2]`. a forma da lista e o `text` aprovados como
descritos. remover pela superfície sai do escopo: um dialog que abre
porque o item está feito diz a coisa errada; a superfície mostra e marca,
a remoção fica no todo fixo, com o limite e a regra 6 no handoff. as
regras do proponente aprovadas como no plano.

## handoff (2026-09-24)

goal: task-95, o todo projeta os itens como lista, a superfície gerada
repete sobre ela e o proponente por regras gera o `repeat`.

status: implementado. `cargo test -p showcase -p propose -p surface -p uir`
verdes, `./tool/tasks/fullcheck.sh` verde com a perna urnaui, `regress check` verde,
sobre a `main` @ bd5f107.

### commits

branch `task/95-todo-items-as-a-list`, todos assinados:

- 4308753 task-95: the probe, in the task file
- 9ca72aa task-95: the shape of the list, and the stop on a regress test outside the list
- 3d8192d task-95: the todo projects its items as a list, with each item's text
- fa1b4ad task-95: the generated surface shows the todo's items and checks them
- 626576d task-95: the rules propose the items, a repeat over a list found by structure
- e o commit do ADR e deste handoff

### a sonda

acima, antes do código. em uma linha: na base o todo projeta um nó por
item visível e nenhuma `List` ou `Set`; o IR sobre `todo:items` cai na
regra 4 ("the app declares no state `todo:items`"); o workbench do ide já
tem a forma (`changes:files`) e o repeat sobre ela verifica e materializa.
a condição de parada não disparou.

### a forma da lista e por quê

- `todo:items`, `List` no nó `todo`, os ids dos itens visíveis na ordem
  da tela. no `todo` porque os itens já são filhos dele (um nó novo pediria
  mudar o parent de cada item); `List` porque a ordem é a da tela; segue o
  filtro porque a projeção só mostra o que existe agora (cada id da lista é
  um nó, cada nó de item está na lista).
- `todo.item[<id>]:text`, o texto do item como estado: o IR nomeia a
  instância pelo id e não interpola label, e sem ele a superfície não
  mostraria o conteúdo.
- só cresce: o diff do golden `rust/plevir/tests/fixtures/uigallery-todo.snapshot.json`
  acrescenta `todo.state.items` e um `text` por item, e nada que existia
  muda (o teste `uir_fixture` o mantém igual ao app).

### a superfície

- as duas fixtures do todo ganham `todo_panel.items` (list, "items"), que
  repete sobre `todo:items` o molde `todo_panel.item[{item}]`: uma linha
  com `todo_panel.item[{item}].text` (`text`, flex 1) e
  `todo_panel.item[{item}].done` (checkbox "done", `checked` de
  `completed`, `toggle`). na compacta, entre a entrada e o filtro; na de
  duas colunas, `todo_panel.main` junta a entrada e a lista ao lado de
  `todo_panel.side`. as duas continuam verificando como uma superfície só
  (mesmo id, mesmos controles). a ordem de foco da compacta, no teste do
  `uir`, ganhou `todo_panel.item[{item}].done`.
- marcar pela superfície é o `toggle` do item, pelo ator que agiu (pessoa
  no clique, agente pelo `act`), pelo dispatcher do todo; o todo fixo
  mostra, e marcar ou remover no todo fixo move a superfície no frame
  seguinte. trocar de forma mantém a instância de cada item: um press
  começado no checkbox do item 1 completa depois da troca, e o checkbox
  aparece em outro lugar.
- remover pela superfície: não. o `remove` do todo é destrutivo e não
  pede confirmação (`Confirm::Never`), e a regra 6 do verifier só liga uma
  ação assim ao `confirm` de um dialog. o todo não tem estado que diga
  "remoção pedida" para abrir um dialog com sentido, então a superfície
  não liga nenhum; quando um item sai da lista (removido no todo fixo, ou
  filtrado), a instância sai da superfície.
- `rust/offers/src/ids.rs` ganhou no commit da superfície a âncora de
  um nó de lista (a lista sobre a qual repete): sem ela, com a fixture de
  duas colunas nova, o teste de continuidade da 3a trocava
  `todo_panel.entry` por `todo_panel.main` num empate.

### as regras novas do proponente (`Rules`)

| palavra | o que faz | a regra sobre o registro |
|---|---|---|
| item, items, itens, list, lista, listar | a lista | um estado `List` ou `Set` de um nó cujos valores são exatamente as chaves dos filhos dele de um mesmo prefixo (`P[<valor>]`), não vazio: um `list` que repete sobre ele, molde `P[{item}]` |

por item, o que todo item declara igual (como o verifier confere um
molde): estado text, int ou num vira `text`; um bool com uma ação sem
parâmetro que lê só ele vira checkbox ligado aos dois (label: a chave do
estado); as outras ações sem parâmetro, permitidas ao autor e não
destrutivas, viram botões (label: o da ação ou o nome, `show_diff` vira
"show diff"). a raiz do molde não tem label (é nomeada pelo item). as
palavras da lista são a chave e o nome dos itens (`item`, `file`), então
"the files" escolhe a lista do ide como qualquer palavra do app. a lista
vai sob a entrada (compacta) ou na coluna dela (duas colunas);
"everything" a inclui. continuidade: o `Namer` entra nos moldes; um molde
sobre a mesma lista mantém o id da raiz e cada nó dele mantém o id que a
âncora tinha.

o que sai sobre os dois apps:

- todo, "items" (e "itens", "the list", "a lista"), sem superfície
  atual: `proposed.todo.items` sobre `todo:items`, molde com
  `text=todo.item[{item}]:text`, `checked=todo.item[{item}]:completed`,
  `toggle=todo.item[{item}] toggle`; nunca o `remove`;
- ide, "items" ou "the files": `changes:files` (o `changes:staged` é só
  parte dos arquivos e fica fora), molde com `text=changes.file[{item}]:status`
  e os botões stage, unstage, ignore, show diff; `stage` lê dois estados,
  então nenhum checkbox; `discard` é destrutivo e fica fora;
- sobre a superfície da página (compacta ou duas colunas), "items", "two
  columns", "compact" e "everything" mantêm `todo_panel.items`,
  `todo_panel.item[{item}]`, `.text` e `.done` (com o label "done");
- lista vazia, valores que não são filhos, ou filhos que a lista não
  cobre: `Nothing` da parte lista.

testes da 3a: nenhum mudou; os oito de `rust/offers/tests/rules.rs`
passam como estavam. o que mudou de saída sem mudar teste: "everything"
passa a trazer a lista, e o texto da gramática (`GRAMMAR`, que o
`NotUnderstood` mostra) e o do `Empty` citam a lista.

### o render

`cargo run -p showcase -- --render Surface --out surface.png --semantics surface.json`
(core, 1200x800, escala 1): "wrote ... surface.json (36 nodes, revision
3293ae75152f9f86, Surface / core at 1200x800 scale 1)". o PNG: à
esquerda o todo escrito à mão, igual; à direita, sob a legenda, a linha de
intenção, o controle compact | two columns e o card: campo "new todo" com
o add desabilitado, os três itens (o texto de cada um e um checkbox "done"
à direita, o terceiro marcado), os chips All, Active, Completed e os stats
2 active e 1 done. em duas colunas (`SURFACE_PAGE_OUT=<dir> cargo test -p
showcase --lib surface_page::tests::forms`): à esquerda os chips e os
stats, à direita o campo, o add e os itens, o texto quebrando na coluna.
em 390x844, tema hoff, a superfície fica sob o todo e os itens cabem na
largura.

a projeção dos itens (forma lean; os outros 25 nós como antes):

```json
[
  { "id": "todo", "role": "region", "label": "todo",
    "state": { "items": { "kind": "list", "value": ["1", "2", "3"] } } },
  { "id": "todo_panel.items", "role": "list", "label": "items", "parent": "todo_panel.card" },
  { "id": "todo_panel.item[1]", "role": "group", "label": "1", "parent": "todo_panel.items" },
  { "id": "todo_panel.item[1].text", "role": "label", "label": "Absorb the plev demos into the showcase",
    "parent": "todo_panel.item[1]",
    "state": { "text": { "kind": "text", "value": "Absorb the plev demos into the showcase" } } },
  { "id": "todo_panel.item[1].done", "role": "checkbox", "label": "done", "parent": "todo_panel.item[1]",
    "state": { "checked": { "kind": "bool", "value": false } },
    "actions": [ { "name": "toggle", "reads": ["todo_panel.item[1].done:checked"] } ], "focusable": true },
  (... item[2] igual, "Measure text, never estimate it", false ...)
  { "id": "todo_panel.item[3]", "role": "group", "label": "3", "parent": "todo_panel.items" },
  { "id": "todo_panel.item[3].text", "role": "label", "label": "Port the todo domain (tested first)",
    "parent": "todo_panel.item[3]",
    "state": { "text": { "kind": "text", "value": "Port the todo domain (tested first)" } } },
  { "id": "todo_panel.item[3].done", "role": "checkbox", "label": "done", "parent": "todo_panel.item[3]",
    "state": { "checked": { "kind": "bool", "value": true } },
    "actions": [ { "name": "toggle", "reads": ["todo_panel.item[3].done:checked"] } ], "focusable": true }
]
```

o `toggle` de cada checkbox lê o estado como a superfície o mostra
(`todo_panel.item[k].done:checked`), então um agente que observou só a
superfície pode marcar; o `route` devolve a observação com
`todo.item[k]:completed`.

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ bd5f107 (`cargo test --workspace` e o `cargo test` do urnaui neste worktree, antes do primeiro código) | 2372 passam, 13 ignorados | 82 passam, 1 ignorado | 2454 |
| depois, esta branch @ 626576d (`./tool/tasks/fullcheck.sh`, saída 0) | 2383 passam, 0 falhas, 13 ignorados | 82 passam, 0 falhas, 1 ignorado | 2465 |

11 testes novos: 1 da projeção (`the_items_are_projected_as_a_list_in_the_order_shown`),
5 da página (`surface_page/tests/items.rs`), 1 unitário do `propose`
(`list.rs`) e 4 das regras. a contagem não cai. clippy `-D warnings`,
`cargo fmt --check` e o check wasm do showcase com `-D warnings` verdes,
dentro do gate. `df -h /` antes do gate: 16 GB livres.

`cargo run -p showcase --example regress -- check`: 157 baselines, 157
unchanged. `bless` (no commit da lista) nas cinco que mostram o todo:
`todo@core@1280x800`, `todo@core@390x844`, `todo@hoff@1280x800`,
`toggle-group@core@1280x800`, `toggle-group@hoff@1280x800`; o relatório
de cada uma: "semantics changed: items none -> [1, 2, 3]", "text none ->
..." nos três itens, "visual unchanged", 6 outros controles iguais.

### arquivo fora da lista

`tool/bench/regress/src/tests.rs`, só o teste
`another_label_in_the_baseline_is_a_semantic_change_of_that_control_only`
(decisão do orquestrador): ele troca o texto de um item na baseline, e
agora vê o label e o estado `text` mudarem, os dois em `todo.item[2]`. é o
único arquivo editado fora da lista do briefing.

### a seção de ADR

- `docs/adr/generated/0052-proposers-propose-the-runtime-verifies.md`: adendo datado "the
  rules propose the items (task-95, proposed)";
- `docs/adr/generated/0051-materialized-surfaces-from-verified-ir.md`: adendo datado "the
  todo's surface shows its items (task-95, proposed)".

linhas do índice para o orquestrador (o `docs/adr/README.md` é proibido
aqui):

`| 2026-09-24 | [proposers-propose-the-runtime-verifies](proposers-propose-the-runtime-verifies.md) | accepted, adendo da task-95 proposto | task-95: parte items (item, items, itens, list, lista) acha por estrutura uma List ou Set cujos valores são as chaves dos filhos do nó, todos; por item text, checkbox de um bool com ação sem parâmetro que só o lê, botões das outras ações sem parâmetro não destrutivas; o Namer mantém os ids do molde sobre a mesma lista; nunca remove (regra 6) |`

`| 2026-09-24 | [materialized-surfaces-from-verified-ir](materialized-surfaces-from-verified-ir.md) | accepted, adendo da task-95 proposto | task-95: o todo projeta todo:items e o text de cada item; as duas formas da página Surface repetem sobre a lista (texto e checkbox done por item), marcar pelo dispatcher do todo, a instância de cada item atravessa a troca de forma; remover fica no todo fixo |`

### o que o checkpoint b ganha

- as duas telas mostram o mesmo conteúdo: os mesmos itens, na mesma
  ordem, com o mesmo texto e o mesmo estado de feito, lidos do mesmo
  estado vivo, e seguindo o mesmo filtro. a comparação pode ser por
  conteúdo, item a item, pelo id (`todo.item[k]` no fixo,
  `todo_panel.item[k]` na gerada, a mesma chave).
- a mesma operação principal nas duas: marcar, pelo mesmo dispatcher, com
  receipts no mesmo lugar e o ator de quem agiu.
- as diferenças que sobram são de desenho, e cada uma tem causa nomeada:
  a gerada não risca o texto de um item feito (o `text` não tem prop de
  risco), não remove (regra 6, o todo não confirma), mostra o label "done"
  ao lado do checkbox (todo checkbox do IR precisa de nome), e não tem
  rolagem na lista (a lista não virtualiza, fase 1).
- uma proposta "items" chega à mesma lista por estrutura, então o
  checkpoint pode comparar a tela escrita à mão, a forma escrita como IR e
  a proposta pelas regras.

### decisões

- a lista segue o filtro. contexto: trazer todos pediria nós para itens
  que a tela esconde. razão: a projeção só mostra o que existe agora.
  consequência: um filtro que esvazia a lista deixa as formas escritas
  como IR sem verificar (um molde precisa de pelo menos um nó que case, o
  limite da task-68); a superfície aberta fica com a lista sem instância,
  e a troca de forma é recusada até a lista ter um item.
- o texto como estado do item. razão: o IR não interpola label; é o único
  jeito de uma superfície mostrar o item.
- o checkbox a partir de "uma ação sem parâmetro que lê só aquele bool",
  e não do nome `toggle`. razão: estrutura, não nome; o `stage` do ide lê
  dois estados e não é um interruptor.
- lista = todos os filhos de um prefixo. razão: separa a lista (os
  arquivos) de uma seleção deles (os staged) sem nome.
- a raiz do molde mantém o id da raiz do molde atual sobre a mesma
  lista, antes de qualquer casamento por conteúdo. razão: a instância é a
  raiz nomeada pelo item; mantê-la mantém `todo_panel.item[k]`.

### limitações

- a superfície não remove (acima); o todo precisaria de um estado de
  confirmação para um dialog com sentido.
- um dialog materializado desenha o cancel mesmo sem `cancel` ligado, e
  um dialog fechado continua projetado com o `confirm` (fase 1): para
  quem ligar uma remoção depois.
- o ide mostra status e botões por arquivo, não o caminho: o caminho só
  nomeia a instância, não é estado do nó do arquivo.
- "os itens ativos" continua fora do registro ("active" é o sentido do
  todo para `completed: false`); "only the active items" dá a lista toda
  com o chip e a contagem Active.
- o texto do item feito não é riscado na gerada; a lista não virtualiza.

### dependências pendentes e riscos de integração

- para o orquestrador: as linhas do índice de ADR (acima); o
  `docs/log/changelog.md` (a página Surface mostra e marca os itens; as regras
  propõem a lista); a consequência do ADR `plev-ir-typed-and-verified.md`
  que diz que o todo não projeta lista (fora da lista de arquivos);
  `docs/ATLAS.toml`, se quiser citar `todo:items` na entrada do
  showcase.
- a fixture de duas colunas mudou de forma (`todo_panel.main` voltou, agora
  num grid): quem a lê por índice de filho precisa conferir; os testes do
  `surface`, do `propose` e da página passam.
- as cinco baselines do `regress` foram abençoadas no commit da lista;
  a página Surface não tem baseline.
