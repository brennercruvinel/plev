---
id: learning-cycle-0068
title: "what the autonomous cycle taught: accessibility is production-ready per platform"
project: plev
type: learning
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [accessibility, platform, semantics, web-wasm]
relations: [adr-0068, task-75]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: accessibility is production-ready per platform

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0068](0068-accessibility-is-production-ready-per-platform.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-25, task-75 fase 1

- **"tem accessibility" media 6 páginas de 82:** a árvore do AccessKit só
  tem conteúdo onde a página projeta significado; as 76 páginas de
  componente o leitor ouve como a janela sozinha. um teste que percorre
  todas as páginas e guarda listas exatas do que falta pega isso; o
  VoiceOver manual numa página só não pegava. fonte:
  `apps/uigallery/tests/access_tree.rs`.
- **a matriz de plataforma só vale lida no lock:** o iOS compila o
  `null.rs` do `accesskit_winit` 0.32.2 e joga fora toda atualização, e
  nenhum adapter do lock tem custom actions. a documentação não diz nem
  uma coisa nem outra. fonte: a sonda no arquivo da task-75.
- **o tema padrão falha contraste no texto secundário:** o cinza do core
  dá 4,13 sobre o fundo (AA pede 4,5); 102 de 301 pares falham nos quatro
  temas. mudar cor é decisão de design e mexe em todo baseline, então o
  teste guarda a lista e acusa quando um par muda de lado. fonte:
  `rust/engine/src/styling/theme/tests/contrast.rs`.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0068: accessibility is production-ready per platform](0068-accessibility-is-production-ready-per-platform.md)
- [task-75: production-ready platform accessibility](../../../Tasks/task-75-platform-accessibility/task-75-platform-accessibility.md)
<!-- /related -->
