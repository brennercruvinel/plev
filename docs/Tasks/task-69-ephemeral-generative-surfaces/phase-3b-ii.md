---
id: task-69/phase-3b-ii
title: "task-69, phase 3b-ii: the page asks Claude for the screen"
project: plev
type: section
parent: task-69
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [llm, agents, generated-surfaces]
relations: [adr-0052, task-95]
evidence: [53535892, 659a562e, fe303efa, 00228db1]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 3b-ii: the Surface page asks Claude for the screen

Records phase 3b-ii: the Surface page chooses the proposer (rules or Claude) and asks Claude for the screen on a thread outside the app, with the wait and the cost visible and a cancellation that kills the child process. The result is that the proposal is verified at the moment of the request and arrives through `Execution::Started` and the dispatcher's `complete`, with the receipt of the right actor; three real calls, of 7 to 78 s, were made in this phase.


branch `task/69e-page-asks-claude`, worktree próprio, sobre a `main` @
cc908560 (a `main` já tinha andado para dcd731f4, só o journal). a
decisão é a 13 do programa: o Claude pelo `claude -p` local, já atrás de
`propose::Model` no crate `claude-model` (3b-i). nesta fase a página usa
esse modelo, fora da thread do app, com a espera visível e um cancelar.

## sonda da fase 3b-ii (antes do primeiro tipo, 2026-09-24)

dois testes descartáveis (`surface_page/tests/probe.rs` no showcase e
`rust/claude/tests/probe.rs`, rodados e jogados fora, nunca
commitados) e uma chamada real pelo exemplo do `claude-model`.

(a) na base, o `propose` da página roda na thread de quem despacha e já
está `completed` quando `dispatch` volta. um `propose` pelas regras
("only what is active and the add field") por `Page::dispatch`: o
receipt tem `requested`, `accepted`, `executing` e `completed` todos no
mesmo `now` do relógio da página (4 ms), `dispatch` voltou depois de
1,9 ms (build debug) na thread do teste, e a superfície abriu
(`todo_panel`). as regras rodam dentro do `execute`.

o que `Execution::Started` mais `Dispatcher::complete` exigem de quem
chama, medido com um `Operable` sobre o todo cujo `toggle` devolve
`Started`:

- o id que `execute` recebe é o que `dispatch` devolve (o `Operable`
  guarda esse id; nenhum outro canal diz qual request acabou);
- o receipt fica aberto em `executing`; enquanto isso o todo recebe e
  completa um `set_value` pelo caminho de sempre, e o item não mudou (a
  execução não terminou);
- `todo.with_dispatcher(|d, _, now| d.complete(id, outcome, now))`
  termina: `completed` no `now` 5, os estágios anteriores no 4. o
  `complete` pede o `RequestId`, um `Outcome` (o mapa de resultado) e um
  `Millis` do relógio da página, que só o `with_dispatcher` dá; não pede
  o `Operable`, só o dispatcher emprestado de novo;
- um segundo `complete` é `IllegalTransition { from: completed, to:
  completed }`; um `fail` num request que terminou na hora é
  `IllegalTransition { from: completed, to: failed }`; um id desconhecido
  é `UnknownRequest`.

a condição de parada não disparou: o `with_dispatcher` empresta o
dispatcher depois, e `complete` e `fail` servem com o id e o `now` da
página.

(b) com o `fake-claude`, `propose_verified(Llm::new(&mut ClaudeCli {
program: fake, .. }))` segura a thread que o chama pela chamada inteira:
o falso que responde na hora segurou 338 ms na primeira rodada (o
primeiro processo) e 16 a 17 ms nas duas seguintes (a chamada em si, 15
a 16 ms, é o `sh` subindo); o falso que nunca responde, com prazo de 2 s,
segurou 2,01 s até o kill. com o Claude de verdade, 5 a 90 s por
tentativa e até três tentativas: a thread do app congelaria isso tudo.

