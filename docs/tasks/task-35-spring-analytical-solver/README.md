---
id: task-35
title: "task-35: fix spring<t>, analytical solver"
project: plev
type: task
status: done
date: 2023-07-23
date_basis: reconstructed
recorded: 2026-03-13
topics: [animation, determinism]
relations: [task-27, research-extracted-patterns]
source: [docs/log/steps/task-35-spring-analytical-solver.md, docs/log/changes/task-35.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-35: fix spring<t>, analytical solver

Records the replacement of the Euler integration in `Spring<T>` with an analytical solution of the damped oscillator, covering the three regimes: underdamped, critically damped and overdamped. The motivation is a correctness bug: with Euler, the result depended on the frame rate and diverged on stiff springs, whereas the analytical solution is stable for any time step. The current code implements this in `Spring::tick` (`rust/engine/src/compose/animation/spring.rs:99`), and the test `spring_frame_rate_independence` compares 30, 60 and 120 fps with a tolerance of 0.5; the idea comes from pattern c1 of [extracted-patterns.md](../../research/extracted-patterns.md).

## objetivo
substituir o forward euler integration do spring<t> por solver analitico com coeficientes pre-computados. corrige bug de corretude: resultados diferentes a 30fps vs 60fps.

## justificativa
bug de corretude. o euler solver diverge com springs rigidos ou dt grande. o solver analitico (pattern c1 de natura) e incondicionalmente estavel para qualquer dt e produz o resultado matematico exato independente de frame-rate.

## dependencias
- task-27 (animation system), concluida

## referencia
- pattern c1 em [`extracted-patterns.md`](../../research/extracted-patterns.md)
- natura source: `bunker/repos/web-frontend/animation/natura/natura/src/spring.rs` l73-271

## estimativa
~100 LOC

## checklist
- [x] pre-computar 4 coeficientes (`pos_pos_coef`, `pos_vel_coef`, `vel_pos_coef`, `vel_vel_coef`) no construtor de spring<t>
- [x] tratar 3 regimes: sub-amortecido (exp*sin/cos), criticamente amortecido (exp*polinomio), super-amortecido (duas exponenciais)
- [x] substituir `tick()` por multiply-add com coeficientes
- [x] adicionar `damping_ratio()` convenience method
- [x] manter API existente (`stiffness/damping/mass`)
- [x] re-computar coeficientes quando `set_target()` e chamado (se parametros mudam)
- [x] testes: comparar resultado a 30fps vs 60fps (devem ser identicos com solver analitico)
- [x] testes: spring convergencia com stiffness alta (que causaria divergencia euler)
- [x] verificar que testes existentes passam

## criterios de aceite
1. spring produz resultado identico a 30fps e 60fps (dentro de epsilon f32)
2. spring com stiffness=10000 nao diverge
3. zero regressao nos 35 testes de animacao
4. API publica nao muda (backward compatible)

## Delivery record

### summary
replaced forward euler integration in spring<t> with an analytical solver using pre-computed coefficients. handles all three damping regimes: under-damped (exp*sin/cos), critically damped (exp*polynomial), and over-damped (two exponentials).

### changes
- pre-computed 4 coefficients (pos_pos_coef, pos_vel_coef, vel_pos_coef, vel_vel_coef) in spring constructor
- replaced tick() with coefficient multiply-add (frame-rate independent)
- added damping_ratio() convenience method
- coefficients re-computed on set_target() when parameters change
- tests confirm identical results at 30fps vs 60fps and stability with high stiffness (10000+)
- all existing animation tests pass (zero regression)

### status
done
