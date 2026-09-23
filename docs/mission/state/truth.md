---
type: truth
base: main @ c5fdf36
date: 2026-09-23
audience: [orchestrator, ai-agents]
tags: [truth, audit, sem, wire, rec, ide, git, engine, comps, showcase]
---

# o que o código da main faz agora

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
| `SemId` | `crates/sem/src/id.rs:28-29` | o app, por texto ou por `name`, `child`, `key` (`crates/sem/src/id.rs:33-74`) | comps, engine, ide, rec, showcase, wire | texto canônico validado (`crates/sem/src/id.rs:118-161`); `Eq`, `Ord` e `Hash` seguem o texto; clone é refcount (`Arc<str>`) |
| `Role` | `crates/sem/src/role.rs:9-43` | projeção do app | comps, engine (mapa AccessKit), ide, rec, showcase | enum fechado de 33 (`crates/sem/src/role.rs:47`); role interativa exige label (`crates/sem/src/role.rs:87-103`, `crates/sem/src/validate.rs:55-57`) |
| `Value`, `ValueKind` | `crates/sem/src/value.rs:24-31`, `:109-116` | projeção | engine, ide, rec, showcase, wire | igualdade exata; `Num` por bits (`crates/sem/src/value.rs:47-59`) |
| `StateRef` | `crates/sem/src/value.rs:138-161` | projeção (`reads`) | ide, rec, showcase | forma `<semid>:<key>`, separada no último `:` (`crates/sem/src/value.rs:149-160`) |
| `Constraints`, `Actors`, `Confirm`, `Policy`, `ParamSpec`, `ActionDecl` | `crates/sem/src/decl.rs:23-53`, `:104-120`, `:128-133`, `:150-159`, `:166-169`, `:177-201` | projeção | comps, ide, rec, showcase; engine lê `ActionDecl` na accessibility | `faults` recusa range, step e options inválidos (`crates/sem/src/decl.rs:57-79`); padrão de `Actors` é todo mundo (`crates/sem/src/decl.rs:111-119`); `editable` é falso por padrão (`crates/sem/src/decl.rs:24-25`) |
| `Node`, `Source` | `crates/sem/src/node.rs:35-75`, `:101-105` | `SnapshotBuilder::node` (`crates/sem/src/builder.rs:56-93`) | engine, ide, rec, showcase, wire | igualdade ignora `source` (`crates/sem/src/node.rs:77-91`); `source` vem de `#[track_caller]` e aponta o site de projeção, não o render (`crates/sem/src/builder.rs:56`, `:79-83`) |
| `Semantics`, `SnapshotBuilder`, `NodeBuilder`, `ActionBuilder` | `crates/sem/src/builder.rs:26-33`, `:38-42`, `:116-119`, `:227-230` | o app implementa `Semantics` | implementado fora de teste só por `TodoPage` (`crates/showcase/src/view/apps/app/semantics.rs:146`) e `Workbench` (`crates/ide/src/workbench/project.rs:39`); `NodeBuilder` devolvido pelos 19 helpers do comps | texto inválido vira erro agregado, nunca panic (`crates/sem/src/builder.rs:5-8`); `finish` devolve todas as violações (`crates/sem/src/validate.rs:16-44`) |
| `Snapshot` | `crates/sem/src/snapshot.rs:28-35` | `finish` (`crates/sem/src/validate.rs:41-43`) | engine, ide, rec, showcase, wire | valor imutável; `inert` herdado (`crates/sem/src/validate.rs:141-171`); valor `secret` vira tamanho (`crates/sem/src/validate.rs:176-188`); `subtree` para escopo (`crates/sem/src/snapshot.rs:135-159`) |
| `Revision` | `crates/sem/src/hash.rs:57-58` | `Snapshot::revision` (`crates/sem/src/snapshot.rs:100-104`) | ide, rec, showcase, wire | FNV-1a 64 escrito à mão sobre a codificação `sem.revision.v1` (`crates/sem/src/hash.rs:10-33`, `:51`); ignora `source`, ordem das chaves de estado e intercalação de subárvores; trafega como 16 hex minúsculos (`crates/sem/src/hash.rs:66-91`) |
| `Change` (12 variantes) | `crates/sem/src/diff.rs:22-83` | `Snapshot::diff` (`crates/sem/src/diff.rs:91-125`) e o delta stale de `check` (`crates/sem/src/diff.rs:199-216`) | ide, rec, showcase, wire | vazio exatamente quando os snapshots são iguais (`crates/sem/src/diff.rs:16-18`); `NodeAdded` carrega só o id (`crates/sem/src/diff.rs:23`) |
| `SemError`, `SemErrors` | `crates/sem/src/error.rs:18-77`, `:92` | parse, `finish`, dispatcher | engine (accessibility), ide, rec, showcase | 14 variantes; erro de uso da API, nunca decisão de negócio (`crates/sem/src/error.rs:1-4`) |
| forma serde | atrás da feature `serde`, padrão ligado (`crates/sem/Cargo.toml:12-16`); `SnapshotWire` privado (`crates/sem/src/snapshot.rs:192-219`) | serde | wire, rec, showcase (`--semantics`) | campo no valor padrão não é escrito e lê de volta como padrão (`crates/sem/src/decl.rs:5-9`, `crates/sem/src/node.rs:27-32`); forma fixada em `crates/sem/tests/wire.rs` |

