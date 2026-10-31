---
id: modelo-decisao
title: Modelo de decisão (ADR)
project: plev
type: modelo
status: ativo
date: null
date_basis: sem-data
topics: [editorial, adr]
relations: [convencoes]
source: docs/adr/TEMPLATE.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Modelo de decisão (ADR)

Uma decisão nova copia esta página para `docs/decisoes/<tema>/NNNN-<titulo-em-kebab>.md`,
com o próximo número livre de toda a sequência, e troca o cabeçalho: `id: adr-NNNN`,
`type: decisao`, `status: proposto` até alguém decidir, e então `aceito` (ou
`substituido`, com a decisão que a substituiu em `relations`). O título diz o que
vale depois da decisão, no presente.

## context

O que é verdade hoje e por que não basta: o código (`caminho:linha`), a medição,
o bug. Nada que o leitor precise aceitar por fé.

## decision

O que muda, como regras que o código vai seguir. Uma decisão por registro.

## consequences

O que fica fácil, o que fica difícil, o que fica de fora, e qual teste ou gate
mantém a decisão verdadeira.

## alternatives

O que foi pesado e por que perdeu.
