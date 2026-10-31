---
id: task-73
title: "task-73: incremental dependency system"
project: plev
type: task
status: in-progress
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [performance, layout, rendering, semantics]
relations: [task-39, task-57, task-59, task-66, task-69, task-73/phase-1-measurements, task-82, experiment-frame-cost-by-stage]
evidence: [91fdb33, ed814da]
source: docs/log/steps/task-73-incremental-dependency-system.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-73: incremental dependency system

This page tracks task-73, the incremental dependency system that propagates minimal invalidation across state, semantics, layout, scene, hit test and rendering. Phases 1 and 1b, for measurement and GPU timing via timestamp queries, are closed and integrated into `main` (`ed814da` and `91fdb33`). The measurement showed that the frame bottleneck is the GPU, and the orchestrator's decision took phase 2 as far as the GPU; phase 2 still depends on the design ADR and on Brenner's decision, and phase 3 implements it per stage.

## fase 1b fechada (2026-09-23)

integrada na `main` em `91fdb33` (quatro commits), validada: gate verde,
identidade de pixel conferida pelo orquestrador (Todo hoff com o mesmo
sha256 da `main`). `RenderConfig::gpu_timestamps` (desligado por padrão;
desligado, o device não pede feature nenhuma) e os estágios `GpuSpan`,
`GpuLayers` e `GpuComposite`, lidos com um frame de atraso para não
travar a GPU. as tabelas estão na seção "fase 1b" de
[`phase-1-measurements.md`](phase-1-measurements.md).

o que os números mostram, em µs no M4 a escala 2:

- o trabalho da própria GPU é de 215 a cerca de 1200 por frame; a espera
  medida na fase 1 carregava de 280 a 400 de latência de envio por cima,
  em todo cenário, qualquer que seja o conteúdo.
- mesmo sem a latência, a GPU é de 4 a 12 vezes a CPU do mesmo frame.
- o pass de uma layer suja custa de 470 a 690 com 3 ou com 22 draws: segue
  os pixels, não a quantidade de coisa.
- o composite de quatro layers custa de 420 a 575 em todo frame
  desenhado; uma tecla divide o custo mais ou menos ao meio entre pass de
  layer e composite, e um frame que não reconstrói layer é só composite.

com isso a fase 2 tem os números. o desenho (ADR) compara as opções da
seção "fase 2" deste arquivo e a decisão é do brenner com o orquestrador.
uma hipótese nova ficou registrada e não testada: o pass de layer grava o
attachment multisample (`StoreOp::Store`) além do resolve.

## fase 1 fechada (2026-09-23)

integrada na `main` em `ed814da` (sete commits), rebaseada pelo
orquestrador sobre a task-59 fase b sem conflito. validada: diff lido (as
sondas são uma linha cada; desligadas custam uma leitura atômica relaxada e
um branch, a closure de itens nem roda), gate verde (1748 testes), e
identidade de pixel conferida pelo orquestrador: Todo (hoff) e Editor
(core) renderizados headless na `main` e na branch com o mesmo sha256. as
tabelas completas estão em [`phase-1-measurements.md`](phase-1-measurements.md); `cargo run --release -p measure` reproduz.

o achado que muda o plano: neste M4 o frame não é limitado pela CPU. a
espera da GPU é de 450 a 2200 µs por frame, 4 a 15 vezes a CPU do mesmo
frame (30 a 400 µs), e cresce com pixels re-renderizados (uma layer MSAA
4x inteira a cada tecla) e com layers compostas (todas as visíveis,
vazias inclusive), não com o tamanho da mudança. as etapas de CPU em que a
fase 2 foi pensada são pequenas: projeção de 9 a 14 µs por clique,
`LayoutEngine` com zero chamadas, construção de cena de 12 a 60 µs.

decisão do orquestrador:

- a fase 2 estende a cadeia de dependência até a GPU: além das chaves de
  estado, semântica, layout, cena e hit test, o desenho inclui região
  danificada por layer, nada de composite para layer vazia ou sem
  mudança, e vértices de texto mantidos por linha ou por run. o ADR de
  desenho trata os dois lados juntos.
- antes do ADR entra uma fase 1b pequena: timestamp queries da GPU (o
  `gpu_micros` do `PerfMonitor`, hoje `None`) para separar latência de
  submit de trabalho real, e a medição refeita com elas.
- o caret que pede 60 frames por segundo enquanto o campo está focado é
  defeito de render on demand, não otimização: virou a task-82.

candidatos da fase 2 levantados pela medição (nenhum implementado): compor
só layers com conteúdo e pular as que não mudaram; redesenhar só a região
danificada de uma layer suja; manter vértices de texto por linha (hoje uma
tecla no editor reemite 1551 glifos e sobe 236 KB); shape de uma run
alterada uma vez só (`cursor_x_styled` reshapeia a run inteira a cada
frame em que o caret aparece); animação só de opacidade reconstrói a cena
inteira; `measure_styled` copia a chave para uma `String` nova mesmo no
acerto de cache; todo resolve escreve o uniform de opacidade de cada
layer, suja ou não.

## objetivo

> criar um sistema de dependências incremental independente da
> representação visual, capaz de propagar invalidação mínima entre estado,
> semântica, layout, scene construction, hit testing e rendering.