(c) chamada real 1 de 6: `cargo run -p intent --
--app todo "the items as a list with the add field on top"`, das
05:04:24 às 05:04:32. as regras recusam (`"top" is not a word of the
app`). o prompt tem 18.760 bytes: a superfície mostrada é a compacta da
task-95, que já repete `todo_panel.items` sobre `todo:items`. uma
tentativa, 7,2 s, 0,038 USD, 1.335 bytes, verificada de primeira. o IR
resumido:

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo", stack vertical md; padding lg, raised)
  todo_panel.entry (container, stack horizontal sm, center)
    todo_panel.input (text_field, "new todo", flex 1): text <- todo.input:text, set_value -> todo.input set_value, submit -> todo.input submit
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.items (list, "items", stack vertical xs)
    repeat over todo:items:
      todo_panel.item[{item}] (container, stack horizontal sm, center)
        todo_panel.item[{item}].text (text): text <- todo.item[{item}]:text
        todo_panel.item[{item}].done (checkbox, "done"): checked <- todo.item[{item}]:completed, toggle -> todo.item[{item}] toggle
```

o Claude usou um `repeat` sobre `todo:items`, com os ids do molde da
superfície mostrada (`todo_panel.item[{item}]`, `.text`, `.done`), tirou
o filtro e as contagens e deixou a entrada em cima. a condição do item 5
do briefing ("enumera os itens em vez de repetir") não disparou: o
`propose::prompt` não muda nesta fase. a ressalva fica registrada: a
superfície mostrada já carregava o `repeat`, e na corrida do ide da 3b-i,
sem superfície mostrada, o Claude escreveu os arquivos um a um. se ele
repete do zero é uma chamada sobressalente desta task, se o orçamento
sobrar, e importa para a fase 4, que nasce sem superfície.

o que a sonda decide para o desenho:

- o `propose` da página com `claude` devolve `Execution::Started` e
  guarda o id; um trabalhador (thread) leva o `ClaudeCli` clonado e as
  cópias do que o `Ask` empresta, roda `propose_verified(Llm)` e devolve
  por canal; o `wake` da página recolhe e chama `complete` ou `fail`
  pelo `with_dispatcher`, com o `now` do relógio da página;
- cancelar é um sinal conferido no laço do `try_wait` do `run`, junto do
  prazo, a cada 10 ms; o filho morre pelo pid como no prazo;
- o proponente padrão é `claude` quando o programa existe
  (`ClaudeCli::exists`: um caminho com diretório é um arquivo ali, um
  nome é achado no PATH), sem chamar o modelo; nos testes do próprio
  crate o programa não é procurado, para que nenhum teste do gate chegue
  ao `claude` de verdade (o teste que quer o falso o aponta).

## chamadas reais desta fase (2026-09-24)

três de seis, todas com o `claude` 2.1.281 desta máquina, sonnet, as
flags do crate, verificadas de primeira; nenhuma repetição foi preciso.
o custo é o de lista que o `claude` conta (a conta é de assinatura).

| # | por onde | intenção | tentativas | tempo | custo | resposta |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `examples/propose`, todo com a compacta mostrada (a sonda c) | the items as a list with the add field on top | 1 | 7,2 s | 0,038 USD | `repeat` sobre `todo:items`, molde com os ids da superfície mostrada (`todo_panel.item[{item}]`); sem filtro e sem contagens |
| 2 | a página Surface pela porta real, sem janela (o teste `#[ignore]` `the_real_claude_proposes_through_the_port`) | a mesma | 1 | 9,3 s (o ato do agente voltou `completed` 9,4 s depois do `executing`) | 0,040 USD | a mesma forma: a entrada em cima e a lista com o `repeat`; 14 nós; a superfície transformou no lugar |
| 3 | `examples/propose`, ide sem superfície mostrada (sobra usada para a pergunta que a sonda deixou) | the changed files as a list with stage and unstage on each, then the message and the commit button | 1 | 77,5 s | 0,102 USD | `repeat` sobre `changes:files`, molde com os ids do app (`changes.file[{item}]`, `.status` badge, `.stage` e `.unstage` botões), depois `review.message` e `review.commit`; superfície nova `review` |

somadas: 94 s e 0,18 USD. as três restantes não foram usadas.

