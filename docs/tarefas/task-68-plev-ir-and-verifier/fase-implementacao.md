---
id: task-68/fase-implementacao
title: "task-68 fase implementação: sonda, handoff e fechamento"
project: plev
type: secao
parent: task-68
status: referencia
date: 2026-09-24
date_basis: texto
topics: [ir, superficies-geradas, semantica]
relations: [adr-0049, task-65, task-69, task-91]
evidence: [871f0dd, 08dd8f8, 12e38d3, d6c66ad, cd9c183, 7380294, 2427a55, 454fe6d, 7721adb, 5350b47, 2461bb5]
source: docs/log/steps/task-68-plev-ir-and-verifier.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# handoff (task-68)

Reúne a sonda das premissas, feita antes do primeiro tipo sobre `main` @ 8a7a0ea, o handoff da implementação do crate `uir` e o fechamento do orquestrador. O handoff descreve o IR, o registro de capacidades, as oito regras do verifier com seus erros por JSON pointer, as fixtures, os 61 testes novos (gate em 2338) e a parada no `arc_sync_guard`, resolvida pela opção 2. O fechamento confirma a integração na `main` em `871f0dd` e registra uma pendência P2 para a task-69 e uma P3 na mensagem da regra 6.

## sonda (antes do primeiro tipo, 2026-09-24)

lida no worktree `task/68-plev-ir` sobre `main` @ 8a7a0ea.

(a) o vocabulário v1, o tipo que existe e o `Role` que o `project` dele
declara. nove dos 21 não têm `project` (o comps não projeta conteúdo
sem ação); para esses o role sai da tabela do `uir`, marcado "uir".

| componente | tipo | onde | role e o que o `project` declara |
|---|---|---|---|
| `button` | `Button` (e `SubmitButton`) | `rust/engine/src/widgets/buttons/button.rs:74`, `submit_button.rs` | `Button`; `activate`. o `SubmitButton` tem `busy` e o `activate` lê `busy` |
| `icon_button` | `IconButton` | `rust/engine/src/widgets/buttons/icon_button.rs:47` | `Button`; `activate`; na variante toolbar, `active` e o `activate` lê `active` |
| `text_field` | `TextField` | `rust/engine/src/widgets/form/text_field.rs:42` | `TextInput`; `text`; `set_value` com `ParamSpec { Text, constrained }`, editable |
| `checkbox` | `Checkbox` | `rust/engine/src/widgets/form/checkbox.rs:22` | `Checkbox`; `checked`; `toggle` lê `checked` |
| `switch` | `Switch` | `rust/engine/src/widgets/form/switch.rs:27` | `Toggle`; `on`; `toggle` lê `on` |
| `slider` | `Slider` | `rust/engine/src/widgets/form/slider.rs:20` | `Slider`; `value` (num) com range e step; `set_value` com `ParamSpec { Num, constrained }`; `increment` e `decrement` só com step |
| `select` | `Select` | `rust/engine/src/widgets/form/select/mod.rs:68` | `Select`; `selected` (text), `expanded`; `select` com `ParamSpec { Text, constrained }`, `open`, `close`; aberto, filhos `Option` `<id>[i]` com `select` |
| `tabs` | `Tabs` | `rust/engine/src/widgets/nav/tabs.rs:24` | `TabList`; `selected` (a chave da aba); filhos `Tab` `<id>[i]` com `select` sem parâmetro |
| `list` | `VirtualList` | `rust/engine/src/widgets/content/list.rs:38` | `List`; `count`, `selected`; filhos `ListItem` `<id>[i]` com `select` |
| `table` | `Table` | `rust/engine/src/widgets/content/table.rs:87` | `Table`; `count`, `selected_row`; filhos `Row` `<id>[i]` com `select` |
| `card` | `Card` | `rust/engine/src/widgets/content/card.rs:140` | sem `project`; uir: `Group` |
| `stat` | `Stat` | `rust/engine/src/widgets/content/stat.rs:26` | sem `project`; uir: `Label` |
| `chart` (line, bar, area, donut) | `LineChart`, `BarChart`, `StackedArea`, `Donut` | `rust/engine/src/widgets/charts/{line,bars,area,donut}.rs` | sem `project`; uir: `Image` |
| `separator` | `Separator` | `rust/engine/src/widgets/content/separator.rs:10` | sem `project`; uir: sem role (decorativo, não é controle) |
| `dialog` | `Modal` | `rust/engine/src/widgets/feedback/modal.rs:43` | `Dialog` nomeado pelo título, descrito pelo corpo; `confirm` (destrutivo quando o intent é) e `cancel`; filhos `Button` `<id>[confirm]` e `<id>[cancel]` |
| `empty_state` | `EmptyState` | `rust/engine/src/widgets/content/empty_state.rs:22` | sem `project`; uir: `Status` |
| `progress` | `ProgressBar` | `rust/engine/src/widgets/feedback/progress.rs:12` | sem `project`; uir: `ProgressBar` |
| `badge` | `Badge` | `rust/engine/src/widgets/content/badge.rs:27` | sem `project`; uir: `Label` |
| `chip` | `Chip` | `rust/engine/src/widgets/content/chip.rs:25` | `Toggle` com `selected` e `toggle` quando selecionável; `Button` com `activate` quando interativo; `Label` quando estático |
| `text` | `engine::builder::text` | `rust/engine/src/compose/builder/element/mod.rs:98` | sem `project`; uir: `Label` |
| `image` | `engine::builder::image` | `rust/engine/src/compose/builder/element/mod.rs:137` | sem `project`; uir: `Image` |

