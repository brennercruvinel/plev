---
id: task-54
title: "task-54: core theme, buttons poc"
project: plev
type: tarefa
status: concluido
date: 2026-09-21
date_basis: cabecalho
topics: [design-system]
relations: []
source: docs/log/steps/task-54-core-theme-buttons-poc.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-54: core theme, buttons poc

## objetivo
o tema principal do plev (HOFF dark glass) não bate com o design system
que o brenner mantém em typescript (kit privado: next +
tailwind, tema claro, Inter, shades 1..9, sombras medidas). a série de
tasks migra o comps pra esse design, um componente por vez, deletando o
widget antigo e reescrevendo do zero sobre o plev engine.

## resultado
feito e no showcase (aba Buttons, tema core default). gate verde.

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
o kit typescript privado (Button, SubmitButton, a página de botões, a
página de cores e a de depths), lido localmente; nada dele entra no repo.
