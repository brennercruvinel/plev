---
id: task-76/item-6-deep-links
title: "task-76, item 6: deep links"
project: plev
type: secao
parent: task-76
status: referencia
date: 2026-09-24
date_basis: texto
topics: [plataforma, agentes, input]
relations: [adr-0063]
evidence: [e3b6c8d2, df91e2a8, edefe39b, 66345cf5, e52359d0]
source: docs/log/steps/task-76-platform-integration.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# handoff do item 6: deep links

Registra o item 6: `engine::platform::links` lê `plev-showcase://<id>/<ação>[?value=<texto>]` com gramática estrita e no máximo 2048 bytes, e o pedido de um link é `Request::observe` por um ator `link`, nunca `Actor::Human`, numa lista de capacidades abertas a link, que no showcase são só `shell.page` e `shell.theme`. No macOS o link chega só pelo argumento de lançamento até o bring-up, porque o sistema entrega o esquema a um `.app` registrado, e trocar o delegado do winit 0.30.13 para receber o evento quebra o laço. O resultado é que nenhum link liga câmera, impressora, diálogo ou janela, cada recusa aparece na tela e em `shell:notice`, e a branch `task/76j-links` ainda não está na `main`.


branch `task/76j-links`, base `bcf6be8e` (main após o item 10). worktree
`.claude/worktrees/agent`, target `target/agents/one`. commits, em ordem:

- `e3b6c8d2` task-76: the deep link probe
- `df91e2a8` task-76: a link read by a strict grammar, and its sources
  with the fake
- `edefe39b` task-76: a link is not a person, and the macOS path recorded
  for the bring-up
- `66345cf5` task-76: the showcase follows a link as the agent link, only
  where it is open
- `e52359d0` task-76: a link is checked as an agent's, and its refusal is
  on screen
- (este) task-76: the item 6 handoff and the close of task-76

a rodada parou uma vez, depois da sonda, em duas condições do briefing ao
mesmo tempo (a seção "a parada", abaixo): o winit 0.30.13 registra o
delegado de aplicação dele no macOS e depende dele, e escrever qualquer
classe objc no engine pede dependência direta que muda o lock. o
orquestrador decidiu a opção 4 (a seção "a decisão"). os testes de cada
comportamento saíram no commit dele; `e52359d0` são os de integração. em
nenhum momento: esquema registrado, `open` de um link, `.app`, janela.

## a sonda

(a) o macOS, por leitura da fonte e por `cargo tree --offline`:

- o engine não tem `objc2`, `objc2-foundation` nem `objc2-app-kit` como
  dependência direta: `rust/engine/Cargo.toml` só os cita no comentário
  do item 5 (o caminho do UserNotifications ficou para o bundle), e a
  entrada `engine` do `Cargo.lock` não os lista. eles estão no lock pelos
  outros: `objc2 0.6.4` (arboard, block2, dispatch2, muda, rfd, tray-icon
  e as crates `objc2-*`), `objc2-foundation 0.3.2` e `objc2-app-kit 0.3.2`
  (arboard, muda, rfd, tray-icon); o winit 0.30.13 usa os seus, `objc2
  0.5.2` e `objc2-app-kit 0.2.2`.
- as features ligadas hoje, pela unificação (`cargo tree --offline -p
  engine -e features -i <crate>`): `objc2` std, alloc, default;
  `objc2-foundation 0.3.2` NSArray, NSString, NSURL, NSNotification,
  NSBundle, NSData, NSDictionary, NSError, NSThread e outras, sem
  NSAppleEventManager e sem NSAppleEventDescriptor; `objc2-app-kit 0.3.2`
  NSApplication, NSResponder, NSRunningApplication, NSEvent, NSMenu,
  NSMenuItem, NSWindow, NSView, NSStatusBar e outras. então
  `define_class!` (`objc2-0.6.4/src/macros/define_class.rs`),
  `NSApplicationDelegate` com `application:openURLs:`
  (`objc2-app-kit-0.3.2/src/generated/NSApplication.rs:1128`, feature
  NSApplication), `NSApplication::setDelegate`, `NSURL` e `NSArray`
  compilariam com as features que já estão ligadas; falta ao engine poder
  nomear as crates.
- o lock, medido numa cópia do workspace no scratchpad, nunca neste
  worktree: com `objc2 = "0.6"`, `objc2-foundation` e `objc2-app-kit` em
  `cfg(target_os = "macos")` no manifesto do engine, `cargo metadata
  --offline` resolve sem baixar nada, e o `Cargo.lock` muda em três
  linhas, todas na lista de dependências da entrada `engine`
  (`"objc2 0.6.4"`, `"objc2-app-kit 0.3.2"`, `"objc2-foundation 0.3.2"`).
  nenhum pacote novo. é dependência nova do engine e muda o lock.
