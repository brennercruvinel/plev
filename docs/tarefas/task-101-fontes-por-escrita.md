---
id: task-101
title: "task-101: fontes sob demanda e fallback determinístico"
project: plev
type: tarefa
status: concluido
date: 2026-10-27
date_basis: git
topics: [fontes, texto, web-wasm]
relations: [adr-0078, adr-0079, task-74]
evidence: [f79400d1, e2bdaa45, bee96c97, 9bdc266f, c243b124]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Task 101 — Fontes sob demanda e fallback determinístico

A entrega separou duas decisões: reduzir os bytes iniciais do wasm e
fazer o desktop preferir as mesmas faces embutidas usadas pela web.

## Carregamento web

O wasm mantém as faces latinas embutidas: Inclusive Sans, Inter e codicons
(`rust/engine/src/drawing/text/fonts.rs`, linhas 206 a 226). O texto solicita os
pacotes Noto das escritas que utiliza; cada arquivo precisa conferir com o sha256
esperado, e o pacote só entra inteiro (`on_demand.rs`, linhas 218 a 237). A chegada
de fontes incrementa uma geração (linha 243), invalida medidas e shaping e acorda
o loop (linhas 246 e 273 a 277). Medidor e renderer carregam as faces na mesma
ordem de `PACKS`, independentemente da ordem de chegada da rede (`on_demand.rs`,
linhas 9 a 13 e 254 a 271).

O [ADR 78](../decisoes/renderizacao/0078-the-web-fetches-the-noto-faces-on-demand.md)
registra o tamanho medido na entrega (6,97 MB de fontes no wasm, linha 23) e a
necessidade de servir `fonts/` ao lado do build (linha 39). Um arquivo que não
confere com o hash deixa o pacote fora; o texto segue com fallback ou glifo ausente.

## Desktop

`EmbeddedFirst` escolhe, para cada escrita coberta pelas faces embutidas, a família
Noto dessa escrita antes das listas da plataforma; as demais escritas mantêm as
listas do sistema como fallback (`fonts.rs`, linhas 160 a 199). O
[ADR 79](../decisoes/renderizacao/0079-a-desktop-draws-the-scripts-with-the-embedded-faces.md)
registra a comparação da página Typography, igual byte a byte entre o render
headless do desktop e o build web (linhas 50 e 51).

## Código e aprendizado

[rust/engine/src/drawing/text/on_demand.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/drawing/text/on_demand.rs) e
[rust/engine/src/drawing/text/fonts.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/drawing/text/fonts.rs).

Invalidar apenas o cache de shaping não basta: `resolve_layer_text` marca todas
as camadas como sujas, porque uma camada limpa ainda guarda o texto como foi
desenhado antes da chegada da face (`f79400d1`).

## Evidências

- [`f79400d1`](https://github.com/brennercruvinel/plev/commit/f79400d1614813f8e2ed7d465da958b3a0d99522)
- [`e2bdaa45`](https://github.com/brennercruvinel/plev/commit/e2bdaa456eb7c5c20f2386e30908c9e853b532fe)
- [`bee96c97`](https://github.com/brennercruvinel/plev/commit/bee96c970a711d43e8754ea9683bb5f3dd7ddb13)
- [`9bdc266f`](https://github.com/brennercruvinel/plev/commit/9bdc266f916b29702c5524973e5bdd2a50c909e3) (merge do PR #139)
- [`c243b124`](https://github.com/brennercruvinel/plev/commit/c243b1248ac191f8974a7727b2822b9d48a3aee6) (merge do PR #140)
