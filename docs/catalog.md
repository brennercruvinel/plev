---
type: catalog
status: living
tags: [ui, components, design-system, comps, tokens, inventory]
date: 2026-09-18
---

# catálogo de componentes de ui

inventário de tudo que desenhava ui no repositório antes do crate `comps`
(seções 1 a 4, mantidas como registro do ponto de partida), o destino de
cada item (seção 5) e o que a migração fez (seção 7). a decisão está em
docs/adr/comps-one-design-system-crate.md.

## 0. estado atual (depois da migração)

| onde | o que |
|---|---|
| `crates/engine/src/theme` | tokens: `colors`, `glass`, `typography`, `spacing`, `radius`, `shape`, `control`, `size`, `duration`, `layout`, `shadows`, `motion`, `effects`. `Theme::hoff()`, `Theme::hoff_light()`, `Theme::named(..)` |
| `crates/comps/src/core.rs` | `Rect`, `WidgetEvent`, `EventResult`, helpers de cor |
| `crates/comps/src/recipe.rs` | `glass_pill`, `glass_surface`, `edge_light`, `inset_keylight`, `shadow_node`, `shadow_stack`, `menu_shadow`, `focus_ring`, `rounded_rect(_stroke)` |
| `crates/comps/src/icons.rs` | 37 ícones lucide |
| `crates/comps/src/action` | `Button`, `IconButton`, `Chip`, `Badge` |
| `crates/comps/src/form` | `TextField` (sobre `TextInput`), `Checkbox`, `Switch`, `Slider`, `Select`, `EditKey` |
| `crates/comps/src/nav` | `Tabs`, `NavLink`, `Sidebar`, `PanelHeader`, `Breadcrumb`, `SplitPane` |
| `crates/comps/src/content` | `Card`, `Avatar`, `Panel`, `Stat`, `CodeBlock`, `Skeleton`, `Table`, `EmptyState`, `VirtualList`, `Tree`, `Separator` |
| `crates/comps/src/feedback` | `Modal`, `ContextMenu`, `Toast`, `Tooltip`, `ProgressBar`, `Spinner`, `Scrollbar` |
| `crates/comps/src/overlay` | `OverlayManager` |
| `crates/comps/src/charts` | geometria + `draw` |
| `crates/comps/src/graph.rs` | `GraphView` |
| `crates/comps/src/editor` | `EditorView`, `EditorConfig::from_theme`, `EditorTheme::from_theme` |
| `crates/comps/src/shell.rs` | `AppShell`, `SafeArea`, `ShellLayout` |
| `crates/comps/src/tests` | `matrix.rs` (todo widget x todo tema x phone/desktop), `components.rs`, `widgets.rs`, `focus.rs` |
| consumidores | showcase (15 seções, chrome via `AppShell`), ide (`status.rs` é o único token local), urnaui (`view/field.rs` é um alias de `TextField`) |

referência visual: o git client (`crates/ide`, que consome `crates/git`)
e a galeria (`crates/showcase`). os dois já falam HOFF dark glass; o
problema é que falam em dialetos diferentes.

## 1. onde a ui vive hoje

