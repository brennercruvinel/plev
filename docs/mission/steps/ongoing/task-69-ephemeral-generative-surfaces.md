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
