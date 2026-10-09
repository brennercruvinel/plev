---
type: truth
base: main @ c5fdf36
date: 2026-09-23
audience: [orchestrator, ai-agents]
tags: [truth, audit, sem, wire, rec, ide, git, engine, comps, showcase]
---

# o que o código da main faz agora

## 0. mudanças desde a base

integrações que mudaram algo descrito abaixo; o texto das seções continua
o da base até ser reescrito.

- `c07a7f0`: `ShapedGlyph`, `shaped_glyphs` e o import de `CacheKey` em
  `rust/engine/src/drawing/text/system.rs` passam a ser só nativos, como o
  export svg; `tool/tasks/fullcheck.sh` roda as pernas wasm (showcase e urnaui) com
  `-D warnings`, como o CI.
- `4896451` (task-84): a corrida da seção 4a está corrigida. a sessão
  conta as leituras pedidas (`Reads`, `apps/idestudio/src/repository/git.rs`) e
  descarta uma leitura feita antes de uma mutação que a view já mostra;
  testes em `apps/idestudio/tests/human_agent.rs` (o primeiro item da seção
  6.6 deixa de valer); o roteirizado do laboratório não espera mais o
  index parar. `GitSession::refresh`, `send` e `Workbench::refresh` pedem
  `&mut self`.

- `2d4aa4b` (docs seguem o código): os itens 1 a 25, 31 e 32 da seção 5
  estão corrigidos nos docs (README, arc.toml, manual do engine, nove
  ADRs, arquivo da task-62); os itens 26 a 30 foram corrigidos no
  programa pelo orquestrador. o `ide` ganhou `default-run = "ide"`: com
  o `ide-lab`, `cargo run -p idestudio` não sabia qual binário rodar.

- `78f463c` (task-83): `Change::NodeAdded(Box<Node>)` (seção 2.1 e 4b:
  o delta traz o nó; o recorder continua gravando snapshot inteiro para
  nó adicionado, pela posição entre irmãos); `Rejection::Stale { changes,
  acted }` e `Acted` (seção 2.2 e 4d: o dispatcher preenche, em
  `Dispatcher::push`); o wire serve o nó de um delta pela mesma função
  do snapshot (`serve`, seção 2.3); a regra de stale está no `plev act
  --help` e no doc do `Act` (seção 4e); o leitor de sessão converte o
  `node_added` antigo (`rust/record/src/persist.rs`, `read_event`); o toast
  do ide diz quem agiu. `tool/tasks/fullcheck.sh`: a perna urnaui usa
  `$CARGO_TARGET_DIR/urnaui` quando o diretório é compartilhado
  (`1bf6606`).

- `476bbb4` (task-85): `git::Commit.files` e `NewCommit.files` com uma
  definição (caminhos contra o primeiro pai, raiz contra a árvore vazia,
  renomeação nos dois caminhos), por `GitRepo::files_of` (gix, cache por
  sha); `git show --name-only` saiu (seção 2.7 e 4c). o workbench mostra
  o commit no passo que conclui o receipt (`GitSession::show_commit`,
  chamado de `truth.rs` antes do `complete`). `stacks.commit[<sha>]` tem
  `files` nos `FILES_PROJECTED` (10) mais recentes e no último commit que
  o ide fez. testemunha do laboratório e dos testes: `git diff-tree
  --no-renames`.

- `365d8fd` (task-87): `StageReply.revision` num estágio terminal, a
  revisão em que o app ficou, servida no anel (seção 2.3); `Server::forward`
  recebe o `Host`; `plev --help` diz que `--keys` corta só estado. o
  anel continua com 16 revisões, e cada ato concluído ocupa uma.

- `a6fec8d` (task-82 fase b): `Wake` em toda a cadeia do ide, na página
  Git e no urnaui; os três shells em `WaitUntil` e com tick de alcance em
  `new_events` (seção 4g: o ide não desenha mais em frame contínuo; a
  página Git não eleva mais o `bool`; o worker do git não desenha mais
  com outra página na tela); `TextField::tick` e `ToastManager::tick`
  saíram; `GitRepo` roda todo `git` com `GIT_OPTIONAL_LOCKS=0` e o watcher
  ignora `.git/index.lock`. fica da 4g: o `EditorView` sem foco, o caret
  do dock, a ponte de acessibilidade.

- `49e2822` (task-67): `tool/bench/regress/src` (check e bless)
  com baselines em `tool/bench/regress/baselines/` (157, imagem só no
  Todo em core e hoff); `StagedPage::compositor`, `steps`, `settled` em
  `apps/uigallery/src/shell/window/headless.rs`; job `regress` no CI (seção 3,
  linha 15: a regressão estrutural existe). `rust/engine/tests/
  file_hygiene_guard.rs` (`7528309`) conta o limite de 639 linhas de
  produção por arquivo no gate.

- `e6560a2` (task-64 fase a): crate `rust/replay` (lib e bin
  `plev-inspect`): `Inspector` sobre uma `Session`, `timeline`, `why`
  (a cadeia causal de um request), `control`, `summary`; só lê. mostra
  quando uma `read` estava fora do escopo da observação em que o request
  se apoiou (seção 2.3: o `from_revision` monta o observado do snapshot
  inteiro; task-89 em andamento).

- `7f866e8` (task-86): o binário `ide` abre a porta (seção 2.7 e 3,
  linha 21): `apps/idestudio/src/capability/agent.rs` (`Served`, `serve`, `port`, só
  unix), o shell serve no `AppEvent::Agent` e depois de cada
  `AppEvent::Git`; `main.rs` dividido em `pace.rs` e `keys.rs`;
  `Binding::visible_rect` e `Compositor::visible_bindings` no engine
  (seção 2.5), lidos pelo `bounds` do showcase e do ide. o `acted` da
  seção 2.2 nomeia só o que foi validado na revisão exata do autor; na
  janela isso esconde o unstage atrás de uma seleção (task-90).

- `79b8c93` (task-89): o anel continua guardando o snapshot inteiro, mas
  um ato com `from_revision` não lê mais nele os valores observados. o
  `Server` guarda o que mostrou a cada ator (`rust/socket/src/shown.rs`,
  por nome de `hello`, 16 revisões por ator, 16 atores), e cada `read`
  vem da última observação daquele ator que o mostrou (seção 2.3). um
  `read` nunca mostrado dá `ProtoError::NotObserved`. a revisão de um
  estágio terminal não mostra estado. o `plev act --help` diz a regra; a
  seção 4f continua valendo (uma conexão por comando), e agora o nome é
  a identidade que liga os comandos. o `Observation::revision` de um ato
  continua `r`, e o `acted` ancora nele (task-91).

- `d02b4d1` (task-90): `Dispatcher::acted` (seção 2.2) parte do menor id
  que rodou validado na revisão do autor, de qualquer ator, e nomeia os
  outros atores que rodaram dali em diante; doc em `rust/accord/src/
  check.rs` (`Acted`). o toast do ide agrupa por ator
  (`apps/idestudio/src/capability/notice.rs`). no ide, a seleção, a digitação,
  o foco, o toast saindo e as linhas do diff mudam a revisão sem receipt
  e não ancoram (task-91).

- `8e6e814` (task-63 fase d): `rec` grava por frame as ligações visíveis
  (`rust/record/src/bound.rs`, seção B, versão 1 do arquivo só quando há
  ligações; `Session::bindings_at`, `binding_at`, `hit_at`) (seção 2.4:
  "ligações não gravadas" deixa de valer). o `ide` grava a própria
  sessão com `PLEV_RECORD=1` em debug (`apps/idestudio/src/interfaces/window/record/mod.rs`,
  `recording.rs`; `agent::serve_recorded`; `Served::observed`), salva em
  `$TMPDIR/plev/sessions/`. `PLEV_BACKGROUND=1` no macOS: política de
  ativação proibida e janela sempre por baixo. o showcase continua sem
  gravar; imagens continuam sem pixels.

- `96c20a0` (task-88): o `ide-lab` mede a checagem de stale pelo valor:
  válida quando um `submit` depois do request da pessoa carrega o
  `changes:staged` de antes dela e o app tem outro no dispatch
  (`apps/agenteval/src/story.rs`, `stale_measurement`). a pessoa tem
  dois gatilhos (`unstage-after-observe` em `Host::observed`,
  `unstage-before-submit` em `LabHost::dispatch`). o roteirizado tem
  `--scripted observe|chain`. `refused.rs` conta as recusas do protocolo;
  `finish.rs` fecha a corrida. `lab.rs` saiu da lista congelada.

- `042d34e` (task-65 fases a e b): `engine::hot_reload::view_source`
  (`origin_at`, `text_at`, `open`, `editor_command`) sob o cfg
  `hot_reload`; o baseline de literais tem linha e coluna
  (`proc-macro2/span-locations` só pela feature `hot-reload`) e ignora
  literais de teste e doc; o showcase tem o atalho em
  `apps/uigallery/src/shell/window/source.rs` (gate por `debug_assertions` e
  desktop), `Access` em `app/access.rs` e a janela em segundo plano em
  `app/background.rs`. `Compositor::hit` tem o primeiro uso de produção.

- `29373d4` (task-91): `sem::Mark`, `Dispatcher::mark`,
  `ReceiptSink::attached` e `looked` (padrões vazios); `Dispatcher::acted`
  com marca nomeia todo request tomado depois do olhar (o autor
  incluído), sem marca a regra da 90 (seção 2.2). o `wire` guarda o olhar
  de cada ato do port (`rust/socket/src/shown.rs`, `Looks`) e adiciona o
  sink na primeira observação. `Traffic.acts_from_revision` e
  `acts_observed` (seção 2.3). o toast do ide nomeia o próprio agente na
  terceira pessoa. o `rec` ainda não grava o olhar.

- `871f0dd` (task-68): crate `rust/plevir` (depende só de `sem` e serde):
  `Ir`, `IrNode`, `Component` (vocabulário v1, `vocab.rs`), `Props` e
  `Layout` por token (`token.rs`), `Capabilities::from_snapshot`
  (`caps.rs`), `decode`/`parse` (leitor único com JSON pointer),
  `verify`/`VerifiedIr` (`verify/`), `GenerationPolicy`, `schema()` (o
  arquivo em `rust/plevir/assets/plev-ir-1.json`). fixtures em
  `rust/plevir/tests/fixtures/`, regeneradas pelos testes `uir_fixture` do ide e
  do showcase. nada materializa ainda (task-69).

- `d611fa4` (task-64 fase c): `apps/idestudio/tests/rerun/` (harness de
  reexecução controlada, trait `Fixture`, formato `.marks`), a fixture
  `apps/idestudio/tests/fixtures/sessions/a2ppa.{monster,marks}`;
  `apps/idestudio/src/repository/eval_repo.rs` (o repositório do laboratório, antes
  `src/bin/lab/repo.rs`, agora público na lib); `plev-inspect --export`.

- `22d59fb` (task-69 fase 1): crate `rust/screen` (`Surface::materialize`
  só com `VerifiedIr`; `expand`, `layout`, `token`, `widget`, `draw`,
  `project`, `host`); a página Surface do showcase
  (`apps/uigallery/src/views/apps/surface/mod.rs`, `Pane::Surface`); o
  módulo `agent` do showcase em `apps/uigallery/src/views/dispatcher/agent.rs`
  (o `dispatch.rs` com 311 linhas, fora do `FROZEN`).
  `official-app-pattern` tem o segundo padrão.

- `f101359` (task-92): `rust/socket/src/sink.rs` (o sink do porto, com
  `Waits`) e o wake do porto quando o sink ouve um estágio de um request
  que ele espera (seção 2.3); `sem::Request.ask` opcional, `asks_a_person`,
  o `check` pede confirmação quando um agente pergunta (seção 2.2);
  `Act.ask` e `plev act --ask` (limite 280, `plev` espera 300 s); o modal
  do ide com a pergunta; `redact` cobre a pergunta; o `why` mostra.

- `654d740` (task-93): o `ide-lab` tem `--answer allow|decline|none` e
  `--answer-after <ms>` (a pessoa responde no modal pelo clique, como
  `Actor::Human`; `apps/agenteval/src/answer.rs`), `--scripted ask`, e o
  relatório conta perguntas e respostas (`asked.rs`: `questions_asked`,
  `questions`, `over_the_person`, `run.no_commit_because`).

- `4593a4c` (task-69 fase 2): `SurfaceHost::transform` e
  `set_reduced_motion`, geometria por `SemId`, `rust/screen/src/transition.rs`
  (a mola do overlay e um `Tween`); fixtures `mixer-left`, `mixer-right` e
  `showcase-todo-columns` em `rust/screen/tests/fixtures/`; a página
  Surface alterna duas formas.

## 1. como ler

- base: `main @ c5fdf36`. todo `arquivo:linha` foi conferido nesse
  commit, lendo o código. nada foi compilado nem rodado.
- regra do projeto: a documentação nunca tem autoridade maior que o
  código. onde os dois divergem, este arquivo descreve o código, e a
  seção 5 registra a divergência.
- caminhos relativos à raiz do repositório.
- "consome de fato" quer dizer: aparece fora de teste, de doc comment e de
  exemplo, segundo `rg` na base (import `use sem::{..}` em várias linhas
  incluído).
- o arquivo descreve o que é. não traz plano, prioridade nem task nova.
  o que não foi possível confirmar está na seção 8, com o motivo.

## 2. contratos que existem

