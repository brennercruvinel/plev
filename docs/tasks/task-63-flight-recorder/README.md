---
id: task-63
title: "task-63: session flight recorder"
project: plev
type: task
status: done
date: 2026-09-24
date_basis: header
recorded: 2026-09-23
topics: [recording, semantics, formats]
relations: [task-57, task-58, task-60, task-61, task-64, task-65, task-66b, task-73, task-77, task-88, task-89, task-90, adr-0037]
evidence: [c62a05d, 7e63db3, 8e6e814]
source: docs/log/steps/task-63-flight-recorder.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-63: session flight recorder

Records the plan and the result of task-63: recording a plev app session as data, to explain what happened without re-executing anything. The semantic track (observations, snapshots, receipts and the person's input) and the visual track (frames through `raise` and through monster's `discover`) are kept in a bounded ring and persisted in the `MONS` format, a sibling of the animation codec. The phases are integrated on `main` at `7e63db3` (a and b), `c62a05d` (c) and `8e6e814` (d); the final result is the ide recording its own session so that `plev-inspect` can answer why an action was refused.

## objetivo

gravar uma sessão de app plev como dado, não vídeo, de um jeito que
explique o que aconteceu: quem observou o quê, quem pediu qual ação sobre
qual estado, qual foi o veredito, o delta, a execução e o resultado, e o
que estava na tela naquele instante. manter os últimos N segundos num ring
buffer limitado, persistir sob demanda ou em panic, exportar um trecho.
replay inspecionável é a promessa; reexecução não é.

## ligação com a tese

seção 11 (monster como memória da interação; replay e reexecução são
promessas diferentes; o arquivo precisa explicar o que aconteceu, não só
desenhar de novo), seção 12 (bug report deixa de ser mídia). do arquivo de
concepção: ring buffer de `.monster`, evoluir monster de animação para
sessão (eventos, semântica, recursos, ações, origem, resultados), deltas,
buffer limitado (preservar só os recursos necessários para reproduzir o
trecho mantido), e a gravação projetada desde o início para não destruir a
característica de engine leve.

## estado atual no código

- `rust/motion`: codec com IR próprio (`ir.rs`), desacoplado de
  `SceneNode` para o formato congelado não perseguir o enum interno; o
  player baixa IR para `SceneNode` (`lower.rs`), não existe o caminho
  contrário. discover (modo B) descobre deltas a partir de frames
  amostrados (`discover.rs`, `discover_fit.rs`).
- container (`rust/motion/src/encoding/container.rs`): magic `MON0`, versão 0,
  seções `K` (keyframe), `D` (delta), `X`, `T` (descrição), cada uma com
  sha256; contadores `u16` (nós por keyframe, assets, tamanho de dado de
  asset). golden em `rust/motion/tests/fixtures/golden_v0_minimal.monster`.
- limites que importam para sessão: `u16` estoura em sessão longa; texto
  é asset `text_style`; imagem é asset.
- depois de 57 e 58: `Snapshot`, `Revision`, `Snapshot::diff`,
  `Receipt`, `ReceiptSink`.

## revisão do orquestrador (2026-09-23, antes do briefing)

- o gravador mora num crate novo, `rust/record` (depende de `sem`,
  `monster` e `engine`). o `monster` continua sendo o codec de animação e
  ganha só o `raise` (`SceneNode` para o IR), o espelho do `lower` que ele
  já tem, em `rust/motion/src/mappings/raise.rs`.
- esta rodada faz as fases a e b em memória (trilha semântica como
  `ReceiptSink`, anel limitado, raise e trilha visual pelo discover) e
  termina apresentando as duas opções de formato de arquivo com custo e
  impacto, sem implementar a fase c. a decisão do formato é do
  orquestrador, e a fase c vem num segundo briefing.
- quem grava hoje: nenhum app. o `Recorder` precisa servir o dispatcher
  de qualquer app (`Dispatcher::add_sink`) e receber observações e
  snapshots por chamada explícita; os testes usam o app de brinquedo do
  `sem` e o todo do showcase projetado headless.

## decisão de arquitetura (a tomar nesta task, com ADR, antes da fase b)

onde mora o formato de sessão. opções:

- a) perfil de sessão dentro do container monster, com seções novas
  (eventos, snapshots semânticos, receipts) e versão nova do container; o
  golden v0 continua decodificando igual.
- b) formato irmão (`.monster` de sessão com magic própria) que reusa o
  codec de cena do monster para a trilha visual.

recomendação: decidir pela opção que mantém o golden e o player de
animação intocados e deixa a trilha semântica legível sem decodificar a
visual. o agente apresenta as duas com custo e impacto no formato de
parada e reporte antes de implementar a fase c.

## escopo

### fase a: trilha semântica em memória

- `Recorder` implementa `ReceiptSink` (task-58) e recebe também
  observações (`Observed { actor, scope, revision }`) e snapshots.
- evento: `{ t: Millis, kind }` com `kind` em `Observed`, `Snapshot`
  (completo a cada intervalo e em cada keyframe, `Change`s entre eles),
  `Stage` de receipt (o `Request` inteiro no primeiro estágio), `Input`
  humano opcional (tipo, `SemId` sob o cursor quando há ligação).
