---
project: plev
audience: [ai-agents, contributors]
status: pending
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
- o golden da projeção do todo em `crates/uir/fixtures` acompanha;
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
(`crates/showcase/src/view/apps/app/tests/probe_95.rs`, rodado com
`cargo test -p showcase --lib probe_95 -- --nocapture` e jogado fora
depois). ele monta o `TodoPage` real, lista os nós `todo.item[...]` e todo
estado `List` ou `Set` da projeção, pede o filtro active pelo dispatcher
do todo (`Actor::Test`) e lista de novo; verifica à mão um IR que repete
sobre `todo:items`; e lê o golden do workbench do ide
(`crates/uir/fixtures/ide-workbench.snapshot.json`), verifica e
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
- aditivo: o diff do golden `crates/uir/fixtures/showcase-todo.snapshot.json`
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
em `crates/showcase/examples/regress/tests.rs` (o briefing não lista
`crates/showcase/examples/**`; é "o resto do showcase"). o `relabel` do
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

recomendação: 2, e a autorização para `crates/showcase/examples/regress/tests.rs`
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
