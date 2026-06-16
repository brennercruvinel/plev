---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-13
domain: competitive
---

# relatorio tecnico: φ vs makepad

analise comparativa de arquitetura, padroes de design, e licoes para construir
uma interface de nivel profissional.

data: 2026-03-13

---

## parte 1 - φ

### 1.1 ideia central

φ e um **motor de composicao GPU-first** - nao um framework de widgets, mas
a camada de renderizacao que fica *embaixo* de um. a tese e: separar completamente
o problema de "colocar pixels corretos na tela de forma eficiente" do problema de
"organizar widgets, estado, e interacao".

a maioria dos frameworks UI em rust (iced, dioxus, xilem, makepad) mistura as duas
camadas. φ aposta que a camada de renderizacao, feita corretamente uma vez, pode
servir multiplos paradigmas de UI acima dela.

alvos: macos/metal, ios/metal, linux/vulkan, android/vulkan, windows/dx12,
browser/webgpu - todos via wgpu, sem abstracoes de GPU customizadas.

### 1.2 design pattern principal

**retained scene graph com dirty tracking por hash.**

o padrao central e:
1. a aplicacao empurra `SceneNode` (rect, roundedrect, text, path) para camadas
2. cada camada e hasheada via fxhasher (hash de 64 bits sobre todos os nodes)
3. se o hash da camada nao mudou desde o frame anterior, zero trabalho de GPU
4. se mudou, os buffers GPU sao atualizados via `GpuVec` (grow-only, nunca shrink)

isso significa que uma UI com 10 camadas onde apenas 1 muda por frame faz 1/10 do
upload de GPU que um renderer naive faria.

### 1.3 divisao de responsabilidades

```
compositor.rs (1047 LOC)
  Dono: scene graph, camadas, dirty tracking, upload de geometria
  NAO faz: rendering real, shaders, estado de widgets

gpu.rs (503 LOC)
  Dono: contexto wgpu, 4 pipelines (quad, rect_sdf, text, composite),
        projecao ortografica, blend state premultiplicado
  NAO faz: decisoes de layout, gerenciamento de cena

text.rs (571 LOC)
  Dono: shaping (cosmic-text), atlas de glifos (etagere + LRU), cache de shaping
  NAO faz: renderizacao (emite quads para o compositor)

window.rs (787 LOC)
  Dono: event loop (winit 0.30), buffering de eventos, ciclo de vida GPU
  NAO faz: logica de aplicacao

signal.rs (1038 LOC)
  Dono: sistema reativo push-pull, thread-local runtime, memos, efeitos
  NAO faz: renderizacao (desacoplado da GPU)

animation.rs (1283 LOC)
  Dono: 33 easings, Tween<T>, Spring<T> analitico, KeyframeSequence
  NAO faz: decisao de quando animar (consumidor decide)

layout.rs
  Dono: wrapper de Taffy 0.9, flexbox, calculo de bounds
  NAO faz: rendering (produz ComputedBounds consumidos por outros)

component.rs (430 LOC)
  Dono: ciclo de vida (mount/update/unmount), cache de nodes, invalidacao
  NAO faz: layout, rendering

builder.rs (1205 LOC)
  Dono: API declarativa (div, text, button, path), conversao para SceneNode
  NAO faz: GPU, layout real

view.rs (361 LOC)
  Dono: trait View, ViewContext, ContainerView/RectView/TextView
  NAO faz: estado, animacao

effects.rs + texture_pool.rs
  Dono: blur gaussiano 13-tap, shadow, composite pass
  NAO faz: decisao de quando aplicar efeitos

input/ (3 arquivos)
  Dono: fila de eventos, hit-testing linear reverso, GestureRecognizer 6-estados
  NAO faz: rendering, layout

path.rs
  Dono: API de paths vetoriais via Lyon, tessellacao
  NAO faz: shader proprio (reutiliza pipeline de quads)

Shaders (5 arquivos WGSL)
  quad.wgsl       -- quad colorido, premultiplied alpha
  rect_sdf.wgsl   -- rounded rect via SDF (Inigo Quilez), anti-aliasing smoothstep
  text.wgsl       -- sampling de atlas R8Unorm
  blur.wgsl       -- gaussiano separavel 13-tap
  shadow.wgsl     -- silhueta para sombras
  composite.wgsl  -- composicao final
```

