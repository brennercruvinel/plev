---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-23
domain: generative-runtime
task: task-69
wave: 6
depends: [task-68, task-59, task-58]
depends-phase-0: []
branch: task/69-surfaces
---

# task-69: superfícies efêmeras e geradas

## decisões do orquestrador (2026-09-24)

- **ordem das fases:** 1 (materializar) agora, depois 2 (ciclo de vida e
  transformação), 3 (proponentes) e 4 (a demo do vídeo). a fase 0 (spike
  em `refs/`) não roda. a pergunta de UX dela é respondida de verdade no
  checkpoint b, com a superfície gerada de verdade.
- **o host mora num crate novo, `surface`.** ele depende de `uir`, `sem`,
  `engine` e `comps`. o `comps` continua biblioteca de widgets e não
  depende do IR.
- **a projeção sai do IR, não do `project` dos widgets.** 9 dos 21
  componentes do vocabulário não têm `project` no comps (achado da
  task-68). a superfície é semântica por construção: role por
  `uir::role_of`, label, estado lido das ligações no snapshot vivo do app,
  e as ações que o IR ligou.
- **a fase 1 prova com as fixtures da task-68**, em testes sem janela e
  numa página do showcase. a demo do vídeo continua na fase 4, como o
  plano diz. o urnaui já usa ffmpeg em subprocesso, então o domínio do
  vídeo tem base no código. a fixture da revisão de silêncios nasce na
  fase 4, com as capacidades de verdade.
- **o provedor do LLM (fase 3) é decisão do brenner**, como o arquivo
  diz. a fase 3 para ali e pergunta.

## objetivo

uma superfície de interface nasce de uma intenção, é materializada a
partir de IR verificado, é operada por humano e agente pelo mesmo
contrato, se transforma sem perder identidade e desaparece quando a
tarefa termina. nunca foi codificada como tela.

## ligação com a tese

seções 1 (o ciclo invertido), 4 (a interface deixa de ser fixa: o editor
de vídeo, os silêncios, a comparação, o espectro de fixa até gerada sob
demanda no mesmo runtime), 5 (LLM como interface semântica, plev como
interface perceptual), 15 (restrições), 18 (GPU-first para superfícies que
existem por poucos segundos), 33 (uma interface pode existir por vinte
segundos e nunca ter sido tela). goal 1 e goal 8 do programa. checkpoint b.

## estado atual no código

- não existe host de superfície materializada; o padrão oficial é estado
  em structs com widgets retained (`docs/ADR/official-app-pattern.md`).
- `crates/engine/src/layout` (taffy) resolve layout a partir de árvore;
  `crates/comps` tem os widgets; `crates/comps/src/overlay` tem spring para
  entrada e saída.
- o urnaui já decodifica frames de mídia com ffmpeg em subprocesso
  (`crates/urnaui`, ADR `urnaui-media-frames-in-place-via-ffmpeg-subfile`).
  ffmpeg tem `silencedetect`.
- depois de 68: `VerifiedIr`, `Capabilities`.

## escopo

### fase 0: spike descartável em `refs/` (pode rodar desde a onda 4)

a revisão de silêncios montada à mão, com capacidades falsas, só para
testar a hipótese de UX cedo: uma superfície que aparece para uma tarefa é
melhor que a tela fixa? nada é integrado. entrega: o relatório do que foi
aprendido e o que muda nas tasks 68 e 69. segue a regra de `refs/` do
AGENTS.md.

### fase 1: materializador

- `SurfaceHost` (nome e lugar decididos no ADR: comps ou crate próprio):
  guarda as superfícies ativas; cada uma é um `VerifiedIr` mais as
  instâncias de widget retained indexadas por `SemId`.
- layout a partir do `Layout` do IR pelo `LayoutEngine` do engine (sem
  segundo motor de layout); tokens resolvidos contra o `Theme` atual.
- render pelos widgets do comps, com `SemId` e ligação (59).
- projeção semântica automática: a superfície é semântica por
  construção, porque o IR já carrega identidade, role, label e as ligações
  de estado e ação.
- eventos dos widgets viram `Request` pelo dispatcher (58, 59). a
  superfície nunca executa nada sozinha.

### fase 2: ciclo de vida e transformação