(b) os papéis de token do theme por categoria de `Props`.

| categoria | onde | papéis |
|---|---|---|
| cor | `ColorTokens`, `rust/engine/src/styling/theme/tokens.rs:200` (`theme.colors`) | `bg`, `surface`, `bg_panel`, `bg_hover`, `text`, `text_dim`, `text_mid`, `accent`, `accent_dim`, `success`, `danger`, `warning`, `info`, `divider`, `border_active`. o `theme.glass` (`tokens.rs:226`) é receita de superfície, não papel de cor, e fica fora do v1 |
| tipografia | `TypographyScale`, `tokens.rs:27` (`theme.typography`, um método por estilo) | `h4`, `headline`, `title`, `base_r`, `base_m`, `base_2r`, `base_2m`, `base_2sm`, `base_2b`, `caption_r`, `caption_sm`, `small_r`, `small_sm`, `body`, `body_2r`, `body_2m`, `body_2b`, `hairline`, `mono`, `readout` |
| espaçamento | `SpacingScale`, `tokens.rs:9` (`theme.spacing`) | `xs`, `sm`, `md`, `lg`, `xl`, `xxl` |
| forma | `ShapeTokens`, `rust/engine/src/styling/theme/scales.rs:22` (`theme.shape`) | `pill`, `dropdown`, `tabs`, `card`, `block`, `item`, `nav`, `cluster`, `tooltip`, `micro` |
| tamanho de controle | `ControlSize`, `scales.rs:88` (alturas em `theme.control.heights`) | `xs`, `sm`, `md`, `lg`, `xl` |
| elevação | `Elevation`, `scales.rs:505` (`theme.shadows`) | `base`, `raised`, `floating`, `overlay`, `hint` |
| tamanho de container (layout `min`, `max`, `min_col`) | `SizeTokens`, `scales.rs:259` (`theme.size`) | `menu_w`, `modal_max_w`, `sidebar_w`, `sidebar_rail_w`, `card_w`, `card_min_w`, `readable_max_w`, `field_min_w`, `toast_w`, `tooltip_max_w`, `dropdown_max_h` |

(c) as capacidades, lidas de projeções reais por dois testes novos que
projetam, imprimem (`--nocapture`) e gravam o snapshot:
`apps/idestudio/tests/plevir_fixture.rs` (um repositório real num temp dir, três
arquivos mudados, `src/lexer.rs` no index, o formulário de commit aberto
com "fix parser" escrito por uma pessoa pelo dispatcher) e
`apps/uigallery/tests/plevir_fixture.rs` (a página Todo encenada headless,
como `showcase --render Todo --semantics`). policy omitida é a padrão
(todos os atores, `Confirm::Never`, não destrutiva).

ide (`Workbench`):

