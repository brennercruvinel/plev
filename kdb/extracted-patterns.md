---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: research
---

# extracted patterns, 56 reference repos

**data:** 2026-03-11
**task:** task-34
**objetivo:** patterns concretos extraidos do codigo-fonte, aplicaveis ao φ

---

## metodologia

cada pattern inclui:
- **repo fonte** e arquivo/funcao
- **descricao** do que faz e como
- **code snippet** essencial
- **aplicabilidade ao φ**
- **decisao:** adopt (usar como esta) / adapt (modificar para φ) / ignore (nao relevante) / already implemented

---

## fase a, prioridade imediata (accesskit, parley, lyon, glam)

### a0-ak1. lazy activation via three-handler protocol (accesskit)
- **fonte:** accesskit, `common/src/lib.rs` l2882-2931, `platforms/winit/src/lib.rs` l81-265
- **descricao:** accesskit usa 3 handler traits: `ActivationHandler` (screen reader conecta), `ActionHandler` (roteia acoes AT de volta ao app), `DeactivationHandler` (screen reader desconecta). o app faz zero trabalho de acessibilidade ate um screen reader conectar, quando `request_initial_tree()` e chamado.
- **snippet:**
```rust
pub trait ActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate>;
}
pub trait ActionHandler {
    fn do_action(&mut self, request: ActionRequest);
}
pub trait DeactivationHandler {
    fn deactivate_accessibility(&mut self);
}
```
- **aplicabilidade:** φ ja usa winit 0.30 e `EventLoopProxy`. o pattern mapeia direto. na `InitialTreeRequested`, setar `is_accesskit_active: bool`, forcar repaint, e construir a arvore inicial do scenenode graph + hit_regions. quando desativado, limpar a flag, custo zero quando nao ha AT.
- **decisao:** **adopt**, pattern exato para task-30. usar `Adapter::with_event_loop_proxy`.

### a0-ak2. per-frame treeupdate com acumulador parent-map (egui)
- **fonte:** egui, `crates/egui/src/pass_state.rs` l70-74, `crates/egui/src/context.rs` l507-520, l2600-2621
- **descricao:** egui acumula nodes durante o frame usando dois maps: `nodes: IdMap<Node>` e `parent_map: IdMap<Id>`. no inicio do frame, cria root window node. durante o frame, cada widget chama `accesskit_node_builder(id, |builder| { ... })` que insere lazily e auto-parenta (encontra o ancestral mais proximo que ja tem node accesskit). no fim do frame, drena para `TreeUpdate`.
- **aplicabilidade:** o φ reconstroi o scenenode graph a cada frame via `build_scene()`. durante `build_scene`, ao empurrar cada scenenode, tambem empurrar accesskit node num acumulador. os `hit_regions` do φ ja contem a maioria da info necessaria. acumulador `Option<AccessKitFrameState>`, `None` quando AT inativo (custo zero).
- **decisao:** **adapt**, usar `FxHashMap` (consistente com todos os caches do φ) e amarrar ao `ViewId` existente.

### a0-ak3. widget-to-role mapping com bounds injection (egui)
- **fonte:** egui, `crates/egui/src/response.rs` l830-910
- **descricao:** cada widget tem `fill_accesskit_node_common` (bounds, focusability) e `fill_accesskit_node_from_widget_info` (mapeia tipo -> role + label/value). bounds sao injetados do layout rect ja computado, nao recomputados. a arvore de acessibilidade fica automaticamente em sync com o visual.
- **mapeamento para φ:**
  - `SceneNode::Rect` sem interacao -> skip ou `Role::GenericContainer`
  - `SceneNode::Text` -> `Role::Label` (estatico) ou `Role::TextInput` (editavel)
  - textinput component -> `Role::TextInput` com `set_value()`, `set_placeholder()`
  - hitregion clicavel -> `Role::Button`
  - `ComputedBounds` (layout) -> `accesskit::Rect`
- **decisao:** **adapt**, derivar bounds de `ComputedBounds`/`HitRegion`, nao de scenenode. trait `AccessibleView` com `accessibility_role()` e `accessibility_label()`.

### a0-ak4. id mapping, viewid para nodeid (egui)
- **fonte:** egui, `crates/egui/src/id.rs` l82-84
- **descricao:** conversao trivial: `Id(NonZeroU64)` -> `NodeId(u64)` via `.value().into()`. root id constante: `Id::new("accesskit_root")`. sem lookup table.
- **aplicabilidade:** φ's `ViewId(u32)` -> `NodeId(view_id.0 as u64)`. root reservado em `NodeId(0)` (viewid comeca em 1).
- **decisao:** **adopt**, cast direto, sem lookup table.

### a0-ak5. focus routing via actionrequest (egui)
- **fonte:** egui, `crates/egui/src/memory/mod.rs` l565-606
- **descricao:** AT envia `ActionRequest { action: Action::Focus, target: NodeId }`. egui armazena `id_requested_by_accesskit`. no proximo frame, ao registrar widgets focusaveis, verifica se AT pediu foco naquele id. se sim, concede foco e limpa o request.
- **aplicabilidade:** o `InputState` do φ ja tem `focused_view: Option<ViewId>`. request AT de foco traduz para: `focused_view = Some(ViewId(target.0 as u32))` + emitir `InputEvent::Focus` sintetico. `Action::Click` -> click sintetico no centro dos bounds do target.
- **decisao:** **adopt**, bridge fino: `ActionRequest -> InputEvent`.

### a0-ak6. null platform adapter para WASM (accesskit)
- **fonte:** accesskit, `platforms/winit/src/platform_impl/null.rs` (24 linhas)
- **descricao:** para platforms sem AT nativo (WASM), accesskit tem adapter null com mesma interface que nao faz nada. dispatch compile-time via `#[path]` com cfg gates. `update_if_active()` em WASM compila para no-op.
- **aplicabilidade:** φ targeta 6 platforms. macos/windows/linux/android tem AT nativo coberto por accesskit. WASM usa null adapter. nenhum codigo de platform customizado necessario no φ.
- **decisao:** **adopt as-is**, depender do dispatch de platform do accesskit. zero overhead em WASM.

