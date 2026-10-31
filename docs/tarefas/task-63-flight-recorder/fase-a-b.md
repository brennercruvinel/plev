---
id: task-63/fase-a-b
title: "task-63 fase a e b: trilha semântica e visual em memória"
project: plev
type: secao
parent: task-63
status: referencia
date: 2026-09-23
date_basis: texto
topics: [gravacao, semantica, formatos]
relations: [task-60, task-73]
evidence: [7e63db3]
source: docs/log/steps/task-63-flight-recorder.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# fases a e b fechadas (2026-09-23)

Registra a implementação em memória das fases a e b. O `Recorder` recebe observações, snapshots, estágios de receipt e input da pessoa como `ReceiptSink`, e a trilha visual sobe os frames pelo `raise` e pelo `discover` do monster, num anel limitado por tempo e por bytes que corta sempre num keyframe. Integrada na `main` em `7e63db3`, com gate verde de 1782 testes e custo medido de 12 µs por frame em média no todo do showcase.

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
