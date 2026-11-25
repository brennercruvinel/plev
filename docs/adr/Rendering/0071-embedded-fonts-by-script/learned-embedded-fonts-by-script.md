---
id: learning-cycle-0071
title: "what the autonomous cycle taught: embed Noto Sans subsets for the non-Latin scripts"
project: plev
type: learning
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [fonts, text, web-wasm, mobile, determinism]
relations: [adr-0071, task-74]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: embed Noto Sans subsets for the non-Latin scripts

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0071](0071-embedded-fonts-by-script.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-24, task-74 fase 1

- **o texto do plev hoje é latino:** com as 9 faces embutidas, árabe,
  hebraico, devanágari, chinês, japonês e coreano viram tofu, e o tofu
  (9,9 px em 16 px) erra a largura medida nos dois sentidos, então até a
  quebra de linha sai errada. o que já funciona sem fonte: a ordem bidi
  (inclusive dígitos em nível 2), as marcas combinantes (o shaper compõe)
  e o kerning. o que não existe: `liga` nas duas faces e o caminho de cor
  no atlas (emoji tem largura e nenhuma tinta, mesmo com Apple Color
  Emoji). embutido e desktop são dois produtos: a mesma frase mede 251 px
  embutida e 151 px com a fonte do sistema. a fase 2 (embutir subsets
  Noto, fallback de sistema, ou os dois) é decisão do brenner, com a
  estimativa de tamanho no relatório. fonte:
  [`task-74-text-and-editing.md`](../../../Tasks/task-74-text-and-editing/task-74-text-and-editing.md), "relatório da fase 1".

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0071: embed Noto Sans subsets for the non-Latin scripts](0071-embedded-fonts-by-script.md)
- [task-74: text and editing](../../../Tasks/task-74-text-and-editing/task-74-text-and-editing.md)
<!-- /related -->