| dado (`StateRef`, `ValueKind`) | ações (alvo, ação, `ParamSpec`, `Policy`) |
|---|---|
| `changes:files` list, `changes:staged` set, `changes:selected` text | |
| `changes.file[<path>]:status` text, `:staged` bool, `:unstaged` bool (uma por arquivo: `src/lexer.rs`, `notes.md`, `src/parser.rs`) | `changes.file[<path>]` `stage`, `unstage`, `ignore`, `show_diff` sem parâmetro; `discard` sem parâmetro, `{ confirm: ForAgents, destructive: true }` |
| `diff:target` text, `diff:lines` list, `diff:count` int | |
| `commit.form:open` bool | `commit.form` `open`, `cancel` sem parâmetro |
| `commit.message:text` text | `commit.message` `set_value` com `ParamSpec { Text, constrained: true }` |
| | `commit.submit` `submit` sem parâmetro (lê `changes:staged` e `commit.message:text`) |
| `stacks:branch` text; `stacks.commit[<sha>]:summary` text, `:author` text, `:files` list | `stacks.commit[<sha>]` `show_diff` sem parâmetro |

todo (showcase):

| dado | ações |
|---|---|
| `todo.input:text` text | `todo.input` `set_value` com `ParamSpec { Text, constrained: true }`; `submit` sem parâmetro |
| `todo.filter:selected` text | `todo.filter[all]`, `todo.filter[active]`, `todo.filter[completed]` `select` sem parâmetro |
| `todo.item[1]`, `[2]`, `[3]` `:completed` bool | `todo.item[<id>]` `toggle` sem parâmetro; `remove` sem parâmetro, `{ destructive: true }` (sem confirmação) |
| `todo.counter:active` int, `todo.counter:done` int | |

o que a sonda achou, nenhum item contradiz o arquivo da task nem as
decisões do orquestrador:

- nove componentes do v1 não têm `project` (tabela a). o role deles é
  decisão do `uir`, declarada como dado na tabela do vocabulário, e a
  task-69 projeta com esse role ao materializar.
- o todo não projeta uma `List`/`Set` com os ids dos itens, só os nós
  `todo.item[<id>]`. então não há repetição sobre os itens do todo; a
  fixture do todo liga o campo, o filtro e o contador. o ide tem
  `changes:files` (list), que é o que a repetição mínima usa.
- as abas do comps e o filtro do todo são capacidades por aba, sem
  parâmetro (`todo.filter[active]` `select`). uma aba do IR que emite a
  chave escolhida não casa com elas; a fixture do todo usa botões para o
  filtro (ver a decisão sobre `tabs` no ADR).
- o id de um commit no ide é o sha, que muda com a data do commit; o
  teste do ide fixa a data do primeiro commit do repositório para que o
  snapshot seja o mesmo a cada execução.
- o briefing proíbe `README.md` e `docs/ATLAS.toml`, mas o
  `tool/tests/test_repoaudit.rs` exige que todo membro do
  workspace apareça nos dois, e o programa (seção 8) diz que o agente que
  cria um crate adiciona a própria entrada. hoje o `arc.toml` passa por
  acaso (contém "acquire", que contém "uir"); o `README.md` não. com
  `rust/plevir` no workspace, o teste do guard fica vermelho até alguém
  pôr o crate no README. isso não bloqueia os tipos; vira uma condição de
  parada no gate (reportada no fim).

## handoff (2026-09-24)

goal: task-68, o plev IR e o verifier.

status: implementado, testado, rebaseado na `main` e com o
`./tool/tasks/fullcheck.sh` verde (perna urnaui incluída). a parada no
`arc_sync_guard` (fim deste handoff) foi resolvida pela opção 2, com a
liberação do orquestrador.

commits (`task/68-plev-ir`, rebaseada sobre `main` @ 39a76cf, que contém
141dbd2; todos assinados):

