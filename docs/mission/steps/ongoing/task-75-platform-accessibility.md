---
project: plev
audience: [ai-agents, contributors]
status: ongoing
last-updated: 2026-09-25
domain: generative-runtime
task: task-75
wave: trilha
depends: [task-60]
branch: task/75-a11y-platforms
---

# task-75: accessibility de plataforma production-ready

## objetivo

a accessibility que a task-60 projeta do contrato passa a funcionar bem
com os leitores de tela reais de cada plataforma, com navegação de teclado
completa e com as semânticas de foco e ação que um produto grande exige
para poder adotar o plev.

## ligação com a tese

seção 21 (esse ponto sozinho mata frameworks alternativos; estar presente
e estar production-ready são coisas diferentes; VoiceOver, NVDA, TalkBack,
keyboard navigation, focus semantics e action semantics são cidadãos do
runtime).

## estado atual no código

- depois da 60: árvore AccessKit a partir do snapshot, `NodeId` estável,
  ações do leitor pelo dispatcher, ordem de foco semântica, ponte no
  showcase, verificado com VoiceOver no macOS.
- AccessKit 0.24 com `accesskit_winit` 0.32. alvos do plev: macOS e web
  shipando; android e ios em andamento (tasks 18 e 19); linux e windows
  pendentes.
- web: AccessKit não tem adapter para navegador.
- recipe de focus ring em `crates/comps/src/recipe.rs`; reduced motion e
  contraste wcag declarados como requisitos nativos no AGENTS.md.

## escopo

1. matriz por plataforma, com o suporte real do AccessKit verificado (não
   presumido): VoiceOver no macOS e no iOS, TalkBack no Android, NVDA e
   Narrator no Windows, Orca no Linux (AT-SPI). onde o adapter não existir
   ou não servir, o registro diz isso.
2. web: decisão por ADR entre um espelho DOM oculto com ARIA gerado a
   partir do snapshot e nenhuma accessibility no web por enquanto; se o
   espelho for o caminho, ele é outra projeção do mesmo snapshot, nunca uma
   segunda fonte.
3. teclado: Tab e Shift-Tab pela ordem semântica, setas em listas, menus,
   tabs e árvores (padrões WAI-ARIA), trap de foco em modal e retorno do
   foco ao fechar, atalhos anunciados.
4. live regions: toast e status anunciados sem roubar foco.
5. reduced motion (a preferência do sistema desliga transições e springs)
   e contraste (verificação dos pares de token de texto e fundo contra
   wcag AA nos temas).
6. roteiros manuais por plataforma, executados e com resultado registrado
   neste arquivo ou num anexo.

## fora de escopo

accessibility de texto rico (74, fase 9); bring-up de linux e windows
como alvo (arc: pendente; esta task só entra lá quando o alvo existir).

## dependências

60 integrada; por plataforma, o alvo precisa rodar.

## superfície sob responsabilidade

`crates/engine/src/accessibility/**`, os shells dos apps na parte da
ponte, `crates/comps` na parte de teclado e foco dos widgets, testes de
contraste em `crates/engine/src/theme` (só testes), ADRs por decisão.

## interfaces que consome

a ponte e o construtor de árvore (60), `Snapshot` (57), dispatcher (58).

## interfaces que pode alterar

`crates/engine/src/accessibility` por inteiro; widgets de comps na parte de
teclado com aprovação.

## critérios de aceitação

- para cada plataforma com alvo rodando: roteiro executado com o leitor
  real sobre o showcase (todo, formulários, modal, tabs, lista) e o ide,
  resultado registrado.
- testes automáticos de árvore para toda página do showcase.
- teste de contraste dos tokens de texto nos temas.
- teclado completo nas páginas de componente, com teste.
- `./script/gate` verde.

## testes esperados

asserções de árvore por página, navegação de teclado simulada, contraste,
reduced motion desligando animação.

## riscos e perguntas abertas

- suporte do AccessKit por plataforma muda por versão; atualizar a versão é
  decisão com aprovação.
- teste manual não roda em CI; o registro é a evidência.

## commits esperados

um grupo por plataforma e por item (teclado, live regions, reduced motion,
contraste), com `task-75:` no início.

## pare e reporte se

- o web exigir o espelho DOM (decisão por ADR);
- um leitor real se comportar diferente do que a árvore promete.

## fase 1: a sonda (2026-09-25)

o que segue foi lido na fonte das versões travadas no `Cargo.lock`
(accesskit 0.24.0, accesskit_winit 0.32.2, accesskit_macos 0.26.1,
accesskit_windows 0.32.1, accesskit_unix 0.21.1 com
accesskit_atspi_common 0.18.1, accesskit_consumer 0.35.0 para o windows e
0.36.0 para o macos e o unix, winit 0.30.13), no registry local do cargo.
cada afirmação traz arquivo e linha; nenhuma vem de documentação.

### (a) a matriz por plataforma

quem escolhe o adapter é o `accesskit_winit` 0.32.2, em
`src/platform_impl/mod.rs`: windows (9-11), macos (13-15), linux e os bsd
com a feature `accesskit_unix` (17-28), android só com a feature
`accesskit_android` (30-32), e todo o resto cai em `null.rs` (34-50), um
adapter que não faz nada (`null.rs:21`, `update_if_active` vazio). o
workspace pede `accesskit_winit = "0.32"` com as features padrão
(`Cargo.toml:128`), que ligam `accesskit_unix` e não ligam
`accesskit_android`.

| plataforma | adapter no lock | papéis | ações | live regions | foco | texto |
|---|---|---|---|---|---|---|
| macOS | `accesskit_macos` 0.26.1 | `ns_role` em `node.rs:36-231` e `ns_sub_role` em `node.rs:233+`, todos os papéis do AccessKit; `Status` vira grupo com subrole `AXApplicationStatus` (161, 273), `Alert` grupo com `AXApplicationAlert` (90, 238), `Switch` checkbox com subrole switch (82, 275), `Tab` radio com subrole tab (165, 276), `ComboBox` popup button (102), `ListItem` grupo (48) | `Click` por `accessibilityPerformPress` (668), `accessibilityPerformPick` (1112) e `setAccessibilitySelected` (1041); `Focus` por `setAccessibilityFocused` (638); `SetValue` por `setAccessibilityValue` (555); `Increment` e `Decrement` (687, 706); `ScrollIntoView` por `AXScrollToVisible` (1180-1205); custom actions: nenhuma ocorrência em `src/` | anúncio `NSAccessibilityAnnouncementRequestedNotification` na janela (`event.rs:65-101`); `Assertive` vira prioridade alta, o resto média (`event.rs:35-44`); dispara quando um nó live entra (`event.rs:236-239`) ou muda de value (`event.rs:298-306`), e só se o nó tem **value** | `NSAccessibilityFocusedUIElementChangedNotification` quando o foco da árvore muda (`event.rs:133-138`, `314-321`); o foco da janela chega por `process_event` (`accesskit_winit` `platform_impl/macos.rs:48-54`) | text ranges (`node.rs:730-1006`) só sobre nós `TextRun`; o plev não emite `TextRun`, então um campo expõe o value inteiro, sem caret nem seleção |
| iOS | nenhum | nenhum | nenhuma | nenhuma | nenhum | nenhum |
| android | nenhum no lock: `accesskit_android` 0.7.2 é dependência opcional do `accesskit_winit` (`Cargo.toml:111-114` do crate), e a feature está desligada | (o adapter existiria em `platform_impl/android.rs`, injetado no `mSurfaceView` da GameActivity, 27-40) | nenhuma | nenhuma | nenhum | nenhum |
| windows | `accesskit_windows` 0.32.1 | `control_type` em `node.rs:55+`; `Status` vira `StatusBar` (191), `Alert` vira `Text` (102), `Switch` vira `Button` (99), `Tab` vira `TabItem` (195) | patterns em `node.rs:1260-1420`: `Invoke` (`Click`, 1268), `Toggle` (1261, só com toggled e sem selection item, 559-561), `Value` (`SetValue` com texto, 1273-1282), `RangeValue` (`SetValue` numérico, 1284-1296), `SelectionItem` (1310), `ScrollItem` (1298); `Focus` por `SetFocus` (1067); sem `Increment`, `Decrement` e `ExpandCollapse` (grep: zero) | propriedade `LiveSetting` (`node.rs:534-540`, 1248); evento `UIA_LiveRegionChangedEventId` quando o nó live entra (`adapter.rs:225-232`) ou muda de **nome** (`adapter.rs:271-281`) | `UIA_AutomationFocusChangedEventId` (`adapter.rs:32`, 298) | `Text` pattern (1357) sobre `TextRun` |
| linux (AT-SPI) | `accesskit_unix` 0.21.1 sobre `accesskit_atspi_common` 0.18.1 | `node.rs:65+`; `Status` vira `StatusBar` (229), `Alert` vira `Notification` (71), `Switch` vira `ToggleButton` (93), `Tab` vira `PageTab` (231) | interface Action com uma ação só, `click`, quando o nó é clicável (`node.rs:467-480`, `do_action` 980-990); `Focus` por `grab_focus` (1044-1052); `SetValue` numérico por `set_current_value` (1604-1611); sem `SetValue` de texto (nenhuma EditableText), sem `Increment` e `Decrement` | `ObjectEvent::Announcement(nome, politeness)` quando o nó live entra (`adapter.rs:71-76`) ou muda de **nome** (`node.rs:543-556`) | `StateChanged(Focused)` (`adapter.rs:322-340`) | interface Text sobre `TextRun` |
| web | nenhum: o AccessKit do lock não tem adapter de navegador | o `Bridge` do engine não compila em wasm32 (`accessibility/mod.rs:13-14`, `bridge.rs:16`) | nenhuma | nenhuma | nenhum | nenhum |

o que isso diz ao plev:

- só o macOS está servido hoje, e é o único alvo que roda com o
  VoiceOver. windows e linux têm adapter completo no lock, mas o alvo
  ainda não existe (arc: pendente). iOS e android não têm adapter nesta
  versão: ligar o do android é uma crate nova no lock (fora desta fase) e
  o do iOS não existe no `accesskit_winit` 0.32.2.
- o anúncio de live region diverge por plataforma: o macOS fala o
  **value** do nó, o windows e o AT-SPI falam o **nome**. uma status que
  guarda a mensagem num estado e tem nome fixo (`silences.status`,
  `surface_page.status`) só pode ser ouvida nos três se nome e value
  mudarem juntos.
- nenhum adapter do lock implementa custom actions, embora o contrato do
  AccessKit 0.24 as tenha. um nó com várias ações (o corte da Silences:
  `preview_segment`, `apply_cut`, `reject_cut`) só chega ao leitor se a
  página projetar cada ação como um controle próprio.
- o windows e o AT-SPI não têm `Increment` e `Decrement`: um slider lá se
  ajusta só por `SetValue`.

### (b) a árvore de hoje

rodei a projeção de cada página da galeria (82, a tabela inteira de
`view/pages/table.rs`), montei a árvore como a ponte monta
(`accessibility::build` sobre um frame desenhado) e percorri cada nó.

- 76 das 82 páginas não projetam nada: o leitor ouve a janela e só. as 6
  que projetam são Toggle Group (mostra o exemplo do todo), Todo,
  Surface, Silences, Files e Capture. `ShowcaseView::semantics`
  (`view/dispatch.rs:358`) só conhece cinco panes; toda página de
  componente fica de fora.
- sete botões focáveis sem `Click`: `surface_page.propose`,
  `silences.detect`, `silences.apply_all`, `silences_surface.propose`,
  `silences_surface.close`, `files.open`, `files.warn`. a ação deles tem
  nome próprio (`propose`, `detect_silence`, `apply_all`, `close`,
  `open_file`, `warn`) e o `action::primary` só aceitava `activate`,
  `toggle`, `submit` e `select`. o leitor focava e não conseguia apertar.
- dois grupos sem nome na quick todo da Surface (`todo_panel.entry`,
  `todo_panel.counts`): o leitor para em "grupo" e não ouve nada.
- as linhas de status com a mensagem num estado (`silences.status`,
  `surface_page.status`) chegam à árvore só com o nome ("status",
  "proposal"); a mensagem nunca era lida.
- o foco: a ordem do Tab e a ordem em que o leitor anda batem em todas as
  páginas, e toda página aceita o foco que o leitor pede. mas treze
  controles aceitam o foco e não o projetam de volta (os filtros do todo,
  os filtros e checkboxes da quick todo, os selects de proposer, o detect
  e o apply all da Silences): a árvore continua dizendo que o foco é da
  janela.
- ações declaradas que nenhuma ação da árvore alcança: o `remove` de cada
  linha do todo, as três ações de cada corte da Silences, o `select` com
  parâmetro, o `open` e o `close` dos selects de proposer, o `drop_files`
  da Files.
- o campo do todo (`todo.input`, e o da quick todo) oferece `Click`
  enquanto o `submit` está desligado (campo vazio): o dispatcher recusa e
  o leitor não ouve nada acontecer. a task-60 fez do `submit` o aperto do
  campo, e o teste da página conta com esse `Click`, então fica como está.
- papel: a linha do todo (`todo.item[n]`) é um `ListItem` que alterna; o
  macOS mapeia `ListItem` para grupo e manda o toggled como value
  booleano. o certo é a linha conter um checkbox, como a quick todo faz.
  os itens da quick todo são grupos com o nome "1", "2", "3" (a chave).
  `capture.level` é uma barra de progresso cujo número está em `level`, e
  a árvore só lê `value`.
- nenhum nó sem nome entre os controles, nenhum controle interativo fora
  do foco, nenhuma ação oferecida que o dispatcher recuse por política.

### (c) live regions

- o AccessKit 0.24 tem `Live` com `Off`, `Polite` e `Assertive`
  (`accesskit` `lib.rs:534-538`, setter em 2064), e o consumer herda o
  live do pai (`accesskit_consumer` 0.36.0 `node.rs:897-901`).
- hoje nada no engine liga live: nenhuma status nem alerta é anunciada
  sem foco.
- o toast não está na árvore. o `ToastManager` do comps sabe se projetar
  (`feedback/toast.rs:161`, `Status` ou `Alert` por intent), mas os toasts
  da galeria (`ShowcaseView::toasts`) não entram em
  `ShowcaseView::semantics`.
- `shell:notice` não está na árvore: é estado da região `shell` em
  `gallery_semantics` (`view/dispatch/shell.rs:218`), e a ponte monta a
  árvore de `view.semantics()`, a página sozinha (`app/pace.rs:55`).
- as status da Surface e da Silences estão na árvore, sem live e sem a
  mensagem (ver (b)).

### (d) reduced motion

- o winit 0.30.13 não expõe a preferência: nada em `src/` fala de reduzir
  movimento; o que ele dá é `WindowEvent::Focused(bool)`
  (`event.rs:197`), útil para reler quando o app volta, e
  `ThemeChanged` (`event.rs:397`), só do tema escuro.
- macOS sem dependência nova: processo filho
  `/usr/bin/defaults read com.apple.universalaccess reduceMotion`.
  conferido nesta máquina (macOS 26.6.2): o domínio lê (163 chaves), a
  chave não existe porque nunca foi ligada, e o `defaults` sai com
  código 1 e "does not exist", em uns 10 ms. ligada, ele imprime `1`. o
  `objc2-app-kit` não é dependência direta do engine.
- windows: `SystemParametersInfo` com `SPI_GETCLIENTAREAANIMATION` pede a
  crate `windows`, que o engine não tem direta. linux: `gsettings get
  org.gnome.desktop.interface enable-animations` como processo filho
  daria, mas o alvo não roda. web: `matchMedia("(prefers-reduced-motion:
  reduce)")` pede a feature `MediaQueryList` do `web-sys`, que o engine
  não declara (`Cargo.toml:123`; o winit liga a dele, `Cargo.toml:377` do
  crate, e contar com isso é depender por acaso). iOS e android: UIKit e
  JNI, nenhum dos dois é dependência do engine. os cinco ficam para
  quando o alvo rodar, ou para uma mudança de `Cargo.toml` aprovada.
