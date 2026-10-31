---
id: task-100
title: "task-100: grafos interativos e importação do corpus"
project: plev
type: tarefa
status: concluido
date: 2026-10-27
date_basis: git
topics: [grafos, layout, plataforma]
relations: [adr-0081, adr-0082]
evidence: [a38eb003, da4be63a, db4b3299, 08420f58, b526a9bd, 4f079852, b8bc7639]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Task 100 — Grafos interativos e importação do corpus

O demo de grafos passou a desenhar dados de d3 e de Cytoscape com o modelo e os
componentes do plev, no desktop e na web (`b526a9bd`, já na main). O leitor ELK
e as páginas `elk` e `elk-groups` (`b8bc7639`) estão na branch
`feat/elk-ir-from-gramorph`, ainda fora da main; por isso a tarefa fica como
`em-andamento`.

## Arquitetura

`Graph` descreve nós, arestas e grupos (`rust/engine/src/widgets/graph/model.rs:12`);
`Look` descreve a aparência; `Layout` escolhe a disposição, force ou compound
(`layout/mod.rs:22`). Os leitores de formatos ficam no demo, fora do modelo da
engine. `LiveGraph` guarda posições, simulação, seleção, gesto e grade de hit
testing (`live_graph/mod.rs:77`). Arrastar um grupo movimenta seus descendentes
(teste `a_group_drag_brings_its_children_and_a_child_drag_grows_the_box`); pan e
zoom mudam a transformação sem reconstruir a grade (`08420f58`).

`App::with_view` hospeda o canvas por `HostedView` (`runtime/window/view.rs:15`).
A janela converte coordenadas físicas em lógicas, entrega eventos e pede novos
frames enquanto a view se move (`runtime/window/lifecycle.rs:175` a `191`). Ver os
ADRs [81](../decisoes/estrutura/0081-grafo-dados-aparencia-layout.md)
e [82](../decisoes/renderizacao/0082-view-hospedada-sob-demanda.md).

## Corpus e limites

Na main, as páginas são `lesmis` e `compound` (`demo/graphs/src/lib.rs:38`);
`elk` e `elk-groups` existem só na branch, e aparecem no código de `ec0a224c`
(`demo/graphs/src/lib.rs:46`, lista `Page::ALL` com quatro páginas). Nome de página inválido produz erro;
não abre silenciosamente a primeira (`4f079852`, na main). O leitor ELK converte
a estrutura, mas o layout em camadas solicitado pela origem ainda não existe no
widget: as páginas usam os layouts force e compound (`b8bc7639`, na branch).

O commit `b8bc7639` relata 14 nós dentro das caixas de seus grupos após
400 frames, pelo relatório `PLEV_REPORT`. É a medição registrada na entrega, não
uma garantia para qualquer grafo. Os testes de `LiveGraph` cobrem interações e
invariantes (`08420f58`); o relatório de posição do demo permite conferir o corpus.

## Código

- [rust/engine/src/widgets/graph/model.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/widgets/graph/model.rs)
- [rust/engine/src/widgets/graph/live_graph/mod.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/widgets/graph/live_graph/mod.rs)
- [rust/engine/src/widgets/graph/live_graph/tests/mod.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/widgets/graph/live_graph/tests/mod.rs)
- [demo/graphs/src/read.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/demo/graphs/src/read.rs)
- [demo/graphs/src/lib.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/demo/graphs/src/lib.rs)

## Aprendizado em Rust

Separar dados, aparência e algoritmo permite testar e substituir cada
parte sem incorporar o formato de origem à API da engine. Índices em
vetores exigem validação explícita: `Graph::holds` verifica limites; essa
verificação não deve ser descrita como prova de toda propriedade topológica.

## Evidências

- [`a38eb003`](https://github.com/brennercruvinel/plev/commit/a38eb00392e9d783c236953237890844ab75c1c2)
- [`da4be63a`](https://github.com/brennercruvinel/plev/commit/da4be63ab386c4a7218add9e320ef6e60d842a1a)
- [`db4b3299`](https://github.com/brennercruvinel/plev/commit/db4b32999ce980e924d15f59b196aa4b01562b05)
- [`08420f58`](https://github.com/brennercruvinel/plev/commit/08420f58d0ffc2d5367929224bf1c48e82a1f8a4)
- [`b526a9bd`](https://github.com/brennercruvinel/plev/commit/b526a9bd3439a58111b33b667d6647ed9a992a5a)
- [`4f079852`](https://github.com/brennercruvinel/plev/commit/4f07985284e00363c5a1ee76c1b4e6e9f5aa0a56)
- [`b8bc7639`](https://github.com/brennercruvinel/plev/commit/b8bc76394e2af1597239965c85ba36588a18fdc5) (branch `feat/elk-ir-from-gramorph`, fora da main)
