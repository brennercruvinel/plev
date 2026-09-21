---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-21
domain: design-system
---

# task-44: core theme, buttons poc

## objetivo
o tema principal do plev (HOFF dark glass) não bate com o design system
que o brenner mantém em typescript (tmp/hoff-research-design-sy: next +
tailwind, tema claro, Inter, shades 1..9, sombras medidas). a série de
tasks migra o comps pra esse design, um componente por vez, deletando o
widget antigo e reescrevendo do zero sobre o plev engine.

## este passo (poc)
- fonte Inter embutida (Regular, Medium, SemiBold), family por tema
- `ShadowSpec` com `inset` (as receitas do core são stacks css com inset)
- `Theme::hoff_core()`: shades, accents, sombras (button, secondary,
  warning, subtle, toolbar, slight hover), tipografia Inter -0.02em,
  raios 8/10/12/16/22, alturas 24/28/36/40/44
- `theme.button: ButtonTokens`: receita por variante (primary, secondary,
  warning, outline, ghost, toolbar, submit) e tamanho; os outros temas
  derivam uma receita chapada a partir da paleta
- `comps::action::{Button, IconButton, SubmitButton}` reescritos
- showcase `Buttons` reescrito como a ButtonsPage: Group, RowCards, Card
  com chip de título, um card por estado

## defeito global pego no caminho
- rect_sdf.wgsl misturava border/fill e os stops do gradiente em alpha
  reto: fill transparente puxava a borda pro preto na aresta interna
  anti-aliased, o cinza escuro em toda quina arredondada. agora mistura
  em premultiplicado linear
- sombra com blur 0 (os rings `0 0 0 1px`) não tinha AA: `shadow_sigma`
  tem piso de 0.5px

## fora deste passo
- tooltip (feedback), demais páginas (cards, inputs, menu, modal...)
- remover o HOFF dark glass: só depois de todas as páginas migrarem

## fonte
- tmp/hoff-research-design-sy/components/Button/index.tsx
- tmp/hoff-research-design-sy/components/SubmitButton/index.tsx
- tmp/hoff-research-design-sy/templates/ButtonsPage/index.tsx
- tmp/hoff-research-design-sy/templates/ColorsPage/content.tsx
- tmp/hoff-research-design-sy/templates/DepthsPage/index.tsx
