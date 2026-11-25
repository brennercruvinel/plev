---
id: task-69/phase-3a
title: "task-69, phase 3a: proposers without a provider"
project: plev
type: section
parent: task-69
status: reference
date: 2026-09-24
date_basis: text
recorded: 2026-09-23
topics: [agents, llm, ir, generated-surfaces]
relations: [adr-0052, task-68, task-95]
evidence: [47ff75d, 8a6123d, 605e235, fb4ca07]
source: docs/log/steps/task-69-ephemeral-generative-surfaces.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 3a (orchestrator, 2026-09-24)

Records phase 3a: the `Proposer` trait, a deterministic, rules-based proposer over the capability registry, and the language model loop behind `Model`, with three attempts and a fake model. The result is that an intent expressed in text reaches the runtime only as verified IR, and the Surface page proposes through the dispatcher of the whole. The real provider was left for phase 3b, by brenner's decision.


a fase 3 se divide. a 3a faz tudo o que não depende do provedor do LLM:
- o trait `Proposer`;
- o proponente determinístico por regras sobre o registro de
  capacidades;
- o proponente LLM atrás de um trait de modelo, com o prompt (o registro,
  o schema, a superfície atual e os ids que devem continuar) e o laço de
  verificação com tentativas limitadas, provado com um modelo falso;
- a página Surface com um campo de intenção que leva a uma superfície
  proposta, verificada e aberta ou transformada.

a 3b liga um provedor real e espera a decisão do brenner (local ou API).

# fase 3a: proponentes sem provedor

branch `task/69c-proposers`, worktree próprio, sobre a `main` @ a678ef4.

## sonda da fase 3a (antes do primeiro tipo, 2026-09-24)

um teste descartável (`rust/plevir/tests/probe_69c.rs`, rodado e depois
jogado fora) lê o golden da projeção real do todo
(`rust/plevir/tests/fixtures/uigallery-todo.snapshot.json`, que o teste
`uir_fixture` do showcase mantém igual à página), monta
`Capabilities::from_snapshot` e escreve à mão o IR que um proponente por
regras escreveria para "só o que está ativo e o campo de adicionar". a
superfície atual é a da página Surface (a fixture da task-68, id
`todo_panel`), e os nós das capacidades que continuam guardam os ids
dela.

o registro, como o proponente o recebe (saída do teste, os itens 2 e 3
iguais ao 1):

```text
data   todo.input:text          text  label="new todo"
data   todo.filter:selected     text  label="filter"
data   todo.item[1]:completed   bool  label="Absorb the plev demos into the showcase"
data   todo.counter:active      int   label="2 active · 1 done"
data   todo.counter:done        int   label="2 active · 1 done"
action todo.input               set_value param=text
action todo.input               submit    param=none
action todo.filter[all]         select    param=none  label="All"
action todo.filter[active]      select    param=none  label="Active"
action todo.filter[completed]   select    param=none  label="Completed"
action todo.item[1]             toggle    param=none
action todo.item[1]             remove    param=none  destructive=true
```

o registro não carrega role nem parent. o `Snapshot`, que o proponente
também recebe, carrega: `todo` region, `todo.input` text_input,
`todo.filter` tab_list com três filhos `tab`, `todo.item[k]` list_item,
`todo.counter` status.

o IR escrito à mão verifica (`verify` com a `GenerationPolicy` padrão; a
ordem de foco é `todo_panel.input`, `todo_panel.add`,
`todo_panel.filter.active`):

```json
{ "plev_ir": 1, "surface": { "id": "todo_panel", "title": "quick todo", "root": {
  "sem": "todo_panel.card", "component": "card", "label": "todo",
  "props": { "padding": "lg", "elevation": "raised" },
  "layout": { "stack": { "axis": "vertical", "gap": "md" } },
  "children": [
    { "sem": "todo_panel.entry",
      "layout": { "stack": { "axis": "horizontal", "gap": "sm", "align": "center" } },
      "children": [
        { "sem": "todo_panel.input", "component": "text_field", "label": "new todo", "size": { "flex": 1 },
          "bind": { "state": { "text": "todo.input:text" },
                    "actions": { "set_value": { "target": "todo.input", "action": "set_value" },
                                 "submit": { "target": "todo.input", "action": "submit" } } } },
        { "sem": "todo_panel.add", "component": "button", "label": "add",
          "bind": { "actions": { "activate": { "target": "todo.input", "action": "submit" } } } } ] },
    { "sem": "todo_panel.filter", "label": "filter",
      "layout": { "stack": { "axis": "horizontal", "gap": "xs" } },
      "children": [
        { "sem": "todo_panel.filter.active", "component": "chip", "label": "Active",
          "bind": { "actions": { "activate": { "target": "todo.filter[active]", "action": "select" } } } } ] },
    { "sem": "todo_panel.active", "component": "stat", "label": "active",
      "bind": { "state": { "value": "todo.counter:active" } } } ] } } }
```

de onde vem cada ligação, e a regra que a acha sem nome do todo:

| nó | ligação | o fato | a regra |
|---|---|---|---|
| `todo_panel.input`, text_field | `text` de `todo.input:text`; `set_value` e `submit` para `todo.input` | um nó de role `text_input` com um estado de kind text, uma ação com `ParamSpec` text e outra sem parâmetro | "adicionar": o campo liga o estado de texto, a ação de parâmetro text e a sem parâmetro do mesmo nó; o label é o do nó |
| `todo_panel.add`, button | `activate` para `todo.input` `submit` | a ação sem parâmetro do mesmo nó | o botão liga a ação que o Enter do campo liga |
| `todo_panel.filter.active`, chip | `activate` para `todo.filter[active]` `select` | filho `tab` de um nó `tab_list`, uma ação só, sem parâmetro, label "Active" | "filtro": um chip por opção de uma escolha (tab_list ou select com filhos tab, option ou radio de uma ação sem parâmetro); o seletor deixa só a opção cujo label ou chave é a palavra |
| `todo_panel.active`, stat | `value` de `todo.counter:active` | um estado de kind int com a chave `active` | "contagem": um stat por estado int ou num; o label é a chave, porque o label do nó (`2 active · 1 done`) muda com o valor |

os containers (`todo_panel.card`, `todo_panel.entry`, `todo_panel.filter`)
guardam os ids da superfície atual porque os nós que eles seguram
continuam.

resposta: basta. cada ligação sai de role, `ValueKind`, `ParamSpec` e
parent, e nenhuma regra cita `todo`, `input`, `filter`, `counter`,
`active` ou `done`. a condição de parada da sonda não disparou. três
achados ficam como limite do proponente por regras:

1. o seletor é uma palavra da intenção comparada com as palavras do
   registro (chave de estado, label e chave de opção), sem caixa, sem
   acento e sem o s do plural. o registro do todo está em inglês. a saída
   do teste: `selector "ativo": stats [], options []` e `selector "active":
   stats ["todo.counter:active"], options ["todo.filter[active] select"]`.
   as palavras de regra (adicionar, filtro, contagem, compacto, duas
   colunas, só) o proponente entende nas duas línguas porque são dele; a
   palavra que escolhe o que mostrar é do app.
2. "o que está ativo" como os itens ativos não sai do registro. ele diz
   `completed: bool` por item, e "ativo quer dizer não concluído" é um
   sentido do todo, não um fato. o todo também não projeta a lista dos
   itens (o P2 da fase 1), então nenhuma superfície repete sobre eles. o
   que o registro dá para "active" é a contagem e a opção do filtro que o
   próprio app chama assim, e é isso que o IR liga.
3. o `Capabilities` sozinho não acha a escolha: role e parent (a tab_list
   e as tabs) só estão no snapshot. o trait recebe os dois, então não
   falta nada, mas a regra lê o snapshot além do registro.

## parada: o campo de intenção e o botão propor não chegam ao dispatcher da página

problema: o item 5 do briefing pede o campo e o botão como capacidades
projetadas, pelo mesmo dispatcher da página, e isso não se faz com os
arquivos permitidos (`surface_page.rs` e `surface_page/**`).

- o único dispatcher da página Surface é o do todo, `TodoPage.dispatcher`,
  campo privado de `apps/uigallery/src/views/apps/todo/mod.rs`.
  `TodoPage::dispatch` (`app/semantics.rs:244`) o empresta com o próprio
  `TodoPage` como `Operable`, e o `Dispatcher` valida cada pedido contra a
  projeção desse `Operable`. um `set_value` no campo de intenção, ou um
  `propose`, volta `NotFound`: o todo não projeta esses nós;
- a página não alcança o campo. a privacidade é por módulo, e um
  `impl TodoPage` escrito em `surface_page.rs` também não vê `dispatcher`;
- um segundo `Dispatcher` na página não serve à porta. o
  `ShowcaseView::add_sink` (`view/dispatch.rs`) põe a sink da porta só no
  dispatcher do todo, e o servidor recusa o ato cujo id a sink não ouviu
  ("request N was dispatched, but the app's dispatcher reports no stage",
  `rust/socket/src/server.rs:516`). e os dois dispatchers numeram a partir
  de 1: a porta casa estágio por `RequestId`, então um id do outro
  dispatcher passaria por esse teste com o estágio errado.

opções:

1. um método em `apps/uigallery/src/views/apps/todo/semantics/mod.rs` que
   empresta o dispatcher do todo para um `Operable` maior, o mesmo
   `mem::take` que `dispatch` já faz, umas dez linhas:
   `pub(crate) fn with_dispatcher<R>(&mut self, f: impl FnOnce(&mut Dispatcher, &mut TodoPage, Millis) -> R) -> R`.
   a página monta um `Operable` que segura o todo, o host e o campo:
   projeta o todo, a superfície e os dois nós da página
   (`surface_page.intent`, text_input com `set_value` de texto;
   `surface_page.propose`, button com `propose` lendo
   `surface_page.intent:text`), executa esses dois e passa o resto para
   o `Operable::execute` do todo. o receipt fica onde fica o de todo ato da
   página, a porta ouve, o ator é o de quem pediu. impacto: um arquivo
   fora da lista, nenhuma mudança na projeção do todo, no golden ou no
   caminho de hoje (o clique no todo e o evento da superfície seguem por
   `TodoPage::dispatch`).
