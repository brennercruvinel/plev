# plev — Plano Técnico de Implementação

> Documento de engenharia. Sem cronogramas ou estimativas — apenas arquitetura,
> tarefas, contratos e ordem de dependência. Cada workstream (WS) é uma branch
> longa (`ws/<nome>`) com dono exclusivo de diretórios, integrada em `main` por
> milestones de dependência (M0–M4).

---

## 1. Visão e princípios

Evoluir o plev de protótipo para um engine de UI GPU-native nível GPUI/Zed, e o
basicIDE para uma aplicação real (git client/IDE), aproveitando que a stack
atual (wgpu + winit + cosmic-text + taffy) é idêntica ao backend Linux atual do
GPUI (`gpui_wgpu`, Apache-2.0 — referência legalmente copiável).

Princípios extraídos do estudo de engines (GPUI, Vello/Parley, Makepad, Slint,
Floem/Lapce, iced/COSMIC, egui/Rerun, Helix/Xi):

1. **Poucas primitivas especializadas, instanced rendering** — um shader por
   primitiva, batching por tipo+textura em ordem de pintura (GPUI).
2. **Render sob demanda é obrigatório; damage parcial de pixels é opcional** —
   GPUI redesenha a janela inteira, mas só quando algo invalida (lição COSMIC:
   reactive rendering = 60–80% menos CPU).
3. **Texto medido de verdade no layout** via measure functions do taffy, com
   `disable_rounding()` + arredondamento próprio a device pixel.
4. **Estado retained, árvore de elementos immediate** — entidades/handles com
   effect queue (run-to-completion, sem reentrância).
5. **Um tema próprio polido > temas nativos por plataforma** (lição Slint
   mar/2026). Design language: shadcn/ui (receita validada pelo gpui-component).
6. **Acessibilidade cedo, como infraestrutura de teste** (padrão egui_kittest:
   dirigir a UI pela árvore AccessKit + screenshots headless).
7. **Não gastar novelty points** (retrospectiva Xi): nada de CRDT, plugins WASM
   ou colaboração antes do núcleo estar sólido.
8. **Isolar dependências de risco atrás de traits** — cosmic-text → Parley é a
   migração que o ecossistema inteiro está fazendo (Floem, Slint, Bevy, egui);
   o engine não pode acoplar API pública à lib de texto.

---

## 2. Arquitetura alvo

```
┌────────────────────────────────────────────────────────────────────┐
│ apps: basicIDE · snakeGame · scene3D · showcase (galeria widgets)  │
├────────────────────────────────────────────────────────────────────┤
│ plev::ui        widgets retained (Button, Input, List, Table, …)   │
│ plev::editor    widget Editor (view sobre editor_core)             │
│ plev::actions   Action + Keymap + CommandPalette + focus tree      │
├────────────────────────────────────────────────────────────────────┤
│ plev::builder   árvore de elementos immediate (Styled/Refinement)  │
│ plev::signal    reativo push-pull (existente, mantido)             │
│ plev::layout    taffy + MeasureFn (texto real)                     │
│ plev::text      trait TextBackend → impl CosmicText (→ Parley?)    │
├────────────────────────────────────────────────────────────────────┤
│ plev::scene     Scene v2: primitivas + batches instanciados        │
│ plev::gpu       pipelines, frame pacing, atlas, instrumentação     │
├────────────────────────────────────────────────────────────────────┤
│ crates externos: editor_core (rope/seleções) · git_backend (gix)   │
└────────────────────────────────────────────────────────────────────┘
```

Crates novos (workspace):

| Crate | Conteúdo | Depende de GPU? |
|---|---|---|
| `crates/editor_core` | Rope, Transaction, Selections, History | Não (100% testável) |
| `crates/git_backend` | Status/log/diff/stage/commit via gix | Não |
| `crates/plev_kittest` | Harness de teste via AccessKit + headless | wgpu headless |

---

## 3. WS-0 · `ws/contracts` — Contratos de fronteira

**Pré-requisito de todos os outros. Apenas traits + stubs, sem implementação.**

### 3.1 `TextBackend` (isola cosmic-text)

