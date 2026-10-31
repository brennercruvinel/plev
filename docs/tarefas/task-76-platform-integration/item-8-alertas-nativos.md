---
id: task-76/item-8-alertas-nativos
title: "task-76, item 8: diálogos nativos de alerta e confirmação"
project: plev
type: secao
parent: task-76
status: referencia
date: 2026-09-24
date_basis: texto
topics: [plataforma, agentes, semantica]
relations: [adr-0058, task-58, task-92]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# handoff do item 8: diálogos nativos de alerta e confirmação

Registra o item 8: `engine::platform::alerts` põe `Alert` e `Confirm` atrás do trait `Alerts`, com `NativeAlerts` sobre `rfd::MessageDialog` no desktop, `FakeAlerts` e `NoAlerts`. No ide, a confirmação humana da task-58 pode ir ao diálogo nativo com `PLEV_NATIVE_CONFIRM=1`, pelo mesmo `confirm` e `decline` de `Actor::Human`; na Files, o botão `warn`, que o agente não pode pedir, mostra o aviso e projeta o texto mostrado em `files:alert`. O diálogo bloqueia o laço enquanto está aberto, e nenhum painel nativo foi aberto nesta máquina: a prova na tela é do orquestrador.


branch `task/76e-alerts`, base `0e49f828` (main após o item 4). worktree
`.claude/worktrees/agent`, target `target/agents/one`.

## a sonda

(a) o caminho de hoje no ide, por teste sem janela
(`cargo test --offline -p idestudio --lib -- workbench::tests`, 22 verdes, 0,19 s):

- um request que exige confirmação chega ao dispatcher e fica em
  `AwaitingConfirmation`: o `discard` de um agente (a policy do nó é
  `Confirm::ForAgents`, `an_agent_discard_waits_and_a_test_cannot_confirm_it`)
  e o `stage` com `--ask` (`Request::ask`, task-92, mesmo sem policy:
  `an_agents_question_is_asked_in_the_modal_and_never_projected`).
- `Workbench::dispatch` chama `report(id)` (`workbench/person.rs`): o
  receipt em `AwaitingConfirmation` entra na fila `awaiting` e
  `ask_next_confirmation` pega o mais velho, monta título, corpo e botão
  em `confirmation(&request)` (`notice.rs`: "Discard changes?",
  "test-agent asks to discard all changes to b.txt. This cannot be
  undone.", "Discard"; para o `--ask`: "Question from test-agent",
  "test-agent asks the person: \"...\" If allowed, it will stage b.txt.",
  "Allow") e chama `workspace.open_confirm_modal(id, title, body,
  confirm)` (`views/workspace/overlays.rs:136`), que empilha um
  `OverlayKind::Modal` com `PendingAction::ConfirmRequest { request }`.
  um modal por vez: o segundo espera o primeiro fechar
  (`requests_waiting_for_a_person_are_asked_one_at_a_time`).
- a pessoa responde pela view: o clique no botão de confirmar vira
  `UiRequest::Confirm { request }` e o cancelar, o Escape e o clique fora
  viram `UiRequest::Decline` (`dismiss_overlays`); o workbench traduz em
  `confirmed_by_person` -> `Workbench::confirm(id, Actor::Human)` ->
  `Dispatcher::confirm` (segunda checagem de stale, `Stage::Confirmed {
  by: human }`, `Executing`) e `declined_by_person` ->
  `Dispatcher::decline` -> `Stage::Rejected(Declined { by: human })`,
  nada roda. `Workbench::confirm` e `decline` fecham o modal
  (`close_confirm_modal`), avisam por toast e chamam
  `ask_next_confirmation` de novo. um agente ou um teste que tenta
  confirmar recebe `SemError::ConfirmByMachine` e o modal continua
  (`a_screen_reader_declines_through_the_workbench_and_a_test_cannot`).
