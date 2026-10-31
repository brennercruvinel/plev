---
id: task-41
title: "task-41: license + preparation for public release"
project: plev
type: task
status: pending
date: 2023-12-14
date_basis: reconstructed
recorded: 2026-03-13
topics: [process, workspace]
relations: [task-23]
source: docs/log/steps/task-41-license-and-public-release.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-41: license + preparation for public release

Records the plan to define the plev license (MIT OR Apache-2.0) and to prepare the repository to be public: a LICENSE file, Cargo.toml metadata, README and a secrets scan. The page body's checklist is still open; according to the note itself, the public repository is enough for the arXiv paper, and publishing on crates.io is not a requirement. It matters because these are the points that help avoid legal and credibility blockers at the public release of the code.

## objetivo
definir licenca, criar arquivo license, e fazer cleanup final para que o repositorio possa ser tornado publico sem blockers legais ou de credibilidade.

## checklist

### licenca
- [ ] criar arquivo `LICENSE` na raiz com texto MIT or apache-2.0
- [ ] atualizar `Cargo.toml` (campo `license = "MIT OR Apache-2.0"`) em todos os crates do workspace
- [ ] verificar que todas as dependencias sao compativel (MIT/apache-2.0/unlicense, todas ok)

### readme.md
- [ ] verificar link `docs/narrate-syntax.md` funciona (arquivo existe)
- [ ] screenshot ou gif da showcase app no topo (aumenta retencao de visitantes)
- [ ] adicionar badge de CI (github actions) quando task-23 for validada remotamente
- [ ] revisar comandos de build, confirmar que todos funcionam do zero em maquina limpa

### cargo.toml (workspace)
- [ ] preencher campos: `description`, `repository`, `keywords`, `categories`
- [ ] verificar que `readme = "README.md"` esta correto para publicacao em crates.io (se planejado)

### seguranca
- [ ] verificar que nao ha segredos, tokens, ou dados pessoais em nenhum arquivo rastreado
- [ ] revisar `.gitignore`, confirmar que `.env`, `*.key`, `ndk/` (NDK local) estao excluidos

## notas
- MIT or apache-2.0 e padrao do ecossistema rust (wgpu, winit, taffy, leptos, etc usam isso)
- dual-license da flexibilidade maxima para usuarios e e compativel com todas as dependencias
- crates.io publicacao nao e requisito para o paper arxiv, repositorio github publico e suficiente
