---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# task-03 changelog, layout engine

## 2026-03-08
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

### problemas encontrados
- outros agentes mudando branch do working dir principal constantemente
- solucao: worktree isolado em /private/tmp/plev-task03
- cherry-picks entre branches causaram perda de pub mod layout em lib.rs, corrigido
- taffy leaf node com auto size = 0px (precisa explicit size ou flex_grow em container)