- quem registra delegado: o `muda` 0.20.0 só põe delegado num `NSMenu`
  (`platform_impl/macos/mod.rs:94`, `ns_menu.setDelegate`), nunca no
  `NSApp`; `tray-icon` 0.25.1, `rfd` 0.15.4, `arboard` 3.6.1 e
  `accesskit_macos` 0.26 não chamam `setDelegate` no app e não registram
  handler de Apple Event (`NSAppleEventManager` não aparece em nenhum). o
  winit 0.30.13 registra: `platform_impl/macos/event_loop.rs:240`, dentro
  de `EventLoop::new`, põe o `WinitApplicationDelegate` no app
  (`app_state.rs:47-72`, um `declare_class!` com
  `applicationDidFinishLaunching:` e `applicationWillTerminate:`, que
  guarda o estado todo do laço). e lê de volta a cada volta:
  `ApplicationDelegate::get` (`app_state.rs:173-184`) pega
  `NSApp.delegate()` e entra em pânico com "tried to get a delegate that
  was not the one Winit has registered" quando não é o dele, e é chamado
  pelo observador do run loop em todo `kCFRunLoopAfterWaiting` e
  `kCFRunLoopBeforeWaiting` (`observer.rs:62` e `:84`) e em todo
  `sendEvent:` (o swizzle de `app.rs:45`).
- o texto de `winit-0.30.13/src/platform/macos.rs:20-80` ("Winit
  guarantees that it will not register an application delegate", com o
  exemplo do `application:openURLs:`) não vale para a 0.30.13, que é a do
  lock: o changelog da 0.30 só tira o delegado do iOS. o briefing leu esse
  texto. um delegado nosso substituiria o do winit, e o laço entraria em
  pânico no primeiro evento ou na primeira volta depois do `setDelegate`.
  não rodei isso (nenhum laço nem janela); é leitura da fonte, linha por
  linha.
- o que entrega a URL sem trocar o delegado: o evento Apple de URL
  (classe `kInternetEventClass`, id `kAEGetURL`, os dois `'GURL'`), por
  `NSAppleEventManager::sharedAppleEventManager()` e
  `setEventHandler_andSelector_forEventClass_andEventID`
  (`objc2-foundation-0.3.2/src/generated/NSAppleEventManager.rs:61` e
  `:71`, feature NSAppleEventManager), com a URL em
  `paramDescriptorForKeyword(keyDirectObject).stringValue()`
  (`NSAppleEventDescriptor.rs:338` e `:288`). é o que chama o
  `application:openURLs:` por baixo: o AppKit instala o handler dele para
  `GURL` ao lançar e repassa ao delegado. um handler nosso instalado no
  `NSApplicationWillFinishLaunchingNotification` (observado pelo
  `NSNotificationCenter`, sem delegado) toma o lugar do do AppKit e deixa
  o do winit em paz. também pede `objc2` e `objc2-foundation` diretos (o
  lock muda nas mesmas três linhas, com o `objc2-app-kit` para o nome da
  notificação), mais as features NSAppleEventManager e
  NSAppleEventDescriptor, que ninguém liga hoje (feature não muda o
  lock). um handler de Apple Event é do processo: não registra esquema no
  sistema.
- no macOS o link nunca chega como argumento: o Launch Services lança o
  app (só um `.app` com `CFBundleURLTypes` no `Info.plist`) e entrega a
  URL como evento `GURL` logo depois do lançamento; com o app aberto, o
  mesmo evento vai para a instância viva, sem processo novo. o handler
  precisa existir antes do fim do `finishLaunching` para pegar o link que
  lançou o app; o `resumed` do showcase vem depois, tarde para esse link.

(b) windows e linux, por leitura (nada registrado):

- windows: o esquema é uma chave `HKEY_CURRENT_USER\Software\Classes\<esquema>`
  com o valor `URL Protocol` e `shell\open\command` igual a `"<exe>"
  "%1"`. abrir o link cria sempre um processo novo com a URL em
  `argv[1]`, com o app aberto ou não.
- linux: um `.desktop` com `MimeType=x-scheme-handler/<esquema>;` e
  `Exec=<exe> %u`, escolhido por `xdg-mime default <app>.desktop
  x-scheme-handler/<esquema>`; o `xdg-open` do link cria um processo novo
  com a URL no lugar do `%u`.
- a segunda instância: nos dois, um link com o app aberto abre outro
  processo, com outra janela e estado novo, e a instância que já roda não
  fica sabendo. encaminhar (mutex nomeado e pipe no windows, D-Bus ou
  socket no linux, e o processo novo sai) é do bring-up. no macOS não
  existe segunda instância: o evento vai para a viva.
- o showcase hoje: `App::new` toma o primeiro argumento como título de
  página (`jump_to_page`), que não acha `plev-showcase://...` e ignora
  sem dizer nada; o segundo seria o tema.

(c) as capacidades do showcase hoje, na projeção da galeria (o shell e a
página ativa; dos goldens da todo, das silences e da files, e das
semânticas da capture e da surface):

- shell: `shell.page` `select` e `shell.theme` `select` (texto dentro das
  opções, qualquer ator); `shell.detach` `toggle` (abre janela),
  `shell.show` `activate` (traz a janela), `shell.print` `activate`
  (impressora) e `shell.save_session` `activate` (só gravando, escreve
  arquivo), os quatro `PERSON_ONLY`.
