---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-23
domain: generative-runtime
task: task-67
wave: 5
depends: [task-59, task-66]
branch: task/67-structural-regression
---

# task-67: regressão estrutural antes de pixel

## objetivo

um teste de regressão descobre primeiro o que mudou (semântica, layer),
depois se aquilo deveria ter mudado, e só então compara pixels, quando é
relevante. a regressão chega nomeada pelo controle, não como contagem de
pixels.

## ligação com a tese

seção 26 (testes estruturais e visuais coexistem; "settings.security
changed / control: encryption.toggle / semantic state unchanged / visual
representation changed" no lugar de "snapshot mismatch: 37,821 pixels").
da rodada de ideias: regressão visual por hash de cena, não por pixel;
custo zero quando nada mudou; a diferença vem com o nome da layer.

## estado atual no código

- cada layer tem hash de conteúdo para dirty tracking
  (`crates/engine/src/compositor/layer/mod.rs:43`, `FxHasher`). o hash é
  determinístico dentro do mesmo build; muda quando a estrutura de dados do
  nó muda, mesmo sem mudança visual.
- validação de visual por pixel é regra do repositório (convenções,
  tests); não há harness de baseline por página.
- depois de 59: ligações `SemId -> rect, layer`. depois de 66: render
  headless e `--semantics`.

## escopo

1. baseline por (página, tema, viewport): o snapshot semântico (JSON), a
   lista de layers com um nome estável e o hash de conteúdo, e a imagem só
   quando pedida.
2. `check`: projeta e constrói a cena; compara semântica (diff da task-57)
   e hashes; se nada mudou, termina sem renderizar; se um hash mudou,
   renderiza os dois lados pelo headless e faz o diff de pixel só na região
   da layer; cruza a região mudada com as ligações para dizer quais
   controles foram afetados.
3. relatório por `SemId`: semântica mudou ou não, representação visual
   mudou ou não, qual layer, quantos pixels, e a região.
4. `bless`: atualiza o baseline, com o relatório do que está sendo aceito.
5. hash estável entre commits: o hash de dirty tracking não serve para
   baseline de longo prazo; esta task define um hash de conteúdo estável e
   documentado para o baseline (sem mexer no `FxHasher` do dirty
   tracking), ou documenta que baseline muda com a estrutura do nó e o
   fluxo cai no diff de pixel.
6. job de CI (macOS, onde o render é o mesmo do desenvolvimento).

## fora de escopo

comparação entre plataformas (77); regressão de interação (64).

## dependências

59 e 66 integradas.

## superfície sob responsabilidade

um exemplo ou binário de regressão (`cargo run -p showcase --example
regress -- check|bless` ou nome aprovado), os baselines em fixtures do
showcase, `.github/workflows` (o job), proposta de ADR
`docs/ADR/regression-by-meaning-before-pixels.md`.

## interfaces que consome

`Snapshot::diff` (57), `Compositor::bindings` (59), headless (66), layers
do compositor (leitura).

## interfaces que pode alterar

leitura de hash e nome de layer no compositor, por adição.

## critérios de aceitação

- sem mudança: `check` roda sem renderizar nada e termina verde.
- mudar a cor de um token em `scales.rs` ou `hoff.rs`: relatório diz
  "semântica igual, visual mudou" para os controles afetados, com região.
- mudar uma label: relatório diz "semântica mudou" para aquele `SemId`.
- tempo do `check` sem mudança medido e registrado.
- `./script/gate` verde.

## testes esperados

testes do comparador com cenas pequenas (igual, só hash, só semântica,
os dois), do cruzamento região para `SemId`, e do relatório.

## riscos e perguntas abertas

- página que anima (spinner, blink de caret) precisa de relógio fixo no
  modo de regressão; o harness injeta.

## commits esperados

1. `task-67: baseline semântico e de layers por página`
2. `task-67: check sem render quando nada mudou`
3. `task-67: diff de pixel por layer e cruzamento com SemId`
4. `task-67: relatório por controle e bless`
5. `task-67: job de CI`

## pare e reporte se

- o nome estável de layer não existir sem mudar a API do compositor;
- a animação não puder ser congelada pelo relógio.
