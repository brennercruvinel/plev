---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-25
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
ADR `docs/adr/workflows/0048-view-source-for-pixels.md`. `rust/engine/src/styling/theme` só
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

## rodada 1: handoff do agente (fases a e b)

### a sonda do `source` (antes de qualquer código)

rodado na base (`76f58a0`), sem janela:

```text
cargo run -p showcase -- --render Todo --semantics todo.json --source
wrote todo.json (10 nodes, revision cfc1185078f7c825, Todo / core at 1200x800 scale 1)
```

o `source` de cada nó, e a linha para onde ele aponta
(`apps/uigallery/src/views/apps/todo/semantics/mod.rs`):

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

### disco antes do gate

na primeira conferência, depois do código, df mostrou 7,81 GiB livres,
abaixo dos 8 GB do briefing (três agentes compilando; o target desta task
cresceu uns 2 GB porque a feature nova do proc-macro2 recompila o engine
e quem depende dele). parei, commitei o ADR e o handoff e não liberei
nada (trash não libera espaço no mesmo volume; rm -rf proibido). minutos
depois havia 9,94 GiB livres, sem nada feito por mim: rodei o gate. ele
desceu a 6,1 GiB durante a perna urnaui e terminou verde. se o
orquestrador preferir que a parada valha até a decisão dele, o gate
desta rodada foi rodado acima do limite na hora da conferência.

### handoff

