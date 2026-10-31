---
id: task-57
title: "task-57: semantic contract (crate `sem`)"
project: plev
type: task
status: done
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [semantics, agents, determinism]
relations: [adr-0032, generative-runtime-thesis, task-58, task-59, task-60, task-61, task-63, task-65, task-68, task-73]
evidence: [860b029]
source: docs/log/steps/task-57-semantic-contract.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-57: semantic contract (crate `sem`)

Records the task that created the plev semantic contract in the `sem` crate (now `accord`, in `rust/accord`): pure, with no GPU dependency, with `SemId`, `Value`, `Constraints`, `ActionDecl`, `Node`, `Snapshot`, a stable `Revision` and `diff`. Semantics starts as a projection of the app state, on demand, and not as a tree accumulated per frame. It was integrated into `main` at `860b029`, with `fullcheck.sh` green and the workspace test count going from 1467 to 1545; tasks 58 to 61 and 63 consume this contract so that agents operate on identity and meaning, not on visual position.

## fechamento (2026-09-23)

integrada na `main` em `860b029` (oito commits, de `35f6b10` a `860b029`),
validada pelo orquestrador: diff inteiro lido, `./tool/tasks/fullcheck.sh` verde na
branch (a perna do urnaui pula no worktree), 78 testes do `sem` com serde
e 72 sem, 7 proptests, contagem do workspace de 1467 para 1545 (exatamente
os testes novos). ADR: [`0032-semantic-contract-as-pure-projection.md`](../../adr/semantics/0032-semantic-contract-as-pure-projection.md).

o que ficou diferente da spec abaixo, aceito como contrato:

- o builder aceita id, chave, nome de ação e ref tipados ou como texto
  (`impl TryInto<T, Error: Into<SemError>>`); texto inválido vira erro
  agregado em `finish`, nunca panic. `node(id: SemId, ...)` continua
  compilando.
- `NodeBuilder::action(name)` devolve um `ActionBuilder` (`label`,
  `enabled`, `reads`, `param`, `policy`) e encerra a cadeia do nó.
- revisão e igualdade seguem a árvore: raízes na ordem de projeção, depois
  cada subárvore em profundidade; estado ordenado por chave. a ordem entre
  irmãos muda a revisão; a intercalação de subárvores sem relação e a
  ordem de declaração das chaves, não. a codificação é `sem.revision.v1`,
  documentada em `rust/accord/src/revision.rs`.
- `Change` tem doze variantes: as sete da spec mais `RoleChanged`,
  `DescriptionChanged`, `ConstraintsChanged`, `Reordered { parent }` e
  `FocusChanged`, de modo que `diff` é vazio exatamente quando os dois
  snapshots são iguais (proptest).
- `Revision` imprime e trafega como 16 dígitos hex minúsculos.
- `secret` guarda só tamanho: `Text` vira `Int(chars)`, `List` e `Set`
  viram `Int(itens)`; `Bool`, `Int` e `Num` ficam.
- defaults de nó: habilitado, não inerte, `focusable` igual a
  `role.is_interactive()`.

limitações declaradas no handoff, que seguem para as próximas tasks:

- a forma JSON não tem golden (sem `serde_json` nos dev-deps); a task-58
  pode adicionar `serde_json` como dev-dependency do `sem`.
- `Value::Num` com NaN ou infinito não trafega em JSON.
- foco em nó não focável ou inerte não é rejeitado.
- id duplicado: o segundo nó sai e as checagens dele não rodam.
- o custo de projetar sob demanda não foi medido (tasks 59 e 73).

## objetivo

dar a todo controle uma identidade lógica persistente, um significado, um
estado, as ações que aceita e as constraints que o limitam, como dado
puro, independente de como ele é desenhado. é a resposta, em tipos, às
perguntas do contrato da tese (seção 30): quem sou, o que significo, qual
é meu estado, quais ações aceito, quais condições as tornam válidas, quem
pode executá-las.

## ligação com a tese

seções 6 (a mesma interface para humano e máquina; a mesma estrutura não
é a mesma árvore literal), 7 (identidade não depende da posição), 9
(invalidação semântica), 30 (o contrato que importa). o mantra: a
capacidade mantém identidade e semântica enquanto a representação visual
nasce, some ou se transforma.

## estado atual no código

