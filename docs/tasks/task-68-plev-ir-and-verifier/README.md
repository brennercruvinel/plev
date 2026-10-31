---
id: task-68
title: "task-68: plev IR and verifier"
project: plev
type: task
status: done
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [ir, generated-surfaces, semantics]
relations: [task-57, task-58, task-69, task-70, task-71, adr-0023, adr-0049, architecture-component-catalog]
evidence: [871f0dd]
source: docs/log/steps/task-68-plev-ir-and-verifier.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-68: plev IR and verifier

Introduces plev IR, a typed and verifiable intermediate representation of interface surfaces, in the `uir` crate, together with the verifier, which is the only path to a `VerifiedIr`. The task was integrated into `main` at `871f0dd` with the gate green (2338 tests), and the ADR `plev-ir-typed-and-verified` was accepted. The record fixes the contract for interface generation: humans, LLMs and importers produce IR, the verifier decides what may exist, and only verified IR reaches the runtime.

## decisões do orquestrador antes do briefing (2026-09-24)

a onda 5 fechou com os critérios essenciais (programa, seção 6), e a
onda 6 abre com esta task. a sonda nas premissas desta task achou:

- **o nome do crate é `uir`**, como sugerido. é curto como os outros
  (`sem`, `wire`, `rec`) e não confunde com `rust/import/src/tree.rs`.
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

- `rust/import/src/tree.rs`: `ParserNode { tag, props, children }`, IR de
  geração de código (props já normalizadas para métodos do builder,
  valores literais ou expressões de token em Rust). é IR de compilação
  para código, não de runtime, e não carrega semântica.
- `rust/engine/src/compose/builder`: árvore declarativa (`div`, `text`,
  `button`, `path`, `image`) com layout taffy, suportada para protótipo
  ([`0023-official-app-pattern.md`](../../adr/structure/0023-official-app-pattern.md)).
- `rust/engine/src/widgets`: o vocabulário real de componentes (catálogo em
  [`component-catalog.md`](../../ref/architecture/component-catalog.md), seção 0).
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

o crate novo; entradas em `Cargo.toml`, `docs/ATLAS.toml`, `README.md`;
proposta de ADR [`0049-plev-ir-typed-and-verified.md`](../../adr/generated/0049-plev-ir-typed-and-verified.md).

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
- `./tool/tasks/fullcheck.sh` verde; crate compila em wasm.

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

## Phases

- [probe, handoff and closing](phase-implementation.md): the probe of the premises in the code before the first type, the handoff with the IR, the verifier rules, the fixtures and the stop at `arc_sync_guard`, and the orchestrator's closing.
