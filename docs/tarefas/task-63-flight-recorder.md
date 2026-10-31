---
id: task-63
title: "task-63: flight recorder de sessão"
project: plev
type: tarefa
status: concluido
date: 2026-09-24
date_basis: cabecalho
topics: [generative-runtime]
relations: [task-57, task-58]
source: docs/log/steps/task-63-flight-recorder.md
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-63: flight recorder de sessão

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

## fase d re-escopada (2026-09-23, decisão do orquestrador)

a fase d original (retenção de pixels de imagem ligada pelo gravador e a
ligação no shell do showcase, com atalho de debug e panic hook) sai desta
task: o ide não desenha imagem, e o shell do showcase fica para depois.
os dois continuam descritos na fase c abaixo e nas consequências do ADR
`session-recording-replay-not-reexecution`, sem dono agora.

a fase d passa a ser o ide gravando a sessão que o inspector lê (branch
`task/63d-ide-records`, base `main` em `2ecaa8d`):

1. `rec`: cada frame gravado leva onde cada id ligado aparece
   (`Compositor::visible_bindings`), só quando muda em relação ao frame
   anterior, aditivo no formato `MONS` (as sessões antigas continuam
   abrindo), e a `Session` responde onde cada id estava num instante. a
   fase b da task-64 precisa disso para ligar um ponto do frame a um
   controle.
2. `ide`: com a gravação ligada num build de debug, o shell grava no
   relógio do workbench o estado, os receipts, quem observou o quê pela
   porta e os frames, e salva o trecho retido por atalho, no panic e ao
   fechar a janela. sem a variável nada é criado.
3. o laboratório não muda (a task-88 mexe nele).
4. a janela pode subir sem roubar o foco, por variável, só no macOS.

o teste que só o brenner pode fazer (ele usando o ide enquanto um agente
age) precisa deixar uma sessão que o `plev-inspect` lê; é isso que esta
fase entrega.

## fase c fechada (2026-09-23)

integrada na `main` em `c62a05d` (seis commits), rebaseada sobre a 60 e a
58b, gate verde num diretório isolado (1855 testes). arquivo de sessão
`.monster` com magic `MONS`: cabeçalho com identidade de build e seções
`S` (JSON lines), `I`, `C` (os bytes do `monster::encode`, verbatim) e `A`,
sha256 por seção; `Recorder::persist` e `export` (trecho a partir do
keyframe que governa `t0`); `Session::open` estrito (erro tipado em todo
arquivo malformado, 2000 danos aleatórios sem panic; animação recusada
pelo nome); `install_panic_hook`; `monster_player` recusa sessão com
mensagem. medido: a janela de 20,6 s do todo dá um arquivo de 76 KB
(3,7 KB/s), escrito em 1 a 1,5 ms e lido em 0,5 ms.

o agente corrigiu uma premissa do orquestrador: o acervo de imagens não
guarda pixels depois do primeiro frame (o `pixels` de `image.rs:65` é do
upload pendente, drenado pela GPU). decisão do orquestrador: retenção de
pixels opt-in, ligada pelo gravador, compartilhando o mesmo `Arc` com o
upload (sem cópia dupla; o perfil de memória não muda quando não se
grava). isso e a ligação no shell do showcase (atalho de debug e panic
hook) são a fase d, que entra depois da task-61 porque as duas mexem no
shell.

## fases a e b fechadas (2026-09-23)

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
- proposta de ADR [`0037-session-recording-replay-not-reexecution.md`](../decisoes/gravacao/0037-session-recording-replay-not-reexecution.md)

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

## fase d: o ide grava a sessão (branch `task/63d-ide-records`)

sobre a `main` em `2ecaa8d`. seis commits, gate verde.

### as ligações de cada frame, no `rec`

- `Recorder::frame(t, &Compositor)` leva, com o frame, as ligações que
  aparecem (`Compositor::visible_bindings`, a parte de cada retângulo
  dentro do clip) numa camada que o frame desenha, em ordem de
  empilhamento, a de cima por último, como `Compositor::hit` as ordena. a
  assinatura não mudou: o laboratório grava as ligações sem mudar uma
  linha. `frame_scene` (cena dada como nós) não liga nada.
- cada chunk guarda as ligações ao lado dos frames, só quando mudam em
  relação ao frame anterior: abre com a lista inteira (o keyframe dele,
  então um corte do anel num chunk não perde nada) e depois cada passo é
  a lista inteira de novo (um id entrou ou saiu, ou a ordem mudou) ou só
  os retângulos que mudaram (os mesmos ids na mesma ordem: um toast
  subindo, uma rolagem). um frame igual ao anterior no conteúdo, que não
  é guardado, ainda conta para as ligações.
