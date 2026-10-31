---
id: adr-0080
title: "O workspace separa bibliotecas, apps, demos e ferramentas"
project: plev
type: decisao
status: aceito
date: 2026-10-09
date_basis: git
topics: [workspace, git]
relations: [task-98]
evidence: [d3e65544, 9e31564e, 6c4d6ff3]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# O workspace separa bibliotecas, apps, demos e ferramentas

## Contexto

A árvore em `crates/` reunia bibliotecas e aplicações (`crates/engine`,
`crates/comps`, `crates/showcase` e `crates/urnaui`, entre outros; relatório
em [reorganizacao-2026-10](../../historia/reorganizacao-2026-10.md), linhas 80 a 98).
A migração também precisava manter imports, manifests, scripts e testes coerentes.

## Decisão

Bibliotecas vivem em `rust/`, aplicações em `apps/`, demos em `demo/` e
ferramentas em `tool/`. Os widgets pertencem ao engine (`rust/engine/src/widgets`);
formatos importados convergem para `import` (`rust/import`). O
[ATLAS](../../ATLAS.toml) descreve a árvore e o [TERMS](../../TERMS.toml) registra
os renomes.

## Consequências

Os caminhos públicos mudam mesmo quando o comportamento do item permanece: o
relatório registra que nenhum comportamento mudou e que 13 de 19 crates têm API
idêntica linha a linha, com os outros 6 explicados no próprio relatório (linhas
49 e 61 a 69). Comparar a API exige traduzir os caminhos, sem esconder
incompatibilidades para consumidores. Commits de movimentos separados permitem
seguir a origem dos arquivos: `d3e65544` contém só renomes R100. As provas e os
limites estão na [task 98](../../tarefas/task-98-reorganizacao-do-workspace.md).

## Alternativas

Manter as bibliotecas e apps misturados conservaria os caminhos antigos,
mas não atenderia à estrutura-alvo documentada na migração.

## Evidências

- [`d3e65544`](https://github.com/brennercruvinel/plev/commit/d3e6554467ad0a84ab9b6eaba01a996c17527354)
- [`9e31564e`](https://github.com/brennercruvinel/plev/commit/9e31564eb75aa261d87f2a9f734a7ebdac97193c)
- [`6c4d6ff3`](https://github.com/brennercruvinel/plev/commit/6c4d6ff3cf292b9d9e815394843933de03c26918)