```rust
pub trait TextBackend {
    type Shaped;                       // linha/parágrafo shaped, cacheável
    fn shape(&mut self, text: &str, attrs: &[StyleRun], max_width: Option<f32>) -> Self::Shaped;
    fn measure(&mut self, text: &str, attrs: &[StyleRun], avail: AvailableSpace) -> Size;
    fn hit_test(&self, shaped: &Self::Shaped, pos: Point) -> CursorPos;   // ponto → byte offset
    fn cursor_geometry(&self, shaped: &Self::Shaped, cursor: CursorPos) -> Rect;
    fn line_height(&self, attrs: &TextStyle) -> f32;
}

pub struct StyleRun { pub range: Range<usize>, pub style: TextStyle }  // rich text por spans
```

`StyleRun` é a unidade de rich text — pré-requisito do syntax highlighting.
Implementação inicial: `CosmicTextBackend` (mapeia `StyleRun` → `Attrs` spans do
cosmic-text). Migração futura para Parley = nova impl do trait, zero mudança acima.

### 3.2 Scene v2 (primitivas instanciadas)

```rust
pub enum Primitive {
    Quad(QuadInstance),        // cor sólida | gradiente linear/radial, borda, radius — 1 shader
    Shadow(ShadowInstance),    // sombra analítica (Evan Wallace), sem render pass de blur
    Path(PathInstance),        // lyon tessellation (ícones SVG, formas livres)
    Underline(UnderlineInstance),
    MonoSprite(SpriteInstance),    // glyphs alpha-only (colorização no shader)
    PolySprite(SpriteInstance),    // imagens RGBA, emoji
}

pub struct Scene {
    // arrays separados por tipo; BatchIterator percorre em ordem de pintura e
    // emite ranges contíguos do mesmo tipo+textura = 1 draw call instanciado
}
```

Cada primitiva carrega `order: DrawOrder` (stacking context) e
`content_mask: Rect` (clip — interseção da stack de clips, aplicada como
scissor por batch ou discard no shader).

### 3.3 Actions

```rust
pub trait Action: 'static { fn name(&self) -> &'static str; /* namespace::Nome */ }
// macro actions!(editor, [MoveLeft, SelectAll, …]);
// dispatch sobe pela árvore de foco; bindings mais profundos vencem
```

### 3.4 Measure function no layout

```rust
pub type MeasureFn = Box<dyn FnMut(Size<Option<f32>>, Size<AvailableSpace>, &mut MeasureCtx) -> Size<f32>>;
// LayoutEngine: taffy.disable_rounding() + arredondamento próprio a device pixel
// (ceil_to_device_pixel) mantendo origens absolutas não-arredondadas — evita
// gaps de 1px com scale factor fracionário (receita GPUI, crates/gpui/src/taffy.rs)
```

### 3.5 Convenções de integração

- **Donos de diretório** (mudança fora do seu diretório = PR separado com
  review do dono):

| Diretório | Dono |
|---|---|
| `src/text`, `src/layout` | WS-A |
| `src/gpu`, `src/compositor` → `src/scene`, `shaders/` | WS-B |
| `crates/editor_core`, `src/editor` | WS-C |
| `src/input`, `src/actions` (novo), `src/dispatch.rs` | WS-D |
| `src/ui`, `src/theme`, `src/overlay`, `src/animation` | WS-E |
| `crates/git_backend`, `crates/basic-ide` | WS-G |
| `src/accessibility`, `crates/plev_kittest` | WS-H |

- **Feature flags**: código novo entra atrás de Cargo feature desligada
  (`scene-v2`, `editor`, `actions`) → merge cedo e frequente, `main` sempre verde.
- **PRs ≤ ~400 linhas**, rebase contínuo sobre `main`.
- **Gate de CI**: `cargo test --workspace` + (após WS-H) snapshot tests headless.

---

## 4. Workstreams

### WS-A · `ws/text-engine` — Texto

**Objetivo:** eliminar toda heurística de texto; medição, hit-testing e rich
text reais.

