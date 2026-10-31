---
id: task-99
title: "Task 99 — Integração do explorador e correções de plataforma"
project: plev
type: tarefa
status: concluido
date: 2026-10-16
date_basis: git
topics: [tarefa]
relations: [adr-0083, task-76, task-98]
evidence: [701d2993, c836ea73, ac2906a2, 14fd20dd, 7f4b9994]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# Task 99 — Integração do explorador e correções de plataforma

Depois da reorganização, o explorador precisava acompanhar os caminhos do
projeto urna, e o workspace ainda tinha diferenças entre os builds nativos.

## Implementação

O explorador passou a localizar os scripts de embedding no layout
`rust/bridge/python/urna` e nas instalações de `urna setup`. O vetor recebido
continua sujeito à validação de modelo, dimensão e hash. A interface recebe
mensagens do worker; criar um processo de embedding não ocorre na thread
que desenha a janela.

O executor compartilhado drena stdout e stderr desde o início, em threads
separadas. Esperar a saída do filho antes de consumir os pipes pode criar
um impasse quando a saída excede o buffer do sistema. O timeout mata e
recolhe o processo. Código: [apps/urnadbapp/src/backend/workers/subprocess.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/apps/urnadbapp/src/backend/workers/subprocess.rs)
e [apps/urnadbapp/src/backend/workers/embed.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/apps/urnadbapp/src/backend/workers/embed.rs).

## Plataforma e reprodução

As correções também selecionaram o backend xdg-portal de `rfd` no Linux,
restringiram o import de background ao macOS, trataram a preferência de
movimento reduzido ausente no macOS e impediram que páginas de regressão
oferecessem o Claude instalado no host.

Essas alterações tratam causas diferentes e não equivalem a certificar
todas as plataformas. Os commits abaixo delimitam cada correção.

## Aprendizado em Rust

Um `cfg` precisa acompanhar tanto a implementação quanto os imports.
Uma tarefa movida para um worker ainda pode bloquear esse worker:
subprocessos exigem um contrato de saída, drenagem concorrente e término
controlado. Ver [ADR 83](../decisoes/processos/0083-subprocessos-drenam-saidas.md).

## Evidências

- [`701d2993`](https://github.com/brennercruvinel/plev/commit/701d2993266b090acdfc13f641d88df2d57c74e6)
- [`c836ea73`](https://github.com/brennercruvinel/plev/commit/c836ea732afd915e14f8109144ebded99a4ccf34)
- [`ac2906a2`](https://github.com/brennercruvinel/plev/commit/ac2906a27cea431a90551f5818fda94840ad1dd7)
- [`14fd20dd`](https://github.com/brennercruvinel/plev/commit/14fd20dd77f405d7ca71270c07067f4be55e054b)
- [`7f4b9994`](https://github.com/brennercruvinel/plev/commit/7f4b99941b987b3fca31333804423a6689daf91f)
