---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
task: task-68
wave: 6
depends: [task-57, task-58]
branch: task/68-plev-ir
---

# task-68: plev IR e verifier

## decisões do orquestrador antes do briefing (2026-09-24)

a onda 5 fechou com os critérios essenciais (programa, seção 6), e a
onda 6 abre com esta task. a sonda nas premissas desta task achou:

- **o nome do crate é `uir`**, como sugerido. é curto como os outros
  (`sem`, `wire`, `rec`) e não confunde com `crates/parser/src/ir.rs`.
- **o vocabulário v1 é o que existe no comps ou no engine**, conferido
  no código:
  - no comps: `Button`, `IconButton`, `TextField`, `Checkbox`, `Switch`,
    `Slider`, `Select`, `Tabs`, `List` (`VirtualList`), `Table`, `Card`,
    `Stat`, `Chart` (tipos linha, barra, área e rosca, de
    `comps::charts`), `Separator`, `Dialog` (`Modal`), `EmptyState`,
    `Progress` (`ProgressBar`), `Badge` e `Chip`;
  - no engine: `Text` e `Image`.

  `Media` (vídeo) e `Toolbar` não têm equivalente e ficam fora: uma barra
  de ferramentas é um `Stack` de botões, e vídeo entra por ADR quando o
  comps tiver o componente.
- **as fixtures ficam no domínio que tem capacidades de verdade.** a
  revisão de silêncios da tese depende de vídeo e de capacidades
  (`detect_silence`, `apply_cut`) que nenhum app do plev tem, então sai
  das fixtures desta task. as três fixtures são:
  - o formulário de commit do ide, a tela fixa;
  - uma superfície gerada no ide, "revisar o que vai no commit": a lista
    de arquivos com stage e unstage, a mensagem e o commit, sobre as
    capacidades que o `Workbench` projeta;
  - uma superfície do todo do showcase.

  o domínio do checkpoint b (vídeo ou ide) se decide na task-69; se
  houver trade-off estratégico, é pergunta para o brenner.
- **o registro de capacidades sai da projeção.** o `sem::Snapshot` de um
  app já declara estado (`StateRef`, `ValueKind`) e ações (`ActionDecl`
  com `ParamSpec` e `Policy`). o registro se constrói dele e não é uma
  segunda declaração à mão, que envelheceria.
- **repetição ligada a lista:** a forma mínima que a sonda pede é um nó
  que repete um molde para cada item de um `List`/`Set` do estado, com o
  `SemId` do molde levando o item (`changes.file[{item}]`). se precisar de
  mais que isso, o agente para e reporta, como a task já diz.



uma representação intermediária própria, tipada e verificável, que
descreve uma superfície de interface com significado (identidade, role,
estado ligado, ações ligadas a capacidades) e com aparência só por tokens
e primitivas de layout permitidas. humanos, LLMs, importers e ferramentas
produzem IR; o verifier decide o que pode existir; só IR verificado chega
ao runtime. o modelo nunca ganha acesso arbitrário ao runtime.

## ligação com a tese

seções 2 (o modelo propõe, o runtime verifica, o plev executa), 3 (plev
IR: não AST de Rust, não DOM, não HTML virtual; preserva significado), 15
(interface generativa precisa de restrições: componentes permitidos, ações
possíveis, design tokens, constraints, layout primitives, capabilities,
políticas de accessibility, permissões; program synthesis, typed IR,
verification, execution). goal 3 do programa.

## estado atual no código

- `crates/parser/src/ir.rs`: `ParserNode { tag, props, children }`, IR de
  geração de código (props já normalizadas para métodos do builder,
  valores literais ou expressões de token em Rust). é IR de compilação
  para código, não de runtime, e não carrega semântica.
- `crates/engine/src/builder`: árvore declarativa (`div`, `text`,
  `button`, `path`, `image`) com layout taffy, suportada para protótipo
  (`docs/ADR/official-app-pattern.md`).
- `crates/comps`: o vocabulário real de componentes (catálogo em
  `docs/catalog.md`, seção 0).
- `engine::theme`: todos os tokens (`colors`, `glass`, `typography`,
  `spacing`, `shape`, `control`, `size`, `duration`, `layout`, `shadows`).
- depois de 57 e 58: `SemId`, `Role`, `Value`, `Constraints`,
  `ActionDecl`, `Policy`.