| origem | linhas | o que é | estado |
|---|---|---|---|
| `engine/src/ui/widgets` | 7410 | 21 widgets retained, `&Theme`, `EventResult` | oficial (adr official-app-pattern) |
| `engine/src/ui/icons.rs` | 913 | 36 ícones lucide tesselados e cacheados | oficial |
| `engine/src/ui/{builder,modifier,node,render,theme}.rs` | ~770 | `Ui` builder imperativo + `UiTheme` (paleta antiga) | legado, zero consumidor fora de testes |
| `engine/src/text_input` | 897 | `TextInput`: buffer, caret, ime, blink, `build_scene` | oficial, restilizado à mão em cada app |
| `engine/src/overlay` | 657 | `OverlayManager`: pilha de menu/modal/toast com spring | oficial |
| `engine/src/charts` | 1536 | geometria pura (line, bars, area, donut) + `draw` | oficial |
| `engine/src/graph.rs` | 528 | layout de grafo (força, CSR, `ViewTransform`) | geometria, fica no engine |
| `engine/src/editor` | 1840 | view do editor de texto (rope) + input + clipboard sync | widget, um consumidor (example editor) |
| `engine/src/builder` | 2445 | `Element` declarativo (div, text, button, path, image) | suportado pra demo/protótipo |
| `engine/src/{view,component,signal}` | 1890 | traits experimentais | experimental, não entra no comps |
| `ide/src/theme.rs` | 493 | tokens HOFF transcritos 1:1 (radii, shadows, motion) | duplicata do `engine::theme` |
| `ide/src/components` | 1670 | button, tabs, modal, checkbox, context_menu, avatar, badge, separator, panel_header, hoff (recipes) | duplicata parcial dos widgets do engine |
| `showcase/src/view` | ~5900 | 14 seções, sidebar/nav/header próprios, dock, fields | galeria; carrega chrome de app reutilizável |
| `urnaui/src/view` | ~4900 | `Field`, `panel`, `group_label`, `text`, `render_frame_box`, 6 telas | app; helpers reinventados |
| `examples/scene3d/{theme,components}` | ~340 | chip, stat_card, hud_panel sobre o builder + paleta própria | demo; quarta paleta |
| `examples/{input,layers,mobile_input}/palette.rs` | ~200 | paletas soltas | demo |
| `examples/todo`, `text_input`, `counter` | ~2100 | apps de demo com ui inline | demo |

## 2. inventário por componente

legenda da coluna destino: `move` (vai pro comps como está, só troca o
caminho), `merge` (duas ou mais implementações viram uma), `novo` (não
existe em lugar nenhum e o design system precisa), `fica` (não é
componente, permanece no engine), `sai` (removido, coberto por outro).

### 2.1 ação

| componente | engine widgets | ide components | outros | destino | notas |
|---|---|---|---|---|---|
| Button | `Button` (Solid/Outline/Ghost/Danger, Sm/Md/Lg, intent, icon, focus ring) | `button::draw` (Glass/Ghost/Danger, Sm 36/Md 44/Lg 52) | `ui::Ui::button`, `builder::button` | merge -> `comps::Button` | alturas divergem: engine Sm=40 (chip social), ide Sm=36 (tab button). vira `ControlSize` nos tokens |
| IconButton | `IconButton` | (inline no header) | | move | |
| Chip | `Chip`, `CHIP_H` | `badge::Tag` | `scene3d::chip` (6 variantes coloridas) | merge -> `comps::Chip` | `CHIP_H` const vira token de tamanho |
| Badge | | `badge::Notification` (20px, vermelho, contador) | `ui::Ui::badge` | merge -> `comps::Badge` | |
| Link / NavLink | | (inline em `views/sidebar.rs`) | showcase `render_sidebar` (NAV_H 48, radius 12) | novo -> `comps::NavLink` | os dois apps desenham o mesmo nav link na mão |

### 2.2 formulário

| componente | engine widgets | ide | outros | destino | notas |
|---|---|---|---|---|---|
| Checkbox | `Checkbox` | `checkbox::draw` | | merge | |
| Switch | `Switch` | | | move | |
| Slider | `Slider` | | | move | |
| Select | `Select` | | | move | |
| TextField | `text_input::TextInput` (cru, sem glass) | | showcase `forms/fields.rs::TextFields`, urnaui `view/field.rs::Field` | merge -> `comps::TextField` | o widget do engine não conhece o tema; cada app reaplica os tokens glass. `FIELD_FONT 16` / `FIELD_H 32` hardcoded em dois lugares |
| TextArea | | | | novo | multi-linha sobre o mesmo buffer |
| SearchField | | | urnaui `search.rs` (inline) | novo | field + ícone + clear |
| Form / Field label + helper + error | | | | novo | agrupamento com intent (erro, sucesso) |

### 2.3 navegação

