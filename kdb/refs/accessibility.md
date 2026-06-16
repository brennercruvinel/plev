---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: accessibility
---

# reference analysis: accessibility (accesskit)

data: 2026-03-11
status: completo

## escopo

analise profunda do accesskit como solucao de acessibilidade para o φ. cobertura: arquitetura interna, platform adapters, API surface, integracoes existentes (egui, bevy, slint), mapeamento concreto para a arquitetura do φ (scenenode, hitregion, builder API, event loop winit), e implicacoes para task-30.

---

## accesskit (accesskit/accesskit), 1.4k stars, v0.24.0

**o que e:** infraestrutura de acessibilidade cross-platform para UI toolkits em rust. resolve o problema de toolkits que renderizam seus proprios elementos de UI (como φ faz via wgpu) e que, por isso, nao recebem acessibilidade automatica do sistema operacional. fornece uma abstracao sobre as apis nativas de acessibilidade de cada plataforma, de modo que o toolkit implementa a acessibilidade uma unica vez. schema canonico definido em rust; bindings automaticos para c (cbindgen) e python (pyo3). licenca: apache-2.0 or MIT (parcialmente BSD-3-clause por derivacoes do chromium). usado por 23.1k projetos. 11m+ downloads no crates.io.

**arquitetura:**

o design e inspirado diretamente na arquitetura multi-processo do chromium para acessibilidade:

1. **modelo push-based**: o toolkit (analogia: "renderer process" do chromium) envia ativamente a arvore de acessibilidade para o platform adapter (analogia: "browser process"). o adapter nunca faz pull, apenas recebe dados e responde quando a assistive technology (AT) requisita acoes.

2. **arvore inicial + updates incrementais**: na primeira interacao, o toolkit envia um `TreeUpdate` com a arvore completa. em frames subsequentes, envia apenas os nos que mudaram (`TreeUpdate` com subset de nos + novo focus). isso e fundamental para performance, nao reconstroi a arvore inteira a cada frame.

3. **schema de nos**: cada no da arvore tem:
   - `NodeId` (u64 ou wrapper, identificador estavel entre updates)
   - `Role` (enum com 182 variantes derivadas da spec ARIA, button, label, textinput, genericcontainer, etc.)
   - propriedades opcionais (label, description, bounds, value, children, etc.)
   - acoes suportadas (click, focus, blur, increment, decrement, etc.)

4. **node class sharing**: nos com role, acoes e conjunto de propriedades identicos compartilham uma unica "class" de 100 bytes (reference-counted). isso reduz drasticamente o uso de memoria, de 1.416 bytes/no para ~212 bytes/no (reducao de 5x+).

5. **inicializacao lazy**: o toolkit pode retornar uma arvore placeholder inicialmente. quando uma AT (ex: voiceover) solicita informacao, o `ActivationHandler` e chamado e o toolkit gera a arvore real. para usuarios sem AT ativa, overhead zero.

**crates do workspace:**

| crate | descricao | versao |
|-------|-----------|--------|
| `accesskit` | schema core (node, role, action, treeupdate, tree) | 0.24.0 |
| `accesskit_consumer` | biblioteca consumer platform-independent |, |
| `accesskit_windows` | adapter windows (UI automation) | 0.32.1 |
| `accesskit_macos` | adapter macos (nsaccessibility/appkit) | 0.26.0 |
| `accesskit_unix` | adapter linux/gnome (AT-spi via d-bus/zbus) | 0.21.0 |
| `accesskit_android` | adapter android (java accessibility API) | 0.7.2 |
| `accesskit_winit` | integracao com winit (recomendada) | 0.32.2 |

**downloads crates.io (accesskit core):** 11m+ total. accesskit_unix: 4m+. accesskit_android: 19k.

---

## platform adapters

### macos, nsaccessibility (appkit)
- crate: `accesskit_macos` v0.26.0
- implementa os protocolos nsaccessibility do appkit
- suporta voiceover nativamente
- status: **producao**, usado em egui, bevy, slint
- compativel com metal (nao depende de renderer)

