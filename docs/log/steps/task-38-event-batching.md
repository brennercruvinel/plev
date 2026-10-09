---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2023-11-09
domain: task-tracking
---

# task-38: event batching, P1, done

## objetivo
batch-drain de eventos winit para reduzir trabalho GPU durante input rapido. 5-10x reducao.

## justificativa
cada evento winit dispara scene rebuild + GPU submit. em touch 120hz+ ou keyboard burst, isso e o(n) renders por frame. batch-drain acumula eventos e processa todos antes de um unico render.

## dependencias
- task-09 (input), concluida

## referencia
- pattern d1 em `mission/knowledge/extracted-patterns.md`
- yazi source: `bunker/repos/devtools/tui-apps/yazi/yazi-fm/src/app/app.rs`

## estimativa
~50-80 LOC

## checklist
- [x] em `window_event()`: acumular eventos em `Vec<InputEvent>` ao inves de processar imediatamente
- [x] em `about_to_wait()`: drenar todos os eventos acumulados, processar, depois um unico `compositor.resolve()` + render_pass
- [x] limite de batch: max 50 eventos por iteracao (evitar starvation)
- [x] testes: verificar que multiplos key events no mesmo frame produzem resultado correto
- [x] benchmark informal: contar render passes por segundo com input rapido (antes vs depois)

## criterios de aceite
1. multiplos eventos no mesmo frame = um unico render pass
2. input nao perde eventos (todos processados, so render que e batched)
3. zero regressao visual ou funcional
4. latencia de input nao aumenta perceptivelmente (max 1 frame de delay)
