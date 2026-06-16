---
project: plev
audience: [ai-agents, contributors]
status: in-progress
last-updated: 2026-03-22
domain: task-tracking
---

# gap-1: hot reload, changelog

## 2026-03-22

### iniciado
- branch task/gap-1-hot-reload criada
- task file criado em ongoing/
- pesquisa de campo completa: 7 implementacoes estudadas (hot-reload-design.md)
- decisao: 3 tiers (shader / DSL / subsecond)
- dylib descartado (typeid UB com actionqueue)

### tier 1 implementado
- `src/hot_reload.rs` (novo): shaderwatcher (notify + debounce 500ms), shader_source() dual-mode, compile_error WASM guard
- `src/gpu.rs`: 4 pipeline creation methods extraidos (create_quad/rect_sdf/text/composite_pipeline), reload_shader() com push_error_scope/guard.pop()
- `src/effects.rs`: 3 pipeline creation methods extraidos (create_blur/shadow/effect_composite_pipeline), reload_shader() com graceful degradation
- `src/window.rs`: shader_watcher field em app, check_shader_reload() poll em about_to_wait()
- `src/lib.rs`: `#[cfg(feature = "hot-reload")] pub mod hot_reload`
- cargo.toml: deps ja estavam declaradas (notify 7, notify-debouncer-full 0.4)
- 4 testes unitarios hot_reload (shader_dir_exists, shader_source_loads_all, poll_changes_empty, fallback_unknown)
- 312 testes existentes continuam passando
- documentacao: rules.md (secao hot reload), claude.md (comando), knowledge/index.md (ja existia)
- wgpu 28 API: push_error_scope retorna errorscopeguard, pop() via guard.pop() (nao device.pop_error_scope)

### tier 2 implementado
- `src/narrate_runtime.rs` (novo, ~600 LOC):
  - tokenizer hand-written (ident, str, int, float, braces, comma, pipe)
  - recursive descent parser: elements, modifiers (30+), body blocks, show
  - on/when/each/bind blocks consumed sem interpretacao (require rust evaluation)
  - extract_narrate_blocks(): encontra plev_narrate!/plev_narrate! em source text
  - 27 testes (tokenizer, parser elements/modifiers/nesting, skipped blocks, extraction)
- `src/hot_reload.rs` (extendido):
  - narrateoverrides: global hashmap<(file,line), DSL text> com lazylock<mutex>
  - narrate_override(): re-parse on access (microseconds, evita element clone)
  - narratewatcher: notify debounce 500ms para .rs files em src/ + examples/
  - process_narrate_file(): extrai blocks de .rs e atualiza override map
  - 4 novos testes (override empty, update+check, replace, dirs exist)
- `src/lib.rs`:
  - `pub mod narrate_runtime` (feature-gated)
  - `pub fn narrate_resolve()`: dual cfg, hot-reload checks override map, release inlines to compiled()
- `src/window.rs`:
  - narrate_watcher field em app
  - check_narrate_reload() poll em about_to_wait()
- `crates/plev_narrate_macro/src/codegen.rs`:
  - wrap output: `::plev::narrate_resolve(file!(), line!(), || { ... })`
  - fix: plev_narrate -> plev_narrate em error messages e use statement
- `crates/plev_narrate_macro/src/parse/block_item.rs`: fix plev_narrate -> plev_narrate
- 343 testes totais passando, zero regressao
- compilacao sem hot-reload feature ok (zero overhead)

#### decisoes de design tier 2
- re-parse on access (nao cache element): element nao e clone (contem box<dyn fnmut>)
- narrate_resolve() no plev crate (nao plev_narrate): evita circular dependency
- runtime parser hand-written (nao syn): DSL simples, evita dependencia syn em runtime
- on/when/each/bind skipped: contem rust code, nao interpretavel sem eval
- preservacao de estado across reload deferred: requer component id system

#### armadilhas tier 2
- edition 2024: `ref` em patterns e implicito quando matching em references
- `file!()` retorna path relativo ao crate root, watcher converte absolute para relativo via strip_prefix
- token cloning em parser: ok para dev-only feature
- div sem dimensoes explicitas (w/h) gera 0 scene nodes (sem visual), testes devem usar `w X h Y`

### sessao 2026-03-22 (validacao e testes)

#### fix compilacao
- `crates/plev_narrate/tests/integration.rs`: renomeado `plev_narrate` -> `plev_narrate` (import + 12 invocacoes)
- 12 integration tests agora compilam e passam

#### path matching validado
- `src/hot_reload.rs`: 2 testes novos
  - `test_path_matching_file_macro_vs_watcher`: prova que `strip_prefix(project_root())` == `file!()`
  - `test_path_matching_roundtrip_override`: simula fluxo watcher->store->lookup, confirma override encontrado

#### skip behavior documentado com assercoes
- `src/narrate_runtime.rs`: testes existentes de skip (on/when/each/bind) fortalecidos com assercoes reais
  - antes: so `let _ = render(&el);` (verificava que nao crasha)
  - agora: verificam contagem de nodes, presenca de bg rect, e que parent modifiers sobrevivem
- 2 testes novos:
  - `parse_skip_preserves_siblings_before_and_after`: prova que text antes e depois de `on click` skipped ambos sobrevivem
  - `parse_skip_mixed_on_when_each_preserves_static_content`: 3 blocos dinamicos skipped consecutivos, header e footer preservados

#### teste e2e pipeline completo
- `e2e_extract_store_lookup_with_dynamic_blocks`: simula arquivo .rs com plev_narrate! contendo on/when/each
  - extract_narrate_blocks() extrai 1 bloco com line number correto
  - update_narrate_overrides() armazena no override map
  - narrate_override() encontra o override e retorna element funcional
  - element renderiza com texto estatico preservado (dashboard, footer)

#### contagem final
- 450 passed, 0 failed, 3 ignored (pre-existentes em plev-clinic)
- delta vs baseline: +18 testes (12 integration + 3 hot_reload + 2 skip + 1 e2e)
