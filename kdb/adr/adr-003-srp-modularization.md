---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-04-05
domain: modularization
---

# adr-003: modularizacao por single responsibility principle

## status

aceita (2026-04-05)

## contexto

o codebase do phi engine acumulou 27 arquivos .rs acima de 300 linhas,
com o maior (narrate_runtime.rs) atingindo 1219 linhas. multiplas
responsabilidades coexistiam em arquivos unicos: definicoes de tipos,
logica de execucao, API publica e testes.

metricas pre-refatoracao:
- 27 arquivos acima de 300 linhas
- maior arquivo: 1219 linhas (narrate_runtime.rs)
- responsabilidades misturadas em todos os modulos centrais

## decisao

adotar limite maximo de 300 linhas por arquivo .rs, dividindo modulos
monoliticos em submodulos por responsabilidade unica.

padrao de divisao aplicado:
1. tipos e structs -> types.rs ou arquivo nomeado pela entidade
2. logica de execucao -> execution.rs, engine.rs, ou processor.rs
3. API publica -> api.rs ou mod.rs com pub use re-exports
4. testes -> tests.rs (ou tests/ diretorio se > 300 linhas)
5. utilitarios -> utils.rs, helpers.rs

convencao de re-export: mod.rs declara submodulos e usa `pub use`
para manter a API publica identica (crate::signal::readsignal continua
funcionando apos a divisao de signal.rs em signal/mod.rs).

## consequencias

positivas:
- cada arquivo tem uma unica razao para mudar
- navegacao e code review simplificados
- testes isolados por responsabilidade
- compilacao incremental mais eficiente (unidades menores)

negativas:
- maior numero de arquivos (de ~60 para 271)
- necessidade de pub(crate)/pub(super) para visibilidade entre submodulos
- impl blocks distribuidos entre arquivos (rust permite, mas exige disciplina)

neutras:
- API publica inalterada (zero breaking changes)
- 470 testes continuam passando sem modificacao

## metricas pos-refatoracao

- 0 arquivos acima de 300 linhas
- maior arquivo: 300 linhas (message_dock/ui.rs)
- 271 arquivos .rs totais
- distribuicao: 45% (0-100 linhas), 31% (101-200), 24% (201-300)
