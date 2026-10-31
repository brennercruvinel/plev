---
id: task-69
title: "task-69: superfícies efêmeras e geradas"
project: plev
type: tarefa
status: concluido
date: 2026-09-24
date_basis: cabecalho
topics: [superficies-geradas, ir, agentes]
relations: [adr-0023, adr-0051, task-58, task-59, task-63, task-68, task-70, task-71]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---
# task-69: superfícies efêmeras e geradas

Registra a task que levou superfícies de interface a nascer de uma intenção: materializadas a partir de IR verificado, operadas por humano e agente pelo mesmo contrato, transformadas por `SemId` e dispensadas quando a tarefa termina. O resultado final está no checkpoint b: a tela gerada dos silêncios ficou usável na janela padrão, e a segunda rodada registrou um empate com vantagens diferentes, sem derrota da geração.

## decisões do orquestrador (2026-09-24)

- **ordem das fases:** 1 (materializar) agora, depois 2 (ciclo de vida e
  transformação), 3 (proponentes) e 4 (a demo do vídeo). a fase 0 (spike
  em `refs/`) não roda. a pergunta de UX dela é respondida de verdade no
  checkpoint b, com a superfície gerada de verdade.
- **o host mora num crate novo, `surface`.** ele depende de `uir`, `sem`,
  `engine` e `comps`. o `comps` continua biblioteca de widgets e não
  depende do IR.
- **a projeção sai do IR, não do `project` dos widgets.** 9 dos 21
  componentes do vocabulário não têm `project` no comps (achado da
  task-68). a superfície é semântica por construção: role por
  `uir::role_of`, label, estado lido das ligações no snapshot vivo do app,
  e as ações que o IR ligou.
- **a fase 1 prova com as fixtures da task-68**, em testes sem janela e
  numa página do showcase. a demo do vídeo continua na fase 4, como o
  plano diz. o urnaui já usa ffmpeg em subprocesso, então o domínio do
  vídeo tem base no código. a fixture da revisão de silêncios nasce na
  fase 4, com as capacidades de verdade.
- **o provedor do LLM (fase 3) é decisão do brenner**, como o arquivo
  diz. a fase 3 para ali e pergunta.

## objetivo

uma superfície de interface nasce de uma intenção, é materializada a
partir de IR verificado, é operada por humano e agente pelo mesmo
contrato, se transforma sem perder identidade e desaparece quando a
tarefa termina. nunca foi codificada como tela.

## ligação com a tese

seções 1 (o ciclo invertido), 4 (a interface deixa de ser fixa: o editor
de vídeo, os silêncios, a comparação, o espectro de fixa até gerada sob
demanda no mesmo runtime), 5 (LLM como interface semântica, plev como
interface perceptual), 15 (restrições), 18 (GPU-first para superfícies que
existem por poucos segundos), 33 (uma interface pode existir por vinte
segundos e nunca ter sido tela). goal 1 e goal 8 do programa. checkpoint b.

## estado atual no código

- não existe host de superfície materializada; o padrão oficial é estado
  em structs com widgets retained ([`0023-official-app-pattern.md`](../decisoes/estrutura/0023-official-app-pattern.md)).
- `rust/engine/src/compose/layout` (taffy) resolve layout a partir de árvore;
  `rust/engine/src/widgets` tem os widgets; `rust/engine/src/widgets/overlay` tem spring para
  entrada e saída.
- o urnaui já decodifica frames de mídia com ffmpeg em subprocesso
  (`apps/urnadbapp`, ADR `urnaui-media-frames-in-place-via-ffmpeg-subfile`).
  ffmpeg tem `silencedetect`.
- depois de 68: `VerifiedIr`, `Capabilities`.

## escopo

### fase 0: spike descartável em `refs/` (pode rodar desde a onda 4)

a revisão de silêncios montada à mão, com capacidades falsas, só para
testar a hipótese de UX cedo: uma superfície que aparece para uma tarefa é
melhor que a tela fixa? nada é integrado. entrega: o relatório do que foi
aprendido e o que muda nas tasks 68 e 69. segue a regra de `refs/` do
AGENTS.md.