### windows, UI automation (uia)
- crate: `accesskit_windows` v0.32.1
- implementa a API UI automation da microsoft
- suporta narrator, nvda, jaws
- status: **producao**, primeiro adapter implementado, mais maduro
- includes exemplos com win32 e sdl

### linux/gnome, AT-spi (d-bus)
- crate: `accesskit_unix` v0.21.0
- implementa interfaces AT-spi via zbus (d-bus async)
- suporta orca (gnome screen reader)
- status: **producao**, "almost ready" no blog virou producao, 4m downloads
- feature opcional no accesskit_winit (habilitada por padrao em linux)

### android, java accessibility API
- crate: `accesskit_android` v0.7.2
- implementa a API de acessibilidade java do android via JNI
- suporta talkback
- status: **early/experimental**, 19k downloads, mas ja presente no accesskit_winit como feature opcional
- funciona com android-activity/gameactivity (mesmo setup que φ usa)

### ios, nsaccessibility (uikit)
- status: **planejado, nao implementado**
- o blog do accesskit lista ios como futuro, dependente de funding
- nao existe crate `accesskit_ios` publicado
- implicacao para φ: ios sera a plataforma sem a11y automatica via accesskit no curto prazo

### web/WASM, ARIA
- status: **planejado, nao implementado**
- o autor reconhece como "provavelmente o mais dificil" dos adapters
- a ideia: gerar elementos DOM ocultos com atributos ARIA que espelham a arvore de acessibilidade do canvas webgpu
- nao existe crate `accesskit_web` publicado
- implicacao para φ: WASM target precisara de solucao manual (DOM overlay) ou aguardar o adapter

---

## integracao com egui

egui (emilk/egui) foi o primeiro toolkit pure-rust e o primeiro immediate-mode gui a integrar accesskit. a integracao foi feita pelo proprio autor do accesskit (matt campbell) no pr #2294, merged dezembro 2022.

**padrao de integracao:**

1. **feature flag**: `accesskit` e feature habilitada por padrao no `eframe`, mas desabilitada nos crates internos (`egui`, `egui-winit`). o crate de nivel mais alto decide.

2. **inicializacao lazy**: o contexto egui so comeca a gerar arvores accesskit quando a integracao (eframe) habilita o suporte. ate la, retorna placeholders. quando uma AT solicita dados, `ActivationHandler` e chamado, que chama `request_repaint()` no egui, que no proximo frame gera a arvore real.

3. **construcao da arvore**: cada widget egui (button, label, slider, dragvalue, checkbox) gera um `accesskit::Node` com:
   - role apropriado (button -> role::button, label -> role::label)
   - `label` com o texto visivel
   - `bounds` com o rect do widget
   - acoes suportadas (click, focus, increment/decrement para sliders)
   - `children` referenciando sub-widgets

4. **treeupdate por frame**: a cada repaint, egui constroi um `TreeUpdate` completo com todos os nos visíveis. o adapter faz diff interno, nao e preciso o toolkit calcular delta.

5. **action handling**: o `ActionHandler` recebe acoes da AT em qualquer thread. as acoes sao enfileiradas e processadas no proximo frame do egui. para click: simula o input no widget. para focus: move focus. para setvalue: atualiza valor do slider/input.

6. **thread safety**: `ActionHandler::do_action()` pode ser chamado de qualquer thread. egui resolve isso enfileirando acoes (mutex) e processando na thread principal no proximo frame.

**licao para φ:** o padrao lazy + placeholder e essencial para nao impactar performance de quem nao usa AT. a construcao de treeupdate por frame (sem delta manual) simplifica a implementacao, o adapter faz o diffing.

---

## integracao com bevy

bevy (bevyengine/bevy) integrou accesskit na v0.10, tornando-se o primeiro game engine de proposito geral com acessibilidade first-party.

**padrao de integracao (ECS):**

1. **component `AccessibilityNode`**: wrapper de `accesskit::Node`. adicionado como component em entidades ECS.

