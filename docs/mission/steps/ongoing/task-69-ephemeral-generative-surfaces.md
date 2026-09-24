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