### fase 1: materializador

- `SurfaceHost` (nome e lugar decididos no ADR: comps ou crate próprio):
  guarda as superfícies ativas; cada uma é um `VerifiedIr` mais as
  instâncias de widget retained indexadas por `SemId`.
- layout a partir do `Layout` do IR pelo `LayoutEngine` do engine (sem
  segundo motor de layout); tokens resolvidos contra o `Theme` atual.
- render pelos widgets do comps, com `SemId` e ligação (59).
- projeção semântica automática: a superfície é semântica por
  construção, porque o IR já carrega identidade, role, label e as ligações
  de estado e ação.
- eventos dos widgets viram `Request` pelo dispatcher (58, 59). a
  superfície nunca executa nada sozinha.

### fase 2: ciclo de vida e transformação

- criar, materializar, operar, transformar, dispensar.
- transformar = receber um novo `VerifiedIr` e reconciliar por `SemId`: o
  mesmo `SemId` mantém a instância do widget e o estado dela (um slider em
  arraste continua em arraste quando muda de painel); `SemId` novo nasce;
  `SemId` que sumiu sai com transição.
- transições de entrada, saída e movimento com `theme.duration` e o
  spring do overlay; reduced motion respeitado.

### fase 3: proponentes

- trait `Proposer`: recebe intenção (texto), `Capabilities`, o
  `Snapshot` atual e a superfície atual (se houver); devolve IR.
- proponente determinístico por regras, para teste e para demo sem rede.
- proponente LLM atrás do mesmo trait: o prompt leva o registro de
  capacidades e o schema; a resposta passa pelo verifier; os erros do
  verifier voltam para o modelo num número limitado de tentativas; IR
  inválido nunca chega ao runtime. o provedor (modelo local ou API) é
  decisão do brenner nesta task, registrada no ADR.
- o modelo propõe superfície; ação continua indo pelo dispatcher com o
  ator certo (humano ou agente), com precondição e receipt.

### fase 4: a demo do editor de vídeo

- capacidades: `detect_silence` (ffmpeg `silencedetect` real sobre um
  arquivo), `preview_segment`, `change_threshold`, `apply_cut`,
  `reject_cut`.
- "corta os silêncios desse vídeo e depois quero revisar os cortes" ->
  superfície de revisão (vídeo, contagem, previews, slider de intensidade,
  aplicar todos, revisar individualmente).
- "quero comparar a versão original com os cortes" -> a superfície se
  transforma em comparação lado a lado com as diferenças na timeline,
  mantendo identidade do que continua.
- terminada a comparação, a composição some.
- lugar: página do grupo Apps Demo do showcase primeiro; app próprio
  depois, se crescer.

### padrão de app

ADR atualizando `official-app-pattern`: superfície materializada a partir
de IR vira o segundo padrão suportado, ao lado de estado em structs com
widgets retained, com as regras de quando usar cada um.

## fora de escopo

transformações adaptativas sobre telas fixas e capacidades compostas
(70); importers (71).

## dependências

fase 0: nenhuma. fases 1 a 4: 68, 59 e 58 integradas.

## superfície sob responsabilidade

o host de superfícies (lugar no ADR), a página demo no showcase
(`apps/uigallery/src/views/apps/` com arquivo novo), o proponente,
`refs/` para a fase 0, propostas de ADR
[`0051-materialized-surfaces-from-verified-ir.md`](../decisoes/geracao/0051-materialized-surfaces-from-verified-ir.md) e a atualização de
`official-app-pattern`.

## interfaces que consome

`VerifiedIr`, `Capabilities` (68), `Dispatcher`, `Request` (58), `SemId`
opcional e helpers dos widgets, `Compositor::bind` (59), `LayoutEngine`,
`Theme`.

## interfaces que pode alterar

as próprias. mudança em widget de comps só com aprovação.

## critérios de aceitação

- teste: materializar a fixture da revisão de silêncios produz um snapshot
  com os mesmos `SemId`, roles e ações do IR.
- teste: transformar revisão em comparação preserva a instância e o estado
  de todo `SemId` que continua.