- leitura: `Session::bindings_at(t)` (a lista do frame na tela em `t`,
  `None` antes do primeiro frame, vazia num arquivo sem ligações),
  `binding_at(id, t)` e `hit_at(t, x, y)`. o tipo público é
  `rec::Bound { id, rect }` (`Serialize`).
- no arquivo: seção `B` (tabela de ids uma vez só; por chunk, os passos:
  `at`, tipo, e as entradas por id ou por posição, retângulo em f32). a
  versão diz as seções: versão 0 é S, I, C, A, o layout de antes, e uma
  janela sem ligações continua saindo nela, byte a byte; versão 1 é a 0
  mais B. o leitor continua estrito nas duas: B numa versão 0 é seção
  desconhecida, versão 1 sem B falta seção, e passo fora de ordem, id que
  não faz parse ou aparece duas vezes numa lista, movimento antes da
  lista ou além dela, retângulo não finito ou sem área e passo no início
  do chunk seguinte ou depois viram `FileError::Bindings` e
  `FileError::BoundId`.
- testes (`rust/record/tests/session/bound.rs`): mudou, não mudou, id que
  some, ordem de empilhamento, clip, camada escondida, leitura em qualquer
  instante da memória e do arquivo; o anel e o export cortando no meio
  (cada chunk abre com a lista); um arquivo versão 0 (a mesma sessão sem
  B) abre com ligações vazias; a seção B lida estritamente, com cada erro
  pelo motivo certo, truncamento em todo comprimento e 500 danos
  aleatórios sem panic. no teste da página todo do showcase, um ponto do
  frame relido, numa grade de 10 px, é o controle que `Compositor::hit`
  nomeou ao vivo.

### a janela sem roubar o foco

`PLEV_BACKGROUND=1`, só macOS, em dois commits. o que o briefing pediu
(`with_activate_ignoring_other_apps(false)` e `with_active(false)`) não
bastou: desde o macOS 14 a ativação é cooperativa, e numa rodada com o
Zed na frente (o app cujo terminal lançou o ide) o ide entrou por cima
dele e tomou o foco; com o navegador na frente, não. a correção: a
política de ativação vira `Prohibited` (o AppKit documenta que um app
assim não pode ser ativado; sem ícone no Dock e sem barra de menu) e a
janela fica abaixo de todas (`WindowLevel::AlwaysOnBottom`), então não
cobre nada nem pega clique de outro app. conferido em rodadas reais, uma
delas lançada com o Zed na frente: o foco ficou onde estava, a janela
desenhou os frames nesse nível, o ato de um agente pela porta desenhou e
fez stage, e um Escape postado no pid fechou o ide. é para uma rodada que
alguém assiste, não para uma que alguém usa: tecla só chega postada no
pid. sem a variável, nada muda.

### como o ide grava e salva

- `PLEV_RECORD=1` num build de debug (`ide::record::enabled`); sem a
  variável não se cria recorder, diretório nem tecla, e o shell paga um
  teste de `Option` por frame e por input.
- a lib (`apps/idestudio/src/interfaces/window/record/mod.rs`, `ide::record::Recording`, testável
  sem janela): um `Recorder` no relógio do workbench (`bench.now()`), um
  sink no dispatcher dele (`attach`), o estado (`state`, guardado só
  quando muda), o frame desenhado com as ligações (`frame`, que grava o
  estado antes), o input da pessoa (`input`: tipo, posição e o controle
  sob o ponteiro no frame na tela, nunca o caractere), `save` e
  `catch_panics` (`rec::install_panic_hook`). em `apps/idestudio/src/capability/agent.rs`,
  `Served` ganhou `recorder` e o `Host::observed` grava quem observou o
  quê (ator, escopo, revisão) no relógio do workbench;
  `serve_recorded(port, bench, compositor, recording)` é o `serve` com a
  gravação, e grava o estado logo depois do drain que rodou um ato.
  `serve` continua com a mesma assinatura.
- o shell (`main.rs`, `keys.rs` e o módulo novo `recording.rs`): serve a
  porta com `serve_recorded`; grava o estado depois de cada resultado do
  git, depois de um clique ou tecla da pessoa que mudou algo, e antes de
  cada frame desenhado; grava cada frame e o clique (botão esquerdo e
  direito) e a tecla da pessoa (tecla do keymap como `key`, texto digitado
  no formulário como `text`).