- a única identidade que existe é `ViewId(pub u64)`, gerada por um
  contador que volta a zero em `InputState::begin_frame`
  (`rust/engine/src/control/input/mod.rs:50-61`). ela é posicional: o terceiro
  controle emitido é `ViewId(2)` neste frame e pode ser outro no próximo.
- nenhum widget de `rust/engine/src/widgets` declara o que é. o estado vive em campos
  privados (`TextField`, `Checkbox`, `Switch`) ou no app
  (`FileEntry.staged` em `apps/idestudio/src/interfaces/panels/unassigned.rs:51-55`).
- a árvore de accessibility é derivada de geometria, sem label
  (`rust/engine/src/runtime/window/render.rs:99-125`).
- `engine::actions` (`rust/engine/src/control/keymap/mod.rs`) resolve tecla em
  nome de ação (`"commit::Submit"`). ele continua existindo: é binding de
  input. o contrato desta task é a capacidade operável que o binding pode
  alvejar (a ponte é da task-59).

## decisão de arquitetura (recomendada; o orquestrador confirma no briefing)

1. crate novo `rust/accord`, puro: sem wgpu, winit, cosmic-text, accesskit.
   motivo pela regra de crate boundary: dependências próprias e mínimas,
   testável isolado, e consumido por quem não pode puxar o stack de GPU
   (agente, recorder, ferramenta headless, CLI).
2. semântica é projeção pura do estado do app. o app implementa
   `Semantics::project(&self, &mut SnapshotBuilder)`, chamada sob demanda:
   no frame (accessibility, task-60), na observação do agente (task-61) e na
   validação de uma ação (task-58), sempre a partir do estado atual. não é
   acumulada por frame dentro do render.
3. a identidade é escolhida pelo app ou pelo IR (task-68). nunca por
   contador, nunca por posição.
4. o que não é projetado não existe para o contrato. hover, animação e
   posição não entram na projeção, então não podem invalidar nada. essa é a
   invalidação semântica por construção.

## escopo

### `SemId`, a identidade lógica

- forma canônica textual: segmentos separados por `.`. segmento de nome
  `[a-z0-9_]+`. segmento de chave entre colchetes, com texto arbitrário,
  escapando `]` e `\` com `\`. exemplos: `commit.submit`,
  `changes.file[src/lib.rs]`, `silence.segment[3].preview`.
- `SemId::parse(&str) -> Result<SemId, SemError>`, `SemId::name(&str)`,
  `.child(&str)`, `.key(impl Into<String>)`, `Display` com a forma
  canônica, `parent_path()` apenas como texto (não implica parent na
  árvore).
- barato de clonar (`Arc<str>` ou equivalente), `Eq`, `Ord`, `Hash`,
  `Send + Sync`.

### `Role`

enum fechado, mapeável para AccessKit na task-60 sem que `sem` dependa de
accesskit: `Window`, `Group`, `Region`, `Toolbar`, `Button`, `Toggle`,
`Checkbox`, `Radio`, `TextInput`, `Slider`, `Select`, `Option`, `List`,
`ListItem`, `Table`, `Row`, `Cell`, `TabList`, `Tab`, `TabPanel`, `Tree`,
`TreeItem`, `Dialog`, `Menu`, `MenuItem`, `Label`, `Image`, `Link`,
`ProgressBar`, `Status`, `Alert`, `Canvas`, `Media`. role nova só com
aprovação do orquestrador. `Role::is_interactive()` define quais exigem
label.

### `Value`, `StateKey`, `StateRef`

- `Value`: `Bool`, `Int(i64)`, `Num(f64)`, `Text(String)`,
  `List(Vec<String>)` (ordem importa), `Set(BTreeSet<String>)` (ordem não
  importa). igualdade de `Num` por bits (`to_bits`), documentada: o
  contrato é determinístico, não aproximado.
- `StateKey`: nome `[a-z0-9_]+`.
- `StateRef { node: SemId, key: StateKey }`, forma textual
  `<semid>:<key>`, por exemplo `changes:staged`, `commit.message:text`.
  parse e display com roundtrip.

### `Constraints`

`editable: bool`, `required: bool`, `range: Option<(f64, f64)>`,
`step: Option<f64>`, `max_len: Option<usize>`,
`options: Option<Vec<String>>`, `secret: bool`. `secret` marca campo cujo
valor nunca sai do processo (senha, token): a projeção guarda só
`len` e o recorder (task-63), o agente (task-61) e a accessibility
(task-60, role de senha) respeitam isso.

### `Policy` e `ActionDecl`

- `Policy { actors: Actors, confirm: Confirm, destructive: bool }`.
  `Actors` diz quem pode pedir (human, agent, assistive, test).
  `Confirm`: `Never`, `ForAgents`, `Always`. a aplicação da policy é da
  task-58; aqui é dado.
- `ActionDecl { name: ActionName, label: Option<String>, enabled: bool,
  reads: Vec<StateRef>, param: Option<ParamSpec>, policy: Policy }`.
  `ActionName` segue `[a-z0-9_]+`. `reads` lista o estado de que a ação
  depende: é a lista que a precondição compara (task-58). `ParamSpec` diz o
  tipo do `Value` esperado e se as `Constraints` do nó se aplicam.

### `Node`

`id: SemId`, `role: Role`, `label: String`, `description: Option<String>`,
`parent: Option<SemId>`, `state: IndexMap<StateKey, Value>` (ordem de
projeção), `actions: Vec<ActionDecl>`, `constraints: Constraints`,
`enabled: bool`, `inert: bool` (efetivo, herdado: um nó é inerte se ele ou
um ancestral é; modal aberto deixa o fundo inerte), `focusable: bool`,
`source: Option<Source>` (arquivo, linha, coluna do site de projeção via
`#[track_caller]`; alimenta a task-65).

