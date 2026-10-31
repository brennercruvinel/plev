---
id: task-69/checkpoint-b-primeira-rodada
title: "task-69, checkpoint b: primeira rodada"
project: plev
type: secao
parent: task-69
status: referencia
date: 2026-09-24
date_basis: texto
topics: [superficies-geradas, agentes, processo]
relations: [task-70]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# checkpoint b, primeira rodada (2026-09-24, orquestrador, a pedido do brenner)

Registra a primeira rodada do checkpoint b, em que o orquestrador compara a tela fixa com a gerada pelas regras e pelo Claude, pelo contrato do agente e por fotos do espelho. O resultado: a gerada pelas regras quebrava na janela padrão, e a do Claude era pior de usar; decidiu-se corrigir os três defeitos de implementação antes de aplicar a regra do checkpoint.


o brenner pediu ("testa você mesmo") que o orquestrador fizesse a
comparação. feita pelo contrato, sem clique sintético: o showcase na
Silences com `PLEV_AGENT=1 PLEV_MIRROR=1 PLEV_BACKGROUND=1 PLEV_RECORD=1`,
as ações por `plev act` como o agente `orquestrador`, e cada tela
fotografada pelo espelho da task-79 (`mirror --frames 1 --png`), na
janela de 1200x800. imagens em `checkpoint-b/`.

- **tela fixa** ([`02-fixed-detected.png`](../../imagens/task-69-checkpoint-b/02-fixed-detected.png)): o frame do vídeo, a linha do
  tempo com os dois silêncios, a detecção, os contadores, e a lista com
  frame, faixa, "2.0 s of silence", o estado e aplicar/rejeitar. o
  segundo corte fica abaixo da dobra.
- **gerada pelas regras** ([`03-generated-rules.png`](../../imagens/task-69-checkpoint-b/03-generated-rules.png), instantânea, grátis):
  quebrada na janela padrão. na linha do corte, os textos saem um
  caractere por linha (a coluna encolhe a quase zero), os números saem
  crus ("3.018594") e os botões "preview segment" e "apply cut" passam
  da borda direita.
- **gerada pelo Claude** ([`04-generated-claude.png`](../../imagens/task-69-checkpoint-b/04-generated-claude.png), 119,2 s, 2
  tentativas, 0,196 USD): completa em função (arquivo, duração, frame
  original e resultado, detecção, resumo com os contadores, e a lista
  com frame, faixa, estado, duração, prévia, aplicar e rejeitar por
  corte), mas pior de usar: a lista, que é a tarefa, vem por último,
  abaixo da dobra; os controles de detecção se repetem, com os sliders
  sem rótulo nem valor na tela; os dois frames são iguais sem nada que
  diga a diferença; as durações saem cruas ("1.996916").
- **o que funcionou:** o estado é um só. aplicar o primeiro corte pela
  fixa e rejeitar o segundo pela gerada deixou as duas telas e os
  contadores iguais ([`05-after-task.png`](../../imagens/task-69-checkpoint-b/05-after-task.png)); cada ação passou pelo
  dispatcher; um propose foi recusado como stale porque o próprio agente
  tinha mudado a intenção depois de olhar, e passou depois de observar.
- **veredito desta rodada: a gerada é pior de usar hoje.** duas causas
  são defeitos de implementação que valem para toda tela gerada (a linha
  encolhe numa coluna estreita; números sem formato); a terceira é da
  proposta (a tarefa vem por último e a tela repete o que a fixa mostra).
  nenhuma imagem de referência cobre a tela gerada aberta, e por isso o
  defeito de layout passou por todas as fases.
- **decisão do orquestrador:** julgar a tese por um defeito de layout
  não é justo. antes de aplicar a regra do checkpoint (gerada pior, a 70
  vai para transformação sobre telas fixas), os três defeitos são
  corrigidos como bug do núcleo e o checkpoint roda de novo do mesmo
  jeito, com as mesmas imagens. a regra do programa não muda.