2. **hierarquia automatica**: se uma entidade com `AccessibilityNode` tem parent que tambem tem `AccessibilityNode`, vira child na arvore. caso contrario, vira child direto da janela root.

3. **system de update**: um system ECS percorre entidades com `AccessibilityNode` e constroi o `TreeUpdate` quando ha mudancas (changed query).

4. **focus via resource**: `Focus` resource armazena qual entidade tem keyboard focus. mapeado para `TreeUpdate.focus`.

5. **desde bevy 0.15**: `accesskit` nao e mais re-exportado por `bevy_a11y`, usuarios adicionam como dependencia separada.

**licao para φ:** o padrao de "wrapper component que traduz para accesskit::node" e diretamente aplicavel. em φ, o equivalente seria um campo `accessibility: Option<AccessibilityInfo>` no element do builder, traduzido para `accesskit::Node` durante o resolve.

---

## integracao com winit (accesskit_winit)

**versao atual:** 0.32.2 (4 marco 2026)
**dependencia winit:** `^0.30.5`, **compativel com φ que usa winit 0.30**

o `accesskit_winit` e o crate recomendado para integrar accesskit com winit. ele abstrai a criacao e gerenciamento do platform adapter correto para cada OS.

**API principal, `Adapter`:**

```rust
// Construtor via EventLoopProxy (recomendado para φ)
Adapter::with_event_loop_proxy<T>(
    window: &Window,
    proxy: EventLoopProxy<T>,
) -> Self
where T: From<accesskit_winit::Event> + Send + 'static

// Construtor com handlers diretos
Adapter::with_direct_handlers(
    window: &Window,
    activation_handler: impl ActivationHandler + 'static + Send,
    action_handler: impl ActionHandler + 'static + Send,
    deactivation_handler: impl DeactivationHandler + 'static + Send,
) -> Self

// Construtor hibrido
Adapter::with_mixed_handlers<T>(
    window: &Window,
    activation_handler: impl ActivationHandler + 'static + Send,
    proxy: EventLoopProxy<T>,
) -> Self
```

**metodos de instancia:**

```rust
// DEVE ser chamado ANTES do app processar cada window event
adapter.process_event(window: &Window, event: &WinitWindowEvent)

// Aplica update apenas se a arvore ja foi ativada
adapter.update_if_active(updater: impl FnOnce() -> TreeUpdate)
```

**fluxo de integracao com winit 0.30:**

1. criar `EventLoop::<AppEvent>::with_user_event()` (φ ja faz isso)
2. no `resumed()`, criar o `Adapter` com o proxy do event loop
3. em `window_event()`, chamar `adapter.process_event()` antes de processar o evento
4. ao receber `accesskit_winit::Event` via user_event, processar action requests
5. apos cada mudanca de UI, chamar `adapter.update_if_active(|| tree_update)`

**implicacao critica:** o `AppEvent` de φ precisara de uma nova variante para eventos accesskit:
```rust
pub enum AppEvent {
    GpuReady { ... },
    AccessKit(accesskit_winit::Event),  // NOVO
}
```

e `AppEvent` precisa implementar `From<accesskit_winit::Event>`.

---

## WASM support

**status atual: nao suportado.**

- nao existe crate `accesskit_web` publicado
- o autor do accesskit reconhece como o adapter mais dificil de implementar
- a abordagem planejada: criar elementos DOM ocultos com ARIA attributes que espelham a arvore de acessibilidade
- prioridade dependente de funding e contribuicoes voluntarias
- nao ha timeline publica

**alternativa para φ (WASM):**
- gerar manualmente um DOM overlay com `<div role="button" aria-label="...">` ocultos que espelham o scene graph
- usar `web-sys` para manipular o DOM (φ ja depende de web-sys)
- cada hitregion geraria um elemento DOM posicionado absolutamente, invisivel visualmente mas acessivel por screen readers
- esta e a mesma abordagem que o adapter accesskit planeja automatizar

---

## API surface

### tipos principais