- criar, materializar, operar, transformar, dispensar.
- transformar = receber um novo `VerifiedIr` e reconciliar por `SemId`: o
  mesmo `SemId` mantém a instância do widget e o estado dela (um slider em
  arraste continua em arraste quando muda de painel); `SemId` novo nasce;
  `SemId` que sumiu sai com transição.
- transições de entrada, saída e movimento com `theme.duration` e o
  spring do overlay; reduced motion respeitado.

### fase 3: proponentes

- trait `Proposer`: recebe intenção (texto), `Capabilities`, o
  `Snapshot` atual e a superfície atual (se houver); devolve IR.
- proponente determinístico por regras, para teste e para demo sem rede.
- proponente LLM atrás do mesmo trait: o prompt leva o registro de
  capacidades e o schema; a resposta passa pelo verifier; os erros do
  verifier voltam para o modelo num número limitado de tentativas; IR
  inválido nunca chega ao runtime. o provedor (modelo local ou API) é
  decisão do brenner nesta task, registrada no ADR.
- o modelo propõe superfície; ação continua indo pelo dispatcher com o
  ator certo (humano ou agente), com precondição e receipt.

### fase 4: a demo do editor de vídeo

- capacidades: `detect_silence` (ffmpeg `silencedetect` real sobre um
  arquivo), `preview_segment`, `change_threshold`, `apply_cut`,
  `reject_cut`.
- "corta os silêncios desse vídeo e depois quero revisar os cortes" ->
  superfície de revisão (vídeo, contagem, previews, slider de intensidade,
  aplicar todos, revisar individualmente).
- "quero comparar a versão original com os cortes" -> a superfície se
  transforma em comparação lado a lado com as diferenças na timeline,
  mantendo identidade do que continua.
- terminada a comparação, a composição some.
- lugar: página do grupo Apps Demo do showcase primeiro; app próprio
  depois, se crescer.

### padrão de app

ADR atualizando `official-app-pattern`: superfície materializada a partir
de IR vira o segundo padrão suportado, ao lado de estado em structs com
widgets retained, com as regras de quando usar cada um.

## fora de escopo

transformações adaptativas sobre telas fixas e capacidades compostas
(70); importers (71).

## dependências

fase 0: nenhuma. fases 1 a 4: 68, 59 e 58 integradas.

## superfície sob responsabilidade

o host de superfícies (lugar no ADR), a página demo no showcase
(`crates/showcase/src/view/apps/` com arquivo novo), o proponente,
`refs/` para a fase 0, propostas de ADR
`docs/ADR/materialized-surfaces-from-verified-ir.md` e a atualização de
`official-app-pattern`.

## interfaces que consome

`VerifiedIr`, `Capabilities` (68), `Dispatcher`, `Request` (58), `SemId`
opcional e helpers dos widgets, `Compositor::bind` (59), `LayoutEngine`,
`Theme`.

## interfaces que pode alterar

as próprias. mudança em widget de comps só com aprovação.

## critérios de aceitação

- teste: materializar a fixture da revisão de silêncios produz um snapshot
  com os mesmos `SemId`, roles e ações do IR.
- teste: transformar revisão em comparação preserva a instância e o estado
  de todo `SemId` que continua.
- teste: IR inválido do proponente nunca materializa; os erros voltam.
- ação disparada numa superfície gerada tem receipt com o ator certo.
- medir o tempo de intenção até primeiro frame e de IR verificado até
  primeiro frame; registrar.
- demo gravada (task-63 se integrada) e roteiro no handoff.
- checkpoint b: a superfície gerada comparada com uma tela fixa bem
  desenhada para a mesma tarefa, com o brenner, resultado escrito.
- `./script/gate` verde.

## testes esperados

materialização, reconciliação por `SemId`, dispensa, proponente
determinístico, loop de verificação do proponente LLM com um cliente falso,
viewports estreito e largo da demo.

## riscos e perguntas abertas

- latência de modelo: a superfície pode chegar tarde para a tarefa; a
  medição decide se o proponente precisa de cache, de modelo local ou de
  proposta parcial.
- usabilidade: a hipótese de que superfície gerada é melhor pode falhar.
  é para isso que existem a fase 0 e o checkpoint b.

## commits esperados

1. `task-69: spike da revisão de silêncios em refs` (relatório; o código
   fica em refs/)