- o que a gravação registra (`apps/idestudio/tests/record.rs`, o sink do
  `rec` no dispatcher): cada estágio do receipt, inclusive
  `awaiting_confirmation`, `confirmed { by: human }` ou `rejected {
  declined { by: human } }`; o `plev-inspect` mostra o estágio com
  `stage_detail` ("by human", `inspect/src/describe.rs:148`). o clique no
  modal entra pelo shell como `Kind::Input { PointerDown, at: Some([x,
  y]), target }` (`main.rs`, `record_input`) com `target =
  compositor.hit(x, y)`: o modal não liga nenhum id, então o alvo gravado
  é o controle ligado que estiver por baixo do diálogo, ou nenhum. quem
  respondeu está no estágio, não no input.

(b) o `rfd` 0.15.4 (`~/.cargo/registry/src/*/rfd-0.15.4/src/`):

- `MessageDialog::show` (`message_dialog.rs:82`) é síncrono e bloqueante:
  no macOS (`backend/macos/message_dialog.rs`) roda em `run_on_main`
  (exige a thread principal; fora dela, `dispatch2::run_on_main` se o
  `NSApp` está rodando, senão pânico) e chama `NSAlert::runModal()`, um
  laço modal aninhado do AppKit; o laço do winit fica parado até a
  resposta. com `set_parent` o alerta abre como sheet da janela
  (`beginSheetModalForWindow_completionHandler` e o mesmo `runModal`);
  sem pai, `sync_pop_dialog` abre um painel solto. `MessageLevel` vira
  `NSAlertStyle` (Informational, Warning, Critical); `MessageButtons`
  vira `addButtonWithTitle` na ordem, e o resultado mapeia pelo índice do
  botão: com `OkCancelCustom(a, b)` a resposta é `Custom(a)` ou
  `Custom(b)`, e qualquer outra coisa (Escape) é `Cancel`.
- `set_parent<W: HasWindowHandle + HasDisplayHandle>(&W)`
  (`message_dialog.rs:70`) guarda `RawWindowHandle` e `RawDisplayHandle`.
  `W` é `Sized`: um `&dyn Trait` não entra direto, então o engine passa um
  wrapper de uma linha (`ParentRef(&dyn Parent)`) que delega os dois
  métodos. a `Arc<Window>` do host serve: o `raw-window-handle` 0.6.2
  (único no lock, o do winit 0.30.13 e o do rfd) implementa
  `HasWindowHandle` e `HasDisplayHandle` para `Arc<H>` (`borrowed.rs:68,
  196`) e o `winit::window::Window` implementa os dois. o winit
  reexporta o crate (`winit::raw_window_handle`), então o engine não
  ganha dependência.
- alvos: macOS (`NSAlert`), windows (`win_cid`, botões custom só com
  `common-controls-v6`), linux gtk3 e portal xdg (o portal não tem
  diálogo de mensagem: `xdg_desktop_portal.rs:273` cai no `zenity` por
  processo). wasm tem `window.alert` e `window.confirm`
  (`backend/wasm.rs:329`), síncronos, mas o `rfd` só entra no engine na
  seção de desktop não móvel (`rust/engine/Cargo.toml:89`), então wasm
  e mobile ficam com `NoAlerts`. android e ios: nenhum backend.
  `AsyncMessageDialog` existe (mesma builder, `show` devolve um future);
  fica fora deste item.

(c) onde os apps mostram aviso e erro hoje, por conta própria:

- ide: o toast (`WorkspaceView::notify(message, intent)`,
  `views/workspace/mod.rs:305`), chamado só por `Workbench::report`
  (`person.rs:29`) com o texto de `notice.rs` (recusa, decline, falha do
  git; `Intent::Destructive` para stale e falha). os toasts são
  projetados como nós `notices.*` (`Role::Alert` para falha,
  `Role::Status` para decline) e o agente lê a mesma frase pela porta,
  em terceira pessoa.
- showcase: a página Files diz no `files.status` (`Role::Status`,
  `message`) o que o último request deu, com cor `Destructive` quando o
  alvo não tem diálogo (`files.rs:368`); as silences e a todo têm
  `Role::Status` para contagens; a página Toast é demonstração do
  widget. nada disso abre painel nativo.