- salva: pela tecla `session::Save`, `cmd-shift-s` ou `ctrl-shift-s`, em
  todo contexto do workspace (o formulário de commit incluído), ligada só
  quando grava (um keymap e um registro a mais, sem mexer em
  `keymap.json` nem em `actions.rs`); ao sair do loop (`exiting`: a
  janela fechada ou o Escape); e no panic. diretório
  `$TMPDIR/plev/sessions/` (ao lado dos sockets da porta, criado 0700),
  arquivos `ide-<pid>-<n>.monster`; o panic hook usa o nome dele,
  `session-<unix ms>-<pid>-<n>.monster`, no mesmo diretório. cada caminho
  vai para o log, e com a gravação ligada as linhas `info` do ide
  aparecem sem `RUST_LOG`.

### o teste sem janela

`apps/idestudio/tests/record.rs` faz o papel do shell pelas mesmas funções
(`Recording::attach`, `state`, `frame`, `input`, `save` e
`ide::agent::serve_recorded`), com um repositório real, o worker do git e
uma porta real. o agente observa com `bounds`, faz stage de A e B, abre o
formulário e escreve; decide; a pessoa clica na linha de B e tira B do
index; o `submit` volta `stale`; o agente observa (com escopo em
`notices` e depois tudo) e submete de novo. o arquivo salvo tem as
observações do agente com escopo e revisão, os sete receipts inteiros, o
clique da pessoa sobre `changes.file[B]`, os frames, e em cada linha o
retângulo que o agente recebeu no primeiro olhar; e o `inspect`, lendo o
arquivo, diz que a recusa se apoiou na decisão do agente e que o
`unstage` da pessoa mexeu em `changes:staged`. sem gravar as observações
(`serve_recorded` com `None`), o teste falha. o arquivo fica em
`$CARGO_TARGET_TMPDIR/ide-record-test.monster`.

o `plev-inspect` desta branch (o crate `inspect` como está na `main`,
sem mudança) sobre esse arquivo:

```text
$ plev-inspect ide-record-test.monster
session   rec 0.1.0, built from c78ab50
span      +0.000s to +0.139s, 40 events, 7 frames
actors    agent:test-agent: 4 observations, 6 requests, 22 stages
          human: 1 request, 4 stages, 1 input
requests  7
  +0.028s  request 1  agent:test-agent  stage on changes.file[A]     completed
  +0.047s  request 2  agent:test-agent  stage on changes.file[B]     completed
  +0.071s  request 3  agent:test-agent  open on commit.form          completed
  +0.072s  request 4  agent:test-agent  set_value on commit.message  completed
  +0.087s  request 5  human             unstage on changes.file[B]   completed
  +0.088s  request 6  agent:test-agent  submit on commit.submit      rejected: stale
  +0.089s  request 7  agent:test-agent  submit on commit.submit      completed: commit = "69f2e6cab5d9bbd96c9cf03e060fe2dbac679175", files = [A]
refusals  1
  request 6, submit on commit.submit by agent:test-agent, rejected at +0.088s: stale
    changes:staged: removed B
    changes:staged moved at +0.087s by human, request 5: unstage on changes.file[B] (executing at +0.087s)

$ plev-inspect ide-record-test.monster --why 6
request 6 (origin 0): submit on commit.submit by agent:test-agent, at +0.088s
  control       commit.submit: button "Commit", enabled
  state         none of its own
  initiated by  agent:test-agent
  action        submit, no parameter
  observed      +0.084s, the whole app, at revision ded79596963b0e1b
                (the last observation by agent:test-agent at the revision the request carries)
  precondition  what the action reads, as agent:test-agent saw it:
                  changes:staged = {A, B}
                  commit.message:text = "two files"
  before        from the observation (+0.084s) to the request (+0.088s):
                  +0.087s changes:staged: removed B
                            by human, request 5: unstage on changes.file[B] (executing at +0.087s)
                  commit.message:text did not move
                  +0.087s input: pointer_down on changes.file[B]
  stages        +0.088s requested
                +0.088s rejected: stale
  verdict       rejected at +0.088s: stale, a state the action reads changed since the observation
  what changed  changes:staged: removed B
  acted         human, request 5: unstage on changes.file[B]
  result        nothing ran: the refusal came before any effect
  note          "by" is a correlation in time, not a declared write: the receipts that ran
                (executing, completed or failed) between the recorded state before a change and
                the change. the contract declares what an action reads, never what it writes.
```