### `SnapshotBuilder`, `Semantics`, `Snapshot`

```rust
pub trait Semantics {
    fn project(&self, b: &mut SnapshotBuilder);
}

impl SnapshotBuilder {
    pub fn new() -> Self;
    #[track_caller]
    pub fn node(&mut self, id: SemId, role: Role, label: impl Into<String>) -> NodeBuilder<'_>;
    pub fn focus(&mut self, id: SemId);
    pub fn finish(self) -> Result<Snapshot, SemErrors>;
}
// NodeBuilder: parent, description, state, action, constraints,
// enabled, inert, focusable (encadeáveis)
```

`Snapshot`:

- `nodes` em `IndexMap<SemId, Node>`, na ordem de projeção
  (determinística);
- `focused: Option<SemId>`;
- `revision(&self) -> Revision`: hash estável e documentado do conteúdo
  semântico (FNV-1a 64 implementado no crate, ou outro hash estável
  justificado; nunca `FxHash`, porque a revisão atravessa processo:
  agente e recorder). exclui `source`;
- `get`, `children` (em ordem), `roots`, `value(&StateRef)`,
  `subtree(&SemId) -> Snapshot` (observação por escopo);
- `Snapshot::diff(before, after) -> Vec<Change>`.

### `Change`

`NodeAdded(SemId)`, `NodeRemoved(SemId)`, `StateChanged { at: StateRef,
before: Option<Value>, after: Option<Value>, added: Vec<String>,
removed: Vec<String> }` (`added`/`removed` preenchidos para `Set` e
`List`), `LabelChanged`, `ActionsChanged`, `FlagsChanged` (enabled, inert,
focusable), `ParentChanged`. é o delta que a task-58 devolve numa rejeição
("removed: B") e o que a task-61 manda como observação incremental.

### `SemError` e `SemErrors`

thiserror. `InvalidId`, `InvalidKey`, `DuplicateId`, `UnknownParent`,
`ParentCycle`, `DanglingRead { action, read }`, `MissingLabel(SemId)` para
role interativa sem label, `DuplicateAction`, `InvalidConstraint`,
`UnknownFocus`. `finish` devolve todas as violações de uma vez
(`SemErrors(Vec<SemError>)`), como o droplist do parser: quem projeta vê
tudo que está errado, não só o primeiro erro. a lib nunca entra em panic
por input.

### serde

`Serialize`/`Deserialize` em todos os tipos públicos, atrás da feature
`serde` (ligada por padrão). nomes de campo explícitos. assim as tasks 61
e 63 não precisam mexer nos tipos depois.

## fora de escopo

`Request`, `check`, `Receipt`, `Dispatcher` (task-58). ligação com
`ViewId`, bounds, layer (task-59). mapeamento AccessKit (task-60).
protocolo e socket (task-61). qualquer mudança em `rust/engine`,
`rust/engine/src/widgets` ou apps.

## dependências

nenhuma.

## superfície sob responsabilidade

