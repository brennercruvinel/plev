---
id: learning-cycle-0053
title: "what the autonomous cycle taught: the silences app is the fixed screen the generated one is compared against"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [generated-surfaces, semantics, agents]
relations: [adr-0053, task-69, task-79]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: the silences app is the fixed screen the generated one is compared against

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0053](0053-the-silences-app-is-the-fixed-screen.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, checkpoint b, segunda rodada

- **uma orientação de prompt corta mais do que o alvo:** "só o que a
  intenção precisa" tirou a repetição que o checkpoint pediu e também o
  frame de cada corte, que é como a pessoa vê o que vai cortar. regra:
  toda orientação de prompt nova é conferida na rodada seguinte contra o
  que a tela fixa considera central, não só contra o defeito que a
  motivou. fonte: [`r2-04-generated-claude.png`](../../../Tasks/task-69-ephemeral-generative-surfaces/img/r2-04-generated-claude.png).
- **a mesma tela, duas escalas:** a primeira rodada saiu a 1200x800
  físicos e a segunda a 2400x1600 (a janela abriu numa tela retina). o
  espelho entrega o físico; para comparar rodadas, reduzir ao lógico.

## 2026-09-24, checkpoint b, primeira rodada

- **o mínimo de um texto num flex era um glifo:** o taffy toma o
  min-content como mínimo de um item, e o `TextMeasurer` mede o
  min-content com largura zero. qualquer linha apertada do IR espremia o
  texto em vez de quebrar. fonte: handoff das correções, sonda (a),
  `rust/screen/src/layout.rs` (`longest_word`).
- **a tela gerada nunca tinha sido vista inteira na janela padrão:** os
  testes da task-69 provavam a projeção, as ações e os frames, e o
  regress só tinha a Silences no estado inicial. a primeira vez que
  alguém olhou a revisão gerada a 1200x800, a linha do corte estava
  quebrada. regra: toda tela gerada que a tese usa tem imagem de
  referência no regress no estado em que a pessoa a usa.
- **o espelho da task-79 virou a ferramenta do checkpoint:** o
  orquestrador operou pelo contrato e fotografou pelo espelho, sem
  tocar a tela do brenner. é o jeito de repetir o checkpoint igual.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0053: the silences app is the fixed screen the generated one is compared against](0053-the-silences-app-is-the-fixed-screen.md)
- [task-69: ephemeral and generated surfaces](../../../Tasks/task-69-ephemeral-generative-surfaces/task-69-ephemeral-generative-surfaces.md)
- [task-79: scene over socket](../../../Tasks/task-79-scene-over-socket/task-79-scene-over-socket.md)
<!-- /related -->