- campos `secret` nunca gravados (a projeção já não os tem; teste).
- ring buffer limitado por tempo e por bytes, os dois configuráveis; o
  corte respeita keyframe: o trecho mantido sempre começa num snapshot
  completo.

### fase b: trilha visual

- `SceneNode -> monster ir` (o caminho que falta, "raise") para os tipos
  de nó que o engine emite, com teste de roundtrip `raise -> lower`.
- frames amostrados na cadência do render (render on demand: frame só
  quando algo muda) passam pelo discover do monster.
- recursos: só os que o trecho mantido referencia (estilos de texto,
  imagens); imagem entra uma vez por hash de conteúdo.
- custo medido: tempo por frame com gravação ligada, bytes por segundo,
  memória do anel.

### fase c: persistência e trecho

- formato decidido no ADR; cabeçalho com identidade do build (versão do
  crate e sha do git no build) para a task-65 chegar no código daquela
  versão.
- persistir os últimos N segundos por atalho em build de debug e no panic
  hook.
- exportar `[t0, t1]`, incluindo o keyframe anterior necessário das duas
  trilhas.
- leitor estrito: erro tipado, nunca panic em arquivo malformado.

### regra de replay

o recorder e qualquer leitor de sessão nunca criam `Request` e nunca
chamam `execute`. assistir à criação de um commit não cria outro commit.
teste que prova.

## fora de escopo

o inspector com scrub e painel de estado (task-64), sessão para teste
(64), link para o código (65), reexecução.

## dependências

57 e 58 integradas.

## superfície sob responsabilidade

- `rust/motion/**` (formato e raise), ou o crate irmão decidido no ADR
- o módulo de gravação (dentro do crate decidido; não em `sem`, que fica
  puro e sem I/O)
- proposta de ADR [`0037-session-recording-replay-not-reexecution.md`](../../adr/recording/0037-session-recording-replay-not-reexecution.md)

proibido: o golden `fixtures/golden_v0_minimal.monster` e o comportamento
do player de animação; `rust/accord` sem aprovação.

## interfaces que consome

`ReceiptSink`, `Receipt`, `Snapshot`, `Revision`, `Change` (57, 58);
`SceneNode` (engine).

## interfaces que pode alterar

`rust/motion` por adição. qualquer mudança que altere o decode do
golden é recusada.

## contrato exposto

- API de gravação: `Recorder::new(limits)`, `record_*`, `snapshot_window`,
  `persist(path)`, `export(t0, t1, path)`.
- leitor de sessão: `Session::open(path)`, iteração por tempo, acesso O(1)
  ao keyframe mais próximo, estado semântico em t, receipts que tocam um
  `SemId`.
- invariantes: trecho sempre começa em snapshot completo; `secret` nunca
  gravado; nenhum caminho de leitura cria request.

## critérios de aceitação

- sessão de teste com o cenário do commit (observe do agente, unstage do
  humano, rejeição, novo observe, aceite, sha) gravada e relida com todos
  os eventos na ordem e o estado semântico certo em cada t.
- golden de animação decodifica igual (teste existente continua verde).
- roundtrip `raise -> lower` para cada tipo de nó.
- ring buffer respeita os limites (teste com relógio injetado).
- números medidos de custo registrados no ADR.
- `./tool/tasks/fullcheck.sh` verde.

## testes esperados

unitários de cada evento e do anel; roundtrip do formato; arquivo
truncado e sha errado viram erro tipado; o teste da regra de replay.

## riscos e perguntas abertas

- contadores `u16` do container; sessão precisa de contagens maiores.
- texto na trilha visual depende das mesmas fontes embutidas para
  reproduzir igual (determinismo, task-77).
- privacidade: sessão contém caminhos de arquivo, mensagens. gravação é
  opt-in, local, e a persistência nunca sai da máquina sem ação humana.

## commits esperados

1. `task-63: trilha semântica do recorder como ReceiptSink`
2. `task-63: ring buffer limitado por tempo e bytes, corte em keyframe`
3. `task-63: raise de SceneNode para o IR do monster`
4. `task-63: trilha visual pelo discover`
5. `task-63: ADR e formato de sessão`
6. `task-63: persistir por atalho e panic hook; exportar trecho`

## pare e reporte se

- a escolha do formato (a ou b) tiver impacto que o ADR não resolve
  sozinho;
- o custo de gravação medido for alto a ponto de exigir mudar a
  arquitetura do frame.

## Phases

- [phase a and b: semantic and visual track in memory](phase-a-b.md): the `Recorder`, the ring bounded by time and bytes, and the measured cost, in memory.
- [phase c: session file, persistence and excerpt](phase-c.md): the `MONS` format, persistence, export of an excerpt and the strict reader.
- [phase d re-scoped: the ide records the session](phase-d-rescope.md): the decision that removed pixel retention and the showcase shell from this task.
- [phase d: the ide records the session the inspector reads](phase-d.md): the per-frame bindings, the ide's recording, the unfocused window, the cost, the handoff and the closure.