```text
Goal: task-65, rodada 1 (fases a e b): em build de debug no desktop,
      cmd+clique num controle abre o código que projetou a capacidade, e
      num texto sem id abre o literal que o produziu
Status: pronto para validação do orquestrador; gate verde

Implementado:
- engine, feature hot-reload: proc-macro2/span-locations. só o
  proc-macro2 do engine ganha a feature (resolver 2; `cargo tree -e
  features -i proc-macro2`: o único caminho é `engine feature
  "hot-reload"`); o dos proc macros fica como era; wasm e mobile não têm
  a dependência; o release compila sem ligar, como antes. Cargo.lock sem
  mudança.
- engine hot_reload::literals: cada literal guarda linha e coluna (de 1,
  coluna em chars, como Location), e marca o que o app nunca desenha:
  texto de doc comment, literal dentro de item sob #[cfg(test)], arquivo
  de teste (regra do file hygiene guard). o mapa do lexer é liberado a
  cada snapshot. um save que mantém o esqueleto atualiza as posições.
  sites_of(texto) devolve os literais que casam, ordenados.
- engine hot_reload::Site (arquivo absoluto, linha, coluna; imprime
  file:line:column) e hot_reload::view_source: origin_at, text_at,
  Origin (Projected, Unsourced, Literal, Formatted, Nothing) com site() e
  Display, editor_command, open, open_with.
- showcase: app/source.rs (App::view_source), o campo source_click e a
  chamada no MouseInput esquerdo; app/background.rs (PLEV_BACKGROUND=1,
  macOS); app/access.rs (Access movido de app.rs, sem mudança).

Como a resolução funciona:
- cmd+clique (ctrl fora do macOS) no MouseInput esquerdo: o shell toma o
  press e o release (mesmo se o cmd sair antes), a página não vê o
  clique, nada desenha.
- origin_at(compositor, snapshot da página ativa, cursor): Compositor::hit
  primeiro; com id, o Node.source do snapshot, absoluto a partir da raiz
  do workspace. sem id, text_at: o último text run cuja caixa medida
  (x, y do nó, tamanho do TextMeasurer com o estilo da chave) contém o
  ponto, dentro do clip, em camada visível, camadas empilhadas como no
  hit; e sites_of do texto.
- um site: abre. vários: log em warn com a lista, não abre (decisão: a
  lista, "lista para escolher" da task; abrir o primeiro seria chute).
  nenhum: log "formatted". sem nó ou sem source: log. o log vai em warn,
  o nível que o shell mostra por padrão; quando abre, info.
- editor, sempre com file:line:column como último argumento (caminho
  absoluto): $PLEV_EDITOR, dividido em palavras (programa e argumentos
  iniciais; caminho de programa com espaço não é suportado); senão
  `zed file:line:column`; senão `code -g file:line:column`; os dois
  procurados no PATH como executável; nenhum: erro dizendo isso. o filho
  é esperado numa thread, sem bloquear o loop.

Arquivos principais:
- rust/engine/Cargo.toml (só a feature)
- rust/engine/src/runtime/hot_reload/{mod.rs, literals.rs, view_source.rs (novo)}
- apps/uigallery/src/shell/mod.rs (595 linhas), apps/uigallery/src/shell/
  {source.rs, background.rs, access.rs} (novos)
- apps/uigallery/tests/view_source.rs (novo)
- docs/adr/workflows/0048-view-source-for-pixels.md (novo, proposed), este arquivo

Interfaces adicionadas/alteradas (todas novas, nenhuma alterada):
- engine::hot_reload::Site
- engine::hot_reload::literals::sites_of
- engine::hot_reload::view_source::{Origin, origin_at, text_at,
  editor_command, open, open_with}
- sem, wire, rec, comps, dispatch.rs e engine fora de hot_reload: sem
  mudança. a forma serde do Node e o source opt-in no wire: iguais.

Itens novos e o gate de cada um (release, wasm e mobile sem o código):
- engine: tudo dentro de `hot_reload`, que o lib.rs declara sob
  #[cfg(hot_reload)] (build.rs: feature, profile debug, macos, linux ou
  windows)
- showcase: `mod source`, o campo `source_click`, a inicialização e a
  chamada em window_event sob #[cfg(all(debug_assertions, any(target_os =
  "macos", target_os = "linux", target_os = "windows")))], os termos do
  build.rs do engine (o showcase não tem o cfg hot_reload: build.rs vale
  por crate); tests/view_source.rs com o mesmo #![cfg]
- showcase `mod background` (PLEV_BACKGROUND): #[cfg(target_os =
  "macos")], em todo profile, como no ide; não é parte do view source.
  se o orquestrador quiser só em debug, é uma linha.
- alternativa ao gate por debug_assertions: um build.rs no showcase
  repetindo a regra do engine daria o cfg hot_reload por nome (arquivo
  fora da lista, por isso não fiz).

Testes (21 novos):
- engine literals: each_literal_keeps_its_line_and_column,
  the_column_counts_chars_and_a_second_file_starts_over,
  a_text_finds_its_literals_and_follows_a_save,
  literals_in_test_code_are_unseen
- engine view_source: a_bound_control_leads_to_where_it_was_projected,
  a_bound_control_the_snapshot_does_not_source_is_said_so,
  a_text_leads_to_the_one_literal_that_holds_it,
  a_text_several_literals_hold_is_listed_and_not_opened,
  a_formatted_text_has_no_literal,
  the_text_is_the_topmost_one_that_shows_at_the_point,
  the_editor_is_plev_editor_then_zed_then_code,
  open_runs_the_editor_with_the_site_and_says_when_there_is_none
- showcase tests/view_source.rs (frame da página pelo stage_page, os
  literais do workspace): the_field_that_adds_leads_to_where_the_todo_
  projects_it, an_item_leads_to_the_list_item_projection,
  a_static_title_leads_to_its_one_literal ("Todo" e o subtítulo, linha
  e coluna conferidas lendo o arquivo), a_text_several_literals_hold_is_
  listed_and_not_opened ("plev", 10 literais), a_formatted_text_has_no_
  literal (célula da Data Table; o contador do todo acha o card)
- showcase app/source.rs: a_primary_click_opens_the_projection_and_
  never_reaches_the_page, a_plain_click_is_the_page_s,
  a_primary_click_on_nothing_opens_nothing,
  the_click_hands_the_editor_the_line_under_the_cursor

Validação executada (CARGO_TARGET_DIR e as variáveis do briefing):
- ./tool/tasks/fullcheck.sh em 41d65d4: verde, com a perna urnaui (../urna presente
  pelo link do worktree): test, clippy -D warnings, fmt, wasm32 com
  RUSTFLAGS -D warnings, e urnaui (test, clippy, fmt, wasm).
- contagem: 2246 testes passaram em todas as pernas somadas (2164 do
  workspace e 82 do urnaui), 0 falhas, 12 ignorados; a main tinha 2225,
  e os 21 novos são os listados acima.
- cargo test -p engine e cargo test -p showcase estão dentro do cargo
  test --workspace do gate.
- cargo check --release -p showcase em 41d65d4: verde; só o showcase foi
  rechecado, os arquivos novos do engine não entram no build de release.
- durante o trabalho, por crate: cargo test -p engine --lib hot_reload,
  cargo test -p showcase, clippy e fmt de engine e showcase.

A prova na janela:
- janela real: o showcase de debug, `PLEV_BACKGROUND=1 PLEV_AGENT=1
  PLEV_EDITOR=<script que só grava os argumentos>`, pelo guard da task-63d
  (mata o pid se a janela pegar o foco em 3 s). o app da frente ficou na
  frente a execução toda; janela na camada -1; baseline "literal hot
  reload on: 584 files"; a porta do agente devolveu os bounds do todo
  iguais aos do frame headless (todo.input [428, 158.2, 592, 44],
  todo.item[1] [428, 216.2, 592, 44]); um Escape postado no pid fechou.
- o cmd+clique postado no pid (CGEventPostToPid, com o número da janela
  nos campos 91 e 92, com e sem mouse moved antes) não chegou à página:
  nem ele nem um clique simples mudaram o foco ou o item. as duas telas
  estavam cobertas (Dia na principal, Zed na segunda), e com a janela no
  nível normal, sem foco, ela ficou sob o app da frente (índice 2, abaixo
  de sarue e Dia). o AppKit só entrega um evento de mouse à janela que
  está em cima no ponto; sem tomar o foco nem flutuar sobre a tela da
  pessoa, nenhum clique postado chega. o editor não abriu nada em
  nenhuma execução (o script não recebeu nada); todo processo foi morto
  pelo pid exato.
- por isso a prova vai pela função que o atalho chama (o briefing prevê):
  the_click_hands_the_editor_the_line_under_the_cursor monta o shell no
  todo, desenha o frame pelo caminho da janela, toma o baseline do
  workspace como o watcher, põe o cursor no campo que adiciona e depois no
  título, com cmd, e chama App::view_source_with com o opener real
  (open_with, PLEV_EDITOR = script gravador). o script recebeu:

  .../apps/uigallery/src/views/apps/todo/semantics/mod.rs:153:14
      .project(b, &ids.input, "new todo", Some(&ids.todo))
  .../apps/uigallery/src/views/pages/table.rs:459:9
      "Todo",

  (caminhos absolutos do worktree; a coluna aponta para `project` e para
  a aspa de abertura). um item leva a semantics.rs:169:30 (`node(&id,
  Role::ListItem, item.text())`) e o item não é marcado.
- se o orquestrador quiser o clique postado mesmo assim: a janela sobe
  flutuando (WindowLevel::AlwaysOnTop) sem foco durante os segundos da
  prova, cobrindo a tela da pessoa nesse tempo. não fiz.

Decisões:
- site de projeção, sem site de render (a sonda, acima).
- vários literais: lista no log, nada abre.
- id antes do texto: um texto dentro de um controle ligado acha o
  controle.
- literais de teste e doc comments fora da busca (senão "Todo" tinha 5
  sites, 4 de teste; agora 1).
- gate do showcase por debug_assertions + os três OS (acima).
- app.rs dividido (Access para app/access.rs) antes do background, que o
  passaria de 639.
- PLEV_BACKGROUND igual ao do ide (abaixo de todas as janelas): uma
  tentativa no nível normal não ajudou a prova e cobria a tela.

Limitações:
- nó projetado dentro de laço: todas as instâncias levam à mesma linha.
- texto cortado (reticências) ou formatado: nenhum literal.
- módulo de teste declarado fora de linha com outro nome ainda conta
  (hoje só showcase semantics/assistive.rs: "2 active · 1 done" tem um
  literal de teste; na janela o contador acha o card, então não aparece).
- um save que muda código e cujo rebuild falha deixa posições velhas.
- o modificador é cmd no macOS e ctrl fora dele; linux e windows não
  foram rodados.

Dependências pendentes: nenhuma de código. docs/adr/README.md (índice
do ADR novo) é do orquestrador.

Riscos de integração:
- a task-63d mexe no shell do ide, não no do showcase; background.rs do
  showcase copia a regra dela (se ela mudar, as duas divergem).
- o custo de memória do baseline sobe 1,2 MB (medido); o tempo, uns 35 ms.

O que a fase c precisa:
- o intervalo em bytes de cada literal (Span::byte_range, mesma feature)
  para trocar exatamente o span; hoje há só o início em linha e coluna.
- na escrita, reler o arquivo e conferir o esqueleto contra o do snapshot
  de onde veio a posição (baseline ou EDITED); diferente: recusa e diz.
- escapar o texto novo na forma do literal (normal ou raw).
- só com Origin::Literal de um site (nunca doc, teste ou vários).
- a edição inline no shell (um campo do comps sobre o texto) e o color
  picker; cor resolve como texto, pela cor do nó sob o cursor contra os
  Lit::Hex.

Commits:
- a7ae568 task-65: the probe of Node.source on the todo, in the handoff
- d57e86d task-65: each literal of the hot reload baseline knows its line and column
- c254236 task-65: a literal in test code is never what a click on a text finds
- dae72fb task-65: a point of the window resolves to the code that made it, testable without a window
- b138ba7 task-65: cmd+click in the showcase window opens the code that made the pixel
- 6e8e134 task-65: the showcase shell's outside actors move to app/access.rs
- 683354b task-65: the showcase window can come up without taking the focus
- 228bb60 task-65: the view-source click proved through the function it calls, with the real opener
- ec77e30 task-65: ADR view source for pixels (proposed)
- 41d65d4 task-65: handoff of round 1, stopped before the gate (disk under 8 GB)
- (este) task-65: handoff, the gate green with the urnaui leg (2246 tests)
```