### 2.2 `sem`: ação, receipt, dispatcher e redação

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `Actor` | `crates/sem/src/request.rs:23-28` | quem pede | engine, ide, rec, showcase, wire | quatro tipos; nenhum representa replay (`crates/sem/src/request.rs:18-19`) |
| `Observation`, `Request`, `Request::observe` | `crates/sem/src/request.rs:52-55`, `:62-72`, `:90-131` | ator, a partir do snapshot que viu | engine (leitor de tela), ide, showcase, wire; `Observation` direto só no wire | guarda o valor de cada `read` da ação; a revisão é só rastreio (`crates/sem/src/request.rs:42-49`) |
| `Verdict`, `Rejection` (9 variantes), `check` | `crates/sem/src/check.rs:17-26`, `:34-62`, `:99-138` | `check`, chamado só pelo dispatcher (`crates/sem/src/dispatch.rs:330`) | `Rejection`: ide, rec, showcase, wire. `Verdict` e `check`: nenhum consumidor fora do `sem` | pura; ordem fixa (`crates/sem/src/check.rs:67-80`); stale compara só as `reads` da ação (`crates/sem/src/check.rs:166-181`); `Stale` não diz qual ator mudou o estado (`crates/sem/src/check.rs:46`) |
| `RequestId`, `Millis` | `crates/sem/src/receipt.rs:29`, `:20` | dispatcher numera a partir de 1 | ide, rec, showcase, wire | `sem` nunca lê relógio; o tempo é de quem chama (`crates/sem/src/receipt.rs:16-20`) |
| `Stage`, `Outcome`, `Receipt` | `crates/sem/src/receipt.rs:87-96`, `:71-73`, `:147-152` | dispatcher | ide, rec, showcase, wire | transições legais em `can_advance_to` (`crates/sem/src/receipt.rs:122-134`) |
| `Operable`, `Execution`, `ReceiptSink` | `crates/sem/src/dispatch.rs:20-28`, `:34-45`, `:50-52` | app (`Operable`), consumidor (`ReceiptSink`) | `Operable`: `TodoPage` (`crates/showcase/src/view/apps/app/semantics.rs:188`) e `Workbench` (`crates/ide/src/workbench/execute.rs:157`); sinks: `wire` (`crates/wire/src/server.rs:133-149`), `rec` (`crates/rec/src/recorder.rs:175-179`), lab (`crates/ide/src/bin/lab/host.rs:146-154`) | `execute` recebe o `RequestId` e pode terminar depois (`Started`) |
| `Dispatcher` | `crates/sem/src/dispatch.rs:65-71` | app | ide (`crates/ide/src/workbench/mod.rs:47`), showcase todo | projeta o app no instante e checa contra ele (`crates/sem/src/dispatch.rs:125-142`, `:324-337`); `execute` só depois de aceito ou confirmado (`:131-134`, `:166-170`); `confirm` e `decline` só por `Human` ou `Assistive` (`:275-283`); recibos terminados limitados a 256 (`:55`, `:310-319`); recibos abertos sem limite e sem expiração (`:60-64`); não é `Send` (sinks `Box<dyn ReceiptSink>`, `:70`) |
| `sem::redact` (`REDACTED`, `is_secret`, `length`, `Secrets`) | `crates/sem/src/redact.rs:26`, `:29-33`, `:37-45`, `:53-137` | quem leva request ou estágio para fora do processo | wire (`crates/wire/src/server.rs:403-404`, `:250`), rec (`crates/rec/src/ring.rs:236`) | falha fechado: nó desconhecido conta como secreto (`crates/sem/src/redact.rs:28-33`); única cópia da regra |
| exemplo do cenário da tese | `crates/sem/examples/commit.rs` | | | roda com `cargo run -p sem --example commit` |