## decisão de arquitetura (vira ADR)

1. crate novo e puro (nome sugerido `uir`; o orquestrador fecha), depende
   de `sem` e de serde. não depende de engine nem de comps: o
   vocabulário de componentes e tokens é declarado como dado, e a
   materialização (task-69) liga ao comps.
2. formato de texto JSON, porque é o que LLM produz melhor, com JSON schema
   exportado para decoding restrito. versão no topo (`"plev_ir": 1`).
3. relação com os IRs existentes, escrita no ADR: `ParserNode` continua
   sendo o IR de geração de código do parser; o plev IR é o IR de runtime;
   a task-71 faz o parser emitir plev IR. o builder não é o alvo de
   materialização por padrão (decisão da task-69).

## escopo

### o IR

```text
Ir {
  plev_ir: 1,
  surface: { id: SemId, title, root: IrNode },
}

IrNode {
  sem: SemId,
  component: Component,          // vocabulário fechado, espelho de comps
  label, description,
  props: Props,                  // só tokens tipados, nunca cor ou px cru
  layout: Layout,                // Stack { axis, gap, align } | Grid { min_col, gap }
                                 // | Split { ratio } | Overlay; tamanho por
                                 // min, max, flex
  bind: {
    state: { prop -> StateRef },         // valor vem do estado do app
    actions: { event -> CapabilityRef }, // ação vem de uma capacidade do app
  },
  children: [IrNode],
}
```

- `Component`: `Button`, `IconButton`, `TextField`, `Checkbox`, `Switch`,
  `Slider`, `Select`, `Tabs`, `List`, `Table`, `Card`, `Stat`, `Chart`
  (kind), `Media` (vídeo, imagem), `Text`, `Separator`, `Toolbar`,
  `Dialog`, `EmptyState`, `Progress`. adicionar componente exige que ele
  exista no comps.
- `Props` tipados por token: `SpacingToken`, `TypographyRole`,
  `ColorRole`, `Elevation`, `ControlSize`, `ShapeRole`. nenhum número de
  pixel e nenhuma cor literal passam.

### registro de capacidades

o app declara o que existe, o espaço formal da geração:

```text
Capabilities {
  data:    [ { ref: StateRef, kind: ValueKind, label } ],
  actions: [ { target: SemId, action: ActionName, param: ParamSpec, policy: Policy, label } ],
}
```

o IR só referencia. não existe forma de o IR criar capacidade.

### verifier

`verify(ir, caps, policy) -> Result<VerifiedIr, Vec<IrError>>`. só o
verifier constrói `VerifiedIr` (newtype sem construtor público). checa:

1. schema e versão;
2. `SemId` válidos e únicos na superfície;
3. componente conhecido; props só com tokens; layout com primitivas
   permitidas;
4. todo estado ligado existe em `caps.data` e o tipo bate com o
   componente;
5. toda ação ligada existe em `caps.actions`; o parâmetro bate com o
   `ParamSpec`;
6. policy de geração: ação destrutiva não pode ser ligada a controle
   primário sem confirmação; um agente não pode ligar capacidade que a
   `Policy` não permite a agentes;
7. accessibility: todo componente interativo tem label; ordem de foco
   derivável; nenhum controle só visual sem role;
8. limites: número máximo de nós, profundidade, tamanho de texto (evita
   superfície patológica gerada por modelo).

erro com caminho no IR (JSON pointer) e motivo, todos de uma vez, como o
droplist do parser.

### fixtures

IR golden de três superfícies: a revisão de silêncios da tese, a
comparação original contra resultado, e o formulário de commit do ide
(prova de que uma tela fixa também cabe no IR).

## fora de escopo

materializar (69), transformar (70), importar de outra fonte (71), chamar
LLM.

## dependências

57 e 58 integradas.

## superfície sob responsabilidade

o crate novo; entradas em `Cargo.toml`, `docs/arc/arc.toml`, `README.md`;
proposta de ADR `docs/ADR/plev-ir-typed-and-verified.md`.

## interfaces que consome

`sem` (57, 58).

## interfaces que pode alterar

as próprias.

## contrato exposto

`Ir`, `IrNode`, `Component`, `Props`, `Layout`, `Capabilities`,
`verify`, `VerifiedIr`, `IrError`, o JSON schema. invariantes: `VerifiedIr`
só existe se passou em todas as checagens; o IR nunca contém literal
visual; toda ação de um `VerifiedIr` é uma capacidade declarada.