- teste: IR inválido do proponente nunca materializa; os erros voltam.
- ação disparada numa superfície gerada tem receipt com o ator certo.
- medir o tempo de intenção até primeiro frame e de IR verificado até
  primeiro frame; registrar.
- demo gravada (task-63 se integrada) e roteiro no handoff.
- checkpoint b: a superfície gerada comparada com uma tela fixa bem
  desenhada para a mesma tarefa, com o brenner, resultado escrito.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

materialização, reconciliação por `SemId`, dispensa, proponente
determinístico, loop de verificação do proponente LLM com um cliente falso,
viewports estreito e largo da demo.

## riscos e perguntas abertas

- latência de modelo: a superfície pode chegar tarde para a tarefa; a
  medição decide se o proponente precisa de cache, de modelo local ou de
  proposta parcial.
- usabilidade: a hipótese de que superfície gerada é melhor pode falhar.
  é para isso que existem a fase 0 e o checkpoint b.

## commits esperados

1. `task-69: spike da revisão de silêncios em refs` (relatório; o código
   fica em refs/)
2. `task-69: host de superfícies e materialização a partir de VerifiedIr`
3. `task-69: reconciliação por SemId e transições`
4. `task-69: proponente determinístico`
5. `task-69: proponente LLM com loop de verificação`
6. `task-69: capacidades do editor de vídeo com silencedetect`
7. `task-69: demo de revisão e comparação no showcase`

## pare e reporte se

- o layout do IR não mapear no `LayoutEngine` sem motor novo;
- a reconciliação exigir estado de widget que o comps não expõe;
- a escolha de provedor de LLM (sempre decisão do brenner).

## Fases

1. [fase 1: materializador de superfícies](task-69-ephemeral-generative-surfaces/fase-1.md): o host `surface` materializa a partir de `VerifiedIr`; a página Surface mostra o todo à mão e o materializado.
2. [fase 2: transformação por SemId](task-69-ephemeral-generative-surfaces/fase-2.md): reconciliação por `SemId`, transições e dispensa; o texto, o caret e o arraste sobrevivem quando o widget é o mesmo.
3. [fase 3a: proponentes sem provedor](task-69-ephemeral-generative-surfaces/fase-3a.md): o trait `Proposer`, o proponente por regras e o laço do modelo atrás de `Model`, provado com um modelo falso.
4. [fase 3b-i: o Claude como modelo do proponente](task-69-ephemeral-generative-surfaces/fase-3b-i.md): o Claude pelo `claude -p` como processo, no crate `claude-model`; as corridas reais do exemplo.
5. [fase 3b-ii: a página pede a tela ao Claude](task-69-ephemeral-generative-surfaces/fase-3b-ii.md): escolha do proponente, pedido fora da thread do app, espera visível e cancelamento.
6. [fase 4a: o app dos silêncios e a tela fixa](task-69-ephemeral-generative-surfaces/fase-4a.md): modelo puro, parser do `silencedetect`, trabalhador do ffmpeg e a tela fixa da revisão.
7. [fase 4b: a tela gerada dos silêncios](task-69-ephemeral-generative-surfaces/fase-4b.md): a tela gerada com frames reais pelo trait `Frames`, a comparação e o fechamento.
8. [fase 4c: gravação e a demo gravada](task-69-ephemeral-generative-surfaces/fase-4c.md): o showcase grava a sessão como o ide; a demo do editor de vídeo fica em dois arquivos `.monster`.
9. [checkpoint b, primeira rodada](task-69-ephemeral-generative-surfaces/checkpoint-b-primeira-rodada.md): a comparação do orquestrador entre a tela fixa e a gerada; as duas gerações saem piores de usar.
10. [checkpoint b, as correções](task-69-ephemeral-generative-surfaces/checkpoint-b-correcoes.md): linha, slider, número e prompt corrigidos; o regress passa a encenar a tela gerada.
11. [checkpoint b, segunda rodada](task-69-ephemeral-generative-surfaces/checkpoint-b-segunda-rodada.md): a mesma comparação com o binário corrigido; a gerada deixou de ser pior.