## handoff da fase 3b-ii (2026-09-24)

goal: task-69 fase 3b-ii, a página Surface pede a tela ao Claude: a
pessoa (ou um agente pela porta) escreve o que quer, escolhe quem propõe
(as regras ou o Claude), e a tela pedida ao Claude nasce ou se
transforma no lugar, verificada, sem travar o app enquanto ele pensa,
com a espera e o custo visíveis e um jeito de cancelar; sem o `claude` na
máquina, a página continua pelas regras.

status: implementado. `cargo test -p uigallery -p claude -p offers`
verdes, `./tool/tasks/fullcheck.sh` verde com a perna urnaui e a perna wasm,
`regress check` verde, sobre a `main` @ cc908560.

### commits

branch `task/69e-page-asks-claude`:

- 53535892 task-69e: a sonda da fase 3b-ii no arquivo da task
- 659a562e task-69e: claude-model, a cancel signal read in the run loop, and exists
- fe303efa task-69e: the Surface page chooses the proposer and asks the Claude off the app's thread, with the wait shown and a cancel
- 00228db1 task-69e: the intent field keeps a card's width beside the proposer, or has the line to itself
- e o commit do ADR e deste handoff

o briefing pedia o proponente e o pedido fora da thread num commit e a
espera e o cancelamento noutro; os dois vieram juntos no fe303efa porque
o mesmo `Pending` (o request aberto, o canal, o tempo, a tentativa, o
sinal de cancelar) carrega os três, e um commit sem ele não compilaria
sozinho. o commit do `claude-model` (o sinal e o `exists`) ficou
separado e compila sozinho. o item 4 do briefing (o prompt) não entrou:
a sonda não pediu.

### a sonda

acima ("sonda da fase 3b-ii"). em uma linha: na base o `propose` da
página roda na thread de quem despacha e está `completed` quando
`dispatch` volta; `Started` mais `complete` pedem o id que `execute`
recebeu, um `Outcome` e o `now` do relógio da página, que o
`with_dispatcher` empresta de novo depois; o `fake-claude` segura a
thread pela chamada inteira (2,01 s num prazo de 2 s); a chamada real 1
veio com um `repeat` sobre `todo:items`, e o item 5 não disparou.
nenhuma condição de parada disparou.

### como a página escolhe o proponente e descobre o programa

`surface_page.proposer` é um `comps::Select` projetado
(`Select::project`: role `select`, estado `selected` e `expanded`,
`constraints.options` com os nomes, ações `select` com um texto entre
as opções, `open` e `close`; aberto, uma linha `surface_page.proposer[i]`
por opção). uma pessoa clica e escolhe; um agente pela porta faz `select`
com `"claude"` ou `"rules"` (o teste da porta faz isso). o `propose` lê
`surface_page.intent:text` e `surface_page.proposer:selected`.

o programa é descoberto sem chamar o modelo por `ClaudeCli::exists`
(novo no `claude-model`): um programa nomeado com diretório é um arquivo
naquele caminho; um nome só é um arquivo em algum diretório do PATH.
`asking::Claude::find` faz `ClaudeCli::new()` (o `PLEV_CLAUDE` ou
`claude`) e guarda o cli quando `exists`; com ele o select oferece
`rules` e `claude` e escolhe `claude`; sem ele oferece só `rules`. o
`claude-model` entrou no `apps/uigallery/Cargo.toml` na seção
`cfg(not(any(wasm32, android, ios)))`, com `ide`, `git` e `wire`: no wasm
o módulo `asking` compila sem thread e sem programa e `find` não acha
nada. nos testes do próprio crate (`cfg(test)`) o programa nunca é
procurado: o gate rodou nesta máquina, que tem o `claude`, e nenhum
teste chegou a ele; o teste que quer o falso o aponta
(`Claude::with(cli)`, `Proposing::set_claude`, ambos só em `cfg(test)`).

### o trabalhador e o `Started`/`complete`

`page.rs` (o `Page`, o `Operable` da página, saiu de `proposing.rs` para
um arquivo próprio: o `proposing.rs` passaria de 850 linhas) e
`asking.rs` (o trabalhador e o `Pending`).