### 2.1 `sem`: identidade, significado e forma serde

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `SemId` | `rust/accord/src/identity.rs:28-29` | o app, por texto ou por `name`, `child`, `key` (`rust/accord/src/identity.rs:33-74`) | comps, engine, ide, rec, showcase, wire | texto canônico validado (`rust/accord/src/identity.rs:118-161`); `Eq`, `Ord` e `Hash` seguem o texto; clone é refcount (`Arc<str>`) |
| `Role` | `rust/accord/src/taxonomy.rs:9-43` | projeção do app | comps, engine (mapa AccessKit), ide, rec, showcase | enum fechado de 33 (`rust/accord/src/taxonomy.rs:47`); role interativa exige label (`rust/accord/src/taxonomy.rs:87-103`, `rust/accord/src/validate.rs:55-57`) |
| `Value`, `ValueKind` | `rust/accord/src/property.rs:24-31`, `:109-116` | projeção | engine, ide, rec, showcase, wire | igualdade exata; `Num` por bits (`rust/accord/src/property.rs:47-59`) |
| `StateRef` | `rust/accord/src/property.rs:138-161` | projeção (`reads`) | ide, rec, showcase | forma `<semid>:<key>`, separada no último `:` (`rust/accord/src/property.rs:149-160`) |
| `Constraints`, `Actors`, `Confirm`, `Policy`, `ParamSpec`, `ActionDecl` | `rust/accord/src/manifest.rs:23-53`, `:104-120`, `:128-133`, `:150-159`, `:166-169`, `:177-201` | projeção | comps, ide, rec, showcase; engine lê `ActionDecl` na accessibility | `faults` recusa range, step e options inválidos (`rust/accord/src/manifest.rs:57-79`); padrão de `Actors` é todo mundo (`rust/accord/src/manifest.rs:111-119`); `editable` é falso por padrão (`rust/accord/src/manifest.rs:24-25`) |
| `Node`, `Source` | `rust/accord/src/controls.rs:35-75`, `:101-105` | `SnapshotBuilder::node` (`rust/accord/src/assembly.rs:56-93`) | engine, ide, rec, showcase, wire | igualdade ignora `source` (`rust/accord/src/controls.rs:77-91`); `source` vem de `#[track_caller]` e aponta o site de projeção, não o render (`rust/accord/src/assembly.rs:56`, `:79-83`) |
| `Semantics`, `SnapshotBuilder`, `NodeBuilder`, `ActionBuilder` | `rust/accord/src/assembly.rs:26-33`, `:38-42`, `:116-119`, `:227-230` | o app implementa `Semantics` | implementado fora de teste só por `TodoPage` (`apps/uigallery/src/views/apps/todo/semantics/mod.rs:146`) e `Workbench` (`apps/idestudio/src/capability/project.rs:39`); `NodeBuilder` devolvido pelos 19 helpers do comps | texto inválido vira erro agregado, nunca panic (`rust/accord/src/assembly.rs:5-8`); `finish` devolve todas as violações (`rust/accord/src/validate.rs:16-44`) |
| `Snapshot` | `rust/accord/src/snapshot.rs:28-35` | `finish` (`rust/accord/src/validate.rs:41-43`) | engine, ide, rec, showcase, wire | valor imutável; `inert` herdado (`rust/accord/src/validate.rs:141-171`); valor `secret` vira tamanho (`rust/accord/src/validate.rs:176-188`); `subtree` para escopo (`rust/accord/src/snapshot.rs:135-159`) |
| `Revision` | `rust/accord/src/revision.rs:57-58` | `Snapshot::revision` (`rust/accord/src/snapshot.rs:100-104`) | ide, rec, showcase, wire | FNV-1a 64 escrito à mão sobre a codificação `sem.revision.v1` (`rust/accord/src/revision.rs:10-33`, `:51`); ignora `source`, ordem das chaves de estado e intercalação de subárvores; trafega como 16 hex minúsculos (`rust/accord/src/revision.rs:66-91`) |
| `Change` (12 variantes) | `rust/accord/src/mutation.rs:22-83` | `Snapshot::diff` (`rust/accord/src/mutation.rs:91-125`) e o delta stale de `check` (`rust/accord/src/mutation.rs:199-216`) | ide, rec, showcase, wire | vazio exatamente quando os snapshots são iguais (`rust/accord/src/mutation.rs:16-18`); `NodeAdded` carrega só o id (`rust/accord/src/mutation.rs:23`) |
| `SemError`, `SemErrors` | `rust/accord/src/failures.rs:18-77`, `:92` | parse, `finish`, dispatcher | engine (accessibility), ide, rec, showcase | 14 variantes; erro de uso da API, nunca decisão de negócio (`rust/accord/src/failures.rs:1-4`) |
| forma serde | atrás da feature `serde`, padrão ligado (`rust/accord/Cargo.toml:12-16`); `SnapshotWire` privado (`rust/accord/src/snapshot.rs:192-219`) | serde | wire, rec, showcase (`--semantics`) | campo no valor padrão não é escrito e lê de volta como padrão (`rust/accord/src/manifest.rs:5-9`, `rust/accord/src/controls.rs:27-32`); forma fixada em `rust/accord/tests/encoding.rs` |

### 2.2 `sem`: ação, receipt, dispatcher e redação

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `Actor` | `rust/accord/src/requests.rs:23-28` | quem pede | engine, ide, rec, showcase, wire | quatro tipos; nenhum representa replay (`rust/accord/src/requests.rs:18-19`) |
| `Observation`, `Request`, `Request::observe` | `rust/accord/src/requests.rs:52-55`, `:62-72`, `:90-131` | ator, a partir do snapshot que viu | engine (leitor de tela), ide, showcase, wire; `Observation` direto só no wire | guarda o valor de cada `read` da ação; a revisão é só rastreio (`rust/accord/src/requests.rs:42-49`) |
| `Verdict`, `Rejection` (9 variantes), `check` | `rust/accord/src/decision.rs:17-26`, `:34-62`, `:99-138` | `check`, chamado só pelo dispatcher (`rust/accord/src/dispatch.rs:330`) | `Rejection`: ide, rec, showcase, wire. `Verdict` e `check`: nenhum consumidor fora do `sem` | pura; ordem fixa (`rust/accord/src/decision.rs:67-80`); stale compara só as `reads` da ação (`rust/accord/src/decision.rs:166-181`); `Stale` não diz qual ator mudou o estado (`rust/accord/src/decision.rs:46`) |
| `RequestId`, `Millis` | `rust/accord/src/receipts.rs:29`, `:20` | dispatcher numera a partir de 1 | ide, rec, showcase, wire | `sem` nunca lê relógio; o tempo é de quem chama (`rust/accord/src/receipts.rs:16-20`) |
| `Stage`, `Outcome`, `Receipt` | `rust/accord/src/receipts.rs:87-96`, `:71-73`, `:147-152` | dispatcher | ide, rec, showcase, wire | transições legais em `can_advance_to` (`rust/accord/src/receipts.rs:122-134`) |
| `Operable`, `Execution`, `ReceiptSink` | `rust/accord/src/dispatch.rs:20-28`, `:34-45`, `:50-52` | app (`Operable`), consumidor (`ReceiptSink`) | `Operable`: `TodoPage` (`apps/uigallery/src/views/apps/todo/semantics/mod.rs:188`) e `Workbench` (`apps/idestudio/src/capability/execute.rs:157`); sinks: `wire` (`rust/socket/src/server.rs:133-149`), `rec` (`rust/record/src/recorder.rs:175-179`), lab (`apps/agenteval/src/host.rs:146-154`) | `execute` recebe o `RequestId` e pode terminar depois (`Started`) |
| `Dispatcher` | `rust/accord/src/dispatch.rs:65-71` | app | ide (`apps/idestudio/src/capability/mod.rs:47`), showcase todo | projeta o app no instante e checa contra ele (`rust/accord/src/dispatch.rs:125-142`, `:324-337`); `execute` só depois de aceito ou confirmado (`:131-134`, `:166-170`); `confirm` e `decline` só por `Human` ou `Assistive` (`:275-283`); recibos terminados limitados a 256 (`:55`, `:310-319`); recibos abertos sem limite e sem expiração (`:60-64`); não é `Send` (sinks `Box<dyn ReceiptSink>`, `:70`) |
| `sem::redact` (`REDACTED`, `is_secret`, `length`, `Secrets`) | `rust/accord/src/redactor.rs:26`, `:29-33`, `:37-45`, `:53-137` | quem leva request ou estágio para fora do processo | wire (`rust/socket/src/server.rs:403-404`, `:250`), rec (`rust/record/src/ring.rs:236`) | falha fechado: nó desconhecido conta como secreto (`rust/accord/src/redactor.rs:28-33`); única cópia da regra |
| exemplo do cenário da tese | `demo/commit/src/main.rs` | | | roda com `cargo run -p commit` |

### 2.3 `wire`: protocolo v1, porta, cli

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| mensagens v1: `Ask` (hello, observe, act, confirm, decline), `Reply` (welcome, snapshot, delta, stage, error), `ProtoError` (4) | `rust/socket/src/protocol.rs:40-46`, `:140-146`, `:210-230`; `VERSION` em `:21` | cliente e `Server` | plev, lab, testes | JSON lines, `"v":1`, `"id"` do cliente; versão conferida antes da forma (`rust/socket/src/framing/mod.rs:101-120`); `SnapshotReply.focused` sempre escrito, `null` sem foco (`rust/socket/src/protocol.rs:160`) |
| `Observe {scope, keys, since, bounds, source}` | `rust/socket/src/protocol.rs:64-75` | cliente | `Server::observe` (`rust/socket/src/server.rs:299-363`) | `revision` é sempre a do snapshot inteiro (`rust/socket/src/protocol.rs:151-155`); `since` fora do anel devolve snapshot inteiro (`rust/socket/src/server.rs:310-314`); `keys` filtra só `StateChanged` no delta (`rust/socket/src/server.rs:333-337`); `source` zerado a não ser que pedido (`rust/socket/src/server.rs:349-351`) |
| `Act {target, action, param, observed or from_revision}` | `rust/socket/src/protocol.rs:82-91` | cliente | `Server::act` (`rust/socket/src/server.rs:365-439`) | exatamente um dos dois (`:377-402`); `from_revision` remonta o `observed` pelo `Request::observe` sobre o snapshot guardado (`:385-391`); ator sempre `Agent` com o nome do hello (`:271-275`) |
| `Ring` | `rust/socket/src/ring.rs:17-20`, `DEFAULT_RING = 16` em `:10` | `Server` a cada observe | `Server` | guarda os últimos N snapshots inteiros servidos; servir de novo reposiciona (`rust/socket/src/ring.rs:52-62`) |
| `Host` (project, bounds, dispatch, add_sink, observed) | `rust/socket/src/server.rs:41-71` | app | showcase (`apps/uigallery/src/views/dispatcher/mod.rs:347-371`), lab (`apps/agenteval/src/host.rs:232-263`), toy de teste | o servidor só alcança o app por aqui; `observed` tem implementação vazia padrão (`rust/socket/src/server.rs:68-70`); só o lab implementa `observed` |
| `Traffic` | `rust/socket/src/server.rs:79-91` | server e threads da porta (`rust/socket/src/port.rs:249-259`) | lab (`apps/agenteval/src/report.rs`) | contadores aditivos; `conns` conta toda conexão aceita (`rust/socket/src/port.rs:373-374`) |
| `Server` | `rust/socket/src/server.rs:103-112` | porta ou teste | porta | encaminha só estágios de requests de agente que esta porta emitiu (`rust/socket/src/server.rs:137-139`, `:226-261`) para a conexão que pediu |
| `AgentPort`, `Config`, `enabled` | `rust/socket/src/port.rs:101-107`, `:69-76`, `:90-92` | shell do app | showcase (`apps/uigallery/src/views/dispatcher/mod.rs:322-327`), lab (`apps/agenteval/src/arms/plev.rs:114`) | desligada sem `PLEV_AGENT=1` em debug (`rust/socket/src/port.rs:90-92`, `:171-179`); socket 0600 em diretório 0700 (`:200`, `rust/socket/src/place.rs:42-67`); fila 256, outbox 1024, 64 mensagens por `drain`, linha até 1 MiB, 16 conexões, 10 s de escrita parada (`rust/socket/src/port.rs:42-62`); `drain` roda no thread do app (`:267-299`) |
| `Client` | `rust/socket/src/client.rs:17-22` | agente | plev, lab (`apps/agenteval/src/arms/scripted.rs`), testes | uma conexão por instância (`rust/socket/src/client.rs:25-37`) |
| binário `plev` | `rust/socket/src/bin/plev.rs:37-53` | | agente | uma conexão nova por invocação (`rust/socket/src/bin/plev.rs:192-199`, `:225-240`); sem `--from-revision`, observa o alvo antes (`:262-272`); saída 0 observado ou concluído, 1 recusado, 2 falhou, 3 erro (`:56-59`, `:298-305`) |

### 2.4 `rec`: gravador, sessão, formato `MONS`

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `Recorder` (`sink`, `observed`, `snapshot`, `snapshot_full`, `input`, `frame`, `frame_scene`, `persist`, `export`, `stats`) | `rust/record/src/recorder.rs:28-158` | app | só o `ide-lab` (`apps/agenteval/src/arms/plev.rs:11`, `:109`; `apps/agenteval/src/host.rs:214-215`, `:252-254`); nenhum app que shipa | nunca lê relógio nem cria request (`rust/record/src/recorder.rs:23-26`); `frame` lê só as layers visíveis, não as ligações (`:99-113`) |
| `Sink` | `rust/record/src/recorder.rs:163-179` | `Recorder::sink` | lab | origem `u16` por dispatcher (`:58-64`, `:165`); redige cada estágio contra o último snapshot (`rust/record/src/ring.rs:228-236`) |
| `Event`, `Kind`, `InputKind` | `rust/record/src/event.rs:15-18`, `:23-63`, `:83-90` | recorder | `Session`, lab | `Changes` nunca guarda nó adicionado, parent mudado ou reordenação: esses viram snapshot inteiro (`rust/record/src/event.rs:39-43`); input sem conteúdo (`:78-80`) |
| `Limits`, `Stats` | `rust/record/src/limits.rs:11-47`, `:52-104` | app | lab, testes | padrão 20 s, 32 MiB, keyframe a cada 2 s nas duas trilhas, 120 frames por chunk (`rust/record/src/limits.rs:35-46`); corte só em keyframe |
| arquivo de sessão | `rust/record/src/format.rs:6-43` | `Recorder::persist`, `export` (`rust/record/src/recorder.rs:139-153`) | `Session::open`, `from_bytes` (`rust/record/src/persist.rs:146`, `:154`) | magic `MONS`, versão 0, flags 0, seções `S` `I` `C` `A` com sha256 (`rust/record/src/format.rs:33-43`); `probe` separa `MON0` de `MONS` (`:57-63`); leitor estrito com erro tipado (`:236-305`); sha do git no cabeçalho por `build.rs` (`rust/record/build.rs:24-25`) |
| `Session` | `rust/record/src/session.rs` (`scene_at` `:114`, `state_at` `:190`, `receipts_for` `:247`, `excerpt` `:103`) | leitura | lab (`apps/agenteval/src/arms/plev.rs`), `apps/agenteval/tests/experiment.rs` | não guarda dispatcher nem app: ler nunca cria request |
| `install_panic_hook` | `rust/record/src/hook.rs:27-36` | | nenhum chamador fora de `rust/record/tests/panic.rs` | nunca faz panic; só tenta o lock (`rust/record/src/hook.rs:46-57`) |