---

### a1. byte-index cursor com affinity (parley)
- **fonte:** parley, `parley/src/layout/cursor.rs`
- **descricao:** `Cursor` armazena `(usize, Affinity)` onde affinity desambigua posicao em quebras de linha soft. no fim de uma linha soft, o cursor pode estar "no fim da linha atual" ou "no inicio da proxima", o byte index e o mesmo, a affinity diferencia. φ's `TextBuffer` nao tem conceito de affinity.
- **aplicabilidade:** necessario quando o texto faz wrapping. sem affinity, o cursor pula para posicao errada em quebras de linha.
- **decisao:** **adapt**, implementar quando melhorar text editing (pos task-32 assessment).

### a2. selection geometry via callback (parley)
- **fonte:** parley, `parley/src/layout/` (selection API)
- **descricao:** `Selection::geometry_with(|rect, line_idx|)` emite retangulos de highlight sem alocacao, caminhando cluster-por-cluster com tratamento de RTL e inline boxes. nao retorna `Vec<Rect>`, usa callback para zero-alloc.
- **snippet:**
```rust
selection.geometry_with(|rect, line_idx| {
    // rect: Rect com coordenadas exatas do highlight
    // Emitir quad de selecao para o compositor
});
```
- **aplicabilidade:** o φ nao tem API de renderizacao de selecao. essencial para text input com selecao visual.
- **decisao:** **adopt**, usar este pattern ao implementar selecao visual no textinput.

### a3. plaineditordriver pattern (parley)
- **fonte:** parley, `parley/src/editor.rs` (plaineditordriver)
- **descricao:** separa estado do editor (`PlainEditor`) do contexto emprestado (`FontContext` + `LayoutContext`). o driver encapsula os borrows temporariamente, resolvendo o desafio do borrow-checker de mutar texto enquanto segura referencia ao font system. similar ao borrow split pattern do φ em text.rs, mas mais ergonomico.
- **aplicabilidade:** o `TextInput` do φ sofre do mesmo problema, precisa do font system para medir texto, mas quer mutar o buffer. o driver pattern formalizaria isso.
- **decisao:** **adapt**, aplicar ao refatorar textinput quando/se migrar para parley.

### a4. inlinebox, elementos non-text no fluxo (parley)
- **fonte:** parley, `parley/src/layout/` (inlinebox)
- **descricao:** elementos nao-texto embedados no fluxo de texto via `{id, index, width, height}`. permite inserir icones, imagens, badges inline com o texto. cosmic-text nao tem isso.
- **aplicabilidade:** major vantagem de parley para rich text. habilita: icones inline, badges, chips, emojis customizados. relevante para task-32 assessment.
- **decisao:** **adapt**, considerar como criterio de decisao em task-32 (parley vs cosmic-text).

### a5. geometrybuilder trait, tessellation desacoplada (lyon)
- **fonte:** lyon, `crates/tessellation/src/` (filltessellator, geometrybuilder trait)
- **descricao:** `GeometryBuilder` desacopla tessellation do formato de output. `FillVertexConstructor<QuadVertex>` produz vertices que mapeiam 1:1 para o `QuadVertex { position: [f32; 2], color: [f32; 4] }` existente no φ. **nenhum shader novo necessario**, paths tessellados renderizam pelo pipeline de quads existente.
- **snippet:**
```rust
struct WithColor([f32; 4]);
impl FillVertexConstructor<QuadVertex> for WithColor {
    fn new_vertex(&mut self, vertex: FillVertex) -> QuadVertex {
        QuadVertex {
            position: vertex.position().to_array(),
            color: self.0,
        }
    }
}
```
- **aplicabilidade:** task-31 fica muito mais simples. nao precisa de `path.wgsl` separado, reusar `quad.wgsl`. tessellate uma vez, armazenar index ranges, draw por muitos frames. integra com dirty tracking existente.
- **decisao:** **adopt**, usar como arquitetura base para task-31. elimina necessidade de novo pipeline/shader.

### a6. lyon+wgpu integration template (lyon)
- **fonte:** lyon, `examples/wgpu/` (exemplo completo)
- **descricao:** demonstra a arquitetura exata: `#[repr(C)] #[derive(Pod, Zeroable)]` vertex, `FillVertexConstructor` bridge, tessellate once + store index ranges + draw many frames. mapeia diretamente para o modelo de dirty tracking do φ.
- **aplicabilidade:** template direto para task-31. o pattern "tessellate once, draw many" e exatamente o que o dirty tracking per-layer do φ ja faz para quads.
- **decisao:** **adopt**, usar como template de implementacao para task-31.

### a7. glam vec2 com bytemuck pod
- **fonte:** glam-rs, `src/f32/vec2.rs`
- **descricao:** `Vec2 { x: f32, y: f32 }` e `#[repr(C)]` com `Pod/Zeroable` opcional via feature. SIMD backends: SSE2/NEON/wasm-simd128, mas so beneficia `Vec4`/`Mat4` (vec2 esta abaixo do width SIMD de 16 bytes). o φ faz math vetorial minima atualmente.
- **aplicabilidade:** adiciona conveniencia mas nao valor suficiente para justificar a dependencia na phase 2. reavaliar quando task-31 (lyon) criar mais math vetorial.
- **decisao:** **ignore por agora**, reavaliar em task-31 se surgir necessidade.

---

## fase b, rendering e compositing (vello, makepad, xilem)

### b1. stream-of-arrays encoding (vello)

**fonte:** vello, `vello_encoding/src/encoding.rs` l26-53
**descricao:** vello armazena a cena como arrays paralelos tipados (structure-of-arrays) em vez de `Vec<SceneNode>`. transforms e styles sao encodados apenas quando *mudam* em relacao ao valor anterior (l206-210, l217-228), deduplicando estado redundante via `self.styles.last() != Some(&style)`.
```rust
pub struct Encoding {
    pub path_tags: Vec<PathTag>,
    pub path_data: Vec<u32>,
    pub draw_tags: Vec<DrawTag>,
    pub draw_data: Vec<u32>,
    pub transforms: Vec<Transform>,
    pub styles: Vec<Style>,
    pub resources: Resources,   // late-bound: glyphs, images, gradients
}
```

