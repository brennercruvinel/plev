---
id: task-65
title: "task-65: pixel para código e inspector de desenvolvimento"
project: plev
type: tarefa
status: concluido
date: 2026-09-25
date_basis: cabecalho
topics: [hot-reload, input, semantica, edicao-de-texto]
relations: [task-57, task-59, task-64, adr-0048]
evidence: [042d34e]
source: docs/log/steps/task-65-pixel-to-source.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-65: pixel para código e inspector de desenvolvimento

Registra o plano da task-65, do pixel ao código: um cmd+clique num controle ou num texto da janela de debug leva à linha do `.rs` que o produziu, e uma edição de texto na janela grava o literal de volta no arquivo, como `git diff` normal. As fases a, b e c estão entregues e o fechamento fechou o critério de aceite. A fase d, o mapa "quem leu este token", não foi implementada e ficou fora do critério.

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

- hot reload de literal (`rust/engine/src/runtime/hot_reload/literals/mod.rs`): no
  start, cada `.rs` vira skeleton (token stream com literais em branco,
  hasheado) mais a lista de literais em ordem; no save, mesmo skeleton
  quer dizer só literal mudou, e o delta é aplicado em `TextNodeKey`,
  `TextMeasurer` e `Compositor::push_to_layer`. limites já documentados:
  literal repetido muda nos dois lugares, string formatada antes de
  aparecer não casa, número é tratado como código.
- `proc-macro2` entra pela feature `hot-reload`
  (`rust/engine/Cargo.toml:35` e `:84`) sem `span-locations`: a lista de
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

`rust/engine/src/runtime/hot_reload/**`, `rust/engine/Cargo.toml` (feature
`span-locations`), o atalho no shell do showcase (dev only), proposta de
ADR [`0048-view-source-for-pixels.md`](../decisoes/processos/0048-view-source-for-pixels.md). `rust/engine/src/styling/theme` só
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
  --release -p uigallery` conferem).
- escrita recusada quando o arquivo mudou de estrutura desde o baseline
  (teste).
- `./tool/tasks/fullcheck.sh` verde.

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

## Fases

- [fases a e b: do pixel ao código](task-65-pixel-to-source/fase-a-b.md): a sonda do `source`, o cmd+clique até o código e o literal do hot reload, e a prova pela função que o atalho chama.
- [fase c: editar o texto e gravar o literal](task-65-pixel-to-source/fase-c.md): a sonda do baseline, a gravação pelo span do literal e o campo de edição na janela.
- [fechamento](task-65-pixel-to-source/fechamento.md): o critério de aceite item por item e as pendências: a fase d e o color picker de hex.
