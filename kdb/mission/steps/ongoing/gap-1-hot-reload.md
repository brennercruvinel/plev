---
project: phi
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-22
domain: task-tracking
---

# gap-1: hot reload system

data: 2026-03-22
status: em andamento
branch: task/gap-1-hot-reload

## objetivo

implementar hot reload para o phi em 3 tiers:
- tier 1: shader hot reload (vello pattern), prioridade
- tier 2: DSL hot reload (makepad pattern, phi_narrate!), complexo
- tier 3: rust code hot reload (subsecond), pesquisa

## design

ver `mission/knowledge/hot-reload-design.md` para pesquisa completa (7 implementacoes estudadas).

## checklist, tier 1 (shader hot reload)

- [x] adicionar notify + notify-debouncer-full ao cargo.toml (feature-gated)
- [x] criar src/hot_reload.rs com shaderwatcher
- [x] implementar shader_source() dual-mode (disco vs include_str!)
- [x] extrair pipeline recreation functions em gpu.rs e effects.rs
- [x] integrar watcher no event loop (about_to_wait)
- [x] graceful degradation: WGSL invalido = log::error, nao crash
- [x] guard: nao compilar watcher em WASM (single-threaded)
- [x] testes unitarios (watcher, debounce, error handling)
- [x] teste manual (editar shader, ver mudanca sem restart)
- [x] documentar em rules.md, knowledge/, claude.md

## checklist, tier 2 (DSL hot reload)

- [x] runtime parser: src/narrate_runtime.rs (tokenizer + recursive descent parser -> element)
- [x] overridemap: hashmap<(file, line), DSL text> com re-parse on access
- [x] narratewatcher: file watcher para src/ e examples/ (.rs files)
- [x] extract_narrate_blocks(): extrai phi_narrate! blocks de source text
- [x] narrate_resolve(): proc-macro wraps output com override check
- [x] codegen modificado: ::phi::narrate_resolve(file!(), line!(), || { ... })
- [x] corrigido plev_narrate -> phi_narrate em codegen e error messages
- [x] integrar narrate watcher no event loop (about_to_wait)
- [x] 27 testes unitarios (tokenizer, parser, block extraction, overrides)
- [x] 343 testes totais passando (312 core + 4 shader + 27 narrate)
- [x] compila sem hot-reload feature (zero overhead path)
- [ ] preservar estado do componente across reload (deferred, requer component id system)
- [ ] teste manual end-to-end (editar .rs com phi_narrate!, ver mudanca)

## checklist, tier 3 (subsecond)

- [ ] research: compatibilidade com wgpu/winit event loop
- [ ] research: bevy PR #19309 analise
- [ ] decisao: go / no-go / defer

## dependencias

| crate | versao | feature | tier |
|-------|--------|---------|------|
| notify | 7 | hot-reload | 1, 2 |
| notify-debouncer-full | 0.4 | hot-reload | 1, 2 |

## arquivos criados/modificados

### tier 1
- src/hot_reload.rs (shaderwatcher, shader_source)
- src/gpu.rs (pipeline recreation methods)
- src/effects.rs (pipeline recreation methods)
- cargo.toml (dependencias)

### tier 2
- src/narrate_runtime.rs (novo, ~600 LOC: tokenizer + parser + extract + tests)
- src/hot_reload.rs (narratewatcher, narrateoverrides, narrate_override, process_narrate_file)
- src/lib.rs (narrate_runtime mod + narrate_resolve fn)
- src/window.rs (narrate_watcher field + check_narrate_reload)
- crates/phi_narrate_macro/src/codegen.rs (wrap com narrate_resolve, fix plev->phi)
- crates/phi_narrate_macro/src/parse/block_item.rs (fix plev->phi)