### 2.3 `wire`: protocolo v1, porta, cli

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| mensagens v1: `Ask` (hello, observe, act, confirm, decline), `Reply` (welcome, snapshot, delta, stage, error), `ProtoError` (4) | `crates/wire/src/proto.rs:40-46`, `:140-146`, `:210-230`; `VERSION` em `:21` | cliente e `Server` | plev, lab, testes | JSON lines, `"v":1`, `"id"` do cliente; versão conferida antes da forma (`crates/wire/src/codec.rs:101-120`); `SnapshotReply.focused` sempre escrito, `null` sem foco (`crates/wire/src/proto.rs:160`) |
| `Observe {scope, keys, since, bounds, source}` | `crates/wire/src/proto.rs:64-75` | cliente | `Server::observe` (`crates/wire/src/server.rs:299-363`) | `revision` é sempre a do snapshot inteiro (`crates/wire/src/proto.rs:151-155`); `since` fora do anel devolve snapshot inteiro (`crates/wire/src/server.rs:310-314`); `keys` filtra só `StateChanged` no delta (`crates/wire/src/server.rs:333-337`); `source` zerado a não ser que pedido (`crates/wire/src/server.rs:349-351`) |
| `Act {target, action, param, observed or from_revision}` | `crates/wire/src/proto.rs:82-91` | cliente | `Server::act` (`crates/wire/src/server.rs:365-439`) | exatamente um dos dois (`:377-402`); `from_revision` remonta o `observed` pelo `Request::observe` sobre o snapshot guardado (`:385-391`); ator sempre `Agent` com o nome do hello (`:271-275`) |
| `Ring` | `crates/wire/src/ring.rs:17-20`, `DEFAULT_RING = 16` em `:10` | `Server` a cada observe | `Server` | guarda os últimos N snapshots inteiros servidos; servir de novo reposiciona (`crates/wire/src/ring.rs:52-62`) |
| `Host` (project, bounds, dispatch, add_sink, observed) | `crates/wire/src/server.rs:41-71` | app | showcase (`crates/showcase/src/view/dispatch.rs:347-371`), lab (`crates/ide/src/bin/lab/host.rs:232-263`), toy de teste | o servidor só alcança o app por aqui; `observed` tem implementação vazia padrão (`crates/wire/src/server.rs:68-70`); só o lab implementa `observed` |
| `Traffic` | `crates/wire/src/server.rs:79-91` | server e threads da porta (`crates/wire/src/port.rs:249-259`) | lab (`crates/ide/src/bin/lab/report.rs`) | contadores aditivos; `conns` conta toda conexão aceita (`crates/wire/src/port.rs:373-374`) |
| `Server` | `crates/wire/src/server.rs:103-112` | porta ou teste | porta | encaminha só estágios de requests de agente que esta porta emitiu (`crates/wire/src/server.rs:137-139`, `:226-261`) para a conexão que pediu |
| `AgentPort`, `Config`, `enabled` | `crates/wire/src/port.rs:101-107`, `:69-76`, `:90-92` | shell do app | showcase (`crates/showcase/src/view/dispatch.rs:322-327`), lab (`crates/ide/src/bin/lab/lab.rs:114`) | desligada sem `PLEV_AGENT=1` em debug (`crates/wire/src/port.rs:90-92`, `:171-179`); socket 0600 em diretório 0700 (`:200`, `crates/wire/src/place.rs:42-67`); fila 256, outbox 1024, 64 mensagens por `drain`, linha até 1 MiB, 16 conexões, 10 s de escrita parada (`crates/wire/src/port.rs:42-62`); `drain` roda no thread do app (`:267-299`) |
| `Client` | `crates/wire/src/client.rs:17-22` | agente | plev, lab (`crates/ide/src/bin/lab/scripted.rs`), testes | uma conexão por instância (`crates/wire/src/client.rs:25-37`) |
| binário `plev` | `crates/wire/src/bin/plev.rs:37-53` | | agente | uma conexão nova por invocação (`crates/wire/src/bin/plev.rs:192-199`, `:225-240`); sem `--from-revision`, observa o alvo antes (`:262-272`); saída 0 observado ou concluído, 1 recusado, 2 falhou, 3 erro (`:56-59`, `:298-305`) |