1. **Measure function real**
   - Implementar `TextBackend::measure` consultando o `TextSystem`
     (cosmic-text `Buffer` + `shape_until_scroll` com largura disponível).
   - Registrar como `MeasureFn` no `LayoutEngine` (taffy leaf nodes).
   - Remover `chars * font_size * 0.6` de `src/builder/layout_pipeline.rs:59`.
   - Cache de medição por `(texto, estilo, largura-bucket)`.
2. **Hit-testing real de cursor**
   - `TextBackend::hit_test` via `cosmic_text::Buffer::hit(x, y)`.
   - Deletar `src/text_input/cursor_map.rs` (ratio 0.6 hardcoded).
3. **Rich text por spans**
   - `StyleRun` → `Attrs` por range no `Buffer` (cosmic-text já suporta).
   - Estender `TextNodeKey` (`src/text/cache.rs`) com hash dos runs.
4. **Subpixel positioning**
   - Posições fracionárias de glyph; variantes por offset subpixel no atlas
     (começar com 4 variantes em X; GPUI usa 4×4).
   - Chave do glyph cache passa a incluir o bucket de subpixel.
5. **DPI de ponta a ponta**
   - `scale_factor` propagado: layout em pontos lógicos, raster em device px
     (`atlas.rs:53` hoje usa scale 1.0 fixo).
6. **Correções de robustez**
   - `src/text/atlas.rs:184` — falha de alocação no atlas é silenciosa:
     logar + fallback (glyph tofu) + métrica.
   - `src/text/system.rs:190` — remover `unwrap` na remoção do shaping cache.

**Entrega:** texto proporcional mede/clica corretamente; spans coloridos na
mesma linha; nítido em retina.

### WS-B · `ws/renderer-v2` — Renderer

**Objetivo:** scene API de primitivas instanciadas, render sob demanda,
frame pacing. Referência direta: `gpui_wgpu` (shaders.wgsl, Apache-2.0).

1. **Primitivas + shaders**
   - `quad.wgsl` unificado: cor sólida, gradiente linear/radial, borda,
     corner radius por canto (SDF), tudo em 1 shader instanciado.
   - `shadow.wgsl`: blur Gaussiano **analítico** (aproximação Evan
     Wallace/Figma — `erf` no eixo X + 4 amostras no Y). Aposentar o blur
     2-pass para sombras de UI; manter o blur real só como `LayerEffect`
     explícito (backdrop blur).
   - `sprite_mono.wgsl` / `sprite_poly.wgsl`: glyphs e imagens.
   - Vertex shader gera bounding box (2 triângulos) por instância; fragment
     avalia SDF/sample.
2. **Batching**
   - `BatchIterator` sobre os arrays da `Scene` em ordem de pintura
     (`DrawOrder` via bounds tree); quebra de batch apenas em mudança de
     tipo de primitiva ou de textura de atlas.
3. **Imagens**
   - `Primitive::PolySprite` + atlas de imagens (etagere, mesmo esquema do
     atlas de glyphs); decodificação via crate `image`; suporte a ícones SVG
     pela rota `Path` (lyon, já existente).
4. **Clipping aninhado**
   - Stack de clip-rects no builder (`content_mask` = interseção); aplicado
     por scissor quando o batch inteiro compartilha mask, senão discard no
     shader (mask como parâmetro de instância).
5. **Render sob demanda + culling**
   - `WindowInvalidator`: `dirty: bool` + `dirty_views: FxHashSet<ViewId>`;
     `request_redraw()` somente quando invalidado (signals/input/animations).
   - Culling por viewport antes do upload (primitiva fora da janela não
     entra na Scene).
6. **Frame pacing**
   - Pool de instance buffers com triple buffering; nunca bloquear a CPU
     esperando a GPU (lição zed.dev/blog/120fps).
7. **Instrumentação**
   - Contadores expostos: draw calls, instâncias por tipo, tempo de
     resolve/encode/present, memória de atlas. Feature `inspector` futura.
8. **Configurabilidade**
   - MSAA (hoje 4x hardcoded em `src/gpu/pipelines.rs:56`), present mode
     (hoje `AutoVsync` em `src/gpu/context.rs:86`), tolerance de tessellation
     (hoje 0.1 fixo) — todos parametrizáveis.