### 2.5 engine: ligação, accessibility, headless, frame, relógio

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `Binding { rect, layer, clip, order, view }` | `rust/engine/src/drawing/compositor/binding.rs:27-42` | `bind`, `bind_to_layer` (`:119-153`), chamados por 19 widgets do comps e pelas views do ide e do todo (`apps/idestudio/src/interfaces/panels/unassigned.rs:333`, `apps/uigallery/src/views/apps/todo/draw.rs:29`, `:73`, `:171`) | accessibility (`rust/engine/src/control/accessibility/tree/mod.rs:161`), `Host::bounds` do showcase (`apps/uigallery/src/views/dispatcher/mod.rs:353-362`) | zerada em `begin_frame` (`rust/engine/src/drawing/compositor/mod.rs:76-81`); fora do hash de layer; segundo bind do mesmo id é ignorado com log (`binding.rs:134-139`); `view` fica `None`: nenhum código de produção chama `set_binding_view` (`binding.rs:158-166`) |
| `Compositor::hit(x, y)` | `rust/engine/src/drawing/compositor/binding.rs:184-208` | | nenhum caminho de produção; só testes, bench e o exemplo `measure` | topo por z-order, criação da layer e ordem de bind; não lê `Layer::clip_rect` físico |
| `ViewId`, `HitRegion`, `InputState` | `rust/engine/src/control/input/events.rs:133-146`, `rust/engine/src/control/input/mod.rs:17-61` | `InputState` | `engine::window::App` (`rust/engine/src/runtime/window/mod.rs:53`), builder, exemplos; nenhum app que shipa | contador que volta a zero por frame (`rust/engine/src/control/input/mod.rs:50-61`) |
| projeção AccessKit: `tree::build`, `NodeIds`, `role::of` | `rust/engine/src/control/accessibility/tree/mod.rs:73-121`, `rust/engine/src/control/accessibility/ids.rs:30-39`, `:59`, `rust/engine/src/control/accessibility/role.rs:17` | `Bridge::update` | shell do showcase, `engine::window::App` sem projeção (`rust/engine/src/runtime/window/render.rs:104-113`) | nó sem ligação fica sem bounds; inerte fica escondido; secreto sem valor (`tree.rs:17-27`, `:198-244`); `NodeId` FNV-1a 64 do texto do `SemId` |
| ações do leitor: `action::translate`, `primary`, `Assist` | `rust/engine/src/control/accessibility/action/mod.rs:99`, `:90`, `:40`; ordem da ação primária em `:36` | `Bridge::take` (`rust/engine/src/control/accessibility/bridge.rs:147-149`) | `Access::carry_out` do showcase (`apps/uigallery/src/shell/mod.rs:518-540`) | request observado sobre o snapshot que o leitor ouviu, com `Actor::Assistive`; `Focus` não é capacidade |
| `focus::next` | `rust/engine/src/control/accessibility/focus.rs:48` | | nenhum caminho de produção; só `apps/uigallery/src/views/apps/todo/semantics/assistive.rs:273`, módulo `#[cfg(test)]` (`apps/uigallery/src/views/apps/todo/semantics/mod.rs:340-342`) | |
| `Bridge` | `rust/engine/src/control/accessibility/bridge.rs:73-150` | shell | showcase (`apps/uigallery/src/shell/mod.rs:499-513`), `engine::window::App` (`rust/engine/src/runtime/window/lifecycle.rs:58`) | recusa janela já visível (`bridge.rs:91-93`); handlers só enfileiram e acordam (`:198-209`); projeta só com cliente ativo (`:119-143`) |
| `window::draw_frame`, `present_frame` | `rust/engine/src/runtime/window/frame.rs:107-208`, `:37-83` | | ide (`apps/idestudio/src/interfaces/window/renderer.rs:30`), showcase, headless | um caminho para janela e textura; compõe sem efeitos de layer (`&[]`, `frame.rs:191`); `engine::window::App::render` tem cauda própria (`rust/engine/src/runtime/window/render.rs:5-10`) |
| headless: `Viewport`, `Renderer`, `HeadlessError`, `Frame`, `read_texture` | `rust/engine/src/drawing/headless/mod.rs:36-67`, `:129-222`, `:69-85` | | showcase `--render` (`apps/uigallery/src/shell/window/headless.rs`), lab (`apps/agenteval/src/arms/screenshot.rs`, `bench.rs`), exemplos `text_probe`, `monster_player`, `measure`, teste de custo do rec | `None` sem adapter vira `NoAdapter` e teste pula (`mod.rs:141-146`); fora de wasm (`rust/engine/src/lib.rs:48-49`) |
| `headless::svg::export`, `SvgExport`, `Approximation` | `rust/engine/src/drawing/headless/svg/mod.rs:77`, `:55-58`, `:41-45` | `Renderer::svg` (`rust/engine/src/drawing/headless/mod.rs:205-211`) | `showcase --render --svg` | texto como contorno dos glifos de `TextSystem::shaped_glyphs` (`rust/engine/src/drawing/text/system.rs:288`), o mesmo `ensure_shaped` do desenho (`:247`); toda aproximação vira item; `Approximation.node` é texto |
| `FrameClock`, `AnimationTick { dt, elapsed, since_last }` | `rust/engine/src/compose/animation/mod.rs:13-61` | shell | showcase lê `since_last` (`apps/uigallery/src/shell/mod.rs:435-438`); ide lê só `dt` (`apps/idestudio/src/main.rs:399-400`) | `dt` travado em 0,1 s; `since_last` real, 0 no primeiro tick (`animation/mod.rs:48-60`) |
| `perf::stages`, `RenderConfig::gpu_timestamps` | `rust/engine/src/runtime/perf/stages.rs:255`, `rust/engine/src/drawing/gpu/config.rs:36` | | exemplo `tool/bench/measure/src` | desligados por padrão |

### 2.6 comps: `Wake`, `Blink`, declaração semântica

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `Wake { now, next }` | `rust/engine/src/widgets/wake.rs:22-31`; `merge` `:79-89`; `moving` `:69-75` | ticks de widget e página | só o shell do showcase (`apps/uigallery/src/shell/mod.rs:152-167`, `:438`) | `Some(ZERO)` é contínuo, `None` é nada pendente; `merge` pega qualquer `now` e o menor `next` |
| `Blink` | `rust/engine/src/widgets/wake.rs:107-175` | | `TextInput` (`rust/engine/src/widgets/form/text_input/component.rs:64-69`), `EditorView` (`rust/engine/src/widgets/editor/view.rs:184-186`) | conta em f64; `dt` ou intervalo que não é tempo é ignorado (`wake.rs:148-174`) |
| pontes `bool` que sobram | `TextField::tick` (`rust/engine/src/widgets/form/text_field.rs:216-218`, `pending()`: verdadeiro em todo frame com foco), `EditorView::tick` (`rust/engine/src/widgets/editor/view.rs:191-193`) | | ide (`apps/idestudio/src/interfaces/panels/commit_form.rs:135-136`), urnaui (`apps/urnadbapp/src/view/field.rs:72-74`), exemplo do editor | |
| `.sem(id)` e `project` | 19 widgets: `Button`, `IconButton`, `SubmitButton`, `Chip`, `TextField`, `Checkbox`, `Switch`, `Slider`, `Select`, `Tabs`, `NavLink`, `Breadcrumb`, `SplitPane`, `VirtualList`, `Table`, `Tree`, `ContextMenu`, `Modal`, `Toast` (ex.: `rust/engine/src/widgets/form/text_field.rs:61`, `:80-104`; `rust/engine/src/widgets/content/list.rs:84`, `:105-126`) | app | fora de teste, só `TextField::project` (todo e ide: `apps/uigallery/src/views/apps/todo/semantics/mod.rs:152-153`, `apps/idestudio/src/capability/project.rs:138-139`) e `ToastManager::project` (ide: `apps/idestudio/src/capability/project.rs:49`); a matriz projeta os 19 (`rust/engine/src/widgets/tests/matrix.rs:357`) | helper só lê o widget; `set_value` lê o texto que substitui (`text_field.rs:97-102`); list, table e tree usam o índice como chave (`rust/engine/src/widgets/content/list.rs:115`, `rust/engine/src/widgets/content/table.rs:136`) |
| widgets com evento e sem `.sem` | `Scrollbar` (`rust/engine/src/widgets/feedback/scrollbar.rs:125`), `Card` (`rust/engine/src/widgets/content/card.rs:245`), `GraphView` (`rust/engine/src/widgets/graph/mod.rs:180`), `AppShell` (`rust/engine/src/widgets/shell.rs:124`), `EmptyState` (`rust/engine/src/widgets/content/empty_state.rs:109`), `Sidebar` (`rust/engine/src/widgets/nav/sidebar/input.rs:14`), `EditorView` (`rust/engine/src/widgets/editor/input.rs:43`) | | | nenhuma identidade nem projeção |

### 2.7 ide: `Workbench`, nós projetados, `CommitIf`

| contrato | onde | garantia que o código dá |
|---|---|---|
| `Workbench` (`snapshot`, `dispatch`, `confirm`, `decline`, `receipt`, `add_sink`, `apply`, `tick`, `render`, `now`) | `apps/idestudio/src/capability/mod.rs:41-57`, `:104-108`, `:114-124`, `:130-141`, `:148-156`, `:159-166`, `:172-177`, `:180-182`, `:186-188`, `:318-320` | um dispatcher para todo ator; o relógio dos recibos é `now` (`Instant` desde a criação) |
| entrada da pessoa | `apps/idestudio/src/capability/mod.rs:194-294` | a view roda primeiro e enfileira `UiRequest`; o workbench observa naquele instante e despacha como `Actor::Human` (`:246-294`); o alvo vem de `aim` (`:297-312`) |
| `Semantics` | `apps/idestudio/src/capability/project.rs:39-54` | projeta a partir das views, não do git |
| `Operable` | `apps/idestudio/src/capability/execute.rs:157-219` | toda mutação sai para o worker como `Execution::Started` (`:85-100`); o otimismo da view acontece aqui (`:186-209`) |
| fim no ponto de verdade | `apps/idestudio/src/capability/truth.rs:39-71` | resposta casada com o request mais antigo do mesmo tipo e caminho (`:14-18`, `:73-77`); `CommitRejected` vira `Rejection::Stale` com um `StateChanged` em `changes:staged` (`:127-147`); `Committed` vira `Outcome { commit, files }` (`:103-122`) |
| `GitCommand::CommitIf` | `rust/gitdrv/src/client.rs:48-51`; executado em `rust/gitdrv/src/client.rs:190-199` | o worker lê `status` e compara o conjunto staged com o esperado antes de `git commit` (`rust/gitdrv/src/repo.rs:205-214`); não usa `sem::check` (`rust/gitdrv/Cargo.toml` não depende de `sem`) |
| `GitEvent::Committed(NewCommit { sha, files })`, `CommitRejected { added, removed }` | `rust/gitdrv/src/client.rs:93-100`, `rust/gitdrv/src/domain.rs:73-93` | `files` de `git show --name-only` (`rust/gitdrv/src/repo.rs:229-236`) |
| worker | `rust/gitdrv/src/client.rs:121-131` | um thread, FIFO de um canal; `Refresh` emite `Status`, `Log`, `Branches` em sequência (`:200-204`) |
| toast e modal | `apps/idestudio/src/capability/notice.rs:24-108`, `apps/idestudio/src/capability/person.rs:16-70` | recusa vira toast; request `Confirm::ForAgents` abre modal; fechar sem confirmar chama `decline` como `Actor::Human` (`person.rs:66-70`) |

nós projetados (`apps/idestudio/src/capability/project.rs`):

| id | role | estado | ações (reads) |
|---|---|---|---|
| `changes` | `List` | `files` (List), `staged` (Set), `selected` (Text, navegação) (`:64-67`) | nenhuma |
| `changes.file[<path>]` | `ListItem`, filho de `changes` | `status`, `staged` (Bool), `unstaged` (Bool) (`:78-82`) | `stage` (habilitada com `unstaged`), `unstage` (com `staged`), `discard` (`Confirm::ForAgents`, destrutiva), `ignore`: todas leem o `staged` e o `unstaged` do arquivo; `show_diff` lê `diff:target` (`:83-98`) |
| `diff` | `Region` | `target`, `lines` (primeiras 200), `count` (`:119-122`, `:37`) | nenhuma |
| `commit.form` | `Group` | `open` (`:129-130`) | `open` (lê `open`), `cancel` (lê `open` e `commit.message:text`) (`:131-137`) |
| `commit.message` | `TextInput` (helper do `TextField`), filho de `commit.form` | `text`; `editable`, `required` (`:138-145`) | `set_value` (texto, lê `text`) |
| `commit.submit` | `Button`, filho de `commit.form` | nenhum; habilitado por `can_submit` (`:146-148`) | `submit` (lê `changes:staged` e `commit.message:text`) (`:149-151`) |
| `stacks` | `List` | `branch` (`:155-156`) | nenhuma |
| `stacks.commit[<sha>]` | `ListItem`, filho de `stacks` | `summary`, `author` (`:170-173`) | `show_diff` (lê `diff:target`) (`:174`) |
| `notices` e `notices[<id>]` | `Region`; filhos `Alert` ou `Status` (`rust/engine/src/widgets/feedback/toast.rs:154-176`) | nenhum | `dismiss`, sem reads |

