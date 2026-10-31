---
id: adr-0085
title: "Cada alvo web tem um comando e uma faixa de portas"
project: plev
type: decisao
status: aceito
date: 2026-10-27
date_basis: git
topics: [decisao]
relations: [task-102]
evidence: [aa51e7d3, 5de80aad]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# Cada alvo web tem um comando e uma faixa de portas

## Contexto

Um servidor deixado aberto ocupava a porta fixa e impedia outra execução.
Os alvos também precisavam selecionar seus próprios arquivos do Trunk.

## Decisão

`webtarget.sh` recebe o nome do alvo e encaminha o modo e os argumentos
ao Trunk correspondente. `serve` tenta a porta inicial e as próximas
50 portas; `--port` explícito prevalece. `build` não procura portas.

## Consequências

O comando informa a URL escolhida. A consulta usa `lsof` ou `nc`; se
nenhum estiver disponível, o Trunk informa uma eventual ocupação. A
sondagem não reserva a porta: outro processo ainda pode ocupá-la antes
do servidor abrir o socket.

## Alternativas

Falhar sempre na porta fixa exigiria descobrir e encerrar outro servidor
mesmo quando uma porta vizinha estivesse livre.

## Evidências

- [`aa51e7d3`](https://github.com/brennercruvinel/plev/commit/aa51e7d33788a679259251c4803eec0d2c9194c3)
- [`5de80aad`](https://github.com/brennercruvinel/plev/commit/5de80aad9378495c53dd053a17fefbe6a0d0c72e)
