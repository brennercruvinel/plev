# plev — Status do Projeto

> Estado vivo do trabalho. Atualizado a cada onda. Complementa
> `docs/PLANO_TECNICO.md` (a arquitetura-alvo, que não muda).

## Resumo

- **Branch:** `main` · **Testes:** 980 passando / 0 falhando · **Build:** limpo
- **Desde o commit base:** ~85 commits, 183 arquivos, +31k/−2.3k linhas
- **Apps que rodam:** `showcase` (galeria de widgets), `basicIDE` (git client
  real), `editor_demo`, `visual_demo`

## Linguagem visual estabelecida (HOFF)

Reproduzimos o design da referência `hoff-research-social` (medido do site
vivo, não do CSS estático):

| Token | Valor | Como foi obtido |
|---|---|---|
| Fonte UI | **Rubik** 400/500/600/700 | A ref usa Rubik (`next/font`); embarcada (OFL) |
| Fundo de página | **grafite #303030** | Medido ao vivo: `rgba(#282828,.7)` compõe #303030 (o `#444444` do `body` nunca aparece) |
| Sidebar/colunas | #2E2E2E | `rgba(#282828,.8)` |
| Card/surface | #343434 (lift sutil sobre o fundo) | post card = `rgba(248,248,248,.02)` |
| Texto | branco .95 / corpo .76 / meta .50 | `variables.sass` |
| Popover/menu | #3B3B3B, radius 32 | medido |
| Glass real | backdrop-blur só em pílulas/search/menu | a ref **não** põe frost nos cards de conteúdo |

## O que foi entregue (por onda)

### Fundação (Ondas 1–2)
- **Engine de texto:** medição real via cosmic-text (fim do `chars*0.6`),
  hit-test de cursor, `TextBackend`, **letter-spacing e line-heights exatos**,
  centralização por métricas reais
- **Renderer:** render sob demanda, culling, `RenderConfig`, `RenderStats`,
  **ordem de desenho por push**, **BackdropBlur por região** (glass de verdade),
  sombra analítica + inset, gradientes, imagens, clipping aninhado
- **`crates/editor_core`:** rope (ropey), transactions, undo/redo, multi-cursor,
  movimento por graphemes — 100% testável, proptests
- **Widget Editor:** virtualização, multi-cursor visual, clipboard, IME inline
- **Actions/keymap:** `Action` tipada, parser de keystrokes, predicates de
  contexto estilo Zed, `KeymapMatcher` multi-stroke
- **Design system:** ~15 widgets + widget `Card` (6 variantes), ícones Lucide,
  scrollbars, animações de overlay, `crates/showcase`
- **`crates/git_backend`:** status/log/diff/stage/commit via gix+CLI; basicIDE
  sem mocks, watcher, dados reais

### Correções de raiz (esta sessão)
1. **Tipografia quebrada** → Inter 500/600/700 ausentes faziam fallback para
   fontes do sistema (+35% advance). Embarcados todos os pesos; depois trocado
   o default para **Rubik** (a fonte da ref)
2. **Scroll/reatividade mortos** → eventos que mudavam visual retornavam `false`
   (tela congelava no render-sob-demanda) + clipping não escalava em HiDPI
3. **Crash ao abrir merge commit** → parser de diff não entendia *combined diff*
   (`@@@`); agora suporta, +2 testes de regressão
4. **"Cinza claro sem contraste" (a queixa central)** → **bug de gamma sRGB**:
   cores sRGB do tema iam direto à surface sRGB como se fossem lineares,
   clareando tudo ~2,5× (fundo medido 118 quando o token é 48). Corrigido:
   `srgb_to_linear` nos shaders + `to_linear_array` nos clear colors. Fundo
   medido **118 → 50**. Validado por medição de pixel, não no olho

## Decisões registradas nesta sessão

- **Fundo = grafite #303030, não preto** — o `GOLDEN_SPEC.md` inicial dizia
  "#0E0E0E preto" (erro de olho); a medição ao vivo provou grafite. Os tokens
  seguem o medido (`src/theme/hoff.rs`, teste `hoff_page_is_measured_graphite_not_black`)
- **Captura de janela** — as janelas abrem no monitor secundário (y~1130);
  capturar por bounds via `osascript`, nunca `screencapture` da tela toda
- **Validação visual por pixel** — script Swift mede o pixel; "parece igual"
  não basta

## Branches NÃO mescladas (e por quê)

| Branch | Estado | Decisão |
|---|---|---|
| `ws/parity-ide` (2 commits) | usa `#121212` preto | **Descartar** — conflita com `ide-parity` (#303030 medido); o verificador reprovou |
| `ws/fix-fidelity` (3 commits) | tentativa de cor interrompida no meio | **Reaproveitar medições se útil** — incompleta; o gamma fix tornou parte dela obsoleta |

## O que falta (backlog priorizado)

### Visual / fidelidade
- [ ] **Propagar `to_linear_array`** aos clear colors de `examples/visual_demo`
      e `editor_demo` (ainda sRGB direto — aparecem mais claros)
- [ ] **Auditar gamma em casos restantes** — confirmar que image atlas (sRGB) e
      backdrop compõem certo após o fix; medir todas as seções do showcase
- [ ] **Decisão de design:** manter grafite #303030 (fiel à ref) ou ir mais
      escuro #1A1A1A se o usuário preferir mais contraste — **aguarda o usuário**
- [ ] Afinar estados (hover/active/focus) de todos os widgets contra a ref
- [ ] Subpixel text + DPI de ponta a ponta (WS-A.4-5) — texto nítido em retina

### Funcional (plano original, ainda pendente)
- [ ] **Editor embutido no basicIDE** — clicar num arquivo abre o EditorView
      para editar (hoje só mostra diff)
- [ ] **Syntax highlight** (tree-sitter / tree-house) → `StyleRun` coloridos
- [ ] **Cliente LSP** (diagnostics, completion, hover) — exige runtime async
- [ ] **Command palette + fuzzy finder** (nucleo) — cmd-p / cmd-shift-p
- [ ] **File tree real** no basicIDE (widget Tree + fs)
- [ ] **Acessibilidade real** (AccessKit) + harness de testes por snapshot

### Estabilidade / dívida
- [ ] Crash sob automação intensa (não reproduz em uso normal; investigar)
- [ ] Captura de janela do basicIDE intermitente (timing do System Events)
- [ ] Limpar branches já mescladas e worktrees temporários
- [ ] `#![allow(dead_code)]` global no basicIDE/snakeGame

### Horizonte (plano §5, pós-fundação)
- [ ] Terminal embutido (alacritty_terminal) · multi-window · vim mode ·
      splits/panes · settings UI · LSP da DSL narrate