- `rust/accord/**` (novo)
- `Cargo.toml` da raiz: entrada em `members` e em
  `[workspace.dependencies]`
- `docs/ATLAS.toml`: `[crates.sem]`
- `README.md`: o crate na tabela e na linha de crates
- proposta de ADR [`0032-semantic-contract-as-pure-projection.md`](../../adr/semantics/0032-semantic-contract-as-pure-projection.md) no
  handoff (o orquestrador integra e atualiza o índice)

## interfaces que consome

só dependências já presentes em `[workspace.dependencies]`: `thiserror`,
`indexmap`, `serde`; `proptest` em dev. dependência nova precisa de
justificativa e aprovação.

## interfaces que pode alterar

apenas as próprias, novas.

## contrato exposto

tipos: os da seção escopo.

invariantes:

1. `SemId::parse(&x.to_string()) == Ok(x)` para todo `x`; nunca vazio;
   ordem total.
2. a mesma sequência de chamadas ao builder produz a mesma `Snapshot` e a
   mesma `Revision`, em qualquer execução e alvo (wasm incluso).
3. `Revision` ignora `source`.
4. `inert` é herdado de qualquer ancestral.
5. toda role interativa tem label não vazia.
6. toda `reads` aponta para nó e chave existentes no mesmo snapshot.
7. valor de campo `secret` nunca aparece no snapshot.

ownership: `Snapshot` é valor imutável, `Clone + Send + Sync`. o app é
dono do estado; `sem` nunca guarda referência a ele.

lifecycle: um snapshot vive o quanto o consumidor quiser. nada é
invalidado por frame.

error model: `SemErrors` agregado em `finish`; `SemError` nas funções de
parse. nada de panic.

compatibilidade: pré-1.0, mas a forma textual de `SemId` e `StateRef` e a
representação serde são contrato com agentes (61) e recorder (63): mudança
exige ADR.

## critérios de aceitação

- `cargo test -p accord`, `cargo clippy -p accord --all-targets -- -D warnings`,
  `cargo check -p accord --target wasm32-unknown-unknown`, `./tool/tasks/fullcheck.sh`.
- zero `unwrap`/`expect` fora de teste.
- `arc_sync_guard` passa com o crate registrado.
- doc test com o exemplo da tese: uma struct `GitState { staged:
  BTreeSet<String>, message: String }` implementando `Semantics` e
  projetando `changes` (state `staged`), `commit.message` (state `text`) e
  `commit.submit` (ação `submit` com `reads` `changes:staged` e
  `commit.message:text`).
- ADR proposto.

## testes esperados

- `SemId`: roundtrip com proptest; rejeição de vazio, maiúscula, `..`,
  chave sem fechamento; escape de `]` e `\`.
- `StateRef`: roundtrip e rejeições.
- builder: cada `SemError`; vários erros reportados juntos; ordem de
  `children`; `subtree`; `inert` herdado; `secret` sem valor.
- `Revision`: igual para projeções iguais; muda com state, label,
  `enabled` de ação, constraints, parent e ordem; não muda com `source`.
  um teste golden fixa o valor numérico de uma revisão conhecida, para
  pegar mudança acidental do hash.
- `diff`: `Set` `{A,B,C}` para `{A,C}` dá `removed: [B]`; `List`; escalar;
  nó adicionado e removido; flags.

## riscos e perguntas abertas

- projeção grande (editor com milhões de linhas, diff enorme): a regra
  vai no ADR: projetar o que é operável e significativo (o documento como
  um nó com tamanho, seleção e cursor), não o conteúdo inteiro. escopo de
  observação é da task-61.
- custo de projetar sob demanda: medido na task-59 e na task-73.
- `#[track_caller]` aponta para o site de projeção, não para o render. a
  task-65 decide se basta.

## commits esperados

1. `task-57: crate sem com SemId e forma canônica`
2. `task-57: Value, StateRef, Constraints, Policy e ActionDecl`
3. `task-57: Node, SnapshotBuilder e validação com erros agregados`
4. `task-57: Snapshot, Revision estável e subtree`
5. `task-57: diff entre snapshots`
6. `task-57: registrar sem em arc.toml e README`

cada commit com os testes do que introduz.

## pare e reporte se

- a forma textual de `SemId` precisar de outro caractere ou outra regra;
- uma role, um tipo de `Value` ou um campo de `Node` parecer faltar;
- o hash estável exigir dependência nova.