- `Page::propose(id, author)`: com `rules`, como na 3a, `Done` na hora.
  com `claude`: `Claude::start(id, intent, app, caps, current, policy)`
  clona o `ClaudeCli` com um `Cancel` novo, e leva para uma thread
  (`surface-page-claude`) o `Snapshot`, o `Capabilities`, o
  `Option<VerifiedIr>` da superfície mostrada e a `GenerationPolicy::by(author)`,
  todos por valor (o `Ask` empresta, então o trabalhador monta o `Ask`
  lá dentro). o `propose` devolve `Execution::Started` e guarda
  `Pending { id, intent, started, attempt, rx, cancel, cancelled }`;
- na thread: `propose_verified(&mut Llm::new(&mut Counted { cli, tx }), &ask)`,
  onde `Counted` é um `Model` que manda `Msg::Attempt(n)` antes de cada
  `complete` do cli; no fim manda `Msg::Done(Answer { proposed, attempts:
  calls.len(), took, cost_usd: spent_usd() })`. só `VerifiedIr` (ou
  `ProposeError`) cruza o canal;
- `SurfacePage::wake(dt, todo)` chama `Page::settle` a cada tick.
  `Pending::poll` lê o canal sem bloquear: `Attempt(n)` move a tentativa,
  a virada do segundo inteiro move o relógio, `Done` traz a resposta;
  um trabalhador que sumiu sem resposta vira `ProposeError::Model`.
  com a resposta, `Page::arrived`: `show(ir)` abre ou transforma
  (`SurfaceHost::transform` quando o id está aberto, `open` senão), e
  `todo.with_dispatcher(|d, _, now| d.complete(id, outcome, now))` com o
  `now` do relógio da página; o outcome leva `surface`, `nodes`,
  `attempts`, `seconds` (num) e `cost_usd` quando o filho disse. um erro
  (o modelo, o verifier, o host) termina em `fail(id, why, now)`;
- enquanto espera, `Proposing::wake` devolve `Wake::after(POLL)` (100
  ms) e `SurfacePage::wake` soma `Wake::now()` quando o `settle` moveu
  algo (a tentativa, o segundo, a resposta): a página pede frame uma vez
  por segundo e a cada tentativa, não a cada frame;
- só um pedido por vez: `propose` fica `enabled(false)` (o nó e a ação)
  enquanto há `Pending`, então um segundo `propose` é `Rejected(Disabled)`
  no check, antes do `execute`; o `execute` também recusa, por garantia.

o receipt do ato real pela porta: `[(requested, 92), (accepted, 92),
(executing, 92), (completed, 9414)]` em ms do relógio da página, com
`Outcome { surface: "todo_panel", nodes: 14, attempts: 1, seconds:
9.32, cost_usd: 0.039856 }`.

### a espera e o cancelamento

- `surface_page.status` projeta `message` (texto) e `busy` (bool);
  enquanto pede, `attempt` (int, 1 a 3) e `seconds` (int, inteiros:
  a revisão muda uma vez por segundo, não a cada frame); depois de uma
  resposta do Claude, `attempt`, `seconds` e `cost_usd` (num). a mensagem
  durante a espera: `asking the Claude for "<intenção>": attempt 1 of 3,
  12 s`; ao terminar: `proposed by the Claude for "<intenção>" in 9.3 s,
  1 attempt, 0.040 USD`; recusa ou cancelamento em vermelho
  (`theme.colors.danger`). a legenda da metade gerada passa a "the same
  capabilities, proposed by the Claude for the intent above";
- o app segue: o teste com o falso lento faz um `toggle` do todo pelo
  dispatcher da página no meio da espera (`completed`), digita "buy" no
  campo da superfície (o texto chega ao app) e desenha frames; o
  `propose` está desabilitado e o `cancel` habilitado;
