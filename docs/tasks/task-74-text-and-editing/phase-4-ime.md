---
id: task-74/phase-4-ime
title: "phase 4: IME, clipboard and field menu"
project: plev
type: section
status: reference
parent: task-74
date: 2026-09-25
date_basis: text
topics: [text, text-editing, input]
relations: []
source: docs/log/steps/task-74-text-and-editing.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [3579e5be, c4866d15, 45c0f909, a6bfcb24, 576417b8, f44c834b, 6215bddf, 5e56db92]
---

# phase 4: IME, clipboard and field menu

Wires IME composition into the `TextInput` and `TextField`, with the preedit drawn in the field, commit and the candidate window at the caret; the gallery only turns IME on with a focused field on the Input page or on the Editor page. It brings together copy, cut, paste and select all, and a context menu on the field, with the clipboard reached only by key or through the person's menu. The manual test script with Japanese and Chinese ran in the editor; the criterion item closes when brenner runs the script on a field of the Input page.

## fase 4: o roteiro do IME no editor (brenner, 2026-09-25)

o brenner rodou o roteiro no macOS, no exemplo do editor
(`cargo run -p engine --example editor`, o único app que liga o IME:
`rust/engine/src/widgets/examples/editor/main.rs:153`), com as fontes de entrada
japonês (Romaji) e chinês simplificado (Pinyin): `nihongo`, espaço e
Enter para 日本語; `zhongwen` e espaço para 中文. o que ele disse: "no
editor de código funciona" e, perguntado pelo japonês e pelo chinês,
"funcionaram". o editor já desenhava a composição sublinhada
(`comps/src/editor/view.rs:369`) e punha a janela de candidatos no
caret (`EditorView::ime_cursor_rect`).

o que falta para fechar o item "roteiro de IME executado e registrado":
o campo de texto. o `TextInput` joga fora o texto em composição
(`comps/src/form/text_input/component.rs:218-219`), e nem a galeria nem
o ide ligam o IME (`set_ime_allowed`: só o exemplo do editor chama),
então lá o japonês sai em letras. depois do código, o brenner roda o
mesmo roteiro num campo da galeria.

## fase 4: a sonda do campo

winit travado em 0.30.13; as linhas da biblioteca são de
`winit-0.30.13/src/platform_impl/macos/view.rs`.

### (a) a tecla latina com o IME ligado

com o IME ligado, toda tecla passa por `interpretKeyEvents` (466). o
`insertText` só manda `Ime::Commit` quando há texto marcado (411-415).
uma letra comum, sem composição, não marca nada: o estado não muda e o
`KeyboardInput` sai com o texto (478-497), como hoje. durante a
composição o estado é `Preedit` e a tecla não vira `KeyboardInput`
(485); o commit põe o estado em `Committed`, e o Enter que confirmou
não chega ao app (426-428, 480-483). então cada tecla chega por um
caminho só: `KeyboardInput` sem composição, `Ime::Preedit` e
`Ime::Commit` com ela. não insere duas vezes nem perde tecla. o
`Preedit("")` vem antes de cada commit (412). setas, Tab e Escape sem
composição passam pelo `doCommandBySelector` e seguem como tecla (430).
desligar (`set_ime_allowed(false)`) limpa a marca e manda
`Ime::Disabled` (880-896); trocar a fonte de entrada também (447-451).
o exemplo do editor já roda assim, e o latim digitado nele funciona.

### (b) onde está o retângulo do campo e o x do caret

o campo focado da página Input é o de `InputPage::focused`, no retângulo
de `InputPage::rects(content)` (`view/components/input.rs:55`), com
`content` o `page_rect` da view (`view/mod.rs:284`, já com a rolagem).
dentro do campo, a origem do texto é `bounds.x + pad_x`, `text_y`
centrado (`text_field.rs:326-327`), e o caret é `TextInput::caret_x`
(`component.rs:188`) com o mesmo `TextStyle` do desenho. o campo ganha
`ime_area(bounds, theme)` com essa conta (o caret do preedit durante a
composição); a view junta os dois. o editor já dá o seu em
`EditorView::ime_cursor_rect` (`editor/view.rs:221`).

### (c) quando ligar e desligar

