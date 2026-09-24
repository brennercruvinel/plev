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