- `surface_page.cancel` é um `Button` projetado com `activate`,
  `enabled` só com `Pending`. o `execute` chama `Pending::cancel(actor)`,
  que levanta o `Cancel` (o `run` do `claude-model` lê o sinal a cada 10
  ms junto do prazo e mata o filho pelo pid, como no prazo) e anota quem
  cancelou; `Page::dispatch`, ainda com o dispatcher emprestado, faz
  `fail(id, "cancelled by human")` no mesmo empréstimo, e o `settle`
  faria o mesmo se o cancel viesse por outro caminho. a mensagem do
  trabalhador, quando chegar, cai num canal sem receptor. o teste confere
  que o pid do filho morre em menos de 5 s (na prática dezenas de ms) e
  que o request está `failed` com o motivo;
- no `claude-model`: `Cancel` (um `Arc<AtomicBool>`; um clone é o mesmo
  sinal, então o cli vai clonado para o trabalhador e a página fica com
  a alça), campo `pub cancel` do `ClaudeCli`, o `run` recebe `&Cancel` e
  confere antes de spawnar ("was cancelled before it started") e no
  laço ("was cancelled: killed (pid N)"); `ClaudeCli::exists`. a mudança
  no `run.rs` são 12 linhas.

### o que o prompt diz e o que o Claude respondeu

o `propose::prompt` não mudou: a sonda (c) veio com o `repeat`. o que o
Claude respondeu está na tabela das chamadas reais. duas leituras para
depois: sobre a superfície compacta mostrada (que já repete
`todo_panel.items` sobre `todo:items`), ele manteve o molde e os ids da
superfície e tirou o que a intenção não pedia (filtro, contagens); do
zero, no ide, com "as a list" na intenção, repetiu sobre `changes:files`
e nomeou o molde com os ids do app (`changes.file[{item}]`), como na
corrida 4 da 3b-i (que, sem a palavra lista, enumerou os arquivos). a
palavra "list" na intenção ou um `repeat` na superfície mostrada bastam
para o `repeat`; os ids do app no molde continuam (o host trata esses nós
como do app, o caso da tela fixa em IR). pedir ids próprios da
superfície é uma mudança de prompt para outra fase, com sonda própria.

### PNG e projeção

`SURFACE_PAGE_OUT=<dir> cargo test -p uigallery --lib
surface_page::tests::claude::the_real -- --ignored --nocapture` escreve
`surface-claude.png` e `surface-claude.json` (1200x800, core). o PNG: à
esquerda o todo escrito à mão, igual; à direita, sob "the same
capabilities, proposed by the Claude for the intent above", o campo com
"the items as a list with the add field on top", o select `claude`, o
botão propose, a linha "proposed by the Claude for ... in 9.3 s, 1
attempt, 0.040 USD" com o botão cancel (desabilitado) na ponta, o
controle compact | two columns, e o card da superfície: o campo "new
todo" com o add desabilitado e os três itens com o checkbox "done" (o
terceiro marcado); sem chips e sem contagens. `cargo run -p uigallery --
--render Surface` (a página em repouso, com o `claude` desta máquina
achado): 38 nós; em 390x844 hoff a linha fica sob o todo, o select e o
propose numa linha própria sob o campo. a projeção depois da proposta
real (31 nós; os 10 do todo como sempre; os da superfície e da linha,
forma lean, sem os reads):