- 08dd8f8 task-68: a sonda no arquivo da task
- 12e38d3 task-68: crate do IR com tipos e serde
- d6c66ad task-68: registro de capacidades
- cd9c183 task-68: verifier com erros por caminho
- 7380294 task-68: regras de policy e accessibility
- 2427a55 task-68: fixtures golden e JSON schema
- 454fe6d task-68: ADR plev IR tipado e verificado (proposto)
- 7721adb task-68: a doc do leitor diz que ele é o único
- 5350b47 task-68: handoff, com a parada no arc_sync_guard
- 2461bb5 task-68: uir in the README and the arc map
- e o commit desta atualização do handoff

(antes do rebase, sobre 8a7a0ea, os mesmos commits eram dca66a7,
50c0495, 26e3aec, 3034a58, 100fa1d, 1a91c3c, f6dcc89, e92b0a9 e
67a0856.)

a sonda: a seção "sonda" acima, escrita antes do primeiro tipo.

### o IR como ficou

o formulário de commit do ide, encurtado (a fixture inteira é
`rust/plevir/tests/fixtures/idestudio-commit-form.ir.json`):

```json
{
  "plev_ir": 1,
  "surface": {
    "id": "commit",
    "title": "commit",
    "root": {
      "sem": "commit.form",
      "label": "commit",
      "props": { "padding": "md", "background": "bg_panel", "shape": "card" },
      "layout": { "stack": { "axis": "vertical", "gap": "sm", "align": "stretch" } },
      "children": [
        { "sem": "commit.message", "component": "text_field", "label": "commit message",
          "bind": { "state": { "text": "commit.message:text" },
                    "actions": { "set_value": { "target": "commit.message", "action": "set_value" } } } },
        { "sem": "commit.submit", "component": "button", "label": "Commit",
          "bind": { "actions": { "activate": { "target": "commit.submit", "action": "submit" } } } }
      ]
    }
  }
}
```

a repetição mínima (da fixture gerada): `"repeat": { "each":
"changes:files", "item": { "sem": "review.file[{item}]", ... } }`, com os
botões do molde ligados a `{ "target": "changes.file[{item}]", "action":
"stage" }`.

interfaces novas (crate `uir`, nenhuma alterada fora dele):

- IR: `Ir`, `Surface`, `IrNode`, `Bind`, `CapabilityRef`, `Repeat`,
  `VERSION`, `ITEM`;
- vocabulário: `Component`, `ChartKind`, `Spec`, `Named`, `StateProp`,
  `Event`, `CONTAINER`, `spec_of`, `role_of`, `is_control`, `names`;
- tokens: `Props`, `SpacingToken`, `TypographyRole`, `ColorRole`,
  `Elevation`, `ControlSize`, `ShapeRole`, `SizeToken`, `Layout`,
  `Axis`, `Align`, `Size`, `MIN_RATIO`, `MAX_RATIO`, `MAX_FLEX`;
- registro: `Capabilities` (`from_snapshot`, `with_data`, `with_action`,
  `data`, `actions`, `datum`, `action`, `nodes`, `has_node`), `DataCap`,
  `ActionCap`;
- leitura e verificação: `parse`, `decode`, `verify`, `verify_json`,
  `VerifiedIr` (`ir`, `focus_order`, `into_ir`; sem construtor público,
  sem `Deserialize`, sem `Default`), `IrError { rule, pointer, reason }`,
  `Rule`;
- `GenerationPolicy { author, max_nodes, max_depth, max_text }`,
  `HARD_DEPTH`; `schema()`.

### as regras do verifier e os erros

todos os erros vêm de uma vez, cada um com a regra, o JSON pointer e o
motivo; `IrError` tem forma serde para voltar a um modelo.