| componente | engine | ide | outros | destino | notas |
|---|---|---|---|---|---|
| Tabs | `Tabs` | `tabs::draw` (bloco ativo com shadow spec) | | merge | |
| Sidebar | | `views/sidebar.rs` | showcase `render_sidebar` + `sidebar_item_rects` | novo -> `comps::Sidebar` | logo, nav links, footer, scroll interno, colapso em viewport estreito |
| Header / PanelHeader | | `panel_header::draw`, `views/header.rs` | showcase `render_header`, urnaui `group_label` | merge -> `comps::PanelHeader` | |
| Breadcrumb | | | | novo | |
| SplitPane | `SplitPane` | | | move | |
| Dock | | | showcase `dock.rs` + `model/dock.rs` | move -> `comps::Dock` | modelo já testado sem gpu |

### 2.4 conteúdo

| componente | engine | ide | outros | destino | notas |
|---|---|---|---|---|---|
| Card | `Card` (Stat, Profile, Media, List, Chart, Cta), `CardListRow` | | `scene3d::stat_card` | merge | 266 literais numéricos no card.rs; maior foco de hardcode |
| Avatar | | `avatar::draw` (44px, inicial) | showcase `cards.rs::avatar_image` | merge -> `comps::Avatar` | |
| Separator | | `separator::{horizontal,vertical}` | | move | |
| EmptyState | `EmptyState` | | | move | |
| VirtualList | `VirtualList` | | | move | |
| Tree | `Tree`, `TreeNode` | | | move | |
| Table / DataGrid | | (diff_view, multi_stack_view inline) | urnaui `chunks.rs`, `stats.rs` (inline) | novo | linhas virtualizadas + colunas com largura derivada |
| Panel / Surface | | `hoff::glass_surface` | urnaui `panel`, showcase inline | merge -> `comps::Panel` | superfície glass com rim e keylight |
| KeyValue / Stat readout | | | urnaui `stats.rs`, showcase `typography.rs` readouts | novo -> `comps::Stat` | |
| Code block / mono | | `diff_view` | showcase `code()` helpers, urnaui | novo -> `comps::CodeBlock` | uma família só (inclusive sans); o helper `code_style` existe em 3 arquivos |
| Editor | `editor::view` | | example editor | move -> `comps::Editor` | rope fica no crate `rope`, view vem pro comps |

### 2.5 feedback e overlay

| componente | engine | ide | outros | destino | notas |
|---|---|---|---|---|---|
| Modal | `Modal`, `ModalAction` | `modal::draw` + `centered_pos` + `dimensions` (max-w 400 hardcoded) | | merge | |
| ContextMenu | `ContextMenu`, `MenuEntry` | `context_menu::draw` (240px hardcoded) | | merge | |
| Toast | `Toast`, `ToastManager` | | | move | |
| Tooltip | `Tooltip` | | | move | |
| ProgressBar | `ProgressBar` | | | move | |
| Spinner | `Spinner`, `SpinnerSize` | | | move | |
| Scrollbar | `Scrollbar` | | | move | |
| OverlayManager | `overlay::*` | | | move | pilha, spring, `OverlayKind` |
| Skeleton | | | | novo | placeholder de carregamento |

### 2.6 dados

| componente | engine | outros | destino | notas |
|---|---|---|---|---|
| charts (line, bars, stacked area, donut, meter, hbars, axis) | `charts::*` | showcase `charts.rs`, urnaui `stats.rs` | move -> `comps::charts` | geometria + draw; a view dona do tween de reveal |
| GraphView | `widgets::GraphView`, `EdgeTone` | urnaui `graph.rs` | move | `engine::graph` (layout) fica no engine |

### 2.7 primitivas e receitas (não são componentes, são a base deles)

