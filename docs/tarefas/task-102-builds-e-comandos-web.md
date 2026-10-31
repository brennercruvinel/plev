---
id: task-102
title: "task-102: builds reutilizáveis e comandos web"
project: plev
type: tarefa
status: concluido
date: 2026-10-27
date_basis: git
topics: [workspace, web-wasm, desempenho]
relations: [adr-0083, adr-0084]
evidence: [540e218d, 706dfa82, aa51e7d3, 62cb40e8, 5de80aad, f57c9054, 229f046b]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Task 102 — Builds reutilizáveis e comandos web

O trabalho reduziu recompilações causadas por conjuntos de features
diferentes e reuniu os comandos para executar os alvos web.

## Dependências

`rust/unions`, gerado por cargo-hakari, solicita a união de features usadas
pelos membros do workspace nos alvos configurados: macOS e Linux; wasm fica de
fora (`.config/hakari.toml`, linhas 21 a 27). `proc-macro2` fica excluído:
`span-locations`, que liga o hot reload da engine (`rust/engine/Cargo.toml`,
linhas 35 a 43), não deve ser copiado para a compilação dos macros
(`.config/hakari.toml`, linhas 29 a 37). Na medição de `540e218d`, cada membro
recompilava crates já compiladas no workspace: 54 em `-p arcade`, 65 em
`-p uigallery` e 7 em `-p commit`; depois da união, zero em cada caso.

O profile de desenvolvimento também limita informações de debug: linhas de
tabela para o código local e nenhuma para as dependências (`Cargo.toml`, linhas
164 a 175). Medido em `706dfa82`, com `cargo test -p engine --no-run` em target
limpo num Mac M-series, o tempo foi de 96 s e 3,8 GB para 85 s e 2,1 GB.
Ver [ADR 26](../decisoes/processos/0083-unificacao-de-features.md).

## Execução web

`webtarget.sh` aponta o trunk para a pasta de cada alvo (`apps/uigallery/web`,
`apps/urnadbapp/web`, `demo/graphs` e `demo/primes`), onde estão o `Trunk.toml`
e o `index.html`, e aceita `uigallery`, `urnadbapp`, `graphs` ou `primes`
(`tool/tasks/webtarget.sh`, linhas 22 a 27). Em `serve`, começa pela porta do
alvo (8080 a 8083) e procura a próxima livre, até 50 portas acima; um `--port`
explícito prevalece. O modo `build` encaminha os argumentos sem procurar portas
(linhas 45 a 61).

O explorador recuperou seu binário web próprio (`62cb40e8`), e os comandos do
README passaram a apresentar linhas executáveis (`f57c9054`, `229f046b`).
Código: [tool/tasks/webtarget.sh](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/tool/tasks/webtarget.sh)
e [.config/hakari.toml](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/.config/hakari.toml).

## Aprendizado em Rust

Compilar um pacote e compilar o workspace podem ativar features diferentes das
mesmas dependências, e cada variante vira um artefato a mais em `target/`
(`540e218d`). A configuração de unificação precisa respeitar a separação entre
dependências de host, proc-macros e alvo: copiar a feature `span-locations` para
o host faria crescer cada `Span` em `rust/macros` (`.config/hakari.toml`, linhas
29 a 33).

## Evidências

- [`540e218d`](https://github.com/brennercruvinel/plev/commit/540e218d61b8529784a9c958ce7bfd6210af60ce)
- [`706dfa82`](https://github.com/brennercruvinel/plev/commit/706dfa82c907975688797748001e2236bdb8faef)
- [`aa51e7d3`](https://github.com/brennercruvinel/plev/commit/aa51e7d33788a679259251c4803eec0d2c9194c3)
- [`62cb40e8`](https://github.com/brennercruvinel/plev/commit/62cb40e8bea9c25d1bf0de183a4fa04b773c2384)
- [`5de80aad`](https://github.com/brennercruvinel/plev/commit/5de80aad9378495c53dd053a17fefbe6a0d0c72e)
- [`f57c9054`](https://github.com/brennercruvinel/plev/commit/f57c905434d2a9cf99e21bc7998beccdc5f31b9b)
- [`229f046b`](https://github.com/brennercruvinel/plev/commit/229f046b71a7135716c6d9064063478009ee9f92)