## fechamento da rodada 1 (orquestrador, 2026-09-24)

integrada na `main` em `042d34e` (onze commits, rebaseados sem
conflito). o ADR view-source-for-pixels foi aceito.

validação do orquestrador:
- diff lido: `view_source.rs`, `literals.rs` com linha e coluna e o
  lexer liberado a cada snapshot, o atalho em `app/source.rs`, `Access`
  em `app/access.rs`, a janela em segundo plano;
- gate verde no diretório da task: 2261 passaram, 0 falharam, 12
  ignorados, com a perna urnaui;
- `cargo test -p showcase --test view_source` verde;
- `cargo check --release -p showcase` verde.

o que ficou provado e o que não ficou: o clique postado não chega a uma
janela que fica por baixo para não roubar o foco do brenner. por isso a
prova foi pela função que o atalho chama, com o abridor real e um script
que grava os argumentos (o campo do todo em `semantics.rs:153:14`, o
título em `table.rs:459:9`).

severidade do que ficou:
- P3: o módulo do showcase usa `debug_assertions` em vez do cfg
  `hot_reload` do engine, e quebraria com `--no-default-features`;
  módulo de teste fora de linha com outro nome ainda entra na busca;
- P2: fases c (gravar no `.rs`) e d (quem leu este token).

## fase c: a sonda