**principio de separacao:** cada arquivo tem uma unica responsabilidade e nenhum deles
conhece a logica de negocio da aplicacao. a aplicacao so fala com `builder.rs` (API
declarativa) ou diretamente com o `Compositor` (API imperativa).

### 1.4 conceito arquitetural

**pipeline de frame (ordem critica):**

```
begin_frame()          limpa scene nodes
    |
build_scene()          empurra SceneNode para camadas
    |
text.begin_frame()     prepara para resolver texto
    |
compositor.resolve()   hash da cena, upload se dirty
    |
text.resolve()         shaping, emissao de glyph quads
    |
render_pass()          2 draw calls: quads, depois texto
    |
present()              exibe frame
```

a ordem e **inviolavel**. `begin_frame()` limpa tudo - chamar depois de `build_scene()`
apaga os nodes. esse bug ja causou tela branca no WASM.

**buffers GPU (gpuvec):** crescem, nunca encolhem. isso elimina fragmentacao e
re-alocacao. o custo e memoria nao liberada, mas para UI o pico e tipicamente
estavel.

**alpha premultiplicado:** enforced em todos os shaders. output e sempre
`vec4(rgb * a, a)`, blend e `One / OneMinusSrcAlpha`. misturar alpha reto
causa artefatos de borda em composicao.

### 1.5 inovacoes

1. **hash-based dirty tracking por camada** - nenhum outro framework rust faz
   isso. makepad compara posicao de retangulo (menos preciso). iced re-renderiza
   tudo. a granularidade do hash permite que uma mudanca de cor em um node de uma
   camada seja detectada sem diff completo.

2. **SDF rounded rectangles com anti-aliasing** - shaders calculam distancia
   assinada, smoothstep de 1px, suporte a borda interna. resolucao-independente.

3. **spring analitico com 3 regimes** - subamortecido, criticamente amortecido,
   superamortecido. resolvidos analiticamente (nao euler), frame-rate independente
   (testado 30/60/120 fps convergem).

4. **signal system com push-pull e RAII guard** - clean/check/dirty state machine,
   diamond problem resolvido via memo comparison, observerguard para panic safety.

5. **borrow split pattern no text system** - remove do cache, usa, reinsere.
   resolve conflito de lifetime entre `&mut self` (atlas) e `&ShapedEntry`.

6. **reutilizacao de pipeline para paths** - lyon tessella, emite vertices no
   formato do quad pipeline. zero shader adicional.

7. **12k LOC para 6 plataformas** - ratio de funcionalidade/codigo extremamente
   alto. makepad precisa de ~1m LOC para cobrir o mesmo escopo (incluindo widgets).

---

## parte 2 - makepad

### 2.1 ideia central

makepad e um **framework UI completo com ide integrada** - do shader ate o
editor de codigo, tudo em um unico projeto. a tese e: controlar toda a stack
permite otimizacoes que frameworks modulares nao conseguem (hot reload de shaders,
DSL que compila para GPU, layout como subproduto do drawing).

makepad quer ser o "unity da UI" - uma ferramenta completa onde o desenvolvedor
cria, testa, e deploya sem sair do ecossistema.

### 2.2 design pattern principal

**widget trait object com DSL declarativa e turtle layout.**

o padrao central e:
1. widgets implementam o trait `Widget` (draw, handle_event, walk)
2. uma DSL (`script_mod!`) declara a arvore de widgets com atributos
3. o turtle (cursor) caminha pelo espaco conforme widgets sao desenhados
4. layout e drawing sao a mesma operacao - nao ha fase de layout separada

diferenca fundamental com φ: **makepad nao separa layout de rendering**.
o turtle avanca enquanto widgets sao desenhados. isso simplifica o modelo mental
mas acopla layout a GPU.

### 2.3 divisao de responsabilidades