| regra | o que checa | exemplo de erro (pointer: motivo) |
|---|---|---|
| 1 schema | forma do JSON, campo desconhecido, `plev_ir` = 1 | `/surface/root/children/2/onclick`: unknown field `onclick`; `/plev_ir`: version 2; this runtime reads version 1 |
| 2 identity | `SemId` válido e único (o id da superfície conta); `[{item}]` só no molde e uma vez por id; nenhum id fixo é instance de um molde; um nó com id de nó do app só manda as ações desse nó | `/.../sem`: `review.message` is already the id at /surface/root/children/1/sem |
| 3 vocabulary | componente conhecido, props só token, children, layout e repeat só em container, split com dois filhos, ratio 10 a 90, flex 1 a 12, repeat só em list ou container, sem repeat dentro de repeat | `/.../props/color`: unknown variant `#ff0000`; `/.../children`: a button holds no children |
| 4 state | estado ligado existe e o kind é um que o componente mostra; repeat sobre `List` ou `Set`; no molde, a referência casa com todos os nós, de um kind só | `/.../bind/state/text`: `changes:staged` is set; a text_field's `text` takes text |
| 5 action | ação ligada existe e o parâmetro do evento é o do `ParamSpec`; no molde, todo nó que casa declara a ação, igual | `/.../bind/actions/activate`: `set_value` takes a text; a button's `activate` sends none |
| 6 policy | o autor só liga o que ele pode pedir (`Policy::actors`); ação destrutiva só no `confirm` de um dialog, ou com `Confirm::Always` | `/.../bind/actions/activate`: agent:unknown may not bind `reset` on `admin`: its policy leaves agents out |
| 7 access | label em todo controle, dialog, imagem, chart, stat, status e progress (text e badge podem ligar `text`); item repetido é nomeado pelo item (molde sem label); nó sem role sem nome; controle sem ação é só visual; num overlay só uma camada tem controles; título da superfície não vazio | `/.../bind`: a text_field with no action bound is only visual: bind the capability it operates |
| 8 limits | nós (molde conta uma vez), profundidade, tamanho de texto | `/surface/root`: 7 nodes; the policy allows 3 |

a policy padrão (`GenerationPolicy::default()`): autor `agent:unknown`
(o caso mais restrito), 256 nós, profundidade 16, 256 caracteres por
título, label ou descrição. `GenerationPolicy::by(actor)` troca só o
autor. independente da policy, nenhuma árvore é percorrida além de
profundidade 256 (`HARD_DEPTH`).

### o schema

`uir::schema()` (função pública, montada das mesmas tabelas dos tipos) e
o arquivo `rust/plevir/assets/plev-ir-1.json`, que é esse valor escrito.
um teste mantém os dois iguais; regenera com `UIR_BLESS=1 cargo test -p
uir --test fixtures`. nenhuma dependência nova: sem `schemars` e sem
validador de schema. o validador do teste (`rust/plevir/tests/support/jsonschema.rs`)
conhece só as palavras que o schema usa e falha em qualquer outra. o
schema não carrega a gramática de id, state ref e nome de ação (sem
`pattern`, o workspace não tem regex); o leitor checa e aponta.

`proptest` já está em `[workspace.dependencies]`; o `uir` o usa como
dev-dependency, sem dependência nova de workspace.

### as fixtures

em `rust/plevir/tests/fixtures/`:

- `ide-commit-form.ir.json`: a tela fixa do ide, com os ids do ide;
- `ide-review-commit.ir.json`: "review what goes into the commit",
  gerada sobre as capacidades do `Workbench`;
- `showcase-todo.ir.json`: uma superfície do todo (campo, add, filtro em
  chips, contadores em stats);
- `ide-workbench.snapshot.json` e `showcase-todo.snapshot.json`: as
  projeções reais de onde saem os registros.

os snapshots são gravados por `apps/idestudio/tests/plevir_fixture.rs` (um
repositório real, três arquivos mudados, `src/lexer.rs` no index, o
formulário aberto com "fix parser" escrito por uma pessoa pelo
dispatcher, a data do primeiro commit fixa para o sha ser estável) e por
`apps/uigallery/tests/plevir_fixture.rs` (a página Todo encenada
headless). os dois testes falham quando a projeção se afasta do arquivo.
regeneração: `UIR_BLESS=1 cargo test -p idestudio --test uir_fixture`,
`UIR_BLESS=1 cargo test -p uigallery --test uir_fixture`, depois `cargo
test -p plevir` diz se os IR ainda verificam.

### validação e contagem

contagem final, depois do rebase:

| | workspace | perna urnaui | total do gate |
|---|---|---|---|
| antes, `main` @ 39a76cf (`cargo test --workspace` com a `main` em detached HEAD neste worktree) | 2195 passam, 11 ignorados | 82 passam, 1 ignorado (o urnaui não mudou) | 2277 |
| depois, esta branch (`./tool/tasks/fullcheck.sh`, saída 0) | 2256 passam, 0 falhas, 11 ignorados | 82 passam, 0 falhas, 1 ignorado | 2338 |