rodada 2 (fase c), na base `d0df381`, lendo o código, antes de qualquer
mudança. cada afirmação aponta arquivo:linha na base.

(a) o que o baseline guarda de cada literal, e o que é "o mesmo skeleton"

- `Snapshot` (rust/engine/src/runtime/hot_reload/literals/mod.rs:51) guarda o hash do
  skeleton (`skeleton: u64`), a lista dos literais em ordem (`lits`) e onde
  cada um começa (`at`). `At` (literals.rs:63) tem linha, coluna (de 1, em
  chars) e `unseen` (doc comment, item sob `#[cfg(test)]`). não há span
  em bytes: nem início nem fim. o `proc-macro2` do lock (1.0.106) tem
  `Span::byte_range` sob a mesma feature `span-locations` (lib.rs:478 do
  crate), com o intervalo relativo ao texto lido.
- o skeleton é o `FxHasher` do `walk` (literals.rs:124): cada ident, cada
  pontuação, o delimitador de cada grupo e, no lugar de cada literal, só
  o marcador `0xB1` (literals.rs:156). comentário e espaço não entram.
- "o mesmo skeleton" hoje é o que `apply_change` confere contra o
  `BASELINE` (literals.rs:301): hash igual e o mesmo número de literais;
  e nenhum literal `Other` (número, char, byte) mudou (literals.rs:311).
  qualquer outra coisa é `Change::Code`, o rebuild.