o binário `plev-inspect` da `main` (construído antes desta branch) recusa
o mesmo arquivo pela versão, com o erro tipado e a saída 3:

```text
plev-inspect: .../ide-record-test.monster: unsupported session version 1; this reader reads 0
exit 3
```

é o leitor estrito fazendo o que deve: ele não sabe pular uma seção que
não conhece. o `inspect` sem nenhuma mudança, compilado sobre o `rec`
desta branch, lê.

### prova numa janela de verdade

método: build de debug desta branch em `1ec952b`, macOS, apple m4. um
repositório de rascunho com um commit, `src/login.rs` e `src/session.rs`
modificados e `todo.txt` não rastreado. o ide sobe com
`PLEV_AGENT=1 PLEV_RECORD=1 PLEV_BACKGROUND=1 RUST_LOG=ide=debug,info
cargo run -p idestudio -- <repo>` (o `cargo run` dá `exec` no ide, o pid é o
mesmo). um vigia olha o app da frente a cada 100 ms nos primeiros 4 s e
mata o ide pelo pid se ele virar o app da frente (não virou). o agente é
o `plev` (nome `claude`), uma invocação por passo. as teclas
(`cmd-shift-s` e Escape) são postadas só no pid do ide
(`CGEventPostToPid`); nenhum clique. o harness (swift e zsh) ficou no
rascunho da sessão, fora da árvore.

a cena: salvar logo depois de subir (save 0); dez segundos parado; salvar
(save 1); o agente observa, faz stage de `todo.txt`, abre o formulário e
escreve a mensagem; dez segundos com o formulário aberto e o caret
piscando; salvar (save 2); o agente observa e submete; três segundos;
Escape, que fecha o ide e salva (save 3).

```text
  0.018 front before: front: Zed pid 639
  2.207 ide 83627 (cargo run exec'd it)
  6.855 after 4 s: front: Terminal pid 643 window pid 83627 layer -1 onscreen true ... name 'plev ide - repo' pid 83627 active false name ide
 31.961 ide exited
 31.978 front after: front: Dia pid 645
```

o log do ide:

```text
recording the session: saves go to $TMPDIR/plev/sessions/ide-83627-0.monster (cmd-shift-s or ctrl-shift-s, and on close); a panic writes session-<unix ms>-<pid>-<n>.monster there
agent port: $TMPDIR/plev/ide-83627.sock
session saved (key): $TMPDIR/plev/sessions/ide-83627-0.monster (30064 B, 6 events, 1 chunks)
session saved (key): $TMPDIR/plev/sessions/ide-83627-1.monster (34968 B, 8 events, 1 chunks)
session saved (key): $TMPDIR/plev/sessions/ide-83627-2.monster (60297 B, 23 events, 7 chunks)
committed c751903c8e4b606b9100012dea7c7bc9a2e597ce ["todo.txt"]
session saved (close): $TMPDIR/plev/sessions/ide-83627-3.monster (76313 B, 32 events, 8 chunks)
```

o agente (nós cortados; `bounds` inteiros):

```text
$ plev observe --app ide --scope changes --keys files,staged --bounds
{"v":1,"id":"2","type":"snapshot","revision":"bcc15ad7af227751","nodes":[...4 nós...],"bounds":[{"id":"changes","rect":[72.0,52.0,280.0,748.0]},{"id":"changes.file[src/login.rs]","rect":[84.0,104.0,256.0,44.0]},{"id":"changes.file[src/session.rs]","rect":[84.0,152.0,256.0,44.0]},{"id":"changes.file[todo.txt]","rect":[84.0,200.0,256.0,44.0]}]}
$ plev act changes.file[todo.txt] stage --from-revision bcc15ad7af227751 --app ide
... {"completed":{"result":{}}},"at":14989,"revision":"b9630e814e948f52"}
$ plev act commit.form open --from-revision b9630e814e948f52 --app ide
... {"completed":{"result":{}}},"at":15030,"revision":"04ad2623ba74d17c"}
$ plev act commit.message set_value --param '"add the todo list"' --from-revision 04ad2623ba74d17c --app ide
... {"completed":{"result":{}}},"at":15070,"revision":"7737078a2acdf252"}
$ plev observe --app ide --keys staged,text
{"v":1,"id":"2","type":"snapshot","revision":"7737078a2acdf252",...}
$ plev act commit.submit submit --from-revision 7737078a2acdf252 --app ide
{"v":1,"id":"2","type":"stage","request":"4","stage":"requested","at":25911,"asked":{"actor":{"agent":{"name":"claude"}},"target":"commit.submit","action":"submit","observed":{"revision":"7737078a2acdf252","values":[["changes:staged",{"kind":"set","value":["todo.txt"]}],["commit.message:text",{"kind":"text","value":"add the todo list"}]]}}}
{"v":1,"id":"2","type":"stage","request":"4","stage":"accepted","at":25911}
{"v":1,"id":"2","type":"stage","request":"4","stage":"executing","at":25911}
{"v":1,"id":"2","type":"stage","request":"4","stage":{"completed":{"result":{"commit":{"kind":"text","value":"c751903c8e4b606b9100012dea7c7bc9a2e597ce"},"files":{"kind":"list","value":["todo.txt"]}}}},"at":25966,"revision":"0f74a3cbffdf9de7"}
```