2. `task-69: host de superfícies e materialização a partir de VerifiedIr`
3. `task-69: reconciliação por SemId e transições`
4. `task-69: proponente determinístico`
5. `task-69: proponente LLM com loop de verificação`
6. `task-69: capacidades do editor de vídeo com silencedetect`
7. `task-69: demo de revisão e comparação no showcase`

## pare e reporte se

- o layout do IR não mapear no `LayoutEngine` sem motor novo;
- a reconciliação exigir estado de widget que o comps não expõe;
- a escolha de provedor de LLM (sempre decisão do brenner).

## handoff (task-69 fase 1)

status: parada antes de implementar. a sonda está completa e nenhuma das
condições dela disparou; a leitura dos arquivos do showcase disparou a
condição do `dispatch.rs` congelado (item 5 do briefing). nada do crate
`surface` foi escrito.

### sonda (antes do primeiro tipo, 2026-09-24)

lida no worktree `task/69-surfaces` sobre `main` @ 1493c6b. as três
fixtures da task-68 usam dez componentes: o container (nó sem
`component`), `card`, `text`, `list` com `repeat`, `badge`, `button`,
`separator`, `text_field`, `chip` e `stat`.

(a) o widget de cada componente, como ele recebe o `SemId` e o que ele
precisa de estado de fora. a projeção sai do IR (decisão do
orquestrador), então nenhum `project` do comps é chamado; o `.sem(id)`
serve só para o widget ligar o rect que desenha e testa.

| componente | fixture | widget | `SemId` | estado de fora |
|---|---|---|---|---|
| container | as três | nenhum: é caixa de layout; o fundo (`background`, `shape`) é `comps::recipe::rounded_rect`, a elevação é `comps::recipe::shadow_stack` | o host liga com `Compositor::bind` | nenhum |
| `card` | todo | `comps::content::Panel`, a casca do card (`glass_surface` no raio `theme.shape.card`, `shadow_stack` pela elevação). o `comps::content::Card` é o card de prévia (`CardVariant`: stat, profile, media, list, chart, cta), sem filhos; a casca dele é a do `Panel` | o host liga (`Panel` não tem `SemId`) | nenhum |
| `text` | review | nenhum widget: `SceneNode::Text` com `TextNodeKey::from_style`, a mesma `TextStyle` na medida (`TextMeasureSpec` do layout) e no desenho, como os widgets do comps desenham texto. o `engine::builder::text` é do caminho declarativo | o host liga | o texto: `bind.state.text` ou o label |
| `list` com `repeat` | review | `comps::content::VirtualList` existe, mas o `.sem(id)` dele liga as linhas como `<id>[<índice>]` e a projeção dele nomeia por índice, e a identidade do IR é pelo item (`review.file[src/lexer.rs]`). ele pede altura fixa de linha e desenha cada linha por closure | o host não passa `.sem`; liga a lista e cada instância pelo id do IR | os itens: o valor vivo do `List`/`Set` de `repeat.each` (`changes:files`); com o `VirtualList`, também a contagem, a altura da linha e o viewport |
| `badge` | review | `comps::action::Badge::tag` | o host liga (`Badge` não tem `SemId`) | o texto: `bind.state.text` (`changes.file[<item>]:status`) |
| `button` | as três | `comps::action::Button` | `.sem(id)`, liga no render | label do IR; `enabled` da ação ligada no snapshot vivo (o `todo.input` `submit` vem `enabled: false` com o campo vazio); `busy` quando ligado (nenhuma fixture liga) |
| `separator` | review | `comps::content::Separator::Horizontal` | nenhum: sem role, fora da projeção e do bind | nenhum |
| `text_field` | as três | `comps::form::TextField` | `.sem(id)`, liga no render | o texto: `bind.state.text`; o placeholder é o label; `enabled` do `set_value` ligado. foco, caret e hover ficam no widget retido |
| `chip` | todo | `comps::action::Chip`, `interactive(true)` quando liga `activate`, `selectable(true)` quando liga `toggle` ou mostra `selected` | `.sem(id)`, liga no render (camada padrão) | label do IR; `selected` quando ligado (nenhum chip da fixture liga: o filtro do todo é uma capacidade por aba, sem um bool por chip) |
| `stat` | todo | `comps::content::Stat` | o host liga (`Stat` não tem `SemId`) | o valor: `bind.state.value` (`todo.counter:active`, int), formatado; label do IR |