**`NodeId`**: wrapper sobre `NonZeroU128` (ou u64 em pratica). identificador unico e estavel para cada no da arvore.

**`Node`**: no da arvore de acessibilidade. construido via `Node::new(role: Role)`. 212 bytes base. propriedades via getters/setters.

metodos principais:
- `new(role: Role) -> Self`
- `set_role(Role)` / `role() -> Role`
- `set_children(Vec<NodeId>)` / `push_child(NodeId)` / `children() -> &[NodeId]`
- `set_label(impl Into<Box<str>>)` / `label() -> Option<&str>`
- `set_description(impl Into<Box<str>>)` / `description() -> Option<&str>`
- `set_value(impl Into<Box<str>>)` / `value() -> Option<&str>`
- `set_bounds(Rect)` / `bounds() -> Option<Rect>`
- `set_text_selection(TextSelection)`
- `add_action(Action)` / `remove_action(Action)` / `supports_action(Action) -> bool`
- flags booleanas: `set_hidden()`, `set_disabled()`, `set_read_only()`, `set_selected()`, `set_expanded()`, etc.
- propriedades numericas: `set_numeric_value(f64)`, `set_min_numeric_value(f64)`, `set_max_numeric_value(f64)`, `set_numeric_value_step(f64)`
- scroll: `set_scroll_x(f64)`, `set_scroll_y(f64)`, `set_scroll_x_min/max`, `set_scroll_y_min/max`
- texto: `set_character_lengths(&[u8])`, `set_word_starts(&[u8])`, `set_text_align(TextAlign)`, `set_text_direction(TextDirection)`
- `set_transform(Affine)`, transformacao 2d
- `set_toggled(Toggled)`, para checkboxes/switches

**`Role`**: enum com 182 variantes. as mais relevantes para φ:

| role | uso em φ |
|------|-------------|
| `GenericContainer` | scenenode::rect sem semantica especifica, div() |
| `Label` / `StaticText` | scenenode::text (texto nao editavel) |
| `TextRun` | texto inline dentro de paragrafo |
| `Button` | hitregion com on_click |
| `Link` | elemento navegavel |
| `TextInput` | campo de texto editavel (task-28) |
| `Image` | image() no builder |
| `Group` | containerview, agrupamento de elementos |
| `Heading` | titulos com nivel (set_level) |
| `List` / `ListItem` | listas |
| `ScrollView` | area scrollavel |
| `Slider` | range input com min/max/value |
| `CheckBox` / `Switch` | toggles |
| `Dialog` | modais |
| `Window` | no root da arvore (a janela) |
| `ProgressIndicator` | loading/progress bars |

**`Action`**: enum com 22 variantes:

| action | descricao |
|--------|-----------|
| `Click` | equivale a single click/tap |
| `Focus` | transfere keyboard focus |
| `Blur` | remove focus |
| `Collapse` / `Expand` | colapsa/expande elemento |
| `Increment` / `Decrement` | altera valor numerico +/- 1 step |
| `SetValue` | define valor (requer actiondata::value) |
| `SetTextSelection` | define selecao de texto |
| `ScrollDown/Up/Left/Right` | scroll direcional |
| `ScrollIntoView` | torna no visivel via scroll |
| `ScrollToPoint` | scroll para coordenada especifica |
| `ShowContextMenu` | exibe menu de contexto |
| `ReplaceSelectedText` | substitui texto selecionado |
| `ShowTooltip` / `HideTooltip` | tooltip visibility |
| `CustomAction` | acao customizada (requer actiondata) |

**`TreeUpdate`**: struct enviada ao adapter a cada mudanca:

```rust
pub struct TreeUpdate {
    pub nodes: Vec<(NodeId, Node)>,  // nos novos ou alterados
    pub tree: Option<Tree>,          // metadados da arvore (presente no primeiro update)
    pub focus: NodeId,               // no com keyboard focus atual
}
```

**`Tree`**: metadados da arvore (enviado uma vez):
```rust
pub struct Tree {
    pub root: NodeId,      // no raiz
    pub app_name: Option<String>,
    pub toolkit_name: Option<String>,
    pub toolkit_version: Option<String>,
}
```