- o ponto único de animação existe: `engine::animation`. toda transição
  do workspace passa por `Spring::tick` ou `Tween::tick`: no comps o knob
  do switch, o fade da scrollbar, o toast e o overlay (modal e menu); no
  `surface` o `motion.rs`; na galeria o todo, o dock e o reveal do chart;
  no engine o scroll suave de `input/scroll.rs`. os laços ambientes
  (ângulo do spinner, pulso do skeleton, `comps` `feedback/spinner.rs:75`
  e `content/skeleton.rs:48`) somam `dt` por conta própria e ficam de
  fora: são do comps.
- a ponte do engine (`Bridge`) já vê todo evento da janela, é criada na
  janela principal da galeria e na do app genérico do engine
  (`window/lifecycle.rs:81`), e roda na thread da interface. é o lugar de
  ler a preferência sem mexer no shell.

### (e) contraste

301 pares de texto e fundo nos quatro temas (core, hoff, dark, light),
medidos como a janela mostra (token translúcido composto sobre o fundo em
luz linear, como a GPU mistura na superfície sRGB). 102 falham o AA. a
lista inteira, com a razão de cada par, está em
`crates/engine/src/theme/tests_contrast.rs` (`KNOWN`) e no handoff
abaixo.

## fase 1: roteiros manuais

### VoiceOver no macOS (para o brenner rodar)

o que cada passo espera vem da árvore que os testes provam
(`crates/showcase/tests/access_tree.rs`, `app/access.rs`) e do
mapeamento do `accesskit_macos` 0.26.1; a frase exata é do VoiceOver, e
o que ele disser de fato é o registro. onde a coluna diz "gap", é o
comportamento esperado hoje e está na lista da fase 2.

preparo: `cargo run -p showcase -- Todo` (ou a página do passo), VoiceOver
com Cmd-F5, VO é Control-Option. o rotor (VO-U) ajuda a listar controles.
nenhum passo pede o Claude: na Surface, nunca aperte propose com o
proposer em `claude` (é chamada paga); troque para `rules` com o mouse
antes.

| passo | o que fazer | o que ouvir |
|---|---|---|
| todo 1 | VO-Right a partir da janela | "todo", região (landmark) |
| todo 2 | chegar no campo | "new todo", campo de texto, vazio |
| todo 3 | VO-Space no campo vazio | nada acontece (o `submit` está desligado; gap conhecido: o aperto é oferecido e recusado) |
| todo 4 | focar o campo, digitar "buy milk", VO-Space | a linha nova aparece e o contador é **anunciado sem o foco sair do campo**: "3 active · 1 done" (novo nesta fase) |
| todo 5 | andar até os filtros | "All", tab, selecionada, 1 de 3; "Active"; "Completed" |
| todo 6 | andar até uma linha e VO-Space | a linha alterna e o contador anuncia a nova conta; registrar como o VoiceOver lê o estado da linha (é um grupo com value booleano: pode sair "0"/"1" ou nada; gap de papel, a linha deveria conter um checkbox) |
| todo 7 | andar até o contador | "2 active · 1 done", status; registrar se o texto sai duas vezes (nome e value são o mesmo texto) |
| formulário 1 | `-- Silences`, andar até "threshold (dB)" | slider, "-30"; VO-Shift-Up e Down (ou as setas de interação) mudam o valor e ele é lido |
| formulário 2 | "detect silences", VO-Space | aperta (novo nesta fase: antes o botão só focava); a linha de status anuncia a mensagem da detecção, polida |
| formulário 3 | "apply all", VO-Space | aplica; "silences.counts" anuncia a nova conta |
| formulário 4 | os cortes da lista | cada um lido pelo intervalo ("0:03.0 to 0:05.0"); aplicar ou rejeitar um corte pelo leitor não existe (gap: várias ações, nenhum adapter tem custom actions) |
| formulário 5 | `-- Surface`, andar até "intent", digitar | campo de texto; "proposer", popup button, "claude"; VO-Space no proposer não abre a lista (gap) |
| formulário 6 | `-- Files`, "open file", VO-Space | abre o diálogo do sistema (novo nesta fase); cancelar volta; a linha de status anuncia |
| toast 1 | `-- Toast`, com o cursor do VoiceOver em qualquer lugar, clicar "Success" com o mouse | "Saved! Everything went fine." anunciado polido; o cursor do VoiceOver não se move (novo) |
| toast 2 | clicar "Error" | "Failed to push: remote rejected." anunciado com prioridade alta (assertive), por cima do que estiver sendo lido |
| modal 1 | `-- Dialog`, abrir o diálogo com o mouse | gap: o diálogo não é projetado, o VoiceOver só tem a janela |
| modal 2 | confirmar com o mouse | "Settings applied." anunciado (é um toast) |
| tabs | `-- Tabs` | gap: a página de componente não projeta nada; ouvir só a janela. as tabs com árvore são os filtros do todo (todo 5) |
| lista | `-- Todo`, VO-Right pelas linhas | cada linha pelo texto, na ordem da tela; o foco do Tab e o do leitor andam na mesma ordem |
| ide | `cargo run -p ide`, ou a página Git da galeria | gap: o ide não liga a ponte, a página Git não projeta; só a janela |
| reduced motion | Ajustes, Acessibilidade, Tela, Reduzir movimento; voltar à janela do showcase | um toast novo aparece sem deslizar, o modal abre sem fade, o knob do switch pula; desligar e voltar à janela: o movimento volta. com `RUST_LOG=info`, o log diz `accessibility: reduce motion on` e `off` |

### as outras plataformas

- windows (NVDA, Narrator): quando o alvo rodar. conferir o anúncio das
  status: lá o que se anuncia é o nome, então `silences.status` e
  `surface_page.status` ficam mudas até o nome seguir a mensagem.
- linux (Orca): quando o alvo rodar. mesma nota do nome; lá o slider só
  se ajusta por valor, sem incremento.
- iOS (VoiceOver): sem adapter no `accesskit_winit` 0.32.2; quando houver.
- android (TalkBack): quando o adapter entrar no lock (decisão com
  aprovação) e o alvo rodar.
- web: depende do ADR `accessibility-is-production-ready-per-platform`
  (espelho DOM ou nada por enquanto).

## handoff da fase 1 (2026-09-25)

branch `task/75a-a11y`, sobre `main` em `bd03459d`. sem push.

### commits