### 2.4 `rec`: gravador, sessão, formato `MONS`

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `Recorder` (`sink`, `observed`, `snapshot`, `snapshot_full`, `input`, `frame`, `frame_scene`, `persist`, `export`, `stats`) | `crates/rec/src/recorder.rs:28-158` | app | só o `ide-lab` (`crates/ide/src/bin/lab/lab.rs:11`, `:109`; `crates/ide/src/bin/lab/host.rs:214-215`, `:252-254`); nenhum app que shipa | nunca lê relógio nem cria request (`crates/rec/src/recorder.rs:23-26`); `frame` lê só as layers visíveis, não as ligações (`:99-113`) |
| `Sink` | `crates/rec/src/recorder.rs:163-179` | `Recorder::sink` | lab | origem `u16` por dispatcher (`:58-64`, `:165`); redige cada estágio contra o último snapshot (`crates/rec/src/ring.rs:228-236`) |
| `Event`, `Kind`, `InputKind` | `crates/rec/src/event.rs:15-18`, `:23-63`, `:83-90` | recorder | `Session`, lab | `Changes` nunca guarda nó adicionado, parent mudado ou reordenação: esses viram snapshot inteiro (`crates/rec/src/event.rs:39-43`); input sem conteúdo (`:78-80`) |
| `Limits`, `Stats` | `crates/rec/src/limits.rs:11-47`, `:52-104` | app | lab, testes | padrão 20 s, 32 MiB, keyframe a cada 2 s nas duas trilhas, 120 frames por chunk (`crates/rec/src/limits.rs:35-46`); corte só em keyframe |
| arquivo de sessão | `crates/rec/src/format.rs:6-43` | `Recorder::persist`, `export` (`crates/rec/src/recorder.rs:139-153`) | `Session::open`, `from_bytes` (`crates/rec/src/persist.rs:146`, `:154`) | magic `MONS`, versão 0, flags 0, seções `S` `I` `C` `A` com sha256 (`crates/rec/src/format.rs:33-43`); `probe` separa `MON0` de `MONS` (`:57-63`); leitor estrito com erro tipado (`:236-305`); sha do git no cabeçalho por `build.rs` (`crates/rec/build.rs:24-25`) |
| `Session` | `crates/rec/src/session.rs` (`scene_at` `:114`, `state_at` `:190`, `receipts_for` `:247`, `excerpt` `:103`) | leitura | lab (`crates/ide/src/bin/lab/lab.rs`), `crates/ide/tests/lab.rs` | não guarda dispatcher nem app: ler nunca cria request |
| `install_panic_hook` | `crates/rec/src/hook.rs:27-36` | | nenhum chamador fora de `crates/rec/tests/panic.rs` | nunca faz panic; só tenta o lock (`crates/rec/src/hook.rs:46-57`) |

