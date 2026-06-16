---
project: phi
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-03-13
domain: task-tracking
---

# task-41: license + preparacao para release publica

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