- o `BASELINE` é tirado uma vez, na thread do watcher, quando ele nasce
  (hot_reload/source.rs:32, literals.rs:256 a 266); o `GpuContext` cria o
  watcher de todo app (gpu/context.rs:491). um save que mantém o skeleton
  guarda o arquivo como está no disco em `EDITED` (literals.rs:323 a 326):
  é de lá que `sites_of` tira linha e coluna depois de um save.

(b) por onde a edição entra na janela, e quem pode

- hoje o shell só vê o clique: `WindowEvent::MouseInput` esquerdo chama
  `App::view_source` antes da página (app/events.rs:137), e o
  cmd+clique vai para `origin_at` (app/source.rs:37). o teclado
  (app/events.rs:52 a 101) não tem gancho do view source: a edição
  precisa de um, o primeiro do `KeyboardInput`, e de um no desenho do
  frame (app/pace.rs:48, depois do `view.render`).
- o widget é o `comps::form::TextField` (rust/engine/src/widgets/form/text_field.rs:49):
  `insert` (:179) para o texto, `edit(EditKey)` (:190) para apagar e
  mover, `render_to_layer` (:273) para desenhar numa camada nossa, acima
  dos toasts. o comps não precisa de widget novo.
- quem pode: só a pessoa, pelo teclado e pelo mouse da janela. a edição
  não é uma capacidade: nada dela entra na projeção nem no dispatcher. o
  que um agente pede pela porta vai para `serve_agent_recorded`, o
  dispatcher da página (app.rs:325), e o `Hand` do mirror aperta pelo
  mesmo dispatcher como `Actor::Agent` (app/mirror.rs:373); nenhum dos
  dois chega ao teclado do shell. o `PERSON_ONLY` da casa é o das
  capacidades do shell no dispatcher (view/dispatch/shell.rs:55); como a
  gravação não passa por ele, a recusa fica na própria gravação: ela
  recebe quem pediu e recusa `Actor::Agent`, com teste.

(c) como o hot reload aplica depois da gravação

- o `SourceWatcher` junta os eventos por 500 ms (hot_reload/source.rs:33),
  fica com os `.rs` criados ou modificados que não começam com ponto
  (:42, :96) e chama `apply_change` (:52): só literais, `Literals(n)` e
  `wake()` (:65), o redraw da janela.
