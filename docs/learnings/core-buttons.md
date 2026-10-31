---
id: learning-core-buttons
title: "core buttons session: what worked and what bit us"
project: plev
type: learning
status: reference
date: 2026-09-21
date_basis: header
topics: [design-system, rendering, gpu]
relations: [adr-0028, adr-0029]
source: docs/log/learnings/core-buttons.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# core buttons session: what worked and what bit us

Records the session in which the showcase began embedding the editor and git, brought in the core theme (the TypeScript kit) and rewrote the buttons on top of it ([adr-0028](../adr/rendering/0028-core-theme-tailwind-classes-as-recipes.md) and [adr-0029](../adr/rendering/0029-embed-an-app-view-by-frame-origin.md)). The result is a template for the kit's next pages: the class list becomes a token struct, then a recipe and a thin widget. It was written for whoever migrates the next page, human or agent, and separates what worked, what bit us and what was left for later.

registro da sessão que embutiu editor e git no showcase, trouxe o tema
core (o kit typescript) e reescreveu os botões sobre ele (adr
core-theme-tailwind-classes-as-recipes, embed-an-app-view-by-frame-origin).
escrito pra quem for migrar a próxima página do kit, humano ou agente.

## o caminho que funcionou

1. **o kit é fonte, não referência de olho.** cada class list do
   tailwind virou uma struct de tokens antes de qualquer widget existir:
   `ButtonSurface` (fill, shadow list, overlay, text) por estado,
   `ButtonRecipe` por variante, as escadas de tamanho em px. o widget só
   lê. quando a página seguinte chegar, o mesmo molde serve: class list
   -> receita -> widget fino -> página do showcase espelhando o template.
2. **um ring css é uma sombra.** `0 0 0 1px #D4D4D4` é `ShadowSpec::new
   ([0,0], 0, 1, color)`; `inset 0 1px 0 white/33` é `ShadowSpec::inset`.
   não inventar um tipo novo pra cada camada de `box-shadow`: a lista
   inteira entra como está, na ordem css (primeira em cima), e o
   `paint_surface` desenha drops antes do fill e insets depois.
3. **diagnosticar cor por pixel, nunca por olho.** a franja escura nas
   quinas parecia "máscara" e "outline"; um `getpixel` numa captura do
   canto do card deu 189 numa borda ECECEC sobre FCFCFC, mais escuro que
   os dois. só isso já dizia: algo mistura com preto transparente. o
   manual já pedia isso (seção color); vale repetir porque a tentação de
   "ajustar o raio" era grande e teria sido band-aid.
4. **mix em premultiplicado linear, sempre.** a causa era um `mix` em
   alpha reto dentro do shader sdf; a correção é uma função
   `premultiply` e todo `mix` sobre ela. um grep por `mix(` em
   `shaders/*.wgsl` é o teste de regressão mais barato que existe.
5. **embutir um view de app pede uma origem, não um refactor.**
   `set_frame(x, y, w, h)` + `origin` nos hit tests + `begin_frame` no
   host bastou pra colocar o workspace do ide numa aba. o rail do comps
   ganhou `origin` pelo mesmo motivo. o que ainda centra na janela
   (header do ide, modal, toasts) é a próxima fronteira.
6. **worker acorda o loop, o view drena no tick.** o git roda numa
   thread; o callback `Wake` manda `UserEvent::Wake`, o showcase
   invalida, e `GitSection::tick` aplica os eventos parados no canal.
   nenhum `about_to_wait` polando.
7. **argumento de launch pra snapshot.** `showcase Buttons core 1400`
   abre na seção, no tema e rolado; sem isso não dá pra capturar uma
   página inteira de 4000px com uma janela de 800.
8. **rebase onto no lugar de resolver conflito de squash.** o PR
   anterior entrou por squash; a branch ainda tinha os commits originais
   e o github via a mudança duas vezes. `git rebase --onto origin/main
   <último commit já mergeado>` replica só o novo. antes, conferir que
   `git diff <esse commit> origin/main` é vazio.

## o que mordeu

- **`zsh` e `set -- $pair` dentro de `for`.** o loop perdeu os
  posicionais e escreveu em `views/.rs`. loop com python ou um `while
  read a b` é mais seguro pra editar vários arquivos.
- **capturar a janela enquanto o usuário navega nela.** metade das
  capturas era de outra seção ou outro tema. quando for medir pixel,
  subir uma instância própria (`pgrep -n` pega o pid mais novo) ou
  avisar antes.
- **o showcase renderiza on demand e não tem como ser acordado de fora**
  até esta sessão; o urnaui já tinha resolvido com `EventLoopProxy`. a
  peça devia estar no engine (`engine::window`), não em cada app.
- **`comps::editor::handle_key` recebe `winit::keyboard::Key`.** o view
  "winit-free" do showcase agora importa winit por causa disso. um enum
  de tecla próprio no comps (o showcase já tem `EditKey`) desacoplaria.
- **`WidgetEvent::MouseDown` não carrega modificadores.** alt+click do
  editor precisou de `set_modifiers` espelhado no view.
- **número literal em doc envelhece** ("15 seções" em três lugares).
  apontar pro símbolo (`Section::ALL`).
- **arquivo grande esconde responsabilidade.** `view/mod.rs` do showcase
  tinha 1543 linhas (estado, input, render, seções, helpers, testes);
  `tests/widgets.rs` 1351; `icons.rs` 955 com tabela e parser juntos.
  divididos nesta sessão pelo mesmo padrão do ide (`mod` / `input` /
  `render`, testes em arquivo próprio, dados em `data.rs`).

## fica pra próxima

- migrar as páginas de cards e inputs do kit, nessa ordem, depois
  menu/dropdown, modal, toolbar/topbar/sidebar, notifications, prompt
  input, comment, iconography (cada uma apaga o widget antigo antes)
- header do ide, `Modal::dialog_rect` e `ToastManager` centrando num
  rect, não na janela
- diff panel vazio do ide: um `EmptyState` ("select a file or commit")
- `Waker` no engine; enum de tecla no comps; `mods` no `WidgetEvent`
- `apps/urnadbapp/src/view/{search,chunks,overview}.rs` e
  `comps/src/editor/tests.rs` passam de 639 linhas; dividir quando forem
  tocados de verdade