foco: `commit.message` quando o form está visível e o campo focado (`project.rs:50-52`). ids: `apps/idestudio/src/capability/ids.rs:9-46`.

o `ide-lab`: binário `[[bin]] ide-lab` em `apps/agenteval/src/` (`apps/idestudio/Cargo.toml:17-19`), só unix (`apps/agenteval/src/main.rs:18-65`). monta um repositório real, sobe o `Workbench` sem janela, abre `AgentPort::open_with` (`apps/agenteval/src/arms/plev.rs:114`), liga o `Recorder` no relógio do workbench, roda o humano roteirizado (`apps/agenteval/src/host.rs:192-229`) e confere a sessão com o git. braços `plev`, `scripted`, `git`, `screenshot` (`apps/agenteval/src/main.rs:53-57`). é o único lugar do repositório que junta `Workbench`, `wire` e `rec`.

### 2.8 showcase: porta do agente, ponte, `pace`

| contrato | onde | garantia que o código dá |
|---|---|---|
| `ShowcaseView::semantics`, `act`, `add_sink`, `assist_focus` | `apps/uigallery/src/views/dispatcher/mod.rs:242-252`, `:259-269`, `:274-276`, `:280-289` | só a página Todo projeta e despacha; `add_sink` põe o sink sempre no dispatcher do todo; a página Git (o `Workbench` embutido) não é alcançável por agente nem por leitor de tela |
| porta do agente | `apps/uigallery/src/views/dispatcher/mod.rs:322-327`, `:332-337`; `Host` em `:347-371`; aberta em `apps/uigallery/src/shell/mod.rs:486-491` | só desktop e `PLEV_AGENT=1` em debug; `UserEvent::Agent` serve a porta e desenha só se um act rodou (`apps/uigallery/src/shell/mod.rs:187-196`, `:295`) |
| ponte de accessibility | `apps/uigallery/src/shell/mod.rs:219-240`, `:453-461`, `:499-540` | janela criada invisível, ponte, depois visível; a árvore sai do frame recém desenhado |
| `pace` | `apps/uigallery/src/shell/mod.rs:152-167`; `new_events` em `:204-209`; `wake(tick.since_last)` em `:435-438` | `Some(ZERO)` pede o próximo frame, `Some(d)` põe `WaitUntil(agora + d)`, `None` põe `Wait`; `ResumeTimeReached` volta a `Wait` e invalida |
| `TodoPage` | `apps/uigallery/src/views/apps/todo/semantics/mod.rs:146-253` | `todo` (Region, inerte com overlay), `todo.input` (`text`, `set_value` e `submit` lendo `text`), `todo.filter` (`selected`) com três `Tab` (`select` lendo `todo.filter:selected`), `todo.item[<id>]` só dos visíveis (`completed`, `toggle` lendo `completed`, `remove` destrutiva sem reads e sem confirmação), `todo.counter` (`active`, `done`) (`:151-184`) |
| `--render`, `--svg`, `--svg-report`, `--semantics`, `--source` | `apps/uigallery/src/main.rs:112-145`; `stage_page`, `semantics_json`, `render_page` em `apps/uigallery/src/shell/window/headless.rs:75`, `:150`, `:171` | `--semantics` sem GPU; página sem projeção dá `NoSemantics` (`apps/uigallery/src/shell/window/headless.rs:133`) |

## 3. o que a tese precisa e o que existe agora

a tabela da seção 3 do programa (`docs/log/generative-runtime-program.md:78-99`), linha por linha, contra a base. as linhas 21 a 24 são novas: o código tem essas peças e a tabela não.

| # | tese | o que existe hoje | lacuna |
|---|---|---|---|
| 1 | identidade lógica persistente | `SemId` escolhido pelo app (`rust/accord/src/identity.rs:28-29`); ids do todo (`apps/uigallery/src/views/apps/todo/semantics/mod.rs:30-58`) e do ide, por caminho e sha (`apps/idestudio/src/capability/ids.rs:9-46`); `ViewId` continua contador por frame (`rust/engine/src/control/input/mod.rs:50-61`) | fechada em parte (57, 59). list, table e tree do comps usam o índice da linha como chave (`rust/engine/src/widgets/content/list.rs:115`, `rust/engine/src/widgets/content/table.rs:136`): inserir acima desloca identidades |
| 2 | identidade temporária de render e posição | `HitRegion` sem mudança (`rust/engine/src/control/input/events.rs:136-146`); tabela `SemId -> Binding` ao lado da cena (`rust/engine/src/drawing/compositor/binding.rs:27-42`) | fechada em parte (59). `Binding.view` é sempre `None` (nenhuma chamada de produção a `set_binding_view`, `binding.rs:158-166`); `Compositor::hit` não tem chamador de produção; apps e widgets seguem com hit test próprio por `Rect::contains` (ex.: `apps/idestudio/src/interfaces/panels/commit_form.rs:97-107`); `InputState` só no `engine::window::App` (`rust/engine/src/runtime/window/mod.rs:53`) |
| 3 | significado, papel, estado | `sem::Node` (`rust/accord/src/controls.rs:35-75`); 19 widgets com `.sem` e `project`; duas projeções de app: `TodoPage` (`apps/uigallery/src/views/apps/todo/semantics/mod.rs:146-186`) e `Workbench` (`apps/idestudio/src/capability/project.rs:39-54`) | fechada em parte (57, 59). dos 19 helpers, só `TextField` e `ToastManager` têm chamador de produção; as outras páginas do showcase não projetam (`apps/uigallery/src/views/dispatcher/mod.rs:242-252`); sete widgets com evento não têm `.sem` (seção 2.6) |
| 4 | accessibility do mesmo contrato | árvore AccessKit do `Snapshot` e das ligações (`rust/engine/src/control/accessibility/tree/mod.rs:73-121`); `NodeId` pelo FNV do `SemId` (`rust/engine/src/control/accessibility/ids.rs:30-39`); mapa de roles (`rust/engine/src/control/accessibility/role.rs:17`) | fechada (60). o caminho derivado de hit region saiu (`rg view_id_to_node_id crates` vazio) |
| 5 | ação vinda da accessibility | `Bridge` enfileira e acorda (`rust/engine/src/control/accessibility/bridge.rs:198-209`); `action::translate` vira `Request` com `Actor::Assistive` (`rust/engine/src/control/accessibility/action/mod.rs:99`); o shell despacha pelo `act` da página (`apps/uigallery/src/shell/mod.rs:518-540`) | fechada (60). no `engine::window::App` a ponte existe sem projeção e o pedido só vai para o log (`rust/engine/src/runtime/window/render.rs:104-113`) |
| 6 | accessibility nos apps que shipam | showcase desktop com ponte (`apps/uigallery/src/shell/mod.rs:219-240`, `:453-461`) | fechada em parte (60). só a página Todo tem árvore além da janela (`apps/uigallery/src/views/dispatcher/mod.rs:242-252`); a página Git embutida não; o binário `ide` e o urnaui não criam ponte (`apps/idestudio/src/main.rs:212-240`, `apps/urnadbapp/src/app.rs`); mobile e web sem ponte; Tab do showcase não usa `focus::next` (só teste) |
| 7 | ações nomeadas | keymap estilo Zed sem mudança (`rust/engine/src/control/keymap/mod.rs`, zero commits desde `b8ba096`); no ide, as ações de tecla que agem sobre capacidade passam pelo `Workbench` e pelo dispatcher (`apps/idestudio/src/main.rs:175-208`, `apps/idestudio/src/capability/mod.rs:246-294`) | fechada em parte (62). nenhum binding de tecla nomeia uma capacidade (`commit.submit/submit`); o keymap não depende de `sem` |
| 8 | ação com precondição | `Request::observe` (`rust/accord/src/requests.rs:90-131`), `check` (`rust/accord/src/decision.rs:99-138`), dispatcher (`rust/accord/src/dispatch.rs:125-142`); no ide: clique em `apps/idestudio/src/interfaces/panels/layout/input.rs:56-58`, `UiRequest::Submit` em `:262-263`, dispatcher em `apps/idestudio/src/capability/mod.rs:261-294`, `CommitIf` em `apps/idestudio/src/capability/execute.rs:147-154`, conferência no worker em `rust/gitdrv/src/repo.rs:205-214` | fechada (58, 62). resta a janela entre `staged_paths()` e `git commit` (`rust/gitdrv/src/repo.rs:206` e `:213`); o ponto de verdade compara o conjunto de caminhos staged, não o conteúdo staged |
| 9 | estado semântico versus visual no ide | `changes:staged` lido por `submit`, `changes:selected` lido por nenhuma ação (`apps/idestudio/src/capability/project.rs:59-67`, `:149-151`) | fechada (62) |
| 10 | reconciliação otimista | otimismo dentro de `execute` (`apps/idestudio/src/capability/execute.rs:85-100`, `:186-209`); refresh depois de toda resposta de mutação (`apps/idestudio/src/repository/git.rs:126-149`); segunda checagem no worker (`rust/gitdrv/src/client.rs:190-199`) | fechada em parte (62). a corrida do `Refresh` continua (seção 4a) |
| 11 | flight recorder | crate `rec`: `Recorder` (`rust/record/src/recorder.rs:28-158`), arquivo `MONS` (`rust/record/src/format.rs:6-43`), `Session` (`rust/record/src/session.rs:114`, `:190`, `:247`) | fechada em parte (63 fases a, b, c). nenhum app que shipa grava: o único chamador é o `ide-lab` (`apps/agenteval/src/arms/plev.rs:109`); `install_panic_hook` sem chamador de produção; as ligações não são gravadas (`rust/record/src/recorder.rs:99-113`); imagens sem pixels; clip, blur e sombra fora do IR v0 (`rust/record/src/limits.rs:80-83`) |
| 12 | compiler target | `rust/import` sem nenhum commit desde `b8ba096` | continua (68, 71) |
| 13 | pixel para código | `Node.source` por `#[track_caller]` (`rust/accord/src/assembly.rs:56`, `:79-83`); identidade do build no cabeçalho de sessão (`rust/record/build.rs:24-25`, `rust/record/src/format.rs:65-80`); hot reload sem mudança de escopo; `proc-macro2` sem `span-locations` (`Cargo.toml:98`, `rust/engine/Cargo.toml:35`, `:87`) | continua (65). as peças que a 65 consome existem; `Compositor::hit` só é chamado por teste, bench e o exemplo `measure` |
| 14 | headless | `GpuContext::new_headless`, `draw_frame`, `Renderer` (`rust/engine/src/runtime/window/frame.rs:107-208`, `rust/engine/src/drawing/headless/mod.rs:129-222`); svg (`rust/engine/src/drawing/headless/svg/mod.rs:77`); `showcase --render --svg --semantics` (`apps/uigallery/src/main.rs:112-145`) | fechada (66, 66b). `engine::window::App::render` mantém cauda própria com efeitos de layer e HUD (`rust/engine/src/runtime/window/render.rs:5-10`); `draw_frame` compõe sem efeitos de layer (`rust/engine/src/runtime/window/frame.rs:191`) |
| 15 | testes estruturais | hash por layer, `pub(crate)` (`rust/engine/src/drawing/compositor/layer/mod.rs:44`, `:426-432`, `FxHasher`); relatório de aproximação do svg; `--semantics` só da Todo | continua (67). `Approximation.node` é texto (`rust/engine/src/drawing/headless/svg/mod.rs:41-45`) |
| 16 | incrementalidade | cena reconstruída por frame; sondas por estágio (`rust/engine/src/runtime/perf/stages.rs:255`) e timestamps de GPU (`rust/engine/src/drawing/gpu/config.rs:36`), medidos por `tool/bench/measure/src` | continua (73 fase 2 espera decisão). nada incremental implementado; a task-39 continua em `docs/log/steps/` |
| 17 | texto | fontes embutidas sem mudança (`rust/engine/src/drawing/text/fonts.rs:38-52`, zero commits desde `b8ba096`); `TextSystem::shaped_glyphs` novo, sobre o mesmo `ensure_shaped` (`rust/engine/src/drawing/text/system.rs:247`, `:288`) | continua (74) |
| 18 | integração de plataforma | `rust/engine/src/runtime/platform` e `rust/engine/src/runtime/platform/clipboard.rs` sem commits desde `b8ba096` | continua (76) |
| 19 | determinismo | render headless pelo caminho da janela; testes em `rust/engine/src/drawing/headless/tests.rs` | continua (77). só verificado em Metal (task-66, fechamento) |
| 20 | padrão de app | `docs/adr/structure/0023-official-app-pattern.md` sem mudança; o formato `Semantics + Operable` com o dispatcher na página existe em dois apps (todo e ide) | continua para superfície vinda de IR (68, 69) |
| 21 | superfície do agente | `wire`: protocolo v1 (`rust/socket/src/protocol.rs`), `Server` e `Host` (`rust/socket/src/server.rs:41-112`), `AgentPort` (`rust/socket/src/port.rs:101-299`), `plev` (`rust/socket/src/bin/plev.rs`) | existe (61, 57b). servida pelo showcase (só Todo) e pelo `ide-lab`; o binário `ide` não abre porta (`apps/idestudio/src/main.rs:13-35`) |
| 22 | render on demand de algo que muda sozinho | `comps::Wake` e `Blink` (`rust/engine/src/widgets/wake.rs:22-175`), `AnimationTick::since_last` (`rust/engine/src/compose/animation/mod.rs:20-23`), `pace` (`apps/uigallery/src/shell/mod.rs:152-167`) | fechada em parte (82 fase a). ide e urnaui ainda em `tick -> bool`; dívidas na seção 4g |
| 23 | segredo nunca sai do processo | regra única `sem::redact` (`rust/accord/src/redactor.rs:26-137`), usada pelo wire (`rust/socket/src/server.rs:403-404`, `:250`) e pelo rec (`rust/record/src/ring.rs:236`) | fechada (57, 57b). o recibo em memória guarda o parâmetro cru (`rust/accord/src/receipts.rs:147-152`): quem lê o dispatcher no processo vê o valor |
| 24 | medição por estágio | `perf::stages` e `gpu_timestamps` (linha 16); tabelas em `docs/log/steps/task-73-measurements.md` | existe (73 fases 1 e 1b) |