## critérios de aceitação

- as três fixtures verificam com o registro de capacidades de cada uma.
- cada checagem do verifier tem um IR inválido de teste com o erro e o
  caminho esperados.
- roundtrip serde; schema exportado validando as fixtures.
- `./script/gate` verde; crate compila em wasm.

## testes esperados

um teste por regra do verifier, proptest de IR aleatório (o verifier nunca
entra em panic e sempre devolve `Ok` ou erros), roundtrip.

## riscos e perguntas abertas

- vocabulário de componentes fechado demais trava geração útil; aberto
  demais vira HTML. começar fechado e crescer por ADR.
- ligação de dados com listas dinâmicas (os 17 silêncios): o IR precisa de
  repetição ligada a um `List`/`Set` do estado. se o desenho simples não
  couber, reportar antes de inventar linguagem de template.

## commits esperados

1. `task-68: crate do IR com tipos e serde`
2. `task-68: registro de capacidades`
3. `task-68: verifier com erros por caminho`
4. `task-68: regras de policy e accessibility`
5. `task-68: fixtures golden e JSON schema`

## pare e reporte se

- repetição ligada a lista exigir mini linguagem;
- algum componente do vocabulário não tiver equivalente no comps.

## handoff (task-68)

### sonda (antes do primeiro tipo, 2026-09-24)

lida no worktree `task/68-plev-ir` sobre `main` @ 8a7a0ea.

(a) o vocabulário v1, o tipo que existe e o `Role` que o `project` dele
declara. nove dos 21 não têm `project` (o comps não projeta conteúdo
sem ação); para esses o role sai da tabela do `uir`, marcado "uir".

| componente | tipo | onde | role e o que o `project` declara |
|---|---|---|---|
| `button` | `Button` (e `SubmitButton`) | `crates/comps/src/action/button.rs:74`, `submit_button.rs` | `Button`; `activate`. o `SubmitButton` tem `busy` e o `activate` lê `busy` |
| `icon_button` | `IconButton` | `crates/comps/src/action/icon_button.rs:47` | `Button`; `activate`; na variante toolbar, `active` e o `activate` lê `active` |
| `text_field` | `TextField` | `crates/comps/src/form/text_field.rs:42` | `TextInput`; `text`; `set_value` com `ParamSpec { Text, constrained }`, editable |
| `checkbox` | `Checkbox` | `crates/comps/src/form/checkbox.rs:22` | `Checkbox`; `checked`; `toggle` lê `checked` |
| `switch` | `Switch` | `crates/comps/src/form/switch.rs:27` | `Toggle`; `on`; `toggle` lê `on` |
| `slider` | `Slider` | `crates/comps/src/form/slider.rs:20` | `Slider`; `value` (num) com range e step; `set_value` com `ParamSpec { Num, constrained }`; `increment` e `decrement` só com step |
| `select` | `Select` | `crates/comps/src/form/select.rs:68` | `Select`; `selected` (text), `expanded`; `select` com `ParamSpec { Text, constrained }`, `open`, `close`; aberto, filhos `Option` `<id>[i]` com `select` |
| `tabs` | `Tabs` | `crates/comps/src/nav/tabs.rs:24` | `TabList`; `selected` (a chave da aba); filhos `Tab` `<id>[i]` com `select` sem parâmetro |
| `list` | `VirtualList` | `crates/comps/src/content/list.rs:38` | `List`; `count`, `selected`; filhos `ListItem` `<id>[i]` com `select` |
| `table` | `Table` | `crates/comps/src/content/table.rs:87` | `Table`; `count`, `selected_row`; filhos `Row` `<id>[i]` com `select` |
| `card` | `Card` | `crates/comps/src/content/card.rs:140` | sem `project`; uir: `Group` |
| `stat` | `Stat` | `crates/comps/src/content/stat.rs:26` | sem `project`; uir: `Label` |
| `chart` (line, bar, area, donut) | `LineChart`, `BarChart`, `StackedArea`, `Donut` | `crates/comps/src/charts/{line,bars,area,donut}.rs` | sem `project`; uir: `Image` |
| `separator` | `Separator` | `crates/comps/src/content/separator.rs:10` | sem `project`; uir: sem role (decorativo, não é controle) |
| `dialog` | `Modal` | `crates/comps/src/feedback/modal.rs:43` | `Dialog` nomeado pelo título, descrito pelo corpo; `confirm` (destrutivo quando o intent é) e `cancel`; filhos `Button` `<id>[confirm]` e `<id>[cancel]` |
| `empty_state` | `EmptyState` | `crates/comps/src/content/empty_state.rs:22` | sem `project`; uir: `Status` |
| `progress` | `ProgressBar` | `crates/comps/src/feedback/progress.rs:12` | sem `project`; uir: `ProgressBar` |
| `badge` | `Badge` | `crates/comps/src/action/badge.rs:27` | sem `project`; uir: `Label` |
| `chip` | `Chip` | `crates/comps/src/action/chip.rs:25` | `Toggle` com `selected` e `toggle` quando selecionável; `Button` com `activate` quando interativo; `Label` quando estático |
| `text` | `engine::builder::text` | `crates/engine/src/builder/element/mod.rs:98` | sem `project`; uir: `Label` |
| `image` | `engine::builder::image` | `crates/engine/src/builder/element/mod.rs:137` | sem `project`; uir: `Image` |

