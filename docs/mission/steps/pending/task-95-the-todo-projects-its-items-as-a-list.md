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