## 4. defeitos e limites conhecidos, conferidos no código

### 4a. a corrida do `Refresh` no ide

confirmada no código. sem correção e sem teste que a exercite.

o mecanismo, com dois arquivos A e B:

1. um ator pede `stage` de A. `execute` põe `GitCommand::Stage { A }` na fila do worker e já marca A como staged na view (`apps/idestudio/src/capability/execute.rs:186-191`, via `mutate` em `:85-100`; `apps/idestudio/src/interfaces/panels/unassigned.rs:153-169`).
2. o worker roda o stage e responde `Staged { A }` (`rust/gitdrv/src/client.rs:174-177`).
3. na thread da UI, `Workbench::apply` fecha o recibo (`apps/idestudio/src/capability/truth.rs:39-71`) e `GitSession::apply` manda `GitCommand::Refresh` para o worker (`apps/idestudio/src/repository/git.rs:129-137`, que chama `:55-59`). chamo esse refresh de R1.
4. o `stage` de B chega ao dispatcher depois do passo 3 e antes de a UI aplicar o `Status` de R1. `execute` põe `Stage { B }` na fila atrás de R1 e marca B como staged na view. no `ide-lab` essa ordem é a regra quando os dois chegam na mesma volta do loop: primeiro aplica os eventos do git (`apps/agenteval/src/arms/plev.rs:178-191`), depois drena a porta do agente (`:193`).
5. o worker é um thread com um canal, FIFO (`rust/gitdrv/src/client.rs:121-131`). ele roda R1 antes de `Stage { B }`, e R1 lê `repo.status()` nesse instante (`rust/gitdrv/src/client.rs:200-201`): A staged, B não.
6. o worker roda `Stage { B }`. o index real agora tem A e B.
7. a UI aplica o `Status` de R1: `set_files` troca a lista inteira (`apps/idestudio/src/repository/git.rs:91-95`, `apps/idestudio/src/interfaces/panels/unassigned.rs:189-193`). não há número de geração nem conferência com mutação pendente. B volta a não staged na view e sai de `changes:staged` (`apps/idestudio/src/capability/project.rs:59-67`), embora já esteja no index.
8. a UI aplica `Staged { B }` e manda R2 (`apps/idestudio/src/repository/git.rs:129-137`). o `Status` de R2 devolve B. entre os passos 7 e 8 a projeção diz que B não está staged.

o que isso produz no intervalo:

- um agente que observou A e B staged antes do passo 7 e pede `submit` depois dele recebe `Stale` do dispatcher, com "B left the staged files", sem ninguém ter tirado B (`rust/accord/src/decision.rs:116-119`; texto em `apps/idestudio/src/capability/notice.rs:121-129`).
- um agente que observa no intervalo vê só A. se pedir `submit`, o worker recusa com `CommitRejected { added: [B] }`, porque o index tem A e B (`rust/gitdrv/src/repo.rs:205-212`).
- o mesmo vale ao contrário: um `unstage` seguido de um refresh velho devolve o arquivo à lista staged.

outras fontes do mesmo refresh velho: `Committed` e `CommitRejected` também mandam `Refresh` (`apps/idestudio/src/repository/git.rs:138-149`); no binário `ide`, o watcher de arquivos manda outro `Refresh` depois de 300 ms de silêncio no disco, `.git` incluído (`apps/idestudio/src/repository/watcher.rs:16`, `:45-58`; `apps/idestudio/src/main.rs:245-249`).

quem desvia dela hoje: o rig dos testes espera o recibo e o `Branches` do refresh antes do passo seguinte (`apps/idestudio/tests/support/mod.rs:79-88`); o agente roteirizado observa depois de cada ato (`apps/agenteval/src/arms/scripted.rs:90-92`) e espera a projeção repetir a mesma revisão com o conjunto da tarefa (`:95-100`, `:184-199`; até 200 tentativas a cada 20 ms, `:25-27`).

### 4b. `Change::NodeAdded` carrega só o id

confirmado. `Change::NodeAdded(SemId)` (`rust/accord/src/mutation.rs:23`), produzido em `rust/accord/src/mutation.rs:98-101`. o delta do wire é o `Snapshot::diff` do escopo (`rust/socket/src/server.rs:329-340`): o agente recebe o id e precisa de outro `observe` para ver o nó. `ParentChanged` e `Reordered` também não carregam o que mudou de lugar (`rust/accord/src/mutation.rs:54-58`, `:76-78`). o rec contorna do lado dele: projeção com nó novo, parent mudado ou reordenação vira snapshot inteiro (`rust/record/src/event.rs:39-43`).

### 4c. o nó `stacks.commit[sha]` não expõe os arquivos

confirmado. o nó projeta só `summary` e `author` (`apps/idestudio/src/capability/project.rs:170-173`). o log não traz arquivos: `Commit { sha, short_sha, message, author, time }` (`rust/gitdrv/src/domain.rs:28-39`). a lista existe em dois lugares que o nó não usa: o `Outcome` do recibo do `submit` (`apps/idestudio/src/capability/truth.rs:114-121`) e `GitRepo::commit_files` (`rust/gitdrv/src/repo.rs:229-236`). hoje um agente chega aos arquivos por `show_diff` e pelos cabeçalhos `path:` das linhas do diff (`rust/gitdrv/src/domain.rs:61-67`), limitados às primeiras 200 linhas projetadas (`apps/idestudio/src/capability/project.rs:37`, `:104-107`).

### 4d. a recusa stale não diz qual ator causou a mudança

confirmado. `Rejection::Stale { changes }` (`rust/accord/src/decision.rs:46`) e `Change::StateChanged` (`rust/accord/src/mutation.rs:29-35`) não têm campo de ator. `check` é pura sobre o request e o snapshot (`rust/accord/src/decision.rs:99`) e não vê recibos. a recusa no ponto de verdade monta o mesmo `Stale` sem ator (`apps/idestudio/src/capability/truth.rs:127-147`). no ide, o toast diz quem foi recusado (`apps/idestudio/src/capability/notice.rs:103-106`) e o que mudou (`:111-144`), nunca quem mudou. o dado existe, sem ligação com a recusa: o recibo do request que mudou o estado fica no dispatcher (`rust/accord/src/dispatch.rs:65-71`) e, quando há gravação, na trilha do rec. uma mudança feita fora da UI (um terminal) não tem ator nenhum.

### 4e. a regra "stale é por `reads`"

no código: `stale` compara um `StateChanged` por `read` da ação, como o snapshot atual declara (`rust/accord/src/decision.rs:166-181`), chamada em `:116-119`, documentada em `:82-87`; a revisão é só rastreio (`rust/accord/src/requests.rs:42-49`).

quem escreve a regra: `docs/adr/semantics/0033-actions-carry-observed-state.md:42-48` e `:148`; `docs/adr/semantics/0039-agents-act-through-the-semantic-contract.md:292-293`; `docs/adr/semantics/0036-commit-checks-the-index-it-observed.md:142`; `docs/ref/code-against-the-plev-engine.md:61-63` e `:151-153`.

quem não escreve: as superfícies que um agente de fora lê. o `USAGE` do `plev` (`rust/socket/src/bin/plev.rs:37-53`) e a doc do protocolo (`rust/socket/src/lib.rs:1-28`, `rust/socket/src/protocol.rs:77-80`) não dizem. o que o agente vê é a lista `reads` de cada ação na observação (`rust/accord/src/manifest.rs:189-193`), omitida quando vazia. é coerente com o experimento: a regra foi deduzida, não lida (`docs/log/steps/task-62-ide-human-agent-proof.md:669-670`).

### 4f. cada comando `plev` abre uma conexão nova

confirmado. `run` chama `report`, que chama `connect` (`rust/socket/src/bin/plev.rs:76-89`, `:192-199`, `:225-240`): um `Client::connect` e um `hello` por processo, e nada guarda conexão entre invocações. o `Client` aceita várias mensagens numa conexão (`rust/socket/src/client.rs:50-150`); só o cli não usa.

a mais: com `--app <nome>`, ou sem `--app`, `find` abre e fecha uma conexão com cada socket candidato para saber se está vivo (`rust/socket/src/place.rs:84-111`, `:100`), e a porta aceita e conta essa conexão (`rust/socket/src/port.rs:373-374`). cada `plev` por nome gera duas conexões na porta. `plev act` sem `--from-revision` manda também um `observe` antes do `act` (`rust/socket/src/bin/plev.rs:262-272`).

### 4g. dívidas da task-82

- `EditorView` pisca sem foco: confirmado. `wake` não olha foco (`rust/engine/src/widgets/editor/view.rs:184-186`), e a página Editor entra no `page_wake` enquanto visível (`apps/uigallery/src/views/dispatcher/mod.rs:157`).
- caret do dock contínuo: confirmado. o modelo acumula `blink_t` (`apps/uigallery/src/domain/dock.rs:130-133`) e mantém `update` verdadeiro enquanto o campo aparece (`:245`); a página eleva isso a movimento contínuo (`apps/uigallery/src/views/dispatcher/mod.rs:152`).
- frames avulsos do worker do git e da ponte de accessibility: confirmado. `UserEvent::Wake` invalida e desenha sempre, mesmo com outra página visível (`apps/uigallery/src/shell/mod.rs:285`, callback em `:492-494`); `UserEvent::Access` idem (`:286-294`); a ativação do leitor também acorda (`rust/engine/src/control/accessibility/bridge.rs:190-194`).
- o ide segue em 60 fps com o campo de commit focado: `TextField::tick` é `pending()` (`rust/engine/src/widgets/form/text_field.rs:216-218`), `CommitForm::tick` repassa (`apps/idestudio/src/interfaces/panels/commit_form.rs:135-136`) e o shell pede frame enquanto for verdadeiro (`apps/idestudio/src/main.rs:398-405`). é a fase b.
- achado a mais, fora da lista da task-82: um toast na tela do ide pede frame contínuo até sair (`apps/idestudio/src/interfaces/panels/layout/mod.rs:317`). e a página Git do showcase eleva o `bool` do `Workbench::tick` a `Wake::moving` (`apps/uigallery/src/views/apps/git/desktop/mod.rs:205-211`, `apps/uigallery/src/views/compositor/render.rs:32-33`): com a página Git visível e o campo de commit focado, ou um toast do ide na tela, o showcase volta a desenhar todo frame.
- urnaui: oito `tick -> bool` em `apps/urnadbapp/src/view/`, o campo via `TextField::tick` (`apps/urnadbapp/src/view/field.rs:72-74`). não compila sem `../urna`; lido, não executado.

### 4h. `TODO`, `FIXME`, `unimplemented!`, `todo!` em código de produção

busca em `rust/accord`, `rust/socket`, `rust/record`, `apps/idestudio`, `rust/gitdrv`, `rust/engine/src`, `rust/engine/src/widgets`, `apps/uigallery/src`, fora de teste:

- `TODO`, `FIXME`, `XXX`, `HACK`: nenhum, em maiúscula ou em comentário minúsculo.
- `unimplemented!`, `todo!`: nenhum.
- da mesma família de panic, `unreachable!`: `rust/gitdrv/src/client.rs:205` (`GitCommand::Shutdown`, que o laço já trata em `:126-128`) e `apps/uigallery/src/views/text_input/input.rs:37` (`EditKey::Tab`).

### 4i. outros limites que os arquivos citam como abertos