o git depois: `c751903 add the todo list`, com `todo.txt`; os dois `.rs`
seguem modificados.

o `plev-inspect` desta branch sobre o arquivo salvo ao fechar:

```text
$ plev-inspect ide-83627-3.monster
session   rec 0.1.0, built from 1ec952b
span      +0.000s to +28.943s, 32 events, 27 frames
actors    human: 4 inputs
          agent:claude: 3 observations, 4 requests, 16 stages
requests  4
  +14.861s  request 1  agent:claude  stage on changes.file[todo.txt]  completed
  +14.917s  request 2  agent:claude  open on commit.form              completed
  +14.957s  request 3  agent:claude  set_value on commit.message      completed
  +25.798s  request 4  agent:claude  submit on commit.submit          completed: commit = "c751903c8e4b606b9100012dea7c7bc9a2e597ce", files = [todo.txt]
refusals  0

$ plev-inspect ide-83627-3.monster --why 4
request 4 (origin 0): submit on commit.submit by agent:claude, at +25.798s
  control       commit.submit: button "Commit", enabled
  state         none of its own
  initiated by  agent:claude
  action        submit, no parameter
  observed      +25.749s, the whole app, at revision 7737078a2acdf252
                (the last observation by agent:claude at the revision the request carries)
  precondition  what the action reads, as agent:claude saw it:
                  changes:staged = {todo.txt}
                  commit.message:text = "add the todo list"
  before        from the observation (+25.749s) to the request (+25.798s):
                  changes:staged did not move
                  commit.message:text did not move
  stages        +25.798s requested
                +25.798s accepted
                +25.798s executing
                +25.853s completed: commit = "c751903c8e4b606b9100012dea7c7bc9a2e597ce", files = [todo.txt]
  verdict       accepted at +25.798s
  result        completed at +25.853s: commit = "c751903c8e4b606b9100012dea7c7bc9a2e597ce", files = [todo.txt]
```

os quatro inputs da pessoa são as teclas postadas no pid (três
`cmd-shift-s` e o Escape). o `plev-inspect` da `main` recusa esse arquivo
como o do teste (versão 1, saída 3). as ligações gravadas no instante da
primeira observação do agente são, em cada id que ele recebeu, o mesmo
retângulo dos `bounds` dele; o arquivo tem também as do resto da tela
(`stacks`, o commit da pilha, `diff`).

### custo

medido na mesma cena rodada duas vezes, com e sem `PLEV_RECORD`, build de
debug: `Instant` em volta do `render_frame` (montar a cena e apresentar) e
em volta do `Recording::frame` a cada `RedrawRequested`, escritos por
frame no log (`drawn: X ms, recorded: Y ms`).

| medida | com gravação | sem gravação |
| --- | --- | --- |
| desenhar, média (sem os 2 frames da subida) | 2,10 ms | 2,37 ms |
| gravar, média por frame | 1,11 ms | |
| gravar, mediana | 0,50 ms | |
| gravar, o frame que fecha um chunk | 2,0 a 8,2 ms | |

gravar (projetar e comparar o estado, subir o frame para o IR, as
ligações) fica abaixo de 2 ms por frame na média e na mediana. o frame
que fecha um chunk (discover e encode do monster, um a cada 2 s enquanto
há frames) passa disso em debug: 8,2, 5,1, 4,7, 2,0 e 2,3 ms nesta
rodada. é o pico que o ADR já nomeava; tirá-lo do caminho do frame não
coube aqui (ver o handoff).