```
widgets/src/widget.rs (1755 LOC)
  Dono: trait Widget + WidgetNode, WidgetRef (smart pointer), WidgetRegistry,
        WidgetSet, DrawStep (rendering incremental)
  Pattern: trait object com type erasure + downcast

widgets/src/dock.rs (1767 LOC)
  Dono: layout tipo IDE (paineis, splitters, tabs)
  Estrutura: DockItem enum recursivo (Splitter{a,b} | Tabs | Tab)
  Pattern: lazy instantiation via ComponentMap, stack-based iteration

widgets/src/portal_list.rs (2452 LOC)
  Dono: virtual scrolling para listas longas
  Estrutura: Fenwick Tree para O(log n) posicao->altura
  Pattern: FSM de scroll (6 estados), bidirectional traversal

draw/src/turtle.rs (2459 LOC)
  Dono: sistema de layout (Walk + Size + Flow + Layout)
  Estrutura: cursor que caminha pelo espaco, 3 retangulos concentricos
             (inner, rect, outer), resolucao diferida de Size::Fill
  Pattern: NaN como sentinel para tamanhos desconhecidos

draw/src/cx_2d.rs (142 LOC)
  Dono: contexto 2D, integracao com Turtle
  Pattern: Deref chain para CxDraw

studio/desktop/src/main.rs (312 LOC)
  Dono: aplicacao IDE completa
  Pattern: hub async para comunicacao com backend
```

**principio de acoplamento:** makepad acopla fortemente widget, layout, e rendering.
um `DrawQuad` sabe como se posicionar (walk), como se desenhar (shader), e como
responder a eventos. isso torna widgets auto-contidos mas dificulta reutilizacao
da camada de rendering isoladamente.

### 2.4 conceito arquitetural

**turtle layout (cursor-based):**

```
Flow::Right   cursor caminha para a direita, opcional wrap
Flow::Down    cursor caminha para baixo
Flow::Overlay empilha no mesmo ponto (z-order)
```

cada widget recebe um `Walk` (margin, width, height) e consome espaco do turtle.
tamanhos podem ser:
- `Fixed(px)` - exato
- `Fill { weight }` - proporcional ao espaco restante
- `Fit { min, max }` - determinado pelo conteudo (propagado como nan)

`Size::Fill` e resolvido em segunda passada: o turtle acumula `DeferredFill` entries
e distribui o espaco restante por peso.

**DSL (script_mod! / live_design!):**

```rust
script_mod! {
    MyWidget = {{MyWidget}} {
        width: Fill, height: Fit,
        label = <Label> { text: "Hello" }
        button = <Button> { text: "Click" }
    }
}
```

a DSL e compilada para um script VM interno. atributos marcados `#[live]` sao
sincronizados automaticamente. `#[rust]` sao invissiveis para a DSL. isso permite
hot reload: editar a DSL, recompilar o script, e os widgets atualizam sem recompilar
rust.

**evento + action:**

```
Event (input do sistema)
  -> Widget::handle_event()
    -> Action (output tipado do widget)
      -> Parent pattern-matching sobre actions dos filhos
```

actions sao enums tipados por widget. o parent faz `match` sobre as actions dos
filhos para coordenar comportamento. nao ha event bus global.

### 2.5 inovacoes

1. **turtle layout** - layout como subproduto do drawing. sem fase separada de
   layout tree. mais simples para o caso comum, mas perde otimizacoes de skip
   de subarvore.

2. **fenwick tree no portallist** - o(log n) para mapear scroll position para
   item index. permite virtual scrolling com 100k+ items sem escanear alturas.

3. **DSL com hot reload** - editar UI sem recompilar rust. o script VM interpreta
   mudancas e atualiza widgets in-place.

4. **dockitem como arvore recursiva** - splitter{a,b} forma uma arvore binaria.
   tabs agrupa folhas. drag-drop reestrutura a arvore. serializa para persistencia.

5. **drawstep incremental** - `draw()` retorna `Result<(), WidgetRef>`. ok = pronto.
   err = precisa de mais um frame. permite rendering pausavel para widgets complexos.

6. **componentmap para lifecycle** - widgets instanciados sob demanda, preservados
   entre frames. template (scriptobjectref) e separado de instancia (widgetref).

7. **stack completa integrada** - do shader ao editor de codigo, tudo e makepad.
   isso permite otimizacoes cross-layer que frameworks modulares nao conseguem.

---

## parte 3 - o que aprender do makepad

### 3.1 o demo no browser (contexto)

o screenshot mostra a ide makepad rodando no browser via webassembly:
- barra lateral esquerda com arvore de arquivos
- area central com tabs e editor de codigo (syntax highlighting, numeros de linha)
- painel inferior com log/output
- sidebar direita com "worldview" (preview 3d)
- tudo responsivo, com splitters arrastáveis

