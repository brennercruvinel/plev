---
id: paper-brief-strengths
title: "plev differentiators for the paper"
project: plev
type: draft
status: draft
date: 2023-01-14
date_basis: header
topics: [paper, competition]
relations: []
source: docs/pub/brief-strengths.md
source_revision: 5de80aad
revised: 2026-10-30
review: reviewed
---

# plev differentiators for the paper

List of six technical differentiators of plev, written as a draft for the paper on 2023-01-14. Each item names a property of the engine: dirty tracking by hash, a glyph atlas with growth and LRU eviction, a signal system comparable to leptos at 1/50 of the size, an analytic spring solver, SDF with Inigo Quilez's formula and premultiplied alpha across the whole stack. The page is a list of claims and does not carry a measurement or reference for each item.

- dirty tracking correto (hash-based, o melhor padrao que vi nos repos analisados)
- text rendering funcional com atlas management completo (grow, evict, LRU)
- signal system equivalente ao leptos em 1/50 do tamanho
- spring solver analitico que a maioria dos frameworks nem tenta
- SDF rendering com inigo quilez formula (padrao da industria)
- premultiplied alpha enforcement em toda a stack (zero bugs de blending possiveis)