9. **Dívidas**
   - Sort de layers a cada resolve mesmo já ordenado (`src/compositor/mod.rs`).

**Entrega:** mesma cena visual com ~⅓ dos draw calls, gradientes/imagens,
clipping correto em listas, CPU ociosa quando a UI está parada.

### WS-C · `ws/editor-core` — Editor

**Objetivo:** widget Editor multi-line, multi-cursor, com undo — o coração do
IDE. Modelo de referência: Helix (`Document = Rope + Selections + Syntax +
History`).

**Fase 1 — crate `editor_core` (sem dependência de UI/GPU):**

1. Buffer **ropey** (`Rope`); snapshots baratos por clone.
2. `Transaction` = lista de edits `(range, replacement)` componível;
   `History` com undo/redo agrupando por coalescência temporal/semântica.
3. Seleções: `Vec<Selection { anchor: usize, head: usize }>` — multi-cursor
   desde o primeiro commit; operações: add-above/below, split-by-lines,
   select-all-matches.
4. Movimento: char (boundary UTF-8), palavra (`unicode-segmentation`),
   linha (home/end inteligente: primeiro não-whitespace ↔ coluna 0), página.
5. Mapeamento de transações sobre seleções (edit em A desloca cursor B).
6. Testes de propriedade (proptest): aplicar/desfazer transações aleatórias
   preserva o invariante `undo(apply(t, doc)) == doc`.

**Fase 2 — widget `plev::editor` (depende de WS-A.1–2):**

7. **Virtualização de linhas**: shaping apenas das linhas visíveis ±margem;
   cache de `Shaped` por linha invalidado por `Transaction` (line-granular).
8. Scroll por linhas + por pixels (suave), sticky horizontal.
9. Mouse: posicionar cursor (hit_test), drag-select, double-click (palavra),
   triple-click (linha), alt+click (adicionar cursor).
10. Clipboard via **arboard**: copy/cut/paste; paste multi-cursor
    (n trechos → n cursores).
11. **IME completo**: preedit renderizado inline com sublinhado
    (hoje ignorado — `src/text_input/component.rs:166`); janela de candidatos
    posicionada via `Window::set_ime_cursor_area`.
12. Gutter: números de linha; slots para diagnostics (WS-F) e git hunks (WS-G).
13. Cursor blink respeitando configuração (hoje 0.53s fixo,
    `src/text_input/component.rs:8`).
14. `TextInput` single-line existente vira wrapper fino do Editor
    (1 linha, sem gutter).

**Entrega:** abrir/editar/salvar arquivo real de 100k linhas com scroll
fluido, multi-cursor e undo.

### WS-D · `ws/actions-keymap` — Actions, keymap, palette

**Objetivo:** todo input vira Action despachada; atalhos declarativos com
contexto (modelo Zed).

1. **Actions tipadas**: macro `actions!(ns, [Nome, …])`; registry global;
   evolução do `src/dispatch.rs` (queue type-erased existente).
2. **Keymap JSON**:
   ```json
   [
     { "context": "Editor && mode == insert",
       "bindings": { "cmd-shift-p": "palette::Toggle", "cmd-k cmd-s": "zed::OpenKeymap" } }
   ]
   ```
   - Predicates: `&&`, `||`, `!`, `()`, `==`, `>` (ancestral na árvore de foco).
   - Dispatch sobe do elemento focado; bindings mais profundos vencem;
     definições posteriores sobrescrevem (user > defaults); `null` desabilita.
   - Multi-stroke (`cmd-k cmd-s`) com timeout.
3. **Árvore de foco real**: key contexts por elemento; tab order; focus trap
   em modal (hoje inexistente — `src/overlay/mod.rs` não bloqueia input).
4. **Fuzzy matching com `nucleo`** (o matcher do Helix/Zed).
5. **Command palette** (`cmd-shift-p`): enumera o registry de actions com os
   bindings atuais; **file finder** (`cmd-p`) sobre walker do workspace.