**aplicabilidade:** φ usa `Vec<SceneNode>` (aos). para o rasterizer de quads+text via wgpu, aos e adequado. porem, a *deduplicacao de transforms/styles* e aplicavel: se 50 rects tem mesma cor, evitar 50 arrays `[f32;4]` identicos.

**decisao:** **ignore** (layout soa) / **adapt** (deduplicacao transform/style), prioridade baixa ate scenenode count exceder centenas.

---

### b2. scene fragment / subtree caching via append() (vello)

**fonte:** vello, `vello/src/scene.rs` l461-467, `vello_encoding/src/encoding.rs` l94-172
**descricao:** `Scene::append(other, transform)` faz merge de uma cena filha na pai como o(n) memcpy (extend_from_slice em cada stream). habilita **fragment caching**: widget constroi sub-cena uma vez, armazena o `Scene`, e faz append a cada frame sem re-encoding.
```rust
pub fn append(&mut self, other: &Self, transform: Option<Affine>) {
    let t = transform.as_ref().map(Transform::from_kurbo);
    self.encoding.append(&other.encoding, &t);
}
```
masonry mantem scenes per-widget em cache e so chama `widget.paint()` quando `request_paint` e true.

**aplicabilidade:** φ reconstroi todos os scenenodes a cada frame em `build_scene()`. para regioes estaticas (headers, sidebars), cachear `Vec<SceneNode>` e reconstruir apenas quando dirty evitaria construcao de cena e o fxhash dirty check. pattern de maior impacto para performance.

**decisao:** **adapt**, introduzir `SceneFragment` cacheavel per-component. components com `needs_render() == false` reutilizam fragmento cacheado. combina com `prev_hash` existente.

---

### b3. late-bound resource resolution com epoch-based eviction (vello)

**fonte:** vello, `vello_encoding/src/resolve.rs` l157-387, `vello_encoding/src/glyph_cache.rs` l88-132
**descricao:** vello separa encoding (rapido, sem GPU) de resolution (toca caches e atlases). recursos sao `Patch` variants durante encoding. `Resolver::resolve()` roda uma vez por frame.

o glyph cache eviction (`maintain()`) e particularmente sofisticado: so itera o cache a cada 64 frames e quando total > 256. entries com serial > max_age sao recicladas para free-list (max 32):
```rust
// GlyphCache::maintain()
if serial - self.last_prune_serial < PRUNE_FREQUENCY
    && self.cached_count < CACHED_COUNT_THRESHOLD { return; }
self.map.retain(|_, entry| {
    if serial - entry.serial > MAX_ENTRY_AGE {
        self.free_list.push(entry.encoding.clone()); // recicla buffer
        false
    } else { true }
});
```

**aplicabilidade:** φ ja tem resolve em duas fases. o pattern epoch/serial de eviction e mais sofisticado que LRU puro: glyph usado 3 frames atras nao e evicted se ha capacidade. free-list evita re-alocacao. aplicavel ao shaping_cache e glyph atlas.

**decisao:** **adapt** (estrategia de eviction), adicionar serial/epoch a shaping cache do φ. evict somente entries mais velhas que n frames and quando capacidade excedida.

---

### b4. turtle layout, layout + draw simultaneo (makepad)

**fonte:** makepad, `draw/src/turtle.rs` l558-600, `draw/src/shader/draw_quad.rs` l111-125
**descricao:** "turtle" e cursor-based layout que computa layout e emite draw calls num *unico passe*:
```rust
pub fn begin(&mut self, cx: &mut Cx2d, walk: Walk, layout: Layout) {
    cx.begin_turtle(walk, layout);  // push layout cursor
    let new_area = cx.add_aligned_instance(&self.draw_vars);
}
pub fn end(&mut self, cx: &mut Cx2d) {
    let rect = cx.end_turtle();  // pop cursor, compute final rect
    self.draw_vars.area.set_rect(cx, &rect);
}
```
`Walk { width: Size, height: Size }` onde `Size` e `Fixed(f64)` | `Fill { weight, min, max }` | `Fit { min, max }`. fill-weights distribui espaco restante proporcionalmente. tres flow modes: `Right`, `Down`, `Overlay`.

beneficio: zero-allocation layout. custo: order-dependent, nao-reversivel.

**aplicabilidade:** φ usa taffy 0.9 flexbox com constraint solver completo, escolha correta para compositing engine. turtle e rapido mas menos capaz. a abstracao `Size::Fill { weight }` e clean e poderia enriquecer layoutstyle futuramente.

**decisao:** **ignore** (modelo de layout) / **note** (`Size::Fill { weight }`) para futuros refinamentos.

---

### b5. instanced draw call batching (makepad)

**fonte:** makepad, `draw/src/shader/draw_quad.rs` l135-184, `draw/src/draw_list_2d.rs` l239-274
**descricao:** makepad agrupa draw calls por tipo de shader. todas instancias do mesmo shader acumulam num unico buffer GPU:
```rust
pub fn draw(&mut self, cx: &mut Cx2d) {
    if let Some(mi) = &mut self.many_instances {
        mi.instances.extend_from_slice(self.draw_vars.as_slice());
    } else if self.draw_vars.can_instance() {
        cx.add_aligned_instance(&self.draw_vars);
    }
}
```
`ManyInstances` toma ownership temporaria do instance buffer via `std::mem::swap`, preenche no CPU, e devolve. `DrawVars` e `#[repr(C)]` e diretamente `as_slice()`-able como instance data raw. `find_appendable_drawcall()` tenta mergear com draw call existente do mesmo shader. centenas de quads = 1 draw call.

**aplicabilidade:** φ agrupa quads e text em 2 draw calls por layer, mas e vertex-based (4 vertices + 6 indices por quad). instanced rendering usa 1 instancia por quad com geometria de 2-triangulos fixa, cada instancia carrega rect_pos/rect_size. reduziria vertex buffer 4x.

