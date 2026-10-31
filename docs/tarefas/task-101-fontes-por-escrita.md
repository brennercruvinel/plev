---
id: task-101
title: "Task 101 — Fontes sob demanda e fallback determinístico"
project: plev
type: tarefa
status: concluido
date: 2026-10-27
date_basis: git
topics: [tarefa]
relations: [adr-0078, adr-0079, task-74]
evidence: [f79400d1, e2bdaa45, bee96c97, 9bdc266f, c243b124]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# Task 101 — Fontes sob demanda e fallback determinístico

A entrega separou duas decisões: reduzir os bytes iniciais do wasm e
fazer o desktop preferir as mesmas faces embutidas usadas pela web.

## Carregamento web

O wasm mantém as faces latinas embutidas. O texto solicita os pacotes
Noto das escritas que utiliza; cada arquivo precisa corresponder ao hash
esperado. A chegada de fontes incrementa uma geração, invalida medidas e
shaping e acorda o loop. Medidor e renderer carregam as faces na mesma
ordem, independentemente da ordem de chegada da rede.

O [ADR 78](../decisoes/renderizacao/0078-the-web-fetches-the-noto-faces-on-demand.md)
registra os tamanhos medidos na entrega e a necessidade de servir `fonts/`
ao lado do build. Uma falha de rede ou hash não deve ser apresentada como
carregamento bem-sucedido; pode permanecer fallback ou glifo ausente.

## Desktop

`EmbeddedFirst` escolhe as famílias Noto por escrita, preservando as
fontes do sistema como fallback para cobertura adicional. O
[ADR 79](../decisoes/renderizacao/0079-a-desktop-draws-the-scripts-with-the-embedded-faces.md)
registra a comparação da página Typography entre desktop e web.

## Código e aprendizado

[rust/engine/src/drawing/text/on_demand.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/drawing/text/on_demand.rs) e
[rust/engine/src/drawing/text/fonts.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/drawing/text/fonts.rs).

Invalidar apenas o cache de shaping não basta: uma layer cujo texto
lógico não mudou pode conter glifos rasterizados antes da chegada da face.
A invalidação precisa atravessar o pipeline até a imagem apresentada.

## Evidências

- [`f79400d1`](https://github.com/brennercruvinel/plev/commit/f79400d1614813f8e2ed7d465da958b3a0d99522)
- [`e2bdaa45`](https://github.com/brennercruvinel/plev/commit/e2bdaa456eb7c5c20f2386e30908c9e853b532fe)
- [`bee96c97`](https://github.com/brennercruvinel/plev/commit/bee96c970a711d43e8754ea9683bb5f3dd7ddb13)
- [`9bdc266f`](https://github.com/brennercruvinel/plev/commit/9bdc266f916b29702c5524973e5bdd2a50c909e3)
- [`c243b124`](https://github.com/brennercruvinel/plev/commit/c243b1248ac191f8974a7727b2822b9d48a3aee6)
