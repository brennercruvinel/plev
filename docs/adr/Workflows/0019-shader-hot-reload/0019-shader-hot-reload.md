---
id: adr-0019
title: "adotamos shader hot reload via notify file watcher sobre dylib reloading"
project: plev
type: adr
status: accepted
date: 2024-02-06
date_basis: reconstructed
recorded: 2026-03-22
topics: [hot-reload, gpu]
relations: [adr-0031, architecture-hot-reload-design, task-56, learning-hot-reload]
source: docs/adr/workflows/0019-shader-hot-reload.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# adotamos shader hot reload via notify file watcher sobre dylib reloading

Decides shader hot reload through a file watcher that recreates the pipeline, following vello's pattern, and not through dylib reloading, which breaks with `Box<dyn Any + Send>` in the action queue because the typeid changes between loads, which is silent UB. The result is shader iteration on WGSL in about 500 ms, and invalid WGSL keeps the old pipeline. A note of 2026-09-22 reverses the polling in `about_to_wait`, which the 2024 decision had chosen.

> 2026-09-22: adr-3 (channel polling in about_to_wait) is reversed: the watcher now wakes the loop with `Window::request_redraw` and the poll sits in `present_frame`, and the watcher moved from `window::App` (which no app uses) into `GpuContext`. see [hot-reload-on-by-default-in-gpu-context](../0031-hot-reload-on-by-default-in-gpu-context/0031-hot-reload-on-by-default-in-gpu-context.md).

brenner@plev.engineer

## contexto

o plev engine embarcava shaders via `include_str!()` em compile-time. qualquer mudanca visual em shaders exigia recompilacao completa (~7s no m4). para iteracao rapida de efeitos visuais, precisavamos de live editing. avaliamos 7 implementacoes de hot reload em projetos rust (vello, rerun, makepad, leptos, dioxus, hot-lib-reloader, subsecond).

## decisao

adotamos o pattern do vello (file watcher + pipeline recreation) como tier 1, descartando dylib reloading (hot-lib-reloader) por incompatibilidade com `Box<dyn Any + Send>` no actionqueue (typeid muda entre loads = ub silencioso).

## consequencias

ganhamos: iteracao instantanea em shaders WGSL (500ms debounce), graceful degradation (WGSL invalido preserva pipeline antigo), zero overhead em release (feature-gated).

perdemos: nao recarrega codigo rust (apenas shaders). tier 2 (DSL) e tier 3 (subsecond) ficam para iteracoes futuras.

fica mais dificil: nada significativo. o refactor extraiu pipeline creation em metodos reutilizaveis, o que melhora a manutenibilidade.

---

## adr-2: adotamos errorscopeguard sobre panic recovery para validacao de shaders

### contexto

wgpu 28 mudou a API de error scopes. `push_error_scope()` agora retorna `ErrorScopeGuard` em vez de `()`. a documentacao e exemplos antigos (incluindo vello) usam a API pre-28. tentamos `device.pop_error_scope()` que nao existe mais.

### decisao

usar `let guard = device.push_error_scope(filter)` seguido de `pollster::block_on(guard.pop())` para capturar erros de compilacao WGSL sem panic.

### consequencias

ganhamos: validacao sincrona de shaders em native (pollster ja e dependencia). WGSL invalido retorna `Some(Error)` com mensagem detalhada incluindo linha e coluna do erro.

perdemos: nada. o pattern e mais limpo que a API antiga.

armadilha para evitar: nunca usar `device.pop_error_scope()` em wgpu 28+. nao existe. o metodo vive no `ErrorScopeGuard`.

---

## adr-3: adotamos channel polling sobre eventloopproxy para integracao do watcher

### contexto

o watcher roda em background thread (notify). precisavamos sinalizar o event loop sobre mudancas. duas opcoes: (a) `EventLoopProxy::send_event(AppEvent::ShaderChanged)` ou (b) `mpsc::channel` polled em `about_to_wait()`.

### decisao

channel polling (b). o `EventLoopProxy` no plev e restrito a `#[cfg(any(wasm32, android))]` e estender para native exigiria refatorar `App::new()` em todas as plataformas.

### consequencias

ganhamos: zero mudanca nos construtores existentes. `try_recv()` e zero-cost quando vazio (hot path sem mudancas = nenhum overhead).

perdemos: polling vs push. em teoria, push seria mais imediato, mas com debounce de 500ms a latencia e irrelevante.

---

## patterns que funcionaram

1. **extrair pipeline creation antes de adicionar reload.** o refactor de `GpuContext::new()` (440 LOC monolitico) em 4 metodos `create_*_pipeline()` foi pre-requisito. sem isso, o reload teria que duplicar 200+ LOC de pipeline config.

2. **dual-mode via `#[cfg]` no call site, nao no metodo.** os metodos `create_*_pipeline()` recebem `shader_source: &str`. o caller decide se vem do disco ou de `include_str!()`. isso mantem os metodos agnosticos a feature flags.

3. **testar backend antes de integrar no event loop.** `cargo check --features hot-reload` em cada passo garantiu que erros de compilacao foram pegos cedo (ex: API wgpu 28 mudou).

4. **`composite.wgsl` e dual-owner.** o caller (`check_shader_reload` em window.rs) chama `gpu.reload_shader()` e `effect_processor.reload_shader()` para cada arquivo mudado. sem mapeamento especial de "este shader vai para dois lugares" - o dispatch e bruto mas correto.

## armadilhas para evitar

1. **`device.pop_error_scope()` nao existe em wgpu 28.** usar `guard.pop()` do `ErrorScopeGuard` retornado por `push_error_scope()`.

2. **notify debouncer manda paths duplicados.** o mesmo path pode aparecer 2-3x no batch. o reload e idempotente (recriar pipeline com mesmo source = ok), entao nao e problema funcional, mas pode gerar log noise.

3. **`env!("CARGO_MANIFEST_DIR")` so funciona via `cargo run`.** se rodar o binario direto (sem cargo), o path compilado aponta para o diretorio de build original. aceitavel porque hot-reload e dev-only.

4. **shaders `include_str!()` em release continuam funcionando.** o feature flag `hot-reload` nao esta no `default`. release builds nunca incluem notify/watcher. verificar com `cargo check --workspace --examples` (sem features) apos qualquer mudanca.

---

## arquivos tocados nesta sessao

| arquivo | LOC delta | descricao |
|---------|-----------|-----------|
| `src/hot_reload.rs` | +148 | shaderwatcher, shader_source(), testes |
| `src/gpu.rs` | +508/-316 | pipeline extraction + reload_shader() |
| `src/effects.rs` | +201/-140 | pipeline extraction + reload_shader() |
| `src/window.rs` | +58 | watcher integration |
| `src/lib.rs` | +4 | mod declaration |
| `mission/rules.md` | +9 | hot reload section |
| [`hot-reload-design.md`](../../../Research/Workflows/hot-reload-design.md) | +123 | design doc (pesquisa) |
| `mission/steps/ongoing/GAP-1-hot-reload.md` | +59 | task checklist |
| [`task-56-hot-reload.md`](../../../Tasks/task-56-hot-reload/README.md) | +22 | changelog |
| `CLAUDE.md` | +3 | comando hot-reload |

## learnings

- [hot reload session: what worked and what bit us](../0031-hot-reload-on-by-default-in-gpu-context/hot-reload.md)