bytes: o arquivo do fechamento tem 76 KB em 28,9 s (2,6 KB/s), 2,3 KB
deles na seção B (3 %). parado, o ide não grava nada: nos dez segundos
entre o save 0 e o save 1, nenhum frame e nenhum evento; o arquivo cresceu
4,9 KB só pela própria tecla de salvar (o input e o keyframe semântico
que ele reabre). com o formulário aberto e o caret piscando, ninguém
agindo, cada chunk de 2 s tem 4 frames em 4,5 KB (2,2 KB/s), mais a lista
inteira de ligações que abre o chunk (cerca de 0,2 KB). as duas situações
ficam bem abaixo de 20 KB/s.

ligações antes e depois: no laboratório (`ide-lab --arm scripted --human
unstage-after-observe:src/session.rs`, 0,2 s, 9 frames, 3 chunks, 14
ids), a seção B é 1,2 KB de 68,5 KB (67,2 KB sem ela, 1,8 %); na medida da
página todo do showcase (em debug), 3,4 KB de 79 KB (75,5 KB sem ela, 160
B/s em 21,1 s, 4,5 %), lista inteira em 34 de 368 frames, nenhum passo de
movimento, 334 frames sem mudança. o laboratório é curto demais para um
número por segundo que signifique algo; o antes e depois está no tamanho.

### as sessões antigas

as nove sessões dos checkpoints (`target/agents/exp/{a1,a2}`,
`exp2/{a1p,a2pa,a2pb}`, `lab-{control,control2,plev,scripted}`) abrem com
o `plev-inspect` da `main` e com o desta branch, com saída 0, e as saídas
do resumo (texto e json) e da timeline (json) são idênticas byte a byte
entre os dois; o sha256 de cada arquivo ficou igual.

### incidente

na primeira rodada de fumaça da janela, antes da correção do foco, o ide
subiu com o Zed na frente e tomou o foco por uns 6 s (até o Escape postado
no pid). a própria gravação mostrou o efeito: dois `pointer_down` da
pessoa nas linhas `src/login.rs` e `src/session.rs`, cada um um
`show_diff` (nada mudou no repositório, que era de rascunho). a correção
é o commit `390ee70`; depois dela, seis rodadas, duas lançadas com o Zed
na frente, e o foco não saiu de onde estava. numa delas o harness errou
(o `cargo run` dá `exec` no ide e o script procurava um processo filho):
o vigia não tinha o pid e o ide ficou aberto uns 15 s, abaixo das outras
janelas e sem foco, até eu fechá-lo com um Escape postado no pid. os
arquivos daquelas rodadas foram para o lixo.

### handoff

