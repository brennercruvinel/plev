---
id: adr-0083
title: "A união de features reduz compilações duplicadas"
project: plev
type: decisao
status: aceito
date: 2026-10-27
date_basis: git
topics: [workspace, desempenho]
relations: [task-102]
evidence: [540e218d, 706dfa82]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# A união de features reduz compilações duplicadas

## Contexto

Executar um pacote isolado e testar o workspace ativavam features diferentes
das mesmas dependências, criando artefatos adicionais. Na medição de `540e218d`,
cada membro recompilava crates já compiladas no workspace: 54 em `-p arcade`, 65
em `-p uigallery` e 7 em `-p commit`.

## Decisão

Cargo-hakari gera `rust/unions` para os alvos configurados de macOS e Linux
(`.config/hakari.toml`, linhas 24 a 27). Os membros dependem desse crate. O gate
verifica a configuração: `tool/tasks/fullcheck.sh` e um job de CI rodam
`cargo hakari generate --diff` e `cargo hakari manage-deps --dry-run` (`540e218d`).
Wasm permanece fora dessa união de plataformas, pois usa outro diretório de
artefatos. `proc-macro2` fica excluído para respeitar a diferença de
`span-locations` entre engine e proc-macros (`.config/hakari.toml`, linhas 29 a 37;
`rust/engine/Cargo.toml`, linhas 35 a 43).

## Consequências

Novas dependências e membros exigem regenerar e verificar a união. O profile de
desenvolvimento guarda tabelas de linhas para o código local e elimina debug das
dependências (`Cargo.toml`, linhas 164 a 175; `706dfa82`). Isso reduz informações
disponíveis ao depurar internamente uma dependência; a sessão que precisa de
variáveis usa `--profile debugging` (linhas 177 a 181).

## Alternativas

Aceitar os conjuntos diferentes conserva a recompilação. A opção de unificação
do próprio Cargo, citada em `.config/hakari.toml` (linhas 6 e 7), exigia nightly
na rodada registrada; a decisão adotou a ferramenta configurada no projeto.

## Evidências

- [`540e218d`](https://github.com/brennercruvinel/plev/commit/540e218d61b8529784a9c958ce7bfd6210af60ce)
- [`706dfa82`](https://github.com/brennercruvinel/plev/commit/706dfa82c907975688797748001e2236bdb8faef)
