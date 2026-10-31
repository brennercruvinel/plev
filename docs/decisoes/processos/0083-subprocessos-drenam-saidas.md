---
id: adr-0083
title: "Subprocessos drenam as saídas enquanto executam"
project: plev
type: decisao
status: aceito
date: 2026-10-16
date_basis: git
topics: [decisao]
relations: [task-99, adr-0026]
evidence: [701d2993]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# Subprocessos drenam as saídas enquanto executam

## Contexto

Uma resposta de embedding ou frame de mídia pode superar o buffer do
pipe. Esperar o filho terminar antes de ler stdout pode bloquear os dois.

## Decisão

O executor compartilhado abre stdout e stderr em pipe e inicia sua
drenagem em threads separadas antes de esperar a saída. O processo recebe
stdin fechado. O timeout encerra e recolhe o filho; o resultado distingue
falha ao iniciar, timeout e saída não zero. O embedding valida o JSON e
a identidade do modelo antes de usar o vetor.

## Consequências

A thread de interface não espera o subprocesso. O worker ainda tem um
limite de execução. Um teste envia uma saída maior que o buffer do pipe.
O código usa `read_to_end`: drenagem concorrente não significa limite de
memória para uma saída arbitrariamente grande.

## Alternativas

Ler os pipes somente depois de `wait` preservaria o impasse. Descartar
stderr removeria o diagnóstico de falhas do embedder.

## Evidências

- [`701d2993`](https://github.com/brennercruvinel/plev/commit/701d2993266b090acdfc13f641d88df2d57c74e6)