"mudei 1 caractere" não pode significar reconstruir uma quantidade grande
de estado de CPU. a incrementalidade é propriedade do runtime inteiro, não
só da GPU.

## ligação com a tese

seção 19 (a cadeia: mudança, quais valores dependem dela, quais controles
foram semanticamente afetados, quais layouts refazer, quais layers
mudaram, quais buffers mudar), seção 18 (superfícies que existem por
segundos precisam ser baratas), seção 2 do programa (mantra).

## estado atual no código

- a cena é reconstruída a cada frame, sem diff retido (`docs/ATLAS.toml`,
  frame-flow); render on demand: sem mudança, sem frame.
- layer com hash de conteúdo: layer limpa custa zero GPU
  (`rust/engine/src/drawing/compositor/layer/mod.rs:43`).
- `TextMeasurer` tem cache; `LayoutEngine` (taffy) não tem cache próprio
  entre frames visível em `rust/engine/src/compose/layout/solver.rs`.
- bench de scene build em `rust/engine/benches/scene_build.rs`; monitor
  de performance em `rust/engine/src/runtime/perf` (fps, dt, cpu micros,
  memória; tempo de GPU ainda `None`).
- `rust/engine/src/runtime/signal`: runtime reativo, experimental sem ADR de
  app.
- task-39 ([`task-39-scene-cache-memoization.md`](../task-39-scene-cache-memoization/README.md),
  parada desde 2023): memoização por `PartialEq` de props (padrão b7) e
  flags de dirty separadas para layout e paint subindo na árvore (b8),
  com a nota de que o plev não tem árvore de componentes retida. esta task
  absorve a 39.

## escopo

### fase 1: medir (pode rodar na onda 3)

- instrumentar, sem mudar comportamento, o custo por estágio para cenários
  definidos: digitar um caractere num `TextField` do showcase; digitar no
  editor com um documento grande; marcar um item do todo; hover numa linha
  de lista; um frame de animação de overlay.
- estágios: projeção semântica (depois de 57), layout, chamadas de shaping
  e taxa de acerto do cache, hit test, construção de cena (nós emitidos),
  resolução de estilo (tokens lidos), hash de layer, layers sujas, bytes
  enviados à GPU.
- tabela publicada nesta task e no ADR. nenhuma otimização nesta fase.

### fase 2: desenho (depois de 57 e 59 integradas)

- chaves de dependência: estado semântico (`StateRef`), estado visual não
  semântico (hover, animação, scroll), tokens do tema, viewport.
- o grafo: quais layouts, quais nós de cena, quais layers e quais regiões
  de hit test dependem de quais chaves.
- opções a comparar no ADR, com os números da fase 1: memoização explícita
  por widget com props comparáveis (a proposta da 39); cache de layout por
  subárvore com chave de conteúdo; invalidação dirigida por chave usando
  (ou não) o runtime de signal; um sistema de query estilo incremental
  computation. a decisão é do brenner com o orquestrador.
- requisito da tese: a invalidação é independente da representação. uma
  superfície que troca de representação (task-69) não recalcula a
  semântica; um hover não invalida semântica nem layout.

### fase 3: implementar por estágio

um estágio por vez, cada um com o bench antes e depois e teste de que o
resultado visual é idêntico (headless, 66). ordem pelo ganho medido.

## fora de escopo

reescrever o modelo de app; tempo de GPU por timestamp query (registrado
como melhoria do perf).

## dependências

fase 1: nenhuma. fase 2: 57 e 59. fase 3: a decisão da fase 2.

## superfície sob responsabilidade

fase 1: `rust/engine/src/runtime/perf`, `rust/engine/benches`, um exemplo de
cenário. fases 2 e 3: definidas no ADR e aprovadas antes do briefing
(provavelmente `layout`, `compositor`, `text/measure`). proposta de ADR
`docs/adr/incremental-invalidation-by-dependency.md`. a task-39 está substituída
por esta task desde 2026-09-25 (status `superseded`).

## interfaces que consome

`perf`, benches, `Snapshot` e `StateRef` (57), ligações (59), headless
(66).

## interfaces que pode alterar

fase 1: nenhuma pública. fases 2 e 3: só as aprovadas no ADR.

## critérios de aceitação

- fase 1: tabela de custo por estágio para cada cenário, reproduzível por
  um comando.
- fase 2: ADR com a decisão e os números que a justificam.
- fase 3: por estágio, bench melhor e pixels idênticos; o cenário "mudei 1
  caractere" reduz o trabalho do estágio no número medido.
- `./tool/tasks/fullcheck.sh` verde em cada commit.

## testes esperados

fase 1: o próprio cenário como bench. fase 3: igualdade de saída antes e
depois, e teste de invalidação (mudar a chave X refaz só o que depende de
X).

## riscos e perguntas abertas

- cache errado mostra frame velho: quebra o contrato "mudança visível
  implica invalidação". todo cache novo precisa de teste de invalidação.

## commits esperados

1. `task-73: instrumentação por estágio sem mudar comportamento`
2. `task-73: cenários de medição e tabela`
3. `task-73: ADR de desenho` (fase 2)
4. `task-73: <estágio>` por estágio (fase 3)

## pare e reporte se

- a medição mostrar que o gargalo não é CPU (então o plano muda);
- um estágio exigir mudar o padrão de app.