- `apply_change` troca o mapa de overrides (`baseline[i] -> agora[i]`),
  que o texto consulta ao virar pixel: `TextNodeKey` (compositor/scene.rs:127)
  e o `TextMeasurer` (text/measure.rs:330). a janela mostra o texto novo
  com o estado intacto.
- então: a gravação escreve num arquivo temporário com ponto no nome, na
  mesma pasta (o watcher ignora), e faz o rename por cima; em seguida
  chama `apply_change` ela mesma, para o próximo frame já mostrar o texto
  sem esperar os 500 ms. o evento do rename chega depois ao watcher, que
  acha o mesmo delta (idempotente) e pede um frame.

## rodada 2: handoff do agente (fase c)

```text
Goal: task-65, fase c: em build de debug no desktop, editar um texto na
      janela grava o literal no `.rs`, e o resultado é um `git diff` de
      uma string
Status: pronto para validação do orquestrador; gate verde, regress 176
        iguais

Como funciona a edição (showcase, app/source.rs):
- cmd+shift+clique (ctrl+shift fora do macOS) num texto: o shell pede
  `origin_at`, como no cmd+clique. se a resposta é um texto que um
  literal só guarda, abre um `comps::form::TextField` sobre ele, numa
  camada acima de todas as do frame; o fundo da página cobre o texto
  velho. a caixa do texto vem de `view_source::text_run_at` (nova, a
  mesma busca do `text_at`, com a caixa).
- com o campo aberto, ele é dono do teclado (o primeiro gancho do
  KeyboardInput): letras, espaço, apagar, setas, home, end, shift para
  selecionar, cmd+a. Enter grava, Esc descarta, um clique fora descarta e
  vai para a página, um clique no campo põe o caret.
- um controle, vários literais ou um texto formatado: nada abre, um toast
  diz por quê. o toast e o log, nunca o `shell:notice` (que um agente lê).

Como funciona a gravação (engine, hot_reload/literals/write.rs):
- `literals::write(site, de, para, quem)`. recusa `Actor::Agent` antes de
  ler qualquer coisa.
- acha o slot do literal (o índice na ordem dos literais do arquivo) onde
  o clique achou: o último save que manteve o skeleton, senão o baseline.
- relê o arquivo: tem de ter o código do baseline (mesmo hash de
  skeleton, mesmo número de literais, todo literal que não é string nem
  hex igual). é o teste que o `apply_change` já fazia, agora uma função só
  (`same_code`). senão recusa: "o código mudou desde este build;
  rebuild, depois edite". o slot tem de guardar ainda o texto de antes.
- o intervalo em bytes do token vem na hora (`Span::byte_range`, a mesma
  feature), relendo o texto do disco; o baseline não guarda bytes, a
  memória dele não mudou. o token achado ali tem de voltar ao texto.
- troca só aqueles bytes, com o escape do Rust: aspas, barra, \n \r \t \0
  e outros controles como \u{..}; unicode vai como foi digitado. raw
  string continua raw, com os # que precisar (nunca menos).
- escrita atômica: arquivo com ponto no nome ao lado (o watcher ignora),
  mesmas permissões, sync, rename por cima; se falha, o arquivo fica como
  estava e o temporário sai. depois chama `apply_change`: o próximo frame
  já mostra o texto novo, sem rebuild; o evento do watcher acha o mesmo
  delta depois.

Arquivos:
- rust/engine/src/runtime/hot_reload/literals/mod.rs (`mod write`, `same_code`)
- rust/engine/src/runtime/hot_reload/literals/write/mod.rs e write/tests.rs (novos)
- rust/engine/src/runtime/hot_reload/view_source.rs (`text_run_at`)
- apps/uigallery/src/shell/window/source.rs (a edição e os testes)
- apps/uigallery/tests/edit_literal.rs (novo)
- fora da lista do briefing, só os ganchos do módulo, sob o mesmo cfg:
  app.rs (o campo `source_click: bool` virou `source: ViewSource`, mesmo
  número de linhas), app/events.rs (+8, o gancho do teclado),
  app/pace.rs (+13, o desenho do campo na janela principal e na da
  página). a rodada 1 já tinha o gancho do clique em events.rs e o campo
  em app.rs. access.rs não mudou.
- docs/adr/workflows/0048-view-source-for-pixels.md (seção da fase c), este arquivo.

Testes (11 novos, nenhum removido):
- item 1 (editar o título grava o `.rs`, `git diff` só aquela string):
  showcase tests/edit_literal.rs
  editing_the_title_writes_the_rs_and_git_diff_shows_only_that_string.
  num repositório git temporário com uma cópia do table.rs (o único
  arquivo que esse processo lê), o título "Todo" do frame leva ao literal
  da cópia; a função que o Enter chama grava "Todo list"; `git diff
  --numstat` é `1 1 table.rs` e o patch é
      -        "Todo",
      +        "Todo list",
  o frame seguinte desenha "Todo list" sem rebuild. no mesmo teste: o
  agente é recusado e o arquivo fica igual; depois de uma mudança de
  código na cópia, a gravação é recusada e o arquivo fica igual.
- item 1, o caminho da janela: app/source.rs
  a_title_edited_in_place_hands_its_literal_and_the_new_text_to_the_writer
  (cmd+shift+clique no título abre o campo sobre ele, na camada de cima;
  as teclas editam; o Enter entrega ao gravador table.rs, "Todo" e o
  texto novo), escape_or_a_press_outside_drops_the_edit_and_writes_nothing,
  a_write_the_engine_refuses_is_told_in_the_window (o toast),
  a_text_that_is_not_one_literal_is_not_edited_and_says_why (o campo do
  todo, "plev"). nesses o gravador é do teste: nenhum grava no
  repositório.
- item 3 (escrita recusada quando o arquivo mudou de estrutura): engine
  a_file_whose_code_changed_since_the_baseline_is_refused_untouched (uma
  função a mais, e um número mudado; conteúdo, mtime e a pasta iguais),
  e o fim do teste do showcase acima.
- escape: quotes_a_backslash_and_unicode_are_written_escaped_and_read_back
  (aspas, barra, acento, ✓, 日本, tab, controle; raw que precisa de ##;
  o arquivo relê como o texto digitado).
- agente: an_agent_cannot_write (engine) e o teste do showcase.
- os outros: the_literal_s_bytes_are_replaced_and_the_hot_reload_applies_it
  (só o token, o comentário com o mesmo texto fica; e volta),
  a_literal_changed_on_disk_since_the_click_is_refused,
  a_file_the_baseline_never_read_is_refused; e text_run_at dentro de
  the_text_is_the_topmost_one_that_shows_at_the_point.

Símbolos (item 2), no metadata do check (rmeta), contando os nomes novos:
- release (`cargo check --release -p showcase`, verde) e wasm32 (o check
  do gate): write_beside, text_run_at, same_code, plev-edit, sites_of,
  view_source no engine; ViewSource, not_one_literal, field_bounds,
  press_while_editing, begin_edit, view_source_with no showcase: 0 em
  todos.
- controle, debug: os mesmos nomes aparecem (1 cada).
- android e iOS: não rodei; o código novo está sob o mesmo cfg (debug e
  macos, linux ou windows), como na rodada 1.

Validação:
- ./tool/tasks/fullcheck.sh em 6910ad37 (CARGO_NET_OFFLINE e as variáveis do
  briefing): verde, com a perna urnaui. 3200 testes passaram (3115 do
  workspace e 85 do urnaui), 0 falhas, 28 ignorados. disco antes: 12 GiB
  livres.
- regress: `touch` no examples/regress/main.rs e `cargo run --offline -p
  showcase --example regress -- check`: 176 iguais, 0 mudaram.
- janela real: nenhuma, como o briefing pede. a prova vai pela função que
  o Enter chama (numa cópia, num git temporário) e pelo caminho das
  teclas com um gravador de teste.

Decisões:
- o gesto é cmd+shift+clique: cmd+clique continua abrindo o editor.
- a edição usa `origin_at` (o id ganha): um texto dentro de um controle
  não é editado aqui. evita editar um literal de teste de um módulo fora
  de linha (o contador do todo acharia o de semantics/assistive.rs).
- a recusa do agente fica na própria gravação, não num PERSON_ONLY do
  dispatcher: a edição não é capacidade e não passa por ele.
- o campo vai do recuo antes do texto até a borda direita da janela (sem
  largura fixa), da altura de um campo ou do texto, o que for maior.

Limitações:
- o campo usa o estilo do corpo, não o do texto (um título fica menor
  enquanto editado), e não segue um scroll embaixo dele (Esc resolve).
- o caret não pisca (o wake do campo não entra no ritmo do shell); fica
  visível. sem IME e sem colar.
- o texto novo aparece onde o velho aparece (o find e replace do override,
  como em qualquer save), até o próximo build.
- uma watch da janela (PLEV_MIRROR) vê o campo como pixels; nada dele
  entra na projeção.

Linha de ADR para o índice (docs/adr/README.md, a linha do
view-source-for-pixels ganha no fim, antes do "(task-65, fases a e b)"):
  ...; fase c: cmd+shift+clique num texto de um literal só abre um
  `TextField` sobre ele e o Enter grava o literal no `.rs`
  (`literals::write`: só o token, escapado na forma dele, só se o arquivo
  ainda tem o código do baseline, escrita atômica, o hot reload aplica),
  um `git diff` de uma string; agente recusado (task-65, fases a, b e c)

Pendências:
- color picker para hex literal (mesma regra da gravação: o slot, o
  código do baseline, o token), sem código nesta rodada.
- fase d (quem leu este token): começa por proposta ao orquestrador.

Commits:
- 1e853d8a task-65: the probe of phase c, what the baseline keeps and where an edit enters
- 4f11e311 task-65: a literal is written back in its span, only while the file keeps the baseline's code
- fc814104 task-65: cmd+shift+click edits a literal in place, Enter writes it as the person
- 6910ad37 task-65: the title edited on a copy in a git repository, git diff is that one string
- (este) task-65: ADR phase c and the handoff of round 2
```