**traits:**

- `ActivationHandler`: chamado quando AT solicita arvore. retorna `Option<TreeUpdate>`, none para lazy init.
- `ActionHandler`: chamado quando AT requisita acao (click, focus, etc.). metodo: `do_action(ActionRequest)`.
- `DeactivationHandler`: chamado quando AT desconecta. permite cleanup.

---

## performance

**overhead com AT inativa:** zero. inicializacao lazy significa que nenhuma arvore e construida ate uma AT solicitar. a feature habilitada no cargo.toml adiciona ~15kb ao binario (medido no hello_world do egui em x86-64 windows).

**overhead com AT ativa:**
- construcao da arvore: o(n) onde n = numero de nos visíveis. nao e shaping/rendering, apenas populacao de structs.
- memoria: ~212 bytes/no com node class sharing. para 100 elementos UI = ~21kb.
- treeupdate diff: feito internamente pelo adapter. o toolkit pode enviar a arvore completa a cada frame, o adapter calcula o delta. simplifica a implementacao.
- thread model: `ActionHandler::do_action()` pode ser chamado de qualquer thread. acoes devem ser enfileiradas e processadas na main thread.

**otimizacoes de memoria (blog "dramatically reducing memory usage"):**
- antes: 1.416 bytes/no (struct flat com todos os campos)
- depois: 212 bytes/no (32 bytes base + 80 bytes props + 100 bytes node class compartilhada)
- tecnica: armazenamento dinamico de propriedades com arrays pareados (indices de 1 byte + valores de 40 bytes), inspirado em librsvg
- node class sharing: nos com mesmo role + acoes + set de propriedades reusam uma unica class (rc/arc)
- reducao total: 5x+

---

## mapeamento para φ

### scenenode -> accesskit node

| φ | accesskit role | propriedades |
|------|---------------|--------------|
| `SceneNode::Rect` | `Role::GenericContainer` | `bounds(Rect{x,y,w,h})` |
| `SceneNode::Text` | `Role::Label` | `bounds`, `label(texto)`, `set_text_direction` |
| `SceneNode::Text` (editavel, task-28) | `Role::TextInput` | `bounds`, `value(texto)`, `add_action(Focus)`, text selection |
| hitregion (on_click) | `Role::Button` | `bounds`, `label`, `add_action(Click)`, `add_action(Focus)` |
| hitregion (link/nav) | `Role::Link` | `bounds`, `label`, `set_url` |
| containerview | `Role::Group` | `bounds`, `children` |
| layer (root) | `Role::Group` | `children`, `set_hidden(!visible)`, `set_opacity?` (n/a, accesskit nao tem opacity) |
| janela root | `Role::Window` | `children([layer_ids])`, usado como root da tree |

**geracao de nodeid:**
- `ViewId(u64)` do φ ja e um identificador unico estavel
- converter para `NodeId`: `NodeId(NonZeroU128::new(view_id.0 as u128 + 1).unwrap())`
- reservar nodeid(1) para o window root
- cada layer pode ter nodeid baseado em `LayerId.0 + offset`

### hitregion -> role mapping

a hitregion ja contem toda a informacao geometrica necessaria:

```rust
// Atual HitRegion
pub struct HitRegion {
    pub view_id: ViewId,  // -> NodeId
    pub x: f32,           // -> bounds.min_x
    pub y: f32,           // -> bounds.min_y
    pub w: f32,           // -> bounds.max_x = x + w
    pub h: f32,           // -> bounds.max_y = y + h
    pub focusable: bool,  // -> add_action(Focus) se true
    pub layer_visible: bool, // -> set_hidden(!layer_visible)
    pub layer_opacity: f32,  // -> set_hidden(layer_opacity == 0.0)
}
```

o role da hitregion depende do contexto semantico, o builder API deve fornecer via `.role()`.

### builder API extensions

extensoes necessarias no `Element` (builder.rs):

