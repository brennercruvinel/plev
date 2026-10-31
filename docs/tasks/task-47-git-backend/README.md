---
id: task-47
title: "task-47: git, backend for plev apps"
project: plev
type: task
status: done
date: 2026-04-08
date_basis: header
topics: [git, platform]
relations: [adr-0020, task-48]
source: [docs/log/steps/task-47-git-backend.md, docs/log/changes/task-47.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-47: git, backend for plev apps

Records the task for the `git` crate, the backend for the apps: history, status, diff and every mutation, with a synchronous `GitRepo` and a `GitClient` on a worker thread, so the UI never blocks on git. The result is 25 tests against real temporary repositories. `git` must be on the PATH, and the pure-gix build is deferred until gix has a stable porcelain.

## objetivo
backend git completo (history, status, diff, stage/unstage/discard/commit) para apps plev, sem nunca travar a ui no frame loop e sem ter que montar a superficie de porcelain do git na mao.

## dependencias
- nenhuma bloqueadora (crate folha, sem dep de ui)
- consumido por task-48 (ide)

## contexto
duas falhas a evitar: fazer git na ui thread trava o frame loop (status ou diff num repo grande engasga o render), e montar status/diff/mutacoes a partir das plumbing crates do gix na mao e uma superficie grande e fragil. a decisao e a rota pragmatica do Zed.

## o que foi entregue
- crate `git`, sem dependencia de ui. duas camadas:
  - `GitRepo`, API sincrona. gix para o lado de leitura (open/discover do repo, walk de historia, listar refs). git CLI para status (porcelain v2), diff (unified) e toda mutacao (stage/unstage/discard/commit), formatos estaveis para tooling.
  - `GitClient`, roda um `GitRepo` numa worker thread. comandos entram por um channel mpsc, resultados saem por callback (tipicamente encaminhado para um `winit::EventLoopProxy`). a ui thread nunca bloqueia.
- enums `GitCommand` (Status, Log, Branches, DiffWorkdir, DiffCommit, Stage, Unstage, Discard, Ignore, Commit, Refresh, Shutdown) e `GitEvent`. `Refresh` emite Status+Log+Branches de uma vez (usado apos mutacao e por file watcher) para a ui reconciliar updates otimistas contra um snapshot consistente.
- tipos `Branch`, `Commit`, `DiffLine`, `Hunk`, `FileStatus`, `StatusKind`.

## numeros honestos
- 7 arquivos .rs, ~1085 LOC, 25 testes contra repositorios temporarios reais (`tests/real_repo.rs`).
- desktop-only (no mobile o ide usa um clipboard fallback, sem git ui).
- git precisa estar no PATH. um build pure-gix removeria isso mas exigiria dono da superficie de mutacao/porcelain na mao, adiado ate gix ter porcelain estavel.

## referencias
- adr [git-backend-gix-reads-cli-mutations](../../adr/structure/0020-git-backend-gix-reads-cli-mutations.md)
- antes era `git_backend`, virou `git` na reorganizacao por crates

## fora de escopo
- build pure-gix (sem dependencia do git no PATH)
- mobile

## Delivery record

### camada sincrona (GitRepo)
- [x] gix para leitura: open/discover, walk de historia, listar refs
- [x] git CLI para status (porcelain v2)
- [x] git CLI para diff (unified)
- [x] git CLI para mutacoes: stage, unstage, discard, commit, ignore
- [x] tipos Branch, Commit, DiffLine, Hunk, FileStatus, StatusKind

### camada threaded (GitClient)
- [x] GitRepo numa worker thread
- [x] comandos via channel mpsc (GitCommand)
- [x] resultados via callback (GitEvent), encaminha para winit EventLoopProxy
- [x] Refresh emite Status+Log+Branches num snapshot consistente
- [x] ui thread nunca bloqueia

### validacao
- [x] 25 testes contra repositorios temporarios reais (tests/real_repo.rs)
- [x] sem dependencia de ui
- [x] desktop-only (git precisa estar no PATH)
