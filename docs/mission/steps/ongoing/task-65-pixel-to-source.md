---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
task: task-65
wave: 5
depends: [task-57, task-59]
branch: task/65-pixel-to-source
---

# task-65: pixel para código e inspector de desenvolvimento

## rodada 1 (orquestrador, 2026-09-23)

esta rodada faz as fases a e b (ler: do pixel ao código). a fase c
(gravar no `.rs`) e a fase d (quem leu este token, proposta primeiro)
ficam para depois de validar a e b. o shell do ide não entra nesta
rodada: outra task mexe nele agora.

## objetivo

cmd+clique num controle ou num texto da janela aberta leva à linha do
código que o produziu. o caminho inverso, em casos definidos: editar o
texto (ou a cor) na janela grava a mudança no `.rs` e vira um diff git
normal. e o mapa "quem leu este token" mostra, no showcase, quais tokens
um widget usa e tudo que um token pinta.

## ligação com a tese

seção 13 inteira (view source para pixels, o inverso para literais, quem
leu este token, no replay chegar ao código daquela versão). do arquivo de
concepção: ligação interface para código (estado observado, controle,
identidade, implementação).

## estado atual no código

- hot reload de literal (`crates/engine/src/hot_reload/literals.rs`): no
  start, cada `.rs` vira skeleton (token stream com literais em branco,
  hasheado) mais a lista de literais em ordem; no save, mesmo skeleton
  quer dizer só literal mudou, e o delta é aplicado em `TextNodeKey`,
  `TextMeasurer` e `Compositor::push_to_layer`. limites já documentados:
  literal repetido muda nos dois lugares, string formatada antes de
  aparecer não casa, número é tratado como código.
- `proc-macro2` entra pela feature `hot-reload`
  (`crates/engine/Cargo.toml:35` e `:84`) sem `span-locations`: a lista de
  literais não tem linha nem coluna.
- tudo gated em `#[cfg(hot_reload)]` (build de debug no desktop; release,
  wasm e mobile nunca veem).
- depois de 57 e 59: `Node.source` (site de projeção via
  `#[track_caller]`) e `Compositor::hit` (qual `SemId` está sob o
  cursor).

## escopo

### fase a: controle para código

cmd+clique (dev only) -> `Compositor::hit` -> `SemId` -> `Node.source` ->
abre o editor em arquivo:linha:coluna (`$PLEV_EDITOR`, senão `zed`, senão
`code -g`). se `source` aponta para o site de projeção e não para o render,
o ADR diz se isso basta ou se o widget também registra o próprio site de
render (`#[track_caller]` no `render`/`sem`), e a decisão é tomada com o
orquestrador.

### fase b: literal para código

- ligar `span-locations` no `proc-macro2` da feature de hot reload e
  guardar linha e coluna de cada literal no baseline.
- cmd+clique num texto sem `SemId`: o texto do `TextNodeKey` sob o cursor
  -> literais que casam -> um só: abre; vários: lista para escolher; nenhum
  (string formatada): diz isso.
- cor: a mesma coisa com hex literal.

### fase c: editar e gravar

- edição inline de um texto cujo literal é único: grava no arquivo
  substituindo exatamente o span, só se o arquivo em disco ainda tem o
  mesmo skeleton do baseline (senão recusa e explica); o hot reload aplica;
  o resultado é `git diff` normal.
- color picker para hex literal com a mesma regra.

### fase d: quem leu este token (proposta primeiro)

`Theme` é struct com campos públicos lidos direto: não dá para
interceptar leitura sem mudar a superfície mais compartilhada do engine.
esta fase começa com uma proposta ao orquestrador (formato de
ambiguidade): acessores com registro em dev, índice reverso por valor
(heurístico e ambíguo quando dois tokens têm o mesmo valor), ou tipo proxy
só em dev. nenhuma implementação antes da decisão.

## fora de escopo

inspector de sessão (64); mudança de estrutura de código pela janela (só
literal); release, wasm e mobile.

## dependências

57 e 59 integradas.

## superfície sob responsabilidade

