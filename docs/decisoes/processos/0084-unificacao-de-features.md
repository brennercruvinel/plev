---
id: adr-0084
title: "A união de features reduz compilações duplicadas"
project: plev
type: decisao
status: aceito
date: 2026-10-27
date_basis: git
topics: [decisao]
relations: [task-102]
evidence: [540e218d, 706dfa82]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# A união de features reduz compilações duplicadas

## Contexto

Executar um pacote isolado e testar o workspace ativavam features
diferentes das mesmas dependências, criando artefatos adicionais.

## Decisão

Cargo-hakari gera `rust/unions` para os alvos configurados de macOS e
Linux. Os membros dependem desse crate. O gate verifica a configuração.
Wasm permanece fora dessa união de plataformas, pois usa outro diretório
de artefatos. `proc-macro2` fica excluído para respeitar a diferença de
`span-locations` entre engine e proc-macros.

## Consequências

Novas dependências e membros exigem regenerar e verificar a união.
O profile de desenvolvimento guarda tabelas de linhas para o código
local e elimina debug das dependências. Isso reduz informações disponíveis
ao depurar internamente uma dependência.

## Alternativas

Aceitar os conjuntos diferentes conserva a recompilação. A opção de
unificação do próprio Cargo citada na configuração exigia nightly na
rodada registrada; a decisão adotou a ferramenta configurada no projeto.

## Evidências

- [`540e218d`](https://github.com/brennercruvinel/plev/commit/540e218d61b8529784a9c958ce7bfd6210af60ce)
- [`706dfa82`](https://github.com/brennercruvinel/plev/commit/706dfa82c907975688797748001e2236bdb8faef)