6. Persistência: `keymap.json` do usuário com hot-reload (notify já é dep).

**Entrega:** zero `match` de teclas hardcoded nos apps
(hoje: `crates/basic-ide/src/main.rs:110-131`).

### WS-E · `ws/design-system` — UI

**Objetivo:** biblioteca de widgets bonita e consistente. Receita validada
pelo gpui-component (11.7k★): design language **shadcn/ui** + ícones
**Lucide** + tokens de tema + tamanhos `xs/sm/md/lg`.

1. **Widgets retained em `plev::ui`** (estado próprio, não draw-functions
   com flags booleanas):
   - Promover do basicIDE: Button, Modal, Tabs, Checkbox, ContextMenu,
     Separator, Badge, Avatar.
   - Novos: Input (sobre WS-C.14), Select, Tooltip, Toast/Notification,
     Switch, Slider, Progress, Tree, Breadcrumb.
   - **List e Table virtualizadas** — extrai o padrão
     header+lista+scrollbar reimplementado 3× no basicIDE
     (`unassigned_view.rs`, `multi_stack_view.rs`, `diff_view.rs`,
     ~1.100 linhas duplicadas).
2. **Ícones Lucide**: pipeline SVG → `Path` (lyon) com cache de tessellation;
   fallback Codicons (já embarcado) para ícones de IDE.
3. **Scrollbars visíveis**: desenhar thumb (`ScrollState::thumb_ratio()` já
   calcula), hit-test, drag, auto-hide; scroll cinético com o `SpringScroll`
   existente.
4. **Animações de overlay**: fade/scale em modal/menu/toast usando física por
   `Intent` (já implementada em `src/theme/intent.rs` — manter, é original e boa).
5. **Tema único polido** (não perseguir look nativo): tokens existentes +
   dark/light + paletas já presentes (catppuccin, dracula, tokyo-night);
   revisar contraste (WCAG AA).
6. **App showcase** `crates/showcase`: galeria de todos os widgets em todos
   os estados/temas (referência: `cargo run --example dock` do gpui-component).
   Serve de base para os snapshot tests do WS-H.

**Entrega:** showcase com 20+ widgets consistentes; basicIDE consumindo
exclusivamente `plev::ui`.

### WS-F · `ws/intelligence` — Tree-sitter + LSP

**Objetivo:** highlight incremental e inteligência de linguagem no Editor.

1. **Syntax highlighting**
   - Avaliar **tree-house** (crate do Helix, bindings+highlighter com
     integração ropey nativa) vs bindings oficiais `tree-sitter` 0.26.
     Critério: integração com `editor_core::Rope` e robustez de injections.
   - Parsing incremental alimentado pelas `Transaction` (edits → `InputEdit`).
   - Saída: `Vec<StyleRun>` por linha → WS-A.3. Temas de highlight mapeados
     aos tokens do tema (WS-E.5).
   - Grammars iniciais: rust, toml, json, markdown, wgsl.
2. **Runtime async**
   - tokio (current-thread) + ponte `EventLoopProxy<UserEvent>` → winit.
     Toda a UI permanece síncrona; async só para processos externos.
3. **Cliente LSP**
   - JSON-RPC sobre stdio (`lsp-types`); ciclo: initialize →
     didOpen/didChange (sincronização incremental via Transactions) →
     publishDiagnostics.
   - Features na ordem: diagnostics (gutter + sublinhado ondulado via
     `Primitive::Underline`), completion (popup no OverlayManager),
     hover, goto-definition.
   - rust-analyzer como servidor de validação.

**Entrega:** abrir um .rs com highlight correto, diagnostics do
rust-analyzer e completions funcionais.

### WS-G · `ws/git-real` — Git + basicIDE real

**Objetivo:** substituir 100% dos mocks do basicIDE por dados reais.

1. **Crate `git_backend`** (puro, sem UI): backend **gix**
   (caminho Helix; alternativa avaliada: git CLI + parsing, caminho Zed).
   API: `status() → Vec<FileStatus>`, `log(branch) → Vec<Commit>`,
   `diff(path|commit) → Vec<Hunk>`, `stage/unstage/discard(path)`,
   `commit(msg)`, `branches()`.
   Operações em thread separada (gix é sync) com resultados via canal →
   `EventLoopProxy`.