nenhum componente das fixtures pede mudança de widget no comps. o prop
que o widget não aceita fica no padrão do widget:

- `typography` num `text_field`: o `TextField` mede, testa e desenha com
  `theme.typography.body()` fixo (`TextField::text_style`), sem API para
  outro estilo. o `ide-commit-form` pede `base_2r` e sai `body`;
- `color` e `background` num controle (botão, chip, campo): quem decide é
  a receita do widget (`theme.button`, `theme.glass`). só o `text` toma
  `color` e só o container toma `background`;
- o `card` toma `elevation` e `padding`; o raio é sempre
  `theme.shape.card`, o do `Panel`.

(b) cada `Layout` do IR no `LayoutEngine` de hoje
(`crates/engine/src/layout`: taffy atrás de `LayoutStyle`, sempre
`Display::Flex`, sem `position` e sem grid; folha de texto medida pelo
`TextMeasurer`):

| IR | `LayoutStyle` | mapeia? |
|---|---|---|
| `stack { axis, gap, align }` | `direction` Row ou Column; `gap` = `theme.spacing.<gap>` (0 sem gap); `align` Start, Center, End ou Stretch (sem align, o padrão `Stretch`) | direto |
| nó sem layout | coluna sem gap, como o IR define | direto |
| `grid { min_col, gap }` | linha com `flex_wrap` e `gap`; cada filho com `flex_basis` e `min_width` = `theme.size.<min_col>` e `flex_grow` 1 | sem motor novo. a diferença do grid de CSS: a última linha incompleta estica os filhos dela. colunas exatas pedem duas passadas (a largura do grid, depois `cols = floor((w + gap) / (min + gap))` e a base `(w - gap * (cols - 1)) / cols` com grow 0) |
| `split { axis, ratio }` | `direction` pelo eixo; os dois filhos com `flex_basis` 0 e `flex_grow` `ratio` e `100 - ratio` | direto |
| `overlay` | o `LayoutStyle` não tem posição absoluta. o container fica no fluxo do pai com a primeira camada; cada camada seguinte é uma passada própria do mesmo `LayoutEngine::compute` no rect do container, deslocada para a origem dele e desenhada depois | sem motor novo, uma passada por camada. nenhuma fixture usa |
| `size.min`, `size.max` | `min_width` e `max_width` = `theme.size.<token>`; `dropdown_max_h`, o único token de altura, vai em `min_height` e `max_height` (o eixo sai do nome do token, `_w` ou `_h`) | direto |
| `size.flex` | `flex_grow` = o peso (1 a 12) | direto |
| `props.padding` | `padding` nos quatro lados = `theme.spacing.<token>` | direto |
| folha | o tamanho pelo `preferred_size(theme)` do widget (`Button`, `Chip`, `Badge`, `Stat`); `TextField::height(theme)` na altura e a largura pelo pai; `text` por `TextMeasureSpec`; `separator` com `Separator::thickness(theme)` | direto |
| `repeat` | o molde vira uma subárvore por item antes do layout | direto |

nada pede motor de layout novo; a condição (b) não disparou.

(c) os tokens do IR no `Theme`, campo a campo:

| token do IR | no `Theme` | casamento |
|---|---|---|
| `SpacingToken`: xs, sm, md, lg, xl, xxl | `theme.spacing` (`SpacingScale`), os seis campos de mesmo nome | 6 de 6 |
| `TypographyRole` | `theme.typography.<nome>()` (`TypographyScale`): h4, headline, title, base_r, base_m, base_2r, base_2m, base_2sm, base_2b, caption_r, caption_sm, small_r, small_sm, body, body_2r, body_2m, body_2b, hairline, mono, readout | 20 de 20, um método por papel |
| `ColorRole` | `theme.colors` (`ColorTokens`): bg, surface, bg_panel, bg_hover, text, text_dim, text_mid, accent, accent_dim, success, danger, warning, info, divider, border_active | 15 de 15 |
| `Elevation`: base, raised, floating, overlay, hint | `engine::theme::Elevation`; a pilha é `theme.shadows.stack(e)`, desenhada por `comps::recipe::shadow_stack` | 5 de 5 |
| `ControlSize`: xs, sm, md, lg, xl | `engine::theme::ControlSize`; a altura é `theme.control.height(cs)` (36, 40, 44, 48, 52 no hoff core) | 5 de 5 no enum. o `Button` não lê `ControlSize`: a escada dele é `ButtonSize` em `theme.button.sizes` (28, 36, 40, 44). o `control_size` de um botão vira o maior `ButtonSize` que não passa da altura do token: xs `Sm`, sm `Md`, md `Lg`, lg e xl `Lg`; o `ButtonSize::Xs` (28) fica fora do alcance do IR, e os botões `xs` do review saem com 36. o `TextField` é sempre um `Md` e o `Chip` tem altura própria: nos dois o token não se aplica |
| `ShapeRole` | `theme.shape` (`ShapeTokens`): pill, dropdown, tabs, card, block, item, nav, cluster, tooltip, micro | 10 de 10 |
| `SizeToken` | `theme.size` (`SizeTokens`): menu_w, modal_max_w, sidebar_w, sidebar_rail_w, card_w, card_min_w, readable_max_w, field_min_w, toast_w, tooltip_max_w, dropdown_max_h | 11 de 11 |

