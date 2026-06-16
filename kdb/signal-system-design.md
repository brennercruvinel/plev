---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-08
domain: signals
---

# signal system design

## data: 2026-03-08

## modelo: push-pull hybrid
- `signal.set()` **push**: marca subscribers diretos como dirty, transitivos como check
- effects **pull**: verificam sources antes de re-executar
- memos comparam valor novo vs velho - se igual, cortam propagação

## storage: slotmap<nodeid, reactivenode>
- nodeid é 8 bytes, copy, generational (previne use-after-free)
- ~96 bytes por node (value + kind + sources + subscribers + state + running)

## borrow safety (crítico)
o runtime vive num `thread_local! { RefCell<ReactiveRuntime> }`. a regra absoluta:

**nunca segurar o borrow do refcell durante execução de closures de usuário.**

closures de efeitos e memos são armazenadas como `Rc<dyn Fn>` (não `Box<dyn Fn>`) para
poderem ser clonadas para fora do borrow. o pattern é:

1. `with_runtime(|rt| { ... extrair Rc::clone da closure, preparar estado ... })` -> borrow liberado
2. `closure()` -> executa código do usuário, que pode chamar signal.get()/set()
3. `with_runtime(|rt| { ... cleanup: pop observer, set Clean ... })` -> borrow liberado

se usar `Box<dyn Fn>` + segurar o borrow, qualquer `signal.get()` dentro da closure
causa `RefCell already borrowed` panic.

## detecção de ciclos
feita em `notify_subscribers`: se um subscriber tem `running == true`, é porque sua closure
está em execução e está tentando escrever em um signal que é sua fonte -> panic.

não funciona checar `running` dentro de `execute_effect` porque `notify_subscribers` ignora
effects já em estado dirty (não os re-adiciona a pending_effects).

## memo comparison (type erasure)
memos armazenam um `CompareFn = Rc<dyn Fn(&dyn Any, &dyn Any) -> bool>` que captura o tipo `T`
via closure no momento da criação. isso permite comparar box<dyn any> sem conhecer o tipo concreto.

## trabalho paralelo
com 17+ agentes claude code compartilhando o mesmo diretório, `git checkout` e `git stash`
causam race conditions constantes. worktrees (`git worktree add /tmp/...`) são a única forma
segura de trabalhar. o worktree `/tmp/plev-task04` foi usado para esta task.
