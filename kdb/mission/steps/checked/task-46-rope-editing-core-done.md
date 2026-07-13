---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-10
domain: task-tracking
---

# task-46: rope, nucleo de edicao de texto

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
- adr [text-editing-core-follows-helix](../../../adr/text-editing-core-follows-helix.md)
- commit e0d6e3c (rename de crates), entrada no changelog unreleased (editor_core to rope)

## fora de escopo
- ui, gpu, render de cursor (isso e do text_input da engine e do ide)
- syntax highlighting, lsp