são 61 testes novos: 59 no `uir` (19 unitários, 1 doctest, 27 de regras,
7 de fixtures, 5 de proptest), 1 no ide, 1 no showcase. a contagem não
cai. o `arc_sync_guard` e os dois `uir_fixture` passam depois do rebase:
a task-91 e a task-65 não mudaram as projeções gravadas.

o registro de antes do rebase, mantido: sobre 8a7a0ea eram 2179 antes e
2239 depois, com 1 falha (o `arc_sync_guard`).

- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt
  --check`, `cargo check --target wasm32-unknown-unknown -p uigallery` e
  `cargo check -p plevir --target wasm32-unknown-unknown` (com `-D
  warnings`): verdes.
- perna urnaui (rodada à mão como o gate roda, com o target dir próprio):
  82 passam, clippy, fmt e wasm verdes. o urnaui não depende do `uir` e
  não mudou.
- `./tool/tasks/fullcheck.sh` (antes da resolução): parava no `arc_sync_guard`.
  depois da resolução e do rebase: verde, todas as pernas.
- proptest também rodado com `PROPTEST_CASES=5000`: verde.

### decisões

- um leitor só. o proptest achou que o `Deserialize` derivado da serde
  aceita uma struct escrita como lista JSON (`"bind": []`), uma forma que
  o schema não tem. o `Deserialize` de `Ir` passa pelo `decode`, os tipos
  do nó derivam só `Serialize`, e a capacidade é lida campo a campo.
- nó sem `component` é container de layout (role `Group`); a barra de
  ferramentas é um stack de botões, como o orquestrador decidiu.
- os nove componentes sem `project` no comps têm role dado pelo `uir`:
  card `Group`, stat `Label`, chart `Image`, separator sem role, empty
  state `Status`, progress `ProgressBar`, badge `Label`, text `Label`,
  image `Image`. o chip toma o role do que liga, como o comps projeta.
- "controle primário" (item 6 da task) ficou como qualquer controle que
  dispara a ação com uma ativação; só o `confirm` de um dialog não é,
  porque ele é a confirmação. uma ação destrutiva só entra ali, ou quando
  a `Policy` do app pede confirmação de todos. o `discard` do ide
  (`ForAgents`) não pode virar um botão direto numa superfície gerada.
- a regra de foco derivável (7) ficou concreta como "num overlay só uma
  camada tem controles": os controles são folhas, então a ordem de árvore
  sempre existe, e o único caso em que ela não segue o que a pessoa vê é
  controle empilhado sobre controle.
- um nó que usa o id de um nó do app só manda as ações desse nó: o IR não
  pode vestir uma capacidade com a identidade de outra.
- `tabs` manda a chave escolhida (`select` com texto). as abas do comps e
  o filtro do todo são uma capacidade por aba, sem parâmetro, e não casam;
  a fixture do todo usa chips.
- o `uir` não entrou em `[workspace.dependencies]`: ninguém o consome
  ainda; a task-69 adiciona `uir = { path = "rust/plevir" }` quando usar.

### limitações

- um molde só verifica enquanto o app projeta pelo menos um nó que casa
  com ele; sobre uma lista vazia o registro não prova a capacidade por
  item.
- o verifier não vê valores: uma `List` com item repetido daria dois
  instances com o mesmo id; a materialização descarta o repetido.
- um `VerifiedIr` vale para o registro de um instante; o dispatcher
  continua checando cada ação quando ela é pedida.
- fora do v1: ícone do `icon_button` (a tabela de ícones é do comps),
  ênfase do botão (as receitas de `theme.button`), abas por capacidade,
  dados de chart como números (o `sem` só tem lista de texto), tabela
  além de `rows`.
- o casamento dos tokens do `uir` com o theme só é garantido quando a
  task-69 fizer o `match` exaustivo; o `uir` não pode depender do engine.
- um campo `secret` entra no registro como o snapshot o guarda (o
  tamanho), então nenhuma superfície o mostra como texto.

### o que a task-69 precisa para materializar

- consumir só `VerifiedIr` (`verify` ou `verify_json`) e o
  `Capabilities::from_snapshot` da projeção viva do app;
- mapear cada `Component` no widget do comps (tabela do `Spec` e a
  sonda), cada token no campo do theme com o mesmo nome, e `Layout`/`Size`
  no layout do engine; os props ausentes são o padrão do widget;
- projetar cada nó com o seu `SemId` e o role de `role_of`, os nove sem
  `project` com o role da tabela, e cada instance de um repeat com o id
  do molde trocando `[{item}]` pelo item (`SemId::key`) e nomeado pelo
  item;
- ligar cada evento à capacidade como um `Request` pelo dispatcher do app
  (a pessoa que clica é `Actor::Human`), nunca por um caminho paralelo;
- decidir o que fazer com um item repetido numa `List` e com uma
  capacidade que sumiu depois da verificação;
- a ordem de foco está em `VerifiedIr::focus_order`.

### dependências pendentes e riscos de integração

- as entradas do `uir` no `README.md` e no `docs/ATLAS.toml` estão
  feitas (2461bb5). fica a linha do índice de ADR, que é do orquestrador:
  `| 2026-09-24 | [plev-ir-typed-and-verified](../../decisoes/geracao/0049-plev-ir-typed-and-verified.md) | proposed | crate uir, puro (sem, serde): o plev IR (id, componente de um vocabulário fechado espelho do comps, props só por token do theme, layout por primitivas, estado e ações ligados a capacidades que o app declara, repetição mínima sobre List/Set com [{item}]); um leitor só com erro por JSON pointer; Capabilities saem do Snapshot; verify com oito regras é o único caminho a um VerifiedIr; JSON schema gerado das mesmas tabelas (task-68) |`
- rebaseada sobre `main` @ 39a76cf (as tasks 91 e 65 e a nota de
  aprendizado sobre este briefing) sem conflito; os testes `uir_fixture`
  do ide e do showcase passam, então as projeções gravadas ainda são as
  de hoje.
- os testes `uir_fixture` do ide e do showcase são guardas: uma mudança
  futura dessas projeções os deixa vermelhos, com o comando para gravar
  de novo.

### parada

problema: o `arc_sync_guard` exige que todo membro do workspace apareça
no `README.md` e no `docs/ATLAS.toml`. o briefing proíbe os dois
arquivos; o programa (seção 8) diz que o agente que cria um crate
adiciona a própria entrada. o `arc.toml` passa por acaso ("acquire"
contém "uir"); o `README.md` falha. é o único vermelho da branch.

opções:

1. o orquestrador põe as entradas na integração. impacto: a branch fica
   vermelha neste teste até lá; nada mais muda.
2. o orquestrador autoriza e eu ponho as duas entradas nesta branch.
   impacto: branch verde; toca dois arquivos do orquestrador.
3. tirar o `uir` do workspace. impacto: quebra a task (o crate precisa
   entrar no workspace e no gate). descartada.

recomendação: 1 ou 2, com o texto abaixo, e uma entrada real no
`arc.toml` em qualquer caso, porque o acaso do "acquire" esconde a
ausência.

- `README.md`, na linha `| crates | crates/ | ... |`: `..., showcase,
  svg, uir, urnaui, wire |`, e depois do parágrafo do `sem`: "`uir` is
  the plev IR: a surface described by meaning (ids, a closed component
  vocabulary, props by theme token, state and actions bound to the
  capabilities an app projects), and `verify`, the only way to a
  `VerifiedIr` ([docs/adr/generated/0049-plev-ir-typed-and-verified.md](../../decisoes/geracao/0049-plev-ir-typed-and-verified.md))."
- `docs/ATLAS.toml`:

  ```toml
  [crates.uir]
  path = "rust/plevir"
  role = "the plev IR (task-68, adr plev-ir-typed-and-verified): Ir, Surface, IrNode with a SemId, a Component of the closed vocabulary (the comps widgets and the builder's text and image, declared as data in Spec), Props by theme token, Layout (stack, grid, split, overlay) and Size by token, Bind of state to StateRefs and of events to CapabilityRefs, Repeat over a List or Set with [{item}] in the template ids; one reader (decode, Ir's Deserialize) with every shape error at its JSON pointer; Capabilities::from_snapshot; verify(ir, caps, GenerationPolicy) -> VerifiedIr or every IrError at once (schema, identity, vocabulary, state, action, policy, access, limits); schema() and schema/plev-ir-1.json; pure, wasm"
  depends = ["sem"]
  ```

decisão necessária: quem escreve as entradas do `uir` no `README.md` e no
`docs/ATLAS.toml`, e em que momento.

### resolução da parada (2026-09-24)

o orquestrador escolheu a opção 2: o erro era do briefing, e a seção 8 do
programa já dizia que o agente adiciona a própria entrada. no commit
2461bb5:

- `README.md`, na tabela de crates: `| crates | crates/ | git, ide,
  inspect, lot, macros, monster, narrate, narrate-macro, parser, prime,
  rec, rope, sem, showcase, svg, uir, urnaui, wire |`. no parágrafo dos
  crates do contrato, depois da frase do `wire`, na mesma linha de "as a
  click.": "`uir` is the plev IR: a surface described by meaning over
  that contract, and only what its verifier accepts reaches the
  runtime." o README cresceu uma linha.
- `docs/ATLAS.toml`: uma entrada `[crates.uir]` antes de
  `[crates.git]`, com `path = "rust/plevir"`, `depends = ["sem"]` e um
  `role` no estilo das vizinhas (o que o IR descreve, o leitor único, o
  registro vindo do snapshot, o `verify` e as oito regras, a policy
  padrão, o schema, as fixtures, puro e wasm). o guard agora acha
  `uir` pela entrada, não pelo "acquire".

outro crate passando por acaso no `arc_sync_guard`: nenhum. conferi os
19 membros do workspace nos dois arquivos: cada um aparece como palavra
inteira (sem letra, dígito, `_` ou `-` colado) no `README.md` e no
`arc.toml`, e cada um tem entrada própria no `arc.toml` (`[crates.<nome>]`
ou o caminho `crates/<nome>` no `path`; `narrate-macro` está no `path`
de `[crates.narrate]`). antes desta mudança o `uir` era o único que só
aparecia dentro de outra palavra. o guard continua testando substring,
então um crate novo cujo nome caiba dentro de uma palavra dos dois
arquivos passaria sem entrada; não mexi nisso.

## fechamento (orquestrador, 2026-09-24)

integrada na `main` em `871f0dd` (onze commits sobre `39a76cf`). o ADR
plev-ir-typed-and-verified foi aceito.

o agente parou uma vez, e o erro foi do briefing: ele proibia o
`README.md` e o `arc.toml`, que o `arc_sync_guard` exige de todo crate
novo. o agente acrescentou as duas entradas, e a do `arc.toml` agora é
uma entrada de verdade, não um acerto por acaso na palavra "acquire".

validação do orquestrador:
- diff lido: a raiz do crate, o verifier, as regras e o suporte de teste;
- gate verde no diretório da task: 2338 passaram, 0 falharam, 12
  ignorados, com a perna urnaui;
- uma sonda à parte, num teste descartável sobre o IR de base dos testes
  de regra, sem entrar na branch. o verifier recusou uma cor crua (`#ff0000`,
  regra 3, com a lista dos papéis de cor), um tamanho em pixel (regra 3),
  uma ação que o app não declara (`force_push`, regra 5), um `discard`
  destrutivo ligado a um botão (regra 6) e dois erros no mesmo documento
  (regras 4 e 7), cada um com o JSON pointer.

severidade do que ficou:
- P2 para a task-69: 9 dos 21 componentes do vocabulário não têm
  `project` no comps (card, stat, chart, separator, empty_state,
  progress, badge, text, image), e o role deles vem da tabela do `uir`;
  o todo não projeta a lista dos itens, então não há repetição sobre
  eles;
- P3: a mensagem da regra 6 diz "the app asks no one to confirm it" quando
  o app pede confirmação só a agentes (`Confirm::ForAgents`); o sentido
  certo é "o app não pede a todo ator". o `arc_sync_guard` compara
  substring.