```rust
// Novos campos em Element
pub struct Element {
    // ... campos existentes ...
    a11y_role: Option<accesskit::Role>,
    a11y_label: Option<String>,
    a11y_description: Option<String>,
    a11y_value: Option<String>,
}

// Novos metodos no builder
impl Element {
    pub fn role(mut self, role: accesskit::Role) -> Self { ... }
    pub fn label(mut self, label: &str) -> Self { ... }
    pub fn description(mut self, desc: &str) -> Self { ... }
    pub fn aria_value(mut self, val: &str) -> Self { ... }
}

// Exemplo de uso
button("Enviar")
    .role(Role::Button)       // explicito (inferivel de button())
    .label("Enviar formulario") // label mais descritivo para AT
    .on_click(|_| { ... })
```

**inferencia automatica de role:**
- `button()` -> `Role::Button` automatico
- `text()` -> `Role::Label` automatico
- `div()` -> `Role::GenericContainer` automatico
- `image()` -> `Role::Image` automatico
- `.role()` permite override explicito

### event loop integration com winit

φ usa `ApplicationHandler<AppEvent>`. mudancas necessarias:

**1. estender appevent:**
```rust
pub enum AppEvent {
    GpuReady { gpu: GpuContext, text_system: TextSystem, ... },
    AccessKit(accesskit_winit::Event),  // NOVO
}

impl From<accesskit_winit::Event> for AppEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        AppEvent::AccessKit(event)
    }
}
```

**2. criar adapter no resumed():**
```rust
fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    // ... criar window ...
    let adapter = Adapter::with_event_loop_proxy(&window, proxy.clone());
    self.accessibility_adapter = Some(adapter);
}
```

**3. interceptar window events:**
```rust
fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    // ANTES de qualquer processamento
    if let Some(ref mut adapter) = self.accessibility_adapter {
        adapter.process_event(&window, &event);
    }
    // ... processamento existente ...
}
```

**4. processar accesskit events:**
```rust
fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
    match event {
        AppEvent::GpuReady { .. } => { /* existente */ }
        AppEvent::AccessKit(ak_event) => {
            // Processar action requests da AT
            self.handle_accessibility_action(ak_event);
        }
    }
}
```

**5. atualizar arvore apos render:**
```rust
fn render(&mut self) {
    // ... build scene, resolve, render_pass ...

    // APOS render, atualizar arvore de acessibilidade
    if let Some(ref mut adapter) = self.accessibility_adapter {
        adapter.update_if_active(|| self.build_accessibility_tree());
    }
}
```

---

## implicacoes para task-30

### recomendacoes concretas

**1. dependencias cargo.toml:**
```toml
[dependencies]
accesskit = "0.24"
accesskit_winit = "0.32"
```
nenhuma outra dependencia necessaria, accesskit_winit puxa os platform adapters corretos via cfg.

**2. novo modulo `src/accessibility.rs`:**
- `AccessibilityState` struct: mantem mapa `ViewId -> NodeId`, focus atual, flag de ativacao
- `fn build_tree(compositor: &Compositor, input_state: &InputState) -> TreeUpdate`, percorre layers + nodes + hit_regions e gera a arvore
- `fn handle_action(request: ActionRequest, input_state: &mut InputState)`, traduz acoes AT para eventos φ

**3. plataformas cobertas automaticamente pelo accesskit_winit 0.32.2:**
- macos: sim (nsaccessibility, voiceover)
- windows: sim (UI automation, narrator/nvda/jaws)
- linux: sim (AT-spi, orca)
- android: sim (talkback), feature opcional, necessita teste
- ios: nao (adapter nao existe)
- WASM: nao (adapter nao existe)

**4. gap de plataformas:**
- ios: sem accesskit. opcao futura: wrapper manual de uiaccessibility, ou aguardar accesskit_ios.
- WASM: sem accesskit. opcao: DOM overlay manual com aria-* attributes via web-sys. φ ja depende de web-sys. viavel como implementacao separada.