| limite | citado em | no código | estado |
|---|---|---|---|
| id duplicado: o segundo nó sai e suas checagens não rodam | `docs/log/steps/task-57-semantic-contract.md:53` | `rust/accord/src/validate.rs:22-28` | confirmado |
| foco em nó não focável ou inerte não é recusado | task-57 done `:52` | `rust/accord/src/validate.rs:33-37` confere só existência | confirmado |
| `Value::Num` NaN ou infinito não trafega em JSON | task-57 done `:51` | o wire escreve `null` e o leitor recusa (`rust/socket/src/framing/mod.rs:58-62`); o rec descarta o evento e os `Changes` que dependem dele (`rust/record/src/persist.rs:43-72`) | confirmado |
| forma JSON sem golden | task-57 done `:49-50` | `rust/accord/tests/encoding.rs` | resolvido (58, 57b) |
| recibos abertos sem limite; confirmação não expira | `docs/log/steps/task-58-actions-preconditions-receipts.md:54-55` | `rust/accord/src/dispatch.rs:60-64`; nenhuma API de expiração | confirmado. recusar uma confirmação existe (`rust/accord/src/dispatch.rs:188-198`, 58b) |
| parâmetro cru no request | task-58 done `:56-57` | redigido ao sair do processo (`rust/socket/src/server.rs:403-404`, `rust/record/src/ring.rs:236`); cru no recibo em memória (`rust/accord/src/receipts.rs:147-152`) | resolvido para quem leva para fora |
| range de `Int` comparado como f64; `step` não validado | task-58 done `:47-48`, `:58` | `rust/accord/src/decision.rs:239-240`, `:97-98` | confirmado |
| `Dispatcher` não é `Send` | task-58 done `:59` | `rust/accord/src/dispatch.rs:70` | confirmado; toda porta drena no thread da UI (`rust/socket/src/port.rs:261-267`) |
| chaves por índice; agente só alcança linhas visíveis | `docs/log/steps/task-59-semantic-binding-and-one-path.md:85-87` | `rust/engine/src/widgets/content/list.rs:113-117`, `rust/engine/src/widgets/content/table.rs:134-138` | confirmado |
| scrollbar e scrim do modal sem ligação | task-59 done `:87` | `rust/engine/src/widgets/feedback/scrollbar.rs` sem `.sem`; o modal liga só a folha (`rust/engine/src/widgets/feedback/modal.rs:239`), o scrim é desenhado em `:231` | confirmado |
| sem helper de volta do id de um filho para o índice | task-59 done `:88` | nenhuma função assim no comps (`rg`) | confirmado |
| `hit` não lê o `clip_rect` físico da layer; `Binding.view` `None` | task-59 done `:106-108` | `rust/engine/src/drawing/compositor/binding.rs:184-208`, `:158-166` | confirmado |
| contador e `todo.filter` sem rect ligado | task-59 done `:108-109` | o todo liga `todo`, itens e abas (`apps/uigallery/src/views/apps/todo/draw.rs:29`, `:73`, `:171`) | confirmado |
| só o todo projeta; chrome não; Tab sem `focus::next`; só ação primária pelo clique | `docs/log/steps/task-60-accessibility-projection.md:97-99` | `apps/uigallery/src/views/dispatcher/mod.rs:242-252`; `rust/engine/src/control/accessibility/focus.rs:48` só em teste; `rust/engine/src/control/accessibility/action/mod.rs:36` | confirmado |
| no macOS a árvore é montada a cada frame depois da primeira consulta | task-60 done `:99-101` | `rust/engine/src/control/accessibility/bridge.rs:119-143` pede `update_if_active` todo frame | não confirmado: quem decide "ativo" é o `accesskit_winit` (seção 8) |
| nome do ator declarado, não autenticado | `docs/log/steps/task-61-agent-surface.md:67-69` | `rust/socket/src/server.rs:277-295` | confirmado |
| processo morto deixa o arquivo do socket | task-61 done `:69` | `rust/socket/src/place.rs:69-80` limpa só o socket do mesmo pid; `find` ignora socket sem resposta (`:100`) | confirmado |
| descritor da porta herdado por processo filho | task-61 done `:69-71`, `docs/adr/semantics/0039-agents-act-through-the-semantic-contract.md:255-276` | `rust/socket/src/port.rs:198-199` não trata `FD_CLOEXEC` | confirmado que o código não trata; o efeito é do std no macOS |
| `SnapshotReply.focused` escrito como `null` | `docs/log/steps/task-57b-lean-wire-form-and-one-redaction.md:65-66` | `rust/socket/src/protocol.rs:160` | confirmado |
| resposta do worker casada pela ordem | `docs/log/steps/task-62-ide-human-agent-proof.md:76-78` | `apps/idestudio/src/capability/truth.rs:14-18`, `:73-77` | confirmado |
| modal de confirmação não projetado | task-62 `:81-82` | `apps/idestudio/src/capability/project.rs:39-54` não projeta modal | confirmado; o ide não tem ponte de accessibility |
| corrida entre a leitura do index e o `git commit` | task-62 `:83-85`; `docs/adr/semantics/0036-commit-checks-the-index-it-observed.md:115-121` | `rust/gitdrv/src/repo.rs:206` e `:213` | confirmado |
| fechar o modal deixa o recibo aguardando | task-62 `:79-80`; ADR commit-checks `:125-127` | `apps/idestudio/src/capability/person.rs:64-70` chama `decline` | resolvido (58b) |
| `set_value` conclui sem eco do valor | task-62 `:670-671` | todo: `Outcome` vazio (`apps/uigallery/src/views/apps/todo/semantics/mod.rs:199`, `:230`); ide: `apps/idestudio/src/capability/execute.rs:180-183` | confirmado |
| nenhum app grava; ligações não gravadas; imagens sem pixels | `docs/log/steps/task-63-flight-recorder.md:64-71`; `docs/adr/recording/0037-session-recording-replay-not-reexecution.md:158-163`, `:236-238` | `rust/record/src/recorder.rs:99-113`; `rust/record/src/persist.rs:39-40` | confirmado |
| cauda própria do `engine::window::App::render` | `docs/log/steps/task-66-headless-first-class.md:53-55` | `rust/engine/src/runtime/window/render.rs:5-10` | confirmado |
| `Approximation.node` texto; imagem, backdrop blur e efeitos de layer fora do svg | `docs/log/steps/task-66b-svg-export.md:251-265` | `rust/engine/src/drawing/headless/svg/mod.rs:41-45`; `rust/engine/src/runtime/window/frame.rs:191` | confirmado para o tipo e para os efeitos de layer; os outros itens são do relatório, não conferidos linha a linha |
| o pass de layer grava o attachment multisample | `docs/log/steps/task-73-incremental-dependency-system.md:41-42` | `rust/engine/src/runtime/window/render_passes.rs:137-148` (`StoreOp::Store` com `resolve_target`) | mecanismo confirmado; custo não medido |
| `Select` sem barra de rolagem; teclado como métodos, não ligado na página | `docs/log/steps/task-81-select-scrolling-panel.md:22-26` | nenhum chamador de `highlight_next` no showcase (`rg`) | confirmado |

## 5. onde a documentação diverge do código

cada item: onde o doc diz, onde o código faz, o que diz e o que faz. vale o código.

### 5.1 README e arc.toml

1. `README.md:10-12`, `docs/ATLAS.toml:339-340` e `:394`: "render on demand: no change, no frame, and an unchanged layer costs zero GPU work". código: o binário `ide` pede frame todo frame com o campo de commit focado ou com um toast na tela (`apps/idestudio/src/main.rs:398-405`, `apps/idestudio/src/interfaces/panels/layout/mod.rs:312-318`); o showcase desenha um frame a cada acordada do worker do git e da ponte de accessibility, mude algo ou não (`apps/uigallery/src/shell/mod.rs:285-294`); e todo frame desenhado compõe todas as layers visíveis, sem mudança inclusive (`rust/engine/src/runtime/window/render_passes.rs:592-609`). sem mudança a layer não é redesenhada, mas custa o composite.
2. `README.md:139`: "`monster` ... magic `MON0`, frozen at v1". código: `VERSION: u16 = 0` (`rust/motion/src/encoding/container.rs:50-51`), fixtures `rust/motion/tests/fixtures/golden_v0_minimal.monster` e `golden_v0_ops.monster`. o programa repete "container v1 congelado" (`docs/log/generative-runtime-program.md:90`). o campo gravado é 0; se "v1" nomeia a spec e não o campo, o texto não diz.
3. `README.md:114`: o tier engine lista "gpu, compositor, text, path, layout, input, animation, signal, theme, graph, builder, window, platform". código: `accessibility`, `headless`, `perf`, `actions`, `effects` e `hot_reload` também são módulos públicos (`rust/engine/src/lib.rs:6`, `:22`, `:38-39`, `:47-58`).
4. `README.md:55` e `docs/ATLAS.toml:160-163`: o ide é "a lib + bin" com `main.rs` como shell. código: o crate tem um segundo binário, `ide-lab` (`apps/idestudio/Cargo.toml:13-19`, `apps/agenteval/src/main.rs:1-65`), o laboratório da fase c da task-62 (seção 2.7). nenhum dos dois docs o nomeia.
5. `docs/ATLAS.toml:163`: `[crates.ide] depends = ["git", "engine", "comps", "sem"]`. código: também `wire` e `rec` (`apps/idestudio/Cargo.toml:38-39`), usados só pelo `ide-lab`.
6. `docs/ATLAS.toml:144`: `[crates.showcase] depends = ["engine", "comps", "sem"]`. código: também `rope`, `ide`, `git` e `wire` (`apps/uigallery/Cargo.toml:22`, `:37-41`).
7. `docs/ATLAS.toml:38-125`: a lista de módulos do engine não tem `[modules.accessibility]` (nem `actions`, `effects`). código: `rust/engine/src/control/accessibility/mod.rs:1-28`, ligado por feature em `rust/engine/src/lib.rs:52-53`, com `Bridge`, `tree`, `action`, `focus`, `ids`, `role`.
8. `docs/ATLAS.toml:390-391` (diagrama) e `:66` (`[modules.input]`): o input vai "hit regions -> events drained" para o app. código: os shells que shipam mandam `WidgetEvent` direto para a view (`apps/uigallery/src/shell/mod.rs:379-423`) ou para o workbench (`apps/idestudio/src/main.rs:290-386`); `InputState` e suas hit regions só existem no `engine::window::App` (`rust/engine/src/runtime/window/mod.rs:53`) e no builder.
9. `docs/ATLAS.toml:180` (`[crates.wire]`): não cita `Host::observed` nem `Traffic`, que existem desde a fase c da 62 (`rust/socket/src/server.rs:68-91`). omissão.
10. `docs/ATLAS.toml:230-246` (`[examples]`): não lista o exemplo `tool/bench/measure/src`, citado só dentro do papel de `perf` (`:74`). omissão.

### 5.2 manual do engine e how-to

11. `docs/ref/code-against-the-plev-engine.md:57-58`: "nothing in the engine, comps or the apps uses it yet". código: engine, comps, showcase e ide dependem de `sem` (`rust/engine/Cargo.toml:59`, `rust/engine/src/widgets/Cargo.toml:28`, `apps/uigallery/Cargo.toml:28`, `apps/idestudio/Cargo.toml:27`).
12. `docs/ref/code-against-the-plev-engine.md:73-76` e `docs/adr/semantics/0035-semantic-binding-beside-the-scene.md:12-14`: "every interactive comps widget takes `.sem(id)` and has a `project` helper". código: 19 têm; `Scrollbar`, `Card`, `GraphView`, `AppShell`, `EmptyState`, `Sidebar` e `EditorView` tratam evento e não têm (`rust/engine/src/widgets/feedback/scrollbar.rs:125`, `rust/engine/src/widgets/content/card.rs:245`, `rust/engine/src/widgets/graph/mod.rs:180`, `rust/engine/src/widgets/shell.rs:124`, `rust/engine/src/widgets/content/empty_state.rs:109`, `rust/engine/src/widgets/nav/sidebar/input.rs:14`, `rust/engine/src/widgets/editor/input.rs:43`).
13. `docs/ref/code-against-the-plev-engine.md:98-102`: "the only identity the engine itself has is still `ViewId`". código: o engine guarda `SemId` na tabela de ligação (`rust/engine/src/drawing/compositor/binding.rs:47`) e na accessibility (`rust/engine/src/control/accessibility/ids.rs:59`).
14. `docs/ref/code-against-the-plev-engine.md:156-159`: "a click, a keystroke, a screen reader action, an agent and a test all become a `sem::Request`". código: digitar não passa pelo dispatcher: no todo (`apps/uigallery/src/views/apps/todo/mod.rs:258-285`) e no ide (`apps/idestudio/src/main.rs:127-138`, backspace em `:199-201`), o texto projetado (`todo.input:text`, `commit.message:text`) muda sem request nem recibo. o ADR de ligação isenta a digitação (`docs/adr/semantics/0035-semantic-binding-beside-the-scene.md:105-107`); o manual não.

### 5.3 ADRs dos contratos

15. `docs/adr/semantics/0033-actions-carry-observed-state.md:85-86`: o ponto de verdade confere os valores observados "with the same `check`". código: o worker compara dois `BTreeSet` de caminhos (`rust/gitdrv/src/repo.rs:205-214`); `rust/gitdrv` não depende de `sem` (`rust/gitdrv/Cargo.toml:9-15`); `sem::check` só é chamado pelo dispatcher (`rust/accord/src/dispatch.rs:330`). o worker confere só `changes:staged`, não `commit.message:text`.
16. `docs/adr/semantics/0036-commit-checks-the-index-it-observed.md:143`: evitar "a view that changes git state or projected state outside `execute`". código: o `Status` do refresh troca `changes:staged` fora de `execute` (`apps/idestudio/src/repository/git.rs:91-95`), a digitação troca `commit.message:text` (`apps/idestudio/src/main.rs:127-138`), a navegação troca `changes:selected` (`apps/idestudio/src/capability/mod.rs:203-209`; o próprio ADR aceita a navegação em `:49-52`). o primeiro é o mecanismo da seção 4a.
17. `docs/adr/semantics/0036-commit-checks-the-index-it-observed.md:125-127`: "`sem` has no way to decline ... the receipt stays awaiting". código: `Dispatcher::decline` (`rust/accord/src/dispatch.rs:188-198`), chamado quando a pessoa fecha o modal (`apps/idestudio/src/capability/person.rs:64-70`). resolvido pela 58b, o ADR não foi atualizado.
18. `docs/adr/semantics/0039-agents-act-through-the-semantic-contract.md:237-240`: "the ide opens the port on its own shell (task-62 phase c) with its `Workbench` as the host". código: o binário `ide` não importa `wire` e não abre porta (`apps/idestudio/src/main.rs:13-35`, `:427-468`); só o `ide-lab` abre (`apps/agenteval/src/arms/plev.rs:114`). o arquivo da task diz o mesmo que o ADR (`docs/log/steps/task-62-ide-human-agent-proof.md:155-156`, `:257`).
19. `docs/adr/semantics/0039-agents-act-through-the-semantic-contract.md:48-49`: os campos de `observe` sem `source`; e `:192-216`: a tabela de bytes e tokens da forma verbosa (6419 B para 3 itens). código: `Observe.source` existe (`rust/socket/src/protocol.rs:73-74`) e a forma é a enxuta (`rust/accord/src/controls.rs:39-74`); o próprio ADR aponta a 57b em `:244-248`, a tabela ficou.
20. `docs/adr/interface/0038-accessibility-projects-the-semantic-snapshot.md:171`: `ShowcaseView::assist`. código: `ShowcaseView::act` (`apps/uigallery/src/views/dispatcher/mod.rs:259`).
21. `docs/adr/interface/0038-accessibility-projects-the-semantic-snapshot.md:218-220`: `Theme::intent_role` "stays until theme is next touched". código: removido (commit `263c00c`; `rg intent_role crates` vazio).
22. `docs/adr/interface/0038-accessibility-projects-the-semantic-snapshot.md:140-144`: Tab e Shift-Tab andam pelo snapshot e substituem o `FocusGraph`. código: `focus::next` (`rust/engine/src/control/accessibility/focus.rs:48`) só tem chamador em teste; o Tab do showcase segue o ciclo de cada página (`apps/uigallery/src/keyboard.rs:18`). o próprio ADR admite em `:203-205`.
23. `docs/adr/semantics/0032-semantic-contract-as-pure-projection.md:104-105`: "`ActionName` is the capability a binding can target". código: o keymap não conhece `sem` (`rust/engine/src/control/keymap/` sem nenhuma referência) e nenhum binding nomeia uma capacidade; o ADR do ide deixou isso de fora de propósito (`docs/adr/semantics/0036-commit-checks-the-index-it-observed.md:63-65`).
24. `docs/adr/recording/0037-session-recording-replay-not-reexecution.md:253-254`: a ligação do gravador no showcase "waits for task-60". código: a 60 está integrada e o showcase continua sem `rec` (`apps/uigallery/Cargo.toml:19-45`); o motivo hoje é a fase d da 63 (programa `:303`).
25. `docs/adr/rendering/0042-a-tick-asks-for-a-deadline.md:53-57`: `tick -> bool` sobra no ide, no exemplo do editor e nos laços de settle. código: sobra também na página Git do showcase, que eleva o `bool` do `Workbench::tick` a movimento contínuo (`apps/uigallery/src/views/compositor/render.rs:32-33`, `apps/uigallery/src/views/apps/git/desktop/mod.rs:205-211`).