### 2.5 engine: ligação, accessibility, headless, frame, relógio

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `Binding { rect, layer, clip, order, view }` | `crates/engine/src/compositor/binding.rs:27-42` | `bind`, `bind_to_layer` (`:119-153`), chamados por 19 widgets do comps e pelas views do ide e do todo (`crates/ide/src/views/unassigned_view.rs:333`, `crates/showcase/src/view/apps/app/draw.rs:29`, `:73`, `:171`) | accessibility (`crates/engine/src/accessibility/tree.rs:161`), `Host::bounds` do showcase (`crates/showcase/src/view/dispatch.rs:353-362`) | zerada em `begin_frame` (`crates/engine/src/compositor/mod.rs:76-81`); fora do hash de layer; segundo bind do mesmo id é ignorado com log (`binding.rs:134-139`); `view` fica `None`: nenhum código de produção chama `set_binding_view` (`binding.rs:158-166`) |
| `Compositor::hit(x, y)` | `crates/engine/src/compositor/binding.rs:184-208` | | nenhum caminho de produção; só testes, bench e o exemplo `measure` | topo por z-order, criação da layer e ordem de bind; não lê `Layer::clip_rect` físico |
| `ViewId`, `HitRegion`, `InputState` | `crates/engine/src/input/types.rs:133-146`, `crates/engine/src/input/mod.rs:17-61` | `InputState` | `engine::window::App` (`crates/engine/src/window/mod.rs:53`), builder, exemplos; nenhum app que shipa | contador que volta a zero por frame (`crates/engine/src/input/mod.rs:50-61`) |
| projeção AccessKit: `tree::build`, `NodeIds`, `role::of` | `crates/engine/src/accessibility/tree.rs:73-121`, `crates/engine/src/accessibility/ids.rs:30-39`, `:59`, `crates/engine/src/accessibility/role.rs:17` | `Bridge::update` | shell do showcase, `engine::window::App` sem projeção (`crates/engine/src/window/render.rs:104-113`) | nó sem ligação fica sem bounds; inerte fica escondido; secreto sem valor (`tree.rs:17-27`, `:198-244`); `NodeId` FNV-1a 64 do texto do `SemId` |
| ações do leitor: `action::translate`, `primary`, `Assist` | `crates/engine/src/accessibility/action.rs:99`, `:90`, `:40`; ordem da ação primária em `:36` | `Bridge::take` (`crates/engine/src/accessibility/bridge.rs:147-149`) | `Access::carry_out` do showcase (`crates/showcase/src/app.rs:518-540`) | request observado sobre o snapshot que o leitor ouviu, com `Actor::Assistive`; `Focus` não é capacidade |
| `focus::next` | `crates/engine/src/accessibility/focus.rs:48` | | nenhum caminho de produção; só `crates/showcase/src/view/apps/app/semantics/assistive.rs:273`, módulo `#[cfg(test)]` (`crates/showcase/src/view/apps/app/semantics.rs:340-342`) | |
| `Bridge` | `crates/engine/src/accessibility/bridge.rs:73-150` | shell | showcase (`crates/showcase/src/app.rs:499-513`), `engine::window::App` (`crates/engine/src/window/lifecycle.rs:58`) | recusa janela já visível (`bridge.rs:91-93`); handlers só enfileiram e acordam (`:198-209`); projeta só com cliente ativo (`:119-143`) |
| `window::draw_frame`, `present_frame` | `crates/engine/src/window/frame.rs:107-208`, `:37-83` | | ide (`crates/ide/src/renderer.rs:30`), showcase, headless | um caminho para janela e textura; compõe sem efeitos de layer (`&[]`, `frame.rs:191`); `engine::window::App::render` tem cauda própria (`crates/engine/src/window/render.rs:5-10`) |
| headless: `Viewport`, `Renderer`, `HeadlessError`, `Frame`, `read_texture` | `crates/engine/src/headless/mod.rs:36-67`, `:129-222`, `:69-85` | | showcase `--render` (`crates/showcase/src/app/headless.rs`), lab (`crates/ide/src/bin/lab/screenshot.rs`, `bench.rs`), exemplos `text_probe`, `monster_player`, `measure`, teste de custo do rec | `None` sem adapter vira `NoAdapter` e teste pula (`mod.rs:141-146`); fora de wasm (`crates/engine/src/lib.rs:48-49`) |
| `headless::svg::export`, `SvgExport`, `Approximation` | `crates/engine/src/headless/svg/mod.rs:77`, `:55-58`, `:41-45` | `Renderer::svg` (`crates/engine/src/headless/mod.rs:205-211`) | `showcase --render --svg` | texto como contorno dos glifos de `TextSystem::shaped_glyphs` (`crates/engine/src/text/system.rs:288`), o mesmo `ensure_shaped` do desenho (`:247`); toda aproximação vira item; `Approximation.node` é texto |
| `FrameClock`, `AnimationTick { dt, elapsed, since_last }` | `crates/engine/src/animation/mod.rs:13-61` | shell | showcase lê `since_last` (`crates/showcase/src/app.rs:435-438`); ide lê só `dt` (`crates/ide/src/main.rs:399-400`) | `dt` travado em 0,1 s; `since_last` real, 0 no primeiro tick (`animation/mod.rs:48-60`) |
| `perf::stages`, `RenderConfig::gpu_timestamps` | `crates/engine/src/perf/stages.rs:255`, `crates/engine/src/gpu/config.rs:36` | | exemplo `crates/showcase/examples/measure` | desligados por padrão |

### 2.6 comps: `Wake`, `Blink`, declaração semântica