sem campo focado as letras são atalhos: `handle_key` tenta o campo
(`view/input.rs:80`) e depois o `t` troca o tema (102). com o japonês
ligado, o `t` viraria composição e o atalho morreria. então o IME fica
ligado só com um campo focado da página Input, sem overlay por cima, e
na página Editor (que já toma todas as teclas, `events.rs:93`); em
qualquer outro caso, desligado. a galeria acerta isso a cada frame
desenhado da janela principal (`events.rs:197`): o foco só muda por um
evento que pede frame. as outras páginas com texto ficam desligadas,
como hoje (pendência no handoff).

## handoff da fase 4: o IME no campo de texto

prepara o item "roteiro de IME executado e registrado": o campo compõe
e a galeria liga o IME. falta o brenner rodar o roteiro num campo da
página Input (japonês `nihongo`, espaço, Enter; chinês `zhongwen`,
espaço); aí o item fecha.

### commits

- `3579e5be` a sonda do campo
- `c4866d15` a composição no `TextInput` e no `TextField`
- `45c0f909` a galeria liga o IME no campo focado e no editor
- `a6bfcb24` os testes e o PNG
- `576417b8` o `Disabled` sem o fold (o clippy)

### o que mudou

- `TextInput` guarda a composição e o caret do IME fora do buffer.
  `handle_ime(committed, preedit)` mantém a assinatura (e agora guarda a
  composição); a irmã `handle_ime_at` recebe o caret. perder o foco ou
  `clear_preedit` largam a composição. `composed`, `preedit_boxes` e
  `caret_x` usam o mesmo `TextStyle` do desenho.
- `TextField` desenha o texto composto (a composição no lugar da
  seleção ou no caret), um sublinhado de `edge_width` em
  `glass.text_active`, e o caret parado no caret do IME. `compose` passa
  o commit pelo mesmo `insert` da digitação; `end_compose` e `ime_area`
  são novos. a projeção lê só o buffer.
- galeria: `ImeEvent` e `handle_ime`/`ime_area` na view, sem winit; a
  página Input e a Editor recebem. o shell (`app/events/ime.rs`) passa o
  `WindowEvent::Ime`, grava o commit como tecla de texto (o tipo, nunca
  o texto) e, depois de cada frame da janela principal, liga o IME só
  quando `ime_area` responde. `app.rs` não cresceu.

### testes

sete no comps (`text_input/tests/tests_ime.rs`), quatro na galeria
(`tests/component_pages/ime.rs`) e um no shell (a gravação guarda um
`text` para o commit e nada da composição, conferido nos bytes do
arquivo salvo).

### o PNG

`/Users/nn/Dev/hoff/plevdev/target/agents/one/task-74h/ime-field.png`:
a página Input a 2x, o primeiro campo com `日本語!` digitado e `にほんご`
compondo, sublinhado, caret no fim. para refazer:
`IME_FIELD_OUT=<dir> cargo test -p uigallery --test component_pages the_focused_field`.

### gate e regress

`./tool/tasks/fullcheck.sh` verde: `cargo test --workspace` 3160 passed, 0 failed,
28 ignored; perna urnaui 86 passed, 1 ignored (3246 no total, eram
3234); clippy, fmt e wasm verdes. `regress -- check`: 176 unchanged, 0
changed. disco: 8,4 GB livres.

### pendências

- a janela da página (`D`) não liga o IME: a área da janela de
  candidatos precisaria do layout da janela da página, fora da rota.
- Todo, Surface, Silences e o formulário do Git continuam sem IME:
  o Todo só aceita um caractere por tecla, os campos das superfícies
  geradas vivem no crate `surface`, e o formulário do Git é do ide.
- o ide e o urnaui não ligam o IME (o mesmo fio).
- os frames gravados mostram o que está na tela, com a composição, como
  mostram o texto digitado.

## handoff: a área de transferência e o menu do campo (pedido do brenner)

nenhum item do critério da task: as duas partes são pedido do brenner.
custo aproximado: o IME uns 290 mil tokens, a parte A uns 65 mil, a
parte B uns 30 mil.

### commits

- `f44c834b` copiar, recortar, colar e selecionar tudo no campo
- `6215bddf` o menu do campo

### parte A: a área de transferência

- `TextInput` e `TextField` copiam, recortam e colam por um
  `ClipboardProvider` que o dono do campo passa (o campo segue `Clone`);
  `ClipKey` nomeia cmd (ctrl fora do macOS) com c, x, v, a. colar é
  digitar, numa linha só: as quebras das pontas saem, as de dentro
  (`\r\n` conta uma) e os tabs viram espaço, e outros caracteres de
  controle saem.