`crates/engine/src/hot_reload/**`, `crates/engine/Cargo.toml` (feature
`span-locations`), o atalho no shell do showcase (dev only), proposta de
ADR `docs/ADR/view-source-for-pixels.md`. `crates/engine/src/theme` só
depois da decisão da fase d.

## interfaces que consome

`Compositor::hit`, `Binding` (59), `Node.source` (57), baseline de
literais (hot reload).

## interfaces que pode alterar

`hot_reload` por inteiro, sob `#[cfg(hot_reload)]`.

## critérios de aceitação

- no showcase em debug: cmd+clique no botão "adicionar" do todo abre o
  site certo; cmd+clique num título estático abre o literal; editar o
  título na janela grava o `.rs` e `git diff` mostra só aquela string.
- build release e wasm sem nenhum símbolo novo (o gate e um `cargo build
  --release -p showcase` conferem).
- escrita recusada quando o arquivo mudou de estrutura desde o baseline
  (teste).
- `./script/gate` verde.

## testes esperados

posição de literal no baseline, casamento texto para literal (único,
vários, nenhum), escrita por span com arquivo inalterado e alterado,
resolução de editor.

## riscos e perguntas abertas

- `span-locations` tem custo de memória no baseline; medir.
- abrir editor externo é efeito colateral: só por ação explícita do
  usuário, nunca automático.

## commits esperados

1. `task-65: cmd+clique do SemId até o site de origem`
2. `task-65: posição de cada literal no baseline do hot reload`
3. `task-65: cmd+clique em texto e cor sem SemId`
4. `task-65: editar na janela e gravar o literal no arquivo`
5. `task-65: proposta de leitura de token` (só o documento)

## pare e reporte se

- a fase d, sempre antes de código;
- `source` do site de projeção não servir e o site de render exigir
  mudança de assinatura em comps.

## rodada 1: handoff do agente (fases a e b)

### a sonda do `source` (antes de qualquer código)

rodado na base (`76f58a0`), sem janela:

```text
cargo run -p showcase -- --render Todo --semantics todo.json --source
wrote todo.json (10 nodes, revision cfc1185078f7c825, Todo / core at 1200x800 scale 1)
```

o `source` de cada nó, e a linha para onde ele aponta
(`crates/showcase/src/view/apps/app/semantics.rs`):

```text
todo                    region     151:11  b.node(&ids.todo, Role::Region, "todo").inert(self.inert);
todo.input              text_input 153:14  .project(b, &ids.input, "new todo", Some(&ids.todo))
todo.filter             tab_list   157:11  b.node(&ids.filter, Role::TabList, "filter")
todo.filter[all|active|completed]  162:15  b.node(tab, Role::Tab, f.label())
todo.item[1|2|3]        list_item  169:30  let mut node = b.node(&id, Role::ListItem, item.text());
todo.counter            status     178:11  b.node(&ids.counter, Role::Status, self.counter_text())
```

os três pedidos: o controle de adicionar, um item e o filtro.

- adicionar: o todo não tem botão de adicionar. quem adiciona é o campo
  (`todo.input`, ação `submit`, "Enter adds it" no placeholder). o
  `source` dele é `153:14`, a chamada `TextField::project` do app: como
  `project` é `#[track_caller]` no comps, o site é o do showcase e não o
  do comps. nesta rodada "o botão de adicionar" é esse campo.
- um item: `todo.item[1]` em `169:30`, o `b.node` do list item dentro do
  laço dos itens visíveis.
- o filtro: `todo.filter` em `157:11`; cada pílula
  (`todo.filter[active]`) em `162:15`, o `b.node` da aba dentro do laço.

serve para quem clica: cada nó leva à linha do código do app que declara
aquela capacidade (id, papel, rótulo, estado e ações), no arquivo do app
e não no comps. a coluna aponta para o nome do método (`node`,
`project`). o site de render (`draw.rs`, que faz o `bind` dos mesmos ids)
fica a um passo, e o módulo `semantics.rs` diz isso no topo. limite
esperado: um nó projetado dentro de um laço (itens, abas) leva todos à
mesma linha, porque é o código, não o dado; o texto do item leva ao
literal pela fase b quando ele existe no código (os três itens do seed
existem). decisão: seguir com o site de projeção, sem `#[track_caller]`
no render.
