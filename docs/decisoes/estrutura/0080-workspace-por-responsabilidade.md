---
id: adr-0080
title: "O workspace separa bibliotecas, apps, demos e ferramentas"
project: plev
type: decisao
status: aceito
date: 2026-10-09
date_basis: git
topics: [decisao]
relations: [task-98]
evidence: [d3e65544, 9e31564e, 6c4d6ff3]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# O workspace separa bibliotecas, apps, demos e ferramentas

## Contexto

A árvore em `crates/` reunia responsabilidades diferentes. A migração
também precisava manter imports, manifests, scripts e testes coerentes.

## Decisão

Bibliotecas vivem em `rust/`, aplicações em `apps/`, demos em `demo/` e
ferramentas em `tool/`. Os widgets pertencem ao engine; formatos importados
convergem para `import`. ATLAS descreve a árvore e TERMS registra renomes.

## Consequências

Os caminhos públicos mudam mesmo quando o comportamento do item permanece.
Comparar a API exige traduzir os caminhos, sem esconder incompatibilidades
para consumidores. Commits de movimentos separados permitem seguir a origem
dos arquivos. As provas e limites estão na task 98.

## Alternativas

Manter as bibliotecas e apps misturados conservaria os caminhos antigos,
mas não atenderia à estrutura-alvo documentada na migração.

## Evidências

- [`d3e65544`](https://github.com/brennercruvinel/plev/commit/d3e6554467ad0a84ab9b6eaba01a996c17527354)
- [`9e31564e`](https://github.com/brennercruvinel/plev/commit/9e31564eb75aa261d87f2a9f734a7ebdac97193c)
- [`6c4d6ff3`](https://github.com/brennercruvinel/plev/commit/6c4d6ff3cf292b9d9e815394843933de03c26918)