| item | onde | destino | notas |
|---|---|---|---|
| `Rect`, `WidgetEvent`, `EventResult` | `widgets/mod.rs` | move -> `comps::core` | contrato de todo widget |
| `glass_pill`, `focus_ring`, `menu_shadow`, `rounded_rect`, `rounded_rect_stroke` | `widgets/mod.rs` | merge com `ide::components::hoff` -> `comps::recipe` | o ide tem `shadow(spec)` com spread emulado e `edge_light` com máscara; o engine tem `glass_pill`. vira uma receita |
| `with_alpha`, `luminance`, `contrast_text`, `intent_fill`, `mix` | `widgets/mod.rs` | move -> `comps::core::color` | |
| `measure_text`, `text_width` | `ide::components::hoff` | sai | `TextMeasurer` do engine já faz |
| `icons` | `engine::ui::icons` | move -> `comps::icons` | 36 ícones; o set cresce aqui |
| `ShadowSpec`, `SHADOW_MENU`, `SHADOW_MODAL`, `SHADOW_TOOLTIP`, `SHADOW_TABS_BLOCK` | `ide::theme` | move -> `engine::theme::EffectTokens` | o engine só tem `shadow_sigma`; as pilhas do spec só existem no ide |
| tabela de radii HOFF (pill 32, dropdown 24, tabs 22, card 20, block 18, item 16, nav 12, cluster 10, tooltip 8, micro 6) | `ide::theme` | move -> `engine::theme::RadiusScale` | o engine tem none/sm/md/lg/xl/full e os widgets improvisam (`theme.radius.lg` pra card que é 20 no spec) |
| motion (.2s / .3s) | `ide::theme` | move -> `engine::theme::MotionPhysics` | duração ao lado da física de spring |
| `StatusColors` (git: modified/added/deleted/renamed/untracked) | `ide::theme` | fica no ide | semântica de domínio, não de design system |
| `UiTheme`, `Accent`, `Ui`, `NodeMod`, `NodeRef` | `engine::ui` | sai | paleta paralela sem consumidor; o `Element` builder cobre o caso declarativo |
| `scene3d::theme` (SURFACE_0..5, CHIP_*) | example | sai | example passa a ler `Theme` |
| `palette.rs` em input, layers, mobile_input | examples | sai | idem |

## 3. hardcode: o que está fora dos tokens

auditoria por grep de literal numérico fora de teste, com o que cada
número é de verdade:

| valor | onde aparece | o que é | token que falta |
|---|---|---|---|
| 36 / 40 / 44 / 52 | `ButtonSize::height` (engine e ide, divergem), `tabs` 36, `context_menu` item 44, `avatar` 44, `NAV_H` 48 | alturas de controle | `ControlSize { xs: 36, sm: 40, md: 44, lg: 48, xl: 52 }` |
| 12 / 16 / 24 / 32 | `pad_x` de button, `PAD` 40 no showcase, modal padding 32, menu padding 8 | padding de controle e de container | `SpacingScale` já cobre 4..48; os widgets precisam ler dela |
| 20, 24, 32, 6, 8 (radius) | card, modal, context_menu, chip, checkbox | radii do spec | `RadiusScale` HOFF (seção 2.7) |
| 240 (menu), 400 (modal), 248 (sidebar), 232 (card chart) | context_menu, modal, showcase, card | larguras fixas | `SizeScale` de containers: menu, modal (max), sidebar, card min |
| `TextStyle::new(32.0).with_line_height(40.0)` | card.rs stat value | ramp fora da `TypographyScale` | `typography.h4()` ou um `stat()` novo na ramp |
| `blur_radius: 16.0/32.0`, `offset [0, 16]`, `[18/255; 0.35]` | `menu_shadow`, card keylight | shadow specs | `EffectTokens::{menu, modal, tooltip, tabs_block}` |
| `0.45` (luminance cut), `[0.02,0.02,0.04]`, `[0.98,0.98,0.99]` | `contrast_text` | contraste WCAG | ok como algoritmo; a cor de saída vem de `colors.text`/`bg` |
| 1.0 / 1.5 (edge width) | button, card, ide hoff | espessura do rim | `GlassTokens::edge_width` |
| `FIELD_FONT 16`, `FIELD_H 32`, `TEXT_PAD 8` | urnaui field, showcase fields | field deriva de font | `TextField` lê `typography.body()` e `ControlSize` |
| `SIDEBAR_W 248`, `HEADER_H 78`, `SIDEBAR_TOP 96`, `SIDEBAR_FOOTER_H 64` | showcase view/mod.rs | chrome de app | `Sidebar` deriva do conteúdo; largura vira token + breakpoint |

