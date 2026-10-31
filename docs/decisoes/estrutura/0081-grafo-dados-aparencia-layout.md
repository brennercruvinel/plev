---
id: adr-0081
title: "Um grafo separa dados, aparência e layout"
project: plev
type: decisao
status: aceito
date: 2026-10-27
date_basis: git
topics: [grafos, layout]
relations: [task-100]
evidence: [db4b3299, b819b3db, b8bc7639]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Um grafo separa dados, aparência e layout

## Contexto

Mais de um formato de origem e mais de um componente desenham grafos: `graph_view`
e `LiveGraph` recebem o mesmo `Graph` desde `b819b3db`. Acoplar formato, estilo e
posições exigiria um renderer por importador.

## Decisão

`Graph` guarda nós, arestas e grupos (`rust/engine/src/widgets/graph/model.rs:12`).
`Look` calcula a aparência por elemento, com cores resolvidas pelo tema no momento
do desenho (`look.rs:4` e `:5`). `Layout` escolhe force ou compound (`layout/mod.rs:22`
e `:26`). Leitores de d3 e Cytoscape convertem para `Graph` fora da engine, em
`demo/graphs/src/read.rs`, que já está na main (`b526a9bd` os trouxe como
`demo/corpus`). O leitor de ELK, também descrito aqui, está só em `b8bc7639`, na
branch `feat/elk-ir-from-gramorph`.

## Consequências

Uma nova entrada pode reutilizar simulação, desenho e hit testing. Um arquivo ELK
não implica suporte ao algoritmo de layout pedido pelo ELK: essa capacidade
continua limitada aos algoritmos implementados, force e compound (`b8bc7639`).

O status fica `proposto` porque a decisão descreve também o leitor ELK, que não
está na main. Ela passa a `aceito` quando essa branch for integrada, ou quando o
texto for reescrito sem o ELK.

## Alternativas

Embutir cada runtime de origem duplicaria os caminhos de interação e
renderização. Converter a estrutura mantém o contrato de desenho do plev.

## Evidências

- [`db4b3299`](https://github.com/brennercruvinel/plev/commit/db4b32999ce980e924d15f59b196aa4b01562b05)
- [`b819b3db`](https://github.com/brennercruvinel/plev/commit/b819b3db7f02164cedbfdb04b5c608ed7ab6ec0d)
- [`b8bc7639`](https://github.com/brennercruvinel/plev/commit/b8bc76394e2af1597239965c85ba36588a18fdc5) (branch `feat/elk-ir-from-gramorph`, fora da main)