```json
[
  { "id": "todo_panel", "role": "region", "label": "quick todo" },
  { "id": "todo_panel.card", "role": "group", "label": "todo", "parent": "todo_panel" },
  { "id": "todo_panel.entry", "role": "group", "parent": "todo_panel.card" },
  { "id": "todo_panel.input", "role": "text_input", "label": "new todo", "parent": "todo_panel.entry",
    "state": { "text": "" }, "actions": ["set_value", "submit"] },
  { "id": "todo_panel.add", "role": "button", "label": "add", "parent": "todo_panel.entry", "actions": ["activate"], "enabled": false },
  { "id": "todo_panel.items", "role": "list", "label": "items", "parent": "todo_panel.card" },
  { "id": "todo_panel.item[1]", "role": "group", "label": "1", "parent": "todo_panel.items" },
  { "id": "todo_panel.item[1].text", "role": "label", "label": "Absorb the plev demos into the showcase", "state": { "text": "Absorb the plev demos into the showcase" } },
  { "id": "todo_panel.item[1].done", "role": "checkbox", "label": "done", "state": { "checked": false }, "actions": ["toggle"] },
  { "...": "item[2] e item[3] iguais, o [3] com checked true" },
  { "id": "surface_page", "role": "region", "label": "propose a surface" },
  { "id": "surface_page.intent", "role": "text_input", "label": "intent", "parent": "surface_page",
    "state": { "text": "the items as a list with the add field on top" }, "actions": ["set_value"] },
  { "id": "surface_page.proposer", "role": "select", "label": "proposer", "parent": "surface_page",
    "state": { "selected": "claude", "expanded": false }, "constraints": { "options": ["rules", "claude"] },
    "actions": ["select", "open", "close"] },
  { "id": "surface_page.propose", "role": "button", "label": "propose", "parent": "surface_page", "actions": ["propose"] },
  { "id": "surface_page.cancel", "role": "button", "label": "cancel", "parent": "surface_page", "actions": ["activate"], "enabled": false },
  { "id": "surface_page.status", "role": "status", "label": "proposal", "parent": "surface_page",
    "state": { "message": "proposed by the Claude for \"the items as a list with the add field on top\" in 9.3 s, 1 attempt, 0.040 USD",
               "busy": false, "attempt": 1, "seconds": 9, "cost_usd": 0.039856 } }
]
```

### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ cc908560 (a árvore da base em detached HEAD neste worktree, `cargo test --workspace -- --list`: 2409 testes, 13 ignorados) | 2396 passam, 13 ignorados | 82 passam, 1 ignorado (a perna não muda: o urnaui é outro workspace) | 2478 |
| depois, esta branch @ 00228db1 (`./tool/tasks/fullcheck.sh`, saída 0) | 2404 passam, 0 falhas, 14 ignorados | 82 passam, 0 falhas, 1 ignorado | 2486 |

9 testes novos: 3 no `claude-model` (2 unitários, 1 com o falso), 5 no
showcase (4 com o falso, um deles pela porta real, e 1 do layout da
linha), e 1 ignorado (o `claude` de verdade). a contagem não cai. clippy
`-D warnings`, `cargo fmt --check` e o check wasm do showcase com `-D
warnings` verdes, dentro do gate, com a perna urnaui. `df -h /` antes do
gate: 22 GB livres. `cargo run -p regress -- check`:
157 baselines conferidas, 157 iguais, nenhum frame desenhado (a página
Surface não tem baseline; nenhuma outra página mudou).

### testes

- `rust/claude/src/lib.rs` (+2): um `Cancel` clonado é o mesmo
  sinal, um novo não; `exists` acha `sh` no PATH, o falso pelo caminho,
  e não acha um nome inexistente, um arquivo inexistente ou um diretório;
- `rust/claude/tests/cli.rs` (+1): uma chamada cancelada de
  outra thread mata o filho que dormia (o pid some), para bem antes do
  prazo de 30 s, o sinal levantado impede a chamada seguinte de começar
  (nenhum `prompt.2`), e um sinal novo deixa a chamada seguinte correr;
- `surface_page/tests/claude.rs` (4 + 1 ignorado), com o falso do
  `claude-model`: o pedido começa e a página segue (o select oferece
  rules e claude e escolhe claude; `Started`, `busy`, a mensagem, propose
  desabilitado e um segundo `propose` `Rejected(Disabled)`, um `toggle`
  do todo completa, uma tecla no campo da superfície chega ao app, a
  tentativa 1 aparece, o cancel pelo clique falha o request com
  "cancelled by human" e mata o filho, a superfície fica como estava); o
  Claude responde (o falso em `repair`: recusado e consertado, 2
  tentativas, 0,100 USD, `completed` com o outcome, a superfície em duas
  colunas no lugar, o texto e o caret do campo continuam no mesmo widget,
  a mensagem, os estados do status, os dois prompts que o falso recebeu,
  em repouso o wake não pede poll); sem programa o select oferece só
  rules (`select "claude"` é `InvalidParam`, as regras propõem na hora,
  o cancel é `Disabled`); pela porta real, sem janela, um agente escolhe
  o claude, escreve a intenção e propõe, o ato volta `completed` depois
  (o loop do shell tica a view), a superfície tem a forma, o status diz
  quem propôs e o custo, o receipt é do agente com `executing` e
  `completed` separados; e o ignorado com o `claude` de verdade;
