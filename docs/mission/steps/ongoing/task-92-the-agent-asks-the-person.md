---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-24
domain: agent-surface
task: task-92
depends: [task-58b, task-89, task-91]
branch: task/92-ask-the-person
origin: terceira rodada do checkpoint a (task-62): o A2''b parou por não ter como perguntar à pessoa
severity: P2 (completude do handoff; nada que existe fica inválido)
---

# task-92: o agente pergunta à pessoa

## por que

na terceira rodada do checkpoint a, dois agentes LLM receberam a mesma
recusa: a pessoa tinha tirado um arquivo do index que a tarefa pedia.
um agente recolocou o arquivo por cima dela; o outro parou, dizendo que
recolocar seria "desfazer uma ação deliberada de um humano no aplicativo
dele, sem ter como perguntar o motivo", e que `notices` só oferece
`dismiss`. o contrato deu aos dois o que aconteceu e quem fez; não deu um
jeito de o agente consultar a pessoa antes de passar por cima.

o handoff humano-agente é um dos itens que a tese promete provar (goal
10, task-80). o mecanismo mais perto já existe: um request pode esperar a
confirmação de uma pessoa (`Stage::AwaitingConfirmation`, task-58 e 58b),
hoje só quando a `Policy` da ação exige (o `discard` do ide pede a um
agente).

## direção (orquestrador; a sonda confirma)