| contrato | onde | quem produz | quem consome de fato | garantia que o código dá |
|---|---|---|---|---|
| `Wake { now, next }` | `crates/comps/src/wake.rs:22-31`; `merge` `:79-89`; `moving` `:69-75` | ticks de widget e página | só o shell do showcase (`crates/showcase/src/app.rs:152-167`, `:438`) | `Some(ZERO)` é contínuo, `None` é nada pendente; `merge` pega qualquer `now` e o menor `next` |
| `Blink` | `crates/comps/src/wake.rs:107-175` | | `TextInput` (`crates/comps/src/form/text_input/component.rs:64-69`), `EditorView` (`crates/comps/src/editor/view.rs:184-186`) | conta em f64; `dt` ou intervalo que não é tempo é ignorado (`wake.rs:148-174`) |
| pontes `bool` que sobram | `TextField::tick` (`crates/comps/src/form/text_field.rs:216-218`, `pending()`: verdadeiro em todo frame com foco), `EditorView::tick` (`crates/comps/src/editor/view.rs:191-193`) | | ide (`crates/ide/src/views/commit_form.rs:135-136`), urnaui (`crates/urnaui/src/view/field.rs:72-74`), exemplo do editor | |
| `.sem(id)` e `project` | 19 widgets: `Button`, `IconButton`, `SubmitButton`, `Chip`, `TextField`, `Checkbox`, `Switch`, `Slider`, `Select`, `Tabs`, `NavLink`, `Breadcrumb`, `SplitPane`, `VirtualList`, `Table`, `Tree`, `ContextMenu`, `Modal`, `Toast` (ex.: `crates/comps/src/form/text_field.rs:61`, `:80-104`; `crates/comps/src/content/list.rs:84`, `:105-126`) | app | fora de teste, só `TextField::project` (todo e ide: `crates/showcase/src/view/apps/app/semantics.rs:152-153`, `crates/ide/src/workbench/project.rs:138-139`) e `ToastManager::project` (ide: `crates/ide/src/workbench/project.rs:49`); a matriz projeta os 19 (`crates/comps/src/tests/matrix.rs:357`) | helper só lê o widget; `set_value` lê o texto que substitui (`text_field.rs:97-102`); list, table e tree usam o índice como chave (`crates/comps/src/content/list.rs:115`, `crates/comps/src/content/table.rs:136`) |
| widgets com evento e sem `.sem` | `Scrollbar` (`crates/comps/src/feedback/scrollbar.rs:125`), `Card` (`crates/comps/src/content/card.rs:245`), `GraphView` (`crates/comps/src/graph.rs:180`), `AppShell` (`crates/comps/src/shell.rs:124`), `EmptyState` (`crates/comps/src/content/empty_state.rs:109`), `Sidebar` (`crates/comps/src/nav/sidebar/input.rs:14`), `EditorView` (`crates/comps/src/editor/input.rs:43`) | | | nenhuma identidade nem projeção |

### 2.7 ide: `Workbench`, nós projetados, `CommitIf`

| contrato | onde | garantia que o código dá |
|---|---|---|
| `Workbench` (`snapshot`, `dispatch`, `confirm`, `decline`, `receipt`, `add_sink`, `apply`, `tick`, `render`, `now`) | `crates/ide/src/workbench/mod.rs:41-57`, `:104-108`, `:114-124`, `:130-141`, `:148-156`, `:159-166`, `:172-177`, `:180-182`, `:186-188`, `:318-320` | um dispatcher para todo ator; o relógio dos recibos é `now` (`Instant` desde a criação) |
| entrada da pessoa | `crates/ide/src/workbench/mod.rs:194-294` | a view roda primeiro e enfileira `UiRequest`; o workbench observa naquele instante e despacha como `Actor::Human` (`:246-294`); o alvo vem de `aim` (`:297-312`) |
| `Semantics` | `crates/ide/src/workbench/project.rs:39-54` | projeta a partir das views, não do git |
| `Operable` | `crates/ide/src/workbench/execute.rs:157-219` | toda mutação sai para o worker como `Execution::Started` (`:85-100`); o otimismo da view acontece aqui (`:186-209`) |
| fim no ponto de verdade | `crates/ide/src/workbench/truth.rs:39-71` | resposta casada com o request mais antigo do mesmo tipo e caminho (`:14-18`, `:73-77`); `CommitRejected` vira `Rejection::Stale` com um `StateChanged` em `changes:staged` (`:127-147`); `Committed` vira `Outcome { commit, files }` (`:103-122`) |
| `GitCommand::CommitIf` | `crates/git/src/client.rs:48-51`; executado em `crates/git/src/client.rs:190-199` | o worker lê `status` e compara o conjunto staged com o esperado antes de `git commit` (`crates/git/src/repo.rs:205-214`); não usa `sem::check` (`crates/git/Cargo.toml` não depende de `sem`) |
| `GitEvent::Committed(NewCommit { sha, files })`, `CommitRejected { added, removed }` | `crates/git/src/client.rs:93-100`, `crates/git/src/types.rs:73-93` | `files` de `git show --name-only` (`crates/git/src/repo.rs:229-236`) |
| worker | `crates/git/src/client.rs:121-131` | um thread, FIFO de um canal; `Refresh` emite `Status`, `Log`, `Branches` em sequência (`:200-204`) |
| toast e modal | `crates/ide/src/workbench/notice.rs:24-108`, `crates/ide/src/workbench/person.rs:16-70` | recusa vira toast; request `Confirm::ForAgents` abre modal; fechar sem confirmar chama `decline` como `Actor::Human` (`person.rs:66-70`) |

