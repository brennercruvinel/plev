---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-09-24
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

status: implementado, testado, rebaseado na `main` @ 86f8b4f, com o
`./script/gate` verde (perna urnaui incluída). a sonda e a parada abaixo
ficaram como foram escritas, antes do código; a resolução do orquestrador
e o handoff da fase vêm depois delas.

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

### commits da sonda

- fa5b332 (antes do rebase, f912bcf sobre 1493c6b e 861fd9c sobre
  51394a9): a sonda e a parada.

### resolução da parada (orquestrador, 2026-09-24)

opção 1 e fixture (i). liberados `view/pages/mod.rs`, `view/panes.rs`,
`view/input.rs` e `view/dispatch.rs`, e no
`crates/engine/tests/file_hygiene_guard.rs` só tirar a linha do
`dispatch.rs` do `FROZEN` quando ele caísse abaixo de 639. primeiro mover
o módulo `agent` sem mudar código, num commit só; depois o `Pane::Surface`.
a página se projeta para o agent port como as outras, provado por um
teste da porta sem janela. a (iii) vira trabalho separado (P2) e a (ii)
não entra. aceitos sem mais nada: o `card` como `Panel`, o `grid` com
`flex_wrap` (a diferença do CSS no ADR), o `overlay` com uma passada por
camada, o botão pela escada `ButtonSize` (aproximado, no ADR). os dois
props que não chegam ao widget vão para o ADR como limite conhecido.

### handoff da fase 1 (2026-09-24)

goal: task-69 fase 1, uma superfície materializada de IR verificado.

status: implementado. `cargo test -p surface -p showcase -p uir` verdes,
`./script/gate` verde com a perna urnaui, sobre a `main` @ 86f8b4f.

#### commits

branch `task/69-surfaces`, rebaseada sobre `main` @ 86f8b4f (que traz a
task-64 fase c); todos assinados:

- fa5b332 task-69: a sonda no arquivo da task, e a parada na página
- fe0855d task-69: the agent module of the showcase dispatch moves to its own file
- 332fe62 task-69: the surface crate and the materialization from VerifiedIr
- 137b91b task-69: the projection and the events of a surface
- d7fa6d4 task-69: the Surface page of the showcase over the todo's capabilities
- e o commit dos ADRs e deste handoff

(sobre 51394a9, antes do rebase, eram 861fd9c, 8f6066b, d9090a0, 7a66429
e c17d2ce. o rebase não teve conflito; o `Cargo.lock` conferido com
`--locked`.)

#### o módulo agent do dispatch

`view/dispatch.rs` tinha 775 linhas de produção para o teto 776. o módulo
`agent` (os ganchos do agent port e os testes dele) foi para
`view/dispatch/agent.rs`, o mesmo caminho (`view::dispatch::agent`), sem
mudar código: saiu só a indentação do invólucro, e o rustfmt juntou uma
linha que a indentação quebrava (conferido por diff sem espaços). o
`dispatch.rs` caiu para 311 linhas e saiu do `FROZEN` no mesmo commit.
testes de agente que rodei sem mudança nenhuma, com `cargo test -p
showcase --lib dispatch::agent`: `an_agent_observes_and_operates_the_todo_page`
e `behind_a_modal_the_agent_is_refused_as_a_click_is` passam;
`agent_port_cost` e `agent_port_new_row` seguem ignorados (são medições).
com a página, o `dispatch.rs` tem 338 linhas.

#### como o host materializa

crate `crates/surface` (depende de `uir`, `sem`, `engine`, `comps`).

- `Surface::materialize(VerifiedIr, &Snapshot)` é a única entrada; nada
  no crate recebe `Ir`. um doctest `compile_fail` mostra que um `Ir` não
  compila ali, e o teste `an_ir_the_verifier_refuses_has_nothing_to_materialize`
  documenta que o IR recusado não dá valor nenhum para materializar.
- o IR se desenrola sobre o estado vivo: cada repeat uma vez por item do
  `List`/`Set` de `repeat.each`, o item no lugar de `[{item}]` em todo id
  e em toda ligação do molde (`review.file[{item}].stage` sobre
  `src/lib.rs` vira `review.file[src/lib.rs].stage`). item repetido numa
  `List` entra uma vez, o primeiro; um valor que sumiu ou mudou de tipo
  não dá instância nenhuma.
- um widget retido do comps por nó, indexado pelo `SemId`. `sync` lê as
  ligações de novo só quando a revisão do app muda: nó que continua
  mantém o widget (hover, foco, caret), nó novo ganha um, nó que saiu
  solta o seu. o snapshot lido é o que a superfície mostra.
- quando lê: a página projeta o app a cada frame que desenha, e a
  superfície relê as ligações só quando a revisão mudou; relê também logo
  depois de cada pedido que ela mesma ou um ato de fora fez.
- layout pelo `LayoutEngine`, tokens pelo `Theme`, como a sonda disse
  (b, c): stack flex, grid linha com `flex_wrap` a partir de `min_col`,
  split por pesos, overlay uma passada por camada; um `match` exaustivo
  por tipo de token; o `control_size` do botão pela escada `ButtonSize`.
  o layout é recalculado a cada frame desenhado.
- render: cada nó no seu widget (tabela no ADR), cada nó projetado ligado
  ao rect que desenha e testa (`Button`, `TextField`, `Chip` e os outros
  controles com `.sem(id)` se ligam sozinhos; o host liga os demais); um
  dropdown aberto por último, por cima. a superfície só desenha quando a
  página desenha; parada, `wake` não pede frame (o caret de um campo com
  foco pede o próximo piscar).
- `SurfaceHost` guarda as superfícies abertas; recusa um id já aberto
  (transformar é a fase 2) e nós que tomariam o id de outra aberta.

#### como projeta

a projeção sai do IR, nunca do `project` dos widgets, no mesmo builder
da projeção do app (um snapshot, um contrato):

- a superfície é um `Region` com o título; cada nó com role de
  `uir::role_of`, label (instância pelo item; text e badge pelo que
  mostram), descrição e o estado ligado como o snapshot mostrado tem;
- cada evento ligado é uma ação do nó, com o nome do evento (`activate`,
  `set_value`, `submit`, `toggle`, `select`, `confirm`, `cancel`) e o
  parâmetro, a policy e o `enabled` da capacidade. os `reads` são os da
  capacidade, com o nome do estado da superfície quando um nó dela o
  mostra (o `activate` de `todo_panel.add` lê `todo_panel.input:text`,
  onde o `submit` lê `todo.input:text`), e o do app quando nenhum mostra;
- nó com id de nó do app (a tela fixa escrita como IR) é do app: o app o
  declara; separator fica de fora; dialog fechado entra, a subárvore não.

#### como despacha

- clique completado num widget, ou tecla num campo da superfície, é o
  evento do componente: `Request::observe` sobre o snapshot que a
  superfície mostrou, pela capacidade ligada, com `Actor::Human`,
  entregue ao dispatcher do app pelo trait `Dispatch` (o dispatcher e a
  projeção do app). a superfície nunca chama `execute`.
- cada edição de campo é o `set_value` dele com o texto novo (o texto é
  do app); Enter é o `submit`, quando ligado. caret e foco ficam no
  widget.
- `route`: o ato de um agente ou de um leitor de tela num nó da
  superfície vira a capacidade ligada, mesmo ator e parâmetro, e a
  observação volta a ter os nomes do estado do app. a página do showcase
  faz `todo.dispatch(host.route(req))`: o receipt é do dispatcher do todo.
- regra de observação: uma pessoa age sobre o que a superfície mostrou
  (clique sobre estado que outro ator mudou e a superfície ainda não
  desenhou volta `Stale` com o delta; teste
  `a_click_on_state_the_surface_has_not_shown_yet_is_stale`); um agente
  sobre o que lhe mostraram (ato cuja capacidade lê estado que a
  superfície não mostra, observado só no escopo da superfície, volta
  `not_observed`; teste da porta).

#### a página e o render headless

`crates/showcase/src/view/apps/surface_page.rs`, grupo Apps Demo,
página "Surface": a fixture `showcase-todo.ir.json` da task-68 verificada
contra o registro que o todo projeta e materializada ao lado do todo
escrito à mão, lado a lado quando largo, o todo em cima quando estreito,
cada metade com uma legenda. as duas metades operam o mesmo estado pelo
dispatcher do todo. o `Panes::new` monta o todo primeiro e a página sobre
ele.

render: `cargo run -p showcase -- --render Surface --out surface.png
--semantics surface.json` (core, 1200x800, escala 1): "wrote ...
surface.json (22 nodes, revision 6ad867dc925c2c1f, Surface / core at
1200x800 scale 1)". o PNG: à esquerda, sob "the todo page, written by
hand", o todo de sempre (campo, três itens, o terceiro riscado, rodapé
"2 active · 1 done" e as três pílulas); à direita, sob "the same
capabilities, materialized from showcase-todo.ir.json", o card elevado
da superfície: campo "new todo" com o botão "add" desabilitado (campo
vazio), os chips All, Active, Completed, e os stats 2 active e 1 done
empilhados (o grid não cabe duas colunas de `card_min_w` na metade). em
390x844, tema hoff, a superfície fica sob o todo.

a projeção (os 10 nós do todo, `todo` a `todo.counter`, como no golden
da task-68; os 12 da superfície colados abaixo, forma lean):

```json
[
  { "id": "todo_panel", "role": "region", "label": "quick todo" },
  { "id": "todo_panel.card", "role": "group", "label": "todo", "parent": "todo_panel" },
  { "id": "todo_panel.entry", "role": "group", "label": "", "parent": "todo_panel.card" },
  { "id": "todo_panel.input", "role": "text_input", "label": "new todo", "parent": "todo_panel.entry",
    "state": { "text": { "kind": "text", "value": "" } },
    "actions": [
      { "name": "set_value", "reads": ["todo_panel.input:text"], "param": { "kind": "text", "constrained": true } },
      { "name": "submit", "enabled": false, "reads": ["todo_panel.input:text"] }
    ],
    "constraints": { "editable": true }, "focusable": true },
  { "id": "todo_panel.add", "role": "button", "label": "add", "parent": "todo_panel.entry",
    "actions": [ { "name": "activate", "enabled": false, "reads": ["todo_panel.input:text"] } ],
    "enabled": false, "focusable": true },
  { "id": "todo_panel.filter", "role": "group", "label": "filter", "parent": "todo_panel.card" },
  { "id": "todo_panel.filter.all", "role": "button", "label": "All", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ], "focusable": true },
  { "id": "todo_panel.filter.active", "role": "button", "label": "Active", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ], "focusable": true },
  { "id": "todo_panel.filter.completed", "role": "button", "label": "Completed", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ], "focusable": true },
  { "id": "todo_panel.counts", "role": "group", "label": "", "parent": "todo_panel.card" },
  { "id": "todo_panel.active", "role": "label", "label": "active", "parent": "todo_panel.counts",
    "state": { "value": { "kind": "int", "value": 2 } } },
  { "id": "todo_panel.done", "role": "label", "label": "done", "parent": "todo_panel.counts",
    "state": { "value": { "kind": "int", "value": 1 } } }
]
```

#### a medição

`VerifiedIr` até o primeiro frame, headless, build debug (a do gate),
apple m4, rustc 1.98.1, com `cargo test -p surface --test measure --
--ignored --nocapture`. "cena" é materializar, sincronizar, o layout e a
cena montada num `Compositor`; "pixels" soma o desenho pelo caminho de
frame da janela num alvo fora da tela e a leitura de volta (que espera a
GPU). o contexto de GPU já existe, como num app rodando; o atlas de
glifos é compartilhado entre as rodadas e as fixtures. duas execuções:

| fixture | nós | primeiro frame: cena, pixels | melhor de 20: cena, pixels |
| --- | --- | --- | --- |
| ide-commit-form | 5 | 6,6 ms, 19 a 24 ms (o primeiro texto e as pipelines do processo) | 53 a 61 us, 3,1 a 3,8 ms |
| ide-review-commit | 18 | 0,38 a 0,44 ms, 6,8 a 7,4 ms | 0,16 ms, 3,4 a 3,7 ms |
| showcase-todo | 11 | 0,17 a 0,21 ms, 5,4 a 7,5 ms | 79 a 82 us, 3,2 a 3,8 ms |

`SURFACE_FRAMES=<dir>` escreve o primeiro frame de cada fixture em png.

#### contagem do gate

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 86f8b4f (`cargo test --workspace` com a `main` em detached HEAD neste worktree) | 2267 passam, 11 ignorados | 82 passam, 1 ignorado | 2349 |
| depois, esta branch (`./script/gate`, saída 0) | 2297 passam, 0 falhas, 12 ignorados | 82 passam, 0 falhas, 1 ignorado | 2379 |

30 testes novos: 23 no `surface` (8 unitários, 5 de materialização, 6
de eventos, 2 de render, 2 doctests, um deles `compile_fail`; a medição
é o ignorado novo) e 7 no showcase (6 da página e 1 da porta). a
contagem não cai. sobre 51394a9, antes do rebase, o gate também passou:
2256 antes, 2286 depois. clippy `-D warnings`, `cargo fmt --check` e o
check wasm do showcase com `-D warnings` verdes (dentro do gate).
`df -h /` antes do gate final: 17 GB livres.

#### testes

- `crates/surface/tests/materialize.rs`: cada fixture projeta os ids, os
  roles e as ações do seu IR, instâncias da repetição incluídas pelo
  valor vivo (o esperado sai de uma leitura do IR feita no teste), os
  reads como a capacidade ou como a superfície os mostra, e o `route` de
  cada ação para a capacidade com a observação renomeada; a ação lê o
  estado onde a superfície o mostra; o repeat segue a lista viva (um
  arquivo sai, outro repete); um nó mantém o widget enquanto o id fica;
  o IR recusado não materializa.
- `crates/surface/tests/events.rs`: clique é a capacidade ligada, pela
  pessoa, pelo dispatcher do app, com o receipt de lá; digitar muda o
  texto do app e a tecla seguinte observa o texto que a primeira fez;
  clique sobre estado ainda não mostrado é stale; o ato de um agente num
  nó da superfície vai para a capacidade; capacidade desabilitada deixa o
  controle desabilitado; superfície inerte não recebe nada.
- `crates/surface/tests/render.rs`: cada fixture desenha e liga cada nó
  com role, em 320 e 1200 de largura, em dois temas; parada não pede
  frame.
- `crates/showcase/src/view/apps/surface_page/tests.rs`: a fixture abre;
  o aceite (adicionar na superfície aparece no todo fixo, receipt do
  dispatcher do todo, a linha nova desenhada); o filtro da superfície
  filtra o todo fixo; o todo fixo move a superfície; um snapshot só;
  lado a lado e empilhado.
- `.../surface_page/tests/agent.rs`: um agente pela porta real, sem
  janela, observa a superfície, escreve e adiciona um todo só pelo
  escopo dela, é recusado (`not_observed`, nomeando
  `todo.filter:selected`) no filtro, filtra depois de observar a página;
  o todo fixo mostra tudo e os receipts são do agente.

#### linhas de ADR

- novo, proposto: `docs/ADR/materialized-surfaces-from-verified-ir.md`.
  linha do índice para o orquestrador:
  `| 2026-09-24 | [materialized-surfaces-from-verified-ir](materialized-surfaces-from-verified-ir.md) | proposed | crate surface (uir, sem, engine, comps): só VerifiedIr materializa; o IR se desenrola sobre o snapshot vivo (repeat por item, ids chaveados pelo item) num widget retido do comps por SemId; layout pelo LayoutEngine (grid como linha com flex_wrap, overlay uma passada por camada), tokens pelo Theme num match exaustivo (control_size do botão aproximado pela escada ButtonSize); projeção do IR no builder do app, reads com o nome do estado que a superfície mostra; cada evento é um Request pelo dispatcher do app, observado no snapshot mostrado; route leva o ato num nó da superfície à capacidade; a página Surface do showcase (task-69) |`
- atualizado: `docs/ADR/official-app-pattern.md` ganhou o adendo "the
  second pattern, a surface materialized from verified IR", com quando
  usar cada padrão.

#### decisões

- reads com o nome da superfície. contexto: um agente que observa só a
  superfície não viu o estado do app que a ação lê. decisão: a ação do nó
  lê o estado como a superfície o mostra quando algum nó dela o liga, e
  o `route` renomeia a observação de volta. razão: a superfície vira uma
  página que se observa sozinha, sem esconder que um ato depende de
  estado que ela não mostra (esse continua pedindo o estado do app).
- uma tecla é um `set_value`: o texto é do app, e o campo da superfície
  não tem outro caminho. custo: um receipt por tecla.
- a leitura do app acontece no frame e depois de cada pedido; o que a
  superfície mostra é o snapshot da última leitura.
- `select` e `tabs` não recebem `.sem(id)`: eles ligariam opções por
  índice (`<id>[<i>]`), ids que o IR não tem; o host liga o nó.
- a página abre a superfície na construção (`Panes::new` monta o todo
  antes), para que a altura (`content_height`, que recebe `&self`) já a
  tenha.

#### limitações

- a lista não virtualiza: todas as instâncias entram no layout (o
  `VirtualList` nomeia linhas por índice e pede altura fixa). uma lista
  longa custa em proporção.
- o layout é refeito a cada frame desenhado, sem cache.
- o dialog não é modal (sem scrim, sem prender o foco) e o dropdown do
  select é desenhado por último na camada padrão, não numa camada de
  overlay. não há travessia de foco por Tab dentro da superfície; só os
  campos de texto recebem o teclado.
- props que não chegam ao widget: `typography` num `text_field` (sai
  `body`), `color` e `background` em controles. o `icon_button` mostra um
  ícone só; `chart` lê números do texto da lista; `table` tem uma coluna;
  `image` é a casca com o label. tudo no ADR.
- entre um clique no todo fixo e o frame seguinte, a projeção da página
  traz os nós do app já novos e os da superfície ainda como mostrados:
  é a regra (a superfície projeta o que mostrou), e o frame seguinte
  alinha.
- o `control_size` de botão é aproximado; `ButtonSize::Xs` (28) fica fora
  do alcance do IR.

#### dependências pendentes e riscos de integração