2. **Substituições no basicIDE**:
   - `mock_files()` (`unassigned_view.rs:300-383`) → `status()` real.
   - `mock_stacks()` (`multi_stack_view.rs:316-371`) → `log()`/`branches()`.
   - `generate_diff_for_file()` (`diff_view.rs:218-316`) → `diff()` real
     com syntax highlight (WS-F) nas linhas.
3. **Ações reais**: stage/unstage/discard (modal de confirmação existente
   para destrutivas), ignore (escreve `.gitignore`), commit real pelo form
   (que passa a usar o Editor single-line+ do WS-C).
4. **Watchers**: `.git` e working tree via notify (já é dependência) →
   refresh automático com debounce.
5. **File tree real**: `std::fs` + lazy-load de diretórios no widget Tree
   (WS-E); abas Branches/History da sidebar ganham conteúdo.
6. Limpeza: remover `serde/serde_json` não usados do Cargo.toml; remover
   `#![allow(dead_code)]` global (`main.rs:11`); desacoplar
   `pending_discard_idx` (`workspace/mod.rs:65` + `overlays.rs`) em um enum
   `PendingAction` único.

**Entrega:** basicIDE aponta para qualquer repositório e funciona de verdade.

### WS-H · `ws/a11y-testing` — Acessibilidade + harness

**Objetivo:** AccessKit funcional + testes dirigidos por acessibilidade
(diferencial: quase todo o ecossistema Rust reprova nisso).

1. **AccessKit real**
   - Árvore gerada automaticamente a partir da árvore de elementos
     (role, label, bounds, estado) — hoje é manual e o
     `PlevActionHandler::do_action()` é stub (`src/window/mod.rs:195-202`).
   - Implementar actions: focus, click, scroll, set_value (inputs).
   - Labels: derivados do conteúdo de texto; API `.label("…")` para override.
2. **Harness `plev_kittest`** (padrão egui_kittest/rerun):
   - Dirigir a UI pela árvore AccessKit: `get_by_name`, `get_by_role`,
     `click()`, `type_text()`, frame a frame, sem janela.
   - Screenshots via wgpu headless + snapshot tests
     (`UPDATE_SNAPSHOTS=1` para regenerar).
   - Migrar os 9 testes de overlay do basicIDE para o harness.
3. **Gate de CI**: snapshots do showcase (WS-E.6) em dark/light rodando em
   todos os PRs.

**Entrega:** screen reader navega o basicIDE; regressões visuais bloqueiam PR.

### WS-I · `ws/dsl-tooling` — narrate como produto *(opcional, adiável)*

Lição Makepad ("load-bearing DSL" sem docs matou a adoção) vs Slint (LSP +
live-preview é o diferencial):

1. Hot-reload interpretando `when`/`each` com dados estáticos
   (hoje `src/narrate_runtime/mod.rs:7-10` pula tudo exceto `show` literal).
2. LSP da narrate: completions de elementos/modificadores e diagnostics
   reaproveitando o parser e o `suggest.rs` existentes.
3. Modelo Slint: **interpretada em dev, compilada em release** (a macro já
   compila; falta o interpretador cobrir a gramática toda).
4. Documentação da gramática (referência por modificador, exemplos).

---

## 5. Ordem de integração (dependências, não datas)

```
M0  WS-0 contratos
     └─► WS-A.1-2 (medição+hit-test)  +  WS-B.5 (render sob demanda)
M1  WS-A completo  +  WS-D completo
     └─► desbloqueiam WS-C fase 2 e WS-F
M2  WS-B completo  +  WS-E completo        (UI linda + renderer rápido)
M3  WS-C completo  +  WS-G completo        (editor + git reais)
M4  WS-F completo  +  WS-H completo        (inteligência + a11y/testes)
─── pós-M4 (backlog): WS-I · terminal embutido (fork alacritty_terminal) ·
    multi-window · vim mode (vira trivial sobre WS-D) · multibuffer ·
    splits/panes · settings UI · hot-patch de Rust (nível Dioxus subsecond)
```

