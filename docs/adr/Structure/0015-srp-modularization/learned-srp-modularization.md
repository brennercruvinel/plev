---
id: learning-cycle-0015
title: "what the autonomous cycle taught: adr-003: modularizacao por single responsibility principle"
project: plev
type: learning
status: reference
date: 2026-09-23
date_basis: text
recorded: 2026-09-23
topics: [workspace, process]
relations: [adr-0015]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: adr-003: modularizacao por single responsibility principle

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0015](0015-srp-modularization.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-23
- **regra sem gate não vale:** o limite de 639 linhas de produção por
  arquivo estava na convenção e nenhum gate conferia; três entregas do
  ciclo passaram dele sem aviso, e o próprio orquestrador contou errado
  uma vez (contou o arquivo inteiro, não a produção). agora o gate conta
  (`tool/tests/test_filepaths.rs`), com a regra exata e os
  seis arquivos que já passavam congelados no tamanho de hoje; um
  arquivo de prova com 700 linhas fez o guard falhar com o nome dele.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0015: adr-003: modularizacao por single responsibility principle](0015-srp-modularization.md)
<!-- /related -->
