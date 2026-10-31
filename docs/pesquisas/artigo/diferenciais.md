---
id: artigo-diferenciais
title: "diferenciais do plev para o artigo"
project: plev
type: rascunho
status: rascunho
date: 2023-01-14
date_basis: cabecalho
topics: [artigo, competicao]
relations: []
source: docs/pub/brief-strengths.md
source_revision: 5de80aad
revised: 2026-10-30
review: revisado
---

# diferenciais do plev para o artigo

Lista de seis diferenciais técnicos do plev, escrita como rascunho para o artigo em 2023-01-14. Cada item nomeia uma propriedade do motor: dirty tracking por hash, atlas de glifos com crescimento e despejo LRU, um sistema de sinais comparável ao leptos em 1/50 do tamanho, solver analítico de spring, SDF com a fórmula de Inigo Quilez e alfa premultiplicado em toda a pilha. A página é uma lista de afirmações, e não traz medição nem referência para cada item.

- dirty tracking correto (hash-based, o melhor padrao que vi nos repos analisados)
- text rendering funcional com atlas management completo (grow, evict, LRU)
- signal system equivalente ao leptos em 1/50 do tamanho
- spring solver analitico que a maioria dos frameworks nem tenta
- SDF rendering com inigo quilez formula (padrao da industria)
- premultiplied alpha enforcement em toda a stack (zero bugs de blending possiveis)
