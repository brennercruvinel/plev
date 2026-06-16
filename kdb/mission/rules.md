---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-13
domain: project-rules
---

# plev, regras técnicas

## arquitetura
- backend sempre antes de UI
- testes sempre antes de mover task para checked
- scenenode é a unidade de compositing, nenhum consumer do plev toca no compositor diretamente
- renderer tem dois targets de compilação, não dois renderers

## performance
- fxhashmap para qualquer cache em hot path, nunca hashmap padrão (siphash é 2-3x mais lento para chaves de inteiros)
- gpuvec cresce, nunca encolhe, re-alocar buffer gera fragmentação e latência
- shaping via harfbuzz é caro, cache keyed por (text, font_size_bits, line_height_bits, max_width_bits), invalidar só quando conteúdo ou tamanho muda
- dirty tracking via fxhasher per layer, no steady state: layer limpa = zero render pass, zero geometry, zero shaping
- premultiplied alpha em todo o pipeline, shaders outputam `rgb*a, a`, blend state `One/OneMinusSrcAlpha`

## armadilhas conhecidas
- atlas de glifos pode fragmentar em textos multilíngues longos, LRU mitiga, mas tuning necessário para fallbacks bidirecionais
- immediate mode por frame sem dirty tracking gera re-submissões desnecessárias à GPU, sempre verificar hash antes de re-upload
- wgpu no WASM é mais lento em compute shaders que nativo, evitar compute shaders em path crítico por enquanto
- glyphon tem issues em shaping complexo (árabe, devanagari), testar com textos multilíngues antes de declarar suporte
- WASM GPU init é async e não pode setar self diretamente, usar eventloopproxy pattern (ver knowledge/wasm-webgpu-validation.md)
- trunk requer `data-target-name` no index.html quando há bin + lib com mesmo package name
- binário é `plev-app`, não `plev`, evita colisão de artifact WASM com a lib
- `Limits::downlevel_webgl2_defaults()` não é para webgpu, usar `Limits::default()` para webgpu
- `main.rs` precisa de `#[cfg(not(target_arch = "wasm32"))]` guard (env_logger não existe no WASM)
- android emulador swiftshader trava em `create_render_pipeline()`, deve usar `hw.gpu.mode=host` no config.ini
- xcrun no macos pode falhar com arm64/arm64e mismatch, usar `DEVELOPER_DIR=/Library/Developer/CommandLineTools SDKROOT=...MacOSX.sdk`
- escalar font_size durante animacao causa re-shaping a cada frame, manter font_size fixo, animar posicao/opacity
- `gen` e keyword reservada em rust edition 2024, nao usar como identificador

## animacao
- pixel-snap (`v.round()`) em todas as coordenadas durante animacao, evita shimmer sub-pixel
- font size fixo durante animacao, escalar font_size muda textnodekey hash, causa re-shaping jitter a cada frame
- usar web_time::instant (nao std::time::instant), panic em wasm32
- snap threshold: 0.5px para posicoes, 0.005 para valores normalizados (opacity)
- spring<t> usa solver analitico (task-35), coeficientes pre-computados, 3 regimes (sub/critico/super amortecido), frame-rate independent. incondicionalmente estavel para qualquer dt e stiffness alto.
- keyframesequence disponivel para animacoes multi-step (fade-in -> hold -> slide-out), task-37
- tween suporta repeat/reverse/delay via with_repeat()/with_reverse()/with_delay(), task-37
- animationstate<s, t> disponivel para blend-on-transition entre estados, task-37

## editable text
- textinput desacoplado do imestate, nao acoplar diretamente, usar `handle_ime()` como ponte
- cursor blink: 530ms toggle, resetar ao digitar
- cursor_to_x/x_to_cursor: aproximacao `font_size * 0.6` por char (monospace assumption ate cosmic-text cursor API)

## convenções de código
- módulos: gpu.rs, gpu_vec.rs, text.rs, compositor.rs, window.rs, view.rs, animation.rs, text_input.rs, dispatch.rs, overlay.rs, lib.rs, main.rs
- shaders em shaders/ como arquivos .wgsl separados (quad.wgsl, text.wgsl, rect_sdf.wgsl, composite.wgsl, blur.wgsl, shadow.wgsl)
- examples em examples/, 20 examples incluindo animation_demo, text_input_demo, todo_app, message_dock
- binário nativo: `cargo run --bin plev-app` (renomeado para evitar colisão com lib WASM)
- WASM: `trunk serve` (usa lib.rs com wasm_main via wasm_bindgen(start))

## trabalho paralelo (múltiplos agentes/devs)
- múltiplos agentes claude code e desenvolvedores humanos trabalham simultaneamente neste projeto
- cada task = uma branch: `task/TASK-XX-nome-curto` criada a partir de `master`
- nunca commitar direto na `master`, todo trabalho na branch da task
- antes de criar branch, verificar `git branch -a` para não duplicar
- **usar git worktree** quando outro agente está modificando o mesmo working directory
- ao concluir task: PR para `master` ou avisar usuário para merge
- se precisar alterar arquivo que outra task também altera: registrar no changelog e avisar usuário
- conflitos de merge são responsabilidade de quem faz merge para `master`, resolver antes de mergear

## adaptação do plano
- o plano evolui com o desenvolvimento, tasks, checklist, dependências e arquitetura podem mudar conforme descobertas surgem durante implementação e testes
- ao descobrir que algo planejado está errado ou incompleto: corrigir a task imediatamente, não esperar
- ao descobrir nova dependência entre tasks: atualizar ambas as tasks e registrar em knowledge/
- ao mudar decisão arquitetural: registrar em knowledge/ com justificativa, atualizar este arquivo, e revisar tasks pendentes afetadas
- tasks descartadas vão para checked/ com nota de descarte, nunca deletar, o histórico importa