```text
Goal: task-63 fase d (re-escopada): o ide que alguém usa grava a sessão
      (opt-in, debug), com quem observou o quê, o estado, os receipts, os
      frames e onde cada id aparece em cada frame, e a salva por atalho,
      no panic e ao fechar, para o plev-inspect responder "por que"
Status: pronto para validação do orquestrador

Implementado:
- rec: as ligações de cada frame (Compositor::visible_bindings, em ordem
  de empilhamento, nas camadas que o frame desenha), guardadas por chunk
  só quando mudam (lista inteira ou retângulos que mudaram), lidas por
  Session::bindings_at, binding_at, hit_at; seção B no arquivo, versão 1;
  janela sem ligações sai em versão 0, byte a byte; leitor estrito
- ide: PLEV_BACKGROUND=1 (macOS): política de ativação Prohibited, janela
  com with_active(false) e no nível AlwaysOnBottom
- ide lib: ide::record (Recording: attach, state, frame, input, save,
  catch_panics; enabled, default_dir, bind_keys, SAVE_KEYS, Save);
  ide::agent::Served ganhou recorder e observed; serve_recorded
- ide shell: PLEV_RECORD=1 em debug liga a gravação; salva por
  cmd-shift-s ou ctrl-shift-s, ao sair do loop e no panic, em
  $TMPDIR/plev/sessions/ide-<pid>-<n>.monster

Arquivos principais:
- rust/record/src/{bound.rs (novo), format.rs, persist.rs, visual.rs,
  screen.rs, session.rs, recorder.rs, ring.rs, limits.rs, error.rs, lib.rs}
- rust/record/tests/session/{bound.rs (novo), file.rs, measure.rs, main.rs}
- apps/idestudio/src/{record.rs (novo, lib), recording.rs (novo, shell),
  agent.rs, lib.rs, main.rs, keys.rs}, apps/idestudio/tests/record.rs (novo),
  apps/idestudio/Cargo.toml (inspect como dev-dependency), Cargo.lock
- docs/adr/recording/0037-session-recording-replay-not-reexecution.md,
  docs/log/steps/task-63-flight-recorder.md

Interfaces adicionadas/alteradas:
- adicionadas: rec::Bound, rec::VERSION_BINDINGS, Session::{bindings_at,
  binding_at, hit_at}, Stats::{bindings_whole, bindings_moved},
  Persisted::bindings_bytes, FileError::{Bindings, BoundId};
  ide::record::*, ide::agent::serve_recorded, Served::recorder e
  Served::observed
- alteradas sem mudar assinatura: Recorder::frame grava também as
  ligações; o texto de FileError::UnsupportedVersion diz "0 and 1"; um
  arquivo com ligações é versão 1 (um leitor anterior o recusa pela
  versão)
- Served é uma struct de campos públicos: quem a monta à mão (ninguém
  fora de agent.rs hoje) passa recorder

Testes:
- rec: 4 novos (bound.rs) e o teste da página todo do showcase conferindo
  hit_at contra Compositor::hit numa grade; file.rs: a versão 1 agora
  existe, então o teste de versão usa 2 e confere que 1 sem B falta B
- ide: 1 unitário (as teclas de salvar em todo contexto) e
  tests/record.rs (o shell sem janela gravando pelo mesmo caminho; falha
  sem gravar as observações, conferido à mão)

Validação executada:
- ./tool/tasks/fullcheck.sh verde com a perna urnaui: 2208 passed, 0 failed, 12
  ignored (88 "test result"); a main tinha 2202 (87); os 6 a mais são
  desta task
- cargo test -p record, -p idestudio, -p replay, -p uigallery verdes dentro do gate
- as nove sessões antigas: saídas idênticas entre o plev-inspect da main
  e o desta branch, arquivos intactos
- prova na janela real com saída colada acima; o plev-inspect desta
  branch lê a sessão do teste e a da janela, o da main recusa as duas
  pela versão (saída 3)

Decisões:
- as ligações vão numa seção nova e a versão sobe para 1 só quando há
  ligações: um leitor estrito não pode pular uma seção que não conhece,
  então nenhum desenho aditivo deixaria o binário da main ler um arquivo
  com ligações (JSON a mais na trilha S seria ignorado por ele, mas só
  pendurado num evento que não é um frame, e o inspect casa Kind de forma
  exaustiva, então um Kind novo quebraria o crate que não posso tocar).
  a versão 1 dá ao leitor antigo um erro claro ("unsupported session
  version 1") em vez de "unknown section"
- ordem de empilhamento e só camadas desenhadas: o que a janela mostrava
  e o que hit respondia; visible_bindings sozinho não diz qual está em
  cima entre camadas
- passos de movimento além da lista inteira: um toast subindo ou uma
  rolagem mudam só retângulos, e a lista inteira a cada frame desses
  custaria a lista inteira por frame
- PLEV_RECORD (como PLEV_AGENT) e PLEV_BACKGROUND como nomes; a tecla
  cmd-shift-s (ctrl-shift-s fora do mac), num keymap e num registro
  acrescentados só quando grava, sem editar keymap.json nem actions.rs
  (fora da lista de arquivos)
- o estado gravado também logo depois de um ato do agente, de um
  resultado do git e de um clique ou tecla que mudou algo, não só no
  frame: a correlação do inspect ("quem mexeu") olha a janela entre o
  estado anterior e a mudança, e um estado gravado um frame depois junta
  atos na mesma janela
- a janela em segundo plano ficou além do que o briefing pediu
  (Prohibited e AlwaysOnBottom): o pedido não segurava o foco com o Zed
  na frente (incidente acima)

Limitações:
- o frame que fecha um chunk custa 2 a 8 ms em debug (discover e encode
  no caminho do frame, antes desta task); a média por frame fica em 1,1 ms
- o tamanho da janela e o fator de escala não são gravados: a fase b da
  64 precisa deles para desenhar o frame no tamanho certo (hoje só dá
  para inferir pelos retângulos)
- a janela com PLEV_BACKGROUND não pode ser usada por uma pessoa (não
  ativa); para usar o ide, sem a variável
- o arquivo com ligações não abre em nada construído antes desta branch
  (inspect, plev-inspect, monster_player continua recusando sessão por
  magic, como antes)
- o salvamento ao fechar depende de o loop sair (janela fechada, Escape);
  um SIGTERM ou SIGKILL não salva (só o panic hook e o atalho cobrem o
  resto)
- o laboratório grava as ligações também (mesmo Recorder::frame), então
  as sessões novas dele são versão 1

Dependências pendentes:
- nenhuma para integrar

Riscos de integração:
- task-89 mexe no wire agora: Served implementa wire::Host; se o trait
  mudar (observed ou bounds), o merge é mecânico em agent.rs
- task-90 mexe em human_agent.rs e no sem; não toquei em nenhum dos dois
- task-88 (laboratório) vai receber sessões versão 1 do ide-lab e pode
  delegar o observed do LabHost a Served com recorder
- quem compara sessões por bytes: uma sessão com ligações não é mais
  igual à de antes (ganha B); sem ligações é igual

ADR proposto:
- já escrito em session-recording-replay-not-reexecution.md (a trilha
  visual, o formato com a versão e a seção B, o custo, as consequências).
  a linha do índice (docs/adr/README.md, fora da minha lista) pode ganhar
  no fim: "; desde a fase d, cada frame leva onde cada id ligado aparece
  (seção B, versão 1; versão 0 continua lida), e o ide grava a própria
  sessão com PLEV_RECORD=1 em debug"

Para a fase b da task-64 (desenhar as ligações):
- ler com Session::bindings_at(t): a lista do frame na tela em t, em
  ordem de empilhamento (a de cima por último), retângulos (x, y, w, h)
  em pixels lógicos da janela, já cortados pelo clip; hit_at(t, x, y) dá
  o mesmo que Compositor::hit deu ao vivo (conferido numa grade da página
  todo); binding_at(id, t) para destacar um controle escolhido
- as ligações mudam nos instantes dos frames, inclusive de frames cujo
  conteúdo era igual ao anterior e não foi guardado: frame_times não
  lista todos os instantes em que a lista mudou; para a mudança exata,
  consultar bindings_at nos instantes que interessam (eventos, frames)
- ids que aparecem no frame e não no escopo de uma observação existem
  (o arquivo tem a tela inteira); os ids batem com os do estado
  (state_at) e dos receipts
- um arquivo versão 0 (tudo que existia antes) volta com lista vazia:
  a interface precisa dizer "sem ligações gravadas" e não "nada na tela"
- falta gravar o tamanho da janela e o fator de escala (ver limitações);
  sem isso a interface precisa assumir o retângulo que cobre as ligações
  e a cena
- o inspect não precisou mudar para ler; para mostrar as ligações na
  timeline ou no --json, é a fase b (Bound é Serialize)

Commits:
- dbc0565 task-63d: phase d re-scoped to the ide recording its session
- 9f27f39 task-63d: rec records where each bound control shows in each frame
- c78ab50 task-63d: the ide's window can come up without taking the focus
- 390ee70 task-63d: the background window never activates and covers nothing
- 1ec952b task-63d: the ide records its session for the inspector
- o commit deste texto: a prova na janela e o handoff
```

