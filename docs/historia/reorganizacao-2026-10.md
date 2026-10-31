---
id: reorganizacao-2026-10
title: "Reorganização da árvore do repositório (issue #130, PR #131)"
project: plev
type: registro
status: concluido
date: 2026-10-09
date_basis: cabecalho
topics: [workspace, git, processo]
relations: [adr-0006, arquitetura-component-catalog]
evidence: [68521101, 3161bb89, d3e65544, 52df2254, 63cdad9d, 4039b6af]
source: docs/log/changes/tree-reorganization.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# Reorganização da árvore do repositório (issue #130, PR #131)

O repositório saiu de `crates/` para quatro casas (`rust/` bibliotecas,
`apps/`, `demo/`, `tool/`), com docs, dados e devops nos lugares da
estrutura-alvo. Nenhum comportamento mudou: mesmos testes, mesma API (no
caminho novo), mesmas dependências externas, mesmos snapshots de
comportamento.

- base: `main` em `68521101`
- commits: 37, cada um com um só tipo de mudança
- moves: 1861 arquivos, todos R100, no commit `d3e65544`
- arquivos rastreados ao fim: 1945, exatamente os da estrutura-alvo

## resumo

Um arquivo, `migration.toml`, decidiu o destino de cada um dos 1979
arquivos de origem: 1870 moves, 14 merges, 20 drops, 75 novos. Os moves
entraram num commit só de renomes, sem uma linha de conteúdo mudada, para
`git log --follow` atravessar a mudança. As provas foram gravadas antes do
primeiro move e repetidas depois, com o lado "antes" traduzido pelo mesmo
mapa (caminhos de arquivo, nomes de pacote e caminhos de módulo). Todas
passam.

## provas