- todo: `todo.input` `set_value` (texto) e `submit`, `todo.filter[..]`
  `select`, `todo.item[n]` `toggle` e `remove` (destrutivo), qualquer
  ator.
- silences: `set_value`, `increment` e `decrement` dos dois sliders,
  `detect_silence` (roda o ffmpeg), `preview_segment`, `apply_cut`,
  `reject_cut`, `apply_all`, qualquer ator.
- files: `files.drop` `drop_files` (lista, qualquer ator); `files.open`
  `open_file` e `files.warn` `warn` (human, assistive e test: diálogo).
- capture: `capture.camera` e `capture.microphone` `toggle`
  (`PERSON_ONLY`: dispositivo).
- surface: `propose` (o claude por filho) e o cancelar.
- a proposta do briefing se sustenta: abertos a link, só `shell.page`
  `select` e `shell.theme` `select`. navegar e trocar o tema não escrevem,
  não apagam, não abrem dispositivo, diálogo, arquivo, janela nem
  impressora, e não rodam processo. o resto que um agente pode (o todo,
  as silences, o drop) fica fechado a link pela lista, mesmo aceito pela
  policy: um link vem de qualquer página web, e a lista é do app.

## a parada

problema: o caminho do macOS que o briefing descreve (um delegado nosso
com `application:openURLs:`) não existe com este winit, e qualquer
caminho objc no engine pede dependência direta que muda o lock.

1. o delegado do winit: um delegado nosso substitui o
   `WinitApplicationDelegate`, e o winit entra em pânico na volta
   seguinte do laço (sonda (a)).
2. o lock: `objc2`, `objc2-foundation` (e `objc2-app-kit`) como
   dependência direta do engine mudam três linhas do `Cargo.lock`, sem
   pacote novo.

opções e impacto:

1. o handler do evento Apple `GURL`, sem delegado, com `objc2`,
   `objc2-foundation` e `objc2-app-kit` diretos no engine em macOS, as
   features NSAppleEventManager e NSAppleEventDescriptor a mais. o lock
   muda três linhas na entrada `engine`, resolve offline; uma classe com
   `define_class!` que observa o `WillFinishLaunching`, instala o handler,
   empurra a URL no canal e chama o wake. o teste chama o método do
   handler pelo objc com um descritor montado por ele, sem esquema e sem
   janela. na tela, só recebe algo depois do bring-up do bundle, como o
   `NativeTray` compilado e não rodado.
2. o mesmo handler contra o runtime objc por `extern "C"` (libobjc:
   `objc_allocateClassPair`, `class_addMethod`, `sel_registerName`,
   `objc_msgSend` transmutado), sem crate. o lock fica intacto; entram
   umas 120 linhas de `unsafe` com `// SAFETY:` e transmutes, contra o
   "unsafe proibido por padrão", com custo de revisão alto.
3. um método `application:openURLs:` posto em runtime na classe do
   delegado do winit. depende do nome privado de uma classe de outra
   crate, quebra calado numa atualização, e ainda pede o objc2 ou o
   runtime cru. não recomendo.
4. no macOS, só a fonte do argumento de lançamento até o bring-up; o
   handler `GURL` entra com o bundle, como a notificação do item 5. o
   manifesto e o lock ficam intactos; tudo o mais do briefing entra (o
   parse, as fontes, o falso, o showcase, o ator, a lista, a gravação, os
   testes, o ADR), e o ADR descreve o handler como o caminho do bring-up.
   `cargo run -p uigallery -- 'plev-showcase://shell.page/select?value=Files'`
   exercita o caminho inteiro no macOS sem registrar nada.

recomendação: a opção 1, se as três linhas do lock forem aceitas: é o
caminho de verdade do macOS, pequeno, sem pacote novo, e testável pelo
objc sem esquema. sem o aceite, a opção 4. as opções 2 e 3 não.

decisão necessária: aceitar as três linhas do `Cargo.lock` e as três
dependências diretas do engine em macOS (opção 1), ou seguir sem
caminho do sistema no macOS até o bring-up (opção 4). o resto do item
(commits 2, 4, 5 e 6 do briefing) não depende da escolha; só o commit 3.

## a decisão

do orquestrador, depois da parada: a opção 4. no macOS o link chega só
pelo argumento de lançamento até o bring-up; o manifesto e o
`Cargo.lock` ficam intactos, e não entram delegado nem handler de Apple
Event. o motivo: o evento `GURL` só chega a um app com o esquema
registrado, e registrar exige o bundle; o código da opção 1 ficaria sem
caminho real até lá, como a notificação do item 5. o commit 3 virou o
registro do desenho para o bring-up (o ADR, seção "bring-up"), sem código
de macOS. a fonte do argumento vale nas três plataformas, e no showcase
um argumento que começa com `plev-showcase://` é link, nunca título de
página.

## a gramática e os erros