**decisao:** **adapt** (otimizacao futura), trocar para instanced quads quando shape count crescer apos task-31. geometria fixa de 2-triangulos + per-instance data (position, size, color, corner_radius).

---

### b6. view/element/widget tree separation (xilem)

**fonte:** xilem, `xilem_core/src/view.rs` l53-102, `xilem_core/src/element.rs` l1-108
**descricao:** xilem separa UI em tres arvores distintas:
1. **view tree** (transiente, reconstruida a cada frame): structs leves com `build()`, `rebuild()`, `teardown()`, `message()`.
2. **element tree** (retida): tipos com gat `Mut<'a>` para acesso mutavel.
3. **widget tree** (retida, nodes de render em masonry): trait `Widget`.

```rust
pub trait View<State, Action, Context: ViewPathTracker>: ViewMarker + 'static {
    type Element: ViewElement;
    type ViewState;
    fn build(&self, ctx: &mut Context, app_state: &mut State)
        -> (Self::Element, Self::ViewState);
    fn rebuild(&self, prev: &Self, view_state: &mut Self::ViewState,
               ctx: &mut Context, element: Mut<'_, Self::Element>, app_state: &mut State);
    fn teardown(...);
    fn message(...) -> MessageResult<Action>;
}
```
`rebuild()` recebe `self` (nova) e `prev` (anterior) para diffing. static typing torna diff extremamente eficiente.

**aplicabilidade:** o `View` do φ (`fn render(&self, cx) -> Vec<SceneNode>`) re-avalia a cada frame. para compositing engine, tres arvores e overkill. mas `build()` + `rebuild(prev)` com diff typed e diretamente aplicavel ao component lifecycle.

**decisao:** **adapt** (component trait), adicionar `fn needs_render(&self, prev: &Self) -> bool` como metodo default-true que components override para otimizacao.

---

### b7. memoization com partialeq data (xilem)

**fonte:** xilem, `xilem_core/src/views/memoize.rs` l56-166
**descricao:** `memoize(data, |data| view_fn)` reconstroi somente quando input data muda:
```rust
fn rebuild(&self, prev: &Self, view_state: &mut Self::ViewState, ...) {
    if core::mem::take(&mut view_state.dirty) || prev.data != self.data {
        let view = (self.init_view)(&self.data);
        view.rebuild(&view_state.view, ...);
        view_state.view = view;
    }
}
```
detalhes chave:
- closures zero-sized forcadas em compile-time: `assert!(size_of::<InitView>() == 0)`.
- usa `PartialEq`, nao hashing, sem colisoes.
- flag `dirty` setada por `message()` quando `RequestRebuild` propaga.
- variante `frozen()` nunca reconstroi.

**aplicabilidade:** φ usa fxhash no nivel de layer. xilem e mais granular: memoizacao per-subtree. para φ, mapeia para component-level: se props nao mudaram (partialeq), pular `render()` e reutilizar `Vec<SceneNode>` cacheado.

**decisao:** **adapt**, implementar memoizacao por componente. opera antes do fxhash (antes de gerar scene nodes).

---

### b8. dirty flag bubbling com merge_up (masonry/xilem)

**fonte:** xilem, `masonry_core/src/core/widget_state.rs` l69-264, l345-358
**descricao:** `WidgetState` tem dirty flags granulares: `request_xxx` (este widget) e `needs_xxx` (este widget ou descendente). flags propagam via `merge_up`:
```rust
pub(crate) fn merge_up(&mut self, child_state: &mut Self) {
    self.needs_layout |= child_state.needs_layout;
    self.needs_compose |= child_state.needs_compose;
    self.needs_paint |= child_state.needs_paint;
    self.needs_anim |= child_state.needs_anim;
    self.needs_accessibility |= child_state.needs_accessibility;
}
```
passes checam `needs_xxx` para pular sub-arvores inteiras. documentacao alerta sobre "zombie flags" (flags que nunca sao limpos, causando re-renders perpetuos). flags separados por tipo: `request_layout`, `request_paint`, etc. mudar cor so dispara repaint, nao relayout.

**aplicabilidade:** dirty tracking do φ e no nivel de layer (fxhash unico). nao distingue "needs layout" vs "needs paint". o merge_up permitiria pular sub-arvores durante scene building. flags relevantes para φ: `needs_scene_rebuild`, `needs_layout`, `needs_paint`, `needs_composite`.

**decisao:** **adapt** (prioridade media), adicionar dirty flags per-component com `request_`/`needs_` e propagacao `merge_up` quando arvore de componentes maturar.

---

### b9. per-widget scene caching (masonry paint pass)

**fonte:** xilem, `masonry_core/src/passes/paint.rs` l57-170
**descricao:** paint pass cacheia `Scene` objects por widget. apenas os cujas flags `request_` estao ativas sao re-renderizados:
```rust
let (pre_scene, scene, post_scene) = scene_cache.entry(id).or_default();
if state.request_paint {
    scene.reset();
    widget.paint(&mut ctx, &props, scene);
}
complete_scene.append(scene, Some(transform));
```
widget inalterado reutiliza scene cacheada; custo e apenas o memcpy do `append()`.

**aplicabilidade:** implementacao concreta do b2 no nivel widget. φ poderia manter `HashMap<ComponentId, Vec<SceneNode>>` como scene cache. components inalterados pulam `render()` e nodes cacheados sao pushados na layer.

**decisao:** **adapt**, maior prioridade para performance. scene cache per-component complementa dirty tracking per-layer: componente pula geracao, layer pula upload GPU.

---

## fase c, animacao e motion (natura, keyframe, mina)

