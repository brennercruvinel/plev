---
id: task-65/fase-a-b
title: "task-65 fases a e b: do pixel ao código"
project: plev
type: secao
parent: task-65
status: referencia
date: 2026-09-24
date_basis: texto
topics: [hot-reload, input, semantica]
relations: [adr-0048, task-63]
evidence: [042d34e, a7ae568, d57e86d, c254236, dae72fb, b138ba7, 6e8e134, 683354b, 228bb60, ec77e30, 41d65d4]
source: docs/log/steps/task-65-pixel-to-source.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# rodada 1: handoff do agente (fases a e b)

Registra a rodada 1, que entregou as fases a e b: um cmd+clique num controle ou num texto da janela de debug leva à linha do código que o produziu, pelo `Node.source` ou pelo literal do hot reload. A sonda do `source` no todo, a prova pela função que o atalho chama e o gate verde de 2261 testes fecharam a rodada; o clique postado no pid não chegou à janela em segundo plano. Integrada na `main` em `042d34e`, com o ADR de view source aceito.

## a sonda do `source` (antes de qualquer código)

rodado na base (`76f58a0`), sem janela:

```text
cargo run -p uigallery -- --render Todo --semantics todo.json --source
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

## disco antes do gate

na primeira conferência, depois do código, df mostrou 7,81 GiB livres,
abaixo dos 8 GB do briefing (três agentes compilando; o target desta task
cresceu uns 2 GB porque a feature nova do proc-macro2 recompila o engine
e quem depende dele). parei, commitei o ADR e o handoff e não liberei
nada (trash não libera espaço no mesmo volume; rm -rf proibido). minutos
depois havia 9,94 GiB livres, sem nada feito por mim: rodei o gate. ele
desceu a 6,1 GiB durante a perna urnaui e terminou verde. se o
orquestrador preferir que a parada valha até a decisão dele, o gate
desta rodada foi rodado acima do limite na hora da conferência.

## handoff

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
- cargo test -p engine e cargo test -p uigallery estão dentro do cargo
  test --workspace do gate.
- cargo check --release -p uigallery em 41d65d4: verde; só o showcase foi
  rechecado, os arquivos novos do engine não entram no build de release.
- durante o trabalho, por crate: cargo test -p engine --lib hot_reload,
  cargo test -p uigallery, clippy e fmt de engine e showcase.

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
- `cargo test -p uigallery --test view_source` verde;
- `cargo check --release -p uigallery` verde.

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