nós projetados (`crates/ide/src/workbench/project.rs`):

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
| `notices` e `notices[<id>]` | `Region`; filhos `Alert` ou `Status` (`crates/comps/src/feedback/toast.rs:154-176`) | nenhum | `dismiss`, sem reads |

foco: `commit.message` quando o form está visível e o campo focado (`project.rs:50-52`). ids: `crates/ide/src/ids.rs:9-46`.

o `ide-lab`: binário `[[bin]] ide-lab` em `crates/ide/src/bin/lab/` (`crates/ide/Cargo.toml:17-19`), só unix (`crates/ide/src/bin/lab/main.rs:18-65`). monta um repositório real, sobe o `Workbench` sem janela, abre `AgentPort::open_with` (`crates/ide/src/bin/lab/lab.rs:114`), liga o `Recorder` no relógio do workbench, roda o humano roteirizado (`crates/ide/src/bin/lab/host.rs:192-229`) e confere a sessão com o git. braços `plev`, `scripted`, `git`, `screenshot` (`crates/ide/src/bin/lab/main.rs:53-57`). é o único lugar do repositório que junta `Workbench`, `wire` e `rec`.

### 2.8 showcase: porta do agente, ponte, `pace`

| contrato | onde | garantia que o código dá |
|---|---|---|
| `ShowcaseView::semantics`, `act`, `add_sink`, `assist_focus` | `crates/showcase/src/view/dispatch.rs:242-252`, `:259-269`, `:274-276`, `:280-289` | só a página Todo projeta e despacha; `add_sink` põe o sink sempre no dispatcher do todo; a página Git (o `Workbench` embutido) não é alcançável por agente nem por leitor de tela |
| porta do agente | `crates/showcase/src/view/dispatch.rs:322-327`, `:332-337`; `Host` em `:347-371`; aberta em `crates/showcase/src/app.rs:486-491` | só desktop e `PLEV_AGENT=1` em debug; `UserEvent::Agent` serve a porta e desenha só se um act rodou (`crates/showcase/src/app.rs:187-196`, `:295`) |
| ponte de accessibility | `crates/showcase/src/app.rs:219-240`, `:453-461`, `:499-540` | janela criada invisível, ponte, depois visível; a árvore sai do frame recém desenhado |
| `pace` | `crates/showcase/src/app.rs:152-167`; `new_events` em `:204-209`; `wake(tick.since_last)` em `:435-438` | `Some(ZERO)` pede o próximo frame, `Some(d)` põe `WaitUntil(agora + d)`, `None` põe `Wait`; `ResumeTimeReached` volta a `Wait` e invalida |
| `TodoPage` | `crates/showcase/src/view/apps/app/semantics.rs:146-253` | `todo` (Region, inerte com overlay), `todo.input` (`text`, `set_value` e `submit` lendo `text`), `todo.filter` (`selected`) com três `Tab` (`select` lendo `todo.filter:selected`), `todo.item[<id>]` só dos visíveis (`completed`, `toggle` lendo `completed`, `remove` destrutiva sem reads e sem confirmação), `todo.counter` (`active`, `done`) (`:151-184`) |
| `--render`, `--svg`, `--svg-report`, `--semantics`, `--source` | `crates/showcase/src/main.rs:112-145`; `stage_page`, `semantics_json`, `render_page` em `crates/showcase/src/app/headless.rs:75`, `:150`, `:171` | `--semantics` sem GPU; página sem projeção dá `NoSemantics` (`crates/showcase/src/app/headless.rs:133`) |
