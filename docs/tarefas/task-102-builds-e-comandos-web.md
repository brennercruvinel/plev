---
id: task-102
title: "Task 102 — Builds reutilizáveis e comandos web"
project: plev
type: tarefa
status: concluido
date: 2026-10-27
date_basis: git
topics: [tarefa]
relations: [adr-0084, adr-0085]
evidence: [540e218d, 706dfa82, aa51e7d3, 62cb40e8, 5de80aad]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# Task 102 — Builds reutilizáveis e comandos web

O trabalho reduziu recompilações causadas por conjuntos de features
diferentes e reuniu os comandos para executar os alvos web.

## Dependências

`rust/unions`, gerado por cargo-hakari, solicita a união de features
usadas pelos membros do workspace nos alvos configurados. `proc-macro2`
fica excluído: `span-locations`, necessário ao hot reload da engine,
não deve ser copiado indiscriminadamente para a compilação dos macros.
O profile de desenvolvimento também limita informações de debug.
Ver [ADR 84](../decisoes/processos/0084-unificacao-de-features.md).

## Execução web

`webtarget.sh` seleciona o Trunk.toml de `uigallery`, `urnadbapp`,
`graphs` ou `primes`. Em `serve`, começa pela porta do alvo e procura
uma livre dentro do limite configurado; uma porta explícita prevalece.
O modo `build` encaminha os argumentos sem procurar portas.

O explorador recuperou seu binário web próprio e os comandos do README
passaram a apresentar linhas executáveis. Código:
[tool/tasks/webtarget.sh](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/tool/tasks/webtarget.sh) e [.config/hakari.toml](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/.config/hakari.toml).

## Aprendizado em Rust

Compilar um pacote e compilar o workspace podem ativar features diferentes
das mesmas dependências. O lockfile sozinho não impede esses artefatos
duplicados. A configuração de unificação precisa respeitar a separação
entre dependências de host, proc-macros e alvo.

## Evidências

- [`540e218d`](https://github.com/brennercruvinel/plev/commit/540e218d61b8529784a9c958ce7bfd6210af60ce)
- [`706dfa82`](https://github.com/brennercruvinel/plev/commit/706dfa82c907975688797748001e2236bdb8faef)
- [`aa51e7d3`](https://github.com/brennercruvinel/plev/commit/aa51e7d33788a679259251c4803eec0d2c9194c3)
- [`62cb40e8`](https://github.com/brennercruvinel/plev/commit/62cb40e8bea9c25d1bf0de183a4fa04b773c2384)
- [`5de80aad`](https://github.com/brennercruvinel/plev/commit/5de80aad9378495c53dd053a17fefbe6a0d0c72e)