### c1. analytical spring solver (pre-computed coefficients)
- **fonte:** natura, `natura/src/spring.rs` l73-271
- **descricao:** natura nao usa euler integration. pre-computa 4 coeficientes (`pos_pos_coef`, `pos_vel_coef`, `vel_pos_coef`, `vel_vel_coef`) baseados na solucao analitica da ode do oscilador harmonico amortecido. trata 3 regimes: sub-amortecido (exp * sin/cos), criticamente amortecido (exp * polinomio), super-amortecido (duas exponenciais distintas). o `update()` e apenas duas operacoes multiply-add, sem branching, sem dt, sem erro de acumulacao.
- **snippet:**
```rust
pub fn update(&mut self, pos: f64, vel: f64, equilibrium_pos: f64) -> (f64, f64) {
    let old_pos = pos - equilibrium_pos;
    let old_vel = vel;
    let new_pos = old_pos * self.pos_pos_coef + old_vel * self.pos_vel_coef + equilibrium_pos;
    let new_vel = old_pos * self.vel_pos_coef + old_vel * self.vel_vel_coef;
    (new_pos, new_vel)
}
```
- **aplicabilidade:** o `Spring<T>::tick()` do φ usa forward euler (`velocity += acceleration * dt; value += velocity * dt`). isso causa: (1) instabilidade numerica com springs rigidos ou dt grande, (2) dependencia de frame-rate (resultados diferentes a 30fps vs 60fps). a solucao analitica e incondicionalmente estavel para qualquer dt.
- **decisao:** **adapt**, substituir o solver euler interno do `Spring<T>::tick()` por coeficientes analiticos pre-computados. manter a API existente (`stiffness/damping/mass`), adicionar `damping_ratio()` como conveniencia.

### c2. keyframesequence com easing per-segment
- **fonte:** keyframe, `src/sequence.rs` l19-330, `src/keyframe.rs` l12-105
- **descricao:** `AnimationSequence<T>` armazena `Vec<Keyframe<T>>` ordenado, cada keyframe com valor, timestamp, e seu proprio `Arc<dyn EasingFunction>`. cada segmento entre keyframes pode ter easing diferente. suporta: `advance_by(duration)`, `advance_and_maybe_reverse(duration)` (ping-pong), `advance_and_maybe_wrap(duration)` (looping), `now()` (valor interpolado atual).
- **snippet:**
```rust
let mut sequence = keyframes![
    (0.5, 0.0),           // EaseInOut default, t=0.0
    (1.5, 0.3, Linear),   // Linear a partir de t=0.3
    (2.5, 1.0)            // fim em t=1.0
];
sequence.advance_by(0.65);
assert_eq!(sequence.now(), 2.0);
```
- **aplicabilidade:** o `Tween<T>` do φ so suporta animacao de dois pontos (from, to) com easing unico. para animacoes multi-step (ex: botao que fade-in, segura, depois slide-out), usuarios precisariam encadear tweens manualmente. keyframesequence preenche essa lacuna.
- **decisao:** **adopt**, adicionar `KeyframeSequence<T>` ao modulo de animacao do φ com easing per-segment, advance/reverse/wrap, e builder API.

### c3. state animator com transition blending
- **fonte:** mina, `core/src/animator.rs` l34-210
- **descricao:** `StateAnimator` e um state machine que mapeia valores de `State` enum para `Timeline` instances. quando `set_state()` e chamado: (1) o keyframe 0% da nova timeline e sobrescrito com os valores animados atuais via `start_with()`, (2) a duracao reseta para zero, (3) a cada `advance()` interpola de onde a animacao anterior parou ate o alvo da nova timeline. se o novo estado nao tem timeline, a animacao anterior e pausada (nao resetada) e retoma quando o estado original retorna. cria transicoes suaves: um botao a 60% da animacao hover que recebe click vai blend suavemente do estado visual atual para a animacao pressed.
- **aplicabilidade:** o φ nao tem conceito de estados de animacao ou blending de transicoes. em UI, botoes tem idle/hover/pressed/disabled, cada um com animacoes diferentes. o pattern integraria bem com o sistema de signals e lifecycle de componentes do φ.
- **decisao:** **adapt**, o stateanimator completo do mina (enummap, proc macros, mergedtimeline) e pesado demais. extrair o pattern core "state -> timeline + blend-on-transition" num `AnimationState<S, T>` leve que mapeia estados para tweens/keyframesequences e faz blending ao mudar estado.

### c4. timeline repeat/reverse/delay
- **fonte:** mina, `core/src/time_scale.rs` l13-155; keyframe, `advance_and_maybe_reverse`/`advance_and_maybe_wrap`
- **descricao:** `TimeScale` do mina converte tempo real em tempo normalizado [0.0, 1.0], tratando: delay (periodo antes de iniciar), repeat (none, times(n), infinite), reverse/ping-pong (tempo sobe 0->1 na primeira metade, desce 1->0 na segunda).
- **snippet:**
```rust
let (normalized_time, is_reversing) = match self.reverse {
    true if cycle_ratio > 0.5 => ((1.0 - cycle_ratio) * 2.0, true),
    true => (cycle_ratio * 2.0, false),
    false => (cycle_ratio, false),
};
```
- **aplicabilidade:** o `Tween<T>` do φ nao tem delay, repeat, nem reverse. necessario para: pulsing glow (repeat infinite + reverse), loading spinner (repeat infinite), entrada com delay, bounce-in (repeat times(1) + reverse).
- **decisao:** **adopt**, adicionar `delay`, `repeat` (enum: none/times(u32)/infinite), e `reverse` (bool) ao `Tween<T>`. ortogonais a funcionalidade existente, mudancas minimas de codigo.

### c5. const-generic array interpolate
- **fonte:** keyframe, `src/easing.rs` l47-69
- **descricao:** `CanTween` implementado para `[T; N]` via const generics, cobrindo todos os tamanhos de array numa unica impl ao inves de impls separadas para `[f32; 2]`, `[f32; 3]`, `[f32; 4]`.
- **snippet:**
```rust
impl<T: CanTween, const N: usize> CanTween for [T; N] { ... }
// Em φ: std::array::from_fn(|i| self[i].lerp(&target[i], t))
```
- **aplicabilidade:** o φ implementa `Interpolate` manualmente para `[f32; 2]`, `[f32; 3]`, `[f32; 4]` e `SpringInterpolate` para os mesmos tres. sao ~50 linhas de boilerplate substituiveis por duas impls const-generic. suporta automaticamente `[f32; 5]`, `[f32; 6]`, etc.
- **decisao:** **adopt**, substituir as 6 impls manuais por 2 const-generic. φ usa edition 2024, rust >= 1.92, const generics totalmente disponiveis.

