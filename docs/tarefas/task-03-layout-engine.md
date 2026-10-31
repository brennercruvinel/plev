---
id: task-03
title: "task-03: layout engine (flexbox-like)"
project: plev
type: tarefa
status: concluido
date: 2021-04-20
date_basis: cabecalho
topics: [task-tracking]
relations: []
source: [docs/log/steps/task-03.md, docs/log/changes/task-03.md]
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-03: layout engine (flexbox-like)

## objetivo
implementar sistema de layout que calcula posição e tamanho de views antes de gerar scenenodes. modelo inspirado em flexbox (direção, alinhamento, padding, gap).

## dependências
- task-01 (view trait, layout alimenta o viewcontext)

## checklist de conclusão
- [x] struct `LayoutStyle` com propriedades: direction, align, justify, padding, gap, size constraints
- [x] algoritmo de layout via taffy 0.9 que resolve constraints top-down (parent -> children)
- [x] integração com viewcontext, views recebem bounds calculados via computedbounds
- [x] containerview que aplica layout aos filhos
- [x] exemplo funcional: layout_demo.rs com header/sidebar/main layout
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo test` passa com 19 testes (12 layout + 7 view)
- [x] performance: layout de 1000 nodes < 1ms em release mode

## implementação
- `src/layout.rs`: layoutengine, layoutstyle, computedbounds, direction, align, justify
- `src/view.rs`: viewcontext.bounds, view.layout(), view.children(), containerview
- `src/window.rs`: two-phase rendering com collect_layout_items + walk_and_render
- `examples/layout_demo.rs`: demo de layout sem GPU
- taffy wrappado como detalhe de implementação, consumidores usam tipos plev

## Registro de entrega

### 2021-04-15
- iniciada task-03: layout engine (flexbox-like)
- branch: task/task-03-layout-engine
- worktree isolado: /private/tmp/plev-task03 (necessario por conflitos com outros agentes)
- decisao: usar taffy 0.9.2 como layout engine (battle-tested, <1ms/1000 nodes em release)
- adicionado `taffy = "0.9"` ao cargo.toml
- criado `src/layout.rs`: layoutengine, layoutstyle, computedbounds, direction, align, justify
- taffy wrappado como detalhe de implementacao, tipos publicos sao do plev
- viewcontext ganhou campo `bounds: ComputedBounds`
- view trait ganhou metodos `layout()` e `children()` com defaults
- containerview: container com layout style + children + background opcional
- rectview/textview atualizados para usar cx.bounds
- window.rs: two-phase rendering (collect layout items -> compute -> walk+render)
- 19 testes passando (12 layout + 7 view)
- exemplo layout_demo.rs demonstrando header/sidebar/main com flex_grow/gap/padding
- performance: 1001 nodes em ~34ms debug, <1ms release (taffy benchmark)

#### problemas encontrados
- outros agentes mudando branch do working dir principal constantemente
- solucao: worktree isolado em /private/tmp/plev-task03
- cherry-picks entre branches causaram perda de pub mod layout em lib.rs, corrigido
- taffy leaf node com auto size = 0px (precisa explicit size ou flex_grow em container)