- `surface_page/tests.rs` (+1): o proponente ao lado do campo quando o
  campo guarda `card_min_w`, sob ele senão (1500, 900 e 358 de largura),
  o cancel na linha da mensagem, nada fora da largura;
- os testes da 3a, da fase 2 e da task-95 passaram sem mudança (o
  `setup()` deles continua `SurfacePage::new`, que nos testes não acha
  programa).

### validação executada

- `cargo test -p uigallery -p claude -p offers`;
- `cargo clippy -p uigallery -p claude -p offers --all-targets -- -D warnings`;
- `cargo fmt --check`;
- `RUSTFLAGS="-D warnings" cargo check --target wasm32-unknown-unknown -p uigallery`;
- `./tool/tasks/fullcheck.sh` e `cargo run -p regress -- check`
  (abaixo);
- o teste ignorado com o `claude` real (chamada 2), `--render Surface`
  em 1200x800 core e 390x844 hoff, e o exemplo do `claude-model` para as
  chamadas 1 e 3. nenhuma janela foi aberta.

### seção de ADR

[`0052-proposers-propose-the-runtime-verifies.md`](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/0052-proposers-propose-the-runtime-verifies.md), seção nova no fim,
"addendum, 2026-09-24: the page asks the Claude off the app's thread
(task-69 phase 3b-ii, proposed)": contexto, decisão (o proponente como
capacidade, a descoberta do programa, o trabalhador pelo contrato do
dispatcher, um pedido por vez, a espera como estado, o cancel no laço do
`run`, o prompt que não muda), custo (as três chamadas), consequências e
o que evitar. para o orquestrador (o [`adr/`](../../adr/adr.md) é proibido
aqui): no fim da linha do índice, algo como "; addendum 3b-ii
(proposed): a página Surface escolhe o proponente (`select` rules |
claude, claude só onde `ClaudeCli::exists`), pede ao Claude numa thread
com `Execution::Started` e termina por `complete`/`fail` no `wake`, a
espera e o custo como estado do status, `Cancel` lido no laço do `run`
mata o filho".

### decisões

- o proponente é um `Select` do comps, não um `Tabs`: ele projeta
  `select` com um texto entre as opções, que é o que um agente manda
  pela porta; o `Tabs` da fase 2 (o controle da forma) continua sem
  projeção (o P2 dela);
- `claude` é o padrão onde o programa existe (decisão 13 do programa);
  nos testes do crate nunca é procurado, por `cfg(test)` em
  `Claude::find`. razão: o gate roda em máquinas com o `claude` e não
  pode chegar a ele;
- o IR é verificado no instante do pedido e não de novo na chegada. o
  que sumiu enquanto o Claude respondia (um item removido) não dá
  instância, e um pedido sobre ele o dispatcher recusa; nada não
  verificado é mostrado. re-verificar recusaria uma superfície inteira
  por um item que saiu, o que pareceu pior;
- o cancel falha o request na hora, sem esperar o trabalhador confirmar
  a morte do filho (que vem em até 10 ms); a mensagem tardia é
  descartada;
- `seconds` inteiro no estado e na mensagem: a projeção muda uma vez
  por segundo enquanto espera, não a cada frame;
- o campo de intenção guarda `card_min_w` ao lado dos controles ou tem
  a linha para si (uma intenção é uma frase). em 1200 de largura a
  metade tem 434 px e os controles vão para baixo;
- `Cancel` compartilhado pelos clones do `ClaudeCli`. razão: o cli vai
  clonado para o trabalhador e a página guarda a alça; um pedido novo
  ganha um `Cancel` novo;