- comps: `Toast`, `Alert` e `Modal` são widgets desenhados. o alerta
  nativo entra como uma capacidade a mais (um botão que pede
  `Alerts::alert` pelo dispatcher, só pessoa), com o texto do último
  alerta projetado como estado: o toast e o `status` continuam sendo a
  verdade do app; o painel nativo é outra representação, nunca uma
  segunda fonte.

## o trait, o falso e o que cada alvo faz

`rust/engine/src/runtime/platform/alerts.rs` (449 linhas de produção, 627 com
os testes), pelo molde do `files.rs`, linha por linha: trait, real sob
o cfg do `arboard` e do `rfd`, falso em todo alvo, nenhum em wasm e
mobile, `default_alerts()` escolhendo.

- `Alert { level: Level (Info, Warning, Error), title, body }`, com
  `Alert::info`, `warning`, `error`; `Confirm { title, body,
  confirm_label, cancel_label, destructive }`, com `Confirm::new(title,
  body)` ("OK", "Cancel", não destrutivo), `.labels(confirm, cancel)` e
  `.destructive(bool)`. `Answered { Confirmed, Declined }`;
  `AlertError::Unsupported(AlertKind { Alert, Confirm })` ("alert: no
  native dialog on this target"); `Shown<T> = Result<T, AlertError>`.
- `Parent`: marcador sobre `HasWindowHandle + HasDisplayHandle`
  (`winit::raw_window_handle`, sem dependência nova), implementado para
  todo tipo `Sized` com os dois handles: a `Arc<Window>` do host entra
  como `&dyn Parent` sem wrapper (teste de compilação
  `the_hosts_window_is_a_parent`). o `rfd` quer um tipo `Sized` no
  `set_parent`, então o `NativeAlerts` embrulha o `&dyn Parent` num
  `ParentRef` de uma linha.
- `trait Alerts { alert(&Alert, Option<&dyn Parent>) -> Shown<()>;
  confirm(&Confirm, Option<&dyn Parent>) -> Shown<Answered> }`,
  síncrono e modal como o `rfd`, na thread que pediu.
- `NativeAlerts` (desktop não móvel): `rfd::MessageDialog` com o nível
  como ícone, o alerta com `MessageButtons::Ok`, a confirmação com
  `OkCancelCustom(confirm_label, cancel_label)` e ícone de aviso quando
  `destructive`; a resposta lida pelo rótulo do botão (`Custom(confirm)`
  confirma; `Custom(cancel)`, `Cancel` do Escape e qualquer outra coisa
  recusa); `set_parent(ParentRef)` quando há pai (sheet da janela no
  macOS). os builders nunca abrem nada; o teste
  `the_native_builders_carry_the_request` confere o `Debug` do builder e
  `the_native_answer_is_read_by_the_button_pressed` o mapa da resposta.
- `FakeAlerts`: `will_confirm()`, `will_decline()`, `will_refuse()`
  respondem em ordem (um alerta consome uma resposta como o diálogo de
  arquivo consome), `asked()`/`take_asked()` devolvem `Asked { dialog:
  Dialog::Alert(..) | Dialog::Confirm(..), with_parent }`, `pending()`.
  sem nada carregado, um alerta é mostrado e uma confirmação é recusada,
  então uma página sob teste que pergunta sem querer não roda nada.
  `Arc<Mutex<A>>` implementa `Alerts` para o teste ficar com a outra
  ponta, como no `files.rs`.
- `NoAlerts` (wasm, android, ios): `Unsupported` nos dois. o wasm até
  tem `window.alert`/`confirm` pelo `rfd`, mas o `rfd` não entra no
  engine nesse alvo; fica como capacidade a nomear.

## a opção do ide e o que a gravação registra

`apps/idestudio/src/capability/mod.rs` (369 linhas), `person.rs` (133),
`main.rs` (588).

- `Workbench::confirm_natively(alerts: Box<dyn Alerts>, parent:
  Option<Box<dyn Parent>>)` e `confirms_natively()`. o `Workbench` guarda
  um `NativeConfirm { alerts, parent }` opcional; `None` (o padrão) é o
  modal desenhado de hoje.
- `ask_next_confirmation` continua igual até ter título, corpo e botão
  de `confirmation(&request)` (o texto nasce no mesmo lugar). com a
  opção ligada, `ask_natively` monta `alerts::Confirm::new(title,
  body).labels(confirm, "Cancel").destructive(d)`, com `d` lido da
  policy da ação no nó projetado agora (`destructive` do `discard` é
  true, do `stage` é false; o modal desenhado usa `Intent::Destructive`
  para toda confirmação), chama `alerts.confirm(&ask,
  parent.as_deref())` e a resposta entra por `confirmed_by_person` ou
  `declined_by_person`: o mesmo `Workbench::confirm`/`decline` com
  `Actor::Human`, a mesma segunda checagem de stale, o mesmo toast de
  decline, o mesmo `ask_next_confirmation` para o próximo da fila.
  `Unsupported` cai no modal desenhado (log de aviso). nenhum modal
  abre no caminho nativo (`modal_open()` fica false).
- o shell: `PLEV_NATIVE_CONFIRM=1` (`native_confirm()` ao lado de
  `background()`), aplicado em `resumed` logo depois de criar a janela:
  `bench.confirm_natively(default_alerts(), Some(Box::new(window.clone())))`,
  então o diálogo abre como sheet da janela do ide. desligado por
  padrão. `views/**`, `record.rs`, `agent.rs` e o `sem` não mudaram.
- o que a gravação registra: os estágios do receipt pelo sink que já
  está no dispatcher (`awaiting_confirmation`, `confirmed { by: human
  }` ou `rejected { declined { by: human } }`) e nenhum `Kind::Input`,
  porque nenhum clique nem tecla chegou à janela: o diálogo é da
  plataforma. o `plev-inspect` diz quem respondeu pelo estágio (`by
  human`, o mesmo `stage_detail` do modal), e `why.between.inputs` fica
  vazio. decidi não inventar um input sem coordenada: o `rec` só tem
  `PointerDown/Up/Move/Wheel/Key/Text`, um deles mentiria sobre o que
  aconteceu, e "quem respondeu" já mora no estágio, como com o modal (o
  clique no modal também não liga id nenhum, ver a sonda). o `rec` e o
  `record.rs` ficaram como estavam.
- um agente pela porta não abre nem responde nada: o request dele fica
  em `AwaitingConfirmation` como hoje e é a pessoa que responde no
  diálogo; `confirm` pelo agente continua `ConfirmByMachine`
  (`an_agents_question_is_asked_in_the_dialog_and_the_agent_never_answers_it`,
  `without_a_native_dialog_the_modal_is_drawn_instead`).

## o alerta no showcase

a página Files, sem página nova (`apps/uigallery/src/views/apps/files/mod.rs`
452 linhas, `files/semantics.rs` 353): um botão "warn" (ícone
`alert-triangle`) ao lado do "open file", com o status depois dos dois.

- `files.warn` (`Role::Button`, ação `warn`, `reads files:received`,
  policy `OPEN_FILE_ACTORS`: human, assistive, test; agente recusado
  como `NotAuthorized` antes do `execute`). o clique e o Enter viram
  `warn` por `Actor::Human` pelo dispatcher da galeria; `execute` monta
  `Alert::warning("Files", "<n> files received; nothing was written to
  disk.")` ("1 file" no singular), chama `alerts.alert(&alert, None)` e
  projeta o corpo mostrado em `files:alert` (texto, vazio até o primeiro
  alerta) e no resultado do receipt (`alert`). o status diz "the warning
  was shown"; em `Unsupported` o request falha ("alert: no native dialog
  on this target") e o status diz "no native alert on this target". a
  página não conhece a janela (o shell é dono das janelas e `app.rs`
  está fora deste item), então o painel abre solto, sem pai.
- Tab agora percorre os dois botões: open, warn, nenhum; `focus(id)`
  aceita `files.warn`; `FilesPage::with(dialogs, alerts)` e
  `set_alerts` (cfg(test)); `FilesPage::new()` usa `default_alerts()` na
  janela e `FakeAlerts` sob `cfg(test)`, como os diálogos.
- a projeção mudou de propósito: `files:alert` e o nó `files.warn`. o
  golden `rust/plevir/tests/fixtures/uigallery-files.snapshot.json` foi regravado
  (`UIR_BLESS=1`; só cresceu: 27 linhas) e as duas baselines da página
  (`files@core@1280x800`, `files@hoff@1280x800`) foram blessed com
  `--png` (a camada `files` mudou num retângulo de 379x48 em 562,300 no
  core e 415x44 em 561,286 no hoff: o botão novo; 5 controles ligados, 5
  nós). o subtítulo da página em `pages/table.rs` ainda fala só do
  diálogo e do drop; `table.rs` está fora da lista, fica para o
  orquestrador se quiser.

## testes

engine: `platform/alerts.rs` 9 unitários (o falso em ordem e o que
mostrou, sem nada carregado, `Unsupported`, o handle compartilhado,
`NoAlerts`, os builders e os defaults, a `Arc<Window>` como `Parent`, o
builder do `rfd` carregando o pedido, a resposta pelo rótulo);
`tests/alerts.rs` 4 sobre um `Operable` de teste (o `warn` da pessoa
mostra o alerta e o texto vira estado que o agente lê; o agente é
recusado antes de abrir e o teste e o `Assistive` não; `NoAlerts` falha
dizendo por quê; a confirmação da plataforma entra no dispatcher como
`confirm`/`decline` de pessoa: confirmado roda depois da segunda
checagem, recusado não roda, o agente não responde).

ide: `tests/native_confirm.rs` 5 contra repositório real e o worker de
verdade, com o `FakeAlerts` (um discard do agente confirmado roda e o
receipt tem `confirmed { by: human }`, recusado não roda e o toast diz,
nenhum modal abre, o falso recebeu título, corpo e botão do modal com
`destructive` true e sem pai; a pergunta do `--ask` chega pelo mesmo
diálogo em terceira pessoa, sem aviso destrutivo, e o agente não
responde; `Unsupported` cai no modal; a opção desligada abre o modal
como hoje; a sessão gravada tem a resposta como estágio, nenhum input, e
o `Inspector::why` diz `Confirmed { by: Human }` e `Declined { by: Human
}`). os testes de hoje do ide (`human_agent` 24, `agent_port` 9, `lab`
10, `record` 1, `rerun` 9, lib 59) verdes sem mudança.

showcase: `files/tests.rs` 10 (os 8 de antes com o nó novo, o Tab de
dois passos e o `files.warn` entre os ids ligados e observados pela
porta, mais `the_warn_button_shows_the_platforms_warning_and_the_text_shown_is_state`
e `an_agent_is_refused_the_alert_and_reads_what_the_person_saw`; pela
porta real o agente é recusado no `warn` com `["requested", "rejected"]`
e lê `files:alert` vazio).

## gate e regress

`df -h /` antes do gate: 29 GB livres; 29 GB depois. `./tool/tasks/fullcheck.sh`
verde no worktree com a perna wasm e a perna urnaui (`../urna`
presente): antes (o fechamento do item 4, base `0e49f828`) 2520
passed, 0 failed, 16 ignored em 118 binários; depois 2540 passed, 0
failed, 16 ignored em 120 binários (+20: 9 unitários do
`platform/alerts`, 4 de `engine/tests/alerts.rs`, 5 de
`ide/tests/native_confirm.rs`, 2 da Files; +2 binários, os dois
`tests/*.rs` novos). perna urnaui: 85 passed, 0 failed, 1 ignored em 6
binários, igual a antes. clippy `-D warnings`, fmt e a checagem wasm
limpos nas duas pernas (o showcase compila para wasm com `NoAlerts`).
`Cargo.lock` e `apps/urnadbapp/Cargo.lock` não mudaram.

`cargo run -p regress -- check`: antes do bless, 161
baselines, 159 iguais, 2 mudaram (`files@core`, `files@hoff`, o botão
novo); depois do bless dito acima, 161 iguais, 0 mudaram, sem frame
desenhado e sem GPU aberta (676 ms). o golden do todo e os das silences
continuam iguais.

linhas de produção (regra do guard): engine `platform/alerts.rs` 449,
`platform/mod.rs` 80; ide `workbench/mod.rs` 369, `workbench/person.rs`
133, `main.rs` 588; showcase `view/apps/files.rs` 452,
`view/apps/files/semantics.rs` 353. tudo abaixo de 639.

## ADR

[`0058-native-dialogs-answer-the-same-contract.md`](../../decisoes/interface/0058-native-dialogs-answer-the-same-contract.md) (proposed): o
trait com o real, o falso e o nenhum; a confirmação nativa como opção do
app sobre o mesmo `confirm`/`decline`; o alerta só de pessoa com o texto
projetado; o que o wasm e o mobile fazem; o que o item 5 e o
`AsyncMessageDialog` levam daqui. linha pro índice ([`indice-historico.md`](../../historia/indice-historico.md)
está fora da lista, fica para a integração):

```text
| 2026-09-24 | [native-dialogs-answer-the-same-contract](native-dialogs-answer-the-same-contract.md) | proposed | task-76 item 8: `engine::platform::alerts` com `Alert` (nível, título, corpo) e `Confirm` (título, corpo, dois rótulos, destrutivo) atrás do trait `Alerts` (`alert`, `confirm` -> `Answered`, `Unsupported`), `NativeAlerts` (`rfd::MessageDialog`, só desktop, `set_parent` com a `Arc<Window>` do host via o marcador `Parent`), `FakeAlerts` (responde em ordem e registra o que mostrou) e `NoAlerts` (wasm e mobile); a confirmação humana da task-58 pelo diálogo nativo como opção do ide (`PLEV_NATIVE_CONFIRM=1`, `Workbench::confirm_natively`, desligada por padrão) sobre o mesmo `confirm`/`decline` de `Actor::Human`, mesmo texto do modal, mesma segunda checagem, gravação e inspector iguais e sem input; o alerta como capacidade só de pessoa na Files do showcase (`files.warn`, `files:alert` com o texto que a pessoa viu); o diálogo bloqueia o laço, o `AsyncMessageDialog` fica nomeado |
```

## interfaces adicionadas (só crescimento)

- `engine::platform::alerts::{Level, Alert, Confirm, Answered,
  AlertKind, AlertError, Shown, Parent, Alerts, Answer, Dialog, Asked,
  FakeAlerts, NoAlerts, NativeAlerts (desktop), default_alerts}`.
- `ide::workbench::Workbench::{confirm_natively, confirms_natively}`.
- showcase (crate-private): `FilesPage::with(dialogs, alerts)`,
  `set_alerts` (cfg(test)), `Ids::warn`, `Target::Warn`; a projeção da
  Files ganhou `files.warn` e `files:alert`.
- variável do shell do ide: `PLEV_NATIVE_CONFIRM`.

## arquivos tocados fora da lista permitida, e por quê

nenhum. `rust/engine/src/runtime/platform/{alerts.rs,mod.rs}`,
`rust/engine/tests/alerts.rs`, `apps/idestudio/src/capability/{mod.rs,person.rs}`,
`apps/idestudio/src/main.rs`, `apps/idestudio/tests/native_confirm.rs`,
`apps/uigallery/src/views/apps/files/mod.rs`, `files/{semantics.rs,tests.rs}`,
`rust/plevir/tests/fixtures/uigallery-files.snapshot.json`,
`tool/bench/regress/baselines/files@{core,hoff}@1280x800/**`, o ADR
novo e este arquivo. `rust/engine/src/lib.rs` não precisou mudar
(`platform` já é `pub mod`). `record.rs` não mudou (ver a decisão da
gravação acima).

## limitações e decisões, sem esconder

- o diálogo nativo bloqueia a thread principal enquanto está aberto
  (`NSAlert::runModal`): com a opção ligada, o `act` de um agente pela
  porta cujo request espera uma pessoa só recebe resposta quando o
  diálogo fecha, e já com o estágio final, em vez de
  `awaiting_confirmation` na hora como com o modal; resultados do git e
  frames ficam na fila do laço enquanto isso. é por isso que o modal
  desenhado continua o padrão. o `AsyncMessageDialog` do `rfd` (mesma
  builder, `show` devolve um future) é o que manteria o laço vivo; fora
  deste item.
- o diálogo é pedido de dentro do `dispatch` (`report` ->
  `ask_next_confirmation`), como o modal é aberto; um por vez, o próximo
  da fila é perguntado quando a resposta entra.
- `destructive` da confirmação nativa segue a policy do nó; o modal
  desenhado continua `Intent::Destructive` para toda confirmação (não
  mexi em `views/**`).
- o alerta da Files abre sem janela pai: a página não tem o handle, e
  dar a ela exigiria mexer em `app.rs`/`dispatch.rs`, fora da lista. o
  ide, que tem a janela, passa o pai.
- nenhum diálogo nativo abriu nesta máquina: a implementação real ficou
  atrás do trait, cada teste usa o falso, e os dois testes do
  `NativeAlerts` só constroem o builder e leem o mapa da resposta. a
  prova na tela é do orquestrador (abaixo).
- windows: rótulos custom no `rfd` exigem a feature
  `common-controls-v6`; não é alvo, não liguei. linux: gtk3 ou `zenity`
  pelo portal, escolha do bring-up.
- o `FakeAlerts` consome uma resposta por chamada, alerta incluído
  (mesma regra do `FakeDialogs`); um teste que carrega `will_confirm()`
  e cuja página alerta antes de confirmar gasta a resposta no alerta.
- o subtítulo da página Files (`pages/table.rs`) não fala do alerta.

## o que o item 5 (notificações e tray) reaproveita, e o que fica para provar na tela

- item 5: uma notificação é um `Alert` que a plataforma mostra sem
  bloquear e sem resposta, então o trait não serve direto: o molde
  (trait, real, falso, nenhum, `default_*()`), o `Level` e o
  `Alert { title, body }` servem como estão; uma notificação com ação é
  um `Clicked` do menu (item 4) pelo mesmo dispatcher; o tray reaproveita
  a policy só pessoa e o `MenuModel`. o `Parent` serve para qualquer
  painel que precise da janela.
- para o brenner provar na tela, do orquestrador, nunca de teste:
  `PLEV_NATIVE_CONFIRM=1 PLEV_AGENT=1 cargo run --offline -p idestudio <repo>`,
  um `plev act --ask` ou um discard pelo agente: o sheet do macOS abre
  sobre a janela do ide com "Discard changes?" / "test-agent asks to
  discard all changes to X. This cannot be undone." e os botões
  "Discard" e "Cancel"; um clique em cada, e o `plev-inspect` da sessão
  (`PLEV_RECORD=1`) mostra `confirmed by human` e `rejected declined`.
  e na Files do showcase, um clique em "warn": o painel de aviso do
  sistema com "Files" / "0 files received; nothing was written to
  disk.", e `files:alert` com o texto pela porta.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
cargo test --offline -p engine --lib platform::alerts
cargo test --offline -p engine --test alerts
cargo test --offline -p idestudio --test native_confirm
cargo test --offline -p uigallery --lib files::
cargo test --offline -p engine -p idestudio -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check
cargo run --offline -p uigallery -- --render Files --theme core --out files.png --semantics files.json
UIR_BLESS=1 cargo test --offline -p uigallery --lib files::tests::the_page_projection   (regrava o golden)
cargo run --offline -p regress -- bless --page Files --theme core --png   (e --theme hoff)
PLEV_NATIVE_CONFIRM=1 PLEV_AGENT=1 PLEV_RECORD=1 cargo run --offline -p idestudio <repo>   (o sheet; do orquestrador)
```
