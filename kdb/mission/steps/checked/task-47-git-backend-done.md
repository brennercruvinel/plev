---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-24
domain: task-tracking
---

# task-47: git, backend para apps plev

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
- adr [git-backend-gix-reads-cli-mutations](../../../adr/git-backend-gix-reads-cli-mutations.md)
- antes era `git_backend`, virou `git` na reorganizacao por crates

## fora de escopo
- build pure-gix (sem dependencia do git no PATH)
- mobile