### c6. step/hold easing functions
- **fonte:** keyframe, `src/functions/static_functions.rs` l18-40
- **descricao:** `Step` (snap em 0.5, arredonda para endpoint mais proximo) e `Hold` (sempre retorna valor inicial, snap para fim em t=1.0). essenciais para animacoes de propriedades discretas (visibilidade toggle, troca de sprite frame).
- **snippet:**
```rust
pub struct Step;
impl EasingFunction for Step {
    fn y(&self, x: f64) -> f64 { x.round() }
}
pub struct Hold;
impl EasingFunction for Hold {
    fn y(&self, _x: f64) -> f64 { 0.0 }
}
```
- **aplicabilidade:** os 31 easing variants do φ nao incluem step nem hold. necessarios para propriedades discretas dentro de keyframesequences.
- **decisao:** **adopt**, adicionar `Easing::Step` e `Easing::Hold`. duas linhas de match arms.

---

## fase d, UX patterns de tui apps (yazi, television, bottom)

### d1. event batching + render throttle (yazi)
- **fonte:** yazi, `yazi-fm/src/app/app.rs` l27-65
- **descricao:** pre-aloca vec com capacidade 50, drena ate 50 eventos por iteracao via `recv_many()`. apos processar cada evento, checa flag atomico `NEED_RENDER`. se render necessario mas <10ms desde ultimo render, defer via `tokio::select!`, ou timeout de 10ms dispara (render), ou mais eventos chegam (processa primeiro). previne o(n) renders por keypress burst.
- **snippet (conceitual):**
```
loop {
    if let Some(t) = timeout.take() {
        select! {
            _ = sleep(t) => { render(); }
            n = rx.recv_many(&mut events, 50) => { drain_events!(); }
        }
    }
}
```
- **aplicabilidade:** o event queue do φ (`src/input/mod.rs`) processa eventos individualmente. durante input rapido (touch/keyboard), cada evento dispara scene rebuild + GPU submit. batch-drain: acumular eventos winit em `window_event()`, processar todos em `about_to_wait()`, depois um unico `compositor.resolve()` + render_pass. especialmente valioso em mobile (touch move a 120hz+).
- **decisao:** **adapt**, alta prioridade. 5-10x reducao de trabalho GPU durante input rapido.

### d2. partial vs full render flag (yazi)
- **fonte:** yazi, `NEED_RENDER` atomicu8 (0=none, 1=full, 2=partial)
- **descricao:** 3 estados: nenhum render, render completo, render parcial (so overlays de progresso/notificacao). parcial e muito mais barato que full.
- **aplicabilidade:** extende o dirty tracking per-layer do φ. alem de "layer dirty/clean", adicionar estados expliciticos "skip frame" e "animation-only" (so atualizar layers com animacao ativa).
- **decisao:** **adapt**, prioridade media. reduz frames desperdicados durante idle.

### d3. layer-based action routing (yazi)
- **fonte:** yazi, 11-variant `Layer` enum com keymaps per-layer
- **descricao:** router resolve teclas para acoes baseado na layer ativa. executor despacha por layer. fallthrough (cmp -> input) habilita modais composiveis. ex: mesmo esc fecha modal ou cancela busca, dependendo da layer ativa.
- **aplicabilidade:** o φ precisa de inputcontext para roteamento modal de teclado. quando textinput esta focado, teclas vao para text editing. quando um dialog esta aberto, esc fecha o dialog, nao o textinput.
- **decisao:** **adapt**, prioridade media. necessario para apps reais com modais/dialogs.

### d4. priority task scheduler com cancellation (yazi)
- **fonte:** yazi, `async_priority_channel` com low/normal/high, `CompletionToken` per-task
- **descricao:** 3 niveis de prioridade. tasks verificam cancellation token via `tokio::select!`. high: operacoes criticas de frame. normal: trabalho visivel em breve. low: background.
- **aplicabilidade:** mapeia para trabalho GPU do φ: frame-critical (render pass) / visible-soon (tessellation de paths off-screen) / background (pre-shaping de texto).
- **decisao:** **adopt**, quando construir sistema de tasks async.

### d5. two-tier plugin isolation (yazi)
- **fonte:** yazi, VM lua compartilhada para UI + vms isoladas slim por task background
- **descricao:** componentes UI compartilham uma VM (acesso a state, rendering). tasks background rodam em vms isoladas slim (sem acesso a state da UI). 6 tabelas globais curadas como sistema de capabilities.
- **aplicabilidade:** arquitetura transfere para WASM: instancia compartilhada para extensoes UI, isolada para tasks.
- **decisao:** **adapt**, referencia para task-33 (plugin architecture).

### d6. channel/injector data abstraction (television)
- **fonte:** television, `Channel<P: EntryProcessor>`
- **descricao:** wraps carregamento async de dados (batch de 10k, 4 flushes concorrentes) com queries sincronas de `Matcher` por frame. bridge entre fonte de dados async e rendering sincronico.
- **aplicabilidade:** pattern para apps φ com dados async (ex: dashboard que carrega dados de API enquanto renderiza skeleton/placeholder).
- **decisao:** **adapt**, para futuras apps demo. nao necessario no engine core.

### d7. render gating por tipo de acao (television)
- **fonte:** television, `affects_results()` + `should_render()` com 5 heuristicas
- **descricao:** classifica acoes: "afeta resultados" vs "so UI". `should_render()` combina: first ticks, interval, running state, UI actions, reload suppression. skip render para acoes que nao mudam nada visual.
- **aplicabilidade:** complementa dirty tracking do φ: antes de computar hash, verificar se a acao e conhecida como no-op. evita ate o custo de hashing.
- **decisao:** **adapt**, prioridade media. otimizacao de segundo nivel.