Grafo de dependência entre workstreams:

```
WS-0 ──┬─► WS-A ──┬─► WS-C(fase2) ──► WS-F
       │          └─► WS-F(spans)
       ├─► WS-B ──► WS-E(ícones/imagens)
       ├─► WS-D ──┬─► WS-G(ações) ─► (WS-G core é independente)
       │          └─► WS-H(focus tree)
       └─► WS-C(fase1, independente)
```

---

## 6. Decisões técnicas registradas (ADR resumido)

| # | Decisão | Alternativa rejeitada | Razão |
|---|---|---|---|
| 1 | Manter cosmic-text atrás de `TextBackend` | Migrar já para Parley | cosmic-text 0.19 já usa HarfRust (shaping equivalente); migração fica barata se necessária; spans suficientes p/ highlight |
| 2 | ropey para o buffer | SumTree próprio (Zed) | novelty points; ropey é maduro (Helix) e suficiente; trocar depois é interno ao `editor_core` |
| 3 | Sombra analítica (Evan Wallace) | blur 2-pass atual p/ sombras | elimina 2 render passes + texturas temporárias por sombra |
| 4 | Render sob demanda, sem damage rects parciais | damage tracking por pixel | GPUI prova que janela inteira <8.33ms basta; damage parcial = complexidade alta, ganho marginal |
| 5 | gix para git | git2 / CLI parsing | Rust puro, sem openssl vendorizado (dor do Lapce); CLI (Zed) fica como plano B |
| 6 | tokio current-thread só para I/O externo | async na UI inteira | UI síncrona é mais simples; lição Xi (async total = complexidade não paga) |
| 7 | Tema próprio único | temas nativos por SO | Slint deprecou nativos em 2026 ("uncanny valley", custo 5×) |
| 8 | nucleo para fuzzy | fzf-like próprio / skim | usado por Helix e Zed; API de injector incremental |
| 9 | Instanced rendering, 1 shader/primitiva | megashader único / vello compute | receita GPUI comprovada; vello_hybrid ainda beta — reavaliar pós-M4 para paths |
| 10 | Keymap JSON com predicates de contexto | atalhos hardcoded / TOML plano | modelo Zed validado; pré-requisito de vim mode e customização |

---

## 7. Referências

- GPUI rendering: <https://zed.dev/blog/videogame> · frame pacing:
  <https://zed.dev/blog/120fps> · ownership/effect queue:
  <https://zed.dev/blog/gpui-ownership> · backend wgpu (Apache-2.0):
  `zed/crates/gpui_wgpu` (shaders.wgsl, cosmic_text_system.rs, taffy.rs)
- gpui-component (widgets/design system, Apache-2.0):
  <https://github.com/longbridge/gpui-component>
- Keymap Zed: <https://zed.dev/docs/key-bindings>
- Helix architecture (Document/Rope/History):
  <https://github.com/helix-editor/helix/blob/master/docs/architecture.md> ·
  tree-house: <https://github.com/helix-editor/tree-house>
- Xi retrospective (anti-padrões): <https://raphlinus.github.io/xi/2020/06/27/xi-retrospective.html>
- Parley/rota de texto: <https://github.com/linebender/parley> ·
  vello_hybrid: <https://github.com/linebender/vello/tree/main/sparse_strips>
- Slint — DSL dev/release híbrida: <https://slint.dev/blog/slint-1.13-released> ·
  deprecação de temas nativos: <https://slint.dev/blog/default-native-style-change>
- iced 0.14 reactive rendering:
  <https://github.com/iced-rs/iced/releases/tag/0.14.0> · COSMIC Epoch 2:
  <https://system76.com/blog/post/cosmic-epoch-2-and-3-roadmap>
- egui_kittest (testes via AccessKit): <https://crates.io/crates/egui_kittest>
- Survey a11y/IME 2025: <https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html>
- Sombras analíticas (Evan Wallace):
  <https://madebyevan.com/shaders/fast-rounded-rectangle-shadows/>