1. `1d2aa2eb` a sonda e a matriz, pela fonte do lock (seção "fase 1: a
   sonda" acima).
2. `20096a0f` os testes de árvore de todas as páginas
   (`crates/showcase/tests/access_tree.rs`) e as duas correções no
   `accessibility`.
3. `ab951681` live regions: status polida, alerta assertivo, o toast na
   árvore do leitor, o foco parado.
4. `e7ca19c1` contraste dos quatro temas contra o AA
   (`crates/engine/src/theme/tests_contrast.rs`).
5. `d811876f` reduced motion no ponto único de animação, lido pela ponte.
6. este commit: roteiros, ADR e handoff.

### a sonda e a matriz

a matriz está na seção (a) acima. em uma linha: macOS, windows e linux
têm adapter no lock e só o macOS roda; iOS e web não têm adapter;
android tem um opcional e desligado, fora do lock. nenhum adapter tem
custom actions; windows e AT-SPI anunciam o nome de uma live region, o
macOS anuncia o value.

### a árvore: o que estava incoerente e o que foi corrigido

82 páginas, 6 projetam (Toggle Group, Todo, Surface, Silences, Files,
Capture), 76 mostram só a janela. corrigido em
`crates/engine/src/accessibility/**`:

- sete botões que o leitor focava e não apertava (`surface_page.propose`,
  `silences.detect`, `silences.apply_all`, `silences_surface.propose`,
  `silences_surface.close`, `files.open`, `files.warn`): um botão, link
  ou item de menu sem nenhuma das quatro ações primárias e com uma única
  ação sem parâmetro é apertado por ela (`action::primary`).
- os grupos sem nome da quick todo (`todo_panel.entry`,
  `todo_panel.counts`) viram `GenericContainer`, que os adapters tiram da
  árvore da plataforma subindo os filhos.
- as status com a mensagem num estado agora dizem a mensagem (ver live
  regions).

os testes (6 em `access_tree.rs`) seguram: árvore válida em toda página;
todo controle com nome e foco; nenhum grupo mudo; a ordem do Tab igual à
do leitor; a página aceita o foco pedido; toda ação oferecida vira um
pedido que o dispatcher admite de `Actor::Assistive`; toda status é live
e diz suas palavras. nada executa pedido (a Files abriria o diálogo do
sistema, a Surface chamaria o modelo).

### a lista para a fase 2 (comps e páginas; não mexi)

- as 76 páginas sem projeção, todas as de componente. `ShowcaseView::
  semantics` conhece cinco panes.
- ações que nenhuma ação da árvore alcança (lista exata `UNREACHABLE` no
  teste): o `remove` das linhas do todo; `preview_segment`, `apply_cut` e
  `reject_cut` de cada corte da Silences; `select`, `open` e `close` dos
  selects de proposer (as opções não são nós); `drop_files` da Files. a
  saída é a página projetar cada ação como um controle, porque custom
  actions não existem nos adapters do lock.
- foco aceito e não projetado (lista `FOCUS_NOT_PROJECTED`): os filtros
  do todo, os filtros e checkboxes da quick todo, os dois selects de
  proposer, `silences.detect` e `silences.apply_all`.
- o campo do todo e o da quick todo oferecem o aperto (`submit`) com o
  campo vazio, e o dispatcher recusa (lista `OFFERED_DISABLED`); a
  task-60 fez desse aperto o caminho do VoiceOver e o teste da página
  conta com ele.
- papel: `todo.item[n]` é um `ListItem` que alterna (deveria conter um
  checkbox); os itens da quick todo são grupos com nome "1", "2", "3";
  `capture.level` guarda o número em `level`, e a árvore só lê `value`.
- `silences.status` e `surface_page.status` têm nome fixo ("status",
  "proposal"): no windows e no AT-SPI, que anunciam o nome, ficarão
  mudas até o nome seguir a mensagem.
- o toast deveria estar na projeção da própria view (hoje é o shell que o
  põe na árvore do leitor, `app/access.rs`); `shell:notice` só vira live
  region mudando a projeção do shell da view.
- o modal e o menu de contexto (overlays) não são projetados.
- os laços ambientes do comps (spinner, pulso do skeleton) não leem a
  preferência de movimento.
- o teclado completo nos widgets (o escopo original da fase 2).

### live regions

`Status` é live `Polite` e `Alert` é `Assertive` (`tree::live`); o foco
do update nunca muda por elas (teste
`an_announcement_never_moves_the_focus`). o que se ouve é o value: o do
nó, senão o estado `message`, senão o label; mensagem vazia não diz
nada; um nó secreto não é live. o nome continua sendo o label, porque o
windows e o AT-SPI anunciam o nome. o toast: `app/access.rs::shown` põe
os toasts na tela, como o `ToastManager::project` do comps os declara,
no snapshot que a ponte recebe (`app/pace.rs` chama `shown` no lugar de
`view.semantics()`), só enquanto há toast; sem toast o snapshot é o da
página, idêntico. testes: 4 no engine (`tree/tests.rs`), 4 em
`app/access.rs` (toast polido e erro assertivo com o foco no campo, o
contador do todo anunciando a conta nova, toast numa página sem
projeção, página intocada sem toast) e 1 por página em `access_tree.rs`.
efeito colateral do macOS a registrar no roteiro: um nó live que aparece
é anunciado, então trocar de página anuncia as status dela uma vez.

### contraste

301 pares, 102 falham (core 36, hoff 15, dark 11, light 40). razão
medida como a janela mostra (composição em luz linear). nenhum token
mudou. por tema, texto e fundo (texto normal precisa de 4,5; gradiente
de título, intents, acento, borda ativa e o botão ghost, que desenha
ícone, precisam de 3):

| tema | frente | fundos e razão |
|---|---|---|
| core | `colors.text_mid` | bg 4.13, surface 4.13, bg_panel 3.96, bg_hover 3.75, popover 4.13, tooltip 4.43 |
| core | `colors.text_dim` | bg 4.13, surface 4.13, bg_panel 3.96, bg_hover 3.75, popover 4.13 |
| core | `glass.text_default` | bg 4.13, surface 4.13, bg_panel 3.96, bg_hover 3.75, popover 4.13 |
| core | `glass.text_faint` | bg 4.13, surface 4.13, bg_panel 3.96, bg_hover 3.75, popover 4.13 |
| core | `glass.text_placeholder` | bg 1.61, surface 1.61, bg_panel 1.61, bg_hover 1.60, popover 1.61 |
| core | `colors.success` | bg 2.41, surface 2.41 |
| core | `colors.border_active` | bg 1.26, surface 1.26 |
| core | botão warning, texto | rest 3.37 e 3.65 (as duas pontas do gradiente), hover 2.70 e 2.86, active 3.37 e 3.65 |
| hoff | `glass.text_placeholder` | bg 3.86, surface 3.95, bg_panel 3.39, bg_hover 2.73, popover 3.39 |
| hoff | `colors.text_dim` | bg_hover 4.45 |
| hoff | `glass.text_faint` | bg_hover 3.76 |
| hoff | `colors.danger` | bg 2.27, surface 2.34 |
| hoff | `colors.border_active` | bg 2.22, surface 2.26 |
| hoff | botão secondary, texto | hover 1.57 |
| hoff | botão warning, texto | rest 3.27, hover 3.40, active 3.27 |
| dark | `colors.text_dim` | bg 2.82, surface 2.66, bg_panel 2.53, bg_hover 2.33, popover 2.53 |
| dark | `glass.text_placeholder` | surface 4.50 (abaixo de 4,5 na terceira casa), bg_panel 4.33, bg_hover 4.05, popover 4.33 |
| dark | `colors.border_active` | bg 2.16, surface 2.03 |
| light | `colors.text_dim` | bg 4.50 (abaixo de 4,5 na terceira casa), bg_panel 4.12, bg_hover 3.84, popover 4.12 |
| light | `glass.text_active` | bg 3.50, surface 3.53, bg_panel 3.45, bg_hover 3.41, popover 3.45 |
| light | `glass.text_default` | bg 2.11, surface 2.12, bg_panel 2.10, bg_hover 2.09, popover 2.10 |
| light | `glass.text_faint` | bg 1.60, surface 1.61, bg_panel 1.60, bg_hover 1.59, popover 1.60 |
| light | `glass.text_placeholder` | bg 1.31, surface 1.31, bg_panel 1.30, bg_hover 1.30, popover 1.30 |
| light | `glass.text_gradient[1]` | bg 1.89, surface 1.89 |
| light | `colors.warning` | bg 2.15, surface 2.24 |
| light | `colors.info` | bg 2.76, surface 2.87 |
| light | `colors.border_active` | bg 2.73, surface 2.84 |
| light | botão primary, texto | rest 4.06, active 4.06 |
| light | botão secondary, texto | rest 1.09, hover 1.17, active 1.09 |
| light | botão warning, texto | rest 2.15, hover 1.83, active 2.15 |

o que salta: no core, o tema padrão, o cinza `#7B7B7B` (shade6) de todo
texto secundário fica em 4.13 sobre o papel; no light, os textos de
vidro vêm de `GlassTokens::derive` com alfa sobre fundo claro e a receita
derivada do botão secondary dá 1.09. são decisões de design do brenner;
o teste guarda a lista até lá.

### reduced motion

caminho sem dependência nova, no ponto único: `engine::animation`
(`mod.rs`, `spring.rs`, `tween.rs`). a preferência é por thread
(`set_reduced_motion`, `reduced_motion`); ligada, `Spring::tick` pousa no
alvo e `Tween::tick` completa uma animação finita no primeiro tick,
mesmo com passo zero; um tween infinito continua. a leitura é
`accessibility::motion` (`/usr/bin/defaults read
com.apple.universalaccess reduceMotion`, chave ausente é desligado), e
quem a aplica é a `Bridge`: ao nascer e a cada `WindowEvent::Focused(true)`,
na thread do event loop. o shell não mudou para isso. outros sistemas
respondem `None` (sem mudança) até o alvo ou a dependência. testes: 5 em
`animation/tests_motion.rs` (a mola e o tween terminam no primeiro
frame, sem a preferência seguem no meio, o laço infinito continua, a
preferência é da thread), 2 em `accessibility/motion.rs` (a leitura do
`defaults`, e o `defaults` real responde no macOS sem mudar nada), 3 em
`crates/showcase/tests/reduced_motion.rs` (switch, toast, modal e a
galeria inteira assentam no primeiro frame; sem a preferência, estão em
movimento).

### o roteiro do VoiceOver

na seção "fase 1: roteiros manuais" acima: todo, formulário (Silences,
Surface, Files), toast, modal, tabs, lista, o ide e reduced motion, com o
que ouvir em cada passo. as outras plataformas ficam marcadas para quando
o alvo rodar.

### gate e regress

- antes (em `bd03459d`, rodado aqui): `./script/gate` verde, 2932
  passam, 26 ignorados, pernas urnaui e wasm rodaram; `regress check`
  175 baselines, 175 iguais.
- depois (em `d811876f`): `./script/gate` verde, 2960 passam (+28), 26
  ignorados, 0 falhas, pernas urnaui e wasm rodaram; `regress check` 175
  baselines, 175 iguais.
- `Cargo.toml` e `Cargo.lock` intocados (md5 do lock igual), nenhuma
  dependência nova, AccessKit na mesma versão. sempre com
  `PLEV_CLAUDE` no stub e `--offline`; nenhuma janela aberta, nenhum
  leitor de tela ligado.

### linha para o índice de ADRs

| 2026-09-25 | [accessibility-is-production-ready-per-platform](accessibility-is-production-ready-per-platform.md) | proposed | a matriz do que o AccessKit do lock entrega por plataforma (macOS roda; windows e linux esperam o alvo; iOS e web sem adapter; android fora do lock); a árvore de toda página testada; status polida e alerta assertivo sem mover o foco, o toast na árvore do leitor; 301 pares de contraste medidos, 102 falhando listados sem mudar token; reduced motion no ponto único de animação, lido do macOS pela ponte; o web em aberto entre espelho DOM gerado do snapshot e nada por enquanto (task-75 fase 1) |

## fase 2a: a sonda (2026-09-25)

branch `task/75b-a11y-pages`, sobre `main` em `ee67f53c`. a mesma leitura
da fase 1: o código com arquivo e linha, o AccessKit pela fonte do lock
(accesskit 0.24.0, accesskit_macos 0.26.1 e accesskit_atspi_common 0.18.1
sobre accesskit_consumer 0.36.0, accesskit_windows 0.32.1 sobre
accesskit_consumer 0.35.0). o escopo são as páginas que já projetam
(Todo e Toggle Group, Surface com a quick todo, Silences, Files, Capture)
e o shell; as 76 páginas de componente e o teclado nos widgets ficam para
a fase 2b.

### a restrição que decide onde cada correção mora

três coisas leem a projeção de uma página parada, e esta fase não pode
mexer nelas:

- os goldens do `uir`: `crates/uir/fixtures/showcase-todo.snapshot.json`
  é o `ShowcaseView::semantics()` da Todo parada
  (`crates/showcase/tests/uir_fixture.rs:32-42`);
  `showcase-silences.snapshot.json` e `showcase-files.snapshot.json` são
  o `snapshot()` da Silences e da Files paradas
  (`view/apps/silences/tests.rs:861-893`, `view/apps/files/tests.rs:833-860`).
  o IR da quick todo verifica contra o primeiro, e os testes de
  `propose`, `surface` e `claude-model` leem os três. `crates/uir/**` é
  arquivo proibido nesta fase.
- o registro de capacidades do runtime generativo:
  `uir::Capabilities::from_snapshot` sobre a projeção da página
  (`view/apps/surface_page.rs:189-190`, `surface_page/page.rs:263`). um
  controle filho que a página projetasse (`todo.item[1].remove` ao lado
  do `remove` da linha) entraria no registro como capacidade própria: um
  IR poderia ligar um ou outro, e o prompt do modelo levaria os dois.
- o `scene.json` de cada baseline do regress guarda a revisão e a
  contagem da projeção (`examples/regress/baseline.rs:91-96, 114-128`) e
  toda binding do frame; um controle novo com binding muda os dois.

daí a regra desta fase: o que é significado da página muda na página (o
foco que ela toma, o `value` do nível da Capture); o que existe porque os
adapters do lock não têm custom actions, ou porque um adapter lê um papel
de outro jeito, é apresentação e mora na árvore do leitor
(`crates/engine/src/accessibility/tree.rs`), onde a fase 1 já pôs regras
desse tipo (o `GenericContainer`, a ação única de um botão). a árvore
segue sendo projeção do mesmo snapshot: nada nela vem de geometria, e o
que o leitor pede a um controle apresentado volta como o pedido da ação
dona, pelo dispatcher da página, como `Actor::Assistive`. o estado que só
o leitor pode ouvir (a preferência, o aviso do shell) entra no caminho do
leitor da view (`ShowcaseView::reader_semantics`, que sucede o `shown` do
`app/access.rs`), nunca na projeção que o port serve.

### (a) cada item, a causa e a correção

1. ações que nenhuma ação da árvore alcança (`UNREACHABLE`, 22 itens):
   - causa: `action::primary` pega a primeira de `activate`, `toggle`,
     `submit`, `select` sem parâmetro, e a ação única só num botão, link
     ou item de menu (`engine/src/accessibility/action.rs:96-116`); a
     árvore só oferece `Click` pela primária (`tree.rs:255-257`). a linha
     do todo tem `toggle` e `remove` (`view/apps/app/semantics.rs:179-190`);
     o corte da Silences tem `preview_segment`, `apply_cut` e
     `reject_cut` num `ListItem` (`view/apps/silences/semantics.rs:249-273`);
     nenhum adapter do lock implementa custom actions (fase 1, (a)).
   - correção, na árvore: dentro de um item de coleção (`ListItem`,
     `Row`, `TreeItem`), cada ação sem parâmetro que o próprio item não
     alcança vira um botão filho, com o id `<item>[<ação>]` (a forma dos
     botões do modal do comps, `comps/src/feedback/modal.rs:72-77`),
     nomeado pelo `label` da ação ou pelo nome dela ("remove", "apply
     cut"), desligado quando a ação está desligada; o `Click` dele volta
     como o pedido da ação no item (`action::translate`). sem bounds: a
     página não liga retângulo para essas ações, e dar ao filho o
     retângulo da linha faria o hit test do leitor cair nele e não na
     linha (accesskit_consumer 0.36.0 `node.rs:346-373`, filhos primeiro).
   - os selects de proposer (`surface_page.proposer`,
     `silences_surface.proposer`): `Select::project` declara `select` com
     parâmetro, `open` e `close` (`comps/src/form/select.rs:130-179`), e
     as opções só existem com a lista aberta (137-143). correção, na
     árvore: o aperto de um select abre a lista e, aberta, fecha (o aperto
     de um pop-up button); cada opção aberta já se aperta (`select` sem
     parâmetro); e `SetValue` num select sem `set_value` pede o `select`
     com o valor.
   - `files.drop drop_files` fica, com o motivo: um leitor não arrasta
     arquivo, e o `open file` leva ao mesmo lugar.
2. foco aceito e não projetado (`FOCUS_NOT_PROJECTED`, 13 itens):
   - os filtros do todo: `TodoPage::focus` aceita a aba e só tira o foco
     do campo (`view/apps/app/semantics.rs:305-325`); a projeção só
     conhece o campo (196-198). correção: a página guarda a aba focada e a
     projeta; o clique da pessoa, o foco no campo e o Escape a soltam.
   - os filtros e checkboxes da quick todo: o `Surface` do crate
     `surface` só guarda foco de campo (`crates/surface/src/surface.rs:404-415`)
     e aceita qualquer nó seu (492-501); o crate é proibido. correção, na
     página: a Surface (e a Silences, para a superfície gerada dela)
     guarda o nó que não é campo e que o leitor focou, e o projeta depois
     da superfície.
   - os selects de proposer: `SurfacePage::focus` e `SilencesPage::focus`
     só focam o campo de intenção (`view/apps/surface_page.rs:537-551`,
     `view/apps/silences/semantics.rs:474-484`), e `Proposing::project`
     só projeta o foco do campo (`surface_page/proposing.rs:424-426`).
     correção: `Proposing` guarda o foco do select e dos dois botões
     (`Select::set_focused` e `Button::set_focused` do comps desenham o
     anel) e o projeta.
   - `silences.detect` e `silences.apply_all`: o foco da página só
     conhece os sliders (`view/apps/silences/semantics.rs:497-506,
     569-574`). correção: o foco da página cobre os dois botões.
3. papéis:
   - `todo.item[n]`: um `ListItem` que alterna (`view/apps/app/semantics.rs:181,
     185`), e a árvore põe o `toggled` nele (`tree.rs:239-242`); o macOS
     mapeia `ListItem` para grupo (accesskit_macos `node.rs:48`) e manda o
     toggled como value booleano (323-326). correção, na árvore: um item
     cuja primária é `toggle` apresenta o toggle como um checkbox filho
     (`<item>[toggle]`, nomeado pelo `label` da ação ou pela chave que ela
     lê, "completed"), e o item fica sem `toggled` e sem `Click`. o
     contrato não muda: o agente e o IR seguem pedindo `toggle` em
     `todo.item[n]`.
   - os itens da quick todo: o crate `surface` nomeia uma instância de
     repeat pela chave (`crates/surface/src/project.rs:38-41`), "1", "2",
     "3", e o IR não interpola rótulo. correção, na árvore: um grupo cujo
     nome é só a chave do próprio id toma o nome dos textos (`Label`)
     filhos dele, o texto do item.
   - `capture.level`: a barra guarda o número em `level`
     (`view/apps/capture/semantics.rs:117-119`) e a árvore só lê `value`
     (`tree.rs:295-308`); sem valor, o AT-SPI marca a barra como
     indeterminada (accesskit_atspi_common `node.rs:336-339`). correção,
     na página: `value` ao lado de `level` (que agentes e testes já
     leem), no intervalo de 0 a 1.
   - `OFFERED_DISABLED` (o campo do todo e o da quick todo, com o
     `submit` desligado): a árvore oferece `Click` sempre que há primária
     (`tree.rs:255-257`). correção, na árvore: `Click` só com a primária e
     o nó ligados. o caminho do VoiceOver da task-60 (focar, digitar,
     apertar) segue: com texto, o `submit` liga e o aperto aparece. a
     outra saída, o nó desligado, diria ao leitor que o campo não se
     edita.
4. as status com a mensagem num estado: `silences.status` tem o nome
   "status" (`view/apps/silences/semantics.rs:289-292`),
   `surface_page.status` e `silences_surface.status` têm "proposal"
   (`surface_page/proposing.rs:386-390`), e a árvore põe a mensagem só no
   value (`tree.rs:334-341`). correção, na árvore: quando as palavras vêm
   de `message`, o nome também é a mensagem; mensagem vazia deixa o nome
   da página. a projeção não muda (agentes seguem lendo `label` e
   `message`), nem os goldens.
5. o toast, o aviso, o modal e o menu:
   - o toast: é o `shown` que o põe no snapshot do leitor
     (`app/access.rs:25-49`); `ShowcaseView::semantics` não o conhece
     (`view/dispatch.rs:358-376`). correção: a view projeta os toasts em
     `semantics()` enquanto houver um na tela (`ToastManager::project`,
     `comps/src/feedback/toast.rs:161-185`), com `ToastManager::sem` para
     ligar cada toast ao retângulo dele, e o `dismiss` corre pelo
     dispatcher da galeria. o `redeclare` do `app/access.rs` sai (a view
     já tem o mesmo em `detached::extend`).
   - `shell:notice`: estado da região `shell` (`view/dispatch/shell.rs:217-219`),
     que só o `gallery_semantics` tem; a ponte monta a árvore da página
     (`app/pace.rs:53-57`). correção: o caminho do leitor projeta o aviso
     como status (`shell.notice`, live polida) enquanto houver aviso; o
     port segue lendo `shell:notice`.
   - o modal e o menu de contexto: a view guarda o overlay ativo
     (`view/mod.rs:45-56`) e não o projeta. o comps já sabe projetar os
     dois (`Modal::project`, `comps/src/feedback/modal.rs:94-121`, um
     `Dialog` com os botões `[confirm]` e `[cancel]`;
     `ContextMenu::project`, `comps/src/feedback/context_menu.rs:136-166`,
     um `Menu` com um `MenuItem` por item). correção, na view: `semantics()`
     projeta o overlay enquanto ele segura o ponteiro, com `.sem` para as
     bindings, o foco no cancel do diálogo e no primeiro item do menu (a
     página fica inerte por baixo e guarda o foco dela, que volta quando o
     overlay fecha); o confirm, o cancel, o item e o close correm por um
     `Operable` do overlay no dispatcher da galeria, o mesmo caminho do
     clique da pessoa. nada muda no comps.
6. spinner e skeleton: somam `dt` sem ler a preferência
   (`comps/src/feedback/spinner.rs:75-78`, `comps/src/content/skeleton.rs:47-50`),
   e as páginas pedem frame sempre que giram
   (`view/components/spinner.rs:79-87`, `view/components/skeleton.rs:65-73`).
   correção: com `engine::animation::reduced_motion()` ligado, o `tick`
   não anda e diz que nada se move; as páginas pedem frame só quando um
   deles se moveu. o arco parado e o bloco no meio do pulso seguem
   dizendo "carregando". arquivos do comps: `feedback/spinner.rs` e
   `content/skeleton.rs`.
7. a preferência de accessibility: ver (d).

### (b) as baselines

pela regra de cima, uma correção só muda a projeção de uma página parada:
o `value` de `capture.level`. mudam `capture@core@1280x800` e
`capture@hoff@1280x800`: no `semantics.json`, um estado `value` e o
intervalo em `capture.level`; no `scene.json`, só o bloco `semantics` (a
revisão; a contagem de nós fica). frame, camadas, hashes e bindings
ficam. o resto (foco, overlays, toasts, aviso, preferência) não existe
numa página parada, e a apresentação da árvore não passa pela projeção;
os goldens do `uir` ficam intactos.

nota para o orquestrador: o `scene.json` guarda a revisão do
`semantics.json` (`examples/regress/baseline.rs:125-128`), então toda
mudança de `semantics.json` mexe nessa linha do `scene.json`. li a
autorização como cobrindo esse espelho e nada mais; o commit das
baselines vai separado, para poder sair sozinho.

### (c) quem mais lê essas projeções

- `crates/wire` e `crates/ide`: nenhum teste lê a projeção da galeria
  (grep por `ShowcaseView` e `gallery_semantics`: zero), e o ide não liga
  a ponte (grep por `Bridge` em `crates/ide/src`: zero). nada muda.
- o port (`gallery_semantics`, `view/dispatch/shell.rs:355-362`): passa a
  ver os toasts e o overlay aberto, como o ADR da fase 1 pedia ("where an
  agent reads them too"); a preferência não entra.
- o menu nativo (`app/menu.rs:195-212`): o menu da página lista as ações
  simples da galeria, e com o overlay e os toasts no snapshot eles
  entrariam na barra ("Apply", "dismiss"). consequência direta:
  `page_menu` deixa de fora os nós do overlay, como já deixa os do shell.
- a gravação (`app/record.rs:164`) lê `semantics()`: uma sessão gravada
  com um toast na tela passa a guardar o toast. a demo gravada
  (`fixtures/demo/*.monster`) é dado; nada relê a projeção viva dela.
- o laboratório é do ide (`crates/ide/src/bin/lab`) e não lê a galeria.
- os testes da galeria: `tests/access_tree.rs` (as listas encolhem, e a
  árvore passa a vir de `reader_semantics`), os testes do `shown` em
  `app/access.rs` (vão para o caminho novo) e
  `view/apps/app/semantics/assistive.rs` (o campo vazio sem aperto).
- o regress: só a Capture, ver (b).

### (d) a preferência para o leitor

o estado de cada preferência já existe: `SurfaceHost::preferences()`
(`view/apps/silences/generated.rs:269`), que a faixa lê para desenhar
(`generated/access.rs:207-219`). a faixa projeta cinco botões sem estado
(`generated/access.rs:173-189`), e isso fica: é o que o port e o prompt
veem. o caminho do leitor passa a ser `ShowcaseView::reader_semantics()`:
a projeção da view refeita num builder novo (`detached::extend`) só
quando há algo que só o leitor ouve, e a página acrescenta o dela por
`SnapshotBuilder::reopen`. a Silences reabre os quatro botões de
preferência da faixa aberta e dá a cada um `on`; a árvore lê `on` como
`toggled` (`tree.rs:273-291`), e o botão vira toggle button nas três
plataformas (ver (e)). `app/pace.rs` passa esse snapshot à ponte. os
dois testes: o leitor ouve ("larger text" ligado, os outros desligados, e
a mudança quando a pessoa aperta), e nem `semantics()`, nem
`gallery_semantics()` (o que o port serve), nem o `snapshot()` da
Silences (o que o prompt do modelo lê) levam `on`.

### (e) o que muda por plataforma

| item | macOS (accesskit_macos 0.26.1) | windows (accesskit_windows 0.32.1) | AT-SPI (accesskit_atspi_common 0.18.1) |
|---|---|---|---|
| botão apresentado (remove, apply cut) | `AXButton` (`node.rs:65-71`), `accessibilityPerformPress` (668) | `Button` (`node.rs:89`), `Invoke` (1268; `is_invocable`, consumer 0.35.0 `node.rs:638-653`) | `Button` (`node.rs:81-87`), ação `click` (411-413, 980-990) |
| checkbox apresentado da linha | `AXCheckBox` (56), value booleano (323-326), press (668) | `CheckBox` (74), `Toggle` (559-561, 1261-1267) | `CheckBox` (92), estado checked (341-350), `click` |
| select: o aperto abre e fecha | `AXPopUpButton` (102), press (668) | sem `Invoke`: com `expanded` o nó é expand-collapse (consumer 0.35.0 `node.rs:634-653`) e o adapter não tem esse pattern; resta o `Value` (575-580), cujo `SetValue` (1273-1282) pede o `select` | `ComboBox` (96), `click` |
| select: o valor | `setAccessibilityValue` quando o nó tem `SetValue` (1238-1239, 555-575) | `Value` pattern, não só leitura num combo box (consumer 0.35.0 `node.rs:814-830`) | sem `SetValue` de texto: pelas opções abertas |
| foco projetado | `FocusedUIElementChanged` (`event.rs:133-138`) | `AutomationFocusChanged` (`adapter.rs:298`) | `StateChanged(Focused)` (`adapter.rs:322-340`) |
| nome da status seguindo a mensagem | já ouvia pelo value (`event.rs:298-306`); o nome igual é lido junto, como no contador do todo | anuncia o nome (`adapter.rs:271-281`): passa a ouvir | anuncia o nome (`node.rs:543-556`): passa a ouvir |
| toast, aviso | anúncio pelo value (`event.rs:236-239`) | `LiveRegionChanged` quando entra (`adapter.rs:225-232`) | `Announcement` quando entra (`adapter.rs:71-76`) |
| diálogo | `AXWindow` com subrole de diálogo (`node.rs:112, 249`) | `Window` (128-133) com `IsDialog` (694-696) | `Dialog` (104) |
| menu | `AXMenu` (83), `AXMenuItem` (52) | `Menu` (100), `MenuItem` (70) | `Menu` (191), `MenuItem` (193) |
| `capture.level` com valor | `AXProgressIndicator` (146), value numérico (334-336) | `RangeValue` (582-584) | `ProgressBar` (204) com `Value` (431-433, 512-514), e deixa de ser indeterminada (336-339) |
| grupo nomeado pelo texto | `AXGroup` (123) com título | `Group` (144), nome pelo label (398-406) | `Panel` (156) com nome |
| preferência ligada | `Button` com toggled vira `AXCheckBox` (65-69) | `Toggle` (559-561) | `ToggleButton` (81-87) com `Pressed` (344-345) |
| spinner e skeleton | nada: é desenho | nada | nada |

o controle apresentado não tem bounds: no macOS o frame dele é zero
(`node.rs:451-472`), no AT-SPI ele fica sem a interface Component
(`node.rs:415-417`). o leitor chega nele andando pela árvore, não
passando o mouse. iOS, android e web seguem sem adapter no lock (fase 1):
nada daqui chega a eles, e tudo compila para eles (a ponte é só desktop).

## fase 2a: o roteiro do VoiceOver (para o brenner rodar)

o que cada passo espera vem dos testes (`crates/showcase/tests/access_tree.rs`,
`view/dispatch/overlay/tests.rs`, `app/access.rs`,
`view/apps/silences/tests/generated/access.rs`) e do mapeamento do
`accesskit_macos` 0.26.1; a frase exata é do VoiceOver, e o que ele
disser de fato é o registro. o preparo é o da fase 1 (`cargo run -p
showcase -- <página>`, VoiceOver com Cmd-F5, VO é Control-Option). os
passos da fase 1 que não aparecem aqui seguem valendo; estes são os que
mudam ou são novos. nenhum passo pede o Claude: na Surface e na Silences o
proposer fica em `rules`.

| passo | o que fazer | o que ouvir |
|---|---|---|
| todo 3 | VO-Space no campo vazio | o aperto não é oferecido (o `submit` desligado não entra na árvore); o campo segue "campo de texto", editável, não "dimmed" |
| todo 4 | digitar "buy milk", VO-Space | agora o aperto existe e adiciona; o contador anuncia sem o foco sair do campo |
| todo 5 | andar até o filtro "Active" com o foco do teclado seguindo o cursor do VoiceOver | "Active, tab"; VO-Shift-F4 (cursor para o foco do teclado) fica em "Active": a página projeta o foco (antes voltava para a janela) |
| todo 6 | andar até uma linha, VO-Shift-Down para entrar | a linha pelo texto, sem estado; dentro dela "completed, checkbox, unchecked" e "remove, button" (novos). VO-Space no checkbox marca a linha e o contador anuncia; VO-Space em remove tira a linha |
| todo 6b | registrar o cursor do VoiceOver nos dois controles de dentro da linha | eles não têm retângulo: o cursor pode aparecer vazio ou no canto. registrar se VO-Right chega neles e se passar o mouse sobre a linha diz a linha (deve dizer), não o botão |
| formulário 2 | "detect silences", VO-Space | aperta; com o foco do teclado seguindo o cursor, VO-Shift-F4 fica no botão (o foco agora é projetado); a status anuncia a mensagem da detecção |
| formulário 4 | andar pelos cortes, entrar num | o corte pelo intervalo; dentro, "preview segment", "apply cut", "reject cut", botões (novos). VO-Space em "apply cut" aplica e "silences.counts" anuncia; o "apply cut" de um corte já aplicado fica "dimmed" |
| formulário 5 | `-- Surface`, "proposer, pop-up button", VO-Space | abre a lista (novo); entrar nela: "rules", e "claude" onde houver o programa; VO-Space em "rules" escolhe e fecha. não apertar propose com `claude` |
| formulário 5b | a quick todo, andar pelos itens | cada item pelo texto do todo ("Absorb the plev demos into the showcase"), não "1", "2", "3"; dentro, o texto e "done, checkbox" |
| formulário 5c | no "intent" escrever "xyzzy", VO-Space em propose | a linha de status anuncia o porquê da recusa, polida, sem tirar o foco do campo; é o value no macOS, e o nome agora é a mesma frase (é o que o windows e o AT-SPI falariam) |
| modal 1 | `-- Dialog`, abrir o diálogo com o mouse | o VoiceOver vai para "Cancel, button" dentro de "Apply settings?", diálogo, com o texto como descrição; a página some da navegação (inerte) |
| modal 2 | VO-Space em "Apply" (ou em "Cancel") | o diálogo fecha; com Apply, "Settings applied." anunciado; o foco volta para onde estava na página |
| menu 1 | `-- Context Menu`, clique direito na área | "context menu, menu", o foco em "Stage file, menu item"; "Locked action" dimmed |
| menu 2 | VO-Space em "Copy path" | o menu fecha e "Context menu: Copy path" é anunciado |
| preferência 1 | `-- Silences`, no "intent" escrever "accessibility", propose | a faixa aparece; "larger text, checkbox, unchecked", o mesmo para "more contrast", "denser", "sparser"; "done, button" |
| preferência 2 | VO-Space em "larger text" | "checked"; a superfície aberta aumenta o texto; VO-Space de novo: "unchecked". o port e o prompt do modelo não levam esse estado (os testes provam) |
| aviso | apertar "detect silences" e trocar de app antes de terminar (ffmpeg no PATH) | registrar se o VoiceOver fala o aviso ("silences detected...") com a janela sem foco; o nó existe na árvore do leitor, polido; o macOS pode calar um app em segundo plano |
| nível | `-- Capture`, "microphone level" | uma barra de progresso com valor (0 sem microfone), não indeterminada |
| reduced motion 2 | Reduzir movimento ligado; `-- Spinner`, "Start spinners"; `-- Skeleton`, clique numa linha | os arcos ficam parados, o pulso fica parado; desligar e voltar à janela: giram e respiram |

no windows (NVDA, Narrator) e no AT-SPI (Orca), quando o alvo rodar: a
linha de status de uma proposta e a da Silences passam a ser anunciadas
(o nome segue a mensagem); no windows o proposer não abre pelo Invoke (o
nó é expand-collapse e o adapter não tem esse pattern) e se escolhe pelo
valor; no AT-SPI ele abre pelo `click` e se escolhe pelas opções.

## handoff da fase 2a (2026-09-25)

branch `task/75b-a11y-pages`, sobre `main` em `ee67f53c`. sem push.

### commits

1. `168b240e` a sonda (seção "fase 2a: a sonda" acima).
2. `b7059376` as ações que o leitor não alcança viram controles que a
   árvore apresenta dentro do item; o select abre e fecha pelo aperto e
   se escolhe pelo valor.
3. `d6a6def9` as páginas projetam o foco que tomam.
4. `bcc57315` os papéis: o grupo nomeado pela chave toma o texto de
   dentro, `capture.level` com `value`, o aperto só com a ação ligada.
5. `b8616294` a mensagem de uma status é o nome dela também.
6. `61bdfcec` o toast, o modal e o menu de contexto na projeção da view,
   pelo dispatcher; o aviso do shell como status só na árvore do leitor;
   o `app/access.rs` sem o remendo. `24df5d78` o teste da barra de menu.
7. `17c903c9` e `f208fbab` spinner e skeleton parados com reduced motion.
8. `0ff5e28d` o leitor ouve qual preferência está ligada; o port e o
   prompt não.
9. `f3e9f914` as baselines da Capture.
10. este commit: roteiro, ADR e handoff.

### onde cada correção mora (o desvio do briefing)

o briefing propunha a página projetar cada ação de uma linha como um
controle filho. a sonda mostrou que a projeção de uma página parada é
golden do `uir` (`crates/uir/fixtures/showcase-{todo,silences,files}.snapshot.json`,
arquivo proibido nesta fase), é o registro de capacidades que o IR e o
prompt leem (os controles entrariam como capacidades duplicadas) e entra
no `scene.json` do regress (revisão e bindings). por isso os controles
das linhas, o checkbox da linha, o aperto do select, o nome da status, o
nome dos itens da quick todo e o aperto só com a ação ligada são
apresentação da árvore, no `crates/engine/src/accessibility/**`
(`present.rs` novo, `tree.rs`, `action.rs`, `ids.rs`, `mod.rs`); o
contrato não mudou. o que é significado da página mudou na página (o
foco, o `value` do nível). o comps mudou só em `feedback/spinner.rs` e
`content/skeleton.rs` (e os testes deles); o modal e o menu de contexto
já sabiam se projetar, nada mudou neles.

### cada item

1. ações como controles: um item (`ListItem`, `Row`, `TreeItem`) mostra
   dentro dele um botão por ação sem parâmetro que ele mesmo não alcança,
   `<item>[<ação>]`, sem bounds; o `Click` volta como a ação do item,
   `Actor::Assistive`, pelo dispatcher. o select: aperto é `open` (e
   `close` aberto), `SetValue` é `select`. `drop_files` fica.
2. foco: a Todo guarda a aba; a Surface e a Silences guardam o nó da
   superfície que não é campo (o crate `surface` só guarda o de campo); o
   `Proposing` guarda o select e os botões; a Silences, os dois botões. o
   clique da pessoa e o Escape soltam.
3. papéis: a linha do todo contém o checkbox "completed" (o toggle
   apresentado; a linha sem `toggled` e sem `Click`); os itens da quick
   todo têm o nome do texto; `capture.level` tem `value` de 0 a 1 (o
   `level` ficou); `OFFERED_DISABLED` resolvido tirando o aperto quando a
   ação está desligada: o campo continua editável, e o caminho da
   task-60 (focar, digitar, apertar) segue, porque o texto liga o
   `submit`.
4. status: nome e value são a mensagem; sem mensagem, o nome da página.
   teste por página (`a_status_says_its_new_message_as_its_name_and_its_value`)
   e no engine.
5. toast, aviso, modal e menu: na projeção da view enquanto estão na tela
   (o agente lê também); o foco vai ao cancel do diálogo e ao primeiro
   item do menu e volta à página quando fecham; tudo pelo dispatcher,
   inclusive o clique da pessoa; a barra de menu os deixa de fora. o
   aviso (`shell.notice`) é status só na árvore do leitor; o port segue
   com `shell:notice`.
6. reduced motion: spinner e skeleton parados, as páginas sem pedir frame.
7. preferência: o `reader_semantics` da view refaz a projeção só quando
   há algo só do leitor; a Silences reabre os quatro botões da faixa
   aberta com `on`; a árvore diz toggle button.

### as listas, antes e depois

| lista | fase 1 | fase 2a |
|---|---|---|
| `UNREACHABLE` | 22 (o `remove` das linhas nas 3 páginas do todo, as 3 ações dos 2 cortes, `select`/`open`/`close` dos 2 proposers, o `drop_files`) | 1: `Files: files.drop drop_files` (um leitor não arrasta arquivo) |
| `FOCUS_NOT_PROJECTED` | 13 | 0 |
| `OFFERED_DISABLED` | 4 | 0 |
| páginas que projetam | 6 | 6 (as 76 de componente são da fase 2b) |

### as baselines re-abençoadas

só `capture@core@1280x800` e `capture@hoff@1280x800`. o `frame.png` não
mudou; no `scene.json` só a revisão da projeção:

```diff
   "semantics": {
-    "revision": "dc796abd4976914f",
+    "revision": "e570b657d60aa4a5",
     "nodes": 6
   },
```

no `semantics.json`, em `capture.level`:

```diff
         "level": {
           "kind": "num",
           "value": 0.0
+        },
+        "value": {
+          "kind": "num",
+          "value": 0.0
         }
+      },
+      "constraints": {
+        "range": [
+          0.0,
+          1.0
+        ]
       }
```

o `regress check` antes do bless dizia "capture.level semantics changed:
value none -> 0; constraints changed; visual unchanged". nota: o
`scene.json` guarda a revisão do `semantics.json`, então a linha da
revisão muda junto; li a autorização como cobrindo esse espelho. se o
orquestrador ler o pare de forma estrita, o commit `f3e9f914` e o
`value` de `capture/semantics.rs` saem juntos, e o leitor volta a não ter
o número da barra.

### os testes da preferência

em `view/apps/silences/tests/generated/access.rs`:
`a_screen_reader_hears_which_preference_is_on` (a faixa aberta, os
quatro botões desligados na árvore do leitor, "larger text" ligado pelo
aperto do leitor, "more contrast" pelo da pessoa, desligado de novo) e
`the_preference_a_reader_hears_never_goes_out_through_the_port` (com
"larger text" ligado, o agente observa pelo port de verdade os cinco
botões sem estado; `semantics()` e `gallery_semantics()` sem `on`; o
`snapshot()` do app, de onde sai o prompt, sem a faixa; só o
`reader_semantics()` diz). o teste do prompt da task-70
(`a_model_s_prompt_is_the_same_with_and_without_the_preferences`) segue
verde.

### gate e regress

- antes (o briefing): 3062 passam, 27 ignorados, 176 baselines.
- depois, em `f3e9f914`: `./script/gate` verde, pernas urnaui e wasm
  rodaram; 3086 passam (+24: 3001 no workspace, 85 no urnaui), 27
  ignorados, 0 falhas. `regress check`: 176 baselines, 176 iguais.
- `Cargo.toml` e `Cargo.lock` intocados (md5 do lock igual), nenhuma
  dependência nova, AccessKit na mesma versão; `crates/uir`,
  `crates/surface`, `crates/sem`, `crates/wire/src`, `crates/propose` e
  `crates/ide/src` intocados. sempre com `PLEV_CLAUDE` no stub e
  `--offline`; nenhuma janela aberta, nenhum leitor ligado, nenhum clique
  sintético, nenhum diálogo nativo.

### linha para o índice de ADRs

| 2026-09-25 | [accessibility-is-production-ready-per-platform](accessibility-is-production-ready-per-platform.md) | proposed | fase 2a: nas páginas que projetam, as ações de um item que ele mesmo não alcança são controles que a árvore apresenta dentro dele, o select abre pelo aperto e se escolhe pelo valor, o aperto só com a ação ligada, a mensagem de uma status é o nome dela; as páginas projetam o foco que tomam; o toast, o modal e o menu na projeção da view, pelo dispatcher; o aviso do shell e a preferência de accessibility só na árvore do leitor, nunca no port nem no prompt; spinner e skeleton seguem o reduced motion (task-75 fase 2a) |

### o que fica para a fase 2b

- as 76 páginas de componente, que não projetam nada.
- o teclado completo nos widgets: Tab em ordem semântica nas páginas que
  projetam (hoje a Todo não anda com Tab), setas em listas, menus, tabs e
  árvores, o trap de foco no modal (o foco já vai e volta, o teclado
  ainda não o prende) e o anel de foco nas pílulas do todo.
- bounds para os controles apresentados: pede a página ligar um retângulo
  por ação, e isso muda o `scene.json` de baselines existentes.
- o ide não liga a ponte (`crates/ide/src`: sem `Bridge`); a página Git da
  galeria não projeta.
- windows, AT-SPI, iOS e android seguem esperando o alvo ou o adapter.

## fase 2b: a sonda (2026-09-25)

branch `task/75c-a11y-component-pages`, sobre `main` em `3048b320`. a
mesma leitura das fases 1 e 2a: o código com arquivo e linha, o AccessKit
pela fonte do lock (accesskit_macos 0.26.1, accesskit_windows 0.32.1,
accesskit_atspi_common 0.18.1). o escopo são as 76 páginas que não
projetam nada (82 na tabela `view/pages/table.rs`, menos as seis que
projetam). os caminhos abaixo são relativos a `crates/showcase/src/view`
quando não dizem o crate.

### (a) as 76 páginas em três grupos

a regra: grupo 1 quando todo widget que a pessoa opera na página já tem
`project` no comps (os 19 arquivos da task-59); grupo 2 quando a página
mostra um widget sem projeção que carrega uma operação ou um dado que o
leitor precisa ouvir; grupo 3 quando a página não mostra nada que se
opere nem nada a ler além de uma amostra de desenho. uma página alias
(`Body::Alias`, `pages/mod.rs:121`) mostra o pane de outra, então vai
para o grupo do pane.

grupo 1: 30 páginas, 19 panes. o que o clique faz hoje é a referência do
"mesmo estado".

| página (pane) | arquivo | widgets e projeção no comps | o clique hoje |
|---|---|---|---|
| Dialog, Alert Dialog | `components/dialog.rs` | 2 `Button` (`comps/src/action/button.rs:119`) | abre o modal da view (`dispatch.rs:121-127`, `input.rs:247`) |
| Toast | `components/toast.rs` | 4 `Button` | põe um toast (`dispatch.rs:148-155`) |
| Button, Button Group | `components/button.rs` | 30 widgets: `Button`, `IconButton` (`action/icon_button.rs:94`), `SubmitButton` (`action/submit_button.rs:62`), em 14 grupos | nada além da pressão (`button.rs:314-329`) |
| Icon Button | `components/icon_button.rs` | 7 `IconButton` | nada (`icon_button.rs:46-58`) |
| Tooltip, Hover Card, Popover | `components/tooltip.rs` | 1 `Button`; o `Tooltip` não se projeta, o texto dele é a descrição do botão | nada; o hover mostra o tooltip (`tooltip.rs:35-54`) |
| Spinner | `components/spinner.rs` | 1 `Button`; os 3 `Spinner` não se projetam (resto no grupo 2) | liga e desliga o giro (`spinner.rs:64-76`) |
| Checkbox | `components/checkbox.rs` | 3 `Checkbox` (`form/checkbox.rs:63`), um desligado | o widget inverte `checked` (`comps/src/form/checkbox.rs:130-137`) |
| Switch | `components/switch.rs` | 3 `Switch` (`form/switch.rs:70`), um desligado | o widget chama `toggle` (`comps/src/form/switch.rs:159-166`) |
| Chip, Toggle | `components/chip.rs` | 6 `Chip` (`action/chip.rs:74`): 5 etiquetas e "click me" | a página inverte `selected` do "click me" (`chip.rs:58-61`) |
| Slider | `components/slider.rs` | 3 `Slider` (`form/slider.rs:74`): contínuo, com passo, desligado | o arrasto muda `value` no widget |
| Input, Field, Label | `components/input.rs` | 3 `TextField` (`form/text_field.rs:80`) e a linha "live value" | foco e caret (`input.rs:67-89`); digitar insere (`input.rs:91-97`) |
| Select, Combobox | `components/select.rs` | 1 `Select` (`form/select.rs:130`) | o widget abre, fecha e escolhe |
| Tabs | `components/tabs.rs` | 1 `Tabs` (`nav/tabs.rs:64`) | o widget muda `active` (`comps/src/nav/tabs.rs:147-151`) |
| Tree | `components/tree.rs` | 1 `Tree` (`content/tree.rs:146`) | folha: seleciona; ramo: abre ou fecha e seleciona (`comps/src/content/tree.rs:260-272`) |
| Data Table, Table | `components/table.rs` | 1 `Table` (`content/table.rs:127`), 2000 linhas virtualizadas | o widget seleciona a linha (`comps/src/content/list.rs:249-253`) |
| Scroll Area, Item | `components/scroll_area.rs` | 1 `VirtualList` (`content/list.rs:105`), 10000 linhas | seleciona a linha, rola |
| Breadcrumb | `components/breadcrumb.rs` | 1 `Breadcrumb` (`nav/breadcrumb.rs:62`) e o eco "clicked crumb n" | a página guarda o crumb e o desenha (`breadcrumb.rs:35-47, 53-62`) |
| Navigation Menu, Kbd | `components/navigation_menu.rs` | 4 `NavLink` (`nav/nav_link.rs:69`) e a forma de trilho | a página move a marca de ativo (`navigation_menu.rs:64-86`) |
| Resizable | `components/resizable.rs` | 1 `SplitPane` (`nav/split_pane.rs:96`) | o arrasto muda `ratio` |

três projeções que a sonda mostra erradas para páginas do escopo, as três
corrigidas na página (nada no comps):

- o chip "click me" é `interactive` (`chip.rs:23`), e o comps projeta um
  chip interativo como `Button` com `activate`
  (`comps/src/action/chip.rs:80-96`); a página usa o chip como alternância
  (inverte `selected` no clique, `chip.rs:58-61`). correção: o chip passa a
  `selectable(true)`, que é a alternância do comps (um `Toggle` com
  `selected` e `toggle`, e o próprio widget inverte no clique,
  `comps/src/action/chip.rs:170-176`), e a página deixa de inverter. o
  desenho é o mesmo: o chip lê só `pressable()`, que é `interactive ||
  selectable` (`comps/src/action/chip.rs:124-126`, 216).
- o rótulo do botão do Spinner muda dentro do `render`
  (`spinner.rs:101-105`): uma projeção tirada depois do pedido e antes do
  próximo quadro diria "Start spinners" com o giro ligado. correção: o
  rótulo muda junto com o giro.
- a Navigation Menu desenha o primeiro link duas vezes, na coluna e no
  trilho (`navigation_menu.rs:93`); com um `SemId`, o segundo `bind`
  seria recusado com aviso (`engine/src/compositor/binding.rs:147-152`).
  correção: o trilho é um `NavLink` próprio, sem id, que espelha o
  primeiro (ativo e hover).

grupo 2: 24 páginas. o que cada widget precisaria para entrar (papel,
estado, ações); nenhum é escrito nesta fase.

| página (pane) | widget sem projeção | o que precisaria |
|---|---|---|
| Theme | os cartões de paleta desenhados à mão (`foundations/theme.rs:169-194`) | um grupo de opções (`Radio` ou `Option`) com `selected`, cada um apertado pelo `shell.theme select` que já existe (`dispatch/shell.rs`) |
| Avatar | `Avatar` (`comps/src/content/avatar.rs:37`) | `Image` nomeado pelo nome, com `online` |
| Badge | `Badge` (`comps/src/action/badge.rs:27`) | `Label` (ou `Status` quando conta) com o texto e a intenção |
| Stat | `Stat` (`comps/src/content/stat.rs:26`) | `Label` com valor, unidade e o delta como descrição |
| Progress | `ProgressBar` (`comps/src/feedback/progress.rs:12`) | `ProgressBar` com `value` e `range` 0 a 1 (o padrão de `capture.level`) |
| Spinner (o resto) | `Spinner` (`comps/src/feedback/spinner.rs:53`) | `ProgressBar` sem valor (indeterminado) enquanto gira, ou `busy` no pai |
| Skeleton | `Skeleton` (`comps/src/content/skeleton.rs:28`) | um nó "carregando" (`busy`); o pulso ligado pelo clique (`skeleton.rs:50-60`) precisaria de uma ação |
| Code Block | `CodeBlock` (`comps/src/content/code_block.rs:79`) | `Label` com o texto e a legenda como nome |
| Card | `Card` (`comps/src/content/card.rs:140`) | um `Group` por variante com os textos e números dela |
| Chart | os gráficos de `comps::charts` (`components/chart.rs`) | `Image` com um texto alternativo por gráfico (ou uma tabela dos dados); o replay do clique (`chart.rs:95-104`) seria uma ação |
| Graph | `GraphView` (`comps/src/graph.rs:65`) | `Canvas` com um item por nó e `select`; pan e zoom como ações |
| Empty | `EmptyState` (`comps/src/content/empty_state.rs:22`) | `Group` nomeado pelo título, descrito pela mensagem, com o `cta` (`Button`, já projetável) dentro |
| Context Menu, Dropdown Menu | a área de clique direito (`components/context_menu.rs:69-99`) | um nó gatilho com uma ação que abre o menu; o menu aberto já se projeta (fase 2a) |
| Sidebar, Drawer, Sheet | `Sidebar` (`comps/src/nav/sidebar/mod.rs:49`) | uma região de navegação com os `NavLink` (projetáveis) e cada `NavGroup` com `expanded` e uma ação que dobra |
| Bubble, Message, Message Scroller, Dock | o dock desenhado à mão (`apps/dock.rs`) | lista de mensagens (`List`/`ListItem`), o campo (`TextInput`) e os contatos (`Button`) |
| Editor | `EditorView` (`comps/src/editor/view.rs:45`) | `TextInput` multilinha com `text`, seleção e caret (o AccessKit do lock só lê texto sobre `TextRun`, fase 1) |
| Git | as views do ide (`apps/git.rs`) | a projeção das views do ide; o ide está fora do escopo |
| Builder | os elementos do `engine::builder` (`apps/builder.rs`) | um `Button` por região de clique (os botões do contador) e o valor como `Status` |
| Panel Header | `PanelHeader` (`comps/src/nav/panel_header.rs:14`) | um cabeçalho: `Label` com o título, a descrição e o badge |

grupo 3: 22 páginas, nada: não há o que operar nem o que ler além de uma
amostra de desenho, e projetar os textos de amostra só daria ao leitor e
ao agente nós que não mudam nada. o nome da página é do cabeçalho do
shell, não da página.

| página | por quê |
|---|---|
| Typography | amostras da escala de tipo (`foundations/typography.rs:231-233`: nenhum evento) |
| Icons | grade de ícones; só hover (`foundations/icons.rs:53-65`) |
| Effects | amostras de sombra, blur, gradiente (`foundations/effects.rs:154-156`) |
| Panel | a superfície de vidro sozinha (`components/panel.rs:30-37`) |
| Separator | réguas entre legendas (`components/separator.rs:23-30`) |
| 17 placeholders (Accordion, Alert, Aspect Ratio, Attachment, Calendar, Carousel, Collapsible, Command, Date Picker, Input Group, Input OTP, Marker, Menubar, Pagination, Questionnaire, Radio Group, Textarea) | `Body::Placeholder` (`pages/mod.rs:123`): um `EmptyState` que diz "Not in comps yet" (`dispatch.rs:513-527`); o widget não existe |

contagem: 30 + 24 + 22 = 76.

### (b) o mecanismo

hoje `page_semantics` (`dispatch.rs:379-397`), `act` (`dispatch.rs:406-442`)
e `assist_focus` (`dispatch.rs:463-495`) conhecem cinco panes cada, e o
Escape (`input.rs:113-133`) conhece os que têm foco de teclado. o menor
mecanismo:

- um trait `Component` em `view/dispatch/component.rs` (novo, filho de
  `dispatch.rs`, que é o único que casa com `Pane` pela convenção): a
  página projeta seus nós sob uma raiz dela, roda um pedido e diz o que a
  view faz depois (um toast, um modal), toma o foco que um leitor pede e o
  solta no Escape.
- uma tabela só, uma linha por pane (`Pane::Checkbox => checkbox`), escrita
  uma vez por uma macro que gera os dois acessos (`&dyn` e `&mut dyn` com o
  todo emprestado para o dispatcher). são 19 linhas, não 70 braços.
- a view entra por quatro pontos, um braço cada: `page_semantics` projeta a
  página do grupo 1 sob uma região nomeada pelo título da página
  (`Role::Region`, inerte enquanto um overlay segura o ponteiro, como a
  todo em `apps/app/semantics.rs:160-162`); `act` roda o pedido por um
  `Operable` que empresta o dispatcher do todo, o padrão do overlay da fase
  2a (`dispatch/overlay.rs:277-290`); `assist_focus` passa o foco à página
  (recusado enquanto inerte); o Escape solta o foco que o leitor deixou.
- os ids: a raiz é `<pane>_page` (`checkbox_page`, `dialog_page`), o que
  não colide com `dialog`, `toasts` e `context_menu` da view
  (`dispatch/overlay.rs:36-38`); os widgets ganham o id pelo `.sem(id)` do
  comps, então o `render` liga cada retângulo (`bind`) sem mudar um nó da
  cena (`engine/src/compositor/binding.rs:1-12`: a ligação fica ao lado da
  cena, fora de todo hash).

o clique da pessoa: direto no widget, com a projeção seguindo o estado (a
regra da task-59). por quê:

1. os widgets do comps mudam o próprio estado no evento: o checkbox inverte
   `checked` (`comps/src/form/checkbox.rs:130-137`), o switch chama
   `toggle` (`switch.rs:159-166`), as tabs mudam `active` (`tabs.rs:147-151`),
   a árvore abre e seleciona (`tree.rs:260-272`), a lista e a tabela
   selecionam (`list.rs:249-253`), o slider e o split arrastam, o campo
   edita. passar o clique pelo dispatcher pediria desfazer o que o widget já
   fez, ou refazer o hit test dele na página; o comps não muda evento nesta
   fase.
2. toda ação do comps lê o que muda (`toggle` lê `checked`, `on` ou
   `selected`; `select` lê `selected`; `set_value` lê `value` ou `text`), e
   a projeção é leitura pura do widget: um pedido decidido antes de um
   clique da pessoa volta `Stale` com o delta, para o leitor e para o
   agente.
3. o que a página faz por cima do clique (o modal, o toast, o eco do crumb,
   a marca do link, o giro) é um método só, que o clique e o `run` chamam.

o custo: o clique da pessoa numa página de componente não deixa recibo no
dispatcher (o do leitor e o do agente deixam). as páginas de app seguem
passando o clique pelo dispatcher, como antes.

o requisito vira um teste por tipo de widget: a pessoa (clique no
retângulo ligado, ou teclas), o leitor (a árvore, `accessibility::translate`,
`act`) e um agente pelo port de verdade (`wire::AgentPort` e um `Client`
numa thread, como `dispatch/agent.rs`) partem de três views iguais e
chegam à mesma projeção.

### (c) quem mais lê a projeção

- as baselines: cada página do grupo 1 ganha um `semantics.json` e, no
  `scene.json`, o bloco `semantics` (de `null` para revisão e contagem) e o
  bloco `bindings` (os widgets). são 60 baselines (30 páginas, core e
  hoff); nenhum `frame.png` existe nelas. **um efeito que o briefing não
  previu**: o nome de uma camada no `scene.json` vem das raízes dos ids
  ligados nela (`examples/regress/scene.rs:6-12, 104-131`), e uma camada
  sem ligação se chama pelo z (`z0`). ligar os widgets renomeia a camada
  da página de `z0` para a raiz (`checkbox_page`), e na Scroll Area e na
  Item a camada da lista de `z10` para `scroll_area_page`. hash, contagem
  de nós, z e limites ficam; só o nome anda, e ele é espelho das
  ligações, como a revisão no bloco `semantics` foi na fase 2a. li a
  autorização como cobrindo esse espelho; o commit das baselines vai
  separado e um script confirma, camada a camada na ordem de composição,
  que z, hash, nós e limites são iguais e que o nome novo é a junção das
  raízes ligadas. a outra saída seria projetar sem ligar (a árvore do
  leitor sem retângulo nenhum nessas páginas, e o port sem `bounds`). as
  seis que já projetam e os goldens do `uir` não mudam.
- a barra de menu (`app/menu.rs:195-212`): `page_menu` lista as ações
  simples da página ativa. numa página de componente ela mostraria 30
  itens "Button" na Button, "Autosave on focus loss: toggle" na Checkbox,
  "Neutral", "Success" na Toast: amostras de widget, não capacidades da
  página. proposta: deixar fora, como o shell e o overlay já ficam
  (`view.is_component`), com teste; a Toggle Group mostra a todo e segue
  com a barra dela.
- o tray (`app/tray.rs:124-145`) e a notificação (`app/notify.rs:200-204`)
  leem só o shell; a impressão pede `shell.print` (`app/print.rs:48-52`); os
  deep links só pedem o que `OPEN` lista, `shell.page` e `shell.theme`
  (`app/links.rs:46`), e recusam o resto antes do pedido; o sleep lê a
  Silences. nada muda neles.
- a gravação (`app/record.rs:162-165`) passa a guardar a projeção das
  páginas de componente numa sessão gravada; é consequência, não pede
  mudança. a demo gravada (`fixtures/demo/`) é da Silences, dado.
- o port (`dispatch/agent.rs:61-67`) serve `gallery_semantics`, que passa a
  levar a página de componente, com os `bounds` das ligações.
- `crates/wire/tests` e `crates/ide/tests`: nenhum lê a galeria (grep por
  `ShowcaseView` e `gallery_semantics`: zero). o laboratório é do ide.
- os testes da galeria que usam uma página de componente como "página sem
  projeção" mudam de página: `apps/app/semantics/assistive.rs:333-345`
  (Button) e `app/headless.rs:549-560` (Button); o de view source que usa um
  texto formatado da Data Table (`crates/showcase/tests/view_source.rs:147-151`)
  cai dentro de uma linha ligada e muda de exemplo. `tests/access_tree.rs`
  encolhe a lista página por página.

### (d) as promessas de privacidade no escopo

- valor secreto: nenhuma página do grupo 1 tem senha, OTP ou campo marcado
  como segredo (os campos da Input são nome, email e codinome,
  `input.rs:22`); a Input OTP é placeholder (grupo 3).
- pixel de câmera: nenhuma página do escopo tem câmera (só a Capture, fora
  do escopo).
- ação `PERSON_ONLY`: nenhuma página do escopo a demonstra (o
  `PERSON_ONLY` é do shell, da Files e da Capture).
- a preferência de accessibility: só a Silences a tem; nenhuma página do
  escopo.

nenhum caso presente; o teste guarda a ausência: se uma página do grupo 1
passar a projetar um segredo, uma mídia, uma ação que recusa agente ou um
estado só do leitor, ele falha e pede o teste do caso.

### (e) o que cada papel vira por plataforma

a matriz da fase 1 cobre o botão, o switch e a tab; o resto pela fonte do
lock:

| papel (sem) | macOS (accesskit_macos 0.26.1 `node.rs`) | windows (accesskit_windows 0.32.1 `node.rs`) | AT-SPI (accesskit_atspi_common 0.18.1 `node.rs`) |
|---|---|---|---|
| `Region` (a raiz da página) | `AXGroup` (148) com subrole `AXLandmarkRegion` (268) | `Group` (169) | `Landmark` (207) |
| `Button` | `AXButton` (65), press (668) | `Button` (89), `Invoke` (571-573) | `Button` (81-87), `click` |
| `Toggle` (switch, chip) | `AXCheckBox` subrole switch (82, 275) | `Button` (99) com `Toggle` (559-561) | `ToggleButton` (93), `Pressed` (344-345) |
| `Checkbox` | `AXCheckBox` (56) | `CheckBox` (74), `Toggle` | `CheckBox` (92), `Checked` (347) |
| `Slider` | `AXSlider` (158), increment e decrement | `Slider` (188), `RangeValue` (582-584), sem increment | `Slider` (225), valor numérico, sem increment |
| `TextInput` | `AXTextField` (58-64), `setAccessibilityValue` (555) | `Edit` (76-88), `Value` (575-580) | texto (239); sem `SetValue` de texto (fase 1) |
| `Select` e `Option` | fase 2a | fase 2a | fase 2a |
| `TabList`, `Tab` | `AXTabGroup` (166); a tab na fase 1 | `Tab` (196), `TabItem` (195), `SelectionItem` (631-649) | `PageTabList` (233), `PageTab` (231) |
| `Tree`, `TreeItem` | `AXOutline` (174); `AXRow` (50) com subrole `AXOutlineRow` (281), selecionável (1030-1039) | `Tree` (204), `TreeItem` (68), `SelectionItem` | `Tree` (258), `TreeItem` (259), `Selected` (320-326) |
| `Table`, `Row` | `AXTable` (78), `AXRow` (47) | `Table` (95), `DataItem` (65); a linha não é `SelectionItem` ("TODO: tables (#29)", 632-634) | `Table` (232), `TableRow` (209) |
| `List`, `ListItem` | `AXList` (77), `AXGroup` (48) | `List` (94), `ListItem` (66), `SelectionItem` | `List` (171), `ListItem` (176) |
| `Link` | `AXLink` (46) | `Hyperlink` (64) | `Link` (170) |
| `Group` (breadcrumb, split, grupos da Button) | `AXGroup` (123), subrole `AXApplicationGroup` (256) | `Group` (144) | `Panel` (156) |
| `Label` (chips estáticos, a linha da Input) | `AXStaticText` (44) | `Text` (62) | `Label` (228) |
| `Status` (o eco do crumb) | fase 1 | fase 1 | fase 1 |

o que isso diz às páginas novas:

- `expanded` não é dito por nenhum adapter do lock: o macOS 0.26.1 não tem
  disclosure (grep "xpand" em `src/`: só o papel `DisclosureTriangle`,
  113), o windows não tem `ExpandCollapse` (fase 1) e o AT-SPI não põe
  estado de expandido (grep: zero). um ramo da Tree é ouvido pelo botão
  apresentado "toggle expanded" (fase 2a) e pelos filhos que aparecem.
- a linha selecionada da tabela: a árvore só marca filhos pelo `selected`
  de um contêiner (`engine/src/accessibility/tree.rs:445-469`), e a `Table`
  do comps guarda `selected_row` (`comps/src/content/table.rs:135-147`).
  correção na árvore, para o papel `Table`: `selected_row` marca a linha
  como `selected` marca o filho. o windows ainda não a diz (a linha não é
  `SelectionItem`); o macOS e o AT-SPI dizem.
- o `SplitPane` é um `Group` com `set_ratio` (`comps/src/nav/split_pane.rs:96-120`):
  o leitor não tem como pedir (a árvore só tem `SetValue` para `set_value`
  e o `select` do select, `engine/src/accessibility/action.rs:136-148`), e
  o papel que caberia (`Splitter` no AccessKit) não existe no `sem`, cujos
  tipos não mudam nesta fase. fica na lista `UNREACHABLE` com o motivo; a
  página diz a proporção como descrição do grupo, e o agente a pede por
  `set_ratio`.
- `active` de um `NavLink`, `active` de um `IconButton` de barra e `busy`
  do `SubmitButton` não são ditos: a árvore só lê `toggled`, `expanded`,
  `selected` e valor (`engine/src/accessibility/tree.rs:281-360`). fica
  para a próxima fase.

## fase 2b: o roteiro do VoiceOver (para o brenner rodar)

o que cada passo espera vem dos testes (`crates/showcase/tests/access_tree.rs`,
`crates/showcase/tests/component_pages/`, `view/dispatch/component/tests.rs`)
e do mapeamento do `accesskit_macos` 0.26.1 (sonda da fase 2b, (e)); a
frase exata é do VoiceOver, e o que ele disser de fato é o registro. o
preparo é o da fase 1 (`cargo run -p showcase -- <página>`, VoiceOver com
Cmd-F5, VO é Control-Option). os passos das fases 1 e 2a seguem valendo;
estes são os das páginas de componente. nenhum passo pede o Claude.

| passo | o que fazer | o que ouvir |
|---|---|---|
| botões 1 | `-- Toast`, VO-Right a partir da janela | "Toast", região (landmark); "Neutral, button", "Success", "Error", "Info" |
| botões 2 | VO-Space em "Error" | "Failed to push: remote rejected." anunciado com prioridade alta; o cursor fica no botão |
| botões 3 | `-- Dialog`, VO-Space em "Delete repository…" | o diálogo "Delete repository?" abre com o foco em "Cancel" (fase 2a); a página some da navegação (inerte) |
| botões 4 | `-- Button`, andar pelos grupos | "Button, medium", grupo; dentro, "Button, button" com a descrição "Default, Hover, Press, Focus", e o do cartão "Disabled" dimmed; os de ícone pelo ícone ("x", "plus", "send"), o de envio "submit"; VO-Space num ligado não muda nada (é o que o clique faz) |
| botões 5 | com o foco do teclado seguindo o cursor, parar num botão | o anel de foco aparece no botão; Esc tira; um clique em outro lugar tira |
| botões 6 | `-- Tooltip`, parar em "Hover me" | "Hover me, button"; a descrição (VO-Shift-H, ou a espera) é o texto do tooltip, sem passar o mouse |
| botões 7 | `-- Spinner`, VO-Space em "Start spinners" | o botão passa a dizer "Stop spinners" e os arcos giram (eles mesmos não têm nó: grupo 2) |
| valores 1 | `-- Checkbox` | "Autosave on focus loss, checkbox, checked"; VO-Space: "unchecked"; "Managed by your organization" dimmed |
| valores 2 | `-- Switch` | "Focus mode", switch, ligado; VO-Space desliga e o knob anda |
| valores 3 | `-- Chip` | as etiquetas lidas como texto ("exact", "ann", ...); "click me", switch, desligado; VO-Space liga |
| valores 4 | `-- Slider`, "stepped" | "stepped, slider, 4"; VO-Shift-Up (ou as setas de interação): 5 |
| valores 5 | o slider "continuous" | "continuous, slider, 65"; ele não tem passo, a árvore só oferece `SetValue`: registrar se o VoiceOver consegue movê-lo (gap provável: o VoiceOver move slider por incremento) |
| valores 6 | `-- Input`, "Full name", digitar "Ada" | campo de texto; o valor "Ada"; mais abaixo, "live value: Ada" como texto |
| valores 7 | `-- Select` | "appearance, pop-up button, System default"; VO-Space abre; "Always light"; VO-Space escolhe e fecha |
| escolhas 1 | `-- Tabs` | "sections", grupo de abas; "Account, tab, selecionada, 1 de 3"; VO-Space em "About": selecionada |
| escolhas 2 | `-- Tree` | "files", outline; as linhas "src", "ui", "widgets.rs"... pelo nome; VO-Space em "main.rs": selecionada |
| escolhas 3 | na Tree, entrar em "assets" (fechado) | "toggle expanded, button" dentro da linha; VO-Space: os filhos aparecem. "expandido" não é dito (nenhum adapter do lock diz): registrar o que o VoiceOver fala |
| escolhas 4 | `-- Data Table` | "files", tabela; cada linha pelas células ("crates/comps/src/file_0.rs, modified, 3 KB, 0000000"); VO-Space numa linha: selecionada (novo na árvore) |
| escolhas 5 | `-- Scroll Area` | "items", lista; "Item 0", "Item 1"...; VO-Space: selecionada; rolar com o trackpad troca as linhas que a árvore tem |
| escolhas 6 | `-- Breadcrumb` | "path", grupo; "crates, link", ..., "table.rs" texto; VO-Space em "comps": "clicked crumb 1" anunciado, polido |
| escolhas 7 | `-- Navigation Menu` | "Workspace, link" com a descrição "1"; "History" com "4"; VO-Space em "Branches": a marca de ativo anda na tela (o "atual" não é dito: gap) |
| escolhas 8 | `-- Resizable` | "panes", grupo, "the first pane takes 35%"; nenhuma ação para o leitor (gap listado); arrastar com o mouse e voltar: a descrição muda |
| barra | na Button, abrir a barra de menu | só "plev" e "view": a página de componente não põe menu |
| sem projeção | `-- Card` e `-- Separator` | só a janela (grupos 2 e 3) |

no windows e no AT-SPI, quando o alvo rodar: a linha selecionada da
tabela não é dita no windows (a linha não é `SelectionItem`); o switch e o
chip são botão com `Toggle` no windows e `ToggleButton` no AT-SPI; o
slider se ajusta só por valor.

## handoff da fase 2b (2026-09-25)

branch `task/75c-a11y-component-pages`, sobre `main` em `3048b320`. sem
push.

### commits

1. `7fa4f84c` a sonda (seção "fase 2b: a sonda" acima).
2. `e5f738e5` o mecanismo (`Component` e a tabela), com as páginas Dialog
   e Toast.
3. `22819cc9` as páginas de botão: Button, Icon Button, Tooltip, Spinner.
4. `a64f9e93` as de alternância e valor: Checkbox, Switch, Chip, Slider,
   Input, Select.
5. `8f8cf475` as de coleção e navegação: Tabs, Tree, Data Table, Scroll
   Area, Breadcrumb, Navigation Menu, Resizable; a árvore diz a linha
   selecionada da tabela.
6. `a66e6e1d` a barra de menu deixa as páginas de componente de fora.
7. `5e0d2c2f` as promessas de privacidade, guardadas.
8. `3badee06` as 60 baselines.
9. este commit: roteiro, ADR e handoff.

### o mecanismo

`crates/showcase/src/view/dispatch/component.rs`: o trait `Component`
(raiz, `project`, `run`, `focus`, `blur`), a tabela (uma macro, uma linha
por pane, 19 linhas) e o `Operable` que empresta o dispatcher do todo. a
view entra por quatro braços: `page_semantics`, `act`, `assist_focus` e o
Escape. a página se projeta sob uma região com o título da página, inerte
com um overlay por cima. o clique da pessoa fica no widget (a projeção
segue o estado); o que a página faz por cima do clique (modal, toast) é um
método só, `after_component`, chamado pelo clique e pelo pedido.

### as páginas, antes e depois

| grupo | antes | depois |
|---|---|---|
| projetam | 6 | 36: as 6 e 30 páginas de componente (19 panes) |
| grupo 1 (widget com projeção no comps) | 30 sem projeção | 0: todas projetam |
| grupo 2 (widget sem projeção) | 24 | 24, na lista `SILENT` com o que falta |
| grupo 3 (nada a operar) | 22 | 22, na lista `SILENT` com o motivo |

as listas de `crates/showcase/tests/access_tree.rs`: `SILENT` 46 (era 76
sem motivo); `UNREACHABLE` 2 (a fase 2a deixou 1; entra `Resizable:
resizable_page.split set_ratio`); `FOCUS_NOT_PROJECTED` e `OFFERED_DISABLED`
seguem vazias com as 36 páginas.

três correções na página (nada no comps): o chip "click me" virou chip de
filtro (a projeção dizia botão); o rótulo do Spinner muda com o giro, não
no quadro; o trilho da Navigation Menu é um `NavLink` próprio sem id. uma
na árvore (`crates/engine/src/accessibility/tree.rs`): a tabela diz a
linha selecionada por `selected_row`.

### os testes

- por tipo de widget, a pessoa, o leitor e um agente pelo port de verdade
  no mesmo estado (`crates/showcase/tests/component_pages/`, 10 testes):
  botão (Toast, Dialog, Alert Dialog, Spinner), botão que não faz nada
  (Button, Icon Button, Tooltip), alternância (Checkbox, Switch, Chip,
  Toggle), slider (valor e passo), campo de texto, select, escolha numa
  coleção (Tabs, as duas tabelas, as duas listas, os dois menus de
  navegação, Breadcrumb), árvore (folha e ramo), split (pessoa e agente; o
  leitor não pede), e o foco que o leitor deixa (Escape e clique).
- o mecanismo (`view/dispatch/component/tests.rs`, 4): a região com o
  título, o dispatcher da galeria ouvindo leitor e agente, a página inerte
  sob o modal.
- a árvore (`accessibility/tree/tests.rs`): a linha selecionada da tabela.
- a barra de menu (`app/menu.rs`): `the_bar_leaves_the_component_pages_out`.
- as promessas (`component_pages/promises.rs`).
- consequência: `app/access.rs`, `app/headless.rs` e
  `apps/app/semantics/assistive.rs` usavam a Button como página sem
  projeção e passam a usar a Card; `tests/view_source.rs` troca o texto
  formatado da Data Table (agora dentro de uma linha ligada, que leva à
  projeção dela) pelo "step 1: 4" da Slider.

### a barra de menu

deixa as páginas de componente de fora (`view.is_component`), como já
deixa o shell e o overlay: numa página de componente ela mostraria
amostras ("Button" 30 vezes). a Toggle Group segue com a barra da todo.

### as promessas

nenhum caso no escopo: nenhum segredo, nenhuma câmera, nenhuma ação só da
pessoa, nenhum estado só do leitor. o teste guarda a ausência e confere que
o que sai pelo port é o que a página projeta.

### as baselines

60, as 30 páginas em core e hoff: alert-dialog, breadcrumb, button,
button-group, checkbox, chip, combobox, data-table, dialog, field,
hover-card, icon-button, input, item, kbd, label, navigation-menu,
popover, resizable, scroll-area, select, slider, spinner, switch, table,
tabs, toast, toggle, tooltip, tree (`@core@1280x800` e `@hoff@1280x800`).
em cada uma: um `semantics.json` novo; no `scene.json`, o bloco
`semantics` (de `null` para revisão e contagem) e o bloco `bindings`; e o
nome da camada da página, que o regress tira das raízes ligadas nela
(o espelho das ligações, ver a sonda (c)). nenhum `frame.png` existe
nelas. as outras 116 baselines, as seis páginas que já projetavam e os
goldens do `uir` não mudaram.

a confirmação por script (compara o `scene.json` de antes e de depois,
camada a camada na ordem de composição: z, hash, contagem de nós e limites
iguais; o nome só muda de `z<z>` para a junção das raízes ligadas na
camada; as ligações e a projeção são novas; o `semantics.json` tem a
contagem do `scene.json`; nenhum `frame.png`):

```text
60 baselines checked
  layer z0 -> breadcrumb_page: 2
  layer z0 -> button_page: 4
  layer z0 -> checkbox_page: 2
  layer z0 -> chip_page: 4
  layer z0 -> dialog_page: 4
  layer z0 -> icon_button_page: 2
  layer z0 -> input_page: 6
  layer z0 -> navigation_menu_page: 4
  layer z0 -> resizable_page: 2
  layer z0 -> select_page: 4
  layer z0 -> slider_page: 2
  layer z0 -> spinner_page: 2
  layer z0 -> switch_page: 2
  layer z0 -> table_page: 4
  layer z0 -> tabs_page: 2
  layer z0 -> toast_page: 2
  layer z0 -> tooltip_page: 6
  layer z0 -> tree_page: 2
  layer z10 -> scroll_area_page: 4
frame.png touched: False
problems: 0
```

os diffs inteiros, um por tipo de widget, estão no anexo no fim deste
arquivo.

### gate e regress

- antes (o briefing): 3086 passam, 27 ignorados, 176 baselines.
- depois, em `3badee06`: `./script/gate` verde, pernas urnaui e wasm
  rodaram; 3103 passam (+17: 3018 no workspace, 85 no urnaui), 27
  ignorados, 0 falhas. `regress check`: 176 baselines, 176 iguais (antes
  do bless: 60 mudadas, as desta fase).
- `Cargo.toml` e `Cargo.lock` intocados (md5 do lock igual), nenhuma
  dependência nova, AccessKit na mesma versão; `crates/comps`,
  `crates/uir`, `crates/surface`, `crates/sem`, `crates/wire/src`,
  `crates/propose` e `crates/ide/src` intocados. sempre com `PLEV_CLAUDE`
  no stub e `--offline`; nenhuma janela, nenhum leitor ligado, nenhum
  clique sintético por coordenada de tela, nenhum diálogo nativo.

### linha para o índice de ADRs

| 2026-09-25 | [accessibility-is-production-ready-per-platform](accessibility-is-production-ready-per-platform.md) | proposed | fase 2b: as páginas de componente cujos widgets se projetam no comps projetam o que mostram (30 páginas, 19 panes), por um trait e uma tabela, sob uma região com o título da página, cada pedido pelo dispatcher da galeria; o clique da pessoa fica no widget e a projeção segue o estado; a pessoa, o leitor e um agente pelo port chegam ao mesmo estado; a barra de menu as deixa de fora; a árvore diz a linha selecionada da tabela; as 46 páginas que ficam, cada uma com o motivo (task-75 fase 2b) |

### o que fica para a próxima fase

- o grupo 2 (24 páginas): a projeção no comps de `Avatar`, `Badge`,
  `Stat`, `ProgressBar`, `Spinner`, `Skeleton`, `CodeBlock`, `Card`, os
  gráficos, `GraphView`, `EmptyState`, `Sidebar`, `PanelHeader`,
  `EditorView`, e o que é desenhado à mão (os cartões de tema, a área do
  menu de contexto, o dock, o builder); a Git espera o ide. o que cada um
  precisa está na sonda (a).
- o teclado completo nos widgets: Tab na ordem da projeção (hoje só as
  páginas com foco próprio andam com Tab), setas em listas, abas, árvore e
  menus, o trap de foco no modal, o anel nas pílulas.
- o que a árvore ainda não diz: `active` de um link de navegação e de um
  botão de barra, `busy` do botão de envio; `expanded`, que nenhum adapter
  do lock diz; a proporção do split como valor que o leitor ajusta (pede
  um papel de splitter no contrato).
- o slider contínuo no VoiceOver (sem passo, só `SetValue`): registrar no
  roteiro.
- bounds para os controles apresentados (fase 2a).
- windows, AT-SPI, iOS e android seguem esperando o alvo ou o adapter.

## anexo da fase 2b: os diffs das baselines, por tipo de widget

o agente colou aqui um diff inteiro por tipo de widget (cerca de 3100
linhas de JSON); o orquestrador tirou na integração, porque o diff mora no
git: o `semantics.json` de cada página é arquivo novo e o `scene.json` muda
nos blocos `semantics`, `bindings` e no nome das camadas. para ler um tipo:
`git show 3badee06 -- crates/showcase/fixtures/regress/<página>@core@1280x800/`.
as páginas escolhidas, uma por tipo:

- botão (Button, IconButton, SubmitButton): `spinner@core@1280x800`
- checkbox: `checkbox@core@1280x800`
- switch: `switch@core@1280x800`
- chip: `chip@core@1280x800`
- slider: `slider@core@1280x800`
- campo de texto: `input@core@1280x800`
- select: `select@core@1280x800`
- tabs: `tabs@core@1280x800`
- árvore: `tree@core@1280x800`
- tabela: `data-table@core@1280x800`
- lista virtual: `scroll-area@core@1280x800`
- breadcrumb: `breadcrumb@core@1280x800`
- link de navegação: `navigation-menu@core@1280x800`
- split: `resizable@core@1280x800`

## fase 2c: a sonda do teclado (2026-09-25)

branch `task/75d-keyboard`, sobre `main` em `73f99094`. o escopo é o
item 3 do escopo acima nas 30 páginas de componente (os 19 panes da
tabela em `view/dispatch/component.rs:108-128`). os caminhos são relativos
a `crates/showcase/src/view` quando não dizem o crate. a leitura de cada
página foi feita com um teste sem janela que projeta a página, lista
`focus::order` e aperta Tab pela view (`handle_edit_key(EditKey::Tab)`).

### (a) o que Tab faz hoje

a ponte manda Tab como `EditKey::Tab`, sem olhar o Shift
(`src/keys.rs:18`): Shift-Tab é Tab. a view só passa Tab a seis panes
(`input.rs:145-167`), e cada um tem a sua própria ordem:

| página (pane) | focáveis na projeção | Tab hoje | por quê |
|---|---|---|---|
| Input, Field, Label (Input) | 3 campos | anda e dá a volta | `components/input.rs:127-133`, índice próprio |
| Checkbox | 2 caixas | anda | `components/checkbox.rs:71-77`, o `Focus` da página |
| Switch | 2 switches | anda | `components/switch.rs:70-76`, idem |
| Slider | 2 sliders | anda | `components/slider.rs:78-84`, idem |
| Select, Combobox (Select) | a pílula | fica na pílula | `components/select.rs:68-74` |
| Tabs | 3 abas | fica na aba ativa | `components/tabs.rs:76-82`: liga o anel da faixa inteira, e a projeção diz a ativa |
| Button, Button Group, Icon Button, Tooltip, Hover Card, Popover, Spinner, Toast, Dialog, Alert Dialog, Chip, Toggle, Breadcrumb, Navigation Menu, Kbd, Tree | de 1 a 16 | nada | `input.rs:165`: o braço `_ => false` |
| Table, Data Table, Scroll Area, Item, Resizable | 0 | nada | nenhum nó focável (ver (c)) |

o `Focus` das páginas (`components/mod.rs:73-92`) é uma segunda fonte de
ordem: um índice que a página avança, que não lê a projeção. Enter,
Espaço e as setas também não chegam às páginas de componente:
`handle_enter` só conhece os panes de app (`input.rs:172-200`), Espaço vira
`handle_key(" ")` e cai no `_ => false` (`input.rs:96`), cima e baixo só
servem à Git (`mod.rs:181-183`), e esquerda, direita, Home e End só editam
o campo focado.

### (b) onde a ordem da projeção dirige o Tab

o engine já tem a ordem: `accessibility::focus::order` e `focus::next`
(`crates/engine/src/accessibility/focus.rs:33-62`) andam o snapshot na
ordem da árvore que o leitor recebe, pulando o que é inerte, desabilitado
ou não focável, e dão a volta. o teste
`tab_and_the_reader_walk_the_same_order_and_the_page_takes_the_focus`
(`crates/showcase/tests/access_tree.rs:397-423`) já prova que essa ordem é
a do leitor em toda página que projeta, e que toda página toma o foco que
o leitor pede (`assist_focus`, `dispatch.rs:464-495`, que numa página de
componente chega em `Component::focus`, `dispatch/component.rs:230-241`).

então Tab é `focus::next(&self.semantics(), direção)` seguido de
`assist_focus` no id que sai: a mesma projeção que o leitor anda, o mesmo
método que o leitor usa para mover o foco, e nenhuma ordem própria. o
Shift vem das teclas modificadoras que a view já recebe
(`mod.rs:169-171`), sem mexer na ponte. com isso o `Focus::next` e os seis
braços de Tab somem.

o que o contrato do engine diz e o padrão WAI-ARIA recomenda diferente: o
engine põe cada nó focável na sequência de Tab (`focus.rs:1-13`), então as
três abas e as nove linhas da árvore são nove paradas de Tab, e o padrão
recomenda uma parada por widget composto (o tabindex que roda). fazer isso
é dizer, na projeção, qual item do composto é focável, o que muda o
`semantics.json` das baselines. fica como pendência; as setas desta fase
funcionam por cima da sequência de hoje.

### (c) os widgets que pedem setas

| widget (papel) | páginas | padrão | o que já tem | o que falta |
|---|---|---|---|---|
| abas (`TabList`, `Tab`) | Tabs | esquerda e direita, Home e End; Enter e Espaço selecionam | `select` na projeção; anel só na faixa inteira (`crates/comps/src/nav/tabs.rs:93-99`, `176-178`) | anel na aba focada |
| árvore (`Tree`, `TreeItem`) | Tree | cima e baixo nas linhas visíveis, Home e End; direita abre o ramo fechado ou desce ao primeiro filho; esquerda fecha o ramo aberto ou sobe ao pai; Enter e Espaço selecionam | `select`, `toggle_expanded`, `expanded` e o pai de cada item na projeção | anel na linha (`components/tree.rs:19`: "a row draws no ring") |
| menu (`Menu`, `MenuItem`) | o menu de contexto, overlay | cima e baixo dão a volta, Home e End; Enter e Espaço acionam | `activate` por item; o destaque do hover (`crates/comps/src/feedback/context_menu.rs:102`) | o destaque seguir o foco |
| select (`Select`, `Option`) | Select, Combobox | Enter, Espaço, cima ou baixo abrem na opção escolhida; cima e baixo andam as opções, Home e End; Enter e Espaço escolhem; Escape fecha | `open`, `close`, `select`; o destaque por tecla no comps (`crates/comps/src/form/select/panel.rs:163-196`), sem uso | o destaque seguir a opção focada |
| slider (`Slider`) | Slider | direita e cima sobem, esquerda e baixo descem, Home e End vão aos extremos | `increment` e `decrement` no que tem passo, `set_value` com a faixa (`crates/comps/src/form/slider.rs:74-104`) | o contínuo não tem passo: a tecla pede `set_value` |
| lista e tabela (`List`, `Table`) | Scroll Area, Item, Table, Data Table | cima e baixo nas linhas | `select` por linha | nenhuma linha é focável na projeção; ver abaixo |
| radio | nenhuma página de componente | | | |

os outros controles não pedem seta, só anel e acionar: botões (Enter e
Espaço), caixas e switches (Espaço), links de navegação e crumbs (Enter).
três desenham o foco hoje sem anel: o chip de filtro
(`crates/comps/src/action/chip.rs`), o crumb (`crates/comps/src/nav/breadcrumb.rs`)
e o link de navegação (`crates/comps/src/nav/nav_link.rs`). Enter e Espaço
pedem a ação primária do nó focado, a mesma que o clique do leitor pede
(`crates/engine/src/accessibility/action.rs:102`), pelo dispatcher da
galeria, como pessoa.

as listas e tabelas: as linhas projetam `select` mas não são focáveis, e a
lista e a tabela também não. nenhum Tab chega nelas sem a projeção dizer
um nó focável, o que muda as oito baselines dessas páginas (core e hoff).
é a condição de parar do briefing; não mexi, está no handoff como decisão.
o split da Resizable pede um papel de splitter no contrato (já na lista da
fase 2b).

### (d) o modal

- o trap já sai da projeção: com o modal aberto a região da página é
  inerte (`dispatch/component.rs:132-135`), e `focus::next` pula o que é
  inerte, então Tab sobre `semantics()` só anda entre os dois botões do
  modal. os toasts são live regions, não focáveis.
- o foco do modal é o cancel, ou o botão que o leitor escolheu
  (`dispatch/overlay.rs:108-115`); o comps `Modal` tem dois `Button` com
  `set_focused` (`crates/comps/src/feedback/modal.rs:47-48`), mas nenhum
  jeito de a view ligar o anel neles.
- o retorno já sai da projeção: a página guarda o foco dela enquanto o
  modal está aberto, e ele volta a ser o foco da projeção quando o modal
  fecha (`dispatch/overlay.rs:12-16`, `dispatch/detached.rs:226`). falta a
  página ter o foco no botão que abriu, o que Tab e Enter dão.
- o Escape tira o foco da página antes de fechar o overlay
  (`input.rs:113-133`): com o modal aberto por Enter, o primeiro Escape
  apagaria o anel do botão que abriu e deixaria o modal aberto. o overlay
  que segura o ponteiro fecha primeiro.

### (e) fora desta fase

os atalhos anunciados: o `sem::Node` não tem campo de atalho
(`crates/sem/src/node.rs:35-73`), e dizer um atalho ao leitor pede o
contrato. vai para as pendências.

## handoff da fase 2c (2026-09-25)

branch `task/75d-keyboard`, sobre `main` em `73f99094`. sem push.

### commits

1. `8ba4ef6d` a sonda (seção "fase 2c: a sonda do teclado" acima).
2. `92635707` Tab e Shift-Tab na ordem da projeção, com o anel em todo
   controle.
3. `f1e04c32` as setas por widget; Enter e Espaço acionam.
4. `5dddcf6a` o modal: o trap, o anel no botão e o foco de volta.
5. `917cdd2a` os testes, sem janela.
6. este commit: o handoff.

### a sonda, curta

- (a) antes, Tab andava em 6 dos 19 panes, cada um com um índice próprio;
  Shift-Tab era Tab; Enter, Espaço e as setas não chegavam às páginas de
  componente.
- (b) a ordem já existia no engine (`focus::next`), e é a que o leitor
  anda. Tab virou `focus::next` sobre `semantics()` seguido de
  `assist_focus`; o índice das páginas saiu.
- (c) pedem setas: abas, árvore, menu de contexto, select e slider.
  listas e tabelas não têm nó focável (ver pendências); nenhuma página de
  componente tem radio.
- (d) o trap e o retorno já saíam da projeção. faltavam o anel no botão do
  modal e o Escape fechar o modal antes de tirar o foco da página.

### o mecanismo

`crates/showcase/src/view/dispatch/keyboard.rs`. a view manda para ele o
Tab, as setas, Home, End, Enter e Espaço quando a página é de componente
ou quando um overlay segura o ponteiro (`input.rs`, `mod.rs`). o Shift
vem das teclas modificadoras que a view já recebia; a ponte
(`src/keys.rs`) não mudou.

- Tab: `focus::next` na projeção inteira e `assist_focus` no id que sai.
- setas: escolhem, na mesma ordem, os nós do widget em que o foco está.
- Enter e Espaço: a ação primária do nó focado
  (`accessibility::primary`), a mesma do clique do leitor.
- tudo que muda estado é pedido da pessoa (`Actor::Human`) pelo
  dispatcher da galeria (`act`). nenhum caminho novo para agente.

### o que mudou por widget

| widget | teclas | comps | página |
|---|---|---|---|
| botões (Button, Icon Button, Tooltip, Spinner, Toast, Dialog) | Tab, Enter, Espaço | nada (o anel já existia) | nada |
| caixa, switch, chip de filtro | Tab, Espaço, Enter | `Chip::set_focused` e o anel | o índice de Tab saiu; o chip liga o anel |
| campo de texto | Tab; as setas editam | nada | o índice de Tab saiu |
| link de navegação, crumb | Tab, Enter | `NavLink::set_focused`, `Breadcrumb::set_focused` e o anel | ligam o anel |
| abas | Tab; esquerda e direita dão a volta; Home, End; Enter e Espaço selecionam | `Tabs::set_focused_tab`: o anel na aba, por dentro | o foco é a aba focada |
| árvore | Tab; cima e baixo; direita abre ou entra; esquerda fecha ou sobe; Home, End; Enter seleciona | `Tree::set_focused` e o anel na linha, por dentro | liga o anel |
| select | Tab; cima, baixo, Enter ou Espaço abrem na opção escolhida; cima e baixo nas opções; Home, End; Enter escolhe; Escape fecha | `Select::set_highlighted`, que rola até a opção | o destaque segue a opção focada |
| slider | Tab; direita e cima sobem, esquerda e baixo descem; Home, End | nada | o índice de Tab saiu |
| menu de contexto | cima e baixo dão a volta e pulam o separador e o item desligado; Home, End; Enter escolhe | `ContextMenu::set_highlighted` | o destaque segue o foco |
| modal | Tab e Shift-Tab ficam nos dois botões; Enter e Espaço respondem; Escape fecha | `Modal::set_focused` e `focused` | o modal aberto pelo teclado mostra o anel no cancel; o foco volta ao botão que abriu |

o slider contínuo (sem passo) anda um centésimo da faixa por tecla, com
`set_value`; o com passo usa `increment` e `decrement`. o anel dentro da
linha e da aba é `recipe::focus_ring_inside`, o mesmo traço do
`focus_ring`, para não invadir a vizinha. o Escape agora fecha primeiro o
overlay que segura o ponteiro, e nada debaixo dele recebe letra ou tecla
de edição.

### os testes

- `crates/showcase/tests/component_pages/keyboard.rs` (6):
  - Tab em todas as 30 páginas: cada controle focável é alcançado na
    ordem da projeção, dá a volta, Shift-Tab volta, o foco nunca some, o
    anel aparece no controle focado (um anel a mais no quadro), e o
    Escape tira. as cinco páginas sem nó focável são a lista conhecida.
  - as setas nas abas, na árvore, no select e nos sliders.
  - Enter e Espaço: caixa, switch e chip terminam no mesmo estado do
    clique; link, crumb e botão do Toast.
  - o modal: o foco entra no cancel com anel, Tab e Shift-Tab ficam
    dentro, Espaço responde, o foco volta ao botão que abriu; o mesmo
    com Escape.
- `crates/showcase/src/view/dispatch/keyboard/tests.rs` (1): o menu de
  contexto com as setas, o destaque, o Tab preso e o Enter.
- `crates/comps/src/tests/focus.rs` (+5): os anéis e destaques novos.
- mudaram dois testes de teclado, nenhum de clique: o de Tab da página
  Input virou "o campo toma o foco que o Tab pede" (o Tab é da view); e
  em `apps/app/tests/semantics.rs` o Enter com o modal aberto agora
  responde o modal (antes caía na página inerte).

### os PNGs (sem janela, tema core, 1280x800)

- `target/task-75d/tree-after-3-tabs.png`: a árvore depois de três Tabs,
  o anel em widgets.rs.
- `target/task-75d/tabs-tab-then-right.png`: Tab e seta para a direita, o
  anel em Appearance, Account segue ativa.
- `target/task-75d/dialog-modal-open-focus-on-cancel.png`: dois Tabs e
  Enter na página Dialog; o modal aberto com o anel no Cancel.

o caminho é relativo ao worktree
(`/Users/nn/Dev/hoff/plevdev/.claude/worktrees/agent`).

### gate e regress

- `./script/gate` verde, pernas urnaui e wasm rodaram: 3213 passam (3128
  no workspace, 85 no urnaui), 28 ignorados, 0 falhas. a branch soma 13
  testes e não tira nenhum.
- `regress check`: 176 baselines, 176 iguais. nenhuma baseline mudou.
- `Cargo.toml` e `Cargo.lock` intocados; engine, tema, sem e wire
  intocados. sempre `--offline`, com `PLEV_CLAUDE` no stub, sem janela e
  sem tecla do sistema.

### pendências

- listas e tabelas (Table, Data Table, Scroll Area, Item): nenhum nó
  focável, então o Tab não chega. é decisão, porque muda baseline.
  opções: (1) toda linha visível focável: 11 a 20 paradas de Tab por
  página, ruim; (2) uma linha focável por vez (a selecionada, ou a
  primeira visível) e as setas andam nas linhas; (3) a lista focável com
  um item ativo, o que pede um conceito novo no `sem`. recomendo (2),
  junto com o item abaixo, num bless só: 8 baselines das listas e
  tabelas, mais 4 de Tabs e Tree.
- uma parada de Tab por widget composto (o tabindex que roda, como o
  WAI-ARIA recomenda): hoje cada aba e cada linha da árvore é uma parada,
  porque o engine põe todo nó focável na sequência. pede a projeção dizer
  qual item é focável; muda o `semantics.json` de Tabs e Tree.
- o split da Resizable: pede um papel de splitter no contrato.
- os atalhos anunciados: o `sem::Node` não tem campo de atalho.
- o menu de contexto não abre pelo teclado (Shift-F10 ou a tecla de
  menu), e PageUp e PageDown não chegam: a ponte (`src/keys.rs`) não
  manda essas teclas.
- o modal aberto por clique não mostra o anel no cancel até o primeiro
  Tab; um clique não dá foco a controle, como antes.
- o roteiro do VoiceOver da fase 2b pode ganhar um passo de teclado
  (Tab, setas e o modal), para o brenner rodar.

## fase 2c integrada (orquestrador, 2026-09-25)

o item "teclado completo nas páginas de componente, com teste" avançou,
sem fechar: o Tab anda na ordem da projeção (a do leitor de tela) nas 25
páginas de componente que têm algo focável, com o anel de foco; setas
pelo padrão WAI-ARIA em abas, árvore, select, slider e menu de contexto;
Enter e Espaço acionam pelo dispatcher como pedido da pessoa; o modal
prende o foco e o devolve ao fechar. 13 testes novos sem janela.

validação do orquestrador: diff lido (`view/dispatch/keyboard.rs` usa
`focus::next` e `assist_focus`, e todo pedido sai como `Actor::Human`
pelo dispatcher da galeria); os PNGs abertos: o anel numa linha da
árvore depois de três Tabs e no Cancelar do modal. o botão que abriu o
modal continua com o anel por baixo do véu (pendência pequena).

o que falta para fechar o item: Table, Data Table, Scroll Area e Item
não têm nó focável, e abas e árvore são uma parada de Tab por item. a
recomendação do agente (uma linha focável por vez, setas andando nas
linhas, cada widget composto uma parada só) muda 12 baselines, que o
orquestrador revisa antes de aceitar. o split da Resizable pede um papel
de splitter no contrato do `sem` e fica como pendência.

o agente gastou uns 350 mil tokens.

## handoff da fase 2d (2026-09-25)

branch `task/75e-roving`, sobre `main` em `12e18669`. sem push. fecha o
item "teclado completo nas páginas de componente, com teste" nas 29
páginas; a Resizable fica de fora (pede o papel de splitter).

### commits

1. `e61cfa14` listas e tabelas focáveis, uma linha por vez.
2. `63e36503` abas e árvore como uma parada só de Tab.
3. `503b9bef` os testes, sem janela.
4. `cc6fafc9` as 12 baselines, só significado.
5. este commit: o handoff.

### o mecanismo

a projeção diz qual item do widget é focável (o tabindex que roda do
WAI-ARIA): só esse entra na sequência de Tab, então `focus::next` para
uma vez em cada widget. quem decide é o comps, no `project` de cada um:
o item focado, senão o escolhido, senão o primeiro. as setas
(`view/dispatch/keyboard.rs`) andam entre todos os itens do widget,
focáveis ou não, e o item que recebe o foco vira a parada na projeção
seguinte. nenhuma ordem própria na view, nada novo no `sem` nem no
engine.

### o que mudou por widget

| widget | parada de Tab | teclas | comps | página |
|---|---|---|---|---|
| lista (Scroll Area, Item) | a linha focada, senão a selecionada em vista, senão a primeira em vista | cima e baixo; Home e End às pontas da lista inteira (10 mil linhas), guardada ou não; Enter e Espaço selecionam | `VirtualList::set_focused` (rola até a linha), `focused` (segue a rolagem), `tab_stop`, `shown`; o anel dentro da linha | toma e larga o foco; o clique larga, como na árvore |
| tabela (Table, Data Table) | a mesma regra, nas linhas | as mesmas | `Table::project` usa a parada da lista | a mesma |
| abas | a aba focada, senão a ativa | as de antes | `Tabs::project` | nada |
| árvore | a linha focada, senão a selecionada, senão a primeira, entre as que aparecem | as de antes | `Tree::project` | nada |

o foco segue a rolagem nos dois sentidos: a seta rola o mínimo para a
linha focada caber inteira, e a roda, quando tira a linha focada de
vista, leva o foco para a linha mais perto que está em vista. o foco
nunca some. a seta que rola mostra a barra de rolagem, como a roda.

### os testes

- `crates/showcase/tests/component_pages/keyboard.rs`: o Tab em todas
  as 30 páginas agora chega em 29 (só a Resizable fica sem nó focável),
  e em cada tab list, árvore, lista e tabela para uma vez só (6 páginas
  com widget assim, lista conferida). novo
  `the_arrows_walk_the_rows_of_a_list_and_a_table`, nas 4 páginas: Tab
  cai na primeira linha, setas, Home e End às pontas com rolagem, o anel
  na linha, a linha focada é a única parada, Enter e Espaço selecionam
  como o clique (o estado inteiro igual ao do clique), Escape e Tab voltam
  na selecionada, a roda leva o foco junto e ele nunca some. os testes de
  abas e árvore conferem a parada única e o Tab voltando na aba ativa e na
  linha selecionada.
- `crates/comps/src/tests/widgets/semantics/roving.rs` (5): a parada de
  cada widget; a linha que rola para a vista; o foco seguindo a roda.
- `crates/comps/src/tests/focus.rs` (+1): o anel dentro da linha da lista
  e da tabela.
- mudou um teste: em `crates/showcase/tests/access_tree.rs`,
  `every_control_has_a_name_and_takes_focus_and_no_group_is_empty` aceita
  uma aba ou linha da árvore sem o `Focus` do leitor quando outro item do
  mesmo widget é a parada (ver pendências). digitar num campo, o clique e
  o mouse: os testes de 2b e 2c seguem verdes sem mudança.

### as baselines regravadas

o antes e o depois de cada uma estão em `target/task-75e/baselines/<página>@<tema>/`
(`before.png`, `after.png`, `before-after.png` lado a lado,
`before.semantics.json`, `after.semantics.json`, `semantics.diff`). os
PNGs antes e depois são iguais byte a byte; o regress disse "visual
unchanged" nas 12, sem desenhar quadro. o `scene.json` de cada uma muda
só a revisão da projeção.

- data-table@core: a linha 0 passa a ser focável.
- data-table@hoff: a linha 0 passa a ser focável.
- table@core: a linha 0 passa a ser focável.
- table@hoff: a linha 0 passa a ser focável.
- scroll-area@core: o item 0 passa a ser focável.
- scroll-area@hoff: o item 0 passa a ser focável.
- item@core: o item 0 passa a ser focável.
- item@hoff: o item 0 passa a ser focável.
- tabs@core: Appearance e About saem da sequência de Tab; Account fica.
- tabs@hoff: Appearance e About saem da sequência de Tab; Account fica.
- tree@core: as 8 linhas depois de src saem da sequência de Tab; src fica.
- tree@hoff: as 8 linhas depois de src saem da sequência de Tab; src fica.

nenhuma outra baseline mudou.

### o PNG

`target/task-75e/data-table-tab-then-two-downs.png` (sem janela, tema
core, 1280x800): Tab e duas setas para baixo na Data Table, o anel na
linha file_2.rs.

### gate e regress

- `./script/gate` verde, pernas urnaui e wasm rodaram: 3220 passam (3135
  no workspace, 85 no urnaui), 28 ignorados, 0 falhas. a branch soma 7
  testes e não tira nenhum.
- `regress check` antes do bless: 176 baselines, 12 mudaram (as da
  lista, só significado). depois do bless: 176 iguais.
- `Cargo.toml`, `Cargo.lock`, engine, tema, `sem` e wire intocados.
  sempre `--offline`, com `PLEV_CLAUDE` no stub, sem janela e sem tecla do
  sistema.

### pendências

- o leitor de tela e o foco: o contrato liga `focusable` à sequência de
  Tab, e o engine só oferece o `Focus` do leitor ao nó focável. então uma
  aba ou linha da árvore que não é a parada não recebe mais o `Focus` do
  leitor; ele ainda lê, seleciona e aperta qualquer uma. o certo é o
  contrato ter "focável fora da sequência de Tab" (o tabindex -1), o que
  pede `sem` e engine. as linhas de lista e tabela não perdem nada (antes
  nenhuma era focável).
- o split da Resizable: pede um papel de splitter no contrato.
- os atalhos anunciados: o `sem::Node` não tem campo de atalho.
- PageUp e PageDown nas listas, e o menu de contexto pelo teclado: a
  ponte (`src/keys.rs`) não manda essas teclas.
- a Data Table desenha as linhas do overscan abaixo do painel (já era
  assim; os PNGs antes e depois são iguais). o anel nunca cai nelas: o
  foco fica nas linhas em vista.
- o roteiro do VoiceOver pode ganhar um passo de teclado (Tab, setas,
  listas e o modal), para o brenner rodar.