`rust/engine/src/runtime/platform/links/mod.rs` (382 linhas, todas de produção; os
testes em `links/tests.rs`) e `links/source.rs` (212 de produção, 333 com
os testes). `engine::platform::links`:

- `Link { target: SemId, action: String, value: Option<String> }`;
  `Link::parse(url, scheme)` lê `<esquema>://<sem id>/<ação>[?value=<texto>]`.
  olha nesta ordem e para no primeiro que vale: o tamanho (no máximo
  `MAX_LEN` = 2048 bytes: um link que navega tem menos de cem; 2048 deixa
  o parse e a linha da recusa limitados, abaixo dos 2083 caracteres que o
  shell do windows repassa); os caracteres (só ASCII imprimível, 0x21 a
  0x7E: espaço, controle e letra fora do ASCII viajam percent-encoded); o
  esquema (o do app, sem diferença de caixa, RFC 3986 3.1); fragmento
  (`#`); usuário ou senha (`@`) e porta (`:`) antes da primeira `/`; barra
  a mais; o id (vazio, depois o percent-decoding, depois `SemId::parse`);
  a ação (vazia, ou fora de `[a-z_]`, sem decoding); a consulta (só a
  chave `value`, uma vez; o valor com percent-decoding, `+` é mais, espaço
  é `%20`). todo decoding recusa um `%` sem dois dígitos hex, bytes que
  não são UTF-8, e texto com caractere de controle ou marca, override ou
  isolate de direção (a recusa é uma linha que a pessoa lê: um `%0A`
  forjaria uma segunda linha, um `%E2%80%AE` inverteria a leitura).
