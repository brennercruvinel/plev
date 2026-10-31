---
id: adr-0084
title: "Cada alvo web tem um comando e uma faixa de portas"
project: plev
type: decisao
status: aceito
date: 2026-10-27
date_basis: git
topics: [web-wasm, plataforma]
relations: [task-102]
evidence: [aa51e7d3, 5de80aad]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Cada alvo web tem um comando e uma faixa de portas

## Contexto

Um servidor deixado aberto ocupava a porta fixa e impedia outra execução: a falha
aparecia depois do build, com "Address already in use" (`5de80aad`). O script
conhecia só a galeria e o explorador; `graphs` e `primes` precisavam de um `cd`
para a pasta do alvo (`aa51e7d3`).

## Decisão

`tool/tasks/webtarget.sh` recebe o nome do alvo (`uigallery`, `urnadbapp`,
`graphs` ou `primes`, linhas 19 a 27) e encaminha o modo e os argumentos ao Trunk
correspondente. `serve` tenta a porta inicial do alvo e as próximas 50 portas
(linhas 49 a 56); `--port` explícito prevalece (linha 47). `build` não procura
portas (linha 61).

## Consequências

O comando informa a URL escolhida (linhas 57 e 58). A consulta usa `lsof` ou `nc`
(linhas 32 a 43); se nenhum estiver disponível, o Trunk informa uma eventual
ocupação. A sondagem não reserva a porta: outro processo ainda pode ocupá-la antes
do servidor abrir o socket.

## Alternativas

Falhar sempre na porta fixa exigiria descobrir e encerrar outro servidor
mesmo quando uma porta vizinha estivesse livre.

## Evidências

- [`aa51e7d3`](https://github.com/brennercruvinel/plev/commit/aa51e7d33788a679259251c4803eec0d2c9194c3)
- [`5de80aad`](https://github.com/brennercruvinel/plev/commit/5de80aad9378495c53dd053a17fefbe6a0d0c72e)
