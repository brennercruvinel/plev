---
id: task-99
title: "task-99: integração do explorador e correções de plataforma"
project: plev
type: tarefa
status: concluido
date: 2026-10-16
date_basis: git
topics: [urna, plataforma, processo]
relations: [adr-0026, task-76, task-98]
evidence: [05fe993c, 701d2993, c836ea73, ac2906a2, 14fd20dd, 7f4b9994]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Task 99 — Integração do explorador e correções de plataforma

Depois da reorganização, o explorador precisava acompanhar os caminhos do
projeto urna, e o workspace ainda tinha diferenças entre os builds nativos.

## Implementação

O embedder de consulta procura os scripts `potionqry.py` e `presetqry.py` em
`rust/bridge/python/urna` (caminho do repositório) e em
`<raiz de dados>/urna/python`, onde `urna setup` os instala
([embed.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/apps/urnadbapp/src/backend/workers/embed.rs),
linhas 34 e 175 a 177; `701d2993`). Na árvore atual não há nenhum `potionqry.py`
versionado: a busca está no código e os scripts vêm do projeto urna.
O vetor recebido continua sujeito à validação de modelo, dimensão e hash
(`embed.rs`, linhas 215 a 247). A interface recebe mensagens do worker; criar
um processo de embedding não ocorre na thread que desenha a janela
(`embed.rs`, linha 18).

O executor compartilhado drena stdout e stderr desde o início, em threads
separadas. Esperar a saída do filho antes de consumir os pipes pode criar
um impasse quando a saída excede o buffer do sistema. O timeout mata e
recolhe o processo. O executor nasceu em `05fe993c` (2026-09-18), em
`crates/urnaui/src/model/subprocess.rs`, com essa drenagem; `d3e65544` o moveu
para `apps/urnadbapp`. Código:
[apps/urnadbapp/src/backend/workers/subprocess.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/apps/urnadbapp/src/backend/workers/subprocess.rs).

## Plataforma e reprodução

As correções também selecionaram o backend xdg-portal de `rfd` no Linux
(`c836ea73`), restringiram o import de background ao macOS (`ac2906a2`),
trataram a preferência de movimento reduzido ausente no macOS (`7f4b9994`) e
fizeram as páginas staged das linhas de base (silences) deixarem de oferecer o
Claude instalado no host (`14fd20dd`).

Essas alterações tratam causas diferentes e não equivalem a certificar
todas as plataformas. Os commits abaixo delimitam cada correção.

## Aprendizado em Rust

Um `cfg` precisa acompanhar tanto a implementação quanto os imports, como
mostra `ac2906a2`. Uma tarefa movida para um worker ainda pode bloquear esse
worker: subprocessos exigem um contrato de saída, drenagem concorrente e término
controlado. Ver [ADR 26](../decisoes/renderizacao/0026-worker-results-wake-the-event-loop.md).

## Evidências

- [`05fe993c`](https://github.com/brennercruvinel/plev/commit/05fe993ce2eac37e92f9b79bb523329da3ffdf6e)
- [`701d2993`](https://github.com/brennercruvinel/plev/commit/701d2993266b090acdfc13f641d88df2d57c74e6)
- [`c836ea73`](https://github.com/brennercruvinel/plev/commit/c836ea732afd915e14f8109144ebded99a4ccf34)
- [`ac2906a2`](https://github.com/brennercruvinel/plev/commit/ac2906a27cea431a90551f5818fda94840ad1dd7)
- [`14fd20dd`](https://github.com/brennercruvinel/plev/commit/14fd20dd77f405d7ca71270c07067f4be55e054b)
- [`7f4b9994`](https://github.com/brennercruvinel/plev/commit/7f4b99941b987b3fca31333804423a6689daf91f)
