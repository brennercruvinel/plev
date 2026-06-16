---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-30
domain: lint
commit: 6b141ed..813a4e5
---

# adotamos phierror como tipo de erro unificado e eliminamos debt de lint

## contexto

o projeto phi v0.3 operava sem tipo de erro próprio. código de produção dependia de
tipos concretos de dependências externas (`notify::Error`, `wgpu::Error`) e de
`.unwrap()` extensivo em paths WASM. paralelamente, 98 atributos `#[allow(dead_code)]`
acumulados em `examples-wip/` e no parser DSL degradavam a relação sinal/ruído do
compilador, impedindo detecção de dead code real em produção.

uma auditoria diagnóstica revelou 8 itens classificados como warn/fail. investigação
empírica reclassificou 3 deles: todos os `panic!()` reportados em `builder/mod.rs`,
`input/mod.rs` e `component.rs` estavam exclusivamente em blocos `#[test]`, não em
código de produção. o `let _ = tx.send()` em `hot_reload.rs` ocorre apenas durante
shutdown do receiver.

## decisões

### adr-1: adotamos phierror sem dependência de thiserror

**decisão.** criamos `src/error.rs` com `enum PhiError` (variantes: `Window`, `Gpu`,
`Wasm`, `Watcher`) implementando `Display`, `Error` e `From` manualmente, sem crate
externo.

**alternativas descartadas.**
- `thiserror v2`: elimina boilerplate, mas adiciona proc-macro dependency para 4
  variantes. proporção custo/benefício não justifica para enum pequeno.
- `anyhow`: perde pattern matching nos callsites. apropriado para aplicações, não para
  engine/biblioteca.
- `miette`: rich diagnostics com source spans. excelente para `phi-clinic` (crate de
  diagnóstico), desproporcional para o core.

**consequências.**
- ganho: propagação de erros via `?` em todo o engine; base para refatorar `.unwrap()`
  remanescentes; zero dependência nova.
- perda: boilerplate manual de display/from (mitigado pela simplicidade do enum).
- risco: se o enum crescer além de ~8 variantes, reconsiderar `thiserror`.

### adr-2: extraímos setup WASM para função com error propagation

**decisão.** os 8 `.unwrap()` encadeados em `window.rs:650-664` foram substituídos
por `setup_wasm_canvas() -> PhiResult<()>`, com `.ok_or()` para cada step web-sys e
fallback 800x600 para dimensões de viewport.

**consequências.**
- ganho: WASM deploy não crasha silenciosamente em contextos restritos (iframe, headless,
  web worker). mensagens de erro claras no console.
- perda: `log::error!` no callsite em vez de panic; se o canvas falhar, a app continua
  sem renderização. para produção, pode ser necessário adicionar um panic explícito ou
  fallback visual.
- o `create_window().expect("Phi: failed to create window")` permanece como panic
  intencional: sem janela, o engine não pode operar.

### adr-3: consolidamos dead_code allows em nível de módulo/arquivo

**decisão.** 83 `#[allow(dead_code)]` individuais em `examples-wip/` substituídos por
7 `#![allow(dead_code)]` (um por arquivo). 7 allows no parser DSL consolidados em 3
`#[allow(dead_code)]` nas declarações `mod` em `parse/mod.rs`.

**consequências.**
- ganho: signal-to-noise ratio do compilador aumenta significativamente. dead code real
  em `src/` agora aparece sem ruído.
- nota: `examples-wip/` está no `.gitignore`, mudanças são locais. os arquivos
  eventualmente migrarão para `examples/` com `[[example]]` em `Cargo.toml`, momento
  em que `#![allow(dead_code)]` funcionará naturalmente como crate-level attribute.

## armadilhas encontradas

### severidade incorreta no diagnóstico inicial

o scan de código reportou `panic!("Expected Rect/Text/Event")` como fail em produção.
verificação empírica revelou que **todos os 22 panics estavam em blocos `#[test]`**.
a lição: análise estática por grep não distingue `#[cfg(test)]` de produção.

**regra para agentes futuros:** antes de classificar `panic!()` como bug de produção,
verificar se o callsite está dentro de `#[cfg(test)]` ou `#[test]`.

### xcode toolchain incompatibility (arm64 vs arm64e)

`cargo check` falha na máquina de desenvolvimento por incompatibilidade de arquitetura
em `libxcrun.dylib` (`have 'arm64', need 'arm64e'`). problema pré-existente, não
relacionado às mudanças. impede validação local, CI remoto é necessário.

### hot_reload `let _ =` é pattern legítimo

o `let _ = tx.send(paths)` em `hot_reload.rs` é idiomático rust para "canal pode estar
fechado, não me importo". a correção aplicada (`log::warn!`) é a abordagem mínima
correta: não panic (seria destrutivo em shutdown), não retry (canal fechado é
terminal), apenas observabilidade.

## relação com regras existentes

- **rule-13 (error handling tipado):** `PhiError` implementa parcialmente esta regra
  para o engine core. a regra descreve `AppError` para apps; `PhiError` é o análogo
  para a camada de engine. a próxima etapa é migrar `Result<_, notify::Error>` em
  `hot_reload.rs` para `PhiResult<_>` e eliminar `.unwrap()` remanescentes em
  `gpu.rs` e `effects.rs`.

## arquivos modificados nesta sessão

| arquivo | natureza da mudança |
|---------|---------------------|
| `src/error.rs` | novo: phierror enum, display, error, from impls |
| `src/lib.rs` | +`pub mod error` |
| `src/window.rs` | setup_wasm_canvas() helper, .expect(), comments em unused_mut |
| `src/hot_reload.rs` | log::warn! substituindo let _ = |
| `src/showcase_scene.rs` | remoção de #[allow(non_snake_case)] desnecessário |
| `src/builder/mod.rs` | 11 test panics -> let-else com {:?} |
| `src/input/mod.rs` | 7 test panics -> let-else com {:?} |
| `src/component.rs` | 4 test panics -> let-else com {:?} |
| `crates/phi_narrate_macro/src/parse/mod.rs` | consolidação de dead_code allows |
| `crates/phi_narrate_macro/src/parse/block_item.rs` | remoção de allows individuais |
| `crates/phi_narrate_macro/src/parse/element.rs` | remoção de allow individual |
| `crates/phi_narrate_macro/src/parse/modifier.rs` | remoção de allow individual |
| `examples-wip/` (7 arquivos) | 83 allows -> 7 #![allow(dead_code)] (gitignored) |
