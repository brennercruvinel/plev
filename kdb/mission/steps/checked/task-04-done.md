---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: task-tracking
---

# task-04: signal system (reatividade), done

## objetivo
implementar primitivas reativas: `create_signal`, `create_effect`, `create_memo`. signals são a base para re-render granular, quando um signal muda, apenas as views que dependem dele re-renderizam.

## checklist de conclusão
- [x] `create_signal<T>(initial: T) -> (ReadSignal<T>, WriteSignal<T>)`
- [x] `create_effect(f: impl Fn())`, re-executa quando signals lidos dentro de `f` mudam
- [x] `create_memo(f: impl Fn() -> T) -> ReadSignal<T>`, computed value cacheado
- [x] tracking automático de dependências (quem lê qual signal)
- [x] batching de updates (múltiplos set_signal num frame = um re-render)
- [x] exemplo funcional: counter reativo com signal (`examples/signal_counter.rs`)
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo test` passa com 15 testes de reatividade (19 total)
- [x] zero alocações em steady state (signal não mudou = zero work)

## concluída em: 2026-03-08
## branch: `task/TASK-04-signal-system`
## commits: f676b7d, 188fb7e
