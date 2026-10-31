---
id: cronologia-fontes
title: "Como ler as datas da trajetória"
project: plev
type: registro
status: referencia
date: 2026-10-30
date_basis: git
topics: [editorial, git]
relations: [adr-0001, adr-0002, apps-contornam-engine, dois-bugs-gamma, estado-codigo-2026-09, task-26, task-40, tokens-hoff]
evidence: [176dd691, 704412f1, b1e820ba, 95d53881, 1a776d04, ec0a224c]
source: reconstrucao
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Como ler as datas da trajetória

O acervo contém datas declaradas desde 2021. O histórico Git disponível
tem um commit raiz em 10 de junho de 2026. Essas duas linhas temporais
representam fontes diferentes e não devem ser fundidas automaticamente.

## Datas revistas posteriormente

O commit `704412f1` alterou o cabeçalho da nota sobre `View` de
2026-03-08 para 2021-03-06 e o [ADR de layout](../decisoes/renderizacao/0002-content-driven-layout-not-fixed-constants.md) de 2026-06-10 para
2024-09-18. A mudança documenta uma revisão retrospectiva da data;
não prova, sozinha, quando o trabalho técnico ocorreu.

O commit `b1e820ba` remapeou referências de hashes após o expurgo de
histórico. Por exemplo, `34a9893` foi substituído por `55c8aa7` no [ADR de
inicialização assíncrona da GPU](../decisoes/renderizacao/0001-async-gpu-init-and-single-wasm-entry.md). Uma referência antiga pode ter uma
correspondência recuperável sem que todo o histórico anterior exista.

## Leitura da linha do tempo

As páginas mantêm a data declarada e sua base no YAML. A linha do tempo
ordena essas declarações; ela não certifica cada data como contemporânea
ao acontecimento. Quando o texto e o cabeçalho abrangem períodos distintos,
as entradas internas podem ter datas próprias. Ausência de data é
representada como ausência, nunca preenchida por conveniência editorial.

## Material recuperado

A [task 40](../tarefas/task-40-paper-arxiv-full-text.md) foi encontrada antes da exclusão em `95d53881`: seu objetivo
era escrever o artigo completo, com checklist pendente. O outline da
[task 26](../tarefas/task-26-arxiv-paper.md) concluído não demonstra que a task 40 também foi concluída.

O rascunho, o outline, os benchmarks e os diferenciais do artigo saíram
em `1a776d04`; [`truth.md`](estado-do-codigo-2026-09.md), em `ec0a224c`. Suas versões recuperadas são
fontes históricas. Os textos sobre gamma, apps contornando a engine e
tokens HOFF também foram recuperados. Os caminhos e revisões exatos
constam em `source` e no catálogo.

## Evidências

- [`704412f1`](https://github.com/brennercruvinel/plev/commit/704412f12300e0a42792745bfd6daccbfb9f666e)
- [`b1e820ba`](https://github.com/brennercruvinel/plev/commit/b1e820baf47d0be24e30890fefbcd0cfdceeb1f3)
- [`95d53881`](https://github.com/brennercruvinel/plev/commit/95d5388140568732f472c6be86205df113c8fcce)
- [`1a776d04`](https://github.com/brennercruvinel/plev/commit/1a776d04c6b392a3afad72e5b8223c116937128d)
- [`ec0a224c`](https://github.com/brennercruvinel/plev/commit/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713)
