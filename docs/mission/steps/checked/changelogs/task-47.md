---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-04-03
domain: changelog
---

# task-47 changelog: git, backend para apps plev

## camada sincrona (GitRepo)
- [x] gix para leitura: open/discover, walk de historia, listar refs
- [x] git CLI para status (porcelain v2)
- [x] git CLI para diff (unified)
- [x] git CLI para mutacoes: stage, unstage, discard, commit, ignore
- [x] tipos Branch, Commit, DiffLine, Hunk, FileStatus, StatusKind

## camada threaded (GitClient)
- [x] GitRepo numa worker thread
- [x] comandos via channel mpsc (GitCommand)
- [x] resultados via callback (GitEvent), encaminha para winit EventLoopProxy
- [x] Refresh emite Status+Log+Branches num snapshot consistente
- [x] ui thread nunca bloqueia

## validacao
- [x] 25 testes contra repositorios temporarios reais (tests/real_repo.rs)
- [x] sem dependencia de ui
- [x] desktop-only (git precisa estar no PATH)