### 5.4 o programa

26. `docs/log/generative-runtime-program.md:70-99` (seção 3): a tabela inteira descreve a base `plevrev b8ba096` (`:72-74`). exemplos que o código já não tem: `PlevActionHandler::do_action` em `rust/engine/src/runtime/window/mod.rs:221` (`:84`; não existe, `rg PlevActionHandler crates` vazio); `GitCommand::Commit` em `apps/idestudio/src/repository/git.rs:76` e `git commit` em `rust/gitdrv/src/repo.rs:187` (`:87`; hoje `CommitIf` em `rust/gitdrv/src/client.rs:48-51` e `commit` em `rust/gitdrv/src/repo.rs:189`); árvore de hit region em `window/render.rs:99-125` (`:83`); "nenhum app que shipa expõe árvore" (`:85`); headless "só texto" (`:93`). a seção 3 deste arquivo refaz a tabela.
27. `generative-runtime-program.md:198` (seção 7): "ligação por frame `SemId -> (ViewId, bounds, layer, clip)`". código: `Binding.view` existe e fica sempre `None` (`rust/engine/src/drawing/compositor/binding.rs:41`, `:158-166`).
28. `generative-runtime-program.md:200-201` (seção 7): protocolo em "crate novo (nome na task)" e formato de sessão em "`rust/motion` ... ou crate irmão, decidido em ADR". código: os dois estão decididos e existem: `rust/socket` e `rust/record/src/format.rs:1-43`.
29. `generative-runtime-program.md:215` (seção 8): `rust/motion/src/encoding/container.rs` e formato pertencem à 63. código: a 63 escolheu o formato irmão e nunca tocou esse arquivo (nenhum commit em `rust/motion/src/encoding/container.rs` desde `b8ba096`). a seção 8 não tem linha para `rust/socket`, `rust/record` nem para o shell `apps/uigallery/src/shell/mod.rs`, que 60, 61 e 82 tocaram e 63 fase d, 65 e 75 vão tocar.
30. `generative-runtime-program.md:300` (seção 12): task-66 "SVG e `--semantics` foram para a task-66b (pendente)". código: existem (`rust/engine/src/drawing/headless/svg/mod.rs:77`, `apps/uigallery/src/main.rs:115-118`); a linha `:310` do mesmo programa já diz que a 66b está integrada.

### 5.5 arquivos de task

31. `docs/log/steps/task-62-ide-human-agent-proof.md:300`: o laboratório em `apps/agenteval/src.rs`. código: é um diretório, `apps/agenteval/src/main.rs` (`apps/idestudio/Cargo.toml:17-19`); a mesma task acerta em `:441`.
32. `docs/log/steps/task-62-ide-human-agent-proof.md:153-156` (fase b): o shell do ide abre o `AgentPort`. código: não abre (item 18).

### 5.6 o `ide-lab`

o que é e onde: binário `ide-lab` do crate `ide` (`apps/idestudio/Cargo.toml:13-19`), fonte em `apps/agenteval/src/` (onze arquivos: `main.rs`, `args.rs`, `bench.rs`, `gitarm.rs`, `host.rs`, `lab.rs`, `repo.rs`, `report.rs`, `screenshot.rs`, `scripted.rs`, `story.rs`), só unix. é o app sob experimento e o experimentador: repositório real em diretório temporário, `Workbench` sem janela, `AgentPort` aberto como `ide`, `Recorder` no relógio do workbench, humano roteirizado, braços `plev`, `scripted`, `git` e `screenshot`, relatório em JSON e sessão `.monster` conferida com o git. roda no gate por `apps/agenteval/tests/experiment.rs`. não aparece em `docs/ATLAS.toml` nem no `README.md`; está descrito só no arquivo da task-62 (`:439-567`).

## 6. testes

contagem de `#[test]` por arquivo, lida no código (não rodada). "ignorado" é `#[ignore]`: fora do gate.

### 6.1 `sem`

| arquivo | testes | cobre |
|---|---|---|
| `rust/accord/tests/identity.rs` | 16 | forma canônica do `SemId`, roundtrip, recusas, escapes |
| `rust/accord/tests/property.rs` | 13 | `Value`, `StateKey`, `ActionName`, `StateRef`, `Constraints`, `Policy`, `ActionDecl` |
| `rust/accord/tests/assembly.rs` | 20 | cada `SemError`, agregação, padrões, `inert` herdado, `secret`, `source` |
| `rust/accord/tests/snapshot.rs` | 11 | consultas, `subtree`, `Revision` estável |
| `rust/accord/tests/mutation.rs` | 12 | `Snapshot::diff` e as variantes de `Change` |
| `rust/accord/tests/requests.rs` | 8 | `Request::observe` e `Actor` |
| `rust/accord/tests/decision.rs` | 25 | ordem de validação, delta stale, invalidação só pelas `reads`, parâmetro, confirmação |
| `rust/accord/tests/receipts.rs` | 5 | `Stage`, transições legais, `RequestId` |
| `rust/accord/tests/dispatch.rs` | 24 | dispatcher com um git de memória: cenário da tese, confirmação, execução assíncrona, rejeição no ponto de verdade, transições ilegais, sinks, retenção |
| `rust/accord/tests/encoding.rs` | 9 | JSON fixado de `Request`, `Receipt`, `Snapshot`; forma antiga lendo |
| `rust/accord/src/redactor.rs` | 6 | a regra de redação |
| `rust/accord/src/snapshot.rs` | 2 | a forma serde refaz o snapshot; um laço de parent continua total |

### 6.2 `wire`

| arquivo | testes | cobre |
|---|---|---|
| `rust/socket/tests/codec.rs` | 8 | roundtrip de cada mensagem, envelope, versão errada, linha quebrada, nenhuma linha faz panic |
| `rust/socket/tests/ring.rs` | 3 | acerto, evicção, reposição no anel |
| `rust/socket/tests/server.rs` | 17 | protocolo contra o app de brinquedo, sem socket: hello, escopo e keys, `source`, `since`, act por revisão, stale com delta, `observed` explícito, act malformado, act antes do hello, revisão fora do anel, confirm e decline recusados, estágio tardio, estágio só para quem pediu, segredo, nada projetado |
| `rust/socket/tests/socket.rs` | 10 | socket real em diretório temporário: permissões e remoção, observe, act e stale, linha quebrada, linha acima do limite, cliente que para de mandar, dois agentes, estágio tardio, diretório ruim, `find`, linhas aleatórias |
| `rust/socket/tests/traffic.rs` | 3 | `Host::observed`, contadores do servidor, linhas e bytes da porta |
| `rust/socket/tests/cli.rs` | 2 | o binário `plev` contra o brinquedo: saída e código de saída |
| `rust/socket/tests/env.rs` | 2 | `PLEV_AGENT` lido de verdade num processo filho |
| `rust/socket/src/port.rs`, `rust/socket/src/bin/plev.rs` | 2, 4 | leitura da variável; parse da linha de comando |

### 6.3 `rec`

| arquivo | testes | cobre |
|---|---|---|
| `rust/record/tests/session/thesis.rs` | 3 | a sessão da tese gravada e lida em ordem |
| `rust/record/tests/session/semantic.rs` | 5 | o que vira snapshot inteiro, diff ou nada; cadência; tempo ordenado; origens; lista de changes malformada |
| `rust/record/tests/session/ring.rs` | 5 | limites de tempo e de bytes com o relógio do teste; corte em snapshot inteiro |
| `rust/record/tests/session/secret.rs` | 2 | segredo nunca gravado |
| `rust/record/tests/session/readback.rs` | 1 | ler a sessão não cria request nem executa |
| `rust/record/tests/session/visual.rs` | 8 | trilha visual: frame no tempo, tela entre frames, o que o IR não carrega, assets |
| `rust/record/tests/session/file.rs` | 10 | arquivo `MONS`: persistido igual à memória, export abre em snapshot inteiro, leitor estrito contra truncado, corrompido, estranho e animação, linha antiga de `Event` |
| `apps/uigallery/tests/record_cost.rs` | 2 (1 ignorado) | custo de gravação sobre o todo do showcase |
| `rust/record/tests/panic.rs` | 1 | o panic hook persiste e encadeia o anterior |

### 6.4 `ide` e `git`

| arquivo | testes | cobre |
|---|---|---|
| `apps/idestudio/tests/human_agent.rs` | 9 | repositório real e worker real: commit com sha e arquivos iguais ao git; unstage de B deixa o submit do agente stale; cursor não invalida; index mexido por fora recusado no worker; mesma observação duas vezes commita uma; handoff com recibos; discard de agente confirmado; modal fechado recusa; clique e `Actor::Test` iguais |
| `apps/agenteval/tests/experiment.rs` | 2 | o binário `ide-lab` com o agente roteirizado: com intervenção (stale e recuperação) e sem |
| `apps/idestudio/src/capability/tests.rs` | 14 | modelo projetado; `submit` desabilitado; cursor move a revisão sem stale; ligações iguais à projeção; tecla, digitação e teste chegam ao mesmo form; clique pede diff; clique em commit desabilitado; recusa vira toast; modal de discard; escape e clique fora recusam; leitor de tela recusa, teste não; um pedido de confirmação por vez; sem repositório nada muda |
| `apps/idestudio/src/interfaces/panels/layout/tests/` | 24 | view: overlays (11), painéis (4), rolagem (9) |
| `apps/idestudio/src/capability/ids.rs`, `watcher.rs`, `bin/lab/args.rs`, `bin/lab/repo.rs` | 3, 1, 2, 2 | ids e escapes; debounce; argumentos e repositório do laboratório |
| `rust/gitdrv/tests/real_repo.rs` | 17 | repositórios reais: status, stage, unstage, commit, log, `commit_if` aceito, recusado com delta e em HEAD sem pai, branches, diffs, discard, ignore, cliente fora da thread (`Refresh` emite três eventos) |

### 6.5 engine, comps, showcase

| arquivo | testes | cobre |
|---|---|---|
| `rust/engine/src/drawing/compositor/binding.rs` | 9 | tabela por frame, clip da própria layer, segundo bind ignorado, layer que não existe, `set_binding_view`, bind não suja layer |
| `rust/engine/src/control/accessibility/tree/tests.rs` | 8 | toda role com label, estados, foco e inerte, ordem, bounds físicos dentro do clip, `NodeId` estável entre layouts, árvore vazia sem projeção |
| `rust/engine/src/control/accessibility/action/tests.rs`, `ids.rs`, `focus.rs`, `role.rs`, `bridge.rs` | 8, 6, 5, 2, 2 | tradução de ação, colisão de `NodeId`, ordem de foco, mapa de roles, handlers fora da thread |
| `rust/engine/src/drawing/headless/tests.rs`, `headless/svg/tests.rs` | 12, 14 | viewport, limite de textura, fundo 48, posição por escala, byte a byte igual, resize, png; svg por tipo de nó, relatório, contorno de glifo |
| `rust/engine/src/compose/animation/tests/clock.rs` | 2 | `since_last` real e `dt` travado |
| `rust/engine/src/runtime/perf/tests/stages.rs` | 15 | sondas por estágio desligadas e ligadas |
| `tool/tests/test_repoaudit.rs` | 2 | todo crate do workspace citado em `arc.toml` e no README; todo exemplo do engine em `arc.toml` |
| `rust/engine/src/widgets/tests/matrix.rs` | 5 | todo widget em todo tema em phone e desktop; os 19 projetam e ligam (`:357`); `Select` longo |
| `rust/engine/src/widgets/tests/wake.rs` | 11 | composição de `Wake` e viradas do `Blink` com dt injetado |
| `rust/engine/src/widgets/tests/focus.rs`, `components.rs`, `widgets/*.rs` | 7, 30, 78 | foco dos formulários; componentes; comportamento por widget |
| `apps/uigallery/src/views/apps/todo/tests/semantics.rs` | 10 | modelo do todo; clique e teste iguais; stale pelo texto e não pelo caret; stale da aba; viewport move ligação e mantém id; linha fora da tela projetada e não ligada; página inerte; modal deixa inerte; custo |
| `apps/uigallery/src/views/apps/todo/semantics/assistive.rs` | 7 (1 ignorado) | árvore do todo; clique do leitor igual ao da pessoa; inerte com modal; foco e digitação; ordem de teclado pelo snapshot; página sem projeção; custo |
| `apps/uigallery/src/views/dispatcher/mod.rs` | 3 (1 ignorado) | o todo por uma porta real sem janela; custo em bytes (`agent_port_cost`, ignorado) |
| `apps/uigallery/src/shell/window/headless.rs`, `apps/uigallery/src/main.rs` | 10, 8 | `--render`, `--svg`, `--semantics`, `--source`; parse do cli |

### 6.6 comportamentos do contrato sem teste

