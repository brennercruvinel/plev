---
id: task-98
title: "Task 98 — Reorganização do workspace"
project: plev
type: tarefa
status: concluido
date: 2026-10-09
date_basis: git
topics: [tarefa]
relations: [adr-0080, task-51, task-52]
evidence: [3161bb89, d3e65544, 6c4d6ff3]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# Task 98 — Reorganização do workspace

A reorganização de outubro reuniu bibliotecas em `rust/`, aplicações em
`apps/`, demonstrações em `demo/` e ferramentas em `tool/`. A sequência
preservou movimentos e alterações de conteúdo em commits separados.

## Problema e execução

A estrutura anterior misturava bibliotecas e aplicações em `crates/`,
com exemplos e ferramentas dependentes de caminhos antigos. O mapa de
migração definiu o destino dos arquivos antes dos movimentos. Depois
vieram manifests, declarações de módulos, imports, strings, testes e docs.
Os widgets de `comps` passaram ao engine; os importadores foram reunidos
em `import`; os nomes de crates e módulos estão em [TERMS](../TERMS.toml).

## Resultado e limites

O [relatório da entrega](../historia/reorganizacao-2026-10.md)
registra 1.861 renomes R100, as comparações de API e os testes da rodada.
Esses são resultados históricos, não uma execução nova da suíte.
O relatório também registra que Android e a perna do explorador de `.urna`
não foram verificados naquela máquina. A task 99 cobre trabalho posterior
nessa integração; ela não prova retrospectivamente o build Android.

## Aprendizado em Rust

Preservar comportamento não torna uma reorganização compatível com todos
os consumidores: mudar o caminho público de um tipo pode quebrar imports.
A comparação de API precisa distinguir identidade do item, caminho público
e expansão de reexports. O [ADR 80](../decisoes/estrutura/0080-workspace-por-responsabilidade.md)
registra a estrutura adotada.

## Evidências

- [`3161bb89`](https://github.com/brennercruvinel/plev/commit/3161bb89b0bb6b490b83d10b72c3f2e50d85fd72)
- [`d3e65544`](https://github.com/brennercruvinel/plev/commit/d3e6554467ad0a84ab9b6eaba01a996c17527354)
- [`6c4d6ff3`](https://github.com/brennercruvinel/plev/commit/6c4d6ff3cf292b9d9e815394843933de03c26918)
