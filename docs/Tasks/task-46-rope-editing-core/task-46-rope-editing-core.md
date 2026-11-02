---
id: task-46
title: "task-46: rope, text editing core"
project: plev
type: task
status: done
date: 2026-03-10
date_basis: reconstructed
recorded: 2026-06-24
topics: [text-editing, text]
relations: [adr-0021, task-28, task-48]
evidence: [e0d6e3c]
source: [docs/log/steps/task-46-rope-editing-core.md, docs/log/changes/task-46.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-46: rope, text editing core

Records the task for the `rope` crate, the core of pure text editing, with no UI or GPU and testable without a window: rope buffer, multi-cursor, atomic transactions and undo and redo by replay. The result is 77 tests, including proptest, and the foundation that the ide and the editable text widgets consume.

## objetivo
nucleo de edicao de texto puro, sem ui nem gpu, testavel headless: buffer rope, multi-cursor, transacoes atomicas e historico de undo/redo. base para o ide e para os widgets de texto editavel.

## dependencias
- nenhuma bloqueadora (crate folha)
- consumido por task-28 (editable text) e task-48 (ide)

## contexto
editar texto sobre uma String plana e O(n) por edit e perde a posicao do cursor a cada mudanca. amarrar o modelo de edicao a camada de gpu/ui o tornaria intestavel headless. o design segue helix, que e estudado e de producao, em vez de inventar.

## o que foi entregue
- crate `rope`, sem dependencia de ui ou gpu. `Document = Rope + Selections + History`.
- buffer e um rope (ropey), edits e slicing sublineares.
- `Transaction`: lista ordenada e nao-sobreposta de (range, replacement) em coordenadas pre-transacao, aplicada atomicamente. insercao pura = range vazio, delecao pura = replacement vazio.
- change set op-based interno (retain/delete/insert): duas transacoes compoem sem tocar o texto, e qualquer transacao inverte para undo.
- `Bias` (before/after) decide de que lado uma posicao gruda num boundary de edit; selecoes sao posicoes, entao mapeiam de graca atraves dos edits.
- `History` (undo steps), `SelectionSet` (multi-cursor), `movement` (movimento de cursor com goal column).

## numeros honestos
- 6 arquivos .rs, ~2142 LOC, 77 testes (proptest), bench `benches/edit.rs` (build + insert/delete roundtrip).
- antes do rename era `editor_core`; virou `rope` na reorganizacao por crates.

## referencias
- adr [text-editing-core-follows-helix](../../adr/Interface/0021-text-editing-core-follows-helix/0021-text-editing-core-follows-helix.md)
- commit e0d6e3c (rename de crates), entrada no changelog unreleased (editor_core to rope)

## fora de escopo
- ui, gpu, render de cursor (isso e do text_input da engine e do ide)
- syntax highlighting, lsp

## Delivery record

### buffer e documento
- [x] Document = Rope + Selections + History (estilo helix)
- [x] buffer sobre ropey (edits e slicing sublineares)

### transacoes
- [x] Transaction: (range, replacement) em coordenadas pre-transacao, ordenado, nao-sobreposto, atomico
- [x] change set op-based interno (retain/delete/insert) para composicao sem o texto
- [x] inversao de transacao para undo exato
- [x] Bias (before/after) para posicao em boundary de edit

### selecoes e historico
- [x] SelectionSet multi-cursor (selecoes sao posicoes, mapeiam pelos edits)
- [x] History com undo/redo (replay, sem snapshot do buffer)
- [x] movement com goal column

### validacao
- [x] 77 testes (proptest), sem ui nem gpu (testavel headless, rul-15)
- [x] bench edit.rs (build + insert/delete roundtrip)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0021: the text editing core follows helix: document is rope plus selections plus history](../../adr/Interface/0021-text-editing-core-follows-helix/0021-text-editing-core-follows-helix.md)
- [task-28: editable text](../task-28-editable-text/task-28-editable-text.md)
- [task-48: ide, native git client in plev](../task-48-ide-git-client/task-48-ide-git-client.md)
- [benchmark results, plev v0.2](../../Research/Exploring/benchmarks-v02.md)
<!-- /related -->