- a corrida do `Refresh` (seção 4a): nenhum teste intercala uma mutação com um refresh pendente. o rig espera o refresh de cada passo (`apps/idestudio/tests/support/mod.rs:79-88`) e o roteirizado espera o index parar (`apps/agenteval/src/arms/scripted.rs:95-100`); por isso nenhum dos dois cai nela.
- a janela entre `staged_paths()` e `git commit` (`rust/gitdrv/src/repo.rs:206`, `:213`): sem teste.
- ponto de verdade com os mesmos caminhos staged e conteúdo diferente: sem teste (a comparação é só de caminhos, `rust/gitdrv/src/repo.rs:207`).
- o binário `ide` (`apps/idestudio/src/main.rs`): sem teste; keymap, digitação direta no form e o pedido de frame contínuo não são exercitados.
- `pace` e `new_events` do shell do showcase (`apps/uigallery/src/shell/mod.rs:152-167`, `:204-209`): sem teste automático (`app.rs` não tem `#[test]`); a task-82 conferiu por log numa janela real e o exemplo `measure` modela o shell.
- `Compositor::hit` e `focus::next`: só testes de unidade; como nenhum caminho de produção os chama, não há teste de ponta a ponta.
- a ponte AccessKit contra um adapter e um leitor reais: sem teste (os testes chamam os handlers direto, `rust/engine/src/control/accessibility/bridge.rs:233-313`); o roteiro de VoiceOver da task-60 não rodou.
- frames sem mudança visível (worker do git, ponte, toast do ide, página Git com campo focado): sem teste.
- recibos abertos sem limite no dispatcher: sem teste de crescimento; confirmação que expira: não existe API.
- `install_panic_hook` instalado num app: só o teste isolado (`rust/record/tests/panic.rs`).
- a porta servindo o binário `ide` ou a página Git do showcase: não existe, não há o que testar.
- sondas de `find` contadas em `Traffic.conns`: sem teste.
- urnaui: fora do workspace; entra só na perna do gate que exige `../urna`.

## 7. ownership agora

o que cada task pendente tocaria, pelo arquivo da task e pelo código de hoje. "pelo código" é o que o arquivo da task não lista e o código obriga, porque é onde o mecanismo mora ou quem consome a interface que muda.

| task | pelo arquivo da task | pelo código de hoje | colide com |
|---|---|---|---|
| 82 fase b | `apps/idestudio/src/interfaces/panels/commit_form.rs`, `apps/idestudio/src/main.rs`, `apps/urnadbapp/src/view/field.rs`, `apps/urnadbapp/src/app.rs` (`docs/log/steps/task-82-caret-blink-render-on-demand.md:231-236`) | o `bool` sobe por `apps/idestudio/src/interfaces/panels/layout/mod.rs:312-318` (toast incluído) e `apps/idestudio/src/capability/mod.rs:180-182`; `Workbench::tick` é lido também pela página Git do showcase (`apps/uigallery/src/views/apps/git/desktop/mod.rs:205-211`, `apps/uigallery/src/views/apps/git/stub.rs:54`, `apps/uigallery/src/views/compositor/render.rs:32-33`); a ponte `TextField::tick` mora no comps (`rust/engine/src/widgets/form/text_field.rs:216-218`); o urnaui tem oito `tick -> bool` em `apps/urnadbapp/src/view/` | 75 (shell do ide e do urnaui), 76 (`apps/urnadbapp/src/app.rs`), 74 (comps `form` e `editor`, se a ponte `bool` sair), 69 (view do showcase) |
| 63 fase d | retenção de pixels opt-in em `rust/engine/src/drawing/gpu/image.rs`; atalho de debug e panic hook no shell do showcase (`docs/log/steps/task-63-flight-recorder.md:64-71`, `:97-102`) | o shell é `apps/uigallery/src/shell/mod.rs` (`Access` em `:98-104`, `run` em `:482-496`); o showcase não depende de `rec` (`apps/uigallery/Cargo.toml:19-45`); o rec guarda só o `ImageHandle` de uma imagem (`rust/record/src/visuals/reader.rs:34-38`, `rust/record/src/visuals/mod.rs:39`) e o formato já tem lugar para os pixels (`rust/record/src/payload.rs:9`, `:99-110`), então a retenção toca o lado do rec que resolve o handle; o outro consumidor da retenção é o svg (`rust/engine/src/drawing/headless/svg`) | 65 e 75 (`apps/uigallery/src/shell/mod.rs`), 64 (`rust/record`), 67 (svg) |
| 64 | crate ou página do inspector, harness de sessão, ADR (`docs/log/steps/task-64-session-inspector-and-tests.md:88-92`) | desenhar as ligações sobre o frame exige gravar ligações, que o rec não grava (`rust/record/src/recorder.rs:99-113`): toca `rust/record`; o harness que reaplica requests precisa de fixture de cada app (`Workbench`, `TodoPage`); crate novo passa por `Cargo.toml`, `docs/ATLAS.toml`, `README.md` | 63 fase d (`rust/record`), 65 (link para o código), 67 (sessão como teste contra regressão estrutural) |
| 65 | `rust/engine/src/runtime/hot_reload/**`, `rust/engine/Cargo.toml`, atalho no shell do showcase, `rust/engine/src/styling/theme` depois da fase d (`docs/log/steps/task-65-pixel-to-source.md:94-98`) | `span-locations` é feature do `proc-macro2` do workspace (`Cargo.toml:98`); a fase a é o primeiro uso de produção de `Compositor::hit` (`rust/engine/src/drawing/compositor/binding.rs:184-208`) e só acha nós da página Todo (`apps/uigallery/src/views/dispatcher/mod.rs:242-252`) | 63 fase d e 75 (`apps/uigallery/src/shell/mod.rs`); `Cargo.toml` da raiz (orquestrador) |
| 67 | exemplo de regressão no showcase, fixtures, `.github/workflows`, ADR (`docs/log/steps/task-67-structural-regression.md:69-74`) | o hash de layer é `pub(crate)` (`rust/engine/src/drawing/compositor/layer/mod.rs:44`, `:426-432`): expor ou criar um hash estável toca o compositor; `Approximation.node` tipado toca `rust/engine/src/drawing/headless/svg/mod.rs:41-45`; `--semantics` só existe para o Todo (`apps/uigallery/src/shell/window/headless.rs:133`) | 73 fase 3 e 79 (compositor), 63 fase d (svg), 64 |
| 68 | crate novo; `Cargo.toml`, `docs/ATLAS.toml`, `README.md` (`docs/log/steps/task-68-plev-ir-and-verifier.md:149-152`) | depende de `sem`; mudança em `sem` é do orquestrador (`docs/log/generative-runtime-program.md:210`) | pouca; `sem` se o registro de capabilities pedir tipo novo |
| 69 | host de superfícies, página nova em `apps/uigallery/src/views/apps/`, proponente, `refs/` (`docs/log/steps/task-69-ephemeral-generative-surfaces.md:121-127`) | página nova entra no despacho por página (`apps/uigallery/src/views/dispatcher/mod.rs`, um `match` por verbo) e em `apps/uigallery/src/views/pages`; `semantics`, `act` e `add_sink` só conhecem o Todo (`apps/uigallery/src/views/dispatcher/mod.rs:242-276`) | 74 e 75 (comps), 73 (layout), 82 fase b (view do showcase) |
| 73 fase 2 | ADR de desenho (`docs/log/steps/task-73-incremental-dependency-system.md:165-171`); fases seguintes em `layout`, `compositor`, `text/measure` e, desde a fase 1, a GPU (`:65-69`) | composite de toda layer visível (`rust/engine/src/runtime/window/render_passes.rs:592-609`), `StoreOp::Store` no attachment (`:137-148`), `draw_frame` compartilhado por janela e headless (`rust/engine/src/runtime/window/frame.rs:107-208`) | 74 (text), 67 e 79 (compositor), 76 e 77 (frame e pixels) |
| 74 | `rust/engine/src/drawing/text/**`, `rust/engine/src/runtime/platform/ime.rs`, `rust/editor/**`, `rust/engine/src/widgets/editor/**`, `rust/engine/src/widgets/form/text_input*`, fontes (`docs/log/steps/task-74-text-and-editing.md:83-88`) | `EditorView::wake` e `TextInput::wake` moram aí (`rust/engine/src/widgets/editor/view.rs:184-193`, `rust/engine/src/widgets/form/text_input/component.rs:64-69`); o svg depende de `ensure_shaped` e `shaped_glyphs` (`rust/engine/src/drawing/text/system.rs:247`, `:288`) | 82 fase b, 75 (comps), 77 (fontes), 73 (`text/measure`) |
| 75 | `rust/engine/src/control/accessibility/**`, shells dos apps na parte da ponte, comps em teclado e foco, testes de contraste (`docs/log/steps/task-75-platform-accessibility.md:70-74`) | shells: `apps/uigallery/src/shell/mod.rs:499-540`, `apps/idestudio/src/main.rs` (sem ponte hoje), `apps/urnadbapp/src/app.rs`; Tab por `focus::next` exige ligar o Tab de cada página (`apps/uigallery/src/keyboard.rs:18`) | 63 fase d e 65 (shell do showcase), 82 fase b (shell do ide e do urnaui), 74 (comps), 76 (window) |
| 76 | `rust/engine/src/runtime/platform/**`, `rust/engine/src/runtime/window` (várias janelas, device lost), urnaui, clipboard (`docs/log/steps/task-76-platform-integration-done.md:83-87`) | device lost passa por `present_frame` (`rust/engine/src/runtime/window/frame.rs:53-68`), no mesmo arquivo que o headless usa | 82 fase b e 78 (urnaui), 75 (window e shells), 73 (frame) |
| 77 | harness de captura, `script/`, shaders com aprovação (`docs/log/steps/task-77-visual-determinism.md:73-77`); cita `rust/engine/src/drawing/text/fonts.rs` | a base é o headless (`rust/engine/src/drawing/headless/mod.rs:129-222`) | 74 (fontes), 73 (passes) |
| 78 | app de referência, módulo de inferência, urnaui se escolhido (`docs/log/steps/task-78-ai-native-apps.md:70-73`) | | 76 e 82 fase b, se for o urnaui |
| 79 | canal de cena no protocolo (com aprovação), visualizador, serde de `SceneNode` (`docs/log/steps/task-79-scene-over-socket.md:62-66`) | protocolo v1 em `rust/socket/src/protocol.rs:21`, `:40-46`; `SceneNode` sem serde (`rust/engine/src/drawing/compositor/scene.rs`) | 73 e 67 (compositor), o delta do wire (seção 4b) |

itens do checkpoint a sem número de task, pelo código:

| item | onde mora | colide com |
|---|---|---|
| nó inteiro em `node_added` | `rust/accord/src/mutation.rs:23` (dono: orquestrador); consumidores do `Change` em `rust/socket/src/server.rs:329-340` e `rust/record/src/event.rs:39-43` | 79 (wire), 64 e 63 fase d (rec) |
| arquivos no nó `stacks.commit[sha]` | `apps/idestudio/src/capability/project.rs:154-176`, `rust/gitdrv/src/domain.rs:28-39`, `rust/gitdrv/src/repo.rs:229-236` | 82 fase b (`apps/idestudio`) |
| ator na recusa stale | `rust/accord/src/decision.rs:46` ou `apps/idestudio/src/capability/notice.rs:79-108` | dono do `sem`; 82 fase b se for no ide |
| corrida do `Refresh` | `apps/idestudio/src/repository/git.rs:89-149`, `apps/idestudio/src/capability/execute.rs:85-100`, `rust/gitdrv/src/client.rs:121-131` | 82 fase b (`apps/idestudio`) |

superfícies com mais de um pretendente:

- `apps/uigallery/src/shell/mod.rs` (o shell): 63 fase d, 65, 75. já foi tocado por 60, 61, 66b e 82; a seção 8 do programa não tem linha para ele.
- `apps/idestudio/src/main.rs` e as views do ide: 82 fase b, 75 e os quatro itens do checkpoint a.
- `apps/urnadbapp/src/app.rs`: 82 fase b, 75, 76, 78.
- `rust/engine/src/widgets/form/text_input*`, `rust/engine/src/widgets/form/text_field.rs`, `rust/engine/src/widgets/editor/**`: 74, 75, 82 fase b.
- `rust/engine/src/drawing/compositor`: 67, 73, 79.
- `rust/record`: 63 fase d, 64, `node_added`.
- `rust/engine/src/runtime/window/frame.rs` (o caminho único de frame): 73, 76, 77.
- `rust/accord`: dono é o orquestrador; 68 e dois itens do checkpoint a.

## 8. não confirmado

- quando o `accesskit_winit` considera o leitor ativo no macOS (a task-60 diz que ele nunca informa desativação e a árvore passa a ser montada a cada frame): o plev pede `update_if_active` todo frame (`rust/engine/src/control/accessibility/bridge.rs:119-143`); a decisão é da biblioteca, que não foi lida.
- VoiceOver: `docs/log/steps/task-75-platform-accessibility.md:33` diz "verificado com VoiceOver no macOS"; `docs/log/steps/task-60-accessibility-projection.md:84-85` diz que não rodou. o código não guarda evidência de nenhum dos dois.
- urnaui: não compila sem `../urna`. foi lido, não executado.
- números medidos nos arquivos de task e nos ADRs (bytes, microssegundos, tokens, frames por segundo): não refeitos, porque nada foi rodado.
- a frequência da corrida da seção 4a no uso humano do binário `ide`: o mecanismo está no código; quanto ela aparece depende do tempo entre dois cliques e a resposta do worker, e não foi medido.
- duas mutações do mesmo tipo e do mesmo caminho em voo ao mesmo tempo: o código casa a resposta com a mais antiga (`apps/idestudio/src/capability/truth.rs:73-77`); nenhum teste cobre esse caso, e o efeito de a primeira falhar não foi conferido.
- as duas conexões por comando `plev` por nome (seção 4f): saem da leitura de `find` e `connect`; não foram observadas num `Traffic` real.
- "v1" do container monster no README e no programa: o código só tem a versão 0 (`rust/motion/src/encoding/container.rs:51`); se o texto nomeia a spec, isso não está escrito em lugar nenhum do código.
- contagens de teste da seção 6: contadas por `#[test]` no texto; proptests entram, doc tests e o gate não foram contados.
- itens do relatório de aproximação do svg que dependem de quem desenha o svg (costura de stroke, `linearRGB` no Chrome): comportamento de renderizador externo.