### d8. auto-navigation from layout geometry (bottom)
- **fonte:** bottom, layout TOML tree + algoritmo two-pass
- **descricao:** arvore de layout em TOML. algoritmo two-pass computa vizinhos direcionais (cima/baixo/esquerda/direita) a partir das posicoes de widgets. navegacao por arrow keys automatica baseada na geometria real do layout. funcao: `compute_focus_graph(layout) -> FocusGraph`.
- **aplicabilidade:** essencial para task-30 (accessibility). screen readers e tab navigation precisam saber a ordem espacial dos elementos. `ComputedBounds` do taffy -> focusgraph com vizinhos direcionais.
- **decisao:** **adapt**, alta prioridade. necessario para task-30.

### d9. widget maximize/restore toggle (bottom)
- **fonte:** bottom, `is_expanded: bool` + branch no render path
- **descricao:** um boolean `is_expanded` e um branch no render: widget expande para full-screen, estado preservado entre toggles.
- **aplicabilidade:** trivialmente implementavel via layer visibility do φ (layer normal + layer maximizada).
- **decisao:** **adopt**, prioridade baixa. util para futuras apps.

### d10. configurable update rate (bottom)
- **fonte:** bottom, thread de coleta com sleep configuravel (default 1000ms, minimo 250ms)
- **descricao:** thread de coleta de dados separada do render loop. sleep configuravel. cancellable sleep para shutdown limpo.
- **aplicabilidade:** utility `PollTimer` para apps data-driven construidas sobre φ.
- **decisao:** **adapt**, prioridade baixa.

---

## fase e, WASM runtime patterns (waforth, extism)

### e1. waforth shared table+memory interop
- **fonte:** waforth, compilacao de words para modulos WASM em runtime
- **descricao:** compila forth words para modulos WASM emitindo bytecodes raw em linear memory, depois instancia com `table` e `memory` imports compartilhados. o principio: modulos WASM novos podem compartilhar estado com o host via imports explicitamente declarados.
- **aplicabilidade:** muito low-level para plugins do φ, mas o principio de shared-table interop e relevante para task-33.
- **decisao:** **watch**, referencia para task-33, nao adotar diretamente.

### e2. extism plugin lifecycle com fuel+epoch sandboxing
- **fonte:** extism, builder pattern para plugins, manifest-based config
- **descricao:** builder pattern para construcao de plugins. manifest define: hosts permitidos, limites de memoria, timeouts. i/o via offsets em linear memory. host functions como closures tipadas. dual fuel+epoch limiting para sandboxing: fuel conta instrucoes executadas, epoch verifica periodicamente timeout wall-clock.
- **snippet (conceitual):**
```rust
let plugin = Plugin::new(manifest, host_functions, /* fuel_limit */ true)?;
plugin.call::<&str, &str>("my_function", "input")?;
```
- **aplicabilidade:** template direto para task-33 (WASM plugin architecture). o pattern manifest + host functions + fuel limiting cobre exatamente o que o φ precisa para plugins isolados.
- **decisao:** **adapt**, usar como referencia arquitetural para task-33.

---

## fase f, competidores (dioxus, slint, leptos)

### f1. fxindexset para subscriber ordering (leptos)
- **fonte:** leptos, `reactive_graph/src/` (subscriber management)
- **descricao:** leptos usa `FxIndexSet` (rustc-hash + indexmap) em vez de vec/hashset para subscribers. garante: (1) iteration em ordem de insercao (efeitos outer rodam antes de inner), (2) o(1) contains/insert/remove (vs o(n) para vec), (3) sem duplicatas. o φ usa `Vec<NodeId>` com `contains()` o(n).
- **aplicabilidade:** o signal system do φ (`signal.rs`) armazena subscribers/sources em vec. com muitos signals, `contains()` linear degrada. fxindexset resolve corretude (ordem) + performance (o(1)).
- **decisao:** **adopt**, alta prioridade. substituir `Vec<NodeId>` por `FxIndexSet<NodeId>` em signal.rs.

### f2. peek() untracked read + drop-guard write (dioxus)
- **fonte:** dioxus, `packages/signals/` (signal API)
- **descricao:** `peek()` le o valor sem se inscrever como subscriber (opt-out local no ponto de leitura). write guards disparam notificacao de subscribers no `Drop` (batching natural, multiplas escritas no mesmo escopo so notificam uma vez quando o guard sai de escopo).
- **aplicabilidade:** o φ nao tem leitura untracked. util quando um componente precisa ler um signal sem criar dependencia (ex: logging, debug views, one-shot reads). drop-guard write e refinamento futuro.
- **decisao:** **adapt**, adicionar `ReadSignal::peek()` ao φ. drop-guard write como melhoria futura.

### f3. lazy binding + constant-signal sentinel (slint)
- **fonte:** slint, `internal/core/properties.rs` (property<t>)
- **descricao:** bindings so recomputam quando `Property::get()` e chamado (pull on demand, nao push eagerly). properties que nunca foram escritas apontam para um sentinel statico, pulando todo o tracking. isso significa que properties com valores default (a maioria num layout) tem custo zero de tracking.
- **aplicabilidade:** o signal system do φ registra tracking para todos os signals, mesmo os que nunca mudam (ex: labels estaticos, cores fixas). um sentinel para "nunca escrito" evitaria overhead desnecessario.
- **decisao:** **adopt**, otimizacao de constant-signal: skip tracking para signals nunca escritos.

### f4. RAII observer drop guard (leptos)
- **fonte:** leptos, `reactive_graph/src/` (observer management)
- **descricao:** `Observer::replace()` retorna `SetObserverOnDrop` guard que restaura o observer anterior quando dropado. previne corrupcao de stack se a closure panic. o φ usa push/pop explicito que e unsafe em caso de panic.
- **snippet (conceitual):**
```rust
let _guard = observer.replace(new_observer);
// Se panic aqui, o Drop do guard restaura o observer anterior
// ... executar computacao ...
// guard dropado normalmente -> restaura observer anterior
```
- **aplicabilidade:** o signal runtime do φ (`with_runtime`) usa push/pop explicito no observer stack. se uma closure dentro de `create_effect` panic, o observer stack fica corrompido. RAII guard previne isso.
- **decisao:** **adopt**, alta prioridade. substituir push/pop explicito por RAII guard em signal.rs.