- galeria: com cmd ou ctrl apertado nenhuma letra entra num campo (o
  cmd+v não escreve mais "v"). com um campo com as teclas, as quatro vão
  para ele: Input, Todo, os campos de intenção do Surface e do Silences
  e o formulário de commit do Git. o campo de uma superfície gerada
  aceita colar e selecionar tudo. sem campo focado, tudo como antes. a
  view tem uma área de transferência, feita no primeiro uso; os testes
  põem uma `LocalClipboard`.
- ide: a linha de intenção e os campos das superfícies na coluna; o
  formulário de commit por ações `edit::` no contexto `CommitForm` do
  keymap (cmd e ctrl ligados, só o principal da plataforma age).
- privacidade: o colar é gravado como texto, nunca o texto. nenhum
  pedido pelo dispatcher chega à área de transferência. o texto colado
  vira o texto do campo, que é estado do app e aparece na projeção,
  como o digitado; a área de transferência em si não aparece.
- testes: 18 (7 no comps, 6 na galeria, 4 no ide, 1 no shell).

### parte B: o menu do campo

- o clique direito num campo da página Input dá as teclas ao campo (um
  campo com seleção a mantém) e abre ali o mesmo `ContextMenu` da página
  Context Menu, com o mesmo overlay e a mesma projeção (`context_menu`);
  o teclado é o do menu (setas, Enter, Escape). itens: Cut, Copy, Paste,
  Select all. cada um fica desligado sem nada para agir.
- a escolha do item, pelo ponteiro, pelas teclas ou por um leitor de
  tela, roda a tecla da área de transferência no campo depois do
  dispatch. a escolha de um agente falha, porque a área de transferência
  é da pessoa. a projeção ainda declara o `activate` dos itens com a
  política padrão (o `ContextMenu` do comps não tem política por menu);
  a recusa vem na execução.
- testes: 4 na galeria.
- PNG: `/Users/nn/Dev/hoff/plevdev/target/agents/one/task-74h/field-menu.png`,
  o menu aberto sobre o campo com `Ada Lovelace` selecionado, Paste
  desligado. para refazer:
  `FIELD_MENU_OUT=<dir> cargo test -p uigallery --test component_pages a_right_click_opens`.
  o `ime-field.png` foi refeito na mesma pasta.

### gate e regress

a parte A foi validada sozinha: tests 3178 no workspace e 86 no urnaui,
clippy, fmt e wasm verdes, 176 baselines iguais. depois da parte B:
`./tool/tasks/fullcheck.sh` verde, 3268 passed (3182 no workspace, 86 no urnaui),
0 failed, 29 ignored; clippy, fmt e wasm verdes. `regress -- check`:
176 unchanged. disco: 11 GB livres.

### pendências

- o menu do campo só existe na página Input; no Todo, no Surface, no
  Silences e no Git o clique direito segue como antes. no ide o menu
  ficou de fora.
- copiar e recortar num campo de uma superfície gerada pedem API no
  crate `surface`.
- um item só da pessoa na projeção pediria uma política por menu no
  `ContextMenu` do comps.
- enquanto o menu está aberto, a projeção mostra se o Paste está ligado,
  ou seja, se a área de transferência está vazia; o conteúdo nunca
  aparece.

## fase 4 no campo, área de transferência e menu integrados (orquestrador, 2026-09-25)

validação do orquestrador: diff lido commit por commit. a composição
fica fora do buffer e da projeção; um teste lê os bytes da gravação
salva e não acha nela a composição. o commit segue o caminho da
digitação, e uma seleção só é trocada no commit. a área de
transferência só é alcançada pelas teclas e pelo menu da pessoa, e a
escolha de um agente é recusada. os dois PNGs abertos: `にほんご`
sublinhado depois de `日本語!` confirmado, com o valor ao vivo mostrando
só o confirmado; o menu aberto sobre o campo com o texto selecionado.
176 baselines iguais.

mudança do orquestrador (`5e56db92`): o Paste do menu fica sempre
ligado. desligado com a área de transferência vazia, ele contava à
projeção, e daí ao agente e à gravação, se a área de transferência da
pessoa tinha algo. colar nada não faz nada. a pendência da parte B
sobre isso fecha.

continua aberto no critério: o roteiro de IME num campo da página Input,
que o brenner roda (japonês `nihongo`, espaço, Enter; chinês `zhongwen`,
espaço). com ele, todos os itens do critério batem e a task fecha.

o agente gastou uns 390 mil tokens nas três partes (o IME uns 290 mil,
a área de transferência uns 65 mil, o menu uns 30 mil).