**5. estimativa de LOC atualizada:**
- `src/accessibility.rs`: ~300-400 LOC (state + tree builder + action handler)
- mudancas em `window.rs`: ~50 LOC (appevent variant, adapter lifecycle, event routing)
- mudancas em `builder.rs`: ~60 LOC (role, label, description, aria_value fields + metodos)
- mudancas em `input/mod.rs`: ~30 LOC (expor semantica do hitregion para a11y)
- testes: ~150-200 LOC
- exemplo: ~100-150 LOC
- **total: ~700-900 LOC** (consistente com estimativa original de 800-1200)

**6. riscos identificados:**
- `accesskit_winit::Event` precisa ser `From<>` para `AppEvent`, e `AppEvent` ja nao e `Clone` (contem gpucontext). verificar que os variants sao independentes.
- o adapter deve ser criado antes da window ser exibida pela primeira vez (restricao do accesskit_winit). em φ, a window e criada no `resumed()`, o adapter deve ser criado imediatamente apos `create_window()`.
- actionhandler e chamado de qualquer thread, acoes devem ser enfileiradas (ex: `Arc<Mutex<Vec<ActionRequest>>>`) e drenadas na main thread antes do render.
- android: `accesskit_android` e feature opcional do accesskit_winit. precisa habilitar explicitamente no cargo.toml ou por cfg.

**7. ordem de implementacao sugerida:**
1. adicionar dependencias e criar `src/accessibility.rs` com struct basica
2. estender appevent com variante accesskit
3. criar adapter no `resumed()`, interceptar events em `window_event()`
4. implementar `build_tree()` percorrendo hitregions (primeiro sem texto)
5. testar com voiceover no macos (botoes do showcase)
6. adicionar mapeamento de scenenode::text -> label nodes
7. estender builder API com `.role()`, `.label()`
8. implementar action handling (focus, click)
9. escrever testes unitarios
10. criar `examples/accessibility_demo.rs`

**8. compatibilidade com features existentes:**
- dirty tracking do compositor: nao afetado. a arvore de a11y e construida a partir dos hitregions e scenenodes, nao altera o pipeline de rendering.
- signal system: pode conectar sinais a labels dinamicos (ex: contador que atualiza label do texto).
- layout (taffy): computedbounds ja fornece as coordenadas para `set_bounds()`.
- effects: layers com blur/shadow nao afetam a11y (sao visuais puras).
- texto (cosmic-text): textnodekey.text fornece o conteudo textual para labels.

---

## referencias

- [github accesskit/accesskit](https://github.com/AccessKit/accesskit)
- [accesskit website](https://accesskit.dev/)
- [how it works](https://accesskit.dev/how-it-works/)
- [accesskit crate docs](https://docs.rs/accesskit/latest/accesskit/)
- [accesskit_winit crate docs](https://docs.rs/accesskit_winit/latest/accesskit_winit/)
- [accesskit node docs](https://docs.rs/accesskit/latest/accesskit/struct.Node.html)
- [accesskit role docs](https://docs.rs/accesskit/latest/accesskit/enum.Role.html)
- [egui accesskit pr #2294](https://github.com/emilk/egui/pull/2294)
- [bevy accesskit announcement](https://accesskit.dev/accesskit-integration-makes-bevy-the-first-general-purpose-game-engine-with-built-in-accessibility-support/)
- [memory optimization blog](https://accesskit.dev/dramatically-reducing-accesskits-memory-usage/)
- [accesskit roadmap](https://accesskit.dev/looking-back-looking-forward/)
- [crates.io accesskit](https://crates.io/crates/accesskit)
- [crates.io accesskit_winit](https://crates.io/crates/accesskit_winit)
- [accesskit_winit adapter API](https://librepvz.github.io/librePvZ/accesskit_winit/struct.Adapter.html)
- [bevy a11y module](https://docs.rs/bevy/latest/bevy/a11y/index.html)
- [slint accesskit pr](https://github.com/slint-ui/slint/pull/2865)
- [libraries.io accesskit_winit](https://libraries.io/cargo/accesskit_winit)
