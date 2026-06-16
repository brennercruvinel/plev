---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# task-04 changelog

## 2026-03-08
- iniciada implementação do signal system (reatividade)
- branch: `task/TASK-04-signal-system` a partir de master (e6f1c7e)
- adicionado `slotmap = "1.1"` ao cargo.toml
- criado `src/signal.rs` (~550 linhas) com runtime reativo completo:
  - `create_signal`, `create_effect`, `create_memo`, `batch`, `dispose_node`
  - push-pull hybrid: set() pushes dirty/check, effects pull via sources_changed
  - thread-local runtime com slotmap<nodeid, reactivenode>
  - closures armazenadas como rc<dyn fn> para evitar reentrance no refcell
  - borrow do refcell liberado antes de executar qualquer closure de usuário
  - memo com comparefn type-erased para cortar propagação (diamond problem)
  - detecção de ciclos via flag `running` em notify_subscribers
- adicionado `pub mod signal;` em `src/lib.rs`
- 15 testes unitários passando (zero dependência de GPU)
- criado `examples/signal_counter.rs` demonstrando signals + views + compositor