o `match` exaustivo de cada enum do `uir` no campo do theme fica no crate
`surface`: um token novo no `uir` sem casa no theme quebra a compilação
ali.

### o plano do host (sem código ainda)

- `Surface::materialize(VerifiedIr)` é o único construtor; nenhuma API
  do crate recebe `Ir`.
- `sync(&Snapshot)`, quando a revisão do app muda (não a cada frame):
  expande cada repeat pelo valor vivo (um item repetido numa `List` fica
  só na primeira ocorrência), cria o widget de cada `SemId` novo, mantém
  o que continua (retido, com hover, foco e caret) e solta o que saiu; lê
  o estado ligado para o widget e guarda o snapshot mostrado.
- `render(c, rect, theme)`: layout pelo `LayoutEngine` quando o rect ou a
  expansão mudam (em cache no resto), tokens contra o theme recebido,
  widgets desenhados, todo nó projetado ligado. a superfície só desenha
  quando a página desenha; a página pede frame quando a revisão do app
  muda ou um widget diz `changed`.
- `project(b, parent)`: cada nó com role (`role_of`), label (a instância
  nomeada pelo item), o estado ligado lido do snapshot mostrado e uma
  ação por evento ligado, com o nome do evento (`activate`, `set_value`,
  `submit`) e o parâmetro, os `reads` e o `enabled` da capacidade.
- eventos: o clique ou a tecla num widget vira um pedido da capacidade
  ligada (alvo e ação do IR, o item trocado no molde); a página monta
  `Request::observe` sobre o snapshot que a superfície mostrou, com o
  ator de quem agiu, e entrega ao dispatcher do app por um trait; a
  superfície nunca chama `execute`. uma tecla num campo da superfície é
  um `set_value` com o texto novo: o texto é do app, e a superfície não
  tem outro caminho até ele.
- o ato de um agente ou de um leitor de tela num nó da superfície é
  traduzido para a capacidade, com o mesmo ator, parâmetro e observação
  (os `reads` da ação do nó são os da capacidade).
- a medida: headless, debug, `Instant` em volta de materializar,
  sincronizar e desenhar o primeiro frame num `Compositor`.

### parada: a página não cabe sem o `dispatch.rs`

problema: a página do item 5 não se faz com os arquivos permitidos (um
arquivo novo em `view/apps/`, o `mod.rs` de lá e a linha de
`view/pages/table.rs`).

- uma página do showcase é uma linha de `PAGES` com um `Pane`
  (`crates/showcase/src/view/pages/mod.rs:50`), o estado dela em `Panes`
  (`view/panes.rs`) e as rotas de `view/dispatch.rs`, que casa `Pane` de
  forma exaustiva em `page_content_height`, `page_event` e
  `page_render`: um `Pane` novo não compila sem um braço em cada um;
- a projeção para o agent port e para `--render --semantics` passa por
  `ShowcaseView::semantics`, `act` e `assist_focus`
  (`dispatch.rs:242-289`), que hoje só atendem `Pane::Todo`; o texto
  digitado num campo passa por `view/input.rs` (`handle_text`,
  `handle_edit_key`, `handle_enter`, `handle_escape`), por pane também;