## 4. o que o design system não tem e precisa ter

pra ser cross-device e responsivo de verdade, e não só "grid deriva de
content.w":

| lacuna | proposta |
|---|---|
| breakpoints | `Breakpoint { Compact (<600), Medium (600..1024), Expanded (>=1024) }` derivado do viewport lógico, com `theme.layout.for_width(w)` retornando gutter, colunas, largura de sidebar, se o sidebar colapsa em drawer |
| escala de controle | `ControlSize` (seção 3), com `min_touch_target` 44 em compact |
| densidade | `Density { Comfortable, Compact }` multiplica spacing e control size; touch pede comfortable |
| tamanhos de container | `SizeScale { menu_w, modal_max_w, sidebar_w, card_min_w, readable_max_w }` |
| escala de ícone | `IconSize { sm: 16, md: 20, lg: 24 }` |
| z-order / elevação | `Elevation { Base, Raised, Overlay, Toast }` mapeando pra shadow spec + layer |
| shadow stacks | as quatro pilhas do spec, hoje só no ide |
| safe area | inset de notch/home indicator lido da plataforma e aplicado pelo `Shell` |
| tipografia | a ramp HOFF já é completa; falta `mono()` (o `code_style` existe em 3 lugares) e `stat()` (32/40/500 do card) |
| acessibilidade por componente | cada widget expõe `role` + `label` pro accesskit; hoje só o que passa pelo builder tem |

## 5. destino: `crates/comps`

```
crates/comps/
  Cargo.toml          depende só de engine (e rope pro editor)
  README.md
  src/
    lib.rs            re-exporta tudo por categoria; `prelude`
    core/             Rect, WidgetEvent, EventResult, color helpers, Widget trait
    recipe/           glass (pill, surface, rim, keylight), shadow(spec), focus ring
    tokens/           ControlSize, Breakpoint, Density, Elevation, IconSize
                      (extensões que leem engine::theme; o Theme fica no engine)
    icons/            o set lucide (movido de engine::ui::icons)
    action/           Button, IconButton, Chip, Badge, NavLink
    form/             TextField, TextArea, SearchField, Checkbox, Switch,
                      Slider, Select, FieldGroup
    nav/              Tabs, Sidebar, PanelHeader, Breadcrumb, SplitPane, Dock
    content/          Card, Avatar, Separator, EmptyState, VirtualList, Tree,
                      Table, Panel, Stat, CodeBlock, Skeleton
    feedback/         Modal, ContextMenu, Toast, Tooltip, ProgressBar,
                      Spinner, Scrollbar
    overlay/          OverlayManager (movido)
    charts/           movido de engine::charts
    graph/            GraphView (o layout fica em engine::graph)
    editor/           view do editor (movido de engine::editor)
    shell/            AppShell: sidebar + header + content + overlays +
                      breakpoint + safe area (o chrome que ide, showcase e
                      urnaui reescrevem)
  tests/
    every_widget_renders_under_every_theme.rs
    every_widget_at_narrow_and_wide.rs
    no_literal_outside_tokens.rs   (grep guard, como o arc_sync_guard)
```

o engine mantém: compositor, gpu, text, layout (taffy), theme (tokens),
input, animation, overlay? não, overlay vem; accessibility, platform,
window, color, path, effects, graph (geometria), builder (declarativo,
demo), view/component/signal (experimental).

quem consome: showcase (vira a galeria do comps, cada seção = um módulo),
ide (troca `components/` e `theme.rs` pelo comps), urnaui (troca `Field`,
`panel`, `group_label` pelo comps), examples todo/text_input/editor/
scene3d/charts.

## 6. ordem de migração (como foi planejado)

cada passo passa o gate inteiro antes do próximo.

