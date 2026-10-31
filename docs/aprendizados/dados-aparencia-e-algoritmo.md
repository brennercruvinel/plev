---
id: rust-grafos-separacao
title: "Separar dados, aparência e algoritmo em Rust"
project: plev
type: aprendizado
status: referencia
date: 2026-10-27
date_basis: git
topics: [grafos, layout, importacao]
relations: [task-100, adr-0081]
evidence: [db4b3299, b8bc7639]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Separar dados, aparência e algoritmo em Rust

No grafo do plev, `Graph` guarda o que existe e `Look` descreve como cada
elemento aparece. `Layout` escolhe a disposição. Essa divisão faz com que
um leitor ELK possa reutilizar o mesmo componente que recebe dados d3.

`Node` mantém a identidade da origem como `String`, enquanto arestas
referenciam posições em `Vec<Node>` por `usize`. Isso evita referências
com lifetimes atravessando a estrutura, mas transfere ao programa a
responsabilidade de validar os índices e preservar sua correspondência.

`Graph::holds` confere se os índices estão dentro dos vetores. Não prova,
por si só, unicidade de pais nem ausência de ciclos. `descendants` usa
um vetor de visitados para não percorrer indefinidamente grupos cíclicos.
São garantias diferentes, que merecem nomes e testes diferentes.

O contrato de importação termina no modelo; o contrato de layout começa
nele. Importar JSON ELK não implementa um layout ELK. Essa fronteira é
o que permite registrar uma limitação sem acoplar a engine ao formato.

Fonte: [rust/engine/src/widgets/graph/model.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/widgets/graph/model.rs) e [task 100](../tarefas/task-100-grafos-interativos.md).

## Evidências

- [`db4b3299`](https://github.com/brennercruvinel/plev/commit/db4b32999ce980e924d15f59b196aa4b01562b05)
- [`b8bc7639`](https://github.com/brennercruvinel/plev/commit/b8bc76394e2af1597239965c85ba36588a18fdc5)
