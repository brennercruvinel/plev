---
id: task-65/fase-c
title: "task-65 fase c: editar o texto e gravar o literal"
project: plev
type: secao
parent: task-65
status: referencia
date: 2026-09-25
date_basis: texto
topics: [hot-reload, edicao-de-texto, input]
relations: [adr-0048, task-65/fase-a-b]
evidence: [1e853d8a, 4f11e311, fc814104, 6910ad37]
source: docs/log/steps/task-65-pixel-to-source.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# fase c: a sonda

Registra a fase c: em build de debug, cmd+shift+clique num texto que um só literal guarda abre um campo sobre ele, e o Enter grava o literal no `.rs` por `literals::write`, só se o arquivo ainda tem o código do baseline. A prova é um `git diff` de uma string, num repositório temporário, e o agente é recusado. O gate do handoff passou com 3200 testes e o regress com 176 iguais; a janela real não foi usada, como o briefing pede.

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
- release (`cargo check --release -p uigallery`, verde) e wasm32 (o check
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