- `LinkError::{TooLong, BadChar, NotALink, OtherScheme, Fragment,
  UserInfo, Port, ExtraPath, NoTarget, BadTarget, NoAction, BadAction,
  NoValue, UnknownKey, ExtraKey, Percent, Utf8, Control}`, cada um com o
  texto que diz por quê ("a link has no fragment (#)", "the link's id is
  not an id: invalid character 'S' at byte 0", "the link has 3000 bytes,
  above the 2048 a link may have").
- `Link::param()` (o valor como `Value::Text`), `Link::is_in(open)` (a
  lista do app, pares `(id, ação)`), `Link::request(seen)`
  (`Request::observe` por `actor()`), `Link::url(scheme)` (a forma que o
  parse lê de volta: os colchetes do id crus, o resto fora de
  `[A-Za-z0-9._~-]` como `%XX`, uma linha de ASCII imprimível);
  `is_link(text, scheme)` (começa com `<esquema>://`, sem diferença de
  caixa); `excerpt(text)` (96 caracteres no máximo e reticências, o resto
  escapado); `ACTOR` = `"link"`.
- `Refused { link: excerpt, why: LinkError }` ("<link>: <por quê>"),
  `Delivered = Result<Link, Refused>`, `read(raw, scheme)`.

## o ator e a lista aberta

- o ator: `Actor::Agent { name: "link" }` (`links::actor()`), nunca
  `Actor::Human` e nunca um ator novo. o `Actor::Agent` bastou: a coluna
  `agent` da policy é o mínimo que o contrato dá a quem vem de fora, o
  `sem` não mudou, e o dispatcher checa um link como checa um agente
  (`PERSON_ONLY` é `NotAuthorized`, stale, inert, disabled, o parâmetro,
  e uma confirmação que a policy pede de agente espera uma pessoa, que o
  link não pode dar: `ConfirmByMachine`).
- a lista do showcase (`showcase::app::links::OPEN`): `shell.page`
  `select` e `shell.theme` `select`, e nada mais. fora dela, nenhum
  request é feito, mesmo quando a policy deixaria um agente (o todo, as
  silences, o drop, a surface). dentro dela, o dispatcher ainda decide.
- a recusa: nada roda, e a pessoa vê uma linha, `link refused: <link>:
  <por quê>`, como toast na tela (`Intent::Destructive`) e em
  `shell:notice` (o agente lê), e o log diz (`warn`). os porquês: o que o
  parse disse; "`<id> <ação>` is not open to links"; o `SemError` do
  `observe` quando a capacidade sumiu; do dispatcher, "`<alvo>` is not
  there now", "agent:link may not ask this", "what it read changed
  before it ran", "`<alvo>` is behind an overlay now", "`<alvo> <ação>` is
  disabled now", o motivo do `InvalidParam` (`"nope" is not one of the
  options`); uma falha é "link failed: ...". um link que espera
  confirmação diz "link waits for a person to confirm it: <link>" pelo
  mesmo caminho (toast informativo). um link que roda não diz nada além
  do que mudou (o log em `info`).

## o caminho por plataforma

- windows e linux: a fonte do argumento de lançamento (`LaunchLinks`),
  que é como o sistema entrega um esquema registrado: um processo novo com
  a URL como argumento. a segunda instância abre outra janela com o link
  aplicado nela; encaminhar é do bring-up.
- macOS: a mesma fonte, hoje (`cargo run -p uigallery -- <link>`, ou `open
  -a <app> --args <link>` depois do bundle). o sistema só entrega o
  esquema a um `.app` com `CFBundleURLTypes`, como evento `GURL`; esse
  handler é do bring-up (abaixo e no ADR). nada de delegado: o do winit
  0.30.13 entra em pânico se for trocado (sonda (a)).
- wasm, iOS, android: `NoLinks`. os shells móveis entregam links pela
  própria entrada (`application:openURL:options:`, o data da intent), do
  bring-up.
- `default_links(scheme, wake)`: o argumento em todo desktop, lido quando
  chamado; `NoLinks` no resto. o `wake` é para a fonte que entrega com o
  app rodando (o handler `GURL` do bring-up); o argumento, lido uma vez,
  nunca o chama.
- o desenho do bring-up no macOS, registrado: `objc2`, `objc2-foundation`
  (features `NSAppleEventManager`, `NSAppleEventDescriptor`,
  `NSNotification`, `NSString`) e `objc2-app-kit` (`NSApplication`) como
  dependências diretas do engine em macOS, as três linhas do lock medidas
  na sonda (`"objc2 0.6.4"`, `"objc2-app-kit 0.3.2"`, `"objc2-foundation
  0.3.2"` na entrada `engine`, sem pacote novo); uma classe pequena
  (`define_class!`) que observa `NSApplicationWillFinishLaunchingNotification`
  pelo `NSNotificationCenter` e então se instala com
  `NSAppleEventManager::sharedAppleEventManager().setEventHandler_andSelector_forEventClass_andEventID`
  para `GURL`/`GURL` (depois que o AppKit instalou o dele, antes que os
  eventos do lançamento sejam despachados, então o link que lançou o app
  é pego), lê a URL de `paramDescriptorForKeyword(keyDirectObject).stringValue()`,
  põe na fila da fonte e chama o `wake` (o showcase manda a si mesmo
  `UserEvent::Link`). o teste chama o método do handler pelo objc com um
  descritor montado por ele, sem esquema e sem janela. o delegado do
  winit fica: `EventLoop::new` põe o `WinitApplicationDelegate`
  (`platform_impl/macos/event_loop.rs:240`), e `ApplicationDelegate::get`
  (`app_state.rs:173-184`) entra em pânico com "tried to get a delegate
  that was not the one Winit has registered" a cada
  `kCFRunLoopAfterWaiting` e `kCFRunLoopBeforeWaiting` (`observer.rs:62`
  e `:84`) e a cada `sendEvent:` (`app.rs:45`) se não for o dele.

## o showcase

`apps/uigallery/src/shell/system/links/mod.rs` (208 de produção; os testes em
`app/links/tests.rs`), `app.rs` 596 (era 571).

- `pub mod links` em `app.rs`, público para o teste de integração:
  `SCHEME` (`plev-showcase`), `OPEN`, `follow(view, delivered, open,
  looked) -> Followed { receipt, told }`. `follow` recusa o que não é
  link, o que não está na lista e o que o `observe` não acha; senão avisa
  `looked` com a revisão vista (a gravação) e pede `view.act(req)`; o
  estágio do receipt decide se há linha.
- o shell: o campo `links: Option<Box<dyn LinkSource>>`;
  `install_links()` (`default_links` com o `wake` que manda
  `UserEvent::Link` pelo proxy no desktop) e `links_arrived()` no
  `resumed`, depois das outras instalações, com a janela principal de pé:
  o link de lançamento é lido e seguido então. `links_arrived` também no
  `resumed` de um app que já tinha janela (depois do `wake`) e no
  `UserEvent::Link`. dormindo (`Background::asleep()`), não faz poll: o
  link espera na fonte e é aplicado no primeiro poll acordado. cada link
  seguido, um `changed()` (estado gravado, frame pedido).
- `App::new` virou `with_view(launch_view(args))`; `launch_view` tira dos
  argumentos posicionais (página, tema, scroll) os que começam com
  `plev-showcase://` (`is_link`). antes, um link como primeiro argumento
  era tomado como título de página e ignorado calado; agora é link, ou
  recusa visível.
- a gravação (`PLEV_RECORD=1`): o `Observed` do `agent:link` sobre a
  galeria inteira na revisão vista (`Recorder::observed`, como a porta do
  agente grava) e cada estágio do receipt pelo sink do dispatcher, com
  `request.actor` = `agent:link`. nenhum `Input`: nada foi clicado nem
  digitado.
- nenhuma página, nenhum arquivo da view, nenhuma projeção mudou: a linha
  vai para o `shell:notice` que o item 5 já projetava (`set_notice`) e o
  toast é o `ToastManager` da view. `view/dispatch/shell.rs` não precisou
  mudar.

## testes

- engine, `platform/links/tests.rs` (12): os válidos (id com chave crua e
  codificada, valor com `%20`, UTF-8, `+`, vazio, esquema em outra
  caixa); 28 malformados, cada um com o seu `LinkError` (sem `://`, outro
  esquema, fragmento, usuário, senha, porta, barra a mais, sem id, sem
  ação, ação fora de `[a-z_]`, `?` vazio, `?value` sem `=`, chave
  desconhecida, chave a mais); o id inválido (caixa, segmento vazio,
  chave sem fechar, `%20`); o caractere cru fora do ASCII imprimível e o
  byte onde está; o percent inválido, o UTF-8 inválido, o controle e a
  direção (`%0A`, `%0D`, `%00`, `%7F`, `%C2%85`, `%E2%80%AE`, `%E2%81%A6`);
  2048 aceito e 2049 recusado, e o tamanho olhado antes de tudo; a forma
  de volta (`url`) e a volta dela; `is_link` sem pânico de fronteira de
  char; `excerpt`; o request por `agent:link`, nunca `Human`, sem `ask`;
  a lista por id e ação; cada erro com texto próprio numa linha.
- engine, `platform/links/source.rs` (5): os argumentos que começam como
  link, em ordem, uma vez, os outros deixados; um argumento que não é
  UTF-8 é recusado, nunca pulado; a recusa é uma linha curta (o longo, o
  `\n`, o `\u{202e}`); o falso em ordem e compartilhado pelo handle;
  `NoLinks` e o `default_links` deste processo (nenhum link nos
  argumentos do harness).
- `rust/engine/tests/links.rs` (5, binário novo), um app de teste pelo
  `Dispatcher` com o `FakeLinks`: o link roda por `agent:link` onde o app
  abriu e a lista vem antes; com qualquer lista a policy vale como para
  agente (`NotAuthorized` no só-de-pessoa, `InvalidParam` fora das opções
  e acima do `max_len`); a confirmação que a policy pede de agente fica
  `AwaitingConfirmation`, o link não confirma (`ConfirmByMachine`), a
  pessoa confirma e roda; o link que viu um estado velho é `Stale`, e o
  de uma capacidade que saiu não faz request (`UnknownAction`); o que não
  é link deste app não faz request nenhum.
- showcase, `app/links/tests.rs` (7), o shell sobre janela nua com o
  `FakeLinks` e um sink no dispatcher da galeria: um link de `shell.page`
  troca a página para Files pelo dispatcher, quatro estágios por
  `agent:link`, frame pedido, nada contado; tema e "Data%20Table" também;
  `capture.camera` (com um `FakeCapture`: 0 starts), `shell.print`,
  `files.open` `open_file`, `todo.item[1]` `remove`, um id que não existe,
  um fragmento e uma chave desconhecida são recusados sem request nenhum,
  a página igual, e a linha exata em `shell:notice` e num toast; dentro
  da lista, `shell.theme` com valor fora das opções é recusado pelo
  dispatcher (`requested`, `rejected`) e a linha diz; uma lista descuidada
  com `shell.print`, `files.open` e `capture.camera` dá `NotAuthorized`
  por `agent:link` nos três e nada roda; um link que chega dormindo não é
  lido em três polls (fica na fonte, nenhum frame) e é aplicado no
  primeiro poll depois do `wake`; o argumento de lançamento é link e não
  título (`launch_view`), com o tema depois dele no lugar, e a
  `LaunchLinks` dos mesmos argumentos leva à Files e recusa
  `plev-showcase://shell.page` na tela; a gravação tem `observed` e os
  quatro `stage` de `agent:link` e nenhum `input`; a lista é a página e o
  tema, abertos a agente, não destrutivos, sem confirmação.
- `apps/uigallery/tests/links.rs` (2, GPU, binário novo), sobre um host
  headless 1200x800: o link da câmera na Capture é recusado sem request,
  o toast com a linha é desenhado (mais da metade dos pixels do retângulo
  do toast mudou) e nenhum pixel mudou fora dele e da margem da sombra
  (48 px), `capture:camera` segue "off"; o link da Files a partir da
  Todo desenha, byte a byte, o frame de uma galeria que foi para a Files
  pelo nome da página.

a prova do roteiro, só por teste e sem janela: o link da Files pelo
argumento é `the_launch_argument_is_a_link_and_never_the_page_title` e
`a_link_to_the_files_page_draws_that_page_byte_for_byte`; o da câmera
recusado e na tela é
`what_is_not_open_to_links_is_refused_nothing_runs_and_the_person_is_told`
e `a_link_to_the_camera_is_refused_and_its_line_is_on_screen`.

## gate e regress

`df -h /` antes de cada gate: 17 GB livres (base) e 16 GB (depois); 16 GB
no fim. `./tool/tasks/fullcheck.sh` verde no worktree com `CARGO_NET_OFFLINE=true`,
com a perna wasm e a perna urnaui (`../urna` presente): antes (a base
`bcf6be8e`, medida neste worktree) 2641 passed, 0 failed, 17 ignored em
128 binários; depois 2672 passed, 0 failed, 17 ignored em 130 binários
(+31: 12 da gramática, 5 das fontes, 5 de `engine/tests/links.rs`, 7 do
shell, 2 de `showcase/tests/links.rs`; +2 binários, os dois
`tests/links.rs`). perna urnaui: 85 passed, 0 failed, 1 ignored em 6
binários, igual a antes. clippy `-D warnings`, fmt e a checagem wasm
limpos nas duas pernas; o único aviso no log é o `block v0.1.6` de
incompatibilidade futura, que já estava. `Cargo.lock`,
`apps/urnadbapp/Cargo.lock`, `rust/engine/Cargo.toml` e o `Cargo.toml`
da raiz sem mudança (`git diff bcf6be8e` vazio neles).

`cargo run --offline -p regress -- check`: 163
baselines, 163 iguais, 0 mudaram, sem frame desenhado e sem GPU aberta
(743 ms). nenhuma projeção de página mudou; nenhum golden, nenhum bless.

linhas de produção (`file_hygiene_guard` verde): engine
`platform/links.rs` 382, `platform/links/source.rs` 212,
`platform/mod.rs` +1; showcase `app.rs` 596 (era 571), `app/links.rs`
208. tudo abaixo de 639.

## ADR

[`0063-a-link-is-not-a-person.md`](../../decisoes/interface/0063-a-link-is-not-a-person.md) (proposed): a gramática, o ator, a
lista aberta, a recusa vista, as fontes, o caminho por plataforma hoje, o
shell, a gravação, e o bring-up (o `CFBundleURLTypes` no `.app` e o
handler `GURL` pelo `NSNotificationCenter` com as três dependências, a
chave do registro do windows, o `.desktop` do linux, a segunda
instância, iOS e android). linha para o índice ([`indice-historico.md`](../../historia/indice-historico.md) está
fora da lista, fica para a integração):

```text
| 2026-09-24 | [a-link-is-not-a-person](a-link-is-not-a-person.md) | proposed | task-76 item 6: `engine::platform::links` lê `<esquema>://<sem id>/<ação>[?value=<texto>]` com gramática estrita (no máximo 2048 bytes de ASCII imprimível, o esquema do app, o id por `SemId::parse` depois do percent-decoding, uma `/`, ação de `[a-z_]`, só a chave `value`, texto UTF-8 sem controle nem marca de direção; sem fragmento, usuário, senha, porta ou barra a mais; `LinkError` diz por quê); o request de um link é `Request::observe` por `Actor::Agent` de nome `link`, nunca `Actor::Human` e sem ator novo no `sem`, e o app só o pede para o que lista como aberto a link (`Link::is_in`), a policy valendo por cima como para agente; `LinkSource` com `LaunchLinks` (os argumentos que começam com `<esquema>://`, em todo desktop), `FakeLinks`, `NoLinks` (wasm e mobile), `default_links`; no macOS o link chega só pelo argumento até o bring-up, porque o sistema entrega o esquema como evento `GURL` a um `.app` registrado e o delegado do winit 0.30.13 entra em pânico se for trocado: o handler `GURL` pelo `NSNotificationCenter`, com `objc2`, `objc2-foundation` e `objc2-app-kit` diretos (três linhas do lock), é do bring-up; o showcase (`plev-showcase`) abre só `shell.page` e `shell.theme` `select`, recusa o resto sem request, conta a recusa numa linha como toast e em `shell:notice`, lê o link de lançamento com a janela de pé (nunca como título de página), deixa o link dormindo na fonte até o wake e grava o link como a observação e o receipt de `agent:link`, sem input |
```

## interfaces adicionadas (só crescimento)

- `engine::platform::links::{Link, LinkError, MAX_LEN, ACTOR, actor,
  is_link, excerpt, Refused, Delivered, read, LinkSource, LaunchLinks,
  FakeLinks, NoLinks, default_links}`; `impl LinkSource for Box<L>` e
  `for Arc<Mutex<L>>`; `engine::platform::links` em `platform/mod.rs`
  (uma linha).
- showcase: `showcase::app::links::{SCHEME, OPEN, Followed, follow}`
  (público); `UserEvent::Link` (privado do app); `App::{install_links,
  set_links, links_arrived}` (`pub(super)`), `App::launch_view`
  (privada; `App::new` tem a mesma assinatura e o mesmo comportamento
  para quem não passa link). nenhuma assinatura pública existente mudou;
  ide, urnaui, wasm e os exemplos compilam sem tocar.

## arquivos tocados fora da lista permitida, e por quê

- `apps/uigallery/src/shell/system/links/tests.rs`: os testes do módulo novo
  `app/links.rs`, num arquivo à parte como `shell/tests.rs` (o guard não
  conta arquivo `tests.rs`). a lista nomeava `app/links.rs` e
  `apps/uigallery/tests/**`.
- nenhum outro. `app/entry.rs` e `view/dispatch/shell.rs` não precisaram
  mudar (o proxy já está no `Access` antes do `run_app`; o `set_notice`
  já era público).

## limitações e decisões, sem esconder

- nenhum link de verdade foi aberto nesta máquina e nenhum esquema foi
  registrado: tudo é o falso, a `LaunchLinks` com argumentos montados no
  teste, e o `follow` sobre a view. o `cargo run -- <link>` com janela é
  do brenner (roteiro abaixo).
- o macOS não recebe link do sistema até o bring-up (a decisão). o texto
  do winit que o briefing leu não vale para a 0.30.13; o pânico de um
  delegado trocado é leitura da fonte, não rodado.
- a lista vem antes da policy de propósito: um link fora da lista não
  vira request, então não deixa receipt nem estágio na gravação (que
  guarda a projeção da página, não o `shell:notice` da galeria); o log,
  o toast e o `shell:notice` dizem. a alternativa (despachar tudo e
  deixar a policy recusar) aceitaria `todo.item[n] remove`, que a policy
  deixa a um agente, vindo de qualquer página web.
- dentro da lista, uma confirmação pedida pela policy espera uma pessoa.
  o showcase não tem hoje resposta de pessoa para confirmação de agente
  (o ide tem o modal e o diálogo nativo do item 8): o request ficaria
  aberto. a lista de hoje não tem nada que peça confirmação (testado).
- o esquema é comparado sem diferença de caixa (RFC 3986): `PLEV-SHOWCASE://`
  é link. o `+` do valor é mais, não espaço: quem monta o link escreve
  `%20` (o `URLSearchParams` do navegador escreve `+`; um título com
  espaço vindo dele seria recusado como opção desconhecida, dito na
  tela).
- o `USAGE` do binário (`src/main.rs`, fora da lista) não fala do link;
  um link como argumento cai no modo janela porque não começa com `--`.
- o toast da recusa some depois da duração do `ToastManager`;
  `shell:notice` fica até a próxima coisa contada.
- `showcase::app::links` é público para o teste de integração; a outra
  forma (o teste dentro do crate) não teria o frame headless com GPU no
  mesmo estilo do `tests/sleep_wake.rs`.

## o roteiro da prova real (do brenner, nunca de teste)

hoje, pelo argumento (nada registrado, nenhum `open`):

```text
RUST_LOG=showcase=info cargo run --offline -p uigallery -- 'plev-showcase://shell.page/select?value=Files'
   a janela abre na Files. no log: "link: plev-showcase://shell.page/select?value=Files: completed".
cargo run --offline -p uigallery -- 'plev-showcase://capture.camera/toggle'
   a janela abre na página de sempre, a câmera não liga (nenhuma luz, nenhum ffmpeg:
   pgrep -fl "ffmpeg.*avfoundation" vazio), e um toast embaixo à direita diz
   "link refused: plev-showcase://capture.camera/toggle: capture.camera toggle is not open to links";
   o log diz o mesmo em warn.
cargo run --offline -p uigallery -- Todo dark 'plev-showcase://shell.theme/select?value=nope'
   a Todo no tema dark (o link não tomou o lugar de nenhum argumento), e o toast
   "link refused: ...: "nope" is not one of the options".
PLEV_AGENT=1 cargo run --offline -p uigallery -- 'plev-showcase://shell.print/activate'
   e em outro terminal: cargo run --offline -p socket --bin plev -- observe --scope shell
   shell:notice = "link refused: plev-showcase://shell.print/activate: shell.print activate is not open to links"
PLEV_RECORD=1 cargo run --offline -p uigallery -- 'plev-showcase://shell.page/select?value=Files'
   cmd-shift-s, e o plev-inspect da sessão: o observed e o receipt de agent:link, nenhum input antes dele.
```

depois do bring-up do bundle (o `.app` com `CFBundleURLTypes` de
`plev-showcase` e o handler `GURL` no engine):

```text
open 'plev-showcase://shell.page/select?value=Files'     (o app fechado: abre na Files; aberto: a janela viva vai para a Files, sem segunda instância)
open 'plev-showcase://capture.camera/toggle'              (recusado na tela, a câmera apagada)
um link numa página web (<a href="plev-showcase://shell.theme/select?value=dark">) clicado no navegador: o tema muda; o navegador pergunta antes de abrir o app
desfazer: tirar o .app de /Applications (o Launch Services esquece o esquema), ou
   /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -u <o .app>
```

o que olhar: nenhum link liga câmera, microfone, impressora, diálogo ou
janela; toda recusa aparece na tela e em `shell:notice`; o link que
lançou o app é aplicado uma vez.

## comandos de validação

```text
export CARGO_TARGET_DIR=/Users/nn/Dev/hoff/plevdev/target/agents/one
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_PROFILE_TEST_DEBUG=line-tables-only
export CARGO_NET_OFFLINE=true
cargo test --offline -p engine --lib platform::links
cargo test --offline -p engine --test links
cargo test --offline -p uigallery --lib app::links
cargo test --offline -p uigallery --test links
cargo test --offline -p engine -p uigallery
./tool/tasks/fullcheck.sh
cargo run --offline -p regress -- check   (163 iguais)
git diff bcf6be8e -- Cargo.lock apps/urnadbapp/Cargo.lock rust/engine/Cargo.toml Cargo.toml   (vazio)
```