- `Page` em `page.rs`. razão: o limite de 639 linhas; `proposing.rs`
  ficou com 555, `page.rs` com 320, `asking.rs` com 288,
  `surface_page.rs` com 564.

### limitações

- um clique da pessoa no select troca o widget direto, sem dispatcher,
  como a tecla no campo de intenção desde a 3a; o `select` de um agente
  passa pelo dispatcher;
- o `Select` aberto vê o evento sozinho e é desenhado por último na
  camada padrão (`render_open`), por cima da superfície; com ele aberto,
  um clique fora fecha e não chega a mais nada;
- o `TextField` do comps quebra um texto comprido em mais de uma linha
  dentro do campo (uma intenção maior que o campo transborda); é
  comportamento do widget, não desta página;
- o botão cancel fica sozinho, desabilitado, na ponta da linha da
  mensagem quando nada é pedido;
- os segundos e a tentativa mudam a revisão da projeção uma vez por
  segundo durante a espera; os deltas da porta carregam isso;
- se o app fechar com um pedido em curso, a thread morre com o processo
  e o filho `claude` não é morto por nós: ele termina sozinho quando
  responde (a stdout dele já não existe) e a chamada é cobrada;
- a thread do trabalhador não é juntada (`join`); termina sozinha depois
  da resposta, do cancel ou do prazo de 180 s por tentativa;
- os testes de processo são só unix (o falso é um script sh); o teste
  da porta e o ignorado são só desktop.

### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima); o
  [`changelog.md`](../../CHANGELOG.md) (a página Surface pede ao Claude, com espera e
  cancel); na entrada `[crates.showcase]` do `docs/ATLAS.toml` a
  dependência `claude-model` (desktop) e os módulos `asking` e `page`;
  o comentário de `app/headless.rs` não muda;
- `Cargo.lock` mudou: o `showcase` passa a depender do `claude-model`
  (só desktop);
- `ClaudeCli::exists` não confere o bit de execução: um arquivo sem ele
  é "existe" e a chamada falha com "could not start";
- um `SESSION_VARS` novo de uma versão futura do `claude` passa ao filho
  (limitação da 3b-i, inalterada).

### o que a fase 4 (o editor de vídeo) precisa

- uma página nova, com suas capacidades (`detect_silence`,
  `preview_segment`, `change_threshold`, `apply_cut`, `reject_cut`) e
  sem superfície mostrada no começo: a chamada 3 mostra que o Claude
  repete do zero quando a intenção diz lista, mas nomeia o molde com os
  ids do app. se a superfície de revisão deve ter ids próprios (para o
  host projetá-la como dela e transformá-la por `SemId`), o prompt
  precisa pedir isso, com sonda própria; hoje o host trata esses nós
  como do app;
- `media` no vocabulário do `uir` (fora do v1, dito na fase 1) para o
  vídeo e as prévias;
- a espera do Claude sobre um registro maior: o ide levou 77 a 89 s; um
  editor de vídeo com muitas capacidades pode passar de um minuto por
  tentativa e três tentativas passam de 180 s. o cancel existe; a
  mensagem de espera diz a tentativa e os segundos;
- a página desta fase reutiliza-se como está: `asking::Claude`,
  `Pending` e `Page::settle` não sabem nada do todo além do
  `TodoPage::with_dispatcher` e do `snapshot`; uma página nova precisa
  do mesmo empréstimo do seu dispatcher (um `with_dispatcher` seu) e de
  um `wake` que chame o `settle`;
- "corta os silêncios desse vídeo e depois quero revisar os cortes" pede
  que a capacidade `detect_silence` rode antes da superfície (o ffmpeg
  em subprocesso, como o urnaui): outro trabalho fora da thread, pelo
  mesmo `Started`/`complete`; o padrão desta fase serve.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0052: proposers propose, the runtime verifies](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/0052-proposers-propose-the-runtime-verifies.md)
- [task-69: ephemeral and generated surfaces](task-69-ephemeral-generative-surfaces.md)
- [task-95: the todo projects its items as a list](../task-95-the-todo-projects-its-items-as-a-list/task-95-the-todo-projects-its-items-as-a-list.md)
<!-- /related -->