## fechamento (orquestrador, 2026-09-25)

fechada pelo critério de aceite, item por item:

- item 1: o cmd+clique que abre o site e o literal fechou na rodada 1;
  editar o título grava o `.rs` e o `git diff` mostra só aquela string
  (`apps/uigallery/tests/edit_literal.rs`, numa cópia do `table.rs` num
  repositório git temporário: `-"Todo",` e `+"Todo list",`, e o frame
  seguinte já desenha o texto novo).
- item 2: os nomes novos aparecem zero vezes no release e no wasm e
  aparecem no debug; `cargo check --release -p showcase` verde.
- item 3: com o código mudado desde o baseline, a gravação é recusada e o
  arquivo, a data e a pasta ficam iguais (engine e teste com git).
- item 4: gate abaixo.

validação do orquestrador: diff lido (`literals::write` recusa agente
antes de ler, confere o código do baseline e o literal no disco, troca só
o token com o escape da forma dele, escrita atômica); os testes só gravam
em diretório temporário, e a árvore do repositório ficou limpa depois do
gate. o agente mexeu em três arquivos fora da lista do briefing
(`app.rs`, `app/events.rs`, `app/pace.rs`), só os ganchos do teclado e do
desenho, sob o mesmo cfg de debug no desktop; aceito, porque sem eles a
edição não recebe tecla nem aparece.

pendências: o color picker de hex e a fase d (quem leu este token), sem
item no critério; o campo usa o estilo do corpo (um título fica menor
enquanto edita) e não acompanha um scroll.

o agente gastou uns 245 mil tokens.