- o `dispatch.rs` tem 775 linhas de produção para o teto 776 do `FROZEN`
  (`crates/engine/tests/file_hygiene_guard.rs:33`); os braços somam umas
  vinte;
- uma linha em `table.rs` com o `Pane::Todo` (como alias) mostra o todo,
  não a superfície, e o teste `every_pane_has_exactly_one_page_of_its_own`
  recusa uma segunda página própria do mesmo pane.

opções:

1. um `Pane::Surface` novo. a página (`view/apps/surface.rs`) desenha o
   todo fixo (o `TodoPage` de `panes.todo`, emprestado a cada chamada) e a
   superfície lado a lado, e despacha pelo dispatcher do todo. fora da
   lista: `view/pages/mod.rs` (a variante), `view/panes.rs` (o campo),
   `view/input.rs` (quatro braços), `view/dispatch.rs` (os braços, depois
   de levar o módulo `agent`, com os testes dele, para um arquivo
   próprio, `view/agent.rs`, sem mudar código; o `dispatch.rs` cai para
   umas 330 linhas) e `crates/engine/tests/file_hygiene_guard.rs` (a
   linha do `dispatch.rs` sai do `FROZEN`, como o guard pede quando uma
   divisão traz o arquivo para baixo do limite). impacto: cinco arquivos
   fora da lista, um deles o congelado; a página é a do briefing, se
   projeta para o agent port como as outras, e a dívida do `dispatch.rs`
   sai do `FROZEN`.
2. a superfície dentro da página Todo (`view/apps/app.rs` e `app/**`),
   lado a lado com o todo fixo: nenhuma rota nova. impacto: muda a página
   que é o gabarito do padrão oficial; a projeção do todo passa a ter os
   nós da superfície, o golden `crates/uir/fixtures/showcase-todo.snapshot.json`
   muda (proibido) e os ids `todo_panel.*` viram nós do app no registro
   de que a fixture verifica. pela regra 2 do verifier, um nó com o id de
   um nó do app só manda as ações desse nó, então a fixture deixa de
   verificar contra a página que a mostra. não recomendo.
3. nenhuma página nesta fase: o crate, a materialização, a projeção, os
   eventos e a medida como o briefing pede, e a prova sobre o todo em
   `crates/showcase/tests` pela API pública (`stage_page`,
   `ShowcaseView::semantics` e `act`), com o PNG da superfície saindo do
   renderer headless num teste, não de `showcase --render`. impacto:
   nenhum arquivo fora da lista; "render headless da página nova" e "a
   página se projeta para o agent port" ficam para quando a rota for
   autorizada.

recomendação: 1. é a página que o briefing descreve, com o contrato das
outras, e a divisão paga uma dívida que o guard já registra. se mexer em
cinco arquivos fora da lista pesar agora, 3, com a página depois. os
itens 1 a 4, 6 e 7 não dependem da escolha.

decisão necessária: qual opção; na 1, a autorização para os cinco
arquivos e para mover o módulo `agent` do `dispatch.rs`.

### o item do todo (não bloqueia; decidir junto)

a missão diz "marcar um item na superfície gerada muda o todo fixo". a
fixture `showcase-todo` não tem item: o todo não projeta uma `List` ou
`Set` dos itens (o P2 da task-68), então nenhuma superfície repete sobre
eles, e a fixture liga o campo, o add, o filtro em chips e os contadores.
com ela, a superfície muda o todo fixo escrevendo e adicionando um item
(`todo.input` `set_value` e `submit`) e trocando o filtro
(`todo.filter[<filtro>]` `select`).

opções: (i) a fixture como está, e o teste do aceite adiciona e filtra;
(ii) a página usa um IR próprio que estende a fixture com um checkbox por
item pelo id fixo (`todo.item[1]` `toggle`, `todo.item[1]:completed`):
verifica contra o registro do instante, mas não mostra item novo e fica
velho quando um item some (o dispatcher recusa); (iii) o todo projeta os
itens como `List` e a fixture ganha o repeat: muda
`view/apps/app/semantics.rs` e os dois arquivos do todo em
`crates/uir/fixtures`, proibidos. recomendação: (i) agora, (iii) numa
task própria ou na fase 2, que transforma por `SemId` e vai querer uma
lista viva.

### commits

- o commit desta sonda e desta parada (o sha está na mensagem final ao
  orquestrador).