para φ atingir esse nivel de interface, precisa adotar padroes especificos.
a seguir, cada padrao necessario e como implementa-lo sobre a arquitetura existente.

---

### 3.2 dock system (paineis + splitters + tabs)

**o que o makepad faz:**
dockitem e uma arvore recursiva binaria. cada no e splitter (divide horizontal ou
vertical) ou tabs (agrupa conteudo). drag-drop entre paineis reestrutura a arvore.
o estado serializa para disco.

**o que φ precisa:**

```rust
enum DockNode {
    Split {
        axis: Axis,           // Horizontal | Vertical
        ratio: f32,           // 0.0..1.0, posicao do splitter
        children: [Box<DockNode>; 2],
    },
    Tabs {
        tabs: Vec<TabId>,
        active: usize,
    },
}
```

sobre a camada existente de φ:
- `DockNode` e pura dados - nao conhece GPU
- um `DockRenderer` traversa a arvore, calcula bounds com taffy (ja integrado),
  e emite `SceneNode::Rect` para cada painel/splitter/tab bar
- drag-drop usa o `GestureRecognizer` existente (6 estados) para detectar arrasto
  sobre zonas de drop (10% das bordas = split, centro = merge tab)
- persistencia: serializar `DockNode` para JSON/ron no disco

**LOC estimado:** ~400 para docknode + renderer, ~200 para drag-drop, ~100 para
serialization. total: ~700 LOC.

**diferencial φ:** como o dock e pura dados, ele pode ser testado unitariamente
sem GPU. makepad precisa do cx para testar dock.

---

### 3.3 virtual scrolling (portallist)

**o que o makepad faz:**
fenwick tree mapeia posicao de scroll para indice de item em o(log n). apenas items
visiveis sao instanciados (~20 simultaneos). fsm de scroll com 6 estados (stopped,
drag, flick, pulldown, scrollingto, tailing).

**o que φ precisa:**

```rust
struct VirtualList {
    height_tree: FenwickTree,       // O(log n) prefix sums
    first_visible: usize,           // ancora de scroll
    scroll_offset: f64,             // offset dentro do primeiro item
    scroll_state: ScrollState,      // FSM de fisica
    item_cache: FxHashMap<usize, Vec<SceneNode>>,  // cache de items renderizados
}
```

sobre a camada existente de φ:
- `FenwickTree` e pura dados, ~150 LOC
- `ScrollState` reutiliza `Spring<f64>` existente para fisica de deceleration
- items renderizados sao `Vec<SceneNode>` cacheados no `item_cache`
- dirty tracking do compositor ja detecta mudanca por hash - items fora da
  janela visivel simplesmente nao sao emitidos, hash da camada muda, upload
  acontece apenas para os nodes visiveis

**LOC estimado:** ~150 fenwick + ~200 virtuallist + ~100 scrollstate. total: ~450 LOC.

**diferencial φ:** o dirty tracking por hash significa que scroll sem mudanca
de conteudo custa zero upload. makepad re-renderiza mesmo items identicos se
a posicao mudou.

---

### 3.4 template + lazy instantiation

**o que o makepad faz:**
`ComponentMap<LiveId, WidgetRef>` armazena widgets instanciados. templates sao
`ScriptObjectRef` (referencia para DSL). widget so e criado quando necessario
(`item_or_create()`). entre frames, widgets persistem no componentmap.

**o que φ precisa:**

```rust
struct ComponentPool<S> {
    templates: FxHashMap<TypeId, Box<dyn Fn() -> Component<S>>>,
    instances: FxHashMap<ComponentId, Component<S>>,
}
```

sobre a camada existente de φ:
- `Component<L>` ja tem lifecycle (mount/update/unmount) e cache de nodes
- `needs_render` flag ja existe - so re-renderizar quando invalidado
- `state_mut()` ja auto-invalida
- falta: pool de instancias reutilizaveis (para virtuallist, onde items saem
  da janela e podem ser reciclados)

**LOC estimado:** ~150 LOC para o pool. reutiliza quase tudo de component.rs.

---

### 3.5 typed action dispatch

**o que o makepad faz:**