### f5. dioxus delega rendering nativo ao vello
- **fonte:** dioxus, renderer nativo
- **descricao:** dioxus nao tem renderer GPU customizado, wraps vello. isso valida a posicao estrategica do φ como renderer GPU leve e fragment-only, sem o overhead de compute shaders do vello.
- **aplicabilidade:** validacao estrategica. confirma que o nicho do φ (GPU-first compositing sem compute shaders) e diferenciado. dioxus depende de terceiros para rendering; φ e o renderer.
- **decisao:** **strategic validation**, nenhum codigo a adotar.

---

## resumo de decisoes

| # | pattern | fonte | decisao | task alvo |
|---|---------|-------|---------|-----------|
| a0-ak1 | lazy activation protocol | accesskit | adopt | task-30 |
| a0-ak2 | per-frame treeupdate accumulator | egui | adapt | task-30 |
| a0-ak3 | widget-to-role mapping | egui | adapt | task-30 |
| a0-ak4 | viewid -> nodeid cast | egui | adopt | task-30 |
| a0-ak5 | focus routing via actionrequest | egui | adopt | task-30 |
| a0-ak6 | null platform adapter (WASM) | accesskit | adopt | task-30 |
| a1 | byte-index cursor + affinity | parley | adapt | task-32 |
| a2 | selection geometry via callback | parley | adopt | task-32 |
| a3 | plaineditordriver pattern | parley | adapt | task-32 |
| a4 | inlinebox (non-text in flow) | parley | adapt | task-32 |
| a5 | geometrybuilder trait | lyon | adopt | task-31 |
| a6 | lyon+wgpu template | lyon | adopt | task-31 |
| a7 | glam vec2 bytemuck | glam | ignore | - |
| b1 | stream-of-arrays encoding | vello | ignore/adapt | future |
| b2 | scene fragment caching (append) | vello | **adapt** | component cache |
| b3 | epoch-based cache eviction | vello | adapt | text.rs |
| b4 | turtle layout | makepad | ignore | - |
| b5 | instanced draw call batching | makepad | adapt | post-task-31 |
| b6 | view/element/widget separation | xilem | adapt | component trait |
| b7 | memoize + partialeq data | xilem | **adapt** | component cache |
| b8 | dirty flag bubbling (merge_up) | masonry | adapt | component tree |
| b9 | per-widget scene caching | masonry | **adapt** | component cache |
| c1 | analytical spring solver | natura | adapt | animation.rs |
| c2 | keyframesequence + easing/segment | keyframe | adopt | animation.rs |
| c3 | state animator + blending | mina | adapt | animation.rs |
| c4 | timeline repeat/reverse/delay | mina/keyframe | adopt | animation.rs |
| c5 | const-generic array interpolate | keyframe | adopt | animation.rs |
| c6 | step/hold easing | keyframe | adopt | animation.rs |
| f1 | fxindexset for subscribers | leptos | adopt | signal.rs |
| f2 | peek() untracked read | dioxus | adapt | signal.rs |
| f3 | constant-signal sentinel | slint | adopt | signal.rs |
| f4 | RAII observer drop guard | leptos | adopt | signal.rs |
| d1 | event batching + render throttle | yazi | **adapt** | input/window.rs |
| d2 | partial vs full render flag | yazi | adapt | compositor.rs |
| d3 | layer-based action routing | yazi | adapt | input system |
| d4 | priority task scheduler | yazi | adopt | future async |
| d5 | two-tier plugin isolation | yazi | adapt | task-33 |
| d6 | channel/injector abstraction | television | adapt | future apps |
| d7 | render gating por tipo de acao | television | adapt | compositor.rs |
| d8 | auto-navigation from layout | bottom | **adapt** | task-30 |
| d9 | widget maximize/restore | bottom | adopt | future apps |
| d10 | configurable update rate | bottom | adapt | future utility |
| e1 | shared table+memory interop | waforth | watch | task-33 |
| e2 | plugin lifecycle + sandboxing | extism | adapt | task-33 |
| f1 | fxindexset for subscribers | leptos | **adopt** | signal.rs |
| f2 | peek() untracked read | dioxus | adapt | signal.rs |
| f3 | constant-signal sentinel | slint | **adopt** | signal.rs |
| f4 | RAII observer drop guard | leptos | **adopt** | signal.rs |
| f5 | dioxus delegates to vello | dioxus | strategic note | - |

---

## top 10, prioridade de implementacao

| rank | pattern | impacto | esforco | task |
|------|---------|---------|---------|------|
| 1 | c1 analytical spring solver | corrige bug de corretude (frame-rate dependency) | ~100 LOC | animation.rs |
| 2 | d1 event batching | 5-10x reducao GPU work em input rapido | ~50 LOC | window.rs/input |
| 3 | f4 RAII observer guard | previne corrupcao em panic | ~30 LOC | signal.rs |
| 4 | f1 fxindexset subscribers | o(1) vs o(n) + ordem garantida | ~20 LOC | signal.rs |
| 5 | b2/b9 scene fragment cache | pula render() para components inalterados | ~200 LOC | compositor/component |
| 6 | a5/a6 lyon geometrybuilder | habilita vector paths no quad pipeline | ~600 LOC | task-31 |
| 7 | ak1-ak6 accesskit integration | acessibilidade completa | ~900 LOC | task-30 |
| 8 | c2 keyframesequence | maior feature gap em animacao | ~300 LOC | animation.rs |
| 9 | d8 auto-navigation layout | essencial para task-30 a11y | ~150 LOC | layout/accessibility |
| 10 | c4 repeat/reverse/delay | patterns comuns de animacao | ~80 LOC | animation.rs |

**total: 38 patterns extraidos de 17 repos em 6 fases.**