- fora da minha lista de arquivos, para o orquestrador: a linha do
  índice de ADR (acima); o `docs/CHANGELOG.md`; o comentário de
  `crates/showcase/src/app/headless.rs` ("today only the todo page
  does"), que agora vale também para a página Surface; a entrada
  `[crates.showcase]` do `docs/arc/arc.toml`, que não cita a página
  Surface nem as dependências `surface` e `uir` (só a entrada do crate
  novo era permitida).
- a página inclui a fixture de `crates/uir/fixtures` por `include_str!`:
  se o golden do todo ou o IR mudarem e o IR deixar de verificar, a
  página mostra um estado vazio com o primeiro erro do verifier (e o
  teste `the_fixture_verifies_against_the_todo_and_opens` fica vermelho).
- `view/dispatch.rs` saiu do `FROZEN`; um próximo arm cabe (338 de 639).

#### pergunta para depois (não bloqueia)

o verifier da 68 aceita props que o widget não toma. recomendo recusar
na regra 3 (vocabulário), com a tabela de props por componente no `Spec`:
um modelo que escreve `typography` num `text_field` aprende no erro, em
vez de ver o prop sumir em silêncio. é mudança no `uir` e decisão do
orquestrador.

#### o que a fase 2 (transformar por `SemId`) precisa

- uma entrada de transformação: `SurfaceHost::transform(id, VerifiedIr)`
  (hoje `open` recusa um id aberto), que troca o IR e refaz a árvore
  reaproveitando os widgets por `SemId`, como o `sync` já faz. um `SemId`
  cujo componente mudou (botão para chip) precisa de widget novo: decidir
  se algum estado passa.
- geometria por `SemId`, não por índice: os rects do último frame estão
  num vetor pela posição na árvore; transições precisam do rect anterior
  de cada id (entrar, sair, mover) e de manter o widget e o rect de quem
  saiu enquanto some. `theme.duration`, o spring do overlay e reduced
  motion entram aí.
- o arraste de um slider que continua no outro painel: o estado de
  arraste já mora no widget retido; falta o evento seguir o rect novo.
- dispensar com transição: `SurfaceHost::close` existe, sem transição.
- a fase 4 precisa de `media` no vocabulário (fora do v1).

#### o que a (iii) precisa (P2, trabalho separado)

para uma superfície repetir sobre os itens do todo:

- o todo projetar a lista dos itens como estado, por exemplo `items` no
  nó `todo` (`todo:items`, `List` com os ids visíveis, na ordem), em
  `crates/showcase/src/view/apps/app/semantics.rs`;
- o texto de cada item como estado (`todo.item[<id>]:text`), porque o
  label de uma instância é o item (o id, "1") e o IR não interpola label:
  um `text` do molde liga o texto;
- gravar de novo `crates/uir/fixtures/showcase-todo.snapshot.json`
  (`UIR_BLESS=1 cargo test -p showcase --test uir_fixture`) e dar ao
  `showcase-todo.ir.json` uma lista com `repeat` sobre `todo:items`,
  molde `todo_panel.item[{item}]` com um checkbox ligado a
  `todo.item[{item}]:completed` e `toggle`;
- lembrar do limite da task-68: um molde só verifica enquanto o app
  projeta pelo menos um nó que casa; com um filtro que esconde todos, o
  IR não verifica.

## fechamento da fase 1 (orquestrador, 2026-09-24)

integrada na `main` em `22d59fb` (seis commits sobre `86f8b4f`). o ADR
materialized-surfaces-from-verified-ir foi aceito.

validação do orquestrador:
- diff lido; o módulo `agent` do showcase conferido como movimento puro
  (as linhas removidas e as adicionadas diferem só na quebra de uma
  linha pelo rustfmt e no `mod agent;`);
- gate verde no diretório da task: 2379 passaram, 0 falharam, 13
  ignorados (a medição nova ignorada), com a perna urnaui;
- render headless da página Surface: o todo à mão à esquerda, a
  superfície materializada à direita, com as contagens ligadas ao estado
  vivo (2 active, 1 done).

aplicado no fechamento: o comentário de `app/headless.rs`, a entrada do
showcase no `arc.toml` (a página e as dependências), o índice de ADR, o
CHANGELOG.

severidade do que ficou:
- P2: o todo projetar os itens como lista (a superfície não repete sobre
  eles); o verifier aceita props que o widget ignora (tipografia num
  `text_field`, cor em controles). decisão: o verifier deve recusar o que
  a materialização não honra, e isso vai para a fase 2 ou uma task
  pequena;
- P3: a lista não virtualiza; o dialog não é modal; não há Tab dentro da
  superfície; `icon_button` com um ícone só; `image` desenha só a casca.

próximo: fase 2 (transformar por `SemId`, com o que o handoff lista).

## fase 2: a superfície se transforma sem perder identidade

branch `task/69b-transform`, worktree próprio, sobre a `main` @ 69b65ed.

### sonda da fase 2 (antes do primeiro tipo, 2026-09-24)

na base não existe transformação: `SurfaceHost::open` recusa um id
aberto, então o único caminho de uma forma verificada para outra é
fechar a superfície e abrir a outra. o teste
`closing_and_reopening_makes_every_widget_new`
(`crates/surface/tests/transform.rs`) faz isso sobre o app de teste com o
golden do todo (`crates/uir/fixtures/showcase-todo.snapshot.json`):

- a forma compacta é a fixture da task-68 (`showcase-todo.ir.json`). a
  outra é nova, `crates/surface/tests/fixtures/showcase-todo-columns.ir.json`,
  verificada contra o mesmo golden: duas colunas, à esquerda o filtro e
  as contagens (`todo_panel.side`), à direita a entrada
  (`todo_panel.main`). os ids dos controles são os mesmos; o grid
  `todo_panel.counts` sai, `todo_panel.side` e `todo_panel.main` entram;
- uma pessoa clica no campo, digita "buy milk", volta o caret três
  posições e digita "X": o app tem "buy mXilk" (a tecla caiu onde o caret
  estava, e o caret mora no widget);
- `close`, depois `open` da forma de duas colunas, verificada contra o
  app como ele está agora. numa área de 900 de largura, o campo estava
  em x 60, 779 de largura; agora está em x 430, 409 de largura: o mesmo
  `SemId` em outro lugar.

o que volta e o que se perde:

- o texto volta. ele é do app (`todo.input:text`), e o widget novo o lê
  pela ligação no `sync` da materialização;
- o foco se perde: `focused()` é `None`, e a tecla seguinte não vai a
  campo nenhum;
- o caret se perde: focado de novo de fora (`Surface::focus`), a tecla cai
  no fim, "buy mXilkY", e não três posições para trás;
- a instância é outra: `close` soltou a `Surface` inteira, e `open`
  criou um widget novo para cada nó. hover, press e o arraste em curso
  iriam junto, pelo mesmo motivo.

onde o estado mora hoje, componente a componente do vocabulário. o que é
do app é relido pela ligação no `sync`. o que é do widget vive no
`Surface.widgets: BTreeMap<SemId, Widget>`. a geometria vive em
`Surface.rects: Vec<Rect>` e `Surface.drawn: Vec<bool>`, pela posição na
árvore, zerados quando a árvore muda.

| componente | do app (ligação) | do widget (retido por `SemId`) | o comps expõe |
|---|---|---|---|
| `text_field` | `text`, o `enabled` do `set_value` | foco, caret, seleção, piscar, hover | `TextField::input` é pub: `focused` e `placeholder` pub, `buffer.cursor()` e `selection()` só para ler; o piscar e o hover são privados |
| `slider` | `value`, faixa e passo do nó alvo, `enabled` | arraste, hover, foco | `is_dragging()`, `is_hovered()`, `is_focused()`, só para ler |
| `switch` | `on`, `enabled` | a mola do knob, hover, press, foco | `on` pub; o resto privado |
| `checkbox` | `checked`, `enabled` | hover, press, foco | `label` pub; o resto privado |
| `button`, `icon_button`, `chip` | label, `enabled`, `active`, `selected` | hover, press, foco (o `Press` de cada um) | `Press` privado |
| `button` que liga `busy` | `busy`, `enabled` | o `Press` do `SubmitButton` | privado |
| `select` | `options`, `selected`, `enabled` | aberto, opção destacada, scroll do painel, foco | `is_open()`; o resto privado |
| `tabs` | `options`, `selected` | hover, foco | `labels`, `active` pub |
| `table` | `rows` | o scroll das linhas | `columns` e `rows` pub |
| `dialog` | `open`, os labels e o `enabled` das ações | hover e press dos dois botões | privado |
| container, `card`, `list`, `text`, `stat`, `badge`, `separator`, `empty_state`, `progress`, `chart`, `image` | tudo | nada (`Widget::Static`) | não se aplica |

a condição de parada não disparou. um nó que continua com o mesmo `SemId`
e o mesmo widget não precisa passar estado de uma instância para outra:
a transformação mantém a instância, e o estado vai junto, exposto ou não.
os props que a forma nova pode mudar num widget mantido (o placeholder do
campo, o label do checkbox, o título da coluna da tabela, o label do
botão e do chip) são campos pub.

o estado só teria de atravessar instâncias quando o widget muda sob o
mesmo id. isso acontece com a troca de componente, que o briefing me
deixa decidir, e num caso dentro do mesmo componente: um `button` que
passa a ligar `busy` (ou deixa de ligar) é outro widget do comps,
`SubmitButton` em vez de `Button`. o `Press` dos dois (hover e press em
curso) é privado e não passa. registro como achado, sem parar. o que se
perde é transitório: o hover volta no movimento seguinte do ponteiro, e
um press em curso no instante da troca não completa. nenhum estado do app
e nenhum estado semântico se perde. se isso tiver de passar, é mudança no
comps (expor o `Press` dos dois) e pede aprovação.

outro achado, fora do comps: o engine não expõe reduced motion. não há
símbolo de preferência de movimento em `crates/engine` nem em
`crates/comps`, e só a task-75 (pendente) traz a preferência do sistema.
a fase 2 não pede mudança no engine para isso: a superfície recebe a
preferência de quem a tiver (`SurfaceHost::set_reduced_motion`), e hoje
ninguém a tem. a task-75 liga a fonte.

correção da sonda (commit 5): a fixture de duas colunas descrita acima
era a primeira versão, duas colunas flex. na metade da página Surface
(420 de largura em 1200) ela empurrava o botão add para fora do card, e
num celular ficaria pior. ela virou um grid sobre `field_min_w`: duas
colunas onde a largura comporta (`todo_panel.side` com o filtro, os chips
um sob o outro, e as contagens; depois `todo_panel.entry`, o campo sobre
o botão), uma coluna onde não. `todo_panel.main` saiu; `todo_panel.side`
continua sendo o nó que nasce e `todo_panel.counts` o que sai. o que a
sonda mostrou (o texto volta, o foco e o caret se perdem) vale igual com
a fixture nova, e o teste da sonda passa com ela.

### handoff da fase 2 (2026-09-24)

goal: task-69 fase 2, a superfície se transforma sem perder identidade.

status: implementado. `cargo test -p surface -p showcase` verdes,
`./script/gate` verde com a perna urnaui, sobre a `main` @ 69b65ed.

#### commits

branch `task/69b-transform`, todos assinados, na ordem do briefing:

- 56d8146 task-69b: a sonda da fase 2 no arquivo da task
- d9dc076 task-69b: the geometry of a surface by SemId, not by place in the tree
- 0215217 task-69b: transform an open surface and reconcile it by SemId
- ef2574a task-69b: the transitions of a change of form, reduced motion, a dismissal that fades
- 6a2cb3e task-69b: the Surface page switches its surface between two verified forms
- 3f510b3 task-69b: a form the Surface page cannot take leaves its surface as it was
- e o commit do adendo do ADR e deste handoff

#### a sonda

acima, antes do código, com a correção da fixture logo depois. em uma
linha: fechar e abrir era o único caminho, o texto voltava (é do app) e
o foco e o caret se perdiam (eram do widget). a condição de parada não
disparou; os dois achados (o `Press` privado entre `Button` e
`SubmitButton`, e o engine sem reduced motion) estão lá.

#### a reconciliação

`SurfaceHost::transform(VerifiedIr, &Snapshot)` transforma a superfície
aberta que o IR nomeia; por baixo, `Surface::transform`. as regras:

- o id da superfície vem do IR. sem superfície aberta com esse id,
  `SurfaceError::NotOpen`; com um nó que tomaria o id de outra superfície
  aberta, `SurfaceError::Collision`. nos dois casos a superfície fica com
  a forma que tinha. só `VerifiedIr` transforma, como só ele materializa;
- a chave é o `SemId` e o widget do comps que o nó pede (`Kind`: o
  componente, e num `button` se ele liga `busy`). mesmo id e mesmo
  widget: o widget fica, e com ele foco, caret, seleção, hover e o
  arraste em curso. o que a forma nova pode mudar nele (label,
  placeholder, título da coluna) é posto de novo no `sync`;
- mesmo id com outro widget (outro componente, ou o botão que passa a
  ligar `busy`): widget novo, e nada do widget velho passa. o que ele
  mostra é do app e é relido pela ligação. decidi assim porque estado de
  widget não tem tradução entre tipos (caret de campo num checkbox), e o
  que precisa sobreviver a uma troca de representação é do app. a imagem
  do widget velho sai enquanto a do novo entra;
- id novo ganha widget; id que sumiu solta o seu. um nó que volta antes
  de a imagem dele sumir é a mesma instância de novo (teste
  `a_node_back_before_its_picture_is_gone_is_the_same_instance`: o campo
  sai no meio da edição, volta, e o caret está onde estava);
- a geometria é por `SemId`: o rect em que cada nó foi desenhado fica sob
  o id dele. um nó que continua mantém o seu até o frame seguinte, então
  um clique nesse meio cai no que a pessoa viu; um nó que saiu perde o
  seu na hora. antes, uma árvore que mudava zerava tudo, e nada era
  atingido até o frame seguinte;
- um `sync` que muda a árvore (a lista de um repeat andou) passa pela
  mesma reconciliação e mostra as mesmas transições.

#### as transições e o custo em frames

a semântica não espera os frames. o nó que nasce entra na projeção na
hora, é ligado e atingido, e aparece com fade. o nó que sai deixa a
projeção na hora. um ato de agente nele é recusado como qualquer alvo
que não existe: o `route` o deixa passar como está, e o dispatcher do app
responde `NotFound`. a imagem dele (um ghost com o nó, o widget, o
snapshot que mostrava e o rect) some com fade onde estava, desenhada
por cima dos nós vivos, sem ligação e sem hit. o nó que mudou de lugar
desliza do rect desenhado até o rect do layout.

nenhum motor de animação novo:

- entrada e saída: a mola de progresso do overlay, `Spring<f32>` sobre
  `theme.motion`, como o `OverlayManager` a roda. vira opacidade: o nó
  desenha num compositor próprio, e cada scene node que empurrou vai para
  o frame com o alfa escalado. num path, os vértices, com o hash movido
  junto; imagem e backdrop blur não têm cor e aparecem inteiros até a
  metade;
- movimento: `Tween` do engine sobre `theme.duration.slow`
  (`EaseInOutCubic`), do rect desenhado ao rect do layout de cada frame
  (um resize no meio aterrissa);
- começam no primeiro frame que desenha a forma nova, com o theme e o
  layout desse frame;
- um fade termina a um degrau de 8 bits do fim (1/255). o limiar de
  repouso da mola do engine pedia metade dos frames a mais, todos com a
  mesma imagem: 56 contra 37 no mixer.

render on demand: `Surface::wake` e `SurfaceHost::wake` pedem um frame
quando uma mudança de forma espera ser desenhada, todo frame enquanto
uma transição roda, e nada depois. contado como a task-82 contou (dois
segundos de turnos de 60 Hz, headless, frame só no turno que pede):

| caso | turnos desenhados de 120 |
|---|---|
| mixer parado | 0 |
| mixer, mudança de forma (slider muda de painel, dica nasce, reset sai, switch vira checkbox) | 37, numa sequência só a partir da mudança, nenhum depois |
| a mesma mudança com reduced motion | 1 |

reduced motion: o engine não expõe preferência de movimento, e a do
sistema chega com a task-75. a superfície recebe a preferência de quem a
tiver (`SurfaceHost::set_reduced_motion`, `Surface::set_reduced_motion`).
com ela, toda mudança é instantânea, inclusive as que estão em curso, e
a dispensa solta tudo na hora. a página do showcase não tem fonte para
ela e não a liga.

dispensar: `SurfaceHost::close(&SemId) -> bool`. a superfície deixa a
projeção, o route e o hit na hora, a imagem dela some com fade onde
estava (`SurfaceHost::draw_closing`) e depois tudo é solto.

o arraste: um slider em arraste continua em arraste quando muda de
painel. no primeiro frame da forma nova ele ainda está onde estava, e o
ponteiro age pelo que a pessoa vê; enquanto desliza e depois de chegar,
age pelo rect novo; soltar termina o arraste (teste
`a_drag_goes_on_after_the_slider_changes_panel`, no mixer, porque
nenhuma fixture da task-68 tem slider).

#### a página

`crates/showcase/src/view/apps/surface_page.rs`: um controle segmentado
(`comps::nav::Tabs`, "compact" e "two columns") acima da metade gerada
troca a forma da superfície. as duas formas:

- compact: a fixture da task-68, `crates/uir/fixtures/showcase-todo.ir.json`;
- two columns: `crates/surface/tests/fixtures/showcase-todo-columns.ir.json`,
  fixture local do crate `surface`, que a página inclui por `include_str!`
  como já incluía a da task-68. fica ali porque `crates/uir/**` é
  proibido nesta fase, e os testes do `surface` a usam.

cada forma é verificada contra o que o todo projeta no momento do
clique (a projeção real). se não verificar ou não transformar, a
superfície fica como estava e o controle volta (teste
`a_form_that_does_not_transform_leaves_the_surface_as_it_was`). o texto
digitado, o caret e o filtro continuam (teste
`the_control_switches_the_form_and_the_typed_text_and_the_filter_go_on`).
o clique no controle tira o foco do campo, como qualquer clique fora dele,
mas o caret fica no widget. a legenda nomeia o IR da forma mostrada.

render headless, 1200x800, core, escala 1, pelo teste
`surface_page::tests::forms::the_page_draws_headless_in_both_forms`, que
com `SURFACE_PAGE_OUT=<dir>` escreve o png e a projeção de cada forma. a
forma compacta também sai de `cargo run -p showcase -- --render Surface`,
com a mesma revisão da fase 1 (6ad867dc925c2c1f, 22 nós):

- compact: à esquerda o todo escrito à mão; à direita, sob a legenda
  "materialized from showcase-todo.ir.json", o controle com "compact"
  ativo e o card da superfície (campo "new todo" e add desabilitado na
  mesma linha, os chips All, Active, Completed, e os stats 2 active e 1
  done empilhados);
- two columns: a legenda nomeia `showcase-todo-columns.ir.json`, "two
  columns" ativo, e o card em duas colunas: à esquerda os três chips um
  sob o outro e os stats; à direita o campo e o add embaixo dele. nada sai
  do card. num celular, o grid vira uma coluna (teste
  `the_two_column_form_fits_a_phone_width`).

a projeção da forma de duas colunas (os 10 nós do todo como na fase 1;
os 12 da superfície, forma lean). `todo_panel.side` entrou,
`todo_panel.counts` saiu, os controles e o estado são os mesmos:

```json
[
  { "id": "todo_panel", "role": "region", "label": "quick todo" },
  { "id": "todo_panel.card", "role": "group", "label": "todo", "parent": "todo_panel" },
  { "id": "todo_panel.side", "role": "group", "label": "filter and counts", "parent": "todo_panel.card" },
  { "id": "todo_panel.filter", "role": "group", "label": "filter", "parent": "todo_panel.side" },
  { "id": "todo_panel.filter.all", "role": "button", "label": "All", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ] },
  { "id": "todo_panel.filter.active", "role": "button", "label": "Active", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ] },
  { "id": "todo_panel.filter.completed", "role": "button", "label": "Completed", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ] },
  { "id": "todo_panel.active", "role": "label", "label": "active", "parent": "todo_panel.side",
    "state": { "value": { "kind": "int", "value": 2 } } },
  { "id": "todo_panel.done", "role": "label", "label": "done", "parent": "todo_panel.side",
    "state": { "value": { "kind": "int", "value": 1 } } },
  { "id": "todo_panel.entry", "role": "group", "label": "", "parent": "todo_panel.card" },
  { "id": "todo_panel.input", "role": "text_input", "label": "new todo", "parent": "todo_panel.entry",
    "state": { "text": { "kind": "text", "value": "" } },
    "actions": [
      { "name": "set_value", "reads": ["todo_panel.input:text"], "param": { "kind": "text", "constrained": true } },
      { "name": "submit", "enabled": false, "reads": ["todo_panel.input:text"] }
    ] },
  { "id": "todo_panel.add", "role": "button", "label": "add", "parent": "todo_panel.entry",
    "actions": [ { "name": "activate", "enabled": false, "reads": ["todo_panel.input:text"] } ],
    "enabled": false }
]
```

#### interfaces adicionadas e alteradas

no crate `surface` (todas próprias):

- nova: `SurfaceHost::transform(VerifiedIr, &Snapshot) -> Result<&mut Surface, SurfaceError>`;
- nova variante: `SurfaceError::NotOpen(SemId)`;
- alterada: `SurfaceHost::close(&SemId)` devolve `bool` (era
  `Option<Surface>`): a superfície não sai inteira, ela some com fade.
  nenhum consumidor chamava `close`;
- novas: `SurfaceHost::draw_closing(&mut Compositor, &Theme)`,
  `SurfaceHost::set_reduced_motion(bool)`, `SurfaceHost::wake(f32) -> Wake`,
  `Surface::transform(VerifiedIr, &Snapshot)`, `Surface::set_reduced_motion(bool)`;
- semântica estendida: `Surface::wake` passa a incluir as transições;
  depois de uma mudança de árvore, o hit usa o rect desenhado de cada id
  até o frame seguinte.

no showcase, a API pública da `SurfacePage` (a que `view/dispatch.rs` e
`view/input.rs` chamam) não mudou. `switch` e `FORMS` são privados da
página.

#### invariantes

- a projeção, o route e o hit seguem a forma atual. uma imagem que ainda
  some nunca é capacidade;
- um widget só é mantido para o mesmo `SemId` e o mesmo `Kind`;
- parada, a superfície não pede frame. uma transição pede até assentar
  e depois nada;
- com reduced motion, nenhuma transição existe.

#### testes

18 testes novos:

- `crates/surface/tests/transform.rs` (5): a sonda (fechar e abrir
  perde foco e caret); transformar mantém widget, foco e caret, projeta a
  forma nova na hora e volta; outro componente no mesmo id é widget novo
  sobre o mesmo estado (switch para checkbox); transformação recusada
  (`NotOpen`, `Collision`); o arraste continua depois de mudar de painel;
- `crates/surface/tests/motion.rs` (6): o que nasce entra na projeção na
  hora, o que sai deixa a projeção na hora e é desenhado até a transição
  acabar, sem ligação e sem hit, e o ato de agente nele é `NotFound`; a
  transição pede frames até assentar e depois nenhum (37 de 120);
  reduced motion faz a troca instantânea (1 de 120); reduced motion no
  meio da transição a termina; um nó que volta antes de a imagem sumir é
  a mesma instância; a dispensa some com fade e depois solta tudo (e na
  hora, com reduced motion);
- `crates/surface/tests/events.rs` (+1): antes do frame seguinte, o
  clique atinge o que a pessoa viu (falha na base);
- `crates/surface/src/widget.rs` (+1, unitário): o botão que liga `busy`
  é outro widget;
- `crates/showcase/src/view/apps/surface_page/tests.rs` (+4): as duas
  formas verificam contra o todo como uma superfície só (mesmo id, mesmos
  controles); o controle troca a forma, e o texto, o caret e o filtro
  continuam; a forma de duas colunas cabe num celular; uma forma que não
  transforma deixa a superfície como estava;
- `.../surface_page/tests/forms.rs` (+1): a página headless nas duas
  formas, cena e projeção (e pixels com GPU).

fixtures novas em `crates/surface/tests/fixtures/`:
`showcase-todo-columns.ir.json`, `mixer-left.ir.json`,
`mixer-right.ir.json`. o app do mixer (`support::mixer()`) é construído
no teste.

#### validação executada

- `cargo test -p surface`, `cargo test -p showcase --lib surface_page`;
- `cargo clippy -p surface -p showcase --all-targets -- -D warnings`;
- `RUSTFLAGS="-D warnings" cargo check --target wasm32-unknown-unknown -p showcase`;
- `./script/gate`, saída 0, com a perna urnaui (`df -h /` antes: 15 GB
  livres);
- `cargo run -p showcase -- --render Surface --out ... --semantics ...` e
  `SURFACE_PAGE_OUT=<dir> cargo test -p showcase --lib surface_page::tests::forms -- --nocapture`
  para os pngs e as projeções.

#### contagem do gate

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 69b65ed (`cargo test --workspace` neste worktree; urnaui como no fechamento da fase 1) | 2297 passam, 12 ignorados | 82 passam, 1 ignorado | 2379 |
| depois, esta branch @ 3f510b3 (`./script/gate`, saída 0) | 2315 passam, 0 falhas, 12 ignorados | 82 passam, 0 falhas, 1 ignorado | 2397 |

18 testes novos; a contagem não cai.

#### linha de ADR

o adendo datado "a surface changes form (task-69 phase 2, proposed)" em
`docs/ADR/materialized-surfaces-from-verified-ir.md`. linha do índice
para o orquestrador (o `docs/ADR/README.md` é proibido aqui):

`| 2026-09-24 | [materialized-surfaces-from-verified-ir](materialized-surfaces-from-verified-ir.md) | accepted, adendo da fase 2 proposto | fase 2: SurfaceHost::transform reconcilia por SemId e pelo widget do comps (o mesmo mantém foco, caret e arraste; outro é widget novo sobre o estado do app); geometria por SemId; a semântica muda na hora (o que sai é NotFound) e os frames depois: entrada e saída na mola do overlay, movimento num Tween de theme.duration.slow, fade até um degrau de 8 bits; render on demand (37 de 120 turnos no mixer); reduced motion recebido de quem o tem (o engine não expõe, task-75); close some com fade e solta; a página Surface alterna compacta e duas colunas (task-69) |`

#### decisões

- widget por `Kind`, não só por componente. contexto: o `button` com
  `busy` é `SubmitButton`, sem o `busy` é `Button`. decisão: a chave é o
  widget. razão: um widget só pode ser mantido se for o mesmo tipo.
  consequência: hover e press não passam nessa troca (`Press` privado no
  comps);
- nada passa entre widgets de tipos diferentes. razão: o que precisa
  sobreviver é do app, e a ligação o relê;
- as transições começam no primeiro frame da forma nova. razão: a mola
  e o tween precisam do theme, e o movimento precisa do rect de chegada,
  que só o layout desse frame conhece;
- o fade é feito por scene node, num compositor próprio do nó. razão: a
  superfície desenha na camada que recebe e não tem camada própria para
  usar a opacidade de camada;
- o fade termina a 1/255 do fim. razão: frames que não mudam pixel são o
  que o render on demand existe para cortar;
- reduced motion como entrada da superfície. razão: o engine não o
  expõe, e a mudança no engine é da task-75.

#### limitações

- o fade é por primitiva, não por grupo: as primitivas de um nó se
  misturam entre si enquanto somem;
- o ghost é desenhado sobre os nós vivos. um container que sai com fundo
  cobriria a forma nova enquanto some (nenhuma fixture tem);
- uma superfície aparece de uma vez ao abrir. não há entrada animada no
  `open`;
- o tempo por frame de uma transição não foi medido. o custo conhecido é
  desenhar duas vezes o nó que está em fade;
- o controle da página é da página, não uma capacidade: um agente não
  troca a forma pela porta;
- a página não tem fonte de reduced motion.

#### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima), o
  `docs/CHANGELOG.md` (a página Surface troca de forma) e, se quiser, a
  entrada do `surface` no `docs/arc/arc.toml`, que descreve a fase 1;
- a página inclui `crates/surface/tests/fixtures/showcase-todo-columns.ir.json`
  por `include_str!`. mudar essa fixture muda a página, e o teste
  `both_forms_verify_against_the_todo_as_one_surface` a verifica contra o
  todo real;
- o `Press` privado entre `Button` e `SubmitButton`, se for preciso que
  passe, é mudança no comps e pede aprovação.

#### o que a fase 3 (proponentes) precisa

- o proponente devolve IR. verificado, ele vai para
  `SurfaceHost::transform` quando a superfície está aberta (o mesmo id de
  superfície) e para `open` quando não está. a superfície atual que o
  `Proposer` recebe é `Surface::verified()` (o IR) com `Surface::shown()`
  (o snapshot que ela mostra);
- continuidade depende de ids estáveis. a reconciliação só casa por
  `SemId`, então o prompt do proponente LLM precisa pedir que as mesmas
  capacidades mantenham os ids da superfície atual. um id renomeado é um
  nó que sai e outro que nasce, e o estado do widget (caret, arraste) se
  perde. o proponente por regras pode garantir isso por construção;
- propostas seguidas durante uma transição já funcionam: o movimento
  recomeça do rect desenhado, e um nó que volta recupera a instância;
- a medida "intenção até o primeiro frame" soma o transform, que não foi
  medido à parte. na fase 1, a cena de uma superfície custava frações de
  milissegundo;
- a entrada da intenção na página (um campo de texto) substitui ou
  acompanha o controle segmentado, e pode ser uma capacidade projetada;
- continua aberta a pergunta da fase 1: o verifier aceita props que o
  widget ignora, e um modelo que escreve `typography` num `text_field` não
  aprende com o erro.

## fechamento da fase 2 (orquestrador, 2026-09-24)

integrada na `main` em `4593a4c` (sete commits, rebaseados sem
conflito). o adendo do ADR foi aceito.

validação do orquestrador:
- diff lido (geometria por `SemId`, `transform`, `motion.rs`, a página);
- gate verde no diretório da task: 2429 passaram, 0 falharam, 13
  ignorados, com a perna urnaui;
- o PNG da forma em duas colunas, conferido: o mesmo todo reorganizado
  (os filtros e as contagens à esquerda, o campo e o add à direita), o
  controle "compact | two columns" em cima.

severidade do que ficou:
- P2: o engine não expõe a preferência de reduced motion do sistema (a
  superfície recebe a flag de fora; a fonte do sistema vem com a
  task-75); um `button` que vira `SubmitButton` perde hover e press (o
  `Press` é privado no comps); o controle de forma da página não é
  capacidade projetada, então um agente não troca a forma pela porta;
- P3: fade por primitiva, não por grupo; a imagem que sai é desenhada por
  cima dos nós vivos; `open` sem entrada animada.

próximo: fase 3 (proponentes). o provedor do LLM é decisão do brenner.

## fase 3a (orquestrador, 2026-09-24)

a fase 3 se divide. a 3a faz tudo o que não depende do provedor do LLM:
- o trait `Proposer`;
- o proponente determinístico por regras sobre o registro de
  capacidades;
- o proponente LLM atrás de um trait de modelo, com o prompt (o registro,
  o schema, a superfície atual e os ids que devem continuar) e o laço de
  verificação com tentativas limitadas, provado com um modelo falso;
- a página Surface com um campo de intenção que leva a uma superfície
  proposta, verificada e aberta ou transformada.

a 3b liga um provedor real e espera a decisão do brenner (local ou API).

## fase 3a: proponentes sem provedor

branch `task/69c-proposers`, worktree próprio, sobre a `main` @ a678ef4.

### sonda da fase 3a (antes do primeiro tipo, 2026-09-24)

um teste descartável (`crates/uir/tests/probe_69c.rs`, rodado e depois
jogado fora) lê o golden da projeção real do todo
(`crates/uir/fixtures/showcase-todo.snapshot.json`, que o teste
`uir_fixture` do showcase mantém igual à página), monta
`Capabilities::from_snapshot` e escreve à mão o IR que um proponente por
regras escreveria para "só o que está ativo e o campo de adicionar". a
superfície atual é a da página Surface (a fixture da task-68, id
`todo_panel`), e os nós das capacidades que continuam guardam os ids
dela.

o registro, como o proponente o recebe (saída do teste, os itens 2 e 3
iguais ao 1):

```text
data   todo.input:text          text  label="new todo"
data   todo.filter:selected     text  label="filter"
data   todo.item[1]:completed   bool  label="Absorb the plev demos into the showcase"
data   todo.counter:active      int   label="2 active · 1 done"
data   todo.counter:done        int   label="2 active · 1 done"
action todo.input               set_value param=text
action todo.input               submit    param=none
action todo.filter[all]         select    param=none  label="All"
action todo.filter[active]      select    param=none  label="Active"
action todo.filter[completed]   select    param=none  label="Completed"
action todo.item[1]             toggle    param=none
action todo.item[1]             remove    param=none  destructive=true
```

o registro não carrega role nem parent. o `Snapshot`, que o proponente
também recebe, carrega: `todo` region, `todo.input` text_input,
`todo.filter` tab_list com três filhos `tab`, `todo.item[k]` list_item,
`todo.counter` status.

o IR escrito à mão verifica (`verify` com a `GenerationPolicy` padrão; a
ordem de foco é `todo_panel.input`, `todo_panel.add`,
`todo_panel.filter.active`):

```json
{ "plev_ir": 1, "surface": { "id": "todo_panel", "title": "quick todo", "root": {
  "sem": "todo_panel.card", "component": "card", "label": "todo",
  "props": { "padding": "lg", "elevation": "raised" },
  "layout": { "stack": { "axis": "vertical", "gap": "md" } },
  "children": [
    { "sem": "todo_panel.entry",
      "layout": { "stack": { "axis": "horizontal", "gap": "sm", "align": "center" } },
      "children": [
        { "sem": "todo_panel.input", "component": "text_field", "label": "new todo", "size": { "flex": 1 },
          "bind": { "state": { "text": "todo.input:text" },
                    "actions": { "set_value": { "target": "todo.input", "action": "set_value" },
                                 "submit": { "target": "todo.input", "action": "submit" } } } },
        { "sem": "todo_panel.add", "component": "button", "label": "add",
          "bind": { "actions": { "activate": { "target": "todo.input", "action": "submit" } } } } ] },
    { "sem": "todo_panel.filter", "label": "filter",
      "layout": { "stack": { "axis": "horizontal", "gap": "xs" } },
      "children": [
        { "sem": "todo_panel.filter.active", "component": "chip", "label": "Active",
          "bind": { "actions": { "activate": { "target": "todo.filter[active]", "action": "select" } } } } ] },
    { "sem": "todo_panel.active", "component": "stat", "label": "active",
      "bind": { "state": { "value": "todo.counter:active" } } } ] } } }
```

de onde vem cada ligação, e a regra que a acha sem nome do todo:

| nó | ligação | o fato | a regra |
|---|---|---|---|
| `todo_panel.input`, text_field | `text` de `todo.input:text`; `set_value` e `submit` para `todo.input` | um nó de role `text_input` com um estado de kind text, uma ação com `ParamSpec` text e outra sem parâmetro | "adicionar": o campo liga o estado de texto, a ação de parâmetro text e a sem parâmetro do mesmo nó; o label é o do nó |
| `todo_panel.add`, button | `activate` para `todo.input` `submit` | a ação sem parâmetro do mesmo nó | o botão liga a ação que o Enter do campo liga |
| `todo_panel.filter.active`, chip | `activate` para `todo.filter[active]` `select` | filho `tab` de um nó `tab_list`, uma ação só, sem parâmetro, label "Active" | "filtro": um chip por opção de uma escolha (tab_list ou select com filhos tab, option ou radio de uma ação sem parâmetro); o seletor deixa só a opção cujo label ou chave é a palavra |
| `todo_panel.active`, stat | `value` de `todo.counter:active` | um estado de kind int com a chave `active` | "contagem": um stat por estado int ou num; o label é a chave, porque o label do nó (`2 active · 1 done`) muda com o valor |

os containers (`todo_panel.card`, `todo_panel.entry`, `todo_panel.filter`)
guardam os ids da superfície atual porque os nós que eles seguram
continuam.

resposta: basta. cada ligação sai de role, `ValueKind`, `ParamSpec` e
parent, e nenhuma regra cita `todo`, `input`, `filter`, `counter`,
`active` ou `done`. a condição de parada da sonda não disparou. três
achados ficam como limite do proponente por regras:

1. o seletor é uma palavra da intenção comparada com as palavras do
   registro (chave de estado, label e chave de opção), sem caixa, sem
   acento e sem o s do plural. o registro do todo está em inglês. a saída
   do teste: `selector "ativo": stats [], options []` e `selector "active":
   stats ["todo.counter:active"], options ["todo.filter[active] select"]`.
   as palavras de regra (adicionar, filtro, contagem, compacto, duas
   colunas, só) o proponente entende nas duas línguas porque são dele; a
   palavra que escolhe o que mostrar é do app.
2. "o que está ativo" como os itens ativos não sai do registro. ele diz
   `completed: bool` por item, e "ativo quer dizer não concluído" é um
   sentido do todo, não um fato. o todo também não projeta a lista dos
   itens (o P2 da fase 1), então nenhuma superfície repete sobre eles. o
   que o registro dá para "active" é a contagem e a opção do filtro que o
   próprio app chama assim, e é isso que o IR liga.
3. o `Capabilities` sozinho não acha a escolha: role e parent (a tab_list
   e as tabs) só estão no snapshot. o trait recebe os dois, então não
   falta nada, mas a regra lê o snapshot além do registro.

### parada: o campo de intenção e o botão propor não chegam ao dispatcher da página

problema: o item 5 do briefing pede o campo e o botão como capacidades
projetadas, pelo mesmo dispatcher da página, e isso não se faz com os
arquivos permitidos (`surface_page.rs` e `surface_page/**`).

- o único dispatcher da página Surface é o do todo, `TodoPage.dispatcher`,
  campo privado de `crates/showcase/src/view/apps/app.rs`.
  `TodoPage::dispatch` (`app/semantics.rs:244`) o empresta com o próprio
  `TodoPage` como `Operable`, e o `Dispatcher` valida cada pedido contra a
  projeção desse `Operable`. um `set_value` no campo de intenção, ou um
  `propose`, volta `NotFound`: o todo não projeta esses nós;
- a página não alcança o campo. a privacidade é por módulo, e um
  `impl TodoPage` escrito em `surface_page.rs` também não vê `dispatcher`;
- um segundo `Dispatcher` na página não serve à porta. o
  `ShowcaseView::add_sink` (`view/dispatch.rs`) põe a sink da porta só no
  dispatcher do todo, e o servidor recusa o ato cujo id a sink não ouviu
  ("request N was dispatched, but the app's dispatcher reports no stage",
  `crates/wire/src/server.rs:516`). e os dois dispatchers numeram a partir
  de 1: a porta casa estágio por `RequestId`, então um id do outro
  dispatcher passaria por esse teste com o estágio errado.

opções:

1. um método em `crates/showcase/src/view/apps/app/semantics.rs` que
   empresta o dispatcher do todo para um `Operable` maior, o mesmo
   `mem::take` que `dispatch` já faz, umas dez linhas:
   `pub(crate) fn with_dispatcher<R>(&mut self, f: impl FnOnce(&mut Dispatcher, &mut TodoPage, Millis) -> R) -> R`.
   a página monta um `Operable` que segura o todo, o host e o campo:
   projeta o todo, a superfície e os dois nós da página
   (`surface_page.intent`, text_input com `set_value` de texto;
   `surface_page.propose`, button com `propose` lendo
   `surface_page.intent:text`), executa esses dois e passa o resto para
   o `Operable::execute` do todo. o receipt fica onde fica o de todo ato da
   página, a porta ouve, o ator é o de quem pediu. impacto: um arquivo
   fora da lista, nenhuma mudança na projeção do todo, no golden ou no
   caminho de hoje (o clique no todo e o evento da superfície seguem por
   `TodoPage::dispatch`).
2. um dispatcher próprio da página, com a porta ouvindo os dois: muda
   `view/dispatch.rs` (o `add_sink`) e o `wire` (uma sink por dispatcher e
   ids que não colidam). impacto: o protocolo da porta muda por causa de
   uma página de demo. não recomendo.
3. o campo e o botão como controles da página, não capacidades, como o
   controle de forma da fase 2: uma pessoa propõe, um agente não. nenhum
   arquivo fora da lista. impacto: o aceite "um agente propõe pela porta"
   não se cumpre, e o P2 da fase 2 (o controle da página não é
   capacidade) cresce.
4. o todo projetar a intenção e o `propose` como capacidades dele: muda
   `app/semantics.rs`, o golden `showcase-todo.snapshot.json` e o registro
   de que as fixtures verificam, e põe o proponente dentro do app de
   exemplo. não recomendo.

recomendação: 1. os itens 1 a 4, 6 e 7 do briefing não dependem da
escolha (o crate `propose`, o trait, o determinístico, o LLM com o modelo
falso, a medição, o ADR).

decisão necessária: qual opção; na 1, a autorização para o método em
`crates/showcase/src/view/apps/app/semantics.rs`.

### o seletor em português (não bloqueia; decidir junto)

opções: (i) o seletor casa com as palavras do registro (sem caixa, acento
e plural), e o que não casa é um erro claro que lista as palavras que o
registro tem; a intenção de exemplo da página usa a palavra do app ("só
active e o campo de adicionar"); (ii) um léxico português e inglês no
proponente (ativo, feito, concluído), que são as palavras do todo com
outro nome, template pela porta dos fundos; (iii) a tradução fica com o
proponente LLM. recomendação: (i) agora, e (iii) na 3b.

### resolução da parada (orquestrador, 2026-09-24)

opção 1: autorizado um método em
`crates/showcase/src/view/apps/app/semantics.rs`, só ele, que empresta o
dispatcher do todo ao `Operable` maior da página, pelo mesmo
`mem::take` de `TodoPage::dispatch`; nada mais daquele arquivo nem de
`app.rs` muda (nem a projeção do todo, nem o golden, nem o caminho de
hoje), com um teste de que a projeção e os receipts do todo não mudam
quando o `Operable` da página repassa a ele. o seletor: (i). a palavra
que escolhe casa só com as palavras do app, a que não casa é um erro que
lista as palavras do registro, sem léxico entre línguas no
determinístico; as palavras da gramática dele valem nas duas línguas;
traduzir a palavra da pessoa para a do app é do proponente LLM. nas
páginas e nos testes, intenções cuja palavra existe no app ("only what is
active and the add field"). o resto do briefing fica como estava.

### handoff da fase 3a (2026-09-24)

goal: task-69 fase 3a, proponentes sem provedor: uma intenção em texto
vira um IR proposto que só chega ao runtime verificado.

status: implementado. `cargo test -p propose -p showcase -p uir -p
surface` verdes, `./script/gate` verde com a perna urnaui, sobre a
`main` @ a678ef4.

#### commits

branch `task/69c-proposers`, todos assinados, na ordem do briefing:

- 47ff75d task-69c: a sonda da fase 3a no arquivo da task, e a parada na página
- 8a6123d task-69c: the Proposer trait and the deterministic proposer by rules
- 605e235 task-69c: the language-model proposer behind a Model trait, proven with a scripted model
- fb4ca07 task-69c: the Surface page proposes a surface from an intent, through the todo's dispatcher
- e o commit do ADR e deste handoff

#### a sonda

acima, antes do código ("sonda da fase 3a"), com a parada e a resolução.
em uma linha: o IR de "só o que está ativo e o campo de adicionar",
escrito à mão sobre o registro da projeção real do todo, verifica, e
cada ligação sai de role, `ValueKind`, `ParamSpec` e parent, sem nome do
todo; a condição de parada não disparou. os três limites dela ficaram no
código como estão lá: a palavra que escolhe é do app, "ativo como não
concluído" não é fato do registro, role e parent vêm do snapshot.

#### onde mora e por quê

crate novo, `crates/propose` (`uir`, `sem`, `serde_json`, `thiserror`;
nunca engine, comps ou rede). um proponente não é parte do IR: o `uir`
continua a língua e o verifier, compila para wasm e não faz I/O, e a fase
3b vai pôr um provedor atrás do `Model`, que não pode morar lá. as
entradas do `README.md` (a lista de crates e uma frase) e do
`docs/arc/arc.toml` (`[crates.propose]`) foram no commit do trait; o
`arc_sync_guard` passa.

a API:

- `Ask { intent, caps, app, current, policy }`: a intenção, o
  `Capabilities`, o `Snapshot` (roles, parents, labels, valores, que o
  registro não carrega), o `VerifiedIr` da superfície mostrada e a
  `GenerationPolicy` sob a qual a proposta será verificada. a policy não
  estava na lista do briefing; entrou porque o laço do LLM verifica dentro
  dele, e tem de verificar sob a mesma policy de quem chama;
- `trait Proposer { fn propose(&mut self, &Ask) -> Result<Ir, ProposeError> }`;
- `propose_verified(&mut proposer, &Ask) -> Result<VerifiedIr, ProposeError>`:
  a única saída. um IR que o verifier recusa sai como
  `ProposeError::Refused` com todos os erros, nunca como IR (teste
  `propose_verified_never_returns_what_the_verifier_refuses`);
- `ProposeError`: `NotUnderstood`, `NoMatch { words, app }`,
  `Nothing { part, missing }`, `Empty`, `Model`, `Exhausted { attempts, errors }`,
  `Refused`. o texto de cada um é o que a página mostra.

#### o vocabulário do proponente determinístico (`Rules`)

a gramática, nas duas línguas porque é dele (as palavras são dobradas:
sem caixa, sem acento, sem o s do plural):

| palavra | o que faz | a regra sobre o registro |
|---|---|---|
| add, adicionar, adiciona, new, novo, entry, entrada | a entrada | nó `text_input` com um estado de texto e uma ação de parâmetro text: um `text_field` ligado aos dois; a ação sem parâmetro do mesmo nó é o `submit` do campo e o `activate` de um botão; sem ela, um botão ao lado (mesmo parent) com uma ação só, sem parâmetro. o label do botão: o da superfície atual, o do botão ao lado, a palavra da intenção, ou o nome da ação |
| filter, filtro, filtrar | as escolhas | tabs, options ou radios cuja única ação não tem parâmetro: um chip por opção, sob o parent |
| counts, count, counter, stats, contagem, contador, number, número, total | os números | cada estado `int` ou `num`: um `stat`, com a chave como label (o label do nó, `2 active · 1 done`, muda com o valor) |
| compact, compacto, "one column", "uma coluna" | a forma compacta | uma coluna: entrada, escolhas, números |
| two columns, duas colunas, columns, colunas | duas colunas | grid sobre `field_min_w`: escolhas e números numa coluna, a entrada na outra |
| everything, tudo | tudo | todas as partes |
| only, just, só, apenas, artigos, "field", "campo" | nada | a gramática pula |

qualquer outra palavra tem de ser do app (label, nome ou chave de id,
chave de estado) e escolhe: a parte que a intenção nomeia fica só com o
que a palavra nomeia (tudo, se ela não nomeia nada da parte); a parte que
ela não nomeia mostra só o que a palavra nomeia. sem forma na intenção,
fica a forma da superfície atual (duas colunas quando a raiz é um grid).
uma intenção só de forma remonta o que a superfície mostra.

o que ele responde quando não entende, nunca um IR vazio:

- nenhuma palavra que as regras leem (`""`, "the only field"):
  `the rules do not understand "...": they read add, filter, counts (or stats), compact, two columns and everything (adicionar, filtro, contagem, compacto, duas colunas, tudo), and words the app uses`;
- uma palavra que não é da gramática nem do app ("só o que está ativo e
  o campo de adicionar", "make it purple"):
  `"ativo" is not a word of the app; its words are active, all, completed, done, filter, input, new, todo`;
- uma parte que o app não tem ("filter" no workbench do ide):
  `a choice (filter) asked for, and the app declares no choice (tabs, options or radios whose action takes no parameter)`;
- um app sem nada disso: `Empty`.

continuidade: sobre a superfície atual, o id da superfície fica (a página
transforma), e cada nó cuja capacidade continua fica com o id e o label
dela; um container fica com o id do container atual que segurava pelo
menos metade do que ele segura, de dentro para fora. o que é novo se
chama `<superfície>.<o nó do app que mostra>` e nunca toma um id que a
superfície atual usa para outra coisa. as regras só ligam o que o autor
pode pedir e nunca uma ação destrutiva.

a mesma regra lê o workbench do ide (teste `the_same_rules_read_the_ide`):
"add" liga `commit.message` e o botão "Commit" ao lado (o campo não tem
ação sem parâmetro), "counts" liga `diff:count`, "filter" é `Nothing`.

#### o laço do LLM (`Llm`)

- `trait Model { fn complete(&mut self, prompt: &str) -> Result<String, ModelError> }`,
  síncrono como o resto de uma proposta;
- `Llm::ATTEMPTS` = 3 (a primeira resposta e dois consertos; o verifier
  dá todos os erros de uma vez, cada um no seu ponteiro, então um
  conserto costuma bastar, e o terceiro limita o custo de um modelo que
  não converge). depois, `Exhausted` com os erros da última. modelo que
  falha para na hora (`Model`);
- a resposta é lida como o objeto dentro dela (do primeiro `{` ao último
  `}`, porque modelo embrulha em prosa e cerca), decodificada por
  `uir::parse` e verificada; recusada, o prompt seguinte é o primeiro mais
  a resposta e cada erro com ponteiro, razão e regra;
- o prompt leva: a intenção; as oito regras do verifier (um `match`
  exaustivo em `uir::Rule`, então uma regra nova tem de ser descrita) e o
  autor e os limites da policy; cada componente com role, como se nomeia,
  o estado que mostra e os eventos que manda (de `Component::ALL` e do
  `Spec`); o registro legível, nó a nó com role, label e parent, cada
  estado com kind e valor atual, cada ação com o que toma, destrutiva,
  confirmação, quem pode pedir, desabilitada agora; a superfície atual,
  o id a manter, cada nó ligado com o que liga (os ids que devem
  continuar) e o IR dela; e o JSON schema (`uir::schema()`). uns 18 KB
  sobre o todo;
- provado com modelos que respondem de um roteiro: um que erra na
  primeira e acerta na segunda (o segundo prompt tem a resposta e cada
  ponteiro e razão do verifier), um que erra sempre (três perguntas,
  nenhuma quarta, `Exhausted` com os erros da terceira), um que falha. o
  teste `the_prompt_carries_the_schema_the_registry_and_the_ids_to_keep`
  confere o schema inteiro, cada estado e ação do registro e cada id a
  manter.

um prompt de exemplo, sobre o todo com a superfície compacta, intenção
"only what is active and the add field" (cortado: os componentes, os
itens 2 e 3, o IR atual e o schema aparecem inteiros no prompt):

```text
you propose one surface of a plev app as a plev IR document: JSON that describes the surface by meaning, by binding the app's own state and actions to components of a closed vocabulary. the runtime verifies it before anything is shown; a document the verifier refuses comes back to you with every error. answer with the IR only: one JSON object, no prose.

## the intent

only what is active and the add field

## the verifier's rules

1. schema: the JSON has the shape of the schema below, no field it lacks, and `plev_ir` is 1.
2. identity: every `sem` id is valid (...) and unique on the surface, the surface's id included; `[{item}]` only inside a repeat's template, once per id; a node that takes the id of an app node sends only that node's actions.
(... 3 a 8 ...)

this proposal is verified under: author agent:unknown, at most 256 nodes, depth 16, 256 characters per title, label or description.

## the components

- (no component): a layout container, role group; holds children.
- button: role button; needs a label; shows busy (bool); sends activate (nothing).
- text_field: role text_input; needs a label; shows text (text); sends set_value (text), submit (nothing).
(...)

## the app's capabilities

todo (region) "todo"
todo.input (text_input) "new todo", in todo
  state todo.input:text: text = ""
  action todo.input set_value: takes a text
  action todo.input submit: takes nothing, disabled now
todo.filter (tab_list) "filter", in todo
  state todo.filter:selected: text = "all"
todo.filter[active] (tab) "Active", in todo.filter
  action todo.filter[active] select: takes nothing
todo.item[1] (list_item) "Absorb the plev demos into the showcase", in todo
  state todo.item[1]:completed: bool = false
  action todo.item[1] toggle: takes nothing
  action todo.item[1] remove: takes nothing, destructive
todo.counter (status) "2 active · 1 done", in todo
  state todo.counter:active: int = 2
  state todo.counter:done: int = 1
(...)

## the surface shown now

its id is `todo_panel`: keep it, and the surface changes form in place.
keep the id of every node whose capability stays on the surface, so what a person holds in it (focus, the caret, a drag) goes on; a new id is a node that leaves and another that arrives:
- `todo_panel.input`: text_field, text <- todo.input:text, set_value -> todo.input set_value, submit -> todo.input submit
- `todo_panel.add`: button, activate -> todo.input submit
- `todo_panel.filter.active`: chip, activate -> todo.filter[active] select
- `todo_panel.active`: stat, value <- todo.counter:active
(...)
its IR:
{"plev_ir":1,"surface":{"id":"todo_panel", ...}}

## the JSON schema of the IR

{ ... uir::schema(), pretty ... }
```

depois de uma recusa, o prompt ganha:

```text
## attempt 1 was refused

your answer:

<a resposta, até 16 000 caracteres>

the verifier's errors, every one, each at its JSON pointer into your IR:
- /surface/root/children/2/children/0/bind/state/value: the app declares no state `todo.counter:gone` (rule 4, state)
- (...)

answer again with the whole corrected IR, one JSON object.
```

#### a página

`crates/showcase/src/view/apps/surface_page.rs` e o módulo novo
`surface_page/proposing.rs`. acima da superfície, sob a legenda: o campo
de intenção (`surface_page.intent`, text_input com `set_value`), o botão
"propose" (`surface_page.propose`, ação `propose` que lê a intenção,
desabilitada com a intenção em branco) e a linha de mensagem
(`surface_page.status`, status com o estado `message`), dentro da região
`surface_page`; abaixo, o controle segmentado da fase 2 e a superfície.
Enter no campo propõe, Esc tira o foco.

o caminho: o clique, o Enter, o ato de um agente pela porta e o de um
leitor de tela são um `Request` sobre a página inteira, pelo dispatcher
do todo, emprestado por `TodoPage::with_dispatcher` ao `Page`, um
`Operable` que projeta o todo, a superfície e a linha, executa `set_value`
e `propose` da linha e passa o resto ao `execute` do todo. o `propose`
chama `Rules` por `propose_verified` sobre a projeção do todo, o registro
dela e o `VerifiedIr` mostrado, com o autor de quem pediu
(`GenerationPolicy::by`), e a superfície se transforma no lugar
(`SurfaceHost::transform`; `open` quando nenhuma estava aberta). a
mensagem diz "proposed by the rules for "..."" ou o erro, em vermelho
(`theme.colors.danger`), e a legenda da metade gerada passa a dizer "the
same capabilities, proposed by the rules for the intent above". um clique
no controle segmentado volta a uma forma escrita à mão.

`TodoPage::with_dispatcher` é a única mudança fora da lista de arquivos
(autorizada): dezessete linhas em `app/semantics.rs`, o mesmo `mem::take`
de `dispatch`, sem import novo, sem mudar a projeção do todo, o golden ou
o caminho de hoje. o teste
`the_page_operable_forwards_to_the_todo_as_the_todo_runs_it` roda seis
pedidos do todo (`set_value`, `submit`, `select`, `toggle`, `remove`, um
item que não existe) pelo caminho do todo num todo e pelo `Operable` da
página noutro: mesmos ids, mesmos pedidos (fora a revisão observada, que
na página cobre mais nós), mesmos estágios e resultados, e a mesma
projeção no fim.

render headless, 1200x800, core, escala 1, pelo teste
`surface_page::tests::forms::the_page_draws_headless_after_a_proposal`
(com `SURFACE_PAGE_OUT=<dir>` escreve `surface-proposed.png` e `.json`):
à esquerda o todo escrito à mão, igual; à direita, sob "the same
capabilities, proposed by the rules for the intent above", o campo com
"only what is active and the add field", o botão propose, a linha
"proposed by the rules for "only what is active and the add field"", o
controle compact | two columns, e o card da superfície com o campo "new
todo" e o add desabilitado, um chip só, "Active", e um stat só, "2
active". em 390x844 (hoff, `cargo run -p showcase -- --render Surface
--width 390 --height 844 --theme hoff`), a linha de intenção fica sob o
todo e acima do controle, o placeholder "what should it show?" cabe no
campo. o `--render Surface` de 1200x800 mostra agora 26 nós (revisão
2922fe5faa6fe87e): os 22 de antes e os 4 da linha.

a projeção depois da proposta (os 10 nós do todo como antes; os 13 da
superfície e da linha, forma lean):

```json
[
  { "id": "todo_panel", "role": "region", "label": "quick todo" },
  { "id": "todo_panel.card", "role": "group", "label": "todo", "parent": "todo_panel" },
  { "id": "todo_panel.entry", "role": "group", "label": "", "parent": "todo_panel.card" },
  { "id": "todo_panel.input", "role": "text_input", "label": "new todo", "parent": "todo_panel.entry",
    "state": { "text": { "kind": "text", "value": "" } },
    "actions": [
      { "name": "set_value", "reads": ["todo_panel.input:text"], "param": { "kind": "text", "constrained": true } },
      { "name": "submit", "enabled": false, "reads": ["todo_panel.input:text"] } ],
    "constraints": { "editable": true }, "focusable": true },
  { "id": "todo_panel.add", "role": "button", "label": "add", "parent": "todo_panel.entry",
    "actions": [ { "name": "activate", "enabled": false, "reads": ["todo_panel.input:text"] } ],
    "enabled": false, "focusable": true },
  { "id": "todo_panel.filter", "role": "group", "label": "filter", "parent": "todo_panel.card" },
  { "id": "todo_panel.filter.active", "role": "button", "label": "Active", "parent": "todo_panel.filter",
    "actions": [ { "name": "activate", "reads": ["todo.filter:selected"] } ], "focusable": true },
  { "id": "todo_panel.counts", "role": "group", "label": "", "parent": "todo_panel.card" },
  { "id": "todo_panel.active", "role": "label", "label": "active", "parent": "todo_panel.counts",
    "state": { "value": { "kind": "int", "value": 2 } } },
  { "id": "surface_page", "role": "region", "label": "propose a surface" },
  { "id": "surface_page.intent", "role": "text_input", "label": "intent", "parent": "surface_page",
    "state": { "text": { "kind": "text", "value": "only what is active and the add field" } },
    "actions": [ { "name": "set_value", "reads": ["surface_page.intent:text"], "param": { "kind": "text", "constrained": true } } ],
    "constraints": { "editable": true }, "focusable": true },
  { "id": "surface_page.propose", "role": "button", "label": "propose", "parent": "surface_page",
    "actions": [ { "name": "propose", "reads": ["surface_page.intent:text"] } ], "focusable": true },
  { "id": "surface_page.status", "role": "status", "label": "proposal", "parent": "surface_page",
    "state": { "message": { "kind": "text", "value": "proposed by the rules for \"only what is active and the add field\"" } } }
]
```

os ids que continuaram são os da fixture da task-68; `todo_panel.filter.all`,
`todo_panel.filter.completed` e `todo_panel.done` saíram.

#### a medição

headless, build debug (a do gate), apple m4, rustc 1.98.1, o todo com a
superfície compacta, intenção "only what is active and the add field",
com `cargo test -p showcase --lib surface_page::tests::measure --
--ignored --nocapture`, três execuções. "as regras" é `propose_verified`
com `Rules` até o `VerifiedIr`; "o pedido" é o `propose` de uma pessoa
inteiro (observado, pelo dispatcher, proposto, verificado, a superfície
transformada); "cena" soma o frame seguinte da página inteira montado num
`Compositor`; "pixels" soma esse frame desenhado pelo caminho da janela
num alvo fora da tela e lido de volta. o contexto de GPU já existe, como
num app rodando.

| passo | primeira rodada | melhor de 20 |
| --- | --- | --- |
| intenção até o `VerifiedIr` (as regras) | 0,23 a 0,34 ms | 0,16 ms |
| o pedido inteiro | 0,54 a 0,55 ms | 0,48 ms |
| até o primeiro frame, cena | 0,77 a 0,80 ms | 0,66 a 0,68 ms |
| até o primeiro frame, pixels | 32 a 33 ms (o primeiro texto e as pipelines do processo) | 4,2 ms |

#### contagem do gate

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ a678ef4 (`cargo test --workspace` e o `cargo test` do urnaui neste worktree, antes do primeiro código) | 2347 passam, 12 ignorados | 82 passam, 1 ignorado | 2429 |
| depois, esta branch @ fb4ca07 (`./script/gate`, saída 0) | 2368 passam, 0 falhas, 13 ignorados | 82 passam, 0 falhas, 1 ignorado | 2450 |

21 testes novos e uma medição ignorada: 14 no `propose` (2 unitários da
gramática, 8 das regras, 4 do LLM) e 7 no showcase (5 da linha de
proposta, 1 da porta, 1 do render headless). a contagem não cai. clippy
`-D warnings`, `cargo fmt --check` e o check wasm do showcase com `-D
warnings` verdes, dentro do gate. `df -h /` antes do gate: 13 GB livres.

#### testes

- `crates/propose/src/words.rs` (2): a dobra das palavras; a gramática
  nas duas línguas;
- `crates/propose/tests/rules.rs` (8): cada intenção da gramática, em
  inglês e português, verifica sobre o todo, sem superfície e sobre a da
  página, e liga o que pediu e nada mais; os ids do que fica são os da
  superfície atual (da compacta para a proposta, e das duas colunas para
  a compacta); proposta sobre proposta mantém os ids; o workbench do ide;
  intenção não entendida é erro claro; só o que o autor pode pedir; a
  mesma pergunta dá o mesmo IR; `propose_verified` nunca devolve o que o
  verifier recusa;
- `crates/propose/tests/llm.rs` (4): o modelo que erra primeiro é
  consertado pelos erros do verifier no segundo prompt; o que erra sempre
  esgota as três tentativas; o que falha para na hora; o prompt leva o
  schema, os componentes, as regras, o registro e os ids a manter;
- `surface_page/tests/proposals.rs` (5): uma pessoa propõe e a
  superfície muda no lugar, o texto digitado continua e o caret está onde
  estava (o mesmo widget); uma intenção não entendida deixa a superfície
  e diz por quê, no status e no receipt; o `Operable` da página repassa ao
  todo como o caminho do todo; a linha cabe num celular; os nós da linha
  são da página;
- `surface_page/tests/agent.rs` (+1): pela porta real, sem janela, um
  agente escreve no campo da superfície, põe a intenção, observa e
  propõe; a superfície toma a forma proposta com o mesmo id e o texto
  "milk" continua no campo; "make it purple" falha com o motivo no
  status; o receipt é do agente, no dispatcher do todo;
- `surface_page/tests/forms.rs` (+1): a página headless depois de uma
  proposta, cena, projeção e pixels;
- `surface_page/tests/measure.rs` (ignorado): a medição.

#### validação executada

- `cargo test -p propose -p showcase -p uir -p surface` (pelo gate);
- `cargo clippy -p propose -p showcase --all-targets -- -D warnings`;
- `RUSTFLAGS="-D warnings" cargo check --target wasm32-unknown-unknown -p showcase`;
- `./script/gate`, saída 0, com a perna urnaui;
- `SURFACE_PAGE_OUT=<dir> cargo test -p showcase --lib surface_page::tests::forms`,
  `cargo run -p showcase -- --render Surface` (1200x800 e 390x844), e a
  medição.

#### linha de ADR

novo, proposto: `docs/ADR/proposers-propose-the-runtime-verifies.md`.
linha do índice para o orquestrador (o `docs/ADR/README.md` é proibido
aqui):

`| 2026-09-24 | [proposers-propose-the-runtime-verifies](proposers-propose-the-runtime-verifies.md) | proposed | crate propose (uir, sem, serde_json; sem engine, comps ou rede): trait Proposer sobre Ask (intenção, Capabilities, Snapshot, VerifiedIr mostrado, GenerationPolicy) devolve Ir não verificado; propose_verified é a única saída, VerifiedIr ou os erros; Rules, determinístico, gramática pequena em inglês e português (add, filter, counts, compact, two columns, everything) ligada por estrutura do registro, nunca por nomes do app, palavras que escolhem só do app (NoMatch lista as dele), ids do que fica mantidos; Llm atrás de Model, prompt com regras, componentes, registro, superfície atual e ids a manter e o schema, erros do verifier de volta, 3 tentativas, Exhausted; a página Surface propõe pelo dispatcher do todo (TodoPage::with_dispatcher); o provedor é a fase 3b (task-69) |`

#### decisões

- a policy no `Ask`. contexto: o laço do LLM verifica dentro dele.
  decisão: `Ask` leva a `GenerationPolicy`. razão: o laço e quem chama
  verificam sob a mesma; consequência: a página passa o autor de quem
  pediu, e uma superfície proposta por um agente só liga o que agentes
  podem pedir;
- `Rules` sem léxico entre línguas (decisão do orquestrador); a
  gramática é bilíngue, a palavra que escolhe é do app;
- ids de container por quanto do novo o atual segurava (pelo menos
  metade), de dentro para fora. razão: estreitar uma parte ("só active")
  mantém o container dela; consequência: das duas colunas para a
  compacta, os números tomam o id `todo_panel.side`, a coluna que os
  segurava lá;
- o label de um nó que continua é o da superfície atual (o "add" da
  fixture fica "add");
- o controle segmentado da fase 2 fica. depois de uma proposta, ele
  continua nomeando a última forma escrita à mão, e um clique volta a
  ela;
- a linha de mensagem reserva uma linha, para que a primeira mensagem
  não mova a superfície; uma mensagem longa quebra e empurra.

#### limitações

- a gramática é pequena de propósito; fora dela é erro, não palpite. uma
  palavra em português sobre um app em inglês é recusada (o `NoMatch`
  lista as palavras do app);
- "os itens ativos" como lista não sai do registro: o todo não projeta a
  lista dos itens e diz `completed: bool`, não "ativo" (o P2 da fase 1,
  no backlog). as regras não mostram itens;
- `Model` é síncrono e `Ask` empresta; um provedor de verdade não roda
  assim na thread do app (fase 3b);
- o `Llm` não confere continuidade: um modelo que renomeia o id de uma
  capacidade que continua passa no verifier, e o widget dela se perde;
- a proposta roda dentro do `execute`, na thread da página. com as
  regras custa menos de um milissegundo; com um modelo não pode;
- o verifier segue aceitando props que o widget ignora (fase 1).

#### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima), o
  `docs/CHANGELOG.md` (a página Surface propõe a partir de uma intenção),
  e na entrada `[crates.showcase]` do `docs/arc/arc.toml` a linha de
  proposta da página e a dependência `propose` (só a entrada do crate
  novo era permitida);
- `TodoPage::with_dispatcher` é `pub(crate)`: outro `Operable` do crate
  pode pedir o dispatcher do todo; o nome e o comentário dizem para quê;
- a página inclui as duas fixtures por `include_str!`, como antes; a
  proposta não depende delas (lê o registro vivo).

#### o que a fase 3b (provedor real) precisa decidir e ligar

- o provedor (modelo local ou API), atrás de `propose::Model`, num crate
  ou feature próprio para que `propose` continue puro;
- rodar o modelo fora da thread do app: o `propose` da página responde
  `Execution::Started` e termina com `Dispatcher::complete` ou `fail`
  quando o modelo responde (o dispatcher e a porta já seguem estágios
  que terminam depois); `Ask` empresta, então o trabalhador precisa de
  dados próprios (o snapshot, o registro e o IR atual clonados);
- onde o prompt se divide em sistema e usuário para uma API de chat, e se
  o provedor aceita decodificação restrita pelo `uir::schema()`;
- o tempo máximo de espera, o que a página mostra enquanto o modelo
  pensa, e se a proposta anterior pode ser cancelada;
- credenciais e custo: onde a chave mora, e quanto custa cada tentativa
  (até três por proposta, uns 18 KB de prompt sobre o todo, mais a
  resposta e os erros nas seguintes);
- se uma proposta que renomeia o id de uma capacidade que continua volta
  ao modelo como erro brando (continuidade não é regra do verifier);
- qual proponente a página usa: as regras primeiro e o modelo quando
  elas não entendem, ou só o modelo.

## fase 3b-i: o Claude como modelo do proponente

branch `task/69d-claude-model`, worktree próprio, sobre a `main` @ ce9acee.
a decisão é a 13 do programa: o modelo é o Claude, pelo `claude -p` do
Claude Code desta máquina (`/Users/nn/.local/bin/claude`, 2.1.281, logado
na conta do brenner), atrás de `propose::Model`, num crate próprio.

### sonda da fase 3b-i (antes do primeiro tipo, 2026-09-24)

um exemplo descartável (`crates/claude-model/examples/probe.rs`, rodado e
jogado fora, nunca commitado) monta o `propose::prompt` para a intenção
"only what is active and the add field" sobre o golden do todo
(`crates/uir/fixtures/showcase-todo.snapshot.json`), com a superfície
compacta da página como a atual (`showcase-todo.ir.json`, id
`todo_panel`), e escreve o prompt num arquivo: 17.704 bytes. o mesmo
exemplo, com `check <arquivo>`, passa uma resposta por
`propose_verified(Llm::with_attempts(<a resposta>, 1), ask)`.

três chamadas reais, todas de um diretório vazio (`probe-cwd`, no
scratchpad), sem as variáveis que a sessão do Claude Code que rodava a
sonda põe no ambiente dos processos dela (`CLAUDECODE`,
`CLAUDE_CODE_ENTRYPOINT`, `CLAUDE_CODE_SESSION_ID`,
`CLAUDE_CODE_CHILD_SESSION`, `CLAUDE_CODE_SESSION_ATTENDED`,
`CLAUDE_CODE_MESSAGING_SOCKET`, `CLAUDE_CODE_MESSAGING_TOKEN`,
`CLAUDE_CODE_EXECPATH`, `CLAUDE_PID`, `CLAUDE_EFFORT`, `AI_AGENT`). elas
são da sessão, não do app, e uma delas, `CLAUDE_EFFORT=xhigh`, mudaria o
esforço do filho. o `env -u ...` abaixo é abreviado; tira as onze.

a primeira, com o system prompt padrão do Claude Code e a saída em
stream-json para ver o `init` (ferramentas, MCP, skills) e cada evento:

```sh
env -u CLAUDECODE -u ... -u AI_AGENT \
  claude -p --model sonnet --output-format stream-json --verbose \
    --tools "" --no-session-persistence --setting-sources "" \
    --strict-mcp-config --disable-slash-commands --permission-prompts none \
    --max-budget-usd 0.50 --debug-file probe-1.debug \
  < probe-prompt.txt > probe-1.out
```

a segunda, com as flags que o crate usa (a saída em json, um objeto só, e
um system prompt curto no lugar do do Claude Code):

```sh
env -u CLAUDECODE -u ... -u AI_AGENT \
  claude -p --model sonnet --output-format json \
    --tools "" --no-session-persistence --setting-sources "" \
    --strict-mcp-config --disable-slash-commands --permission-prompts none \
    --max-budget-usd 0.50 \
    --system-prompt "you are called by a program, not by a person. its message is the whole task: answer exactly what it asks for, in the form it asks for, with nothing before or after it." \
    --debug-file probe-2.debug \
  < probe-prompt.txt > probe-2.out
```

| | 1 (system prompt do Claude Code) | 2 (as flags do crate) |
| --- | --- | --- |
| tempo de parede | 40,1 s (API 39,3 s, primeiro byte em 2,0 s) | 24,5 s (API 23,9 s) |
| saída do processo | 25.470 bytes de eventos | 3.145 bytes, um objeto json |
| a resposta (`result`) | 1.324 caracteres | 1.322 caracteres |
| tokens de entrada | 11.150 | 6.983 |
| tokens de saída (de pensamento) | 3.667 (3.063) | 2.628 (2.026) |
| custo que o `claude` mostra | 0,069 USD | 0,054 USD |
| exit code, stderr | 0, vazio | 0, vazio |
| `propose_verified` de primeira | passa | passa |

o custo é o de lista que o `claude` calcula (`costBasis: list`); a conta
é de assinatura (`apiKeySource: none`, login OAuth do keychain), então é
uma medida de tamanho, não uma cobrança. a diferença de tempo entre as
duas é mais pensamento do que prompt (mil tokens de pensamento a menos na
segunda); com uma chamada de cada não dá para separar. o system prompt
próprio tira uns 4.200 tokens de entrada (o prompt do Claude Code, que
fala de ferramentas que o filho não tem, e do diretório e do git dele).

nenhuma pediu interação, login ou permissão. o que o `claude` usou:

- `init` da primeira: `tools: []`, `mcp_servers: []`, `skills: []`,
  `slash_commands: []`, `permissionMode: default`, modelo
  `claude-sonnet-5`. as duas: `num_turns: 1`, `permission_denials: []`,
  `stop_reason: end_turn`. nenhuma ferramenta, nenhuma tentativa;
- o log de debug: "project memory is off" (nenhum CLAUDE.md), 0 hooks, 0
  skills, só o plugin embutido `agents-md` carregado (sem hooks
  registrados). o `claude` escreveu o próprio `~/.claude.json` (três
  escritas atômicas, o estado dele, como em qualquer uso) e buscou a
  lista de conectores do claude.ai (1 servidor), que não conectou
  (`mcp_servers: []`). nenhum arquivo do repositório: o diretório era
  vazio e não há ferramenta de leitura.

a resposta da segunda, um objeto json puro, sem prosa e sem cerca (a
primeira é igual, fora `gap: sm` e os ids `item_1`, `item_2`):

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo")
  todo_panel.entry (container)
    todo_panel.input (text_field, "new todo"): text <- todo.input:text, set_value -> todo.input set_value, submit -> todo.input submit
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.active_list (list, "active")
    todo_panel.active_list.item1 (checkbox, "Absorb the plev demos into the showcase"): checked <- todo.item[1]:completed, toggle -> todo.item[1] toggle
    todo_panel.active_list.item2 (checkbox, "Measure text, never estimate it"): checked <- todo.item[2]:completed, toggle -> todo.item[2] toggle
```

o Claude leu "o que está ativo" como os itens não concluídos agora
(`todo.item[1]` e `[2]` têm `completed: false`, o `[3]` tem `true`), e
não como o chip "Active" que as regras escolhem. os ids da entrada
(`todo_panel.input`, `todo_panel.add`) são os da superfície atual; os
chips e os números saíram. é uma leitura dos valores de agora, não um
filtro: um item concluído depois continua na lista, marcado, e um item
novo não entra. a lista que se repete sobre os itens é a task-95.

a terceira mediu o que acontece quando o orçamento estoura: o mesmo
comando da segunda com `--max-budget-usd 0.0001` e um prompt de uma linha
(`answer with the JSON object {"ok": true} and nothing else.`). 2,8 s,
exit 1, stderr vazio, e na stdout o objeto com `is_error: true`,
`subtype: error_max_budget_usd`, `errors: ["Reached maximum budget
($0.0001)"]`, sem `result`, custo 0,0013 USD. o limite é conferido
depois do turno: ele não corta uma chamada no meio. com um turno só e
sem ferramentas, quem limita uma chamada é o tempo máximo do crate e o
tamanho do prompt; o `--max-budget-usd` fica como teto.

o que a sonda decide:

- o `propose::prompt` não muda: as duas respostas vieram só com o json,
  a instrução do fim do primeiro parágrafo ("answer with the IR only: one
  JSON object, no prose") bastou, e o `read` do `Llm` já tira cerca e
  prosa se vierem;
- as flags do crate são as da segunda chamada, com o motivo de cada uma
  no handoff; a saída em json porque ela separa resposta de erro
  (`is_error`, `subtype`, `errors`) e diz o custo e os turnos; um erro
  vem com exit 1 e a explicação na stdout, não no stderr;
- o crate tira do ambiente do filho as variáveis da sessão do Claude
  Code acima e roda o filho num diretório dele;
- a condição de parada não disparou: sem interação, sem login, sem
  permissão, as duas chamadas longe dos 5 minutos, custo pequeno.

### corridas reais do exemplo (2026-09-24)

`cargo run -p claude-model --example propose -- --app todo|ide "<intenção>"`,
com o `claude` de verdade (sonnet, as flags do crate), da branch @ 31ff396
mais o exemplo; o todo com a superfície compacta mostrada, o ide sem
superfície. as quatro primeiras rodaram com o tempo máximo em 120 s; a
do ide levou 89 s, e o padrão subiu para 180 s antes do commit. a
quinta repete a terceira com o exemplo como foi commitado (ele passou a
imprimir o tamanho do prompt e o layout de cada nó).

| # | app | intenção | as regras | tentativas | tempo | custo | resposta |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | todo | the add field and the counts in two columns | verificam, 7 nós | 1, verificada | 4,5 s | 0,033 USD | 1.051 bytes |
| 2 | todo | a compact card with the input on top and the counts below | `"card", "top", "below" are not a word of the app` | 1, verificada | 5,6 s | 0,034 USD | 1.049 bytes |
| 3 | todo | show the filter as chips and hide the counts | `"chips", "hide" are not a word of the app` | 1, verificada | 6,3 s | 0,035 USD | 1.308 bytes |
| 4 | ide | review what goes into the commit: each changed file with stage and unstage, then the message and the commit button | `"review", "goes", "each", "changed", "file", "stage", "unstage", "then" are not a word of the app` | 1, verificada | 88,9 s | 0,111 USD | 5.653 bytes |
| 5 | todo | a 3 de novo | igual | 1, verificada | 8,7 s | 0,011 USD (prompt em cache) | 1.308 bytes, o mesmo IR |

os prompts: 17.710, 17.724 e 17.711 bytes no todo, 17.075 no ide (sem
superfície atual, mas com mais nós). nenhuma resposta foi recusada, então
o laço de conserto não rodou de verdade; ele está provado com o `claude`
falso (teste `a_refused_answer_is_repaired_by_the_second_call`).

os IRs, resumidos (id, componente, label, layout, o que liga):

1, "the add field and the counts in two columns":

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo", stack vertical md; padding lg, raised)
  todo_panel.entry (container, stack horizontal sm)
    todo_panel.input (text_field, "new todo"): text <- todo.input:text, set_value -> todo.input set_value, submit -> todo.input submit
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.counts (container, grid min_col card_min_w, gap sm)
    todo_panel.active (stat, "active"): value <- todo.counter:active
    todo_panel.done (stat, "done"): value <- todo.counter:done
```

o Claude leu "two columns" como os dois números lado a lado num grid, e
não a superfície inteira em duas colunas, como as regras leem. as duas
leituras cabem na frase.

2, "a compact card with the input on top and the counts below":

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo", stack vertical sm; padding sm, raised)
  todo_panel.entry (container, stack horizontal sm)
    todo_panel.input (text_field, "new todo"): (as mesmas ligações)
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.counts (container, stack horizontal sm)
    todo_panel.active (stat, "active"): value <- todo.counter:active
    todo_panel.done (stat, "done"): value <- todo.counter:done
```

"compact" virou padding e gap `sm`; o filtro saiu, porque a frase não o
pede.

3, "show the filter as chips and hide the counts":

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo", stack vertical md; padding lg, raised)
  todo_panel.entry (container, stack horizontal sm)
    todo_panel.input (text_field, "new todo"): (as mesmas ligações)
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.filter (container, "filter", stack horizontal xs)
    todo_panel.filter.all (chip, "All"): activate -> todo.filter[all] select
    todo_panel.filter.active (chip, "Active"): activate -> todo.filter[active] select
    todo_panel.filter.completed (chip, "Completed"): activate -> todo.filter[completed] select
```

nas três do todo, cada nó cuja capacidade continua ficou com o id da
superfície compacta (`todo_panel.input`, `todo_panel.add`,
`todo_panel.filter.*`, `todo_panel.active`, `todo_panel.done`), e o id da
superfície também: a página transformaria no lugar.

4, "review what goes into the commit: ...", sobre o workbench do ide:

```text
surface commit_review "Commit Review"
commit_review.root (container, stack vertical md)
  commit_review.changes (card, "Changes", stack vertical sm)
    commit_review.changes.list (list, "Changed files")
      changes.file[src/lexer.rs] (container, stack horizontal sm)
        changes.file[src/lexer.rs].name (text, "src/lexer.rs")
        changes.file[src/lexer.rs].status (badge, "src/lexer.rs status"): text <- changes.file[src/lexer.rs]:status
        changes.file[src/lexer.rs].stage_btn (button, "Stage src/lexer.rs"): activate -> changes.file[src/lexer.rs] stage
        changes.file[src/lexer.rs].unstage_btn (button, "Unstage src/lexer.rs"): activate -> changes.file[src/lexer.rs] unstage
      changes.file[notes.md] (...o mesmo, para notes.md)
      changes.file[src/parser.rs] (...o mesmo, para src/parser.rs)
  commit_review.commit (card, "Commit", stack vertical sm)
    commit_review.commit.message (text_field, "Commit message"): text <- commit.message:text, set_value -> commit.message set_value
    commit_review.commit.submit (button, "Commit"): activate -> commit.submit submit
```

duas coisas desse IR importam para depois. primeiro, o Claude escreveu os
três arquivos um a um, e não um `repeat` sobre `changes:files` (a lista
existe no registro, e a fixture `ide-review-commit.ir.json` repete sobre
ela): é a mesma leitura dos valores de agora que a sonda mostrou nos
itens do todo, e um arquivo que mudar depois não aparece. segundo, os nós
dos arquivos tomam o id do nó do app (`changes.file[src/lexer.rs]`) e os
filhos nascem embaixo dele. o verifier aceita (regra identity: um nó com
o id de um nó do app só manda as ações dele), e o host da superfície
trata esse nó como do app, o caso da tela fixa escrita em IR
(`crates/surface/src/project.rs`): não o projeta de novo, e os filhos
novos ficam sob o nó do app na projeção. se é isso que uma superfície
gerada ao lado do app deve fazer, a 3b-ii decide. nenhuma das duas é
deste crate; estão no handoff.

### handoff da fase 3b-i (2026-09-24)

goal: task-69 fase 3b-i, o Claude como modelo do proponente: um crate
novo implementa `propose::Model` chamando o `claude -p` como processo, e
um exemplo mostra uma intenção em texto virando um `VerifiedIr` sobre a
projeção real do todo e do ide, pelo `propose::Llm` e pelo
`propose_verified`.

status: implementado. `cargo test -p claude-model -p propose` verdes,
`./script/gate` verde com a perna urnaui, sobre a `main` @ ce9acee. a
`main` andou para 9299f2d depois (só o journal); o rebase é do
orquestrador.

#### commits

branch `task/69d-claude-model`, na ordem do briefing:

- b771ead task-69d: a sonda da fase 3b-i no arquivo da task
- 31ff396 task-69d: claude-model, the Claude behind propose::Model through claude -p
- 524332f task-69d: the propose example, and the Claude run for real on the todo and the ide
- e o commit do ADR e deste handoff

#### a sonda

acima ("sonda da fase 3b-i"). em uma linha: o prompt do `propose` para
"only what is active and the add field" sobre o golden do todo, 17,7 KB,
pelo `claude -p` sem ferramentas, sem sessão salva, sem MCP e sem as
configurações do usuário, voltou só com o json e passou pelo
`propose_verified` de primeira nas duas chamadas (40,1 s com o system
prompt do Claude Code, 24,5 s com o do crate); o `claude` não usou
ferramenta nem arquivo do repositório (`tools: []`, `num_turns: 1`,
`permission_denials: []`); a terceira chamada mostrou que um orçamento
estourado sai com exit 1 e o motivo no json da stdout. o `propose::prompt`
não mudou. nenhuma condição de parada disparou.

#### o crate

`crates/claude-model` (pacote `claude-model`, lib `claude_model`).
depende de `propose` (o trait) e de `serde_json` (o resultado do
filho); nunca de `uir`, `sem`, engine, comps, showcase, ide ou de um
cliente HTTP. `uir`, `sem` e `tempfile` só nos testes e no exemplo.

o `serde_json` não estava na lista do briefing (`propose`, a std e
`thiserror`). entrou porque a saída em json é o que separa resposta de
erro: um orçamento estourado sai com exit 1, nada no stderr e o motivo no
objeto da stdout; em texto, o erro e a resposta seriam o mesmo texto. o
`propose` já depende dele, então nada novo compila. o `thiserror` não
foi preciso: todo erro é o `ModelError` do `propose`.

a API:

- `ClaudeCli { program, model, timeout, budget_usd, dir }`, campos
  públicos, e o registro das chamadas, privado. `ClaudeCli::new()` (e
  `Default`): `PLEV_CLAUDE` ou `claude`, `sonnet`, 180 s, 0,5 USD, o
  diretório temporário do sistema;
- `impl propose::Model for ClaudeCli`: roda o programa uma vez por
  `complete`;
- `args()`: os argumentos exatos (os testes comparam com o que o
  `claude` falso recebeu); `calls()`: cada chamada como `Call { took,
  cost_usd, turns, reply: Result<usize, ModelError> }`; `spent_usd()`;
- as constantes `PROGRAM`, `PROGRAM_ENV` (`PLEV_CLAUDE`), `MODEL`,
  `TIMEOUT`, `BUDGET_USD`, `SYSTEM` e `SESSION_VARS`.

três módulos, todos pequenos: `lib.rs` (a configuração, os argumentos, o
`Model`, 263 linhas com os testes), `run.rs` (o processo: stdin numa
thread, stdout e stderr em threads, o prazo, o kill; 129) e `reply.rs`
(o json do resultado virando resposta ou `ModelError`; 177 com os
testes).

#### as flags do `claude` e o motivo de cada uma

| flag | por quê |
| --- | --- |
| `-p` | uma resposta e o processo sai; o prompt vem da stdin, sem argumento posicional |
| `--output-format json` | um objeto só: `result`, ou `is_error` com `subtype` e `errors`, mais `total_cost_usd` e `num_turns`. é o que separa resposta de erro (a sonda: orçamento estourado sai com exit 1 e o motivo na stdout) |
| `--tools ""` | nenhuma ferramenta: o filho responde texto, não lê arquivo, não roda comando, e o turno é um só |
| `--no-session-persistence` | nada salvo em disco para retomar |
| `--setting-sources ""` | nenhum arquivo de configuração (usuário, projeto, local): sem hooks, sem permissões, sem a memória do usuário; o log da sonda diz "project memory is off" e 0 hooks |
| `--strict-mcp-config` | só os servidores MCP do `--mcp-config`, que não é passado: nenhum (`mcp_servers: []`) |
| `--disable-slash-commands` | nenhuma skill listada no prompt (`skills: []`) |
| `--permission-prompts none` | o que pediria permissão é negado sozinho: nada espera uma pessoa |
| `--system-prompt <SYSTEM>` | duas frases no lugar do system prompt do Claude Code, que fala de ferramentas, diretório e git que o filho não tem; uns 4.200 tokens de entrada a menos (11.150 para 6.983 na sonda) |
| `--model <model>` | `sonnet` por padrão: a sonda verificou de primeira em 24 a 40 s; `opus` ou um nome completo pelo campo |
| `--max-budget-usd <budget>` | 0,5 por padrão, umas dez vezes uma chamada do todo. teto de sessão: o filho confere depois do turno, não corta uma chamada no meio |

e fora das flags: o filho roda em `dir` (o diretório temporário do
sistema por padrão, nenhum projeto do app é lido dali) e sem as onze
variáveis que uma sessão do Claude Code põe nos processos dela
(`SESSION_VARS`: `CLAUDECODE`, `CLAUDE_CODE_ENTRYPOINT`,
`CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_CHILD_SESSION`,
`CLAUDE_CODE_SESSION_ATTENDED`, `CLAUDE_CODE_MESSAGING_SOCKET`,
`CLAUDE_CODE_MESSAGING_TOKEN`, `CLAUDE_CODE_EXECPATH`, `CLAUDE_PID`,
`CLAUDE_EFFORT`, `AI_AGENT`). um app rodado de dentro de uma sessão (como
o exemplo, rodado por um agente) passaria essas variáveis ao filho, e o
`CLAUDE_EFFORT=xhigh` desta sessão mudaria o esforço dele. o resto do
ambiente passa (PATH, HOME, e o que o usuário configurou para o login).

não usadas: `--bare` (lê só `ANTHROPIC_API_KEY`, nunca o login OAuth da
conta), `--json-schema` (decodificação restrita pelo `uir::schema()`; não
testada, porque o prompt já leva o schema e o verifier é quem decide, e
fica como opção para a 3b-ii), `--safe-mode` (o `--setting-sources ""`
com as outras flags já deixou tudo vazio na sonda), `--effort` (o padrão
do modelo verificou de primeira).

#### o tempo máximo

180 s por chamada (`TIMEOUT`), o dobro da chamada mais lenta medida (89
s, o workbench do ide, 5,6 KB de IR); as do todo levaram de 4,5 a 40 s.
começou em 120 s (três vezes a sonda) e subiu depois da corrida do ide.
uma proposta faz até três chamadas (`Llm::ATTEMPTS`), então espera no
máximo 9 minutos; a página (3b-ii) não deve esperar isso sem mostrar e
sem deixar cancelar.

ao estourar, `Child::kill` (SIGKILL no pid do filho), `wait` para colher,
e `ModelError("<programa> did not answer in 180 s: killed (pid N)")`; o
`Llm` para na hora. o filho é consultado a cada 10 ms (`try_wait`).
depois que ele sai, o fim da saída é esperado no máximo 1 s: um neto que
segura o pipe não segura a chamada, e o que o filho escreveu fica (teste
`a_grandchild_holding_the_pipes_does_not_hold_the_call`). o neto não é
morto: só o pid do filho.

#### as corridas reais

a tabela e os IRs estão acima ("corridas reais do exemplo"). resumo:

| app | intenção | as regras | tentativas | tempo |
| --- | --- | --- | --- | --- |
| todo | the add field and the counts in two columns | entendem | 1 | 4,5 s |
| todo | a compact card with the input on top and the counts below | não entendem | 1 | 5,6 s |
| todo | show the filter as chips and hide the counts | não entendem | 1 | 6,3 s (8,7 s de novo, com o exemplo final) |
| ide | review what goes into the commit: each changed file with stage and unstage, then the message and the commit button | não entendem | 1 | 88,9 s |

as quatro verificaram de primeira; nas do todo, os ids do que ficou são
os da superfície compacta. o custo que o `claude` mostra: uns 0,034 USD
por chamada no todo, 0,111 no ide, um terço com o prompt em cache.

#### contagem do gate

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ ce9acee (`./script/gate` neste worktree, antes do primeiro código) | 2372 passam, 0 falhas, 13 ignorados | 82 passam, 0 falhas, 1 ignorado | 2454 |
| depois, esta branch @ 524332f (`./script/gate`, saída 0) | 2385 passam, 0 falhas, 13 ignorados | 82 passam, 0 falhas, 1 ignorado | 2467 |

13 testes novos, todos no `claude-model`: 6 unitários e 7 com o `claude`
falso. a contagem não cai. clippy `-D warnings`, `cargo fmt --check` e o
check wasm do showcase verdes, dentro do gate. `df -h /` antes do gate:
14 GB livres.

#### testes

- `src/lib.rs` (2): os argumentos são os da sonda, cada flag uma vez,
  as listas vazias como argumento vazio, o modelo e o orçamento dos
  campos; os padrões;
- `src/reply.rs` (4): um resultado é a resposta com o custo e os
  turnos; um resultado de erro é o erro com o tipo e o motivo (o do
  orçamento da sonda); um resultado só é resposta quando tudo diz que é
  (exit 0, `is_error: false`, `result` presente); saída que não é
  resultado vira um erro curto, numa linha;
- `tests/cli.rs` (7), com `tests/fake-claude` (um script sh que responde
  de arquivos no diretório que o `ClaudeCli` dá a ele, e guarda os
  argumentos, o ambiente, cada prompt e o pid): uma resposta boa é uma
  superfície verificada, e o filho recebeu exatamente `args()`, o prompt
  inteiro pela stdin (17 KB, mais do que um pipe segura) e nenhuma das
  variáveis da sessão (o gate rodou dentro de uma, então o teste
  conferiu de verdade); uma resposta ruim é consertada pela segunda
  chamada, que leva a resposta e os erros; um filho além do tempo morre
  (o pid dele não existe mais) e a proposta para em 1 s, não em 30; um
  neto que segura os pipes não segura a chamada; um programa que falha
  é o erro na hora; um orçamento estourado é o erro com o custo; um
  programa que não existe é o erro.

nenhum teste chama o `claude` de verdade: todos apontam `program` para o
script. os testes de processo são `#[cfg(unix)]`.

#### validação executada

- `cargo test -p claude-model -p propose`;
- `cargo clippy -p claude-model --all-targets -- -D warnings`,
  `cargo fmt -p claude-model --check`;
- `cargo test -p engine --test arc_sync_guard`;
- `./script/gate`, saída 0, com a perna urnaui;
- o exemplo com o `claude` de verdade: as quatro corridas e a repetição
  acima, e com `PLEV_CLAUDE=/nonexistent/claude` (erro "could not
  start", exit 1) e `--app nope` (uso, exit 2).

para reproduzir sem rede: `cargo test -p claude-model`. com rede, uma
chamada: `cargo run -p claude-model --example propose -- --app todo
"show the filter as chips and hide the counts"`.

#### seção de ADR

`docs/ADR/proposers-propose-the-runtime-verifies.md`, seção nova no fim,
"addendum, 2026-09-24: the Claude as the model (task-69 phase 3b-i,
proposed)": contexto (a decisão 13), decisão (o crate, o processo, cada
flag com o motivo, o filho fora da sessão, o prazo, os erros, o prompt
que não muda, os testes sem rede, o exemplo), custo (a sonda e as
corridas), consequências (o que ele lê que as regras não leem, valores
de agora em vez de listas, os ids tomados do app, o teto de orçamento, o
que o `claude` escreve, o que a 3b-ii decide) e o que evitar.

para o orquestrador (o `docs/ADR/README.md` é proibido aqui): no fim da
linha do índice desse ADR, algo como "; addendum 3b-i (proposed): crate
claude-model, `ClaudeCli` implementa `Model` pelo `claude -p` como
processo (json, sem ferramentas, sem sessão salva, sem MCP, sem
configurações, system prompt próprio, fora da sessão do Claude Code,
180 s e kill pelo pid), testes com um `claude` falso".

#### decisões

- saída em json e `serde_json`. contexto: a sonda mostrou que um erro
  sai com exit 1 e o motivo na stdout. decisão: ler o objeto de
  resultado. razão: só ele separa resposta de erro, e dá custo e turnos.
  consequência: uma dependência a mais no crate (já compilada pelo
  `propose`);
- system prompt próprio. razão: o do Claude Code descreve ferramentas
  que o filho não tem e o diretório e o git dele; tirar poupa 40% da
  entrada. consequência: a instrução de responder só o json continua no
  prompt do `propose`, e o do crate só diz que quem chama é um programa;
- o filho fora da sessão do Claude Code (as onze variáveis) e num
  diretório dele. razão: um agente roda o exemplo de dentro de uma
  sessão, e o app também pode ser rodado assim;
- `sonnet` por padrão, 180 s, 0,5 USD; tudo campo;
- `PLEV_CLAUDE` só para o programa. os testes não usam a variável
  (mudar o ambiente num teste é `unsafe` na edição 2024); apontam
  `program` direto;
- o registro das chamadas no `ClaudeCli` (tempo, custo, turnos), para
  a página mostrar e o exemplo imprimir;
- o `propose` não mudou em nada.

#### limitações

- síncrono: `complete` bloqueia a thread de quem chama de segundos a um
  minuto e meio por tentativa. rodar fora da thread do app é da 3b-ii;
- cancelar só pelo prazo: não há como matar o filho de fora antes dele;
- o Claude lê valores de agora, não listas (os itens do todo, os
  arquivos do ide), e responde diferente a cada chamada (as duas da
  sonda diferem no gap e nos ids dos itens);
- `SESSION_VARS` é a lista da versão 2.1.281; uma variável nova de uma
  versão futura passa;
- o custo mostrado é o de lista. as chamadas contam no plano do brenner,
  o mesmo das sessões dos agentes (a sonda viu `rate_limit_event` com a
  janela de cinco horas em 20%);
- o `--max-budget-usd` não corta uma chamada; o prazo é que corta;
- o `claude` escreve o próprio `~/.claude.json` e busca a lista de
  conectores do claude.ai ao iniciar, como em qualquer uso;
- um neto que segura um pipe continua vivo depois da chamada;
- os testes de processo são só unix (o `claude` falso é um script sh).

#### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima); o
  `docs/CHANGELOG.md` (crate novo e o exemplo); no `docs/arc/arc.toml`,
  a linha `claude-model = ["propose"]` em `[examples]` (só a entrada do
  crate era permitida; o exemplo está citado nela); no `AGENTS.md`,
  "## running", o comando do exemplo, se quiser;
- rebase sobre a `main` @ 9299f2d (só o journal; sem conflito
  esperado);
- o exemplo chama o `claude` de verdade e custa uma chamada ou mais;
  nada no gate o roda (é exemplo, compila no clippy, não executa);
- o `tests/fake-claude` precisa do bit de execução (está como 100755 no
  git).

#### o que a 3b-ii (a página) precisa

- rodar fora da thread do app: o `propose` da página responde
  `Execution::Started` e termina com `Dispatcher::complete` ou `fail`
  quando o Claude responde. o `ClaudeCli` é `Clone` e `Send` e pode ir
  inteiro para um trabalhador; o `Ask` empresta, então o trabalhador
  leva o snapshot, o registro e o `VerifiedIr` atual clonados e monta o
  `Ask` lá dentro;
- cancelar: hoje só o prazo mata o filho. um cancelamento de fora pede
  uma mudança pequena no `run` (um sinal conferido a cada 10 ms, junto
  do prazo), e é a forma de a página abandonar uma proposta;
- o que a página mostra durante 5 a 90 s por tentativa, até três; o
  `calls()` dá tempo e custo de cada uma;
- qual proponente: as regras primeiro (0,16 ms) e o Claude quando elas
  não entendem, ou só o Claude. as duas leem "only what is active"
  diferente (o chip "Active" contra os itens não concluídos), então
  "regras primeiro" muda o sentido de uma intenção que as duas leem.
  decisão do orquestrador ou do brenner;
- o prompt: pedir um `repeat` quando uma lista mostra cada membro (o
  Claude desenrolou os itens e os arquivos), agora que a task-95 projeta
  os itens do todo como lista; e se um id tomado de um nó do app cabe
  numa superfície gerada ao lado dele. as duas mexem no `propose` e
  pedem uma sonda nova;
- continuidade: se uma proposta que troca o id de uma capacidade que
  continua volta ao modelo como erro brando (nas corridas, o Claude
  manteve todos);
- custo e limites: cada proposta custa uma a três chamadas do plano do
  brenner; o cache do prompt (uma hora) barateia a segunda proposta
  sobre o mesmo app.

## fase 3b-ii: a página Surface pede a tela ao Claude

branch `task/69e-page-asks-claude`, worktree próprio, sobre a `main` @
cc908560 (a `main` já tinha andado para dcd731f4, só o journal). a
decisão é a 13 do programa: o Claude pelo `claude -p` local, já atrás de
`propose::Model` no crate `claude-model` (3b-i). nesta fase a página usa
esse modelo, fora da thread do app, com a espera visível e um cancelar.

### sonda da fase 3b-ii (antes do primeiro tipo, 2026-09-24)

dois testes descartáveis (`surface_page/tests/probe.rs` no showcase e
`crates/claude-model/tests/probe.rs`, rodados e jogados fora, nunca
commitados) e uma chamada real pelo exemplo do `claude-model`.

(a) na base, o `propose` da página roda na thread de quem despacha e já
está `completed` quando `dispatch` volta. um `propose` pelas regras
("only what is active and the add field") por `Page::dispatch`: o
receipt tem `requested`, `accepted`, `executing` e `completed` todos no
mesmo `now` do relógio da página (4 ms), `dispatch` voltou depois de
1,9 ms (build debug) na thread do teste, e a superfície abriu
(`todo_panel`). as regras rodam dentro do `execute`.

o que `Execution::Started` mais `Dispatcher::complete` exigem de quem
chama, medido com um `Operable` sobre o todo cujo `toggle` devolve
`Started`:

- o id que `execute` recebe é o que `dispatch` devolve (o `Operable`
  guarda esse id; nenhum outro canal diz qual request acabou);
- o receipt fica aberto em `executing`; enquanto isso o todo recebe e
  completa um `set_value` pelo caminho de sempre, e o item não mudou (a
  execução não terminou);
- `todo.with_dispatcher(|d, _, now| d.complete(id, outcome, now))`
  termina: `completed` no `now` 5, os estágios anteriores no 4. o
  `complete` pede o `RequestId`, um `Outcome` (o mapa de resultado) e um
  `Millis` do relógio da página, que só o `with_dispatcher` dá; não pede
  o `Operable`, só o dispatcher emprestado de novo;
- um segundo `complete` é `IllegalTransition { from: completed, to:
  completed }`; um `fail` num request que terminou na hora é
  `IllegalTransition { from: completed, to: failed }`; um id desconhecido
  é `UnknownRequest`.

a condição de parada não disparou: o `with_dispatcher` empresta o
dispatcher depois, e `complete` e `fail` servem com o id e o `now` da
página.

(b) com o `fake-claude`, `propose_verified(Llm::new(&mut ClaudeCli {
program: fake, .. }))` segura a thread que o chama pela chamada inteira:
o falso que responde na hora segurou 338 ms na primeira rodada (o
primeiro processo) e 16 a 17 ms nas duas seguintes (a chamada em si, 15
a 16 ms, é o `sh` subindo); o falso que nunca responde, com prazo de 2 s,
segurou 2,01 s até o kill. com o Claude de verdade, 5 a 90 s por
tentativa e até três tentativas: a thread do app congelaria isso tudo.

(c) chamada real 1 de 6: `cargo run -p claude-model --example propose --
--app todo "the items as a list with the add field on top"`, das
05:04:24 às 05:04:32. as regras recusam (`"top" is not a word of the
app`). o prompt tem 18.760 bytes: a superfície mostrada é a compacta da
task-95, que já repete `todo_panel.items` sobre `todo:items`. uma
tentativa, 7,2 s, 0,038 USD, 1.335 bytes, verificada de primeira. o IR
resumido:

```text
surface todo_panel "quick todo"
todo_panel.card (card, "todo", stack vertical md; padding lg, raised)
  todo_panel.entry (container, stack horizontal sm, center)
    todo_panel.input (text_field, "new todo", flex 1): text <- todo.input:text, set_value -> todo.input set_value, submit -> todo.input submit
    todo_panel.add (button, "add"): activate -> todo.input submit
  todo_panel.items (list, "items", stack vertical xs)
    repeat over todo:items:
      todo_panel.item[{item}] (container, stack horizontal sm, center)
        todo_panel.item[{item}].text (text): text <- todo.item[{item}]:text
        todo_panel.item[{item}].done (checkbox, "done"): checked <- todo.item[{item}]:completed, toggle -> todo.item[{item}] toggle
```

o Claude usou um `repeat` sobre `todo:items`, com os ids do molde da
superfície mostrada (`todo_panel.item[{item}]`, `.text`, `.done`), tirou
o filtro e as contagens e deixou a entrada em cima. a condição do item 5
do briefing ("enumera os itens em vez de repetir") não disparou: o
`propose::prompt` não muda nesta fase. a ressalva fica registrada: a
superfície mostrada já carregava o `repeat`, e na corrida do ide da 3b-i,
sem superfície mostrada, o Claude escreveu os arquivos um a um. se ele
repete do zero é uma chamada sobressalente desta task, se o orçamento
sobrar, e importa para a fase 4, que nasce sem superfície.

o que a sonda decide para o desenho:

- o `propose` da página com `claude` devolve `Execution::Started` e
  guarda o id; um trabalhador (thread) leva o `ClaudeCli` clonado e as
  cópias do que o `Ask` empresta, roda `propose_verified(Llm)` e devolve
  por canal; o `wake` da página recolhe e chama `complete` ou `fail`
  pelo `with_dispatcher`, com o `now` do relógio da página;
- cancelar é um sinal conferido no laço do `try_wait` do `run`, junto do
  prazo, a cada 10 ms; o filho morre pelo pid como no prazo;
- o proponente padrão é `claude` quando o programa existe
  (`ClaudeCli::exists`: um caminho com diretório é um arquivo ali, um
  nome é achado no PATH), sem chamar o modelo; nos testes do próprio
  crate o programa não é procurado, para que nenhum teste do gate chegue
  ao `claude` de verdade (o teste que quer o falso o aponta).

### chamadas reais desta fase (2026-09-24)

três de seis, todas com o `claude` 2.1.281 desta máquina, sonnet, as
flags do crate, verificadas de primeira; nenhuma repetição foi preciso.
o custo é o de lista que o `claude` conta (a conta é de assinatura).

| # | por onde | intenção | tentativas | tempo | custo | resposta |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `examples/propose`, todo com a compacta mostrada (a sonda c) | the items as a list with the add field on top | 1 | 7,2 s | 0,038 USD | `repeat` sobre `todo:items`, molde com os ids da superfície mostrada (`todo_panel.item[{item}]`); sem filtro e sem contagens |
| 2 | a página Surface pela porta real, sem janela (o teste `#[ignore]` `the_real_claude_proposes_through_the_port`) | a mesma | 1 | 9,3 s (o ato do agente voltou `completed` 9,4 s depois do `executing`) | 0,040 USD | a mesma forma: a entrada em cima e a lista com o `repeat`; 14 nós; a superfície transformou no lugar |
| 3 | `examples/propose`, ide sem superfície mostrada (sobra usada para a pergunta que a sonda deixou) | the changed files as a list with stage and unstage on each, then the message and the commit button | 1 | 77,5 s | 0,102 USD | `repeat` sobre `changes:files`, molde com os ids do app (`changes.file[{item}]`, `.status` badge, `.stage` e `.unstage` botões), depois `review.message` e `review.commit`; superfície nova `review` |

somadas: 94 s e 0,18 USD. as três restantes não foram usadas.

### handoff da fase 3b-ii (2026-09-24)

goal: task-69 fase 3b-ii, a página Surface pede a tela ao Claude: a
pessoa (ou um agente pela porta) escreve o que quer, escolhe quem propõe
(as regras ou o Claude), e a tela pedida ao Claude nasce ou se
transforma no lugar, verificada, sem travar o app enquanto ele pensa,
com a espera e o custo visíveis e um jeito de cancelar; sem o `claude` na
máquina, a página continua pelas regras.

status: implementado. `cargo test -p showcase -p claude-model -p propose`
verdes, `./script/gate` verde com a perna urnaui e a perna wasm,
`regress check` verde, sobre a `main` @ cc908560.

#### commits

branch `task/69e-page-asks-claude`:

- 53535892 task-69e: a sonda da fase 3b-ii no arquivo da task
- 659a562e task-69e: claude-model, a cancel signal read in the run loop, and exists
- fe303efa task-69e: the Surface page chooses the proposer and asks the Claude off the app's thread, with the wait shown and a cancel
- 00228db1 task-69e: the intent field keeps a card's width beside the proposer, or has the line to itself
- e o commit do ADR e deste handoff

o briefing pedia o proponente e o pedido fora da thread num commit e a
espera e o cancelamento noutro; os dois vieram juntos no fe303efa porque
o mesmo `Pending` (o request aberto, o canal, o tempo, a tentativa, o
sinal de cancelar) carrega os três, e um commit sem ele não compilaria
sozinho. o commit do `claude-model` (o sinal e o `exists`) ficou
separado e compila sozinho. o item 4 do briefing (o prompt) não entrou:
a sonda não pediu.

#### a sonda

acima ("sonda da fase 3b-ii"). em uma linha: na base o `propose` da
página roda na thread de quem despacha e está `completed` quando
`dispatch` volta; `Started` mais `complete` pedem o id que `execute`
recebeu, um `Outcome` e o `now` do relógio da página, que o
`with_dispatcher` empresta de novo depois; o `fake-claude` segura a
thread pela chamada inteira (2,01 s num prazo de 2 s); a chamada real 1
veio com um `repeat` sobre `todo:items`, e o item 5 não disparou.
nenhuma condição de parada disparou.

#### como a página escolhe o proponente e descobre o programa

`surface_page.proposer` é um `comps::Select` projetado
(`Select::project`: role `select`, estado `selected` e `expanded`,
`constraints.options` com os nomes, ações `select` com um texto entre
as opções, `open` e `close`; aberto, uma linha `surface_page.proposer[i]`
por opção). uma pessoa clica e escolhe; um agente pela porta faz `select`
com `"claude"` ou `"rules"` (o teste da porta faz isso). o `propose` lê
`surface_page.intent:text` e `surface_page.proposer:selected`.

o programa é descoberto sem chamar o modelo por `ClaudeCli::exists`
(novo no `claude-model`): um programa nomeado com diretório é um arquivo
naquele caminho; um nome só é um arquivo em algum diretório do PATH.
`asking::Claude::find` faz `ClaudeCli::new()` (o `PLEV_CLAUDE` ou
`claude`) e guarda o cli quando `exists`; com ele o select oferece
`rules` e `claude` e escolhe `claude`; sem ele oferece só `rules`. o
`claude-model` entrou no `crates/showcase/Cargo.toml` na seção
`cfg(not(any(wasm32, android, ios)))`, com `ide`, `git` e `wire`: no wasm
o módulo `asking` compila sem thread e sem programa e `find` não acha
nada. nos testes do próprio crate (`cfg(test)`) o programa nunca é
procurado: o gate rodou nesta máquina, que tem o `claude`, e nenhum
teste chegou a ele; o teste que quer o falso o aponta
(`Claude::with(cli)`, `Proposing::set_claude`, ambos só em `cfg(test)`).

#### o trabalhador e o `Started`/`complete`

`page.rs` (o `Page`, o `Operable` da página, saiu de `proposing.rs` para
um arquivo próprio: o `proposing.rs` passaria de 850 linhas) e
`asking.rs` (o trabalhador e o `Pending`).

- `Page::propose(id, author)`: com `rules`, como na 3a, `Done` na hora.
  com `claude`: `Claude::start(id, intent, app, caps, current, policy)`
  clona o `ClaudeCli` com um `Cancel` novo, e leva para uma thread
  (`surface-page-claude`) o `Snapshot`, o `Capabilities`, o
  `Option<VerifiedIr>` da superfície mostrada e a `GenerationPolicy::by(author)`,
  todos por valor (o `Ask` empresta, então o trabalhador monta o `Ask`
  lá dentro). o `propose` devolve `Execution::Started` e guarda
  `Pending { id, intent, started, attempt, rx, cancel, cancelled }`;
- na thread: `propose_verified(&mut Llm::new(&mut Counted { cli, tx }), &ask)`,
  onde `Counted` é um `Model` que manda `Msg::Attempt(n)` antes de cada
  `complete` do cli; no fim manda `Msg::Done(Answer { proposed, attempts:
  calls.len(), took, cost_usd: spent_usd() })`. só `VerifiedIr` (ou
  `ProposeError`) cruza o canal;
- `SurfacePage::wake(dt, todo)` chama `Page::settle` a cada tick.
  `Pending::poll` lê o canal sem bloquear: `Attempt(n)` move a tentativa,
  a virada do segundo inteiro move o relógio, `Done` traz a resposta;
  um trabalhador que sumiu sem resposta vira `ProposeError::Model`.
  com a resposta, `Page::arrived`: `show(ir)` abre ou transforma
  (`SurfaceHost::transform` quando o id está aberto, `open` senão), e
  `todo.with_dispatcher(|d, _, now| d.complete(id, outcome, now))` com o
  `now` do relógio da página; o outcome leva `surface`, `nodes`,
  `attempts`, `seconds` (num) e `cost_usd` quando o filho disse. um erro
  (o modelo, o verifier, o host) termina em `fail(id, why, now)`;
- enquanto espera, `Proposing::wake` devolve `Wake::after(POLL)` (100
  ms) e `SurfacePage::wake` soma `Wake::now()` quando o `settle` moveu
  algo (a tentativa, o segundo, a resposta): a página pede frame uma vez
  por segundo e a cada tentativa, não a cada frame;
- só um pedido por vez: `propose` fica `enabled(false)` (o nó e a ação)
  enquanto há `Pending`, então um segundo `propose` é `Rejected(Disabled)`
  no check, antes do `execute`; o `execute` também recusa, por garantia.

o receipt do ato real pela porta: `[(requested, 92), (accepted, 92),
(executing, 92), (completed, 9414)]` em ms do relógio da página, com
`Outcome { surface: "todo_panel", nodes: 14, attempts: 1, seconds:
9.32, cost_usd: 0.039856 }`.

#### a espera e o cancelamento

- `surface_page.status` projeta `message` (texto) e `busy` (bool);
  enquanto pede, `attempt` (int, 1 a 3) e `seconds` (int, inteiros:
  a revisão muda uma vez por segundo, não a cada frame); depois de uma
  resposta do Claude, `attempt`, `seconds` e `cost_usd` (num). a mensagem
  durante a espera: `asking the Claude for "<intenção>": attempt 1 of 3,
  12 s`; ao terminar: `proposed by the Claude for "<intenção>" in 9.3 s,
  1 attempt, 0.040 USD`; recusa ou cancelamento em vermelho
  (`theme.colors.danger`). a legenda da metade gerada passa a "the same
  capabilities, proposed by the Claude for the intent above";
- o app segue: o teste com o falso lento faz um `toggle` do todo pelo
  dispatcher da página no meio da espera (`completed`), digita "buy" no
  campo da superfície (o texto chega ao app) e desenha frames; o
  `propose` está desabilitado e o `cancel` habilitado;
- `surface_page.cancel` é um `Button` projetado com `activate`,
  `enabled` só com `Pending`. o `execute` chama `Pending::cancel(actor)`,
  que levanta o `Cancel` (o `run` do `claude-model` lê o sinal a cada 10
  ms junto do prazo e mata o filho pelo pid, como no prazo) e anota quem
  cancelou; `Page::dispatch`, ainda com o dispatcher emprestado, faz
  `fail(id, "cancelled by human")` no mesmo empréstimo, e o `settle`
  faria o mesmo se o cancel viesse por outro caminho. a mensagem do
  trabalhador, quando chegar, cai num canal sem receptor. o teste confere
  que o pid do filho morre em menos de 5 s (na prática dezenas de ms) e
  que o request está `failed` com o motivo;
- no `claude-model`: `Cancel` (um `Arc<AtomicBool>`; um clone é o mesmo
  sinal, então o cli vai clonado para o trabalhador e a página fica com
  a alça), campo `pub cancel` do `ClaudeCli`, o `run` recebe `&Cancel` e
  confere antes de spawnar ("was cancelled before it started") e no
  laço ("was cancelled: killed (pid N)"); `ClaudeCli::exists`. a mudança
  no `run.rs` são 12 linhas.

#### o que o prompt diz e o que o Claude respondeu

o `propose::prompt` não mudou: a sonda (c) veio com o `repeat`. o que o
Claude respondeu está na tabela das chamadas reais. duas leituras para
depois: sobre a superfície compacta mostrada (que já repete
`todo_panel.items` sobre `todo:items`), ele manteve o molde e os ids da
superfície e tirou o que a intenção não pedia (filtro, contagens); do
zero, no ide, com "as a list" na intenção, repetiu sobre `changes:files`
e nomeou o molde com os ids do app (`changes.file[{item}]`), como na
corrida 4 da 3b-i (que, sem a palavra lista, enumerou os arquivos). a
palavra "list" na intenção ou um `repeat` na superfície mostrada bastam
para o `repeat`; os ids do app no molde continuam (o host trata esses nós
como do app, o caso da tela fixa em IR). pedir ids próprios da
superfície é uma mudança de prompt para outra fase, com sonda própria.

#### PNG e projeção

`SURFACE_PAGE_OUT=<dir> cargo test -p showcase --lib
surface_page::tests::claude::the_real -- --ignored --nocapture` escreve
`surface-claude.png` e `surface-claude.json` (1200x800, core). o PNG: à
esquerda o todo escrito à mão, igual; à direita, sob "the same
capabilities, proposed by the Claude for the intent above", o campo com
"the items as a list with the add field on top", o select `claude`, o
botão propose, a linha "proposed by the Claude for ... in 9.3 s, 1
attempt, 0.040 USD" com o botão cancel (desabilitado) na ponta, o
controle compact | two columns, e o card da superfície: o campo "new
todo" com o add desabilitado e os três itens com o checkbox "done" (o
terceiro marcado); sem chips e sem contagens. `cargo run -p showcase --
--render Surface` (a página em repouso, com o `claude` desta máquina
achado): 38 nós; em 390x844 hoff a linha fica sob o todo, o select e o
propose numa linha própria sob o campo. a projeção depois da proposta
real (31 nós; os 10 do todo como sempre; os da superfície e da linha,
forma lean, sem os reads):

```json
[
  { "id": "todo_panel", "role": "region", "label": "quick todo" },
  { "id": "todo_panel.card", "role": "group", "label": "todo", "parent": "todo_panel" },
  { "id": "todo_panel.entry", "role": "group", "parent": "todo_panel.card" },
  { "id": "todo_panel.input", "role": "text_input", "label": "new todo", "parent": "todo_panel.entry",
    "state": { "text": "" }, "actions": ["set_value", "submit"] },
  { "id": "todo_panel.add", "role": "button", "label": "add", "parent": "todo_panel.entry", "actions": ["activate"], "enabled": false },
  { "id": "todo_panel.items", "role": "list", "label": "items", "parent": "todo_panel.card" },
  { "id": "todo_panel.item[1]", "role": "group", "label": "1", "parent": "todo_panel.items" },
  { "id": "todo_panel.item[1].text", "role": "label", "label": "Absorb the plev demos into the showcase", "state": { "text": "Absorb the plev demos into the showcase" } },
  { "id": "todo_panel.item[1].done", "role": "checkbox", "label": "done", "state": { "checked": false }, "actions": ["toggle"] },
  { "...": "item[2] e item[3] iguais, o [3] com checked true" },
  { "id": "surface_page", "role": "region", "label": "propose a surface" },
  { "id": "surface_page.intent", "role": "text_input", "label": "intent", "parent": "surface_page",
    "state": { "text": "the items as a list with the add field on top" }, "actions": ["set_value"] },
  { "id": "surface_page.proposer", "role": "select", "label": "proposer", "parent": "surface_page",
    "state": { "selected": "claude", "expanded": false }, "constraints": { "options": ["rules", "claude"] },
    "actions": ["select", "open", "close"] },
  { "id": "surface_page.propose", "role": "button", "label": "propose", "parent": "surface_page", "actions": ["propose"] },
  { "id": "surface_page.cancel", "role": "button", "label": "cancel", "parent": "surface_page", "actions": ["activate"], "enabled": false },
  { "id": "surface_page.status", "role": "status", "label": "proposal", "parent": "surface_page",
    "state": { "message": "proposed by the Claude for \"the items as a list with the add field on top\" in 9.3 s, 1 attempt, 0.040 USD",
               "busy": false, "attempt": 1, "seconds": 9, "cost_usd": 0.039856 } }
]
```

#### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ cc908560 (a árvore da base em detached HEAD neste worktree, `cargo test --workspace -- --list`: 2409 testes, 13 ignorados) | 2396 passam, 13 ignorados | 82 passam, 1 ignorado (a perna não muda: o urnaui é outro workspace) | 2478 |
| depois, esta branch @ 00228db1 (`./script/gate`, saída 0) | 2404 passam, 0 falhas, 14 ignorados | 82 passam, 0 falhas, 1 ignorado | 2486 |

9 testes novos: 3 no `claude-model` (2 unitários, 1 com o falso), 5 no
showcase (4 com o falso, um deles pela porta real, e 1 do layout da
linha), e 1 ignorado (o `claude` de verdade). a contagem não cai. clippy
`-D warnings`, `cargo fmt --check` e o check wasm do showcase com `-D
warnings` verdes, dentro do gate, com a perna urnaui. `df -h /` antes do
gate: 22 GB livres. `cargo run -p showcase --example regress -- check`:
157 baselines conferidas, 157 iguais, nenhum frame desenhado (a página
Surface não tem baseline; nenhuma outra página mudou).

#### testes

- `crates/claude-model/src/lib.rs` (+2): um `Cancel` clonado é o mesmo
  sinal, um novo não; `exists` acha `sh` no PATH, o falso pelo caminho,
  e não acha um nome inexistente, um arquivo inexistente ou um diretório;
- `crates/claude-model/tests/cli.rs` (+1): uma chamada cancelada de
  outra thread mata o filho que dormia (o pid some), para bem antes do
  prazo de 30 s, o sinal levantado impede a chamada seguinte de começar
  (nenhum `prompt.2`), e um sinal novo deixa a chamada seguinte correr;
- `surface_page/tests/claude.rs` (4 + 1 ignorado), com o falso do
  `claude-model`: o pedido começa e a página segue (o select oferece
  rules e claude e escolhe claude; `Started`, `busy`, a mensagem, propose
  desabilitado e um segundo `propose` `Rejected(Disabled)`, um `toggle`
  do todo completa, uma tecla no campo da superfície chega ao app, a
  tentativa 1 aparece, o cancel pelo clique falha o request com
  "cancelled by human" e mata o filho, a superfície fica como estava); o
  Claude responde (o falso em `repair`: recusado e consertado, 2
  tentativas, 0,100 USD, `completed` com o outcome, a superfície em duas
  colunas no lugar, o texto e o caret do campo continuam no mesmo widget,
  a mensagem, os estados do status, os dois prompts que o falso recebeu,
  em repouso o wake não pede poll); sem programa o select oferece só
  rules (`select "claude"` é `InvalidParam`, as regras propõem na hora,
  o cancel é `Disabled`); pela porta real, sem janela, um agente escolhe
  o claude, escreve a intenção e propõe, o ato volta `completed` depois
  (o loop do shell tica a view), a superfície tem a forma, o status diz
  quem propôs e o custo, o receipt é do agente com `executing` e
  `completed` separados; e o ignorado com o `claude` de verdade;
- `surface_page/tests.rs` (+1): o proponente ao lado do campo quando o
  campo guarda `card_min_w`, sob ele senão (1500, 900 e 358 de largura),
  o cancel na linha da mensagem, nada fora da largura;
- os testes da 3a, da fase 2 e da task-95 passaram sem mudança (o
  `setup()` deles continua `SurfacePage::new`, que nos testes não acha
  programa).

#### validação executada

- `cargo test -p showcase -p claude-model -p propose`;
- `cargo clippy -p showcase -p claude-model -p propose --all-targets -- -D warnings`;
- `cargo fmt --check`;
- `RUSTFLAGS="-D warnings" cargo check --target wasm32-unknown-unknown -p showcase`;
- `./script/gate` e `cargo run -p showcase --example regress -- check`
  (abaixo);
- o teste ignorado com o `claude` real (chamada 2), `--render Surface`
  em 1200x800 core e 390x844 hoff, e o exemplo do `claude-model` para as
  chamadas 1 e 3. nenhuma janela foi aberta.

#### seção de ADR

`docs/ADR/proposers-propose-the-runtime-verifies.md`, seção nova no fim,
"addendum, 2026-09-24: the page asks the Claude off the app's thread
(task-69 phase 3b-ii, proposed)": contexto, decisão (o proponente como
capacidade, a descoberta do programa, o trabalhador pelo contrato do
dispatcher, um pedido por vez, a espera como estado, o cancel no laço do
`run`, o prompt que não muda), custo (as três chamadas), consequências e
o que evitar. para o orquestrador (o `docs/ADR/README.md` é proibido
aqui): no fim da linha do índice, algo como "; addendum 3b-ii
(proposed): a página Surface escolhe o proponente (`select` rules |
claude, claude só onde `ClaudeCli::exists`), pede ao Claude numa thread
com `Execution::Started` e termina por `complete`/`fail` no `wake`, a
espera e o custo como estado do status, `Cancel` lido no laço do `run`
mata o filho".

#### decisões

- o proponente é um `Select` do comps, não um `Tabs`: ele projeta
  `select` com um texto entre as opções, que é o que um agente manda
  pela porta; o `Tabs` da fase 2 (o controle da forma) continua sem
  projeção (o P2 dela);
- `claude` é o padrão onde o programa existe (decisão 13 do programa);
  nos testes do crate nunca é procurado, por `cfg(test)` em
  `Claude::find`. razão: o gate roda em máquinas com o `claude` e não
  pode chegar a ele;
- o IR é verificado no instante do pedido e não de novo na chegada. o
  que sumiu enquanto o Claude respondia (um item removido) não dá
  instância, e um pedido sobre ele o dispatcher recusa; nada não
  verificado é mostrado. re-verificar recusaria uma superfície inteira
  por um item que saiu, o que pareceu pior;
- o cancel falha o request na hora, sem esperar o trabalhador confirmar
  a morte do filho (que vem em até 10 ms); a mensagem tardia é
  descartada;
- `seconds` inteiro no estado e na mensagem: a projeção muda uma vez
  por segundo enquanto espera, não a cada frame;
- o campo de intenção guarda `card_min_w` ao lado dos controles ou tem
  a linha para si (uma intenção é uma frase). em 1200 de largura a
  metade tem 434 px e os controles vão para baixo;
- `Cancel` compartilhado pelos clones do `ClaudeCli`. razão: o cli vai
  clonado para o trabalhador e a página guarda a alça; um pedido novo
  ganha um `Cancel` novo;
- `Page` em `page.rs`. razão: o limite de 639 linhas; `proposing.rs`
  ficou com 555, `page.rs` com 320, `asking.rs` com 288,
  `surface_page.rs` com 564.

#### limitações

- um clique da pessoa no select troca o widget direto, sem dispatcher,
  como a tecla no campo de intenção desde a 3a; o `select` de um agente
  passa pelo dispatcher;
- o `Select` aberto vê o evento sozinho e é desenhado por último na
  camada padrão (`render_open`), por cima da superfície; com ele aberto,
  um clique fora fecha e não chega a mais nada;
- o `TextField` do comps quebra um texto comprido em mais de uma linha
  dentro do campo (uma intenção maior que o campo transborda); é
  comportamento do widget, não desta página;
- o botão cancel fica sozinho, desabilitado, na ponta da linha da
  mensagem quando nada é pedido;
- os segundos e a tentativa mudam a revisão da projeção uma vez por
  segundo durante a espera; os deltas da porta carregam isso;
- se o app fechar com um pedido em curso, a thread morre com o processo
  e o filho `claude` não é morto por nós: ele termina sozinho quando
  responde (a stdout dele já não existe) e a chamada é cobrada;
- a thread do trabalhador não é juntada (`join`); termina sozinha depois
  da resposta, do cancel ou do prazo de 180 s por tentativa;
- os testes de processo são só unix (o falso é um script sh); o teste
  da porta e o ignorado são só desktop.

#### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima); o
  `docs/CHANGELOG.md` (a página Surface pede ao Claude, com espera e
  cancel); na entrada `[crates.showcase]` do `docs/arc/arc.toml` a
  dependência `claude-model` (desktop) e os módulos `asking` e `page`;
  o comentário de `app/headless.rs` não muda;
- `Cargo.lock` mudou: o `showcase` passa a depender do `claude-model`
  (só desktop);
- `ClaudeCli::exists` não confere o bit de execução: um arquivo sem ele
  é "existe" e a chamada falha com "could not start";
- um `SESSION_VARS` novo de uma versão futura do `claude` passa ao filho
  (limitação da 3b-i, inalterada).

#### o que a fase 4 (o editor de vídeo) precisa

- uma página nova, com suas capacidades (`detect_silence`,
  `preview_segment`, `change_threshold`, `apply_cut`, `reject_cut`) e
  sem superfície mostrada no começo: a chamada 3 mostra que o Claude
  repete do zero quando a intenção diz lista, mas nomeia o molde com os
  ids do app. se a superfície de revisão deve ter ids próprios (para o
  host projetá-la como dela e transformá-la por `SemId`), o prompt
  precisa pedir isso, com sonda própria; hoje o host trata esses nós
  como do app;
- `media` no vocabulário do `uir` (fora do v1, dito na fase 1) para o
  vídeo e as prévias;
- a espera do Claude sobre um registro maior: o ide levou 77 a 89 s; um
  editor de vídeo com muitas capacidades pode passar de um minuto por
  tentativa e três tentativas passam de 180 s. o cancel existe; a
  mensagem de espera diz a tentativa e os segundos;
- a página desta fase reutiliza-se como está: `asking::Claude`,
  `Pending` e `Page::settle` não sabem nada do todo além do
  `TodoPage::with_dispatcher` e do `snapshot`; uma página nova precisa
  do mesmo empréstimo do seu dispatcher (um `with_dispatcher` seu) e de
  um `wake` que chame o `settle`;
- "corta os silêncios desse vídeo e depois quero revisar os cortes" pede
  que a capacidade `detect_silence` rode antes da superfície (o ffmpeg
  em subprocesso, como o urnaui): outro trabalho fora da thread, pelo
  mesmo `Started`/`complete`; o padrão desta fase serve.

## fase 4a: o app dos silêncios e a tela fixa

branch `task/69f-silences-app`, worktree próprio, sobre a `main` @
205d0e1d. o cenário da seção 4 da tese (o editor de vídeo) precisa, antes
da tela gerada (4b), do app com as capacidades de verdade e de uma tela
fixa bem desenhada para a mesma tarefa: é contra ela que o checkpoint b
compara a tela gerada. esta fase é esse app e essa tela, uma página do
grupo Apps Demo.

### sonda da fase 4a (antes do primeiro tipo, 2026-09-24)

o `ffmpeg` desta máquina é o 9.0.1 em `/opt/homebrew/bin/ffmpeg`. tudo
rodou num diretório temporário da sessão; um teste descartável do
showcase (`tests/silences_probe.rs`, rodado e jogado fora, nunca
commitado) fez a parte (b) que precisa do engine. nenhuma janela.

(a) o vídeo de teste, gerado por `lavfi` sem arquivo de entrada: 12 s,
`testsrc` 320x180 a 10 fps, áudio `sine` de 440 Hz a 44100 Hz, dois
trechos de silêncio (3 a 5 s e 8 a 10 s) pelo filtro `volume` com
`enable` na linha do tempo:

```text
ffmpeg -hide_banner -nostdin -y -v error \
  -f lavfi -i "testsrc=size=320x180:rate=10" \
  -f lavfi -i "sine=frequency=440:sample_rate=44100" -t 12 \
  -af "volume=enable='between(t,3,5)+between(t,8,10)':volume=0" \
  -c:v libx264 -preset ultrafast -crf 28 -pix_fmt yuv420p -c:a aac -b:a 64k \
  silences-test.mp4
```

162.908 bytes, os mesmos nas três gerações (o arquivo é determinístico);
`ffprobe`: h264 320x180 10/1, aac 44100 Hz mono, `duration=12.000000`;
81 a 83 ms de parede por geração. a detecção, `ffmpeg -hide_banner
-nostdin -nostats -i silences-test.mp4 -af silencedetect=noise=-30dB:d=0.5
-f null -`, o stderr inteiro (o stdout tem 0 bytes):

```text
Input #0, mov,mp4,m4a,3gp,3g2,mj2, from 'silences-test.mp4':
  Metadata:
    major_brand     : isom
    minor_version   : 512
    compatible_brands: isomiso2avc1mp41
    encoder         : Lavf63.1.101
  Duration: 00:00:12.00, start: 0.000000, bitrate: 108 kb/s
  Stream #0:0[0x1](und): Video: h264 (Constrained Baseline) (avc1 / 0x31637661), yuv420p(progressive), 320x180 [SAR 1:1 DAR 16:9], 59 kb/s, 10 fps, 10 tbr, 10240 tbn (default)
    Metadata:
      handler_name    : VideoHandler
      encoder         : Lavc63.1.101 libx264
  Stream #0:1[0x2](und): Audio: aac (LC) (mp4a / 0x6134706D), 44100 Hz, mono, fltp, 45 kb/s (default)
    Metadata:
      handler_name    : SoundHandler
Stream mapping:
  Stream #0:0 -> #0:0 (h264 (native) -> wrapped_avframe (native))
  Stream #0:1 -> #0:1 (aac (native) -> pcm_s16le (native))
Output #0, null, to 'pipe:':
  Metadata:
    major_brand     : isom
    minor_version   : 512
    compatible_brands: isomiso2avc1mp41
    encoder         : Lavf63.1.101
  Stream #0:0(und): Video: wrapped_avframe, yuv420p(progressive), 320x180 [SAR 1:1 DAR 16:9], q=2-31, 200 kb/s, 10 fps, 10 tbn (default)
    Metadata:
      encoder         : Lavc63.1.101 wrapped_avframe
      handler_name    : VideoHandler
  Stream #0:1(und): Audio: pcm_s16le, 44100 Hz, mono, s16, 705 kb/s (default)
    Metadata:
      encoder         : Lavc63.1.101 pcm_s16le
      handler_name    : SoundHandler
[Parsed_silencedetect_0 @ 0x71903dd40] silence_start: 3.018594
[Parsed_silencedetect_0 @ 0x71903dd40] silence_end: 5.01551 | silence_duration: 1.996916
[Parsed_silencedetect_0 @ 0x71903dd40] silence_start: 8.010884
[Parsed_silencedetect_0 @ 0x71903dd40] silence_end: 10.0078 | silence_duration: 1.996916
[out#0/null @ 0x71903c300] video:50KiB audio:1034KiB subtitle:0KiB other streams:0KiB global headers:0KiB muxing overhead: unknown
frame=  120 fps=0.0 q=-0.0 Lsize=N/A time=00:00:12.00 bitrate=N/A speed=2.07e+03x elapsed=0:00:00.00
```

os dois trechos: `silence_start: 3.018594`, `silence_end: 5.01551 |
silence_duration: 1.996916`; `silence_start: 8.010884`, `silence_end:
10.0078 | silence_duration: 1.996916` (o aac tem 18 ms de ataque e o
`d=0.5` conta a partir do limiar cruzado). a linha `Duration:
00:00:12.00` do mesmo stderr dá a duração sem um `ffprobe` à parte.
cinco corridas da detecção: 77 ms a primeira (o cache do disco), depois
41 a 43 ms. um áudio de 6 s calado a partir dos 4 s (o caso "silêncio
até o fim"): o ffmpeg 9 escreve `silence_end: 6 | silence_duration:
1.983039` no fim do stream, mas versões anteriores deixavam o
`silence_start` sem `silence_end`, e o parser cobre os dois.

(b) um frame do meio do primeiro trecho, `ffmpeg -hide_banner -nostdin
-v error -ss 4 -i silences-test.mp4 -frames:v 1 -f image2 -c:v png
pipe:1`: 12.978 bytes, `PNG image data, 320 x 180, 8-bit/color RGB`; com
`-vf "scale='min(160,iw)':-2"` aos 9 s: 6.954 bytes, 160x90. 39 a 42 ms
de parede por frame. o teste descartável leu os dois arquivos e chamou
`engine::gpu::image::load_image_bytes`: `12978 bytes -> ImageHandle {
id: 0, 320x180, atlas (1, 1) }` e `6954 bytes -> ImageHandle { id: 1,
160x90, atlas (513, 193) }`, sem GPU (o atlas é a metade de CPU).

(c) o `silencedetect` escreve no stderr, não no stdout: com o stderr
descartado o stdout tem 0 bytes; com o stdout descartado o stderr tem
as quatro linhas acima. o nível: `-loglevel quiet`, `error` e `warning`
não escrevem nenhuma linha com `silence` (o filtro loga em `info`);
`info` (o padrão) escreve as quatro mais a linha `Input #0`; `verbose`
escreve sete (duas linhas de configuração do filtro a mais);
`level+info` prefixa `[info]` depois da tag do filtro:
`[Parsed_silencedetect_0 @ 0xc1503dd40] [info] silence_start: 3.018594`.
a tag com o endereço muda a cada corrida e o parser não a lê: procura
`silence_start:`, `silence_end:` e `silence_duration:` em qualquer
lugar da linha. `-nostats` tira a linha de progresso (que, num vídeo
longo, vem com `\r`); `-nostdin` impede o filho de ler o terminal.

nenhuma condição de parada disparou: o ffmpeg gerou o vídeo, detectou os
dois trechos, decodificou o frame e o atlas o aceitou.

o que a sonda decide para o desenho:

- o trabalhador roda o ffmpeg com `-hide_banner -nostdin -nostats
  -loglevel info` na detecção e lê o stderr até o fim (a saída pode
  passar do buffer do pipe: um dreno numa thread, como o `run.rs` do
  `claude-model`); com `-v error` na decodificação e lê o png do stdout;
- o parser lê `Duration:` e as três chaves do `silencedetect` de
  qualquer linha, fecha um `silence_start` sem `silence_end` na duração
  e ignora o resto do texto;
- o vídeo de teste é gerado pelo mesmo trabalhador, pelo comando acima,
  em `std::env::temp_dir()/plev-silences/silences-test.mp4`, quando não
  existe;
- um frame por instante, em png de até 480 px de largura (o vídeo de
  teste fica em 320x180), carregado no atlas na thread do trabalhador;
  o `ImageHandle` cruza o canal.

### commits até a parada

- 4e3595ae task-69f: a sonda da fase 4a no arquivo da task
- ae7f113d task-69f: the silences model and the silencedetect parser,
  tested first (`crates/showcase/src/model/silences.rs`, `silences/parse.rs`,
  `silences/probe.stderr` como fixture e semente, `silences/tests.rs`,
  11 testes; a linha `pub mod silences;` em `model/mod.rs`)

### parada: registrar a página passa pelo `dispatch.rs`

problema: a página do item 3 não se registra só com os arquivos da
lista. uma página do showcase é uma linha de `PAGES` (`table.rs`, na
lista) com um `Pane` (`view/pages/mod.rs`), o estado dela em `Panes`
(`view/panes.rs`) e as rotas de `view/dispatch.rs`, que casa `Pane` de
forma exaustiva em sete lugares: `page_content_height`, `page_event`,
`page_wake`, `page_render`, `semantics`, `act` e `assist_focus`. um
`Pane` novo não compila sem um braço em cada um. o briefing diz para
parar antes de mexer no `dispatch.rs` "que está congelado no tamanho";
o estado da árvore: o `dispatch.rs` saiu do `FROZEN` na fase 1 (o
guard `crates/engine/tests/file_hygiene_guard.rs` não o lista mais) e
tem 338 linhas para o teto 639; os sete braços somam umas 30. o
trabalhador do ffmpeg e a página, escritos sem o registro, seriam código
morto para o clippy (`-D warnings`), então os commits 3 e 4 esperam a
decisão.

opções:

1. um `Pane::Silences` novo, como o `Pane::Surface` da fase 1 (a
   resolução daquela parada liberou os mesmos arquivos). fora da lista:
   `view/pages/mod.rs` (a variante), `view/panes.rs` (o campo e a
   construção), `view/dispatch.rs` (os sete braços; de 338 para umas
   370 linhas) e `view/input.rs` (dois braços, `handle_edit_key` para o
   Tab entre os sliders e os botões e `close_top_overlay` para o Escape
   tirar o foco; opcionais, sem eles a página fica sem ciclo de Tab
   próprio). impacto: quatro arquivos fora da lista, nenhum congelado; a
   página se projeta para o agent port, o leitor de tela e o `--render
   --semantics` pelas mesmas rotas das outras, e o `regress` a abençoa.
2. nenhuma página nesta fase: o modelo (feito), o trabalhador e o
   `SilencesPage` com a projeção e o `execute` provados por testes de
   unidade sobre a struct (sem passar pela `ShowcaseView`), e o PNG
   saindo do renderer headless num teste `#[ignore]`. impacto: nenhum
   arquivo fora da lista; sem `--render Silences`, sem baseline do
   `regress`, sem a prova pela porta real, e a página só aparece quando
   a rota for autorizada. o checkpoint b compara contra uma tela que
   ninguém abre no showcase.

recomendação: 1. é a página que o briefing descreve, com o contrato das
outras, e o motivo da parada (o arquivo congelado) não vale mais.

decisão necessária: qual opção; na 1, a autorização para os quatro
arquivos.

### decisões de desenho fixadas até aqui (para objeção cedo)

- a página abre com a semente da sonda (`SilencesModel::seeded`: o vídeo
  de teste com 12 s e os dois trechos, tudo pendente, o instante 0
  mostrado), como o todo abre com três itens. razão: o golden, a
  baseline do `regress` e o `--render` ficam iguais em toda máquina, com
  ou sem ffmpeg, e `apply_cut`, `reject_cut` e `apply_all` funcionam
  também no wasm. a detecção real substitui a semente com ids novos.
- o ffmpeg só roda depois de um pedido de um ator (`detect_silence` ou
  `preview_segment`), nunca ao abrir a página. o briefing diz "gerado
  quando a página abre"; o headless (`stage_page`) abre a página do
  mesmo jeito que a janela, e um processo começado ali deixaria o frame
  da baseline dependente do tempo do filho (120 passos de settle contra
  80 ms de geração) e escreveria em `/tmp` num render. o gancho que
  distingue a janela é o shell (`connect_git` em `app.rs`, proibido
  nesta fase); um `connect_silences` ali, noutra fase, faz a geração ao
  abrir. o vídeo é gerado pelo primeiro job que precisa dele, em
  `std::env::temp_dir()/plev-silences/silences-test.mp4`, e fica lá (só
  o sistema limpa); existindo, é reutilizado.
- o trabalhador roda um job por vez (gerar se preciso, detectar,
  decodificar um frame), cada um um processo filho numa thread com
  prazo e morte pelo pid (a forma do `run.rs` do `claude-model`, sem
  depender dele); o resultado volta por canal e o `wake` da página
  recolhe (`Wake::after(100 ms)` enquanto há job). `detect_silence` e um
  `preview_segment` cujo frame ainda não existe devolvem
  `Execution::Started` e terminam por `complete`/`fail` no `wake`, pelo
  `with_dispatcher` da página; os frames dos trechos (o meio de cada
  um) são pedidos pela própria página depois de uma detecção, um por
  vez, e ficam em cache por instante (`ImageHandle`, carregado no atlas
  na thread do trabalhador). nos testes do crate o ffmpeg nunca é
  procurado (`cfg(test)`, como `Claude::find`); o teste que quer o
  falso o aponta, e o `#[ignore]` aponta o real.
- os sliders são do widget: o arraste da pessoa muda o valor no `Slider`
  sem request, como a tecla no campo de intenção e o clique no select
  da 3b-ii; `set_value` de um agente passa pelo dispatcher. o modelo
  guarda as configurações com que os trechos foram detectados
  (`detected_with`), não as dos sliders; quando os sliders se afastam
  delas o status diz "detect again".
- os ids: `silences` (região; `cuts`, a lista dos ids na ordem),
  `silences.video` (`Role::Media`; `file`, `duration`, `shown`),
  `silences.threshold` e `silences.min_duration` (`Slider::project`:
  `value`, `set_value`, `increment`, `decrement`), `silences.detect`
  (botão; `detect_silence` lê `silences:cuts`, os dois `value`),
  `silences.cut[<id>]` (item de lista; `start`, `end`, `duration`,
  `decision`; `preview_segment` lê `silences.video:shown`, `apply_cut` e
  `reject_cut` leem `:decision`, cada um desabilitado quando já é a
  decisão), `silences.counts` (status; `total`, `applied`, `rejected`,
  `pending`, `cut_secs`), `silences.apply_all` (botão; `apply_all` lê
  `silences.counts:pending`, habilitado com pendentes) e
  `silences.status` (`message`, `busy`). os botões aplicar e rejeitar de
  cada linha são o afforance do item, não nós próprios (como o x do
  todo), para que a superfície da 4b ligue um botão a
  `silences.cut[i] apply_cut`.
- `apply_all` aplica só o que está pendente: o que alguém rejeitou fica
  rejeitado.
- a tela: dois cards em cima (o vídeo com a linha do tempo por baixo, os
  trechos marcados pela decisão e a agulha no instante mostrado; ao
  lado, a detecção com os dois sliders e seus valores, o botão detectar
  e o status) e, embaixo, o card dos trechos (as contagens em `Stat` na
  cabeça com o aplicar todos à direita; uma linha por trecho com o frame
  do meio, o intervalo, a duração, a decisão e os dois botões). estreito,
  os cards empilham. sem scroll interno: a página cresce e o scroll do
  gallery cobre.

### resolução da parada (orquestrador, 2026-09-24)

opção 1. liberados, só para o registro da página, `view/pages/mod.rs`
(a variante), `view/panes.rs` (o campo e a construção),
`view/dispatch.rs` (os sete braços; saiu do `FROZEN` na fase 1, o
briefing estava desatualizado) e `view/input.rs` (os dois braços
opcionais); `model/mod.rs` para o `pub mod silences;`. as decisões de
desenho acima aprovadas: a semente da sonda, o ffmpeg só depois de um
pedido de um ator, um job por vez com prazo e morte pelo pid,
`Started` -> `complete`/`fail` no `wake`, frames em cache por instante,
`cfg(test)` nunca procura o ffmpeg, o arraste dos sliders fica com o
widget. o resto como no briefing.

### handoff da fase 4a (2026-09-24)

goal: task-69 fase 4a, o app dos silêncios com capacidades reais e a
tela fixa bem desenhada, contra a qual o checkpoint b compara a tela
gerada (4b): uma página do grupo Apps Demo, operável por pessoa, agente
pela porta e leitor de tela pelo mesmo dispatcher.

status: implementado. `cargo test -p showcase` verde, `./script/gate`
verde com a perna urnaui e a perna wasm, `regress check` verde, sobre a
`main` @ 205d0e1d (a contagem abaixo).

#### commits

branch `task/69f-silences-app`:

- 4e3595ae task-69f: a sonda da fase 4a no arquivo da task
- ae7f113d task-69f: the silences model and the silencedetect parser, tested first
- 857cdd77 task-69f: a parada do registro da página e as decisões de desenho no arquivo da task
- 436c0647 task-69f: the ffmpeg worker, the Silences page and its projection, on the gallery's dispatcher
- d8641a40 task-69f: the golden projection and the regress baselines of the Silences page
- 72f49b00 task-69f: the projection carries seconds at ffmpeg's six decimals
- f305e67e task-69f: the frame's width cap lives with the desktop child
  (a constante estava morta no wasm sob `-D warnings`; a perna wasm do
  gate pegou)
- e o commit do ADR e deste handoff

o briefing pedia o trabalhador (3) e a página (4) em commits separados;
vieram juntos no 436c0647 porque o trabalhador sem a página é código
morto para o clippy do gate (`-D warnings`), e um commit dele sozinho
não passaria. o modelo (2) e a sonda (1) estão separados.

#### a sonda

acima ("sonda da fase 4a"). em uma linha: o ffmpeg 9 gera o vídeo de
teste por `lavfi` em 81 ms (162.908 bytes, determinístico), detecta os
dois trechos (`3.018594` a `5.01551` e `8.010884` a `10.0078`) em 41 a
43 ms escrevendo só no stderr no nível `info`, e decodifica um frame em
png para `pipe:1` em 39 a 42 ms (12.978 bytes, 320x180), que o
`load_image_bytes` aceita. nenhuma condição de parada disparou.

#### o modelo e o parser

`crates/showcase/src/model/silences.rs` (303 linhas),
`silences/parse.rs` (76), `silences/probe.stderr` (o stderr real da
sonda, fixture e semente) e `silences/tests.rs` (11 testes). `Segment`
(início, fim; `duration`, `middle`, `contains`), `Decision` (pending,
apply, reject; `name`), `Cut` (id, segmento, decisão), `Settings`
(limiar em dB e duração mínima em s; faixas `-60..0` e `0.1..3.0`,
`clamped`; o padrão é o da sonda, `-30`/`0.5`), `Video` (nome, duração),
`Counts` (total, aplicados, rejeitados, pendentes, segundos cortados),
`Applied` (quantos o "aplicar todos" aplicou e as contagens depois),
`SilencesModel` (`new`, `seeded`, `set_detected` com ids novos a cada
detecção, `decide`, `apply_all` só nos pendentes, `counts`, `show`
clamped ao vídeo, `cut_at`). `parse(stderr) -> Detected { duration,
segments }`: lê `Duration: hh:mm:ss.cc` e as chaves `silence_start` e
`silence_end` em qualquer lugar da linha (a tag do filtro muda de
endereço), fecha um start sem end na duração (ffmpeg 9 escreve o end no
fim do stream; versões anteriores não), descarta um start sem end e sem
duração, ignora um end sem start, um valor que não é número e o resto do
texto. nada de ffmpeg no modelo.

#### o trabalhador

`crates/showcase/src/view/apps/silences/worker.rs` (600 linhas, as do
desktop num módulo `child` com `cfg`; fora do desktop `find` e `spawn`
respondem "ffmpeg runs on the desktop only" sem thread nem processo).

- jobs: `Detect(Settings)` e `Frame { ms }`; um por vez, numa fila que
  não repete um job igual (`submit`); cada um numa thread
  `silences-ffmpeg` que roda o filho e devolve `Done` por canal; `poll`
  recolhe o que acabou e começa o próximo; uma thread que some sem
  resposta vira um erro ("the worker ended without an answer");
- o filho: `run(command, timeout)` na forma do `run.rs` do
  `claude-model`, sem depender dele: stdin nulo, stdout e stderr drenados
  em threads (um png passa do buffer do pipe), `try_wait` a cada 10 ms,
  no prazo `child.kill()` mais `wait` (morte pelo pid, o erro diz "did
  not finish in N s: killed (pid P)"), 1 s de graça pelo fim da saída;
  prazos: 30 s para gerar, 60 s para detectar, 20 s por frame;
- o programa: `PLEV_FFMPEG`, ou `ffmpeg` num diretório do PATH, um
  arquivo ali, sem rodar; procurado no primeiro job e guardado (o
  caminho ou o motivo). em `cfg(test)` nada é procurado
  (`Program::Nothing`, todo job falha com "ffmpeg is not looked for in
  the crate's tests"); um teste aponta o falso (`Worker::fake(detected,
  hold)`: a detecção da fixture e um frame sintético 32x18 por instante,
  segurando a resposta até `release` quando `hold`) ou o real
  (`Worker::real`, o teste ignorado);
- o vídeo: `std::env::temp_dir()/plev-silences/silences-test.mp4` (nesta
  máquina `/var/folders/.../T/plev-silences/silences-test.mp4`), gerado
  pelo primeiro job que precisa dele quando o arquivo não existe, pelo
  comando da sonda (`-f mp4`, escrito em `.part` e movido inteiro, para
  um filho morto não deixar meio vídeo a reutilizar); existindo, é
  reutilizado. ninguém apaga: só a limpeza do sistema do diretório
  temporário. nada commitado;
- a detecção: `-hide_banner -nostdin -nostats -loglevel info -i <vídeo>
  -af silencedetect=noise=<dB>dB:d=<s> -f null -`, o stderr inteiro
  parseado; sem `Duration` é erro. o frame: `-v error -ss <s> -i <vídeo>
  -frames:v 1 -vf scale='min(480,iw)':-2 -f image2 -c:v png pipe:1`, o
  png carregado no atlas na thread do trabalhador (`load_image_bytes`),
  o `ImageHandle` cruza o canal; 480 é o teto de largura (o vídeo de
  teste fica em 320x180).

#### a página

`crates/showcase/src/view/apps/silences.rs` (341 linhas: a struct, os
eventos, Tab/setas/Escape, o `wake`), `silences/semantics.rs` (605: os
ids, a projeção, o `execute`, o `dispatch` pelo `with_dispatcher` do
todo, `act`, `focus`, `ask`, `settle`, a mensagem do status),
`silences/layout.rs` (371: geometria pura), `silences/draw.rs` (344: a
cena e as bindings), `silences/tests.rs`, `tests/agent.rs` e
`tests/ffmpeg.rs`. registrada em `view/apps/mod.rs`, `view/pages/mod.rs`
(`Pane::Silences`), `view/pages/table.rs` (a linha, ícone `play`),
`view/panes.rs`, `view/dispatch.rs` (os sete braços: `content_height`,
`event`, `wake`, `render`, `semantics`, `act`, `assist_focus`) e
`view/input.rs` (`handle_edit_key` e `close_top_overlay`).

o dispatcher: a página é `Semantics + Operable` e despacha pelo
dispatcher do todo, emprestado por `TodoPage::with_dispatcher`, como a
Surface. razão: o agent port põe a sink uma vez só, nesse dispatcher
(`ShowcaseView::add_sink`), e casa o ato com os estágios por
`RequestId`; um dispatcher próprio da página seria surdo para a porta e
colidiria nos ids (os dois numeram de 1); a resolução da parada da fase
1 recusou "a porta ouvindo dois dispatchers". os receipts dos silêncios
ficam onde ficam os da galeria, numerados com eles.

a tela (1200x800 core, `cargo run -p showcase -- --render Silences --out
silences.png --semantics silences.json`; e 390x844 hoff com `--theme
hoff --width 390 --height 844`): dois cards em cima e um embaixo. o card
do vídeo: a caixa 16:9 com o frame do instante mostrado (ajustado, nunca
ampliado; em repouso a nota "no frame decoded yet: detect, or preview a
cut, runs ffmpeg"), o relógio `0:00.0 / 0:12.0` numa pílula, a linha do
tempo com cada trecho marcado pela decisão (âmbar pendente, vermelho o
que sai, apagado o que fica) e a agulha no instante mostrado, `0:00.0` e
`0:12.0` nas pontas; um clique numa marca faz `preview_segment`. o card
da detecção: "threshold" com `-30 dB` e o slider, "minimum silence" com
`0.5 s` e o slider, o botão "detect silences" (ícone play, a largura do
card) e o status. o card dos trechos: as contagens em quatro `Stat` (2
silences, 0 applied, 0 rejected, 0.0 s cut), "apply all" à direita, e
uma linha por trecho: o frame do meio (ou o instante num tile escuro
até haver frame), "0:03.0 to 0:05.0" sobre "2.0 s of silence", a
decisão num badge (pending, cut, kept) e os botões "apply" e "reject"
(o escolhido lê escuro, `Secondary`; o outro `Outline`); a linha do
trecho que segura o instante mostrado leva a barra accent. estreito, os
cards empilham e a linha põe o badge e os botões sob os textos quando
não cabem ao lado. depois de uma detecção real (o PNG do teste ignorado,
`silences-detected.png`): o frame do `testsrc` aos 4,0 s na caixa, a
agulha dentro da primeira marca, um frame por linha, o status "2
silences in 28 ms at -30 dB, 0.5 s minimum".

a projeção em repouso (10 nós; `--semantics`, forma lean, sem os reads):

```json
[
  { "id": "silences", "role": "region", "label": "silences", "state": { "cuts": ["silences.cut[1]", "silences.cut[2]"] } },
  { "id": "silences.video", "role": "media", "label": "silences-test.mp4", "state": { "file": "silences-test.mp4", "duration": 12.0, "shown": 0.0 } },
  { "id": "silences.threshold", "role": "slider", "label": "threshold (dB)", "state": { "value": -30.0 }, "constraints": { "editable": true, "range": [-60, 0], "step": 1 }, "actions": ["set_value", "increment", "decrement"] },
  { "id": "silences.min_duration", "role": "slider", "label": "minimum silence (s)", "state": { "value": 0.5 }, "constraints": { "range": [0.1, 3.0], "step": 0.1 }, "actions": ["set_value", "increment", "decrement"] },
  { "id": "silences.detect", "role": "button", "label": "detect silences", "actions": ["detect_silence"] },
  { "id": "silences.cut[1]", "role": "list_item", "label": "0:03.0 to 0:05.0", "state": { "start": 3.018594, "end": 5.01551, "duration": 1.996916, "decision": "pending" }, "actions": ["preview_segment", "apply_cut", "reject_cut"] },
  { "id": "silences.cut[2]", "role": "list_item", "label": "0:08.0 to 0:10.0", "state": { "start": 8.010884, "end": 10.0078, "duration": 1.996916, "decision": "pending" }, "actions": ["preview_segment", "apply_cut", "reject_cut"] },
  { "id": "silences.counts", "role": "status", "label": "2 silences, 0 applied, 0 rejected, 0.0 s cut", "state": { "total": 2, "applied": 0, "rejected": 0, "pending": 2, "cut_secs": 0.0 } },
  { "id": "silences.apply_all", "role": "button", "label": "apply all", "actions": ["apply_all"] },
  { "id": "silences.status", "role": "status", "label": "status", "state": { "message": "2 silences from the recorded run at -30 dB, 0.5 s minimum: detect runs ffmpeg here", "busy": false } }
]
```

os reads: `detect_silence` lê `silences:cuts`, `silences.threshold:value`
e `silences.min_duration:value`; `preview_segment` lê
`silences.video:shown`; `apply_cut` e `reject_cut` leem
`silences.cut[i]:decision` e cada um fica desabilitado quando já é a
decisão; `apply_all` lê `silences.counts:pending` e fica desabilitado
sem pendentes; os sliders leem o próprio `value`; `detect` fica
desabilitado enquanto o trabalhador está ocupado (um segundo
`detect_silence` é `Rejected(Disabled)` antes do `execute`). o golden
`crates/uir/fixtures/showcase-silences.snapshot.json` é essa projeção
inteira, mantida igual ao app por
`crates/showcase/tests/uir_fixture.rs::the_silences_projection_is_the_uir_fixture`
(`UIR_BLESS=1` regrava).

#### a medição

nesta máquina (apple m4, ffmpeg 9.0.1), pelo teste ignorado
(`SILENCES_OUT=<dir> cargo test -p showcase --lib silences::tests::ffmpeg
-- --ignored --nocapture`):

| o que | ffmpeg (ms, medido no trabalhador) | o request |
| --- | --- | --- |
| `detect_silence`, o vídeo ainda não existe (gera e detecta no mesmo job) | 194 | terminou 202 ms depois do `dispatch` |
| `detect_silence`, o vídeo existe (confere o arquivo e detecta) | 128 na primeira corrida, 28 a 29 nas seguintes | dentro de 10 ms da resposta (o poll do teste) |
| um frame (`-ss`, png, atlas) | 26 a 30, cada um dos três | um `preview_segment` de instante já decodificado: `completed` na hora |

na página em janela o `wake` olha o canal a cada 100 ms enquanto há
job, então um request termina até 100 ms depois da resposta e a tela
pede um frame só então; em repouso o `wake` é `idle` (o `stage_page`
diz `settled` em 2 passos).

#### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 205d0e1d (o handoff da 3b-ii sobre a mesma árvore) | 2404 passam, 0 falhas, 14 ignorados | 82 passam, 1 ignorado | 2486 |
| depois, esta branch @ f305e67e (`./script/gate`, saída 0) | 2428 passam, 0 falhas, 15 ignorados (109 suítes) | 82 passam, 0 falhas, 1 ignorado (6 suítes) | 2510 |

24 testes novos: 11 no modelo, 11 na página, 1 pela porta real e 1 o
golden; e 1 ignorado (o ffmpeg de verdade). a contagem não cai. clippy
`-D warnings`, `cargo fmt --check` e o check wasm do showcase com `-D
warnings` verdes, dentro do gate, com a perna urnaui (nenhuma perna
pulada). `df -h /` antes de cada gate: 17, 16 e 16 GB livres. o gate
rodou três vezes nesta branch: a primeira caiu num teste do ide alheio a
esta fase (abaixo, em limitações), a segunda na perna wasm por uma
constante morta fora do desktop (o f305e67e), a terceira inteira verde.
`cargo run -p showcase --example regress -- check`: 159 baselines
conferidas (as 157 de antes e as duas novas, `silences@core@1280x800` e
`silences@hoff@1280x800`, abençoadas com a imagem), 159 iguais, nenhum
frame desenhado. `view/dispatch.rs`: 338 linhas antes, 363 depois.

#### testes

- `model/silences/tests.rs` (11): o parser sobre a fixture real (a
  duração e os dois trechos, com as durações que o ffmpeg imprimiu), sem
  linhas de silêncio, silêncio até o fim com e sem duração, texto
  estranho e as chaves em qualquer lugar da linha; a semente; as
  decisões e as contagens; id desconhecido; aplicar todos deixa os
  rejeitados; o instante mostrado clamped; uma detecção nova com ids
  novos; as faixas;
- `view/apps/silences/tests.rs` (11): a projeção com toda capacidade e o
  que cada uma lê; aplicar por clique e rejeitar por `Actor::Test` no
  mesmo estado, a decisão tomada desabilitada e o "apply all" sem
  pendentes recusado, a tela seguindo; aplicar todos por clique;
  `detect_silence` `Started` com o falso segurando a resposta (o status
  ocupado, um segundo detect `Disabled`, um `reject_cut` no meio
  `completed`, o `wake` pedindo o poll), a resposta no `wake` com
  `completed` e o outcome, ids 3 e 4, o trecho velho fora da projeção,
  os três frames pedidos um por vez e desenhados (três nós `Image`),
  depois `idle`; `preview_segment` por clique na linha (`Started`, o
  instante mostrado, o frame no `wake`, `completed`), de instante já
  decodificado (`completed` na hora), e por clique numa marca; o caso
  stale com o delta; inerte atrás de um overlay; os sliders por
  `set_value`, `increment` e `decrement`, fora da faixa `InvalidParam`,
  no topo `Failed`, e o status dizendo "detect again" e levando as
  configurações para a detecção; Tab, setas e Escape; largo e estreito
  com as mesmas bindings e nada fora da largura; a galeria roteando a
  página (`semantics`, `act` como `Assistive`, `assist_focus`, Tab e
  Escape pelo chrome, e um detect sem ffmpeg terminando `failed` com o
  motivo no `tick`);
- `tests/agent.rs` (1): pela porta real, sem janela, um agente observa o
  escopo `silences` com as bounds, aplica um corte da revisão que viu
  (`completed`), tenta de novo e é `rejected`, pede `detect_silence` (o
  ato `completed` depois, quando o falso responde no `tick` do loop do
  shell) e observa os trechos novos; a galeria guardou o ato como do
  agente, no alvo e na ação certos;
- `tests/ffmpeg.rs` (1, `#[ignore]`): o ffmpeg real gera o vídeo,
  detecta os dois trechos (a 0,1 s dos da sonda), decodifica os três
  frames 320x180, detecta de novo com o vídeo já lá, preview de um
  trecho já decodificado, e escreve o PNG e a projeção com
  `SILENCES_OUT`; sem o programa diz "skipping" e passa;
- `crates/showcase/tests/uir_fixture.rs` (+1): o golden dos silêncios;
- os testes da galeria (`every_page_renders_a_scene`,
  `every_page_renders_in_short_viewports`, os invariantes de `PAGES`) e
  os 31 do `regress` passaram sem mudança.

#### validação executada

- `cargo test -p showcase` (196 na lib com 6 ignorados, 8 no main, 2 no
  `uir_fixture`, 5 no `view_source`, 31 no `regress`);
- `cargo clippy -p showcase --all-targets -- -D warnings`, `cargo fmt`;
- `./script/gate` (abaixo) e `cargo run -p showcase --example regress --
  check`;
- o teste ignorado com o ffmpeg real, duas vezes (com e sem o vídeo já
  gerado); `--render Silences` em 1200x800 core (com `--semantics`) e
  em 390x844 hoff, com e sem `--scroll 620`; `--render Surface` em
  390x844 para conferir que a sobreposição do blurb do header no card
  em largura de telefone é do shell e não da página. nenhuma janela foi
  aberta.

#### linha de ADR

`docs/ADR/the-silences-app-is-the-fixed-screen.md` (proposed): o
contexto (o cenário da tese e o checkpoint b), o modelo puro, a semente,
o ffmpeg só depois de um pedido de um ator (e por que não ao abrir), o
`Started`/`complete` no `wake`, o dispatcher da galeria, a projeção, a
tela, as consequências (a medição, o golden e as baselines), o que a 4b
precisa e o que evitar. para o orquestrador (o `docs/ADR/README.md` é
proibido aqui): uma linha no índice, algo como "the silences app is the
fixed screen (proposed): a página Silences do showcase é o app do editor
de vídeo da tese e a tela fixa do checkpoint b; o modelo puro com o
parser do `silencedetect`, a semente da sonda, o ffmpeg em processo
filho só depois de `detect_silence`/`preview_segment`, `Started` ->
`complete`/`fail` no `wake`, pelo dispatcher da galeria (o do todo)".

#### decisões

- a semente: a página abre com a detecção gravada da sonda, como o todo
  abre com três itens. razão: golden, baseline e `--render` iguais em
  toda máquina; as decisões funcionam no wasm;
- o ffmpeg só depois de um pedido de um ator, nunca ao abrir (o desvio
  do briefing, aprovado na resolução): o headless abre a página como a
  janela; um `connect_silences` no shell faria a geração ao abrir, noutra
  fase;
- o dispatcher do todo emprestado, não um próprio (acima);
- um job por vez, na ordem: gerar, detectar, um frame de cada vez; os
  frames que a tela mostra (o instante mostrado e o meio de cada trecho)
  são pedidos pela própria página depois de uma detecção e depois de
  cada frame que chega, e ficam em cache por instante (o erro também,
  para não repetir em laço);
- `apply_all` aplica só os pendentes: o que alguém rejeitou fica;
- o arraste dos sliders fica com o widget (como o campo de intenção e o
  select da 3b-ii); o modelo guarda `detected_with` e o status diz
  "detect again" quando os sliders se afastam;
- os botões aplicar e rejeitar de uma linha não são nós: a linha é o nó e
  eles a affordance, como o x do todo; assim a superfície da 4b liga um
  botão a `silences.cut[i] apply_cut`;
- segundos com seis decimais na projeção (`micros`): a diferença e a
  soma do que o ffmpeg imprimiu não carregam ruído binário para o golden;
- o vermelho é o que sai: o badge "cut" e a marca vermelha na linha do
  tempo são o trecho aplicado (a semântica de diff); "kept" verde é o
  rejeitado; âmbar o pendente;
- `Said` guarda o que a última corrida deu (semente, detectado com o
  tempo, falhou com o motivo); a mensagem do status é derivada, com o
  job em curso na frente ("detecting silences at ...", "decoding the
  frame at ...").

#### limitações

- o vídeo não é cortado de verdade: aplicar é uma decisão no modelo (o
  briefing deixou o corte para depois, se o checkpoint b pedir);
- o ffmpeg não roda ao abrir: até o primeiro `detect_silence` ou
  `preview_segment` não há frames, e a lista mostra o instante do meio
  num tile escuro;
- só um job por vez: com muitos trechos os frames chegam em fila (30 ms
  cada nesta máquina);
- o `find` do programa confere que é um arquivo, não o bit de execução;
- a thread do trabalhador não é juntada (`join`); um filho morto pelo
  prazo é morto e colhido, mas se o app fechar com um job em curso a
  thread morre com o processo e o filho termina sozinho;
- a mensagem de status muda a revisão da projeção a cada job que começa
  e acaba (uma detecção seguida de três frames são oito revisões);
- a sobreposição do blurb do header sobre o primeiro card em 390 px de
  largura é do shell (a página Surface mostra o mesmo) e não desta
  página;
- o teste `the_agent_asks_the_person_who_declines_and_it_stops` de
  `crates/ide/tests/lab.rs` (task-93) falhou uma vez no primeiro gate
  desta branch ("the person's click at 1701 ms is in the session": casa
  um evento pelo milissegundo) e passou três vezes sozinho e no gate
  seguinte; é sensível à carga da máquina e não toca em nada desta
  fase;
- os testes de processo são só desktop; o ignorado só com o ffmpeg.

#### dependências pendentes e riscos de integração

- para o orquestrador: a linha do índice de ADR (acima); o
  `docs/CHANGELOG.md` (a página Silences); na entrada `[crates.showcase]`
  do `docs/arc/arc.toml` os módulos `model::silences` e
  `view::apps::silences` (com `worker`, `semantics`, `layout`, `draw`) e
  a página; o `docs/catalog.md` se listar as páginas do Apps Demo;
- `Cargo.toml` e `Cargo.lock` não mudaram: nenhuma dependência nova (o
  `image` que decodifica o png já vinha pelo engine);
- a página depende de `TodoPage::with_dispatcher` e `receipt` (públicos
  no crate desde a 3a); quem mover o dispatcher para a `ShowcaseView` (o
  refactor que a fase 1 deixou) troca uma linha em `semantics.rs`;
- a fixture `probe.stderr` é do ffmpeg 9; outra versão que mude o
  formato das linhas muda o parser, não a semente.

#### o que a fase 4b (a tela gerada) precisa

- um componente de imagem no vocabulário do IR (o `media` ficou fora do
  v1): um nó que mostra o frame de um nó `media` num instante
  (`silences.video:shown`, ou o `start`/`middle` de um trecho), e um
  thumbnail por instância de um `repeat` sobre `silences:cuts`. a
  página decodifica um frame por instante e guarda o `ImageHandle`; a
  superfície pede instantes, nunca bytes. o host precisa de um jeito de
  a superfície ler o frame de um instante do app (hoje só a página tem o
  cache): um estado projetado com a chave do frame, ou um acesso do host
  ao cache pela `Dispatch`;
- as capacidades que a superfície de revisão liga: as contagens
  (`silences.counts:*`), o limiar (`silences.threshold` com `set_value`),
  `detect_silence`, um `repeat` sobre `silences:cuts` com o frame, o
  intervalo, `decision`, `apply_cut` e `reject_cut` de cada item, e
  `apply_all`; o prompt do proponente precisa dizer que os ids são do app
  (a chamada 3 da 3b-ii nomeou o molde com os ids do app);
- a comparação lado a lado: duas vistas `media` do mesmo vídeo em
  instantes diferentes (o original e o resultado) e as diferenças na
  linha do tempo são os trechos com `decision` apply: nenhuma capacidade
  nova, uma segunda leitura de `silences:cuts`;
- "corta os silêncios desse vídeo e depois quero revisar os cortes" pede
  `detect_silence` antes da superfície: o request `Started` termina no
  `wake`, e a superfície nasce quando ele completa; o padrão desta fase
  serve;
- cortar de verdade (um `export` com o ffmpeg tirando os trechos
  aplicados) não existe; se o checkpoint pedir, é uma capacidade nova
  com o mesmo trabalhador.

## fase 4b: a tela gerada dos silêncios

branch `task/69g-silences-surface`, worktree próprio, sobre a `main` @
4a7a8abf (o fechamento da 4a). a pessoa diz "corta os silêncios desse
vídeo e depois quero revisar os cortes" e a tela de revisão nasce sobre
o app dos silêncios, com frames de verdade; diz "quero comparar o
original com os cortes" e a mesma tela vira comparação, mantendo a
identidade do que continua; terminou, some. esta fase deixa as duas
telas (a fixa da 4a e a gerada) prontas para o checkpoint b, que é
decisão do brenner.

### sonda da fase 4b (antes do primeiro tipo, 2026-09-24)

um teste descartável do showcase (`tests/probe69g.rs`, rodado com
`--nocapture` e jogado fora, nunca commitado) fez as três partes sobre o
golden `crates/uir/fixtures/showcase-silences.snapshot.json` e sobre a
página real montada sem janela (`stage_page`). nenhuma janela, nenhum
ffmpeg, nenhum `claude`.

(a) o IR de revisão escrito à mão: um `card` com um `image` ligado ao
vídeo, quatro `stat` das contagens (`silences.counts:total`, `applied`,
`rejected`, `cut_secs`), uma linha com o `slider` do limiar
(`silences.threshold:value`, `set_value`), o botão de detectar
(`detect_silence`) e o de aplicar todos (`apply_all`), e uma `list` com
`repeat` sobre `silences:cuts` cujo item tem um `image` do corte, três
`text` (`start`, `end`, `duration`), um `badge` da `decision` e os botões
`apply_cut` e `reject_cut` sobre `silences.cut[{item}]`. três variantes
pelo `verify`:

- com `source` ligado a `silences.video:frame` e a
  `silences.cut[{item}]:frame` (o que a superfície precisa): recusado, 2
  erros, os dois da regra 4 (state): `the app declares no state
  silences.video:frame` e `no state of the app fits
  silences.cut[{item}]:frame`. só isso: nenhuma regra nova, falta o
  estado;
- com o que a projeção tem hoje (`silences.video:file` no vídeo e, como
  tapa-buraco, `silences.cut[{item}]:decision` no corte, os dois texto):
  ok; a ordem de foco é `review.threshold`, `review.detect`,
  `review.apply_all`, `review.cut[{item}].apply`,
  `review.cut[{item}].reject`;
- com um número (`silences.video:shown`, `silences.cut[{item}]:start`):
  recusado pela regra 4, `is num; a image's source takes text`.

o que o `image` de hoje liga: `source`, um estado de texto qualquer do
app (a regra 4 confere só o tipo). o que falta para um frame de um
instante: o instante. nada no vocabulário nem no verifier impede; o que
não existe é um estado de texto que nomeie o vídeo e o instante, e um
caminho do host até o cache de frames da página.

(b) o IR verificado (a variante que passa) materializado sobre a página
real, sem janela, desenhado em 900 px: 28 nós, `SceneNode::Image` no
frame: zero. o `image` do vídeo desenha o que desenha hoje: o retângulo
arredondado (`glass.surface`) com o label "the video" centrado, ligado
em `[308, 138, 860, 140]` (140 é `card_min_w / 2`, a altura de bloco da
fase 1). o `draw::image` de `crates/surface/src/draw.rs` é onde o host
precisaria perguntar o frame ao app antes de desenhar o label. e um
achado que a sonda não esperava: os itens saíram como
`review.cut[silences.cut[1\]]`, e os `text` do corte vazios. a projeção
da 4a põe em `silences:cuts` os ids inteiros (`"silences.cut[1]"`), não
as chaves (`"1"`), e o `repeat` põe o item onde está o `[{item}]`: as
ligações viraram `silences.cut[silences.cut[1\]]:start`, que não existe.
o todo projeta `todo:items` como `["1", "2", "3"]` (task-95), e a regra
das listas do proponente pede o mesmo ("os valores são as chaves dos
filhos do nó"). o app dos silêncios é que está fora da convenção, não o
`repeat` nem o verifier: a projeção passa a dar as chaves, o golden é
regravado, e o ADR da 4a (`the-silences-app-is-the-fixed-screen.md`, que
esta fase não pode editar) fica com "the ids in order" onde é "the keys
in order": uma linha para o orquestrador.

(c) as regras (`Rules`) sobre o golden, sem superfície atual:

- "the cuts as a list with the video and the counts": `"video" is not a
  word of the app; its words are applied, cut, duration, end, pending,
  rejected, sec, shown, start, total, value`. as regras não têm parte de
  vídeo, então "video" não é palavra de nada;
- "the cuts as a list with the counts" e "the cuts as a list": `a list
  (items) asked for, and the app declares no list of items`: a mesma
  causa de (b), os ids em vez das chaves;
- "the counts": verifica, com quinze `stat`: `duration` e `shown` do
  vídeo, o `value` dos dois sliders (os dois com o label "value"),
  `start`, `end` e `duration` de cada corte, e os cinco de
  `silences.counts`. a regra das contagens é "cada int ou num vira um
  stat"; sobre o todo isso eram dois. fica como está (fora do escopo), e
  a observação vai para o handoff;
- "everything": o mesmo que "the counts" (nenhuma outra parte);
- "compare the original with the cuts": `"compare" is not a word of the
  app` (e "original" também não seria).

o que falta nas regras: uma parte de vídeo/imagem achada pela estrutura,
as palavras da gramática para ela ("video", "compare") e, na lista, o
estado de frame de cada item virando `image` em vez de `text`.

decisão da forma do frame, pela sonda: a forma (i), aditiva na projeção.
o app projeta um estado de texto por frame que quer mostrar, com a chave
do frame no valor: o id do nó de mídia e o instante em segundos,
`"silences.video@4.017052"`. no vídeo, `original` (o frame do original
no instante mostrado) e `result` (o frame do resultado no mesmo instante:
dentro de um trecho aplicado, o fim do trecho; fora, o mesmo instante);
em cada corte, `frame` (o meio do trecho). o `image` liga `source` a um
deles; o host pergunta o frame ao app por um trait (`surface::Frames`,
`fn frame(&self, source: &str) -> Option<ImageHandle>`) passado por quem
desenha a superfície; sem frame, desenha o que desenha hoje, e a página
pede a decodificação no `wake` (um job por vez, em cache). a forma (ii)
(um componente `media` com `at`) mudaria o vocabulário, o schema, o
verifier e o prompt para dizer o que um estado de texto já diz, e a
regra do proponente achá-lo-ia pelo mesmo caminho: o valor nomeia um nó
de mídia do app e um instante, que é estrutura, não nome de app.

### handoff da fase 4b (2026-09-24)

goal: task-69 fase 4b, a tela gerada dos silêncios: uma superfície
materializada sobre o app da 4a mostra frames de verdade (o do instante
mostrado e um por corte), nasce de uma frase pelas regras e pelo Claude,
opera os cortes pelo dispatcher do app, vira uma comparação lado a lado
mantendo a identidade do que continua, e se desfaz quando a pessoa
termina. as duas telas (a fixa da 4a, a gerada) ficam prontas para o
checkpoint b, que é decisão do brenner.

status: implementado. `cargo test -p surface -p showcase -p propose -p
uir` verde, `./script/gate` verde com a perna urnaui e a perna wasm,
`regress check` verde (as duas baselines dos silêncios abençoadas, dito
abaixo), sobre a `main` @ 4a7a8abf (a contagem abaixo). duas chamadas
reais ao Claude, as duas verificadas.

#### commits

branch `task/69g-silences-surface`:

- 3fb83d34 task-69g: a sonda da fase 4b no arquivo da task
- 50a815e9 task-69g: the silences page projects the cuts by their keys, a
  frame ref per instant and the range (a projeção; o golden e as duas
  baselines regravados, nada visual mudou)
- af79fd3b task-69g: the frames trait and the image drawn from the app's
  frame (`crates/surface`; as duas fixtures; `tests/frames.rs`)
- 191aa93e task-69g: the generated surface beside the fixed screen, on
  the same dispatcher (a página; o bloco compartilhado da Surface;
  `dispatch.rs` e `input.rs`; as baselines com a linha de intenção)
- b30a5b81 task-69g: the rules find the media by structure and show a
  frame per item (`crates/propose`; o prompt; o exemplo com `--app
  silences` e `--shown`)
- e o commit dos adendos de ADR e deste handoff

o briefing pedia (2) o trait e (3) as fixtures com a página; a mudança da
projeção (as chaves em `silences:cuts`, os frame refs, o `range`) que a
sonda pediu veio antes, num commit só dela, para o golden ser o de
verdade quando o trait e as fixtures chegaram. as fixtures vieram com o
trait (o teste dele precisa de um IR sobre o golden).

#### a sonda

acima ("sonda da fase 4b"). em uma linha: o verifier não precisa de
nada (o `image` liga qualquer texto; faltava um texto que nomeie o
frame), o `image` de hoje desenha o retângulo com o label, as regras não
tinham parte de vídeo, e a projeção da 4a listava os cortes pelos ids
inteiros em vez das chaves, o que quebrava o `repeat` e a parte de lista.
nenhuma condição de parada disparou.

#### a forma do frame: (i), aditiva na projeção

o app projeta um estado de texto por frame que oferece, com o id do nó
de mídia e o instante em segundos no valor (`jobs::frame_ref`):
`silences.video:original` (o frame do original no instante mostrado),
`silences.video:result` (o frame do resultado no mesmo instante: dentro
de um trecho aplicado, o fim do trecho, o primeiro frame que fica; fora,
o mesmo instante) e `silences.cut[<id>]:frame` (o meio do trecho). um
`image` liga `source` a um deles. nada muda no vocabulário, no schema ou
no verifier; o prompt ganhou uma frase dizendo o que o `source` de um
`image` liga. a forma (ii) (um `media` com `at`) mudaria os quatro para
dizer o que um texto já diz, e as regras achariam o frame pelo mesmo
caminho (o valor nomeia um nó de mídia do app, achado pelo role).

junto, dois estados a mais que a sonda pediu: `silences:cuts` passa a
dar as chaves (`["1", "2"]`, como `todo:items`), e cada corte ganha
`range` (o texto do label, `0:03.0 to 0:05.0`), para a superfície mostrar
o intervalo sem formatar número. o golden e as baselines foram
regravados (commit 50a815e9); a projeção em repouso tem os mesmos dez
nós.

#### o trait

`surface::Frames`, `fn frame(&self, source: &str) -> Option<ImageHandle>`
(`crates/surface/src/host.rs`, ao lado do `Dispatch`), e `NoFrames`.
`Surface::render_with(c, rect, theme, &dyn Frames)` e
`SurfaceHost::draw_closing_with` perguntam o frame de cada `image` pelo
texto ligado no `source` quando desenham (`draw::picture`); `render` e
`draw_closing` são os mesmos com `NoFrames`, então a página Surface e os
testes de antes não mudam. um ghost de `image` pergunta o mesmo, e a
imagem some com o resto. `Surface::sources` (e o do host) lista os frame
refs que as imagens mostram: é o que o app decodifica. a superfície nunca
guarda bytes, nunca decodifica, nunca pede decodificação.

o desenho (`draw.rs`): a casca de antes e, dentro, a imagem ajustada e
centrada, nunca ampliada; sem imagem, a casca com o label. o tamanho é o
bloco (`card_min_w / 2`) ou, com `control_size`, a altura desse controle
em 16:9 (thumbnail numa linha; `FRAME_ASPECT`, geometria do conteúdo,
não token). o layout é o mesmo com e sem imagem, então um frame que
chega não move nada.

o app: `jobs::PageFrames` (o cache da página por instante, o ref
parseado em `instant_of`) implementa o trait; a página pede ao
trabalhador, no `wake` e nunca num frame, os refs que a superfície mostra
e ela não tem (`want_sources`), um job por vez, a falha em cache para
não pedir em laço. quando um frame chega, `settle` diz que moveu, a
página pede um frame, a superfície desenha.

#### as fixtures

`crates/surface/tests/fixtures/showcase-silences-review.ir.json` (id
`review`, "review the cuts"): o card com o `image` do vídeo
(`silences.video:original`), quatro `stat` das contagens, a linha da
detecção (o slider do limiar com `set_value`, detectar, aplicar todos), e
a `list` com `repeat` sobre `silences:cuts` cujo item é uma linha:
`image` do corte (`control_size` xl, `silences.cut[{item}]:frame`), uma
coluna (`review.cut[{item}].text`, flex 1) com o `text` do `range` e o
`badge` da `decision`, e os botões `apply_cut` e `reject_cut`.

`showcase-silences-compare.ir.json` (o mesmo id, "compare the original
with the cuts"): um `grid` sobre `field_min_w` com duas colunas, "the
original" com `review.video` (o mesmo id, `silences.video:original`) e
"with the cuts" com `review.result.video` (`silences.video:result`); os
`stat` de aplicados e segundos cortados (os mesmos ids); e a mesma lista
(`review.cuts`, o mesmo template) como as marcas da linha do tempo, cada
corte com o frame, o intervalo, a decisão e os dois botões. a linha da
detecção sai. as duas verificam sobre o golden real.

a primeira versão das fixtures punha o intervalo, a duração, o badge e os
botões numa linha só: a metade da página tem uns 430 px dentro do card, a
linha não cabia e os textos quebravam letra a letra. a coluna de textos
entre o thumbnail e os botões resolve; a duração por corte (`1.996916`,
sem formatação) saiu da revisão.

#### a página

`crates/showcase/src/view/apps/silences/generated.rs` (o bloco: o
`SurfaceHost`, a superfície aberta, a linha de intenção da Surface, o
botão de fechar, as metades) e `generated/page.rs` (o `Operable` sobre a
página inteira, o padrão do `page.rs` da Surface sobre este app:
propor pelas regras dentro do `execute`, pelo Claude fora da thread com
`Started` e o `settle` no `wake`, cancelar, e `close`). o resto em
`silences.rs` (os eventos, as teclas, o `wake`, `with_host`), `draw.rs`
(a linha, a tela fixa, a superfície com os frames), `semantics.rs`
(`projection`, `act`, `focus`) e `jobs.rs`.

a linha de intenção é a da página Surface, compartilhada: `proposing.rs`
e `asking.rs` viraram `pub(crate)` e a linha toma o nome da região como
único parâmetro (`surface_page` lá, `silences_surface` aqui); a Surface
passa `REGION` e não muda de resto. o `Operable` (`page.rs` da Surface)
não foi compartilhado: ele fala com o todo pelo `with_dispatcher` com o
todo dentro; o dos silêncios copia o padrão (~250 linhas) sobre este
app. dito e a favor de uma generalização depois, se um terceiro app
precisar.

o dispatcher: o mesmo caminho da 4a, o do todo emprestado por
`with_dispatcher`. a superfície chega ao app por `App { page, todo }`
(`surface::Dispatch`: `page.dispatch(req, todo)` e `page.snapshot()`, a
projeção da tela fixa); como o host mora dentro da página, `with_host`
o empresta para os eventos de uma superfície como o todo empresta o
dispatcher. o `dispatch.rs` mudou uma linha (`semantics` projeta a
página inteira, `projection()`) e o `input.rs` duas (digitar e Enter
chegam à página), ditos no commit. nada no `sem`, nada no `app/**`.

a projeção em dois níveis: `snapshot()` é o app (a tela fixa, o registro
que a superfície verifica e lê; o golden), `projection()` é a página
(o app, a superfície, a linha e o botão de fechar: o que o chrome e a
porta veem). o golden por isso mudou de teste: de
`crates/showcase/tests/uir_fixture.rs` (que só chega à página inteira)
para `silences::tests::the_app_projection_is_the_uir_golden`
(`UIR_BLESS=1` regrava).

onde as coisas ficam (`Generated::halves`): a linha em cima, o botão de
fechar à direita dela; sem superfície, a tela fixa na largura toda; com
uma, larga, a fixa na metade esquerda e a superfície na direita; estreita,
a superfície embaixo da fixa na altura natural. fechada, a fixa toma a
largura de volta.

`close`: `silences_surface.close`, botão com a ação `close`, habilitado
só com superfície aberta, pela tela (o clique) e pela porta (um `act`);
`SurfaceHost::close`, a transição de saída da fase 2 (os frames somem
com o resto), a linha volta a dizer nada.

#### as regras

`crates/propose`: uma quinta parte, mídia, achada pela estrutura: um nó
com role `media` ou `image` cujos estados de texto são frame refs (o
valor nomeia um nó do app com esse role, e um instante); cada estado é
uma vista (`original`, `result`); as palavras da parte são as do nó e as
chaves das vistas. palavras da gramática: `video`, `image`, `frame`
(`vídeo`, `imagem`, `quadro`, `media`, `preview`) para a parte;
`compare`, `comparar`, `comparison`, `versus`, `diff` para todas as
vistas lado a lado (um `grid` sobre `field_min_w`), e `compare` nomeia a
parte. sem `compare`, a primeira vista, ou as que uma palavra nomeia
pela chave ("the original", "the result"). na lista, o frame ref de um
item vira `image` (thumbnail, `control_size` xl) antes dos textos, nunca
`text`. a mídia vai no topo, antes das entradas e das listas.

uma mudança a mais, forçada: os números de um item de uma lista achada
não viram `stat` (são da lista). sobre os silêncios, "everything" fazia
um `stat` de `silences.cut[1]:start` com o id que o template faz para o
item 1, e o verifier recusava tudo (regra 2). o todo e o ide não têm
número em item: os 12 testes das regras e os 4 do llm passam sem
mudança. e uma observação que fica: `cut` é palavra da lista (o nome dos
itens) e de uma contagem (`cut_secs`), então "the cuts as a list with the
video and the counts" mostra a lista e só a contagem `cut_secs`, pela
regra da 3a ("a parte nomeada fica com o que a palavra nomeia"); "the
list with the video and the counts" mostra as cinco. e "the counts" sobre
os silêncios traz também `duration` e `shown` do vídeo e o `value` dos
dois sliders (os dois com label "value"): a regra "cada int ou num é um
stat" lê o valor de um slider como número que o app mostra. estreitar
isso (um número que uma ação do próprio nó recebe é do controle) é regra
de outra fase.

`crates/propose/tests/media.rs` (5 testes): a revisão (a vista do vídeo
em cima, `cut_secs`, a lista com `image` do frame, `range`, `decision` e
os dois botões, nenhum `text` de frame ref, nenhum `stat` de corte), a
comparação (as duas vistas lado a lado, `original`/`result` sozinhas), a
comparação sobre a revisão mantendo os ids da vista, da lista e do
template (e de volta, os mesmos ids), "everything" com a mídia em cima
e as contagens certas, e o todo e o ide recusando as palavras de mídia
com `Nothing` e sem `image` nas listas.

#### as chamadas reais ao Claude

pelo `cargo run -p claude-model --example propose -- --app silences`,
sonnet, prompt de 16.958 bytes (a comparação, com a revisão como atual
pelo `--shown`, 22.032 bytes). duas chamadas, as duas verificadas; as
duas de reserva não foram usadas.

| chamada | intenção | superfície atual | tentativas | tempo | custo | o que veio |
| --- | --- | --- | --- | --- | --- | --- |
| 1 (07:20:40) | "cut the silences of this video, then I want to review the cuts" | nenhuma | 2: a primeira recusada com um erro só (o root com o id da superfície, `cuts_review`, regra 2); a segunda verificada | 123,0 s + 14,6 s = 137,6 s | 0,155 + 0,055 = 0,210 USD | a revisão (abaixo), 6.169 bytes |
| 2 (07:23:33) | "compare the original with the cuts" | a da chamada 1 | 1, verificada | 40,4 s | 0,076 USD | a comparação (abaixo), 3.748 bytes |

a revisão da chamada 1, resumida: `cuts_review` "cut silences, then
review", um `stack` vertical com um `text` de título, um `image` "video
preview" ligado a `silences.video:result`, um `card` "detection settings"
com os dois `slider` (`set_value`) e o botão de detectar (com `busy`
ligado a `silences.status:busy`, um `SubmitButton`), um `text` do status,
cinco `stat` das contagens, o botão de aplicar todos, e uma `list` com
`repeat` sobre `silences:cuts` cujo item é um `card` com o `text` do
`range`, o `text` da `decision`, um `image` "cut preview frame" ligado a
`silences.cut[{item}]:frame`, e três botões (`preview_segment`,
`apply_cut`, `reject_cut`). o template tomou os ids do app
(`silences.cut[{item}]`, o mesmo que a 3b-ii viu no ide): o verifier
aceita e o host trata a raiz da instância como nó do app; os filhos são
da superfície. a frase nova do prompt bastou para ele ligar o frame de
cada corte ao ref, sem instrução de `repeat`. sem as regras: "then" e
"review" não são palavras do app.

a comparação da chamada 2: o mesmo id `cuts_review`, o título trocado, e
em cima um `split` horizontal 50/50 com `cuts_review.original` (novo,
`silences.video:original`) e `cuts_review.preview` (o id da chamada 1,
mantido, `silences.video:result`); o card da detecção, o status, as
contagens, o aplicar todos e a lista inteira com os mesmos ids. os ids do
que continua foram mantidos: sim, todos, a superfície inclusa. as regras
dão 17 nós para a mesma frase.

os dois IRs inteiros ficaram em
`/private/tmp/claude-501/-Users-nn-Dev-hoff-plevdev/ffe9cd40-0b24-414e-be8c-32dec2cf8b3d/scratchpad/claude-1.txt`
e `claude-2.txt` (a saída do exemplo), e o da chamada 1 em
`claude-1-review.ir.json`; nada foi commitado.

#### PNG e projeção

`SILENCES_OUT=<dir> cargo test -p showcase --lib
silences::tests::generated::the_real -- --ignored --nocapture` (o ffmpeg
real) escreveu `silences-review-real.png` e `silences-compare-real.png`
com os `.json`, em 0,68 s, em
`/private/tmp/claude-501/-Users-nn-Dev-hoff-plevdev/ffe9cd40-0b24-414e-be8c-32dec2cf8b3d/scratchpad/out/`;
o teste do gate (`the_page_draws_headless_with_the_review_and_the_comparison`,
o falso, com GPU) escreve `silences-review.png` e `silences-compare.png`
no mesmo `SILENCES_OUT`.

a revisão (1200x800 core): a linha de intenção em cima (o campo, o
select `rules`, propor desabilitado sem intenção, fechar; cancelar na
linha do status), a tela fixa na metade esquerda (os cards empilhados,
a caixa do vídeo com o frame do `testsrc` aos 0,0 s, a banda com as duas
marcas), e na direita o card gerado: o frame do vídeo em cima (320x180
ajustado em 140 de altura), `2 silences / 0 applied / 0 rejected / 0.0
seconds cut`, o slider do limiar com "detect silences" e "apply all", e
as duas linhas de corte, cada uma com o thumbnail do meio do trecho
(78x44), "0:03.0 to 0:05.0" sobre o badge "pending", "apply" e "reject":
6 `SceneNode::Image` no frame, 3 da fixa e 3 da gerada.

a comparação, depois de `preview_segment` e `apply_cut` no primeiro
corte: "the original" com o frame aos 4,0 s (o dígito 4 do `testsrc`)
ao lado de "with the cuts" com o frame aos 5,0 s (o fim do trecho
aplicado, o dígito 5), `1 applied / 1.996916 seconds cut`, e as duas
linhas com o primeiro corte em "apply" (o botão aplicar desabilitado): 7
imagens no frame (a quarta da gerada é o frame de 5,016 s). a tela fixa
ao lado mostra a agulha dentro da primeira marca, vermelha.

a projeção das duas (a página inteira, 44 nós cada; os ids dos cortes
são 3 e 4 porque a detecção real cunhou ids novos): o app (os dez de
sempre, com `original`, `result`, `range` e `frame`), a superfície
(`review`, o card, o `image` do vídeo, os `stat`, a linha da detecção
com `set_value`/`activate`, a lista com `review.cut[3]` e `review.cut[4]`
nomeados pelo item, cada um com o `image`, a coluna, o `range`, o badge
e os dois botões) e a linha (`silences_surface` com `intent`,
`proposer`, `propose`, `cancel`, `status`, `close`). na comparação,
`review.video` continua com o mesmo id e `review.result.video` nasce; a
lista é a mesma; `review.cut[3].apply` lê desabilitado. os `.json` estão
no diretório acima.

#### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ 4a7a8abf (o handoff da 4a sobre a mesma árvore) | 2428 passam, 0 falhas, 15 ignorados (109 suítes) | 82 passam, 1 ignorado (6 suítes) | 2510 |
| depois, esta branch @ b30a5b81 (`./script/gate`, saída 0) | 2444 passam, 0 falhas, 16 ignorados (111 suítes) | 82 passam, 0 falhas, 1 ignorados (6 suítes) | 2526 |

16 testes novos passando (3 em `surface/tests/frames.rs`, 7 em
`silences/tests/generated.rs`, 5 em `propose/tests/media.rs`, 1 em
`jobs.rs`, e o golden dos silêncios que saiu de `uir_fixture.rs` e entrou
em `silences/tests.rs`) e 1 ignorado a mais (o ffmpeg real das duas
formas). a contagem não cai. clippy `-D warnings`, `cargo fmt --check`
e o check wasm do showcase com `-D warnings` verdes, dentro do gate, com
a perna urnaui (nenhuma perna pulada; `../urna` resolve deste worktree).
`df -h /` antes do gate e depois: 16 GB livres, os dois. o gate rodou uma
vez nesta branch, inteiro verde, em 2 min 13 s (07:24:51 a 07:27:04).
`cargo run -p showcase --example regress -- check`: 159 baselines
conferidas, 159 iguais, nenhum frame desenhado (as duas dos silêncios
abençoadas duas vezes nesta branch: pela projeção em 50a815e9, nada
visual mudou, e pela linha de intenção em 191aa93e, com a imagem).
`view/dispatch.rs`: 363 linhas antes, 363 depois (uma linha trocada);
`view/input.rs`: duas linhas de rota a mais.

#### testes

- `crates/surface/tests/frames.rs` (3): a revisão sobre o golden com
  `sources` iguais aos três refs, zero imagem sem frames, três com o
  provedor falso (uma imagem 32x18 por ref, gerada em memória com
  `load_image_rgba`), cada uma dentro do rect do seu nó e nunca ampliada,
  o layout igual com e sem, o label do vídeo de volta quando só os
  cortes têm frame, a projeção (`image`, a instância nomeada pelo item,
  o `range`); a transformação em comparação mantendo a instância dos
  cortes (um press começado no botão de um corte completa depois e
  aplica o corte pelo dispatcher do app de teste), o `image` do vídeo
  deslizando, a linha da detecção saindo, quatro imagens, e o `close`
  desenhando os ghosts com as imagens até sumir; o todo sem `source`;
- `silences/tests/generated.rs` (8, um `#[ignore]`): a linha projetada
  com o app e fora do registro (o `close` desabilitado recusado como
  `Disabled`, o select só com `rules`); a revisão aberta ao lado (a
  fixa na esquerda, a superfície na direita; embaixo em estreito), os
  frames pedidos no `wake` (o worker ocupado, `POLL`) e desenhados
  quando chegam (6 imagens); aplicar pela gerada mudando a fixa (o
  receipt do `Human` em `silences.cut[1] apply_cut`, o botão da linha
  escuro, o badge e a contagem da gerada), rejeitar pela fixa mudando
  a gerada, e um `act` de `Assistive` num botão da gerada roteado para o
  app; a comparação mantendo a intenção digitada e a instância de um
  corte (o press em flight completa), o vídeo movendo de coluna, e o
  `close` por clique (fora da projeção, a fixa na largura toda, a saída
  em vários frames, o receipt `Completed`) e por request depois
  (`Disabled`); a galeria roteando a linha, as teclas, uma proposta pelas
  regras por Enter ("the counts"), Escape e o `close` por `act`; o
  Claude falso respondendo a revisão fora da thread (`Started`, o poll,
  `Completed` com `cost_usd`, a superfície aberta); a página sem janela
  com as duas formas sobre o falso (6 e 7 imagens, PNG com GPU); e o
  ignorado com o ffmpeg real;
- `silences/tests.rs` (+1, 2 mexidos): o golden do app; os dois de
  layout medindo a tela fixa no rect que as metades dão;
- `crates/propose/tests/media.rs` (5, acima); `words.rs` (+1 caso);
  `jobs.rs` (1: o frame ref e o instante);
- os testes de antes: os 6 da Surface page (`surface_page::tests`, 29),
  os 12 das regras e os 4 do llm, os do `surface` (todos), e o golden do
  todo, sem mudança.

#### validação executada

- `cargo test -p surface -p showcase -p propose -p uir` (e
  `claude-model` pelo gate);
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt`, o check wasm
  do showcase com `-D warnings`, dentro do gate, com a perna urnaui;
- `cargo run -p showcase --example regress -- check`;
- o ignorado do ffmpeg real (`the_real_ffmpeg_draws_the_review_and_the_comparison`),
  o headless do falso com `SILENCES_OUT`, o exemplo do `claude-model`
  duas vezes (a tabela). nenhuma janela foi aberta; nenhum teste do gate
  chama o `claude` nem exige o ffmpeg.

#### seções de ADR

- `docs/ADR/materialized-surfaces-from-verified-ir.md`, adendo datado
  "an image draws the frame the app holds (task-69 phase 4b, proposed)":
  o frame ref como forma do app, o trait `Frames`, `sources`, a imagem
  ajustada, a página, as consequências (as fixtures, as chaves na lista,
  o tamanho do frame falso, a largura da metade) e o que evitar;
- `docs/ADR/proposers-propose-the-runtime-verifies.md`, adendo datado
  "the rules show the media by structure (task-69 phase 4b, proposed)":
  a quinta parte, as vistas e o `compare`, o frame de item como imagem,
  os números de item fora das contagens, a mídia no topo, o prompt, o
  exemplo, e as consequências (o que cada frase dá, `cut` em duas
  partes, as contagens dos sliders);
- `plev-ir-typed-and-verified.md` e o schema não mudaram (forma (i)).
- para o orquestrador (fora do que esta fase pode editar): no índice
  `docs/ADR/README.md` os dois adendos; em
  `docs/ADR/the-silences-app-is-the-fixed-screen.md` a projeção diz "the
  ids in order" onde agora é "the keys in order", e os estados
  `original`, `result`, `range` e `frame` entram na lista da seção "the
  projection"; o `docs/CHANGELOG.md` (a superfície gerada na página
  Silences); no `docs/arc/arc.toml` os módulos `view::apps::silences::{jobs,
  generated, generated::page}` e o `tests/frames.rs` do `surface`.

#### decisões

- a forma (i) do frame, pela sonda (acima);
- `Frames::frame(&self, ..)` só consulta: a decodificação é pedida pela
  página no `wake` a partir de `sources`, nunca num frame (a regra da 4a
  de não rodar ffmpeg num frame vale para a gerada também), e nunca pela
  superfície;
- o tamanho de um `image` não depende do frame (o layout não pula quando
  o frame chega); `control_size` é a única alavanca do IR para um
  thumbnail;
- a linha de intenção compartilhada por nome de região, o `Operable`
  copiado (acima);
- a tela fixa fica na largura toda em repouso e cede a metade quando a
  gerada nasce (as baselines dos silêncios mudam pela linha em cima,
  nada mais);
- o golden é o app, a semântica da página é a página inteira;
- `result` como o fim do trecho aplicado que contém o instante mostrado:
  o único "resultado" que existe sem cortar o vídeo de verdade; fora de
  um trecho aplicado é o mesmo frame do original;
- os números de um item de lista saem das contagens (forçada pelo
  verifier, acima).

#### limitações

- o frame falso das provas é 32x18 e nunca é ampliado: no PNG do falso
  cada imagem é uma pílula pequena no meio da casca; o real é 320x180 e
  enche a altura;
- a metade gerada tem uns 430 px dentro do card em 1200 de largura: os
  textos de uma linha de corte quebram em duas linhas ("0:03.0 to" /
  "0:05.0"), e um IR do Claude com mais coisa numa linha quebra pior. o
  motor de layout dá a um `text` o que sobra e o texto quebra letra a
  letra em vez de o container transbordar: comportamento do engine, fora
  desta fase;
- a revisão do Claude nomeou o template com o id do app
  (`silences.cut[{item}]`), como a 3b-ii já tinha visto no ide: o host
  aceita e a raiz da instância fica sendo o nó do app; pedir ids
  próprios é mudança de prompt de outra fase;
- o `Operable` da página é uma cópia adaptada do da Surface (~250
  linhas); dois apps, duas cópias;
- "the cuts as a list with the video and the counts" pelas regras dá só
  `cut_secs` como contagem (a palavra `cut` nomeia essa contagem); e "the
  counts" traz os valores dos sliders como stats;
- as contagens de uma detecção real cunham ids novos (3 e 4), então os
  `.json` da projeção real não são o golden;
- o `close` não confirma: uma superfície com texto digitado num campo
  some com ele (nenhuma fixture tem campo);
- o clippy com `-D warnings` recusou um `mod support` sem `allow` no
  teste novo do `propose` (os outros binários usam tudo dele); ficou o
  `#[allow(dead_code)]` com o comentário, como nos testes do `surface`.

#### dependências pendentes e riscos de integração

- para o orquestrador: as linhas de ADR e o CHANGELOG acima; o
  `arc.toml`;
- `Cargo.toml` e `Cargo.lock` não mudaram: nenhuma dependência nova;
- a Surface page depende de `Proposing::new(REGION)` (uma linha
  mudada); quem mover o dispatcher para a `ShowcaseView` troca duas
  linhas aqui (`App::dispatch` e `Page::dispatch`);
- a projeção dos silêncios mudou (as chaves, quatro estados): um agente
  que lia `silences:cuts` como ids lê chaves agora;
- as duas baselines dos silêncios mudaram duas vezes nesta branch (a
  projeção, depois a linha em cima); `regress check` verde.

#### o que a 4c (o vídeo da demo) e o checkpoint b precisam

- as duas telas estão prontas e lado a lado na página Silences: a fixa
  (a 4a) e a gerada, pela mesma frase da tese no campo de intenção com
  `claude` escolhido (137 s na chamada 1; a página mostra a espera e
  cancela), ou pelas regras em menos de um milissegundo com "the list
  with the video and the counts" e "compare the original with the cuts";
  o `close` desfaz. o roteiro: abrir a página, "detect silences" (o
  ffmpeg gera o vídeo e detecta), a frase, a revisão nasce ao lado com os
  frames chegando, aplicar um corte em qualquer das duas telas, "compare
  the original with the cuts", a comparação com o resultado diferente
  onde há corte aplicado, fechar;
- para gravar (task-63, se integrada): a janela real só com
  `PLEV_BACKGROUND=1`; sem janela, os PNGs do teste ignorado e do falso;
- o que o checkpoint pode pedir e não existe: o corte real (`export`),
  um `result` que reflita mais de um trecho aplicado (hoje é o instante
  mostrado dentro do trecho que o contém), ids próprios no template do
  Claude, e a linha de intenção sem o botão de fechar quando nada está
  aberto.

## fase 4c: o showcase grava a sessão e a demo fica gravada

branch `task/69h-recorded-demo`, sobre a `main` em `a9edcbaa`.

### sonda da fase 4c (antes do primeiro tipo, 2026-09-24)

(a) o showcase não grava. por leitura: `grep -rn "rec::\|Recorder\|Recording\|PLEV_RECORD"` em `crates/showcase/{src,Cargo.toml,examples}` não acha nada (o ide tem os três em `record.rs`, `recording.rs` e `lib.rs`); `crates/showcase/Cargo.toml` não depende de `rec` (chega a ele só por `ide`). por binário: `strings target/debug/showcase | grep -c PLEV_RECORD` dá 0 (o `ide`, 1) e "plev/sessions" não aparece. por um teste descartável: `PLEV_RECORD=1 showcase --render Silences --semantics probe-a.json` escreveu o json (17 nós, revisão `d3c64aa094fd9c52`) e `$TMPDIR/plev/sessions/` ficou sem nenhum `showcase-*` antes e depois.

os cinco pontos do shell do ide, por linha, e o ponto equivalente no shell do showcase (linhas da base):

| o que | onde o ide grava | onde o showcase tinha o ponto e não gravava |
| --- | --- | --- |
| sink no dispatcher único | `crates/ide/src/main.rs:530` `recording::start` -> `recording.rs:16` `attach` -> `record.rs:129` `bench.add_sink(rec.sink())` | `crates/showcase/src/view/dispatch.rs:316` `add_sink` (o dispatcher do todo, emprestado à Surface e aos silêncios); só a porta chamava (`dispatch/agent.rs:56`) |
| a porta: quem observou o quê, e o estado depois de um ato | `main.rs:188` `ide::agent::serve_recorded(port, bench, compositor, recording)` -> `agent.rs:57` (`Served.recorder`, `Host::observed` em 112, o estado em 69) | `app.rs:179` `serve_agent` -> `view.serve_agent(port, compositor)` -> `dispatch/agent.rs:21` (`Served { view, compositor }`, sem `observed`, nada gravado) |
| o estado depois de clique ou tecla que mudou algo, depois de um resultado do git, antes do frame | `main.rs:258` (git), `371` e `390` (`changed()` depois do clique), `keys.rs` (tecla), `record.rs:143` (antes do frame) | `app.rs:285` (git: só `invalidate`), `410` e `421` (cliques: só `invalidate`), `341` a `363` (teclas: só `invalidate`) |
| cada frame com as ligações | `main.rs:441` `recording.frame(&bench, &compositor)` depois do `render_frame` | `app.rs:460` `render_frame` e a árvore da acessibilidade em `470`; nada gravado |
| os inputs sem caractere | `main.rs:369` (esquerdo), `388` (direito), `keys.rs:36` (`text`), `keys.rs:50` (`key`) | `app.rs:393` (esquerdo), `415` (direito), `334` (teclado): nada gravado |
| salvar: atalho, fechar, pânico | `keys.rs:82` (`cmd-shift-s`), `main.rs:463` (`exiting`), `recording.rs:17` (`catch_panics`) | `app.rs:327` `CloseRequested => event_loop.exit()` sem `exiting`; sem atalho; sem hook |

o relógio: o ide grava em `bench.now()`, o relógio dos receipts. no showcase os receipts levam o relógio do dispatcher do todo (`apps/app/semantics.rs:259` e `279`, `clock.elapsed()`), lido só dentro de `with_dispatcher` (`pub(crate)`, `&mut self`); a `ShowcaseView` não tinha `now()`.

(b) o showcase tem um modo servido sem janela: `ShowcaseView::serve_agent(&mut AgentPort, &Compositor)` sobre um `AgentPort::open_with` num diretório, a forma dos testes de `view/dispatch/agent.rs` e de `silences/tests/agent.rs` (este thread é o shell: serve, dá o tick, desenha num `Compositor` sem GPU; um thread é o agente pelo `wire::Client`). o roteiro rodou assim, por `crates/showcase/examples/demo.rs`, sem janela nenhuma e sem `PLEV_BACKGROUND`: 3,3 s, 17 observações, 10 requests, 101 frames desenhados. três coisas apareceram na corrida: as regras recusam a frase da tese ("then" e "review" não são palavras do app, o que a 4b já tinha dito), então pelas regras a revisão é "the list with the video and the counts" e a frase da tese fica para o Claude; em 1200 por 800 uma linha da revisão das regras (thumbnail, dois textos, dois botões) é mais larga que a metade e o botão de rejeitar fica em x = 1232..1341, fora da janela, onde um clique não chega (pelo id o agente chega), então a demo desenha em 1600 por 1000 (`--width`, `--height`); e a rolagem da página não resolve isso (é horizontal), o exemplo diz o retângulo no erro. com janela de verdade não foi preciso; se um dia for, `PLEV_BACKGROUND=1 PLEV_AGENT=1 PLEV_RECORD=1 cargo run -p showcase -- Silences` abre a janela por baixo e o mesmo roteiro cabe em `plev observe`/`act` contra `--app showcase`.

(c) o tamanho: a sessão da demo (3 s, 100 frames, 73 eventos) tem 186.837 bytes: eventos 81.078 (4 keyframes de até 51 nós, 10 passos de mudança, 40 estágios, 17 observações, 2 inputs), índice 6.812, chunks 67.984 (100 frames em 12 chunks, uns 680 B por frame), assets 21.085 (134: estilos de texto e glifos; 3 imagens só com o tamanho), ligações 9.600. o `rec` guarda de imagem `Asset::Live(LoweredAsset::Image(handle))`, o handle do atlas, e persiste só largura e altura (`Persisted::images_without_pixels = 3`): os pixels de um frame do vídeo nunca entram no arquivo, por construção (a retenção de pixels ficou fora da task-63 fase d, sem task). então 30 s custam o que os frames desenhados e os eventos custam, não o tempo: render on demand, a página parada não desenha; mil frames seriam uns 700 KB. a fixture cabe no repositório (187 KB, limite de 2 MB) e fica sem os pixels dos frames: leva a cena de cada frame (retângulos, textos, as imagens com tamanho) e as ligações; `Session::scene_at` lê a cena sem nó de imagem, `to_bytes` conta as três imagens sem pixels.

### handoff da fase 4c (2026-09-24)

```text
Goal: task-69 fase 4c: o showcase grava a própria sessão como o ide
      (PLEV_RECORD=1, debug) e a demo do editor de vídeo, dirigida pela
      porta do agente, fica gravada num .monster de fixture que o
      plev-inspect lê, com os frames
Status: pronto para validação do orquestrador
```

#### commits

- 29efeb93 task-69h: the probe of phase 4c, before the recording
- fb57e228 task-69h: the showcase records its session as the ide does
- 8c0652e7 task-69h: the demo's script through the port, with no window
- 94942d80 task-69h: the demo recorded by the rules and by the Claude, and the test that reads it
- 82db3ddf task-69h: the demo example split under the file limit
- o commit deste texto: o adendo do ADR e o handoff

#### a sonda

está na seção "sonda da fase 4c" acima: (a) o showcase não gravava
(leitura, binário, uma corrida descartável), com os cinco pontos do ide
por linha e o ponto equivalente do showcase; (b) o modo servido sem
janela existia (`ShowcaseView::serve_agent` sobre um `AgentPort`) e o
roteiro rodou por ele sem `PLEV_BACKGROUND`; (c) a sessão da demo pesa
187 KB, os frames custam uns 680 B cada e as imagens entram só com o
tamanho.

#### a gravação (os cinco pontos)

`crates/showcase/src/app/record.rs` (novo): `ENV`, `enabled`,
`default_dir`, `SAVE_KEYS`, `is_save_key`, `Recording` (`from_env`,
`new`, `recorder`, `dir`, `attach`, `state`, `frame`, `input`,
`next_path`, `save`, `catch_panics`) e os ganchos do shell (`App::record`,
`record_state`, `record_key`, `record_pointer`, `record_frame`,
`save_session`). o relógio é o dos receipts: `ShowcaseView::now(&mut self)`
em `view/dispatch.rs`, o `now` do dispatcher do todo lido por
`with_dispatcher` (daí o `&mut`; um `TodoPage::now(&self)` seria uma linha
em `apps/app/semantics.rs`, fora da lista desta fase).

1. sink: `Recording::attach` põe `rec.sink()` no dispatcher único
   (`ShowcaseView::add_sink`) e grava o estado agora, o primeiro keyframe;
   `App::record` chama em `run()`, logo depois de `App::new()`, e instala o
   hook de pânico.
2. porta: `ShowcaseView::serve_agent_recorded(port, compositor,
   Option<&Recorder>)` em `view/dispatch/agent.rs`; `Served` ganhou
   `recorder` e `Host::observed` (ator, escopo e revisão, no relógio da
   galeria); depois de um drain que rodou um ato, o estado. `serve_agent`
   continua com a mesma assinatura, delegando com `None`; `App::serve_agent`
   passa `self.recording.as_ref().map(Recording::recorder)`.
3. estado: `App::changed()` (o estado quando grava, e o frame) onde havia
   `invalidate()` depois de algo que mudou: um resultado do git na tela
   (`UserEvent::Wake`), o leitor de tela (`UserEvent::Access`), Escape que
   fechou um overlay, tecla do editor, tecla nomeada, caractere, clique
   esquerdo e direito; e antes de cada frame (`Recording::frame` grava o
   estado primeiro). estado igual custa um diff e não guarda nada; página
   sem projeção (Button, Card) não guarda nada.
4. frame: `App::redraw` (o corpo de `RedrawRequested`, movido de `app.rs`
   para `app/pace.rs`) chama `record_frame()` depois de `render_frame` e da
   árvore da acessibilidade: `rec.frame(now, compositor)`, com as ligações.
5. input: `record_key(&Key)` antes do match do teclado (o atalho de salvar
   consome a tecla; qualquer outra entra como `key` ou `text` pelo tipo,
   nunca o caractere); `record_pointer(state, x, y)` antes de `handle_event`
   no botão esquerdo (press e release) e no direito (press): tipo, posição
   e o controle sob o ponteiro (`Compositor::hit`).

salvar: `exiting` (a janela fechada, ou Escape sem overlay) ->
`save_session("close")`; `cmd-shift-s` (`ctrl-shift-s` fora do mac) só
enquanto grava (a galeria não tem keymap: `is_save_key` confere `s` com
shift e o modificador primário); o hook de pânico do `rec`. arquivos
`showcase-<pid>-<n>.monster` em `$TMPDIR/plev/sessions` (0700), o caminho
no log (`run()` liga `warn,showcase=info` quando grava). sem a variável:
`from_env()` é `None`, nenhum diretório, nenhuma tecla, um `Option` por
frame e por input. `app.rs`: 595 linhas antes, 611 depois (o `redraw` foi
para `pace.rs`, 66 -> 120 com testes); `dispatch.rs` 363 -> 371;
`dispatch/agent.rs` +30 na produção. wasm e mobile: nada disso compila lá
(`record` e `rec` sob o cfg do desktop; o check wasm com `-D warnings`
verde no gate).

#### o roteiro (o programa e cada passo)

`crates/showcase/examples/demo/{main,shell,agent}.rs`:

```text
PLEV_RECORD=1 cargo run -p showcase --example demo -- [--out f.monster]
    [--review rules|claude] [--compare rules|claude] [--name demo]
    [--width 1600] [--height 1000]
```

sem janela: este thread é o shell (`Shell`: serve a porta com
`serve_agent_recorded`, `tick` pelo tempo real, `render` num `Compositor`,
`Recording::frame` a cada frame; `click` faz a pessoa; `settle` no fim), o
outro é o agente (`wire::Client`, `hello demo`, todo ato `from_revision`
de uma observação da página inteira feita logo antes). a gravação com
`Limits { max_age: 600 s, max_bytes: 64 MiB }` (a corrida inteira, não os
20 s do shell), salva mesmo quando o roteiro falha no meio. os passos,
como ficaram na fixture pelas regras (requests 1 a 10):

1. observe com bounds;
2. `detect_silence` em `silences.detect` (request 1): o ffmpeg gera o
   vídeo se faltar e detecta (61 ms; os cortes viram 3 e 4); a página pede
   os frames do instante e dos cortes, e o agente observa
   `silences.status:busy` até ficar ocioso duas vezes seguidas;
3. `select` "rules" em `silences_surface.proposer` (2), `set_value` "the
   list with the video and the counts" em `silences_surface.intent` (3),
   `propose` (4: `surface = "proposed"`, 33 nós); os frames da superfície
   chegam (ocioso de novo);
4. `activate` em `proposed.silences.cut[4].apply_cut`, o segundo corte
   (o agente age por id; o cartão dele fica abaixo da dobra): request 5,
   gravado como `apply_cut on silences.cut[4]` (roteado por
   `Surface::route`);
5. a pessoa: `Poke::Click` em `proposed.silences.cut[3].reject_cut`, o
   primeiro corte; o shell rola a página com a roda se o botão não estiver
   inteiro na janela (não precisou em 1600 por 1000), grava `pointer_down`
   e `pointer_up` com o botão sob o ponteiro, e o clique vira request 6,
   `reject_cut on silences.cut[3]` por `human`;
6. `select` (7), `set_value` "compare the original with the cuts" (8),
   `propose` (9: o mesmo `proposed`, 27 nós, a comparação no lugar);
7. `close` em `silences_surface.close` (10: `closed = "proposed"`); a
   saída em vários frames.

3,3 s, 17 observações, 101 frames desenhados (100 guardados: um igual ao
anterior). com o Claude na revisão, o mesmo roteiro com a frase da tese:
127,5 s.

#### a demo (arquivo, tamanho, o que tem dentro)

`crates/showcase/fixtures/demo/silences-demo.monster`: 186.837 bytes
(eventos 81.078, índice 6.812, chunks 67.984, assets 21.085, ligações
9.600); 73 eventos (4 keyframes, 10 mudanças, 40 estágios, 17
observações, 2 inputs), 100 frames em 12 chunks, 134 assets, 3 imagens sem
pixels. `silences-demo-claude.monster`: 861.354 bytes (eventos 495.271,
chunks 255.517, assets 45.056, ligações 32.760, índice 32.472); 200
eventos, 226 frames em 63 chunks: o contador da espera muda a cada segundo
por 124 s, uma mudança e um frame por segundo.

`crates/showcase/tests/demo.rs` abre os dois como o `plev-inspect`
(`rec::Session::open`) e confere por dentro: os 10 requests na ordem, com
ator, alvo, ação e `completed` inteiros, o `close` por último com `closed
= <superfície>`; as intenções como escritas; as observações só do agente,
a página inteira e o escopo `silences.status`; o clique da pessoa
(`pointer_down` e `pointer_up` com o botão da superfície sob eles, e
`hit_at` no mesmo ponto dá o mesmo id); no instante do apply, as ligações
da superfície na tela ao lado das da fixa (`bindings_at`) e a superfície
no estado (`state_at`), sumida no fim; os frames do vídeo no arquivo como
imagens sem pixels (`to_bytes().images_without_pixels >= 3`); e o `why`
do inspector sobre o apply (a observação da página inteira como base, a
`decision = "pending"` sem mover, `Completed`). na do Claude, mais:
`attempts = 2`, `seconds > 60`, `cost_usd` entre 0 e 1. nenhum teste do
gate roda o ffmpeg, o demo ou o claude.

#### o que o plev-inspect disse

`plev-inspect crates/showcase/fixtures/demo/silences-demo.monster`:

```text
session   rec 0.1.0, built from a9edcbaa
span      +0.000s to +2.951s, 73 events, 100 frames
actors    agent:demo: 17 observations, 9 requests, 36 stages
          human: 1 request, 4 stages, 2 inputs
requests  10
  +0.039s  request 1   agent:demo  detect_silence on silences.detect     completed: cuts = 2, seconds = 0.060720708
  +0.835s  request 2   agent:demo  select on silences_surface.proposer   completed
  +0.839s  request 3   agent:demo  set_value on silences_surface.intent  completed
  +0.845s  request 4   agent:demo  propose on silences_surface.propose   completed: surface = "proposed", nodes = 33
  +1.565s  request 5   agent:demo  apply_cut on silences.cut[4]          completed: decision = "apply"
  +1.594s  request 6   human       reject_cut on silences.cut[3]         completed: decision = "reject"
  +1.604s  request 7   agent:demo  select on silences_surface.proposer   completed
  +1.613s  request 8   agent:demo  set_value on silences_surface.intent  completed
  +1.621s  request 9   agent:demo  propose on silences_surface.propose   completed: surface = "proposed", nodes = 27
  +2.353s  request 10  agent:demo  close on silences_surface.close       completed: closed = "proposed"
refusals  0
```

`--why 5` (o apply pela superfície gerada):

```text
request 5 (origin 0): apply_cut on silences.cut[4] by agent:demo, at +1.565s
  control       silences.cut[4]: list_item "0:08.0 to 0:10.0", enabled
  state         start = 8.010884, end = 10.0078, duration = 1.996916, range = "0:08.0 to 0:10.0", decision = "pending", frame = "silences.video@9.009342"
  initiated by  agent:demo
  action        apply_cut, no parameter
  observed      +1.560s, the whole app, at revision 19e5c9c8117320ad
                (the last observation by agent:demo at the revision the request carries)
  precondition  what the action reads, as agent:demo saw it:
                  silences.cut[4]:decision = "pending"
  before        from the observation (+1.560s) to the request (+1.565s):
                  silences.cut[4]:decision did not move
  stages        +1.565s requested
                +1.565s accepted
                +1.565s executing
                +1.565s completed: decision = "apply"
  verdict       accepted at +1.565s
  result        completed at +1.565s: decision = "apply"
```

`--control proposed.silences.cut[4].apply_cut --at 1600` (o botão da
superfície, com ela aberta) e `--control silences.cut[4] --at 1600`:

```text
control proposed.silences.cut[4].apply_cut at +1.600s
  node          proposed.silences.cut[4].apply_cut: button "apply cut", disabled
  state         none of its own
  actions       activate (reads proposed.silences.cut[4].decision:text), disabled
  source        crates/surface/src/project.rs:104:23, in build a9edcbaa
  requests      0 touched it up to +1.600s (their target, or a state they read)

control silences.cut[4] at +1.600s
  node          silences.cut[4]: list_item "0:08.0 to 0:10.0", enabled
  ...
  requests      1 touched it up to +1.600s (their target, or a state they read)
  request 5 (origin 0): apply_cut on silences.cut[4] by agent:demo, at +1.565s
```

`--why 6` (o clique da pessoa na superfície), o trecho que importa:

```text
request 6 (origin 0): reject_cut on silences.cut[3] by human, at +1.615s
  observed      not recorded: no observation by human before the request
                (a person acts on the frame on screen; the recorder keeps only the observations the app reports)
                the recorded state never had revision 3e714d915fb24cbe in this window
  before        the recording does not say when the state was at that revision, so not what moved
```

os achados, registrados no ADR e não corrigidos:

1. um ato numa superfície gerada chega ao dispatcher como a capacidade
   roteada (`Surface::route`): o receipt diz `apply_cut on
   silences.cut[4]`, nunca `proposed.silences.cut[4].apply_cut`. o `--why`
   explica sobre o nó do app, o `--control` do botão da superfície diz "0
   requests touched it" e o do nó do app diz request 5. a sessão não
   guarda que o ato veio pela superfície gerada; só o input da pessoa
   nomeia o botão dela.
2. o clique da pessoa na superfície (e na tela fixa) se apoia no snapshot
   só do app (`Surface::shown`, `SilencesPage::snapshot()`), com uma
   revisão que a gravação nunca tem, porque o gravador guarda a projeção
   da página inteira (`ShowcaseView::semantics()`, a `projection()`): duas
   projeções, duas revisões, e o inspector não coloca a observação da
   pessoa.
3. a revisão do Claude nomeou os controles com os ids do app
   (`silences.threshold`, `silences.detect`, `silences.cut[3]` como raiz da
   instância): o engine avisa "bound twice in one frame; the first binding
   holds" e a cópia da superfície não é atingível por `hit` nem aparece
   nas ligações (o botão `silences.cut[3].reject_btn` é só da superfície e
   ficou ligado). a 4b tinha visto o template; aqui são os controles.

#### a corrida com o Claude

uma corrida, 08:04:52 a 08:07:00, pelo exemplo com `--review claude
--compare rules`, com `PLEV_CLAUDE` apontando para um script no
scratchpad que conta as invocações e recusa a terceira sem rodar nada (o
laço da página tenta até 3 vezes por proposta; o orçamento era 2).
intenção "cut the silences of this video, then I want to review the
cuts"; sonnet; 2 chamadas reais (08:04:53 e 08:06:19): a primeira recusada
pelo verifier (a página mostrou "attempt 2 of 3" a +87,2 s), a segunda
verificada a +125,1 s. `attempts = 2`, `seconds = 124,27`, `cost_usd =
0,194`. superfície `silence_review`, 34 nós: título, preview, um card de
controles com os dois sliders e o detectar, status, cinco stats, e a
lista com `range_text`, `frame_img`, `duration_stat`, `decision_text` e
três botões por corte. a comparação pelas regras sobre ela: o mesmo id,
27 nós, no lugar. a sessão está em
`fixtures/demo/silences-demo-claude.monster`; o resumo do inspector:

```text
span      +0.000s to +127.174s, 200 events, 226 frames
actors    agent:demo: 17 observations, 9 requests, 36 stages
          human: 1 request, 4 stages, 2 inputs
  +0.773s    request 4   agent:demo  propose on silences_surface.propose   completed: surface = "silence_review", nodes = 34, attempts = 2, seconds = 124.2747015, cost_usd = 0.194234
  +125.793s  request 5   agent:demo  apply_cut on silences.cut[4]          completed: decision = "apply"
  +125.826s  request 6   human       reject_cut on silences.cut[3]         completed: decision = "reject"
  +125.844s  request 9   agent:demo  propose on silences_surface.propose   completed: surface = "silence_review", nodes = 27
  +126.579s  request 10  agent:demo  close on silences_surface.close       completed: closed = "silence_review"
```

na timeline: `+87.194s changes silences_surface.status:attempt: 1 -> 2` e
`+125.057s snapshot keyframe, 46 nodes: silence_review added; ...`. a
segunda chamada de reserva não foi usada; nenhuma outra chamada nesta
task.

#### contagem do gate e o regress

| | workspace | perna urnaui | total |
| --- | --- | --- | --- |
| antes, `main` @ a9edcbaa (o handoff da 4b, integrado nela) | 2444 passam, 0 falhas, 16 ignorados (111 suítes) | 82 passam, 1 ignorado (6 suítes) | 2526 |
| depois, esta branch @ 82db3ddf (`./script/gate`, saída 0) | 2451 passam, 0 falhas, 16 ignorados (113 suítes) | 82 passam, 0 falhas, 1 ignorado (6 suítes) | 2533 |

7 testes novos (4 unitários em `app/record.rs`, 1 em `tests/record.rs`, 2
em `tests/demo.rs`) e 2 suítes; nenhum ignorado a mais; a contagem não
cai. o gate rodou duas vezes: a primeira vermelha só no
`file_hygiene_guard` (`examples/demo.rs` com 736 linhas: o guard conta
exemplos como produção), corrigida pelo split; a segunda inteira verde em
2 min 6 s (08:15:50 a 08:17:56), com a perna urnaui (`../urna` resolve
deste worktree) e o check wasm com `-D warnings`. `df -h /` antes e
depois: 16 GB livres. `cargo run -p showcase --example regress -- check`:
159 baselines conferidas, 159 iguais, nenhum frame desenhado (a gravação
não muda o que uma página desenha). `cargo test -p showcase -p rec -p
inspect` verdes.

#### testes

- `app/record.rs` (4): só "1" liga, nada sem a variável; as teclas de
  salvar (`s` com shift e cmd ou ctrl; não sem shift, não `t`, não Enter);
  a tecla gravada pelo tipo; a página Silences gravada guardando o estado,
  um frame, um `pointer_down` com `silences.apply_all` sob ele e um `text`
  sem caractere, os nomes `showcase-<pid>-0` e `-1`, `hit_at` no frame.
- `tests/record.rs` (1): o shell sem janela pela porta real: o agente
  observa com bounds e aplica o corte 1; a pessoa clica em "apply all"
  (press e release gravados com o controle); o agente rejeita o corte 2 da
  revisão velha e volta `stale`. o arquivo salvo tem as observações com
  escopo e revisão, os 3 receipts inteiros, os dois inputs, os frames com
  as ligações iguais aos bounds servidos no primeiro olhar; o inspector diz
  que o `reject_cut` se apoiou nesse olhar e que `silences.cut[2]:decision`
  mudou pelo `apply_all` da pessoa, com o clique no trecho. o arquivo fica
  em `$CARGO_TARGET_TMPDIR/showcase-record-test.monster`.
- `tests/demo.rs` (2): as duas fixtures, acima.

#### seção de ADR

`docs/ADR/session-recording-replay-not-reexecution.md`, consequências,
um item datado "the showcase records itself too (added on 2026-09-24,
task-69 phase 4c)": o que grava e onde, a demo e as fixtures, e os três
achados sobre superfície gerada com o que cada um pede do checkpoint.
escolhi o ADR da gravação, não o do inspector: os achados são sobre o que
a sessão guarda (o request roteado, a revisão do snapshot só do app, a
ligação duplicada), não sobre como o inspector lê. para o orquestrador
(fora da lista): a linha do índice `docs/ADR/README.md` pode ganhar no
fim "; desde a task-69 fase 4c o showcase grava como o ide, e a demo do
editor de vídeo é uma sessão em `crates/showcase/fixtures/demo`";
`docs/arc/arc.toml`: `showcase::app::record`, o exemplo `demo` e
`tests/{record,demo}.rs`; o CHANGELOG (`PLEV_RECORD=1` no showcase).

#### decisões

- `record.rs` tem a `Recording` e os ganchos do `App` num módulo só: o
  shell do showcase vive na lib (o ide tem `record.rs` na lib e
  `recording.rs` no bin);
- o `redraw` saiu de `app.rs` para `pace.rs`: sem isso `app.rs` passaria
  de 639; o frame é do ritmo do loop, e `pace.rs` já tinha `frame_wake` e
  `pace`;
- `ShowcaseView::now(&mut self)` pelo `with_dispatcher`: a única forma de
  ler o relógio do todo sem tocar `apps/**`;
- o estado depois de um ato do agente gravado dentro de
  `serve_agent_recorded` (como o `serve_recorded` do ide), para o shell e
  o exemplo não esquecerem;
- as teclas de salvar como uma conferência de `Key` e `ModifiersState`: a
  galeria não tem keymap nem `ActionRegistry`;
- os inputs: press e release do esquerdo, press do direito, teclas pelo
  tipo; roda e movimento não (o ide também não);
- a demo como exemplo (`examples/demo`), não teste `#[ignore]`: é o
  programa que o brenner roda para gravar de novo, com flags; um teste
  ignorado não teria `--out` nem `--review`;
- a frase da revisão pelas regras é a delas, a da tese fica para o Claude
  (as regras recusam "then" e "review"); a comparação é a mesma frase nos
  dois;
- 1600 por 1000 para a demo, pelo botão fora da janela em 1200 por 800; o
  exemplo aceita outra medida;
- a pessoa clica na superfície gerada, não na tela fixa: é a tese; a
  rolagem até o botão fica no shell da demo (`reveal`), não na página;
- a sessão do Claude commitada (861 KB, abaixo de 2 MB) e lida pelo mesmo
  teste: é a demo de verdade que o brenner abre sem rodar nada;
- o guard de linhas conta exemplos: o exemplo em três arquivos.

#### limitações

- o relógio da gravação é o da `ShowcaseView` (o todo nasce nela), então
  a sessão começa em +0.012 s, não em 0;
- as duas fixtures foram gravadas pelo exemplo antes do split (o mesmo
  roteiro; o split juntou o select, a intenção e o propose numa função e
  faz uma observação a mais, 18 em vez de 17); o cabeçalho da sessão diz o
  git do `rec` (`a9edcbaa`), não o do exemplo;
- os frames do vídeo não têm pixels no arquivo (a retenção continua fora,
  task-63); `scene_at` lê a cena sem imagem;
- a revisão das regras em 1200 por 800 não cabe na metade (o botão de
  rejeitar em x = 1232..1341): a galeria no tamanho de lançamento mostra a
  linha cortada à direita; achado para o checkpoint b, a página está fora
  da lista desta fase;
- o cartão do segundo corte fica abaixo da dobra nas duas superfícies em
  1600 por 1000 (o agente age por id; a pessoa rolaria);
- a sessão do Claude tem 124 mudanças de status e 124 frames só do
  contador da espera (uma linha por segundo);
- a gravação é só desktop; `PLEV_BACKGROUND` não foi preciso (sem janela);
- o `plev-inspect` não diz que um ato veio pela superfície gerada (achado
  1), nem coloca a observação de um clique da pessoa (achado 2);
- o script guarda do claude e a contagem ficaram no scratchpad, nada
  commitado.

#### dependências pendentes e riscos de integração

- `Cargo.toml` do showcase: `rec` (desktop) e `inspect` como
  dev-dependency (desktop); `Cargo.lock` duas linhas; o ciclo `rec -> dev
  -> showcase -> rec` já existia por `ide`;
- `Served` de `dispatch/agent.rs` ganhou o campo `recorder`: quem monta à
  mão passa `None`; ninguém fora do arquivo;
- `ShowcaseView::now` e `serve_agent_recorded` novos; `serve_agent`,
  `add_sink`, `semantics`, `act` iguais;
- `app.rs`: `render_frame` agora é chamado de `pace.rs`; quem mexer no
  `RedrawRequested` acha `redraw` lá;
- as baselines do regress não mudaram; `.gitattributes` já trata
  `*.monster` como binário.

#### o que falta para a task-69 fechar

- o checkpoint b, do brenner: as duas telas lado a lado (a fixa da 4a e a
  gerada da 4b), agora com as duas demos gravadas para abrir com
  `plev-inspect` (`crates/showcase/fixtures/demo/*.monster`) e regravar
  com o exemplo; os achados 1 a 3 e a linha fora da janela em 1200 por
  800 são material para ele;
- um mp4 da tela é decisão do brenner (a demo é a sessão `.monster`);
- as linhas do índice de ADR, do arc e do CHANGELOG, pelo orquestrador.

## checkpoint b, primeira rodada (2026-09-24, orquestrador, a pedido do brenner)

o brenner pediu ("testa você mesmo") que o orquestrador fizesse a
comparação. feita pelo contrato, sem clique sintético: o showcase na
Silences com `PLEV_AGENT=1 PLEV_MIRROR=1 PLEV_BACKGROUND=1 PLEV_RECORD=1`,
as ações por `plev act` como o agente `orquestrador`, e cada tela
fotografada pelo espelho da task-79 (`mirror --frames 1 --png`), na
janela de 1200x800. imagens em `checkpoint-b/`.

- **tela fixa** (`02-fixed-detected.png`): o frame do vídeo, a linha do
  tempo com os dois silêncios, a detecção, os contadores, e a lista com
  frame, faixa, "2.0 s of silence", o estado e aplicar/rejeitar. o
  segundo corte fica abaixo da dobra.
- **gerada pelas regras** (`03-generated-rules.png`, instantânea, grátis):
  quebrada na janela padrão. na linha do corte, os textos saem um
  caractere por linha (a coluna encolhe a quase zero), os números saem
  crus ("3.018594") e os botões "preview segment" e "apply cut" passam
  da borda direita.
- **gerada pelo Claude** (`04-generated-claude.png`, 119,2 s, 2
  tentativas, 0,196 USD): completa em função (arquivo, duração, frame
  original e resultado, detecção, resumo com os contadores, e a lista
  com frame, faixa, estado, duração, prévia, aplicar e rejeitar por
  corte), mas pior de usar: a lista, que é a tarefa, vem por último,
  abaixo da dobra; os controles de detecção se repetem, com os sliders
  sem rótulo nem valor na tela; os dois frames são iguais sem nada que
  diga a diferença; as durações saem cruas ("1.996916").
- **o que funcionou:** o estado é um só. aplicar o primeiro corte pela
  fixa e rejeitar o segundo pela gerada deixou as duas telas e os
  contadores iguais (`05-after-task.png`); cada ação passou pelo
  dispatcher; um propose foi recusado como stale porque o próprio agente
  tinha mudado a intenção depois de olhar, e passou depois de observar.
- **veredito desta rodada: a gerada é pior de usar hoje.** duas causas
  são defeitos de implementação que valem para toda tela gerada (a linha
  encolhe numa coluna estreita; números sem formato); a terceira é da
  proposta (a tarefa vem por último e a tela repete o que a fixa mostra).
  nenhuma imagem de referência cobre a tela gerada aberta, e por isso o
  defeito de layout passou por todas as fases.
- **decisão do orquestrador:** julgar a tese por um defeito de layout
  não é justo. antes de aplicar a regra do checkpoint (gerada pior, a 70
  vai para transformação sobre telas fixas), os três defeitos são
  corrigidos como bug do núcleo e o checkpoint roda de novo do mesmo
  jeito, com as mesmas imagens. a regra do programa não muda.

## checkpoint b, as correções (task/69i)

branch `task/69i-checkpoint-b`, sobre a `main` em `5cfb0dfb`.

### sonda das correções (antes do primeiro tipo, 2026-09-24)

reproduzido em teste, na galeria de 1200x800 com o worker falso
(`silences/tests/generated/checkpoint.rs`, a versão de sonda que só
imprime): a coluna da superfície vai de x = 740 a 1160 (420 px); o card
das regras tem `padding: lg` (20 px), então a linha do corte tem 380 px.

(a) a linha. `crates/surface/src/layout.rs:138` passa o texto ao engine
como `Leaf::Text` sem largura mínima, e o `LayoutStyle` nasce com
`flex_shrink: 1.0` e `flex_wrap: false` (`crates/engine/src/layout/types.rs:75`
e `:77`); o `stack` só põe direção, gap e alinhamento (`layout.rs:175` a
`183`), nunca `flex_wrap` (só o `grid` quebra, `layout.rs:184` a `188`).
no taffy 0.9.2 o mínimo automático de um item flex é o seu min-content
(`compute/flexbox.rs:798` a `818`), e o engine mede o min-content de um
texto com largura 0 (`crates/engine/src/layout/engine.rs:143`): o shaper
quebra dentro da palavra, e o texto encolhe até um glifo. os botões e a
imagem são `Leaf::Fixed` com `flex_shrink = 0` (`layout.rs:133` a `137`),
não encolhem, e numa linha que não quebra passam da borda. a linha das
regras é um `stack` horizontal (`crates/propose/src/rules.rs:429`) com a
imagem `xl` (`:386`), cinco textos com `flex: 1` (`:395`) e três botões.
os números: os cinco textos com 9 a 10 px de largura e 158 a 360 px de
altura (um caractere por linha), quando a palavra mais longa de cada um
mede 43,8 a 63,8 px; "preview segment", "apply cut" e "reject cut"
terminam em x = 1107, 1224 e 1341, este 181 px além da borda do card. a
fixture do Claude (item 5 abaixo) repete o defeito na linha do corte: a
faixa com 9 px e os botões até x = 1348. não precisa mexer no engine
nem no comps: a largura mínima do texto e a quebra da linha são estilos
que a superfície já passa ao `LayoutEngine`.

(b) o slider. `crates/surface/src/widget.rs:185` cria o
`Slider::new(0.0, 1.0, 0.0).sem(id)` sem rótulo; o `sync_slider`
(`widget.rs:438` a `454`) põe faixa, passo, valor e `disabled`; o
`draw.rs:141` dá à folha só a altura do controle (`ControlSize::Xs`, 36
px) e o `draw.rs:330` chama `s.render`, que desenha trilho e botão e
nenhum texto (`crates/comps/src/form/slider.rs:243` a `330`). o rótulo do
nó nunca é desenhado. a tela fixa desenha ela mesma o nome em
`caption_sm`/`text_mid` e o valor em `caption_r`/`text`, alinhado à
direita, numa linha acima do slider (`silences/draw.rs:209` a `247`,
`silences/layout.rs:201` a `211`: a linha do rótulo, `spacing.xs`, o
slider).

(c) o número. `crates/surface/src/widget.rs:94`, `Value::Num(n) =>
n.to_string()`, no `text_of`, que serve o texto e o badge (`shown_text`,
`draw.rs:63`), o stat (`draw.rs:84`) e o rótulo que o texto e o badge
projetam (`project.rs:43`). a tela fixa formata com o que sabe do app:
`"{:.0} dB"` (`silences/draw.rs:222`), `"{:.1} s"` (`:228`), `"{:.1} s of
silence"` (`:319`). o IR não tem campo de unidade nem de formato (os
`prop` do stat e do texto são só `Text`, `Int`, `Num`,
`crates/uir/src/vocab.rs:246` e `:290`), e não precisa: a unidade já vem
no rótulo, o do app ("threshold (dB)", "minimum silence (s)") e o do
Claude ("duration (s)"). o formato mais justo sem tocar o IR: no máximo
duas casas, sem zero à direita. o vocabulário fica como está; não há
parada.

(d) o prompt. `crates/propose/src/prompt.rs:271` a `306`: o pedido, a
intenção, as regras do verificador, a política, os componentes, o
registro, a superfície de agora e o schema. nada sobre ordem, sobre o
que vem primeiro, sobre repetir um controle; de rótulo, só a regra de
acesso (`prompt.rs:49` a `53`: todo controle tem rótulo), que o Claude
cumpriu e o slider não desenhou. o `SYSTEM`
(`crates/claude-model/src/lib.rs:55` a `57`) só pede a resposta na forma
pedida: não precisa mudar.

(e) o regress. `run::stage` (`examples/regress/run.rs:52` a `76`) chama
`showcase::app::stage_page` (`src/app/headless.rs:77` a `111`): a view
nova, a página, o tema, 120 passos de `tick` e `render`. só o estado
inicial; a chave do baseline é página, tema e viewport
(`examples/regress/baseline.rs:40` a `64`). a Silences parada nunca chama
o ffmpeg (`silences/worker.rs:10` a `15`), mas uma superfície com imagem
pede os frames no `wake` (`silences/jobs.rs:70` e `184` a `190`), e fora
dos testes o worker é o ffmpeg de verdade (`worker.rs:88` a `95`); o falso
(`worker.rs:98` a `103`, `256` a `283`) só existe sob `cfg(test)` e o
exemplo, que é outro crate, não o vê. do exemplo, a página só é
alcançável pela API pública da `ShowcaseView` (`view/mod.rs:10`, `mod
apps` privado; `panes` privado em `view/mod.rs:97`): `act`, `semantics`,
`tick`, `render`. as regras chegam por `act` (proposer, intenção,
propose, como no roteiro da 4c); a fixture do Claude não tem caminho
público (as operações da linha são escrever, propor, escolher, abrir,
fechar e cancelar, `surface_page/proposing.rs:401` a `412`). o que
precisa: no regress, encenar um estado com nome (o roteiro pelo `act`) e
um tipo de página encenada que não seja o `StagedPage` (campos privados,
`run.rs:32`); no showcase, em `generated/page.rs`, dois ganchos de
encenação: os frames dos instantes que a página mostra postos no cache
antes do `wake` pedir (o conteúdo vem do exemplo, sintético e
determinístico; o hash de uma imagem é o tamanho, `examples/regress/hash.rs:30`
a `33`), e a fixture verificada pelo `uir` contra o registro do app e
mostrada por `Page::show` (`page.rs:156` a `170`).

