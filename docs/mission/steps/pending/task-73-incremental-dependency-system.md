---
project: plev
audience: [ai-agents, contributors]
status: pending
last-updated: 2026-09-23
domain: generative-runtime
task: task-73
wave: trilha (medição na onda 3)
depends: []
depends-design: [task-57, task-59]
absorbs: [task-39]
branch: task/73-incremental
---

# task-73: sistema de dependências incremental

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

- a cena é reconstruída a cada frame, sem diff retido (`docs/arc/arc.toml`,
  frame-flow); render on demand: sem mudança, sem frame.
- layer com hash de conteúdo: layer limpa custa zero GPU
  (`crates/engine/src/compositor/layer/mod.rs:43`).
- `TextMeasurer` tem cache; `LayoutEngine` (taffy) não tem cache próprio
  entre frames visível em `crates/engine/src/layout/engine.rs`.
- bench de scene build em `crates/engine/benches/scene_build.rs`; monitor
  de performance em `crates/engine/src/perf` (fps, dt, cpu micros,
  memória; tempo de GPU ainda `None`).
- `crates/engine/src/signal`: runtime reativo, experimental sem ADR de
  app.
- task-39 (`docs/mission/steps/pending/task-39-scene-cache-memoization.md`,
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

fase 1: `crates/engine/src/perf`, `crates/engine/benches`, um exemplo de
cenário. fases 2 e 3: definidas no ADR e aprovadas antes do briefing
(provavelmente `layout`, `compositor`, `text/measure`). proposta de ADR
`docs/ADR/incremental-invalidation-by-dependency.md`. mover a task-39 para
`checked/` como superseded quando esta abrir.

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
- `./script/gate` verde em cada commit.

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