o agente pode pedir, num ato, que a pessoa confirme antes: o request
espera a confirmação de uma pessoa como o `discard` já espera, com um
texto curto do agente dizendo por quê ("você tirou src/session.rs; a
tarefa pede os três arquivos: posso recolocar?"). a pessoa confirma ou
recusa no modal que o ide já tem; o agente recebe o estágio como recebe
hoje. é uma extensão do que existe: nada de canal novo de conversa.

## aceite

um agente pelo `plev` pede para recolocar o arquivo com a pergunta; o
ide mostra o modal com o texto do agente; a pessoa (pelo dispatcher,
como `Actor::Human`) confirma e o stage roda, ou recusa e o agente recebe
o `declined`; tudo gravado e legível pelo inspector; formato das
mensagens só cresce; gate verde.

## handoff (2026-09-24): parada na sonda

goal: task-92, o agente pergunta à pessoa.
status: parada na sonda. a condição "a sonda sair diferente" disparou; só
a sonda foi feita, como teste, e nada do pedido foi implementado.

### a sonda

dois testes em `crates/ide/tests/agent_port.rs`, na base (`88923f2`), com
o porto real, o git worker real e um shell que serve o porto como a
janela do ide serve (`crates/ide/src/main.rs`, `serve_agent`): depois do
wake do porto e depois de cada resposta do worker.

- `an_agent_discard_through_the_port_waits_for_the_person_who_confirms_it`:
  o agente pede `discard` de `README` pela porta; recebe `requested` e
  `awaiting_confirmation` (`Confirm::ForAgents`); o ide abre o modal; a
  pessoa clica em Discard, que confirma pelo dispatcher como
  `Actor::Human` (`UiRequest::Confirm`, `Workbench::confirmed_by_person`,
  `Dispatcher::confirm` com a segunda checagem); o discard roda, e
  `confirmed`, `executing` e `completed` chegam ao agente no drain depois
  da resposta do worker. confere com a direção.
- `a_decline_reaches_the_agent_only_at_the_next_drain`: a pessoa clica em
  Cancel, que recusa pelo dispatcher como `Actor::Human`; nada roda, e o
  toast diz "Declined: discard of new.txt by test-agent did not run". o
  sink do porto ouve o `rejected` na hora, mas o estágio fica parado no
  porto: nada serve o porto depois da entrada de uma pessoa, e uma recusa
  não manda nada ao worker. em 500 ms o agente não recebe nada; o
  `declined` só sai no drain da próxima mensagem dele (o teste manda um
  `observe` e recebe as duas coisas). aqui a sonda sai diferente da
  direção ("recusa e o agente recebe o estágio").

o que o modal mostra hoje, e onde o texto nasce: título "Discard
changes?", corpo "test-agent asks to discard all changes to README. This
cannot be undone.", botão "Discard" (e "Cancel"). o texto nasce em
`Workbench::confirmation` (`crates/ide/src/workbench/notice.rs`), a partir
do ator, da ação e do alvo do request; `ask_next_confirmation`
(`person.rs`) o passa a `WorkspaceView::open_confirm_modal`
(`views/workspace/overlays.rs`), que abre o `comps::feedback::Modal` e o
overlay `OverlayKind::Modal`. o modal não entra na projeção: nenhum nó o
representa, e um agente só lê `notices` (os toasts).

### problema

um `declined` não chega a um agente que só espera por ele. o porto manda
estágios só quando o shell o drena (`AgentPort::drain`, `Server::forward`),
e a janela do ide drena só no wake do porto e depois de um resultado do
git. o `plev act` manda o act e fica esperando o estágio terminal: com a
recusa, espera até o timeout (30 s por padrão) e sai com 3. quando a
conexão fecha, `Server::close` esquece os requests dela, e o estágio que
sair depois se perde.

com o pedido da task, o buraco cresce: o agente poderá perguntar antes de
qualquer ação, e toda confirmação que termina sem o worker fica igual à
recusa. `open`, `cancel`, `set_value`, `show_diff` e `dismiss` terminam na
hora (`Execution::Done`), e a segunda checagem que recusa como stale na
confirmação também. no cenário do aceite (A2''b, a pergunta num `stage`),
confirmar funciona, porque o `stage` vai ao worker; recusar é exatamente
o caso quebrado ("o agente recebe o `declined` e para").

nada disso se resolve nos arquivos permitidos: o drain é de
`crates/wire/src/port.rs` e de quem o chama (`crates/ide/src/main.rs`,
`crates/ide/src/agent.rs`, o laboratório).

### opções

1. o porto acorda o próprio loop quando o sink ouve, fora de um drain, um
   estágio de um request que ele emitiu. arquivos: `crates/wire/src/port.rs`
   (fora da lista) e `server.rs` (na lista; está com 610 das 639 linhas).
   impacto: todo host (a janela do ide, o `ide-lab`, o showcase) passa a
   entregar o estágio sem mudar o shell; nenhuma mudança de protocolo; um
   wake a mais por estágio ouvido fora de um drain, e os wakes já
   coalescem. o risco é pequeno: o sink roda na thread do app e o wake é
   o mesmo closure que o leitor já chama.
2. a janela do ide serve o porto depois de toda entrada de uma pessoa,
   como já serve depois de um resultado do git. arquivo:
   `crates/ide/src/main.rs` (proibido). impacto: uma linha, mas vale só
   para a janela; cada host precisa lembrar disso (o laboratório também,
   quando tiver uma pessoa que responde), e o teste precisa imitar o shell.
3. aceitar a espera e documentar: o agente precisa mandar outra mensagem
   para receber a resposta da pessoa. impacto: o `plev act --ask` que
   espera trava até o timeout, o aceite da recusa não fecha com o `plev`,
   e o próximo experimento com LLM mediria o timeout, não o handoff.

### recomendação

a opção 1. a causa está no porto: só ele sabe que tem requests emitidos
esperando, e ele entrega estágios só quando alguém o drena. corrigir ali
serve todo host de uma vez e deixa o shell como está. é aditivo e não muda
o protocolo.

### decisão necessária

1. autorizar `crates/wire/src/port.rs` (opção 1) ou `crates/ide/src/main.rs`
   (opção 2), ou aceitar a espera (opção 3). a sonda da recusa vira o
   teste que prova a escolha (ela hoje fixa a espera).
2. junto, dois arquivos fora da lista que a implementação vai pedir de
   qualquer jeito, aditivos:
   - `crates/sem/src/redact.rs`: a regra única de segredo é
     `Secrets::request`, e é ela que o `rec` e o porto aplicam ao request.
     sem ela, a pergunta sobre um alvo secreto vai inteira para a gravação
     e para o `asked`. a mudança: a pergunta vira `REDACTED` quando o alvo
     é secreto, como o texto livre de um erro. três linhas e um teste.
   - `crates/inspect/src/why.rs`: `text::why` recebe só o `Why`, e o
     `Asked` dele não carrega o campo novo do request; mexer só em
     `text/**` não põe a pergunta no `why`. a mudança: um campo em
     `Asked`, preenchido do request gravado (aditivo também no `--json`).
     quem respondeu já aparece hoje: "confirmed: by human" nos estágios, e
     "human declined to confirm" no veredito.

### o que eu faria depois da decisão (nada implementado)

- a forma: `sem::Request.ask: Option<String>`, com `serde(default,
  skip_serializing_if = "Option::is_none")`, então um request sem pergunta
  e as sessões antigas (`MONS`) leem igual e o `rec` não muda; `Act.ask`
  no `wire`, aditivo no v1; `plev act <target> <action> --ask "<texto>"`.
- o limite: 280 caracteres, não vazio, sem caractere de controle, cobrado
  no porto como `bad_message` (onde o `MAX_NAME` já é cobrado) e no parse
  do `plev`.
- o dispatcher: `check` responde `NeedsConfirmation` quando a `Policy`
  exige ou quando um agente pergunta; quem já exigia continua igual, e a
  segunda checagem na confirmação não muda.
- uma pessoa não pergunta a outra: a pergunta de `Actor::Human` ou
  `Actor::Assistive` não pede nada, como o `Confirm::ForAgents` não pede
  nada a uma pessoa. recusar com um motivo novo exigiria uma variante em
  `Rejection`, e o `rec` tem um `match` completo sobre ela
  (`crates/rec/src/weigh.rs`), o que é condição de parada.
- o modal: diz que o agente pergunta e mostra o texto dele, na terceira
  pessoa ("the person", nunca "you"); confirmar e recusar como hoje.
- a projeção: a pergunta não entra em `notices` nem num nó do modal (o
  modal não é projetado, e `project.rs` está fora da lista). o autor a vê
  no `asked` do `requested`; outro agente não vê requests alheios pela
  porta.
- ninguém responde: o request espera, como o `discard` espera hoje; não há
  api de expirar.
- o `rec` estima o peso de um request sem o texto da pergunta
  (`weigh.rs`); com 280 caracteres no máximo, é um erro pequeno e fica
  registrado.

### contagem

`cargo test --workspace` na base: 2267 passaram, 11 ignorados. com a sonda:
2269 e 11. `./script/gate` verde no commit da sonda, com a perna urnaui
(82 passaram, 1 ignorado), o clippy, o fmt e o wasm check.

### ADR

nenhum escrito, porque parou. a linha que eu proporia: uma seção datada no
ADR da 58b (`docs/ADR/actions-carry-observed-state.md`): "um agente pode
pedir, no próprio request, que uma pessoa confirme antes; o request espera
como um `Confirm::ForAgents`".

### o que o próximo experimento com LLM precisa dizer sobre isso

- o laboratório não tem uma pessoa que responde ao modal: os modos
  `--human` só tiram um arquivo do index. para medir o handoff, ele precisa
  de uma pessoa que confirma ou recusa (o `ide-lab` está fora desta task),
  e a recusa precisa chegar ao agente (esta parada).
- o briefing do agente continua sem ensinar a regra, como na terceira
  rodada: a pergunta aparece no `plev act --help` e na doc do `Act`, e a
  corrida mede se o agente a encontra, não se obedece.
- a hipótese a escrever antes: com a pessoa antes do submit, o agente
  pergunta em vez de passar por cima (A2''a) ou de parar (A2''b), e a
  resposta da pessoa decide o commit.

### commits

- `2f61463` task-92: the probe, an agent's discard through the port and
  the person's answer