2. um dispatcher próprio da página, com a porta ouvindo os dois: muda
   `view/dispatch.rs` (o `add_sink`) e o `wire` (uma sink por dispatcher e
   ids que não colidam). impacto: o protocolo da porta muda por causa de
   uma página de demo. não recomendo.
3. o campo e o botão como controles da página, não capacidades, como o
   controle de forma da fase 2: uma pessoa propõe, um agente não. nenhum
   arquivo fora da lista. impacto: o aceite "um agente propõe pela porta"
   não se cumpre, e o P2 da fase 2 (o controle da página não é
   capacidade) cresce.
4. o todo projetar a intenção e o `propose` como capacidades dele: muda
   `app/semantics.rs`, o golden `showcase-todo.snapshot.json` e o registro
   de que as fixtures verificam, e põe o proponente dentro do app de
   exemplo. não recomendo.

recomendação: 1. os itens 1 a 4, 6 e 7 do briefing não dependem da
escolha (o crate `propose`, o trait, o determinístico, o LLM com o modelo
falso, a medição, o ADR).

decisão necessária: qual opção; na 1, a autorização para o método em
`apps/uigallery/src/views/apps/todo/semantics/mod.rs`.

## o seletor em português (não bloqueia; decidir junto)

opções: (i) o seletor casa com as palavras do registro (sem caixa, acento
e plural), e o que não casa é um erro claro que lista as palavras que o
registro tem; a intenção de exemplo da página usa a palavra do app ("só
active e o campo de adicionar"); (ii) um léxico português e inglês no
proponente (ativo, feito, concluído), que são as palavras do todo com
outro nome, template pela porta dos fundos; (iii) a tradução fica com o
proponente LLM. recomendação: (i) agora, e (iii) na 3b.

## resolução da parada (orquestrador, 2026-09-24)

opção 1: autorizado um método em
`apps/uigallery/src/views/apps/todo/semantics/mod.rs`, só ele, que empresta o
dispatcher do todo ao `Operable` maior da página, pelo mesmo
`mem::take` de `TodoPage::dispatch`; nada mais daquele arquivo nem de
`app.rs` muda (nem a projeção do todo, nem o golden, nem o caminho de
hoje), com um teste de que a projeção e os receipts do todo não mudam
quando o `Operable` da página repassa a ele. o seletor: (i). a palavra
que escolhe casa só com as palavras do app, a que não casa é um erro que
lista as palavras do registro, sem léxico entre línguas no
determinístico; as palavras da gramática dele valem nas duas línguas;
traduzir a palavra da pessoa para a do app é do proponente LLM. nas
páginas e nos testes, intenções cuja palavra existe no app ("only what is
active and the add field"). o resto do briefing fica como estava.

## handoff da fase 3a (2026-09-24)

goal: task-69 fase 3a, proponentes sem provedor: uma intenção em texto
vira um IR proposto que só chega ao runtime verificado.

status: implementado. `cargo test -p offers -p uigallery -p plevir -p
surface` verdes, `./tool/tasks/fullcheck.sh` verde com a perna urnaui, sobre a
`main` @ a678ef4.

### commits

branch `task/69c-proposers`, todos assinados, na ordem do briefing:

- 47ff75d task-69c: a sonda da fase 3a no arquivo da task, e a parada na página
- 8a6123d task-69c: the Proposer trait and the deterministic proposer by rules
- 605e235 task-69c: the language-model proposer behind a Model trait, proven with a scripted model
- fb4ca07 task-69c: the Surface page proposes a surface from an intent, through the todo's dispatcher
- e o commit do ADR e deste handoff

### a sonda

acima, antes do código ("sonda da fase 3a"), com a parada e a resolução.
em uma linha: o IR de "só o que está ativo e o campo de adicionar",
escrito à mão sobre o registro da projeção real do todo, verifica, e
cada ligação sai de role, `ValueKind`, `ParamSpec` e parent, sem nome do
todo; a condição de parada não disparou. os três limites dela ficaram no
código como estão lá: a palavra que escolhe é do app, "ativo como não
concluído" não é fato do registro, role e parent vêm do snapshot.

### onde mora e por quê

crate novo, `rust/offers` (`uir`, `sem`, `serde_json`, `thiserror`;
nunca engine, comps ou rede). um proponente não é parte do IR: o `uir`
continua a língua e o verifier, compila para wasm e não faz I/O, e a fase
3b vai pôr um provedor atrás do `Model`, que não pode morar lá. as
entradas do `README.md` (a lista de crates e uma frase) e do
`docs/ATLAS.toml` (`[crates.propose]`) foram no commit do trait; o
`arc_sync_guard` passa.

a API:

- `Ask { intent, caps, app, current, policy }`: a intenção, o
  `Capabilities`, o `Snapshot` (roles, parents, labels, valores, que o
  registro não carrega), o `VerifiedIr` da superfície mostrada e a
  `GenerationPolicy` sob a qual a proposta será verificada. a policy não
  estava na lista do briefing; entrou porque o laço do LLM verifica dentro
  dele, e tem de verificar sob a mesma policy de quem chama;
- `trait Proposer { fn propose(&mut self, &Ask) -> Result<Ir, ProposeError> }`;
- `propose_verified(&mut proposer, &Ask) -> Result<VerifiedIr, ProposeError>`:
  a única saída. um IR que o verifier recusa sai como
  `ProposeError::Refused` com todos os erros, nunca como IR (teste
  `propose_verified_never_returns_what_the_verifier_refuses`);
- `ProposeError`: `NotUnderstood`, `NoMatch { words, app }`,
  `Nothing { part, missing }`, `Empty`, `Model`, `Exhausted { attempts, errors }`,
  `Refused`. o texto de cada um é o que a página mostra.

### o vocabulário do proponente determinístico (`Rules`)

a gramática, nas duas línguas porque é dele (as palavras são dobradas:
sem caixa, sem acento, sem o s do plural):

| palavra | o que faz | a regra sobre o registro |
|---|---|---|
| add, adicionar, adiciona, new, novo, entry, entrada | a entrada | nó `text_input` com um estado de texto e uma ação de parâmetro text: um `text_field` ligado aos dois; a ação sem parâmetro do mesmo nó é o `submit` do campo e o `activate` de um botão; sem ela, um botão ao lado (mesmo parent) com uma ação só, sem parâmetro. o label do botão: o da superfície atual, o do botão ao lado, a palavra da intenção, ou o nome da ação |
| filter, filtro, filtrar | as escolhas | tabs, options ou radios cuja única ação não tem parâmetro: um chip por opção, sob o parent |
| counts, count, counter, stats, contagem, contador, number, número, total | os números | cada estado `int` ou `num`: um `stat`, com a chave como label (o label do nó, `2 active · 1 done`, muda com o valor) |
| compact, compacto, "one column", "uma coluna" | a forma compacta | uma coluna: entrada, escolhas, números |
| two columns, duas colunas, columns, colunas | duas colunas | grid sobre `field_min_w`: escolhas e números numa coluna, a entrada na outra |
| everything, tudo | tudo | todas as partes |
| only, just, só, apenas, artigos, "field", "campo" | nada | a gramática pula |

qualquer outra palavra tem de ser do app (label, nome ou chave de id,
chave de estado) e escolhe: a parte que a intenção nomeia fica só com o
que a palavra nomeia (tudo, se ela não nomeia nada da parte); a parte que
ela não nomeia mostra só o que a palavra nomeia. sem forma na intenção,
fica a forma da superfície atual (duas colunas quando a raiz é um grid).
uma intenção só de forma remonta o que a superfície mostra.

o que ele responde quando não entende, nunca um IR vazio:

- nenhuma palavra que as regras leem (`""`, "the only field"):
  `the rules do not understand "...": they read add, filter, counts (or stats), compact, two columns and everything (adicionar, filtro, contagem, compacto, duas colunas, tudo), and words the app uses`;
- uma palavra que não é da gramática nem do app ("só o que está ativo e
  o campo de adicionar", "make it purple"):
  `"ativo" is not a word of the app; its words are active, all, completed, done, filter, input, new, todo`;
- uma parte que o app não tem ("filter" no workbench do ide):
  `a choice (filter) asked for, and the app declares no choice (tabs, options or radios whose action takes no parameter)`;
- um app sem nada disso: `Empty`.

continuidade: sobre a superfície atual, o id da superfície fica (a página
transforma), e cada nó cuja capacidade continua fica com o id e o label
dela; um container fica com o id do container atual que segurava pelo
menos metade do que ele segura, de dentro para fora. o que é novo se
chama `<superfície>.<o nó do app que mostra>` e nunca toma um id que a
superfície atual usa para outra coisa. as regras só ligam o que o autor
pode pedir e nunca uma ação destrutiva.

a mesma regra lê o workbench do ide (teste `the_same_rules_read_the_ide`):
"add" liga `commit.message` e o botão "Commit" ao lado (o campo não tem
ação sem parâmetro), "counts" liga `diff:count`, "filter" é `Nothing`.

### o laço do LLM (`Llm`)

- `trait Model { fn complete(&mut self, prompt: &str) -> Result<String, ModelError> }`,
  síncrono como o resto de uma proposta;
- `Llm::ATTEMPTS` = 3 (a primeira resposta e dois consertos; o verifier
  dá todos os erros de uma vez, cada um no seu ponteiro, então um
  conserto costuma bastar, e o terceiro limita o custo de um modelo que
  não converge). depois, `Exhausted` com os erros da última. modelo que
  falha para na hora (`Model`);
- a resposta é lida como o objeto dentro dela (do primeiro `{` ao último
  `}`, porque modelo embrulha em prosa e cerca), decodificada por
  `uir::parse` e verificada; recusada, o prompt seguinte é o primeiro mais
  a resposta e cada erro com ponteiro, razão e regra;
- o prompt leva: a intenção; as oito regras do verifier (um `match`
  exaustivo em `uir::Rule`, então uma regra nova tem de ser descrita) e o
  autor e os limites da policy; cada componente com role, como se nomeia,
  o estado que mostra e os eventos que manda (de `Component::ALL` e do
  `Spec`); o registro legível, nó a nó com role, label e parent, cada
  estado com kind e valor atual, cada ação com o que toma, destrutiva,
  confirmação, quem pode pedir, desabilitada agora; a superfície atual,
  o id a manter, cada nó ligado com o que liga (os ids que devem
  continuar) e o IR dela; e o JSON schema (`uir::schema()`). uns 18 KB
  sobre o todo;
- provado com modelos que respondem de um roteiro: um que erra na
  primeira e acerta na segunda (o segundo prompt tem a resposta e cada
  ponteiro e razão do verifier), um que erra sempre (três perguntas,
  nenhuma quarta, `Exhausted` com os erros da terceira), um que falha. o
  teste `the_prompt_carries_the_schema_the_registry_and_the_ids_to_keep`
  confere o schema inteiro, cada estado e ação do registro e cada id a
  manter.

um prompt de exemplo, sobre o todo com a superfície compacta, intenção
"only what is active and the add field" (cortado: os componentes, os
itens 2 e 3, o IR atual e o schema aparecem inteiros no prompt):

```text
you propose one surface of a plev app as a plev IR document: JSON that describes the surface by meaning, by binding the app's own state and actions to components of a closed vocabulary. the runtime verifies it before anything is shown; a document the verifier refuses comes back to you with every error. answer with the IR only: one JSON object, no prose.

## the intent

only what is active and the add field

## the verifier's rules

1. schema: the JSON has the shape of the schema below, no field it lacks, and `plev_ir` is 1.
2. identity: every `sem` id is valid (...) and unique on the surface, the surface's id included; `[{item}]` only inside a repeat's template, once per id; a node that takes the id of an app node sends only that node's actions.
(... 3 a 8 ...)

this proposal is verified under: author agent:unknown, at most 256 nodes, depth 16, 256 characters per title, label or description.

## the components

- (no component): a layout container, role group; holds children.
- button: role button; needs a label; shows busy (bool); sends activate (nothing).
- text_field: role text_input; needs a label; shows text (text); sends set_value (text), submit (nothing).
(...)

## the app's capabilities

todo (region) "todo"
todo.input (text_input) "new todo", in todo
  state todo.input:text: text = ""
  action todo.input set_value: takes a text
  action todo.input submit: takes nothing, disabled now
todo.filter (tab_list) "filter", in todo
  state todo.filter:selected: text = "all"
todo.filter[active] (tab) "Active", in todo.filter
  action todo.filter[active] select: takes nothing
todo.item[1] (list_item) "Absorb the plev demos into the showcase", in todo
  state todo.item[1]:completed: bool = false
  action todo.item[1] toggle: takes nothing
  action todo.item[1] remove: takes nothing, destructive
todo.counter (status) "2 active · 1 done", in todo
  state todo.counter:active: int = 2
  state todo.counter:done: int = 1
(...)

## the surface shown now

its id is `todo_panel`: keep it, and the surface changes form in place.
keep the id of every node whose capability stays on the surface, so what a person holds in it (focus, the caret, a drag) goes on; a new id is a node that leaves and another that arrives:
- `todo_panel.input`: text_field, text <- todo.input:text, set_value -> todo.input set_value, submit -> todo.input submit
- `todo_panel.add`: button, activate -> todo.input submit
- `todo_panel.filter.active`: chip, activate -> todo.filter[active] select
- `todo_panel.active`: stat, value <- todo.counter:active
(...)
its IR:
{"plev_ir":1,"surface":{"id":"todo_panel", ...}}

## the JSON schema of the IR

{ ... uir::schema(), pretty ... }
```

depois de uma recusa, o prompt ganha:

```text
## attempt 1 was refused

your answer:

<a resposta, até 16 000 caracteres>

the verifier's errors, every one, each at its JSON pointer into your IR:
- /surface/root/children/2/children/0/bind/state/value: the app declares no state `todo.counter:gone` (rule 4, state)
- (...)

answer again with the whole corrected IR, one JSON object.
```

### a página

`apps/uigallery/src/views/apps/surface/mod.rs` e o módulo novo
`surface_page/proposing.rs`. acima da superfície, sob a legenda: o campo
de intenção (`surface_page.intent`, text_input com `set_value`), o botão
"propose" (`surface_page.propose`, ação `propose` que lê a intenção,
desabilitada com a intenção em branco) e a linha de mensagem
(`surface_page.status`, status com o estado `message`), dentro da região
`surface_page`; abaixo, o controle segmentado da fase 2 e a superfície.
Enter no campo propõe, Esc tira o foco.

o caminho: o clique, o Enter, o ato de um agente pela porta e o de um
leitor de tela são um `Request` sobre a página inteira, pelo dispatcher
do todo, emprestado por `TodoPage::with_dispatcher` ao `Page`, um
`Operable` que projeta o todo, a superfície e a linha, executa `set_value`
e `propose` da linha e passa o resto ao `execute` do todo. o `propose`
chama `Rules` por `propose_verified` sobre a projeção do todo, o registro
dela e o `VerifiedIr` mostrado, com o autor de quem pediu
(`GenerationPolicy::by`), e a superfície se transforma no lugar
(`SurfaceHost::transform`; `open` quando nenhuma estava aberta). a
mensagem diz "proposed by the rules for "..."" ou o erro, em vermelho
(`theme.colors.danger`), e a legenda da metade gerada passa a dizer "the
same capabilities, proposed by the rules for the intent above". um clique
no controle segmentado volta a uma forma escrita à mão.

`TodoPage::with_dispatcher` é a única mudança fora da lista de arquivos
(autorizada): dezessete linhas em `app/semantics.rs`, o mesmo `mem::take`
de `dispatch`, sem import novo, sem mudar a projeção do todo, o golden ou
o caminho de hoje. o teste
`the_page_operable_forwards_to_the_todo_as_the_todo_runs_it` roda seis
pedidos do todo (`set_value`, `submit`, `select`, `toggle`, `remove`, um
item que não existe) pelo caminho do todo num todo e pelo `Operable` da
página noutro: mesmos ids, mesmos pedidos (fora a revisão observada, que
na página cobre mais nós), mesmos estágios e resultados, e a mesma
projeção no fim.

render headless, 1200x800, core, escala 1, pelo teste
`surface_page::tests::forms::the_page_draws_headless_after_a_proposal`
(com `SURFACE_PAGE_OUT=<dir>` escreve `surface-proposed.png` e `.json`):
à esquerda o todo escrito à mão, igual; à direita, sob "the same
capabilities, proposed by the rules for the intent above", o campo com
"only what is active and the add field", o botão propose, a linha
"proposed by the rules for "only what is active and the add field"", o
controle compact | two columns, e o card da superfície com o campo "new
todo" e o add desabilitado, um chip só, "Active", e um stat só, "2
active". em 390x844 (hoff, `cargo run -p uigallery -- --render Surface
--width 390 --height 844 --theme hoff`), a linha de intenção fica sob o
todo e acima do controle, o placeholder "what should it show?" cabe no
campo. o `--render Surface` de 1200x800 mostra agora 26 nós (revisão
2922fe5faa6fe87e): os 22 de antes e os 4 da linha.

a projeção depois da proposta (os 10 nós do todo como antes; os 13 da
superfície e da linha, forma lean):

```json
[
  { "id": "todo_panel", "role": "region", "label": "quick todo" },
  { "id": "todo_panel.card", "role": "group", "label": "todo", "parent": "todo_panel" },
  { "id": "todo_panel.entry", "role": "group", "label": "", "parent": "todo_panel.card" },
  { "id": "todo_panel.input", "role": "text_input", "label": "new todo", "parent": "todo_panel.entry",
    "state": { "text": { "kind": "text", "value": "" } },
    "actions": [
      { "name": "set_value", "reads": ["todo_panel.input:text"], "param": { "kind": "text", "constrained": true } },
      { "name": "submit", "enabled": false, "reads": ["todo_panel.input:text"] } ],
    "constraints": { "editable": true }, "focusable": true },
  { "id": "todo_panel.add", "role": "button", "label": "add", "parent": "todo_panel.entry",
    "actions": [ { "name": "activate", "enabled": false, "reads": ["todo_panel.input:text"] } ],
    "enabled": false, "focusable": true },
  { "id": "todo_panel.filter", "role": "group", "label": "filter", "parent": "todo_panel.card" },
  { "id": "todo_panel.filter.active", "role": "button", "label": "Active", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ], "focusable": true },
  { "id": "todo_panel.counts", "role": "group", "label": "", "parent": "todo_panel.card" },
  { "id": "todo_panel.active", "role": "label", "label": "active", "parent": "todo_panel.counts",
    "state": { "value": { "kind": "int", "value": 2 } } },
  { "id": "surface_page", "role": "region", "label": "propose a surface" },
  { "id": "surface_page.intent", "role": "text_input", "label": "intent", "parent": "surface_page",
    "state": { "text": { "kind": "text", "value": "only what is active and the add field" } },
    "actions": [ { "name": "set_value", "reads": ["surface_page.intent:text"], "param": { "kind": "text", "constrained": true } } ],
    "constraints": { "editable": true }, "focusable": true },
  { "id": "surface_page.propose", "role": "button", "label": "propose", "parent": "surface_page",
    "actions": [ { "name": "propose", "reads": ["surface_page.intent:text"] } ], "focusable": true },
  { "id": "surface_page.status", "role": "status", "label": "proposal", "parent": "surface_page",
    "state": { "message": { "kind": "text", "value": "proposed by the rules for \"only what is active and the add field\"" } } }
]
```

os ids que continuaram são os da fixture da task-68; `todo_panel.filter.all`,
`todo_panel.filter.completed` e `todo_panel.done` saíram.

### a medição

headless, build debug (a do gate), apple m4, rustc 1.98.1, o todo com a
superfície compacta, intenção "only what is active and the add field",
com `cargo test -p uigallery --lib surface_page::tests::measure --
--ignored --nocapture`, três execuções. "as regras" é `propose_verified`
com `Rules` até o `VerifiedIr`; "o pedido" é o `propose` de uma pessoa
inteiro (observado, pelo dispatcher, proposto, verificado, a superfície
transformada); "cena" soma o frame seguinte da página inteira montado num
`Compositor`; "pixels" soma esse frame desenhado pelo caminho da janela
num alvo fora da tela e lido de volta. o contexto de GPU já existe, como
num app rodando.

| passo | primeira rodada | melhor de 20 |
| --- | --- | --- |
| intenção até o `VerifiedIr` (as regras) | 0,23 a 0,34 ms | 0,16 ms |
| o pedido inteiro | 0,54 a 0,55 ms | 0,48 ms |
| até o primeiro frame, cena | 0,77 a 0,80 ms | 0,66 a 0,68 ms |
| até o primeiro frame, pixels | 32 a 33 ms (o primeiro texto e as pipelines do processo) | 4,2 ms |

### contagem do gate

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ a678ef4 (`cargo test --workspace` e o `cargo test` do urnaui neste worktree, antes do primeiro código) | 2347 passam, 12 ignorados | 82 passam, 1 ignorado | 2429 |
| depois, esta branch @ fb4ca07 (`./tool/tasks/fullcheck.sh`, saída 0) | 2368 passam, 0 falhas, 13 ignorados | 82 passam, 0 falhas, 1 ignorado | 2450 |

21 testes novos e uma medição ignorada: 14 no `propose` (2 unitários da
gramática, 8 das regras, 4 do LLM) e 7 no showcase (5 da linha de
proposta, 1 da porta, 1 do render headless). a contagem não cai. clippy
`-D warnings`, `cargo fmt --check` e o check wasm do showcase com `-D
warnings` verdes, dentro do gate. `df -h /` antes do gate: 13 GB livres.

### testes

- `rust/offers/src/words.rs` (2): a dobra das palavras; a gramática
  nas duas línguas;
- `rust/offers/tests/rules.rs` (8): cada intenção da gramática, em
  inglês e português, verifica sobre o todo, sem superfície e sobre a da
  página, e liga o que pediu e nada mais; os ids do que fica são os da
  superfície atual (da compacta para a proposta, e das duas colunas para
  a compacta); proposta sobre proposta mantém os ids; o workbench do ide;
  intenção não entendida é erro claro; só o que o autor pode pedir; a
  mesma pergunta dá o mesmo IR; `propose_verified` nunca devolve o que o
  verifier recusa;
- `rust/offers/tests/llm.rs` (4): o modelo que erra primeiro é
  consertado pelos erros do verifier no segundo prompt; o que erra sempre
  esgota as três tentativas; o que falha para na hora; o prompt leva o
  schema, os componentes, as regras, o registro e os ids a manter;
- `surface_page/tests/proposals.rs` (5): uma pessoa propõe e a
  superfície muda no lugar, o texto digitado continua e o caret está onde
  estava (o mesmo widget); uma intenção não entendida deixa a superfície
  e diz por quê, no status e no receipt; o `Operable` da página repassa ao
  todo como o caminho do todo; a linha cabe num celular; os nós da linha
  são da página;
- `surface_page/tests/agent.rs` (+1): pela porta real, sem janela, um
  agente escreve no campo da superfície, põe a intenção, observa e
  propõe; a superfície toma a forma proposta com o mesmo id e o texto
  "milk" continua no campo; "make it purple" falha com o motivo no
  status; o receipt é do agente, no dispatcher do todo;
- `surface_page/tests/forms.rs` (+1): a página headless depois de uma
  proposta, cena, projeção e pixels;
- `surface_page/tests/measure.rs` (ignorado): a medição.

### validação executada

- `cargo test -p offers -p uigallery -p plevir -p screen` (pelo gate);
- `cargo clippy -p offers -p uigallery --all-targets -- -D warnings`;
- `RUSTFLAGS="-D warnings" cargo check --target wasm32-unknown-unknown -p uigallery`;
- `./tool/tasks/fullcheck.sh`, saída 0, com a perna urnaui;
- `SURFACE_PAGE_OUT=<dir> cargo test -p uigallery --lib surface_page::tests::forms`,
  `cargo run -p uigallery -- --render Surface` (1200x800 e 390x844), e a
  medição.

### linha de ADR

novo, proposto: [`0052-proposers-propose-the-runtime-verifies.md`](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/0052-proposers-propose-the-runtime-verifies.md).
linha do índice para o orquestrador (o [`adr/`](../../adr/adr.md) é proibido
aqui):

`| 2026-09-24 | [proposers-propose-the-runtime-verifies](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/0052-proposers-propose-the-runtime-verifies.md) | proposed | crate propose (uir, sem, serde_json; sem engine, comps ou rede): trait Proposer sobre Ask (intenção, Capabilities, Snapshot, VerifiedIr mostrado, GenerationPolicy) devolve Ir não verificado; propose_verified é a única saída, VerifiedIr ou os erros; Rules, determinístico, gramática pequena em inglês e português (add, filter, counts, compact, two columns, everything) ligada por estrutura do registro, nunca por nomes do app, palavras que escolhem só do app (NoMatch lista as dele), ids do que fica mantidos; Llm atrás de Model, prompt com regras, componentes, registro, superfície atual e ids a manter e o schema, erros do verifier de volta, 3 tentativas, Exhausted; a página Surface propõe pelo dispatcher do todo (TodoPage::with_dispatcher); o provedor é a fase 3b (task-69) |`

### decisões

- a policy no `Ask`. contexto: o laço do LLM verifica dentro dele.
  decisão: `Ask` leva a `GenerationPolicy`. razão: o laço e quem chama
  verificam sob a mesma; consequência: a página passa o autor de quem
  pediu, e uma superfície proposta por um agente só liga o que agentes
  podem pedir;
- `Rules` sem léxico entre línguas (decisão do orquestrador); a
  gramática é bilíngue, a palavra que escolhe é do app;
- ids de container por quanto do novo o atual segurava (pelo menos
  metade), de dentro para fora. razão: estreitar uma parte ("só active")
  mantém o container dela; consequência: das duas colunas para a
  compacta, os números tomam o id `todo_panel.side`, a coluna que os
  segurava lá;
- o label de um nó que continua é o da superfície atual (o "add" da
  fixture fica "add");
- o controle segmentado da fase 2 fica. depois de uma proposta, ele
  continua nomeando a última forma escrita à mão, e um clique volta a
  ela;
- a linha de mensagem reserva uma linha, para que a primeira mensagem
  não mova a superfície; uma mensagem longa quebra e empurra.

### limitações

- a gramática é pequena de propósito; fora dela é erro, não palpite. uma
  palavra em português sobre um app em inglês é recusada (o `NoMatch`
  lista as palavras do app);
- "os itens ativos" como lista não sai do registro: o todo não projeta a
  lista dos itens e diz `completed: bool`, não "ativo" (o P2 da fase 1,
  no backlog). as regras não mostram itens;
- `Model` é síncrono e `Ask` empresta; um provedor de verdade não roda
  assim na thread do app (fase 3b);
- o `Llm` não confere continuidade: um modelo que renomeia o id de uma
  capacidade que continua passa no verifier, e o widget dela se perde;
- a proposta roda dentro do `execute`, na thread da página. com as
  regras custa menos de um milissegundo; com um modelo não pode;
- o verifier segue aceitando props que o widget ignora (fase 1).

### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima), o
  [`changelog.md`](../../CHANGELOG.md) (a página Surface propõe a partir de uma intenção),
  e na entrada `[crates.showcase]` do `docs/ATLAS.toml` a linha de
  proposta da página e a dependência `propose` (só a entrada do crate
  novo era permitida);
- `TodoPage::with_dispatcher` é `pub(crate)`: outro `Operable` do crate
  pode pedir o dispatcher do todo; o nome e o comentário dizem para quê;
- a página inclui as duas fixtures por `include_str!`, como antes; a
  proposta não depende delas (lê o registro vivo).

### o que a fase 3b (provedor real) precisa decidir e ligar

- o provedor (modelo local ou API), atrás de `propose::Model`, num crate
  ou feature próprio para que `propose` continue puro;
- rodar o modelo fora da thread do app: o `propose` da página responde
  `Execution::Started` e termina com `Dispatcher::complete` ou `fail`
  quando o modelo responde (o dispatcher e a porta já seguem estágios
  que terminam depois); `Ask` empresta, então o trabalhador precisa de
  dados próprios (o snapshot, o registro e o IR atual clonados);
- onde o prompt se divide em sistema e usuário para uma API de chat, e se
  o provedor aceita decodificação restrita pelo `uir::schema()`;
- o tempo máximo de espera, o que a página mostra enquanto o modelo
  pensa, e se a proposta anterior pode ser cancelada;
- credenciais e custo: onde a chave mora, e quanto custa cada tentativa
  (até três por proposta, uns 18 KB de prompt sobre o todo, mais a
  resposta e os erros nas seguintes);
- se uma proposta que renomeia o id de uma capacidade que continua volta
  ao modelo como erro brando (continuidade não é regra do verifier);
- qual proponente a página usa: as regras primeiro e o modelo quando
  elas não entendem, ou só o modelo.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0052: proposers propose, the runtime verifies](../../adr/Synthesis/0052-proposers-propose-the-runtime-verifies/0052-proposers-propose-the-runtime-verifies.md)
- [task-68: plev IR and verifier](../task-68-plev-ir-and-verifier/task-68-plev-ir-and-verifier.md)
- [task-69: ephemeral and generated surfaces](task-69-ephemeral-generative-surfaces.md)
- [task-95: the todo projects its items as a list](../task-95-the-todo-projects-its-items-as-a-list/task-95-the-todo-projects-its-items-as-a-list.md)
<!-- /related -->
