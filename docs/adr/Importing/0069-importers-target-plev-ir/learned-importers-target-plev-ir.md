---
id: learning-cycle-0069
title: "what the autonomous cycle taught: importers target plev IR, with a report in four categories"
project: plev
type: learning
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [importing, ir, formats]
relations: [adr-0069, task-71]
source: docs/log/learnings/general.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# what the autonomous cycle taught: importers target plev IR, with a report in four categories

Records what the experiments and implementations of the autonomous cycle of September 2026 taught about [ADR 0069](0069-importers-target-plev-ir.md), in the words written at the time (in Portuguese), newest first, each with its source.

## 2026-09-25, task-71 fase 1

- **o importer perdia a semântica antes de emitir:** o `ParserNode` era
  normalizado para o builder, então `<button>` e `<div>` eram o mesmo
  `Tag::Div` e os atributos sumiam. o IR não precisou mudar; o que faltava
  era carregar a origem de cada nó e prop, de forma aditiva. regra: um
  emissor novo lê a representação intermediária antes da normalização
  para o emissor antigo, ou a estende sem mexer no que o antigo lê.
  fonte: `rust/import/src/tree.rs`, `Origin`.
- **"nada some" só se prova com um censo independente:** comparar o
  relatório com o que o resolver contou prova só o resolver. o teste lê o
  censo direto do parse (cada elemento, atributo, declaração, chamada) e
  achou dois silêncios antigos na droplist. fonte:
  `rust/import/tests/plevir.rs`.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0069: importers target plev IR, with a report in four categories](0069-importers-target-plev-ir.md)
- [task-71: plev as compiler target (importers to IR)](../../../Tasks/task-71-compiler-target/task-71-compiler-target.md)
<!-- /related -->