```rust
// Widget emite action tipada
cx.action(TabBarAction::TabWasPressed(tab_id));

// Parent faz match
for action in cx.actions() {
    match action.downcast_ref::<TabBarAction>() {
        Some(TabBarAction::TabWasPressed(id)) => { ... }
        _ => {}
    }
}
```

nao ha event bus global. actions sobem pela arvore. o parent decide o que fazer.

**o que φ precisa:**

o sistema de eventos atual (`input/mod.rs`) trata eventos de sistema (mouse, touch,
keyboard). falta a camada de actions de widget. proposta:

```rust
trait WidgetAction: Any + Send {}

struct ActionQueue {
    actions: Vec<(ComponentId, Box<dyn WidgetAction>)>,
}

// Widget emite
action_queue.emit(self.id, MyAction::ButtonPressed);

// Parent consome
for (source, action) in action_queue.drain() {
    if let Some(btn_action) = action.downcast_ref::<ButtonAction>() {
        match btn_action { ... }
    }
}
```

**LOC estimado:** ~100 LOC. combina com task-42 (event dispatch em andamento).

---

### 3.6 theming com variaveis globais

**o que o makepad faz:**
a DSL define variaveis globais de tema:

```
THEME_COLOR_BG = #2A2A2A
THEME_FONT_SIZE = 11.0
```

widgets referenciam: `color: (THEME_COLOR_BG)`. mudar a variavel atualiza todos.

**o que φ precisa:**

```rust
struct Theme {
    colors: FxHashMap<&'static str, Color>,
    sizes: FxHashMap<&'static str, f32>,
    fonts: FxHashMap<&'static str, FontId>,
}

// Uso no builder:
div().bg(theme.color("surface")).p(theme.size("spacing_md"))
```

**LOC estimado:** ~80 LOC. o builder ja aceita color e f32, so precisa do
lookup centralizado.

---

### 3.7 arvore de arquivos (filetree widget)

**o que o makepad faz:**
widget customizado que renderiza arvore hierarquica com indentacao, icones de
pasta, expand/collapse animado. usa portallist internamente para virtualizar
arvores grandes.

**o que φ precisa:**

```rust
struct FileTree {
    nodes: Vec<FileNode>,           // flat list (pre-order traversal)
    expanded: FxHashSet<PathBuf>,   // quais diretorios estao abertos
    virtual_list: VirtualList,      // reutiliza virtual scrolling
    indent_px: f32,                 // pixels por nivel
}
```

cada `FileNode` tem `depth: usize` para calcular indentacao. o flat list e mais
eficiente que arvore recursiva para rendering (acesso sequencial de memoria).

**LOC estimado:** ~250 LOC (com virtuallist ja implementado).

---

### 3.8 editor de codigo

**o que o makepad faz:**
`CodeEditor` widget com:
- syntax highlighting (token coloring, treesitter integration)
- numeros de linha
- cursor blinking, selecao
- scroll virtual (portallist de linhas)
- undo/redo stack

**o que φ precisa (simplificado):**

a base ja existe em `text_input.rs` - textbuffer com cursor, selecao, insert/delete,
blink 530ms. para chegar a editor de codigo:

1. **syntax highlighting:** integrar `tree-sitter` (parsing) + mapa de cores por
   token type. cada linha emite multiplos `SceneNode::Text` com cores diferentes.
   ~300 LOC.

2. **numeros de linha:** coluna fixa a esquerda, virtuallist de linhas. ~50 LOC.

3. **scroll virtual por linhas:** reutiliza virtuallist. cada "item" e uma linha
   do arquivo. ~100 LOC de integracao.

4. **undo/redo:** stack de operacoes (insert, delete, replace) com agrupamento
   por tempo. ~200 LOC.

**total:** ~650 LOC incrementais sobre text_input.rs.

---

### 3.9 mapa de prioridades

para atingir o nivel do demo makepad no browser, na ordem de impacto:

```
Prioridade 1 (fundacao):
  [P1.1] Dock System (paineis + splitters + tabs)          ~700 LOC
  [P1.2] Virtual Scrolling (Fenwick + FSM)                 ~450 LOC
  [P1.3] Typed Action Dispatch                             ~100 LOC
  [P1.4] Theme System                                      ~80 LOC
  Total P1: ~1330 LOC

Prioridade 2 (widgets):
  [P2.1] FileTree widget                                   ~250 LOC
  [P2.2] Code Editor (sobre text_input.rs)                 ~650 LOC
  [P2.3] ComponentPool (lazy instantiation)                ~150 LOC
  Total P2: ~1050 LOC

Prioridade 3 (polish):
  [P3.1] Drag-drop entre paineis                           ~200 LOC
  [P3.2] Resize de splitters com cursor visual             ~100 LOC
  [P3.3] Transicoes animadas (tab switch, panel resize)    ~150 LOC
  [P3.4] Persistencia de layout (serialize/deserialize)    ~100 LOC
  Total P3: ~550 LOC
```

**total geral estimado: ~2930 LOC incrementais** sobre os 12k existentes.

para comparacao, o equivalente no makepad ocupa ~6500 LOC (dock.rs 1767 +
portal_list.rs 2452 + widget.rs 1755 + turtle.rs parcial). φ precisa de
menos da metade porque reutiliza infraestrutura existente (taffy para layout,
gesturerecognizer para drag, dirty tracking por hash, spring para animacao).

---

## parte 4 - diferencas fundamentais de filosofia

| aspecto | φ | makepad |
|---------|------|---------|
| **escopo** | motor de composicao | framework completo + ide |
| **layout** | taffy (flexbox externo) | turtle (cursor interno) |
| **dirty tracking** | hash de 64 bits por camada | comparacao de posicao de rect |
| **separacao** | rendering desacoplado de widgets | tudo acoplado |
| **DSL** | phi_narrate! (proc-macro) | script_mod! (script VM + hot reload) |
| **shaders** | WGSL padrao (wgpu) | DSL propria compilada por plataforma |
| **texto** | cosmic-text (externo) | sistema proprio |
| **tamanho** | 12k LOC (core) | 1m LOC (tudo) |
| **testabilidade** | testes unitarios sem GPU | requer cx para maioria dos testes |
| **hot reload** | nao (ainda) | sim (DSL + shaders) |

### vantagens do φ

1. **testabilidade** - scene graph, signals, animations, dock (futuro) sao todos
   testáveis sem GPU. makepad precisa inicializar cx para testar quase tudo.

2. **modularidade** - posso trocar taffy por outro layout engine sem tocar
   no compositor ou GPU. no makepad, turtle esta entrelaçado com o drawing.

3. **tamanho de codigo** - 12k LOC com 370+ testes. complexidade controlável.

4. **wgpu padrao** - nao inventa abstracão de GPU. beneficia-se de melhorias
   upstream automaticamente.

5. **dirty tracking mais preciso** - hash de 64 bits detecta qualquer mudanca
   (cor, tamanho, posicao, texto). makepad compara rect position apenas.

### vantagens do makepad

1. **hot reload** - editar UI sem recompilar rust. produtividade superior.

2. **widgets prontos** - dock, portallist, filetree, codeeditor, button, slider,
   dropdown, etc. φ tem a base mas nao os widgets.

3. **ecossistema integrado** - shader DSL + layout + widgets + ide = tudo
   funciona junto sem fricção.

4. **virtual scrolling maduro** - fenwick tree, fsm de 6 estados, deceleration
   com fisica. testado com 100k+ items.

5. **drag-drop complexo** - dock restructuring com preview visual, 7 zonas
   de drop por widget.

---

## parte 5 - conclusao

φ nao precisa virar makepad. a forca do φ e a separacao limpa entre
rendering e semantica de UI. o que precisa e construir os **widgets de infraestrutura**
(dock, virtuallist, filetree, codeeditor) *sobre* a camada de composicao existente,
preservando a testabilidade e modularidade.

o investimento estimado de ~2930 LOC para atingir o nivel do demo makepad e viavel
porque a infraestrutura de base ja esta solida:
- dirty tracking por hash ja funciona
- spring analitico ja esta pronto para animacoes de scroll
- gesturerecognizer ja suporta drag-drop
- textinput ja tem cursor + selecao + blink
- taffy ja resolve layout flexbox

o caminho nao e copiar makepad, mas **aprender os padroes certos** (fenwick tree,
docknode recursivo, lazy instantiation, typed actions) e implementa-los de forma
que se integrem naturalmente com a arquitetura hash-based dirty tracking do φ.

a meta e: **mesma qualidade visual, metade do codigo, o dobro dos testes.**