1. tokens no engine: `RadiusScale` HOFF, `EffectTokens` com shadow specs, `MotionPhysics` com durações, `ControlSize`, `SizeScale`, `IconSize`, `Breakpoint`, `Density`, `Elevation`, `typography.mono()/stat()`. testes: cada valor bate com o spec (os testes do `ide/theme.rs` migram pra cá)
2. crate `comps` nasce com `core` + `recipe` + `icons` movidos, engine re-exporta nada (corte limpo), showcase e urnaui apontam pro comps
3. widgets movem por categoria (ação, form, nav, content, feedback), um PR por categoria; em cada um os literais viram leitura de token e o widget ganha teste de tema x tamanho x breakpoint
4. `text_input`, `overlay`, `charts`, `GraphView`, `editor` movem
5. merges: ide components somem, ide lê comps; `Sidebar`, `PanelHeader`, `Avatar`, `Badge`, `Separator` nascem dos dois lados
6. novos: `TextField` (glass), `Table`, `Stat`, `CodeBlock`, `Skeleton`, `Breadcrumb`, `AppShell`
7. showcase vira galeria completa do comps (uma seção por categoria, toda variante, todo tema, compact/medium/expanded), `engine::ui` legado sai, paletas de example saem
8. arc.toml, README, AGENTS.md e code-against-the-plev-engine.md apontam pro comps

## 7. o que a migração fez

- tokens: `ShapeTokens`, `ControlTokens` (heights, pad_x, icons, rims, focus ring, tabs/menu pad, box/switch/slider/scrollbar/spinner/progress/divider), `SizeTokens`, `DurationScale`, `LayoutTokens` + `Breakpoint` + `SidebarMode` + `Density`, `ShadowTokens` + `ShadowSpec` + `Elevation`; `GlassTokens` ganhou `tabs`, `text_active`, `text_default`, `disabled_alpha`, `wash_alpha`, `wash_hover_alpha`; a ramp ganhou `headline`, `mono`, `readout`; `Theme::hoff_light`. `ScrollState::max_offset` ficou público. `TextMeasurer::elide_path` veio do ide.
- corte limpo: `engine::ui`, `text_input`, `overlay`, `charts`, `GraphView`, `editor` saíram do engine; `engine::clipboard` ficou com os providers. `Ui`/`UiTheme` apagados.
- todos os widgets movidos passaram a ler tokens (button, icon_button, chip, checkbox, switch, slider, select, tabs, split_pane, card, empty_state, list, tree, modal, context_menu, toast, tooltip, progress, spinner, scrollbar, graph, charts::draw, editor). modal e toast viraram responsivos (largura pelo breakpoint).
- novos: `TextField`, `Badge`, `Avatar`, `Separator`, `Panel`, `Stat`, `CodeBlock`, `Skeleton`, `Table`, `NavLink`, `Sidebar`, `PanelHeader`, `Breadcrumb`, `AppShell`.
- ide: `theme.rs` e `components/` apagados; views sobre comps + `Theme::hoff()`/`hoff_light()`; `status.rs` com as cores de status de arquivo.
- showcase: chrome via `AppShell` (sidebar cheia, rail, drawer), seção `Chrome` nova, fields e app section sobre `TextField`.
- urnaui: `Field` vira alias de `TextField`; layouts recebem o tema.
- examples: `todo` e `text_input` absorvidos pelo showcase; `editor` foi pra `crates/comps/examples`; `input`, `layers`, `mobile_input`, `scene3d` leem `Theme::default()`.
- o que ficou como constante nomeada, de propósito: tempos de plataforma (blink 0,53 s, delay de tooltip 0,45 s, multi-click 0,4 s, fade da scrollbar 1 s, velocidade do spinner), a geometria pura dos charts (gaps mínimos, raio máximo de barra, furo do donut), o mundo do grafo (`WORLD`, `NODE_R`, `CLICK_DIST`), a fração de largura da mensagem do empty state. nada disso é cor, raio, altura ou padding de design.