(b) os papéis de token do theme por categoria de `Props`.

| categoria | onde | papéis |
|---|---|---|
| cor | `ColorTokens`, `crates/engine/src/theme/tokens.rs:200` (`theme.colors`) | `bg`, `surface`, `bg_panel`, `bg_hover`, `text`, `text_dim`, `text_mid`, `accent`, `accent_dim`, `success`, `danger`, `warning`, `info`, `divider`, `border_active`. o `theme.glass` (`tokens.rs:226`) é receita de superfície, não papel de cor, e fica fora do v1 |
| tipografia | `TypographyScale`, `tokens.rs:27` (`theme.typography`, um método por estilo) | `h4`, `headline`, `title`, `base_r`, `base_m`, `base_2r`, `base_2m`, `base_2sm`, `base_2b`, `caption_r`, `caption_sm`, `small_r`, `small_sm`, `body`, `body_2r`, `body_2m`, `body_2b`, `hairline`, `mono`, `readout` |
| espaçamento | `SpacingScale`, `tokens.rs:9` (`theme.spacing`) | `xs`, `sm`, `md`, `lg`, `xl`, `xxl` |
| forma | `ShapeTokens`, `crates/engine/src/theme/scales.rs:22` (`theme.shape`) | `pill`, `dropdown`, `tabs`, `card`, `block`, `item`, `nav`, `cluster`, `tooltip`, `micro` |
| tamanho de controle | `ControlSize`, `scales.rs:88` (alturas em `theme.control.heights`) | `xs`, `sm`, `md`, `lg`, `xl` |
| elevação | `Elevation`, `scales.rs:505` (`theme.shadows`) | `base`, `raised`, `floating`, `overlay`, `hint` |
| tamanho de container (layout `min`, `max`, `min_col`) | `SizeTokens`, `scales.rs:259` (`theme.size`) | `menu_w`, `modal_max_w`, `sidebar_w`, `sidebar_rail_w`, `card_w`, `card_min_w`, `readable_max_w`, `field_min_w`, `toast_w`, `tooltip_max_w`, `dropdown_max_h` |

(c) as capacidades, lidas de projeções reais por dois testes novos que
projetam, imprimem (`--nocapture`) e gravam o snapshot:
`crates/ide/tests/uir_fixture.rs` (um repositório real num temp dir, três
arquivos mudados, `src/lexer.rs` no index, o formulário de commit aberto
com "fix parser" escrito por uma pessoa pelo dispatcher) e
`crates/showcase/tests/uir_fixture.rs` (a página Todo encenada headless,
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
- o briefing proíbe `README.md` e `docs/arc/arc.toml`, mas o
  `crates/engine/tests/arc_sync_guard.rs` exige que todo membro do
  workspace apareça nos dois, e o programa (seção 8) diz que o agente que
  cria um crate adiciona a própria entrada. hoje o `arc.toml` passa por
  acaso (contém "acquire", que contém "uir"); o `README.md` não. com
  `crates/uir` no workspace, o teste do guard fica vermelho até alguém
  pôr o crate no README. isso não bloqueia os tipos; vira uma condição de
  parada no gate (reportada no fim).
