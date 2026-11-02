---
id: task-75/phase-2a-app-pages
title: "task-75 phase 2a: app pages that already project"
project: plev
type: section
parent: task-75
status: reference
date: 2026-09-25
date_basis: text
recorded: 2026-09-23
topics: [accessibility, semantics, agents]
relations: [adr-0068, task-60, task-70]
evidence: [168b240e, b7059376, d6a6def9, bcc57315, b8616294, 61bdfcec, 24df5d78, 17c903c9, f208fbab, 0ff5e28d, f3e9f914]
source: docs/log/steps/task-75-platform-accessibility.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# phase 2a: the probe (2026-09-25)

Records phase 2a on the pages that already project (Todo, Toggle Group, Surface, Silences, Files and Capture) and on the shell. The probe showed that the projection of a static page is a golden of `uir` and enters the regress's `scene.json`, so changes to the page's meaning show up on the page, and changes to the adapters' presentation show up in the reader's tree. The result: actions that no item reaches become controls inside it, the select opens and closes on press, the projected focus is what the page takes, and the accessibility preference reaches only the reader, never the port or the prompt. The baselines that change are only the two of Capture, in the `value` of `capture.level`.

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

- os goldens do `uir`: `rust/plevir/tests/fixtures/uigallery-todo.snapshot.json`
  é o `ShowcaseView::semantics()` da Todo parada
  (`apps/uigallery/tests/plevir_fixture.rs:32-42`);
  `showcase-silences.snapshot.json` e `showcase-files.snapshot.json` são
  o `snapshot()` da Silences e da Files paradas
  (`view/apps/silences/tests.rs:861-893`, `view/apps/files/tests.rs:833-860`).
  o IR da quick todo verifica contra o primeiro, e os testes de
  `propose`, `surface` e `claude-model` leem os três. `rust/plevir/**` é
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
(`rust/engine/src/control/accessibility/tree/mod.rs`), onde a fase 1 já pôs regras
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
     `surface` só guarda foco de campo (`rust/screen/src/surface.rs:404-415`)
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
     repeat pela chave (`rust/screen/src/project.rs:38-41`), "1", "2",
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

- `rust/socket` e `apps/idestudio`: nenhum teste lê a projeção da galeria
  (grep por `ShowcaseView` e `gallery_semantics`: zero), e o ide não liga
  a ponte (grep por `Bridge` em `apps/idestudio/src`: zero). nada muda.
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
- o laboratório é do ide (`apps/agenteval/src`) e não lê a galeria.
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

o que cada passo espera vem dos testes (`apps/uigallery/tests/access_tree.rs`,
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
golden do `uir` (`rust/plevir/tests/fixtures/showcase-{todo,silences,files}.snapshot.json`,
arquivo proibido nesta fase), é o registro de capacidades que o IR e o
prompt leem (os controles entrariam como capacidades duplicadas) e entra
no `scene.json` do regress (revisão e bindings). por isso os controles
das linhas, o checkbox da linha, o aperto do select, o nome da status, o
nome dos itens da quick todo e o aperto só com a ação ligada são
apresentação da árvore, no `rust/engine/src/control/accessibility/**`
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
- depois, em `f3e9f914`: `./tool/tasks/fullcheck.sh` verde, pernas urnaui e wasm
  rodaram; 3086 passam (+24: 3001 no workspace, 85 no urnaui), 27
  ignorados, 0 falhas. `regress check`: 176 baselines, 176 iguais.
- `Cargo.toml` e `Cargo.lock` intocados (md5 do lock igual), nenhuma
  dependência nova, AccessKit na mesma versão; `rust/plevir`,
  `rust/screen`, `rust/accord`, `rust/socket/src`, `rust/offers` e
  `apps/idestudio/src` intocados. sempre com `PLEV_CLAUDE` no stub e
  `--offline`; nenhuma janela aberta, nenhum leitor ligado, nenhum clique
  sintético, nenhum diálogo nativo.

### linha para o índice de ADRs

| 2026-09-25 | [accessibility-is-production-ready-per-platform](../../adr/Interface/0068-accessibility-is-production-ready-per-platform/0068-accessibility-is-production-ready-per-platform.md) | proposed | fase 2a: nas páginas que projetam, as ações de um item que ele mesmo não alcança são controles que a árvore apresenta dentro dele, o select abre pelo aperto e se escolhe pelo valor, o aperto só com a ação ligada, a mensagem de uma status é o nome dela; as páginas projetam o foco que tomam; o toast, o modal e o menu na projeção da view, pelo dispatcher; o aviso do shell e a preferência de accessibility só na árvore do leitor, nunca no port nem no prompt; spinner e skeleton seguem o reduced motion (task-75 fase 2a) |

### o que fica para a fase 2b

- as 76 páginas de componente, que não projetam nada.
- o teclado completo nos widgets: Tab em ordem semântica nas páginas que
  projetam (hoje a Todo não anda com Tab), setas em listas, menus, tabs e
  árvores, o trap de foco no modal (o foco já vai e volta, o teclado
  ainda não o prende) e o anel de foco nas pílulas do todo.
- bounds para os controles apresentados: pede a página ligar um retângulo
  por ação, e isso muda o `scene.json` de baselines existentes.
- o ide não liga a ponte (`apps/idestudio/src`: sem `Bridge`); a página Git da
  galeria não projeta.
- windows, AT-SPI, iOS e android seguem esperando o alvo ou o adapter.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0068: accessibility is production-ready per platform](../../adr/Interface/0068-accessibility-is-production-ready-per-platform/0068-accessibility-is-production-ready-per-platform.md)
- [task-60: accessibility as a projection of the contract](../task-60-accessibility-projection/task-60-accessibility-projection.md)
- [task-70: adaptive transformations](../task-70-adaptive-transformations/task-70-adaptive-transformations.md)
- [task-75: production-ready platform accessibility](task-75-platform-accessibility.md)
<!-- /related -->