| prova | ferramenta | resultado |
|---|---|---|
| commit só de moves | `git diff -M --name-status` | 1861 arquivos, todos R100 |
| mapa nos dois sentidos | `cargo xtask audit post` | toda origem tem destino ou drop; os 1945 arquivos rastreados são exatamente os alvos |
| mesma lista de testes | `cargo nextest list` + `testlist.py` | 3198 testes iguais um a um; +6 testes de arquitetura novos; os 2 do `arc_sync_guard` viraram `test_repoaudit` |
| todos passando | `cargo nextest run --workspace` | 3180 passam, 26 ignorados (main: 3174 + 26) |
| doctests | `cargo test --doc` | 12 passam, 2 ignorados, os mesmos itens do main |
| mesma API, crates renomeados | `cargo public-api` + `apidiff.py` | 13 de 19 crates idênticos linha a linha; os outros 6 só com as diferenças explicadas abaixo |
| mesma API, mesmo nome | `cargo semver-checks` | inspector 196/196; engine: 6 checks de caminho, os 546 itens apontados existem todos no caminho novo |
| snapshots de comportamento | insta, 11 programas | 11/11 iguais aos gravados antes do primeiro move |
| mesmo metadata | `cargo metadata` + `metacmp.py` | 567 pacotes externos iguais (nome, versão, fonte); por família de crates, mesmas dependências |
| mesma árvore | `cargo tree` | mesmos 321 pacotes externos no grafo |
| arquitetura | `tool/tests` | casas e setas de dependência, raiz, docs, numeração de ADR, ATLAS cobre todo crate e demo |
| links | `lychee --offline` | 0 quebrados (o main tinha 14) |
| gate | `tool/tasks/fullcheck.sh` | verde: testes, doctests, clippy `-D warnings`, fmt, wasm32, lychee |
| iOS simulador | `cargo check --target aarch64-apple-ios-sim` | compila com `-D warnings` |
| Android | `cargo ndk` | não verificado: a máquina não tem NDK (issue #132) |
| urnadbapp | sua perna do gate | não verificado: precisa do checkout irmão `../urna` (issue #132) |

As diferenças de API que sobram:

- **falso positivo do normalizador**: funções ou macros com o nome de um
  módulo renomeado (`accord::check`, `import::resolve_*`, `emit_ir`,
  `engine::actions!`, `engine::component`, a raiz do `macros`); a linha é
  idêntica nos dois lados.
- **módulos-grupo novos**: `engine::compose`, `import::formats`,
  `motion::encoding` e afins; só adição.
- **comps dentro do engine**: o `pub use engine::theme` do comps era uma
  linha e agora o rustdoc expande o módulo (299 linhas); 107 assinaturas
  diferem só na grafia do caminho do tipo.

## mapa de crates

| novo | vem de | nota |
|---|---|---|
| `rust/accord` | `crates/sem` | |
| `rust/claude` | `crates/claude-model` | |
| `rust/editor` | `crates/rope` | |
| `rust/engine` | `crates/engine` + `crates/comps` | comps vira `engine::widgets` |
| `rust/gitdrv` | `crates/git` | |
| `rust/import` | `crates/lot` + `svg` + `parser` | formatos em `import::formats`; conversores num binário `convert` |
| `rust/llmrun` | `crates/llm` | workspace próprio |
| `rust/macros` | `crates/macros` + `narrate-macro` | um crate proc-macro |
| `rust/markup` | `crates/narrate` | |
| `rust/motion` | `crates/monster` | o formato continua `.monster` |
| `rust/offers` | `crates/propose` | |
| `rust/plevir` | `crates/uir` | |
| `rust/record` | `crates/rec` | |
| `rust/replay` | `crates/inspect` | binário continua `plev-inspect` |
| `rust/screen` | `crates/surface` | |
| `rust/socket` | `crates/wire` | binário `plev` |
| `apps/idestudio` | `crates/ide` | módulos do binário via `#[path]` |
| `apps/agenteval` | `crates/ide/src/bin/lab` | o binário ide-lab virou app |
| `apps/inspector` | `crates/inspector` | |
| `apps/mirroring` | `crates/showcase/examples/mirror` | |
| `apps/uigallery` | `crates/showcase` | Android `com.plev.uigallery` |
| `apps/urnadbapp` | `crates/urnaui` | workspace próprio |
| `demo/*` (13) | examples de engine, sem, parser, claude-model; `crates/prime` | cada demo é um crate |
| `tool/bench/measure`, `regress` | `crates/showcase/examples` | |
| `tool/tests` | guardas do engine | + testes de arquitetura novos |

Os nomes antigos e novos, módulo a módulo, estão em
[TERMS.toml](../TERMS.toml).

## consequências da estrutura-alvo

- **os caminhos públicos do engine mudaram.** O alvo agrupa os módulos do
  engine (`compose`, `styling`, `control`, `drawing`, `runtime`,
  `widgets`). Todo uso no repositório foi reescrito, inclusive comentários
  e docs atuais; não há atalhos com os caminhos antigos. Por isso o
  `cargo semver-checks` classifica como major.
- **o engine depende do `editor`.** comps foi fundido no engine e o editor
  de texto do comps usa o crate `editor`; quem usa o engine compila também
  o `editor`. No `rust/llmrun` isso trouxe `ropey` e `str_indices`;
  nenhuma outra versão mudou.

## o que falta (issue #132)

- build Android numa máquina com NDK;
- a perna do urnadbapp (CI ou com `../urna`), e com ela tirar o `wgpu` que
  o `cargo machete` aponta sem uso.

## decisões e imprevistos

Onde o alvo não dizia a origem:

- **drops (20)**: exemplos builder, charts, text e visual do engine,
  exemplo editor do comps, `comps/Cargo.toml`, e `3Dscene.wgsl` (cópia
  idêntica do shader do demo camera, nunca carregada).
- **merges (14)**: `view/trait_def` e `component/mod` em `compose/view`;
  `theme/core` em `styling/theme`; os conversores lot2monsters e
  svg2monster num binário `convert`; os manifests fundidos; os scripts de
  captura, web e mobile num script cada.
- **nomes**: o preview do parser virou `demo/mockup`; o example propose
  virou `demo/intent`; os guardas do engine viraram `tool/tests`.
- `docs/ADR` → `docs/adr` por um nome temporário (o macOS não distingue
  caixa).
- o binário ide-lab virou o app agenteval: nome, mensagens e diretório de
  saída acompanham.
- idestudio: os 6 módulos que só o binário usa ficaram onde o alvo os põe
  e são declarados no `main.rs` com `#[path]`.

Imprevistos encontrados e resolvidos:

- **clippy já vermelho no main**: o `.cargo/config.toml` apontava para
  `.contracts/`, apagado. O único efeito real era o limite de 8
  argumentos; duas funções ganharam `#[allow(clippy::too_many_arguments)]`.
- **hot reload**: vigiava `crates/*/src`; incluir os benches (eram
  examples, nunca vigiados) fazia o view-source achar 5 literais em vez de
  1. Ficaram de fora.
- **caminhos Rust em strings** (código que o `import` gera, tokens de tema
  que o demo mockup lê): nenhum passe cobria strings; 21 caminhos
  corrigidos e os goldens regerados (só o caminho muda).
- **código em comentários de doc**: o nextest não roda doctests; o gate
  pegou `use sem::` num doctest do accord. 23 linhas corrigidas.
- **comentários e docs atuais**: 365 linhas em 202 arquivos citavam os
  módulos pelos caminhos antigos. Logs de task, ADRs e
  [`component-catalog.md`](../referencia/arquitetura/component-catalog.md) (o plano que mapeou os nomes antigos)
  mantêm os nomes da época.
- **links**: 12 só resolviam porque o macOS ignora caixa (`docs/adr` vs
  `docs/ADR`); os 13 que o main já tinha quebrados também foram
  corrigidos.
- **manifests**: comentários em 15 `Cargo.toml` ainda usavam nomes e
  caminhos antigos; o lock do llmrun ainda dizia `llm` e `sem`.
- **texto da página Editor**: dizia `comps::editor`; agora
  `engine::widgets::editor`, com as 2 baselines de regress regravadas (só
  a camada z0 muda; as outras 176 seguem iguais).
- **regra de texto** (ADR
  [rendering/0006](../decisoes/renderizacao/0006-one-text-style-for-measurement-and-drawing.md)):
  `test_namerules` só varria o engine; ao varrer tudo, revelou 7 arquivos
  que montavam texto sem `TextStyle`. Os 7 usam `TextNodeKey::from_style`
  com os mesmos valores (chaves iguais bit a bit) e a lista de pendências
  acabou.
- **dependências sem uso** (`cargo machete`): 26 linhas, quase todas
  sobras dos crates que a divisão criou (`wgpu`/`winit` em 10 demos,
  `record` no player, 4 no agenteval, 2 no mirroring) e o `wgpu` de
  uigallery e idestudio. `image` e `tempfile` do uigallery estão em uso.
  Ficam de propósito: `burn-ndarray` no llmrun (o comentário diz por quê)
  e o `wgpu` do urnadbapp (issue #132).
- **iOS**: o aviso `unused variable: window`, que vinha do main, foi
  corrigido; compila com `-D warnings`.

## como revisar

Commit a commit; cada um tem um só tipo de mudança. O commit de moves se
confere pelo `--name-status`; os seguintes são pequenos o bastante para
ler. As ferramentas da migração (`xtask/`, `migration.toml`, o alias
`cargo xtask`) saem em `52df2254` e ficam no histórico: `63cdad9d` é o
último commit que as tem, para rodar qualquer prova de novo.

```sh
git checkout 63cdad9d
git diff -M --name-status d3e65544^ d3e65544 | grep -vc '^R100'   # 0
cargo xtask audit post
cargo nextest run --workspace --no-fail-fast
INSTA_UPDATE=no cargo test --manifest-path xtask/Cargo.toml --test behaviour
MIGRATION_BASE=3161bb89 python3 xtask/scripts/testlist.py before.json after.json
python3 xtask/scripts/apidiff.py before/ after/   # cargo +nightly public-api -p X -sss
python3 xtask/scripts/metacmp.py before.json after.json
lychee --config .devops/check/lychee.toml --offline .
```

| commit | o que faz |
|---|---|
| `3161bb89` | o mapa, a ferramenta e os snapshots de comportamento, antes do primeiro move |
| `d3e65544` | move: todo arquivo para a estrutura-alvo, só moves |
| `5be51655` | merge e drop: os 14 merges e os 20 drops |
| `9e31564e` | manifests: workspace e crates com nome e lugar novos |
| `a76790e3` | módulos: as declarações seguem os arquivos |
| `7adad900` | caminhos Rust seguem os renomes |
| `4c6f9d15` | caminhos de arquivo escritos no código seguem os moves |
| `3105abce` | build: o workspace compila |
| `118946b4` | lint: clippy limpo |
| `8624f5a1` | fmt |
| `59986b31` | testes: o que a suíte lê por caminho |
| `832f9ff1` | tooling: gate, CI e contratos das ferramentas |
| `dc2b3609` | mobile e web: os projetos da galeria com o nome novo |
| `f5789701` | texto: caminhos em docs, scripts, configs e comentários |
| `c7271036` | docs: o ATLAS mapeia a árvore nova |
| `57bed778` | docs: os índices dos docs |
| `bc616be8` | novos: os arquivos que o alvo acrescenta |
| `cb08f13f` | remove o esqueleto `boiler/` |
| `fb438af8` | texto: comandos cargo e os últimos caminhos antigos |
| `0121ea4e` | caminhos Rust dentro de strings |
| `16695e70` | testes: as últimas expectativas da árvore antiga |
| `e5e4886b` | comentários de manifesto |
| `5452cd93` | docs: todo link resolve |
| `c1f3b772` | tooling: as provas traduzem caminhos de módulo |
| `35404b35` | uigallery: a página Editor mantém o texto (revertido em `ca8caca4`) |
| `71ac2568` | llmrun: o lock com os nomes novos |
| `06fa5569` | tooling: as provas de API e de metadata |
| `63cdad9d` | docs: o código em comentários de doc |
| `52df2254` | remove a ferramenta da migração |
| `1b4516dc` | texto: comentários e docs atuais com os caminhos novos de módulo |
| `ca8caca4` | uigallery: a página Editor diz `engine::widgets::editor` |
| `44c8678d` | texto: todo texto fora do engine vem de um `TextStyle` |
| `79f8c43d` | manifests: sem dependências sem uso |
| `cf5c9c44` | uigallery: sem aviso no iOS e no Android |
| `6ee6d4c5` | engine: o doctest do `actions!` chama o macro pelo nome |
| `e35e2ce6` | testes: o guarda de ADR ignora arquivos do sistema (`.DS_Store`) |
| `4039b6af` | docs: este relatório |