## integridade de informação
- nunca escrever código ou documentação baseado em suposição sobre apis, comportamentos de plataforma ou convenções
- se não souber: pesquisar na documentação oficial (docs.rs, github, web), ler código-fonte da dependência, verificar issues/changelogs
- se após pesquisar ainda houver dúvida: registrar em knowledge/ e perguntar ao usuário
- apis de wgpu, cosmic-text, winit mudam entre versões, sempre confirmar contra a versão exata no cargo.toml antes de usar
- nunca chutar nome de método, flag ou comportamento, o custo de pesquisar é minutos, o custo de código errado é horas

## acessibilidade
- todo elemento interativo deve ter semantic role para accessibility (accesskit, task-30)
- screen readers sao requisito de producao, nao feature opcional
- accesskit usa lazy activation (zero cost sem screen reader), pattern ak1
- per-frame treeupdate acumulado via fxhashmap durante build_scene, pattern ak2
- viewid(u32) -> nodeid(u64) via cast direto, root em nodeid(0), pattern ak4
- focus routing: action::focus -> inputstate.focused_view + inputevent::focus sintetico, pattern ak5
- WASM: null adapter compile-time, zero overhead, pattern ak6
- auto-navigation de layout: computedbounds -> focusgraph com vizinhos direcionais, pattern d8

## vector paths
- shapes vetoriais passam por tessellation (lyon), nao gerar triangulos manualmente
- fill only primeiro; stroke/caps/joins em iteracao futura
- **lyon reusar quad pipeline existente** via fillvertexconstructor<quadvertex>, nao criar shader path.wgsl separado (pattern a5/a6)
- tessellate uma vez, armazenar index ranges, draw por muitos frames, integra com dirty tracking existente

## tempo cross-platform
- usar web-time (nao std::time::instant) para tempo cross-platform, wraps performance.now() no WASM
- std::time::instant causa panic em wasm32 (ver armadilhas conhecidas)

## avaliacao de dependencias
- antes de adicionar qualquer dependencia: consultar mission/knowledge/refs/integration-candidates.md
- categorias: adopt (usar agora) / evaluate (testar) / watch (acompanhar) / hold (nao usar)

## signal system (corrigido task-36)
- subscribers usam fxindexset, o(1) insert/remove/contains (era vec o(n), pattern f1)
- observer stack usa RAII `ObserverGuard`, panic-safe, impossivel esquecer pop (pattern f4)
- `ReadSignal::peek()` disponivel, leitura untracked para logging/debug (pattern f2)
- constant-signal sentinel (pattern f3 de slint), skipped, overhead minimo nao justificava complexidade

## event processing (implementado task-38)
- bufferedevent enum + batch-drain: acumular em window_event(), processar todos em about_to_wait(), um unico render por frame
- 5-10x reducao de trabalho GPU durante input rapido (touch 120hz+)

## scene caching
- scene cache per-component ja implementado em `component.rs`: `cached_nodes: Option<Vec<SceneNode>>`, `needs_render: bool`, `invalidate()`, `state_mut()` seta needs_render automaticamente
- a cache e do `Component<L>`, nao do compositor, o dirty tracking do compositor opera no nivel de layer (fxhasher)
- para reutilizar nodes cacheados: nao chamar `state_mut()` (usa `state()` readonly) e nao chamar `invalidate()`
- proximo passo (sem task criada): b7 memoizacao via partialeq para pular render() quando props nao mudam, b8 dirty flag bubbling per-component com merge_up

## patterns de referencia
- documento master: `mission/knowledge/extracted-patterns.md`, 38 patterns de 17 repos
- consultar antes de implementar task-30 (a11y), task-31 (lyon), melhorias em animation.rs ou signal.rs

## action dispatch (task-42)
- `ActionQueue` vive no core plev (`src/dispatch.rs`), infra generica, nao especifica de app
- `WidgetAction` trait = `Any + Send + 'static`, send para futuro multi-thread
- `emit<A>(source, action)` + `drain_typed<A>()`, typed no site de uso, erased internamente
- filhos emitem, parents drenam, fluxo unidirecional, sem event bus global
- `drain_typed` retorna itens do tipo pedido, devolve o resto ao vec interno (o(n) por drain)

## overlay system (task-42)
- `OverlayManager` vive no core plev (`src/overlay.rs`), pure data, sem GPU refs
- z-order base 1000 (constante `BASE_Z`), overlays sempre acima do conteudo principal
- `push()` aceita w/h = 0.0 para bounds desconhecidos; `set_bounds()` apos primeiro render
- `hit_test_outside()` ignora overlays com zero bounds (nao conta como hit)
- `pop_id()` reassigna z-orders para manter contiguidade
- renderizacao: consumer itera `stack` e cria scenenodes em layers com z_order do overlay
- dismiss: click-outside via `hit_test_outside()`, escape via `pop()`

## hot reload (gap-1 tier 1)
- feature flag `hot-reload`: `cargo run --bin plev-app --features hot-reload`
- shaders: file watcher em `shaders/*.wgsl` via notify + debounce 500ms
- WGSL invalido = log::error, pipeline antigo preservado (graceful degradation via push_error_scope/guard.pop())
- WASM: nao suportado (compile_error guard em hot_reload.rs)
- sem feature flag: shaders embutidos via include_str!(), zero overhead
- `composite.wgsl` e usado por gpucontext e effectprocessor, reload atualiza ambos
- pipeline creation extraido em metodos reutilizaveis (create_*_pipeline) para reload e init

## regras que o claude deve seguir
- antes de qualquer implementação: ler esta lista inteira
- ao descobrir nova armadilha: adicionar aqui imediatamente
- ao mudar decisão arquitetural: registrar em knowledge/ com justificativa