## fechamento da fase d (orquestrador, 2026-09-24)

integrada na `main` em `8e6e814` (seis commits, rebaseados sobre
`6dd1f95` sem conflito).

validação do orquestrador:
- diff lido commit a commit: `rec::bound`, a seção B com versão 1 só
  quando há ligações, o `Recording` do ide na lib, `serve_recorded` e o
  `Host::observed` do `Served`, os ganchos do shell, a janela em segundo
  plano só sob `PLEV_BACKGROUND=1` e só no macOS;
- gate verde no diretório da task: 2231 passaram, 0 falharam, 12
  ignorados, com a perna urnaui;
- o `plev-inspect` desta branch lê a sessão da janela real (o agente
  `claude` com três observações e quatro requests, o commit com
  `todo.txt`) e as três sessões da segunda rodada do checkpoint a.

decisões do agente aceitas:
- o `plev-inspect` antigo recusa a versão 1 com erro claro. um leitor
  estrito não pula seção desconhecida, e o leitor novo lê as duas
  versões;
- a política de ativação `Prohibited` com a janela sempre por baixo em
  `PLEV_BACKGROUND=1`, porque só não ativar não segurou o foco com o Zed
  na frente.

severidade do que ficou:
- P3: o frame que fecha um chunk custa de 2 a 8 ms em debug (o discover
  do monster, anterior a esta fase);
- P2 para a fase b da 64: o tamanho da janela e a escala não são
  gravados.

incidente: na primeira rodada, antes da correção, o ide tomou o foco
por uns 6 s e gravou dois cliques do brenner num repositório de
rascunho. nada mudou fora dele. retenção de pixels de imagem e o shell do
showcase continuam fora, sem task (P2 quando a 64 fase b precisar de
imagem).
