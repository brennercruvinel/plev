---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-06-24
domain: changelog
---

# task-46 changelog: rope, nucleo de edicao de texto

## buffer e documento
- [x] Document = Rope + Selections + History (estilo helix)
- [x] buffer sobre ropey (edits e slicing sublineares)

## transacoes
- [x] Transaction: (range, replacement) em coordenadas pre-transacao, ordenado, nao-sobreposto, atomico
- [x] change set op-based interno (retain/delete/insert) para composicao sem o texto
- [x] inversao de transacao para undo exato
- [x] Bias (before/after) para posicao em boundary de edit

## selecoes e historico
- [x] SelectionSet multi-cursor (selecoes sao posicoes, mapeiam pelos edits)
- [x] History com undo/redo (replay, sem snapshot do buffer)
- [x] movement com goal column

## validacao
- [x] 77 testes (proptest), sem ui nem gpu (testavel headless, rul-15)
- [x] bench edit.rs (build + insert/delete roundtrip)
