---
id: task-63/phase-a-b
title: "task-63 phases a and b: semantic and visual trail in memory"
project: plev
type: section
status: reference
parent: task-63
date: 2026-09-23
date_basis: text
topics: [recording, semantics, formats]
relations: [task-60, task-73]
source: docs/log/steps/task-63-flight-recorder.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [7e63db3]
---

# phases a and b closed (2026-09-23)

Records the in-memory implementation of phases a and b. The `Recorder` receives observations, snapshots, receipt stages and the person's input as a `ReceiptSink`, and the visual trail carries the frames through `raise` and `discover` of the monster, in a ring bounded by time and by bytes that always cuts at a keyframe. Integrated into `main` at `7e63db3`, with a green gate of 1782 tests and a measured cost of 12 µs per frame on average across the whole showcase.

integradas na `main` em `7e63db3` (sete commits), rebaseadas pelo
orquestrador sobre a task-73 sem conflito, gate verde (1782 testes). o
crate `rec` grava a trilha semântica (observações, snapshots inteiros ou
como diff, estágios de receipt com o request inteiro no primeiro, input
sem o caractere digitado) e a trilha visual (frames pelo `monster::raise`,
frames idênticos descartados, chunks pelo `discover` e `encode`, cada
chunk um v0 com assets locais), num anel limitado por tempo e por bytes
que só corta em keyframe. a leitura (`Session`) não guarda dispatcher nem
app: ler nunca cria request. a redação falha fechado (alvo desconhecido
conta como secreto). custo medido no todo do showcase: 12 µs por frame em
média, menos de 0,1 % de um frame a 60 fps, 8,7 KB/s no anel visual.

decisões do orquestrador para a fase c:

1. formato irmão dentro do `rec` (opção b): o codec de animação e o golden
   ficam intocados por construção; o limite de `u16` vale só dentro de
   cada chunk.
2. a trilha semântica `S` é JSON lines da forma serde, um evento por
   linha, para exportar um intervalo sem decodificar a trilha visual.
3. a extensão continua `.monster`, como a tese chama a sessão gravada; a
   magic é `MONS` (animação continua `MON0`); as ferramentas decidem pela
   magic, e o `monster_player` recusa uma sessão com mensagem clara.
4. um acessor aditivo em `gpu/image.rs` devolve tamanho e pixels RGBA8 de
   uma imagem; a task-66b reaproveita.
5. identidade de build (versão do crate e sha do git) no cabeçalho, por
   um `build.rs` que nunca falha o build; a task-65 usa.
6. a gravação não é ligada em nenhum app nesta rodada: a task-60 está no
   shell do showcase; o atalho de debug e o panic hook entram depois dela.
