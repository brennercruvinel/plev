---
id: task-52
title: "task-52: modularizacao por single responsibility principle"
project: plev
type: tarefa
status: concluido
date: 2024-05-29
date_basis: cabecalho
topics: [task-tracking]
relations: []
source: [docs/log/steps/task-52-srp-modularization.md, docs/log/changes/task-52.md]
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-52: modularizacao por single responsibility principle

## objetivo
nenhum arquivo .rs acima de 300 linhas. quebrar os monolitos em submodulos por responsabilidade unica, sem mudar a API publica.

## dependencias
- task-51 (reestruturacao do workspace) como contexto de organizacao
- nenhuma bloqueadora

## contexto
o codebase acumulou 27+ arquivos acima de 300 linhas, o maior (`narrate_runtime.rs`) com 1219. tipos, logica de execucao, API publica e testes coexistiam num arquivo so. um arquivo com varias razoes para mudar e dificil de revisar e de cocriar com llm. (o limite ecoa a regra do dono de modularizar quando o arquivo passa de algumas centenas de linhas; uma passada anterior usou ~369, a decisao consolidou em 300.)

## o que foi entregue
- limite de 300 linhas por arquivo .rs.
- 44 monolitos convertidos em submodulos por responsabilidade: tipos (types.rs), execucao (execution/engine/processor), API publica (api.rs ou mod.rs com pub use), testes (tests.rs ou tests/), utilitarios.
- re-export via mod.rs com `pub use` mantem a API publica identica (zero breaking change): `crate::signal::readsignal` segue funcionando apos a divisao.
- de ~60 para 271 arquivos .rs; maior arquivo 300 linhas; distribuicao 45% (0-100), 31% (101-200), 24% (201-300).
- 470 testes continuam passando sem modificacao.

## numeros honestos
- a divisao expoe `pub(crate)` / `pub(super)` entre submodulos e distribui impl blocks por arquivo (rust permite, exige disciplina).
- so divisao estrutural, nenhum refactor de logica.

## referencias
- adr [adr-003-srp-modularization](../decisoes/estrutura/0015-srp-modularization.md) (decisao formal)
- adr [srp-modularization](../referencia/arquitetura/srp-modularization.md) (detalhes de sessao, armadilhas, metricas antes/depois)
- adr [clippy-zero-warnings](../referencia/arquitetura/clippy-zero-warnings.md) (a passada de lint que acompanhou)
- commit b9d9ee7 (divisao por SRP dos arquivos grandes)

## fora de escopo
- refactor de logica ou de comportamento
- mudanca de API publica

## Registro de entrega

### divisao
- [x] limite de 300 linhas por arquivo .rs
- [x] 44 monolitos divididos em submodulos por responsabilidade
- [x] maior arquivo era 1219 linhas (narrate_runtime.rs)
- [x] padrao: types / execution / api / tests / utils

### preservacao de API
- [x] re-export via mod.rs com pub use (API publica identica)
- [x] zero breaking change (crate::signal::readsignal segue funcionando)
- [x] pub(crate)/pub(super) entre submodulos

### metricas
- [x] de ~60 para 271 arquivos .rs
- [x] distribuicao 45% (0-100), 31% (101-200), 24% (201-300)
- [x] 470 testes continuam passando sem modificacao
